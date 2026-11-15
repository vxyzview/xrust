//! End-to-end: our PTY + parser stack driving a REAL Alpine shell through
//! PRoot. Skipped unless both `PROOT_BIN` and `ALPINE_ROOTFS` env vars are
//! set (see scripts/e2e.sh which wires them up).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use xrust_core::android_jni::Session;
use xrust_core::pty::PtyHandle;
use xrust_core::term::Terminal;

static NEXT_ID: AtomicI64 = AtomicI64::new(10_000);

fn env_ready() -> Option<(PathBuf, PathBuf)> {
    let proot = std::env::var_os("PROOT_BIN").map(PathBuf::from)?;
    let rootfs = std::env::var_os("ALPINE_ROOTFS").map(PathBuf::from)?;
    if !proot.exists() || !rootfs.exists() {
        return None;
    }
    Some((proot, rootfs))
}

fn make_session() -> Arc<Session> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    Arc::new(Session {
        id,
        pty: Mutex::new(None),
        term: Mutex::new(Terminal::new(100, 30, 2000)),
        listener: Mutex::new(None),
        exit_code: Mutex::new(None),
        reader: Mutex::new(None),
    })
}

fn run_in_alpine(session: &Arc<Session>, proot: &Path, rootfs: &Path, script: &str) {
    let cmd = proot.to_string_lossy().into_owned();
    let args = vec![
        "-R".to_string(),
        rootfs.to_string_lossy().into_owned(),
        "-b".to_string(),
        "/proc".to_string(),
        "/bin/sh".to_string(),
        "-c".to_string(),
        script.to_string(),
    ];
    let (cols, rows) = {
        let t = session.term.lock().unwrap();
        (t.cols as u16, t.rows as u16)
    };
    let h = PtyHandle::open_and_spawn(cols, rows, &cmd, &args, &[], "/").expect("spawn proot");
    *session.pty.lock().unwrap() = Some(h);
    let s2 = session.clone();
    let jh = std::thread::spawn(move || xrust_core::android_jni::reader_loop_for_tests(s2));
    *session.reader.lock().unwrap() = Some(jh);
}

fn wait_for(session: &Arc<Session>, needle: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let t = session.term.lock().unwrap();
        for r in 0..t.rows {
            if t.grid().row_text(r, false).contains(needle) {
                return true;
            }
        }
        drop(t);
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

fn teardown(session: &Arc<Session>) {
    {
        let guard = session.pty.lock().unwrap();
        if let Some(h) = guard.as_ref() {
            h.signal(libc::SIGKILL);
        }
    }
    std::thread::sleep(Duration::from_millis(100));
    if let Some(jh) = session.reader.lock().unwrap().take() {
        let _ = jh.join();
    }
    {
        let mut guard = session.pty.lock().unwrap();
        if let Some(h) = guard.take() {
            h.close();
        }
    }
}

#[test]
fn alpine_shell_reports_kernel_and_userland() {
    let Some((proot, rootfs)) = env_ready() else {
        eprintln!("SKIP: PROOT_BIN/ALPINE_ROOTFS not set");
        return;
    };
    let session = make_session();
    run_in_alpine(
        &session,
        &proot,
        &rootfs,
        "echo MARKER-$(uname -m)-$(cat /etc/alpine-release)",
    );
    let ok = wait_for(&session, "MARKER-", Duration::from_secs(30));
    let text = {
        let t = session.term.lock().unwrap();
        (0..t.rows)
            .map(|r| t.grid().row_text(r, false))
            .collect::<Vec<_>>()
            .join("\n")
    };
    teardown(&session);
    assert!(ok, "marker not seen; output:\n{text}");
    assert!(
        text.contains("x86_64"),
        "uname must report x86_64: {text:?}"
    );
    assert!(
        text.contains("3.24.4"),
        "alpine-release must be 3.24.4: {text:?}"
    );
}

#[test]
fn alpine_shell_filesystem_is_isolated() {
    let Some((proot, rootfs)) = env_ready() else {
        eprintln!("SKIP: PROOT_BIN/ALPINE_ROOTFS not set");
        return;
    };
    let session = make_session();
    run_in_alpine(
        &session,
        &proot,
        &rootfs,
        "mkdir -p /e2e-dir && echo e2e-content > /e2e-dir/f.txt && ls /e2e-dir && cat /e2e-dir/f.txt && echo ROOT-OK",
    );
    let ok = wait_for(&session, "ROOT-OK", Duration::from_secs(30));
    let text = {
        let t = session.term.lock().unwrap();
        (0..t.rows)
            .map(|r| t.grid().row_text(r, false))
            .collect::<Vec<_>>()
            .join("\n")
    };
    teardown(&session);
    assert!(ok, "rootfs ops failed; output:\n{text}");
    assert!(text.contains("f.txt"));
    assert!(text.contains("e2e-content"));
    // Isolation: the file must exist inside the rootfs dir on the host.
    let host_file = rootfs.join("e2e-dir/f.txt");
    assert!(host_file.exists(), "file must land in the rootfs directory");
    std::fs::remove_dir_all(rootfs.join("e2e-dir")).ok();
}

#[test]
fn alpine_shell_pipeline_and_colors() {
    let Some((proot, rootfs)) = env_ready() else {
        eprintln!("SKIP: PROOT_BIN/ALPINE_ROOTFS not set");
        return;
    };
    let session = make_session();
    run_in_alpine(
        &session,
        &proot,
        &rootfs,
        "printf '\\033[1;32mgreen\\033[0m\\n'; seq 1 5 | tail -2; echo PIPE-OK",
    );
    let ok = wait_for(&session, "PIPE-OK", Duration::from_secs(30));
    let text = {
        let t = session.term.lock().unwrap();
        (0..t.rows)
            .map(|r| t.grid().row_text(r, false))
            .collect::<Vec<_>>()
            .join("\n")
    };
    teardown(&session);
    assert!(ok, "pipeline failed; output:\n{text}");
    assert!(
        text.contains("green"),
        "SGR text must reach the grid: {text:?}"
    );
    assert!(
        text.contains("4") && text.contains("5"),
        "seq|tail output: {text:?}"
    );
}
