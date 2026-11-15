//! JNI bridge for the Android app: session manager, per-session reader
//! thread, and `native*` exports used by `TerminalNative.kt`.
//!
//! Lock discipline: the reader thread works on the raw master fd/pid and
//! never holds the session mutex while polling (holding it across `poll()`
//! starved concurrent writers for seconds — found by the acceptance test).
//! Writes to the pty copy the fd under the mutex and write without it
//! (write_all can block on EAGAIN backpressure; holding the mutex across
//! that froze concurrent keystrokes). `nativePoll` sleeps on the session
//! condvar so output wakes the UI within microseconds instead of up to one
//! 20ms sleep quantum later.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use jni::objects::{GlobalRef, JByteArray, JClass, JObject, JObjectArray, JString, JValue};
use jni::JNIEnv;
use jni_sys::{jboolean, jbyteArray, jint, jlong, jstring};

use crate::pty::PtyHandle;
use crate::term::{EventKind, Terminal};

pub const KEY_ENTER: i32 = 1;
pub const KEY_BACKSPACE: i32 = 2;
pub const KEY_TAB: i32 = 3;
pub const KEY_ESCAPE: i32 = 4;
pub const KEY_HOME: i32 = 5;
pub const KEY_END: i32 = 6;
pub const KEY_PAGE_UP: i32 = 7;
pub const KEY_PAGE_DOWN: i32 = 8;
pub const KEY_INSERT: i32 = 9;
pub const KEY_DELETE: i32 = 10;
pub const KEY_ARROW_UP: i32 = 11;
pub const KEY_ARROW_DOWN: i32 = 12;
pub const KEY_ARROW_LEFT: i32 = 13;
pub const KEY_ARROW_RIGHT: i32 = 14;
pub const KEY_FN_BASE: i32 = 20; // F(n) = KEY_FN_BASE + n - 1

static MANAGER: OnceLock<Mutex<HashMap<i64, Arc<Session>>>> = OnceLock::new();
static NEXT_ID: AtomicI64 = AtomicI64::new(1);
static JVM: OnceLock<jni::JavaVM> = OnceLock::new();

fn manager() -> &'static Mutex<HashMap<i64, Arc<Session>>> {
    MANAGER.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct Session {
    pub id: i64,
    pub pty: Mutex<Option<PtyHandle>>,
    pub term: Mutex<Terminal>,
    /// Signalled by the reader thread after every feed; `nativePoll` sleeps
    /// on this instead of a sleep-poll loop, so keystroke echo and program
    /// output reach the UI at output speed rather than one quantum later.
    pub seq_cv: Condvar,
    pub listener: Mutex<Option<GlobalRef>>,
    pub exit_code: Mutex<Option<i32>>,
    pub reader: Mutex<Option<JoinHandle<()>>>,
}

impl Session {
    fn is_dead(&self) -> bool {
        self.exit_code.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    /// Fire a listener callback; all failures are swallowed (UI may be gone).
    fn notify(&self, what: Notify) {
        let listener = match self.listener.lock() {
            Ok(g) => g.clone(),
            Err(_) => return,
        };
        let listener = match listener {
            Some(l) => l,
            None => return,
        };
        let vm = match JVM.get() {
            Some(v) => v,
            None => return,
        };
        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return,
        };
        let obj = listener.as_obj();
        let res = (|| -> Result<(), jni::errors::Error> {
            match what {
                Notify::Title(t) => {
                    let s = env.new_string(&t)?;
                    env.call_method(
                        obj,
                        "onTitle",
                        "(Ljava/lang/String;)V",
                        &[JValue::Object(&s)],
                    )?;
                }
                Notify::Bell => {
                    env.call_method(obj, "onBell", "()V", &[])?;
                }
                Notify::Exit(code) => {
                    env.call_method(obj, "onExit", "(I)V", &[JValue::Int(code)])?;
                }
                Notify::Cwd(c) => {
                    let s = env.new_string(&c)?;
                    env.call_method(obj, "onCwd", "(Ljava/lang/String;)V", &[JValue::Object(&s)])?;
                }
            }
            Ok(())
        })();
        let _ = res;
    }
}

enum Notify {
    Title(String),
    Bell,
    Exit(i32),
    Cwd(String),
}

/// Reader thread: pty → parser → snapshot seq; responses → pty.
///
/// Concurrency note: this thread works on the raw master fd/pid captured up
/// front and does NOT hold the session pty mutex while polling — holding it
/// across `poll(100ms)` starved concurrent writers for seconds (observed in
/// the acceptance test). The mutex is only taken for short, bounded ops.
fn reader_loop(session: Arc<Session>) {
    let mut buf = vec![0u8; 16384];
    let (master_fd, child_pid) = {
        let guard = match session.pty.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match guard.as_ref() {
            Some(h) => (h.master, h.pid),
            None => return,
        }
    };
    loop {
        if session.is_dead() {
            break;
        }
        // 1) poll for readability on the raw fd (no locks held).
        let readable = crate::pty::wait_readable_fd(master_fd, 100);
        let mut got = 0usize;
        if readable {
            got = crate::pty::read_fd(master_fd, &mut buf);
        }
        if got > 0 {
            let (responses, events) = {
                let term = match session.term.lock() {
                    Ok(mut t) => {
                        t.feed(&buf[..got]);
                        (t.take_responses(), t.take_events())
                    }
                    Err(_) => return,
                };
                term
            };
            // Wake poll waiters immediately; they re-check seq under the
            // term mutex, so this cannot be a lost wakeup.
            session.seq_cv.notify_all();
            if !responses.is_empty() {
                crate::pty::write_all_fd(master_fd, &responses);
            }
            for ev in events {
                match ev {
                    EventKind::Title => {
                        let title = session
                            .term
                            .lock()
                            .map(|t| t.title().to_string())
                            .unwrap_or_default();
                        session.notify(Notify::Title(title));
                    }
                    EventKind::Bell => session.notify(Notify::Bell),
                    EventKind::ClipboardSet => { /* clipboard handled via OSC 52 query */ }
                    EventKind::CwdChanged => {
                        let cwd = session.term.lock().ok().and_then(|t| t.cwd.clone());
                        if let Some(c) = cwd {
                            session.notify(Notify::Cwd(c));
                        }
                    }
                }
            }
        }
        // 2) exit detection + reaping (waitpid directly on the pid).
        let exited = crate::pty::try_wait_pid(child_pid);
        if let Some(code) = exited {
            // Drain whatever is still buffered before declaring death.
            let mut drained = 0;
            loop {
                let n = crate::pty::read_fd(master_fd, &mut buf);
                if n == 0 {
                    break;
                }
                let resp = {
                    let mut term = match session.term.lock() {
                        Ok(t) => t,
                        Err(_) => break,
                    };
                    term.feed(&buf[..n]);
                    term.take_responses()
                };
                crate::pty::write_all_fd(master_fd, &resp);
                drained += 1;
                if drained > 16 {
                    break;
                }
            }
            *session.exit_code.lock().unwrap_or_else(|p| p.into_inner()) = Some(code);
            session.notify(Notify::Exit(code));
            break;
        }
    }
    // Mark dead if we exited via EOF without a status.
    let mut code = session.exit_code.lock().unwrap_or_else(|p| p.into_inner());
    if code.is_none() {
        let final_code = crate::pty::try_wait_pid(child_pid).unwrap_or(-1);
        *code = Some(final_code);
        drop(code);
        session.notify(Notify::Exit(final_code));
    }
}

/// Public shim for integration tests (e2e_proot.rs): run the same reader
/// loop the JNI layer uses, without a JVM.
pub fn reader_loop_for_tests(session: Arc<Session>) {
    reader_loop(session)
}

fn get_session(id: i64) -> Option<Arc<Session>> {
    manager().lock().ok()?.get(&id).cloned()
}

// ---------------------------------------------------------------------------
// JNI exports (package com.xrust.app.nativejni.TerminalNative)
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeCreate(
    env: JNIEnv,
    _class: JClass,
    cols: jint,
    rows: jint,
    capacity: jint,
) -> jlong {
    let _ = env.get_java_vm().map(|vm| {
        if JVM.get().is_none() {
            let _ = JVM.set(vm);
        }
    });
    let cols = cols.clamp(2, 500) as usize;
    let rows = rows.clamp(1, 300) as usize;
    let capacity = capacity.clamp(100, 100_000);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let term = Terminal::new(cols, rows, capacity as usize);
    let session = Arc::new(Session {
        id,
        pty: Mutex::new(None),
        term: Mutex::new(term),
        seq_cv: Condvar::new(),
        listener: Mutex::new(None),
        exit_code: Mutex::new(None),
        reader: Mutex::new(None),
    });
    manager()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(id, session);
    id
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeSpawn<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    id: jlong,
    cmd: JString<'local>,
    args: JObjectArray<'local>,
    env_vars: JObjectArray<'local>,
    cwd: JString<'local>,
) -> jboolean {
    let session = match get_session(id) {
        Some(s) => s,
        None => return 0,
    };
    let cmd_s = match env.get_string(&cmd) {
        Ok(s) => s.to_string_lossy().into_owned(),
        Err(_) => return 0,
    };
    let cwd_s = match env.get_string(&cwd) {
        Ok(s) => s.to_string_lossy().into_owned(),
        Err(_) => return 0,
    };
    let read_list = |env: &mut JNIEnv, arr: &JObjectArray| -> Vec<String> {
        let mut out = Vec::new();
        let len = env.get_array_length(arr).unwrap_or(0);
        for i in 0..len {
            if let Ok(elem) = env.get_object_array_element(arr, i) {
                let js = JString::from(elem);
                let text = env
                    .get_string(&js)
                    .ok()
                    .map(|s| s.to_string_lossy().into_owned());
                if let Some(t) = text {
                    out.push(t);
                }
            }
        }
        out
    };
    let args_v = read_list(&mut env, &args);
    let env_v = read_list(&mut env, &env_vars);
    let (cols, rows) = {
        let term = session.term.lock();
        match term {
            Ok(t) => (t.cols as u16, t.rows as u16),
            Err(_) => (80, 24),
        }
    };
    let handle = PtyHandle::open_and_spawn(cols, rows, &cmd_s, &args_v, &env_v, &cwd_s);
    match handle {
        Ok(h) => {
            *session.pty.lock().unwrap_or_else(|p| p.into_inner()) = Some(h);
            let s2 = session.clone();
            let jh = std::thread::Builder::new()
                .name(format!("xrust-reader-{id}"))
                .spawn(move || reader_loop(s2))
                .ok();
            *session.reader.lock().unwrap_or_else(|p| p.into_inner()) = jh;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeWrite(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
    bytes: JByteArray,
) {
    let session = match get_session(id) {
        Some(s) => s,
        None => return,
    };
    let env = _env;
    if let Ok(data) = env.convert_byte_array(&bytes) {
        // Copy the fd under the mutex, then write without it: write_all can
        // block on EAGAIN backpressure, and holding the pty mutex across
        // that froze concurrent keystrokes for up to 200ms each (e.g. pastes
        // into a flooded pipe). The master fd never changes after spawn.
        let master = session
            .pty
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|h| h.master));
        if let Some(fd) = master {
            let _ = crate::pty::write_all_fd(fd, &data);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeWriteString(
    mut env: JNIEnv,
    _class: JClass,
    id: jlong,
    text: JString,
) {
    let session = match get_session(id) {
        Some(s) => s,
        None => return,
    };
    if let Ok(s) = env.get_string(&text) {
        let data = s.to_string_lossy().into_owned().into_bytes();
        // See nativeWrite: fd copy under the mutex, blocking write without it.
        let master = session
            .pty
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|h| h.master));
        if let Some(fd) = master {
            let _ = crate::pty::write_all_fd(fd, &data);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativePaste(
    mut env: JNIEnv,
    _class: JClass,
    id: jlong,
    text: JString,
) {
    let session = match get_session(id) {
        Some(s) => s,
        None => return,
    };
    if let Ok(s) = env.get_string(&text) {
        let text = s.to_string_lossy().into_owned();
        let resp = {
            let mut term = session.term.lock().unwrap_or_else(|p| p.into_inner());
            term.paste(&text);
            term.take_responses()
        };
        // See nativeWrite: fd copy under the mutex, blocking write without it.
        let master = session
            .pty
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|h| h.master));
        if let Some(fd) = master {
            let _ = crate::pty::write_all_fd(fd, &resp);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeEncodeChar(
    _env: JNIEnv,
    _class: JClass,
    ch: jint,
    mods: jint,
    app_cursor: jboolean,
    app_keypad: jboolean,
) -> jbyteArray {
    let env = _env;
    let key = crate::keymap::Key::Char(char::from_u32(ch as u32).unwrap_or('?'));
    let bytes = crate::keymap::encode(key, mods as u8, app_cursor != 0, app_keypad != 0);
    env.byte_array_from_slice(&bytes)
        .map(|a| a.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeEncodeKey(
    _env: JNIEnv,
    _class: JClass,
    key: jint,
    mods: jint,
    app_cursor: jboolean,
    app_keypad: jboolean,
) -> jbyteArray {
    let env = _env;
    let k = match key {
        KEY_ENTER => crate::keymap::Key::Enter,
        KEY_BACKSPACE => crate::keymap::Key::Backspace,
        KEY_TAB => crate::keymap::Key::Tab,
        KEY_ESCAPE => crate::keymap::Key::Escape,
        KEY_HOME => crate::keymap::Key::Home,
        KEY_END => crate::keymap::Key::End,
        KEY_PAGE_UP => crate::keymap::Key::PageUp,
        KEY_PAGE_DOWN => crate::keymap::Key::PageDown,
        KEY_INSERT => crate::keymap::Key::Insert,
        KEY_DELETE => crate::keymap::Key::Delete,
        KEY_ARROW_UP => crate::keymap::Key::ArrowUp,
        KEY_ARROW_DOWN => crate::keymap::Key::ArrowDown,
        KEY_ARROW_LEFT => crate::keymap::Key::ArrowLeft,
        KEY_ARROW_RIGHT => crate::keymap::Key::ArrowRight,
        k if k >= KEY_FN_BASE => crate::keymap::Key::F((k - KEY_FN_BASE + 1) as u8),
        _ => return std::ptr::null_mut(),
    };
    let bytes = crate::keymap::encode(k, mods as u8, app_cursor != 0, app_keypad != 0);
    env.byte_array_from_slice(&bytes)
        .map(|a| a.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeResize(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
    cols: jint,
    rows: jint,
) {
    let session = match get_session(id) {
        Some(s) => s,
        None => return,
    };
    let cols = cols.clamp(2, 500) as usize;
    let rows = rows.clamp(1, 300) as usize;
    {
        let mut term = session.term.lock().unwrap_or_else(|p| p.into_inner());
        term.resize(cols, rows);
    }
    let guard = session.pty.lock();
    if let Ok(g) = guard.as_ref() {
        if let Some(h) = g.as_ref() {
            let _ = h.resize(cols as u16, rows as u16);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeSnapshot(
    env: JNIEnv,
    _class: JClass,
    id: jlong,
) -> jbyteArray {
    let session = match get_session(id) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let data = session
        .term
        .lock()
        .map(|t| t.encode_snapshot())
        .unwrap_or_default();
    env.byte_array_from_slice(&data)
        .map(|a| a.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativePoll(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
    last_seq: jlong,
    timeout_ms: jint,
) -> jlong {
    let session = match get_session(id) {
        Some(s) => s,
        None => return -1,
    };
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.clamp(0, 2000) as u64);
    let mut guard = match session.term.lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    loop {
        if session.is_dead() {
            return -1;
        }
        let seq = guard.seq as i64;
        if seq != last_seq {
            return seq;
        }
        let now = Instant::now();
        if now >= deadline {
            return seq;
        }
        // Sleep on the seq condvar. The predicate was just checked while
        // holding the term mutex and the reader notifies the same condvar
        // after every feed, so output wakes this call within microseconds —
        // instead of up to one 20ms sleep quantum later, with no busy
        // mutex-churn while idle.
        let (g, _timed_out) = match session.seq_cv.wait_timeout(guard, deadline - now) {
            Ok(pair) => pair,
            Err(_) => return -1,
        };
        guard = g;
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeSeq(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
) -> jlong {
    match get_session(id) {
        Some(s) => s.term.lock().map(|t| t.seq as i64).unwrap_or(-1),
        None => -1,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeIsAlive(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
) -> jboolean {
    match get_session(id) {
        Some(s) => !s.is_dead() as jboolean,
        None => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeExitCode(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
) -> jint {
    match get_session(id) {
        Some(s) => s
            .exit_code
            .lock()
            .map(|c| c.unwrap_or(i32::MIN))
            .unwrap_or(i32::MIN),
        None => i32::MIN,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeSignal(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
    sig: jint,
) {
    if let Some(s) = get_session(id) {
        let guard = s.pty.lock();
        if let Ok(g) = guard.as_ref() {
            if let Some(h) = g.as_ref() {
                h.signal(sig);
            }
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeSetListener(
    env: JNIEnv,
    _class: JClass,
    id: jlong,
    listener: JObject,
) {
    if let Some(s) = get_session(id) {
        if listener.is_null() {
            *s.listener.lock().unwrap_or_else(|p| p.into_inner()) = None;
            return;
        }
        if let Ok(g) = env.new_global_ref(&listener) {
            *s.listener.lock().unwrap_or_else(|p| p.into_inner()) = Some(g);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeSetScrollback(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
    cap: jint,
) {
    if let Some(s) = get_session(id) {
        let mut term = s.term.lock().unwrap_or_else(|p| p.into_inner());
        term.set_scrollback_capacity(cap.clamp(100, 100_000) as usize);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeClearScrollback(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
) {
    if let Some(s) = get_session(id) {
        let mut term = s.term.lock().unwrap_or_else(|p| p.into_inner());
        term.clear_scrollback();
    }
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeGetTitle(
    env: JNIEnv,
    _class: JClass,
    id: jlong,
) -> jstring {
    let t = match get_session(id) {
        Some(s) => s
            .term
            .lock()
            .map(|t| t.title().to_string())
            .unwrap_or_default(),
        None => String::new(),
    };
    env.new_string(t)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeGetHistoryLen(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
) -> jint {
    match get_session(id) {
        Some(s) => s.term.lock().map(|t| t.history_len() as jint).unwrap_or(0),
        None => 0,
    }
}

/// Destroy: terminate the child, wait for the reader, drop everything.
#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    id: jlong,
) {
    let session = match get_session(id) {
        Some(s) => s,
        None => return,
    };
    {
        let guard = session.pty.lock();
        if let Ok(g) = guard.as_ref() {
            if let Some(h) = g.as_ref() {
                h.terminate(300);
            }
        }
    }
    {
        let mut guard = session.pty.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(h) = guard.take() {
            h.close();
        }
    }
    let reader = session
        .reader
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .take();
    if let Some(jh) = reader {
        // Reader exits when it sees the dead pty; bounded join.
        let deadline = Instant::now() + Duration::from_millis(500);
        while !jh.is_finished() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        if !jh.is_finished() {
            // Detach rather than leak a blocked thread join on the UI thread.
            std::mem::forget(jh);
        }
    }
    *session.listener.lock().unwrap_or_else(|p| p.into_inner()) = None;
    manager()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .remove(&id);
}

/// Destroy all sessions (called from `App.onTerminate`).
#[no_mangle]
pub extern "system" fn Java_com_xrust_app_nativejni_TerminalNative_nativeDestroyAll(
    _env: JNIEnv,
    _class: JClass,
) {
    let ids: Vec<i64> = manager()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .keys()
        .copied()
        .collect();
    for id in ids {
        let s = match get_session(id) {
            Some(s) => s,
            None => continue,
        };
        {
            let guard = s.pty.lock();
            if let Ok(g) = guard.as_ref() {
                if let Some(h) = g.as_ref() {
                    h.terminate(200);
                }
            }
        }
        {
            let mut guard = s.pty.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(h) = guard.take() {
                h.close();
            }
        }
    }
    manager().lock().unwrap_or_else(|p| p.into_inner()).clear();
}

#[allow(dead_code)]
fn key_constants_used() {
    let _ = (
        KEY_ENTER,
        KEY_BACKSPACE,
        KEY_TAB,
        KEY_ESCAPE,
        KEY_HOME,
        KEY_END,
        KEY_PAGE_UP,
        KEY_PAGE_DOWN,
        KEY_INSERT,
        KEY_DELETE,
        KEY_ARROW_UP,
        KEY_ARROW_DOWN,
        KEY_ARROW_LEFT,
        KEY_ARROW_RIGHT,
        KEY_FN_BASE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full JNI-level session smoke test without a JVM: uses the internal
    /// session API directly (same path as JNI exports except marshalling).
    #[test]
    fn session_lifecycle_smoke() {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let term = Terminal::new(80, 24, 1000);
        let session = Arc::new(Session {
            id,
            pty: Mutex::new(None),
            term: Mutex::new(term),
            seq_cv: Condvar::new(),
            listener: Mutex::new(None),
            exit_code: Mutex::new(None),
            reader: Mutex::new(None),
        });
        manager().lock().unwrap().insert(id, session.clone());
        let h =
            PtyHandle::open_and_spawn(80, 24, "/bin/sh", &["-c", "echo jni-ok"], &[], "/").unwrap();
        *session.pty.lock().unwrap() = Some(h);
        let s2 = session.clone();
        let jh = std::thread::spawn(move || reader_loop(s2));
        *session.reader.lock().unwrap() = Some(jh);
        // Wait for output to be parsed. Generous deadline: CI runners and
        // sandboxed CI environments can stall the whole process for seconds
        // at a time; only a real parser failure should exhaust it.
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if session.term.lock().unwrap().seq > 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let snap = session.term.lock().unwrap().encode_snapshot();
        assert!(snap.len() > 80 * 24 * 14, "snapshot must contain all cells");
        let text: String = session.term.lock().unwrap().grid().row_text(0, false);
        assert!(
            text.contains("jni-ok"),
            "output reached the terminal: {text:?}"
        );
        // Wait for exit detection (generous: see note above).
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline && !session.is_dead() {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(*session.exit_code.lock().unwrap(), Some(0));
        if let Some(jh) = session.reader.lock().unwrap().take() {
            jh.join().expect("reader thread must not panic");
        }
        {
            let mut guard = session.pty.lock().unwrap();
            if let Some(h) = guard.take() {
                h.close();
            }
        }
        manager().lock().unwrap().remove(&id);
    }

    #[test]
    fn write_reaches_shell() {
        // A runner VM freeze (cgroup/VM suspension) can swallow one attempt
        // without losing data: the round trip resumes and completes once the
        // freeze lifts, but not always inside a single wall-clock deadline.
        // Two fresh attempts (own session, own deadline) ride out a freeze;
        // a genuine write-path regression fails both, bounded at ~4 min.
        fn attempt(id: i64) -> bool {
            let term = Terminal::new(80, 24, 1000);
            let session = Arc::new(Session {
                id,
                pty: Mutex::new(None),
                term: Mutex::new(term),
                seq_cv: Condvar::new(),
                listener: Mutex::new(None),
                exit_code: Mutex::new(None),
                reader: Mutex::new(None),
            });
            manager().lock().unwrap().insert(id, session.clone());
            let found = (|| -> bool {
                let h = match PtyHandle::open_and_spawn(
                    80,
                    24,
                    "/bin/sh",
                    &["-c", "read line; echo got:$line"],
                    &[],
                    "/",
                ) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                *session.pty.lock().unwrap() = Some(h);
                let s2 = session.clone();
                let jh = std::thread::spawn(move || reader_loop(s2));
                *session.reader.lock().unwrap() = Some(jh);
                std::thread::sleep(Duration::from_millis(300));
                {
                    let guard = session.pty.lock().unwrap();
                    guard.as_ref().unwrap().write_all(b"hello-jni\r").unwrap();
                }
                let deadline = Instant::now() + Duration::from_secs(120);
                let mut found = false;
                while Instant::now() < deadline && !found {
                    let t = session.term.lock().unwrap();
                    for r in 0..t.rows {
                        if t.grid().row_text(r, false).contains("got:hello-jni") {
                            found = true;
                            break;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                found
            })();
            let g = session.pty.lock().unwrap();
            if let Some(h) = g.as_ref() {
                h.signal(libc::SIGKILL);
            }
            drop(g);
            if let Some(jh) = session.reader.lock().unwrap().take() {
                let _ = jh.join();
            }
            let mut g = session.pty.lock().unwrap();
            if let Some(h) = g.take() {
                h.close();
            }
            manager().lock().unwrap().remove(&id);
            found
        }
        // Reserve both ids up front so a parallel test cannot take id + 1.
        let first = NEXT_ID.fetch_add(2, Ordering::Relaxed);
        assert!(
            attempt(first) || attempt(first + 1),
            "write path works end-to-end"
        );
    }
}
