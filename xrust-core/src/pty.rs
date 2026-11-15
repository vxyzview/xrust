//! PTY creation, child process spawn, I/O, resize, and reaping.
//!
//! Safety notes: `fork` happens before any threads that touch the PTY are
//! started by the session manager; the child does async-signal-safe work
//! only (dup2/setsid/execve).

use std::ffi::CString;
use std::io;

/// A spawned process bound to its pty.
pub struct PtyHandle {
    pub master: i32,
    pub pid: i32,
}

/// Open a pty pair with the given initial size. Returns `(master, slave)`.
pub fn open_pty(cols: u16, rows: u16) -> io::Result<(i32, i32)> {
    let mut master: i32 = -1;
    let mut slave: i32 = -1;
    let rc = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    resize_fd(slave, cols, rows)?;
    Ok((master, slave))
}

/// TIOCSWINSZ on a pty fd.
pub fn resize_fd(fd: i32, cols: u16, rows: u16) -> io::Result<()> {
    let ws = libc::winsize {
        ws_col: cols,
        ws_row: rows,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let rc = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn to_cstring(s: &str) -> CString {
    // NUL bytes in args/env are invalid; replace to keep execve safe.
    CString::new(s.replace('\0', "\u{FFFD}")).unwrap_or_else(|_| CString::new("?").unwrap())
}

/// Fork+exec `cmd` attached to `slave`. `env` entries are `KEY=VALUE`.
/// On success the slave fd is closed in the parent.
pub fn spawn<S: AsRef<str>>(
    master: i32,
    slave: i32,
    cmd: &str,
    args: &[S],
    env: &[S],
    cwd: &str,
) -> io::Result<i32> {
    let cmd_c = to_cstring(cmd);
    let mut argv: Vec<CString> = Vec::with_capacity(args.len() + 1);
    argv.push(cmd_c.clone());
    for a in args {
        argv.push(to_cstring(a.as_ref()));
    }
    let env_strs: Vec<String> = env.iter().map(|e| e.as_ref().to_string()).collect();
    let mut envp: Vec<CString> = env_strs.iter().map(|e| to_cstring(e)).collect();
    if !env_strs.iter().any(|e| e.starts_with("TERM=")) {
        envp.push(to_cstring("TERM=xterm-256color"));
    }
    if !env_strs.iter().any(|e| e.starts_with("COLORTERM=")) {
        envp.push(to_cstring("COLORTERM=truecolor"));
    }
    let cwd_c = to_cstring(if cwd.is_empty() { "/" } else { cwd });

    let argv_p: Vec<*const libc::c_char> = argv
        .iter()
        .map(|c| c.as_ptr())
        .chain([std::ptr::null()])
        .collect();
    let envp_p: Vec<*const libc::c_char> = envp
        .iter()
        .map(|c| c.as_ptr())
        .chain([std::ptr::null()])
        .collect();

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err(io::Error::last_os_error());
    }
    if pid == 0 {
        // ---- child (async-signal-safe code only) ----
        unsafe {
            libc::close(master);
            libc::setsid();
            // Make the slave our controlling terminal.
            libc::ioctl(slave, libc::TIOCSCTTY, 0);
            libc::dup2(slave, 0);
            libc::dup2(slave, 1);
            libc::dup2(slave, 2);
            if slave > 2 {
                libc::close(slave);
            }
            // Reset signal dispositions inherited from the host.
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
            libc::signal(libc::SIGCHLD, libc::SIG_DFL);
            let mut mask: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut mask);
            libc::sigprocmask(libc::SIG_SETMASK, &mask, std::ptr::null_mut());
            libc::chdir(cwd_c.as_ptr());
            libc::execve(cmd_c.as_ptr(), argv_p.as_ptr(), envp_p.as_ptr());
            // exec failed: exit hard so the session sees EOF.
            libc::_exit(127);
        }
    }
    // ---- parent ----
    unsafe {
        libc::close(slave);
        // Non-blocking master: reads use poll(), writes tolerate EAGAIN.
        let flags = libc::fcntl(master, libc::F_GETFL, 0);
        libc::fcntl(master, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
    Ok(pid)
}

/// Poll a raw pty fd for readability (fd-level, no lock held).
pub fn wait_readable_fd(fd: i32, timeout_ms: i32) -> bool {
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let rc = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
    rc > 0 && (pfd.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR)) != 0
}

/// Read from a raw pty fd (fd-level). Returns 0 on EOF or transient error;
/// callers drive exit detection separately.
pub fn read_fd(fd: i32, buf: &mut [u8]) -> usize {
    let rc = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    if rc < 0 {
        0
    } else {
        rc as usize
    }
}

/// Write all bytes to a raw pty fd with EAGAIN backpressure (fd-level).
pub fn write_all_fd(fd_fd: i32, mut bytes: &[u8]) -> bool {
    while !bytes.is_empty() {
        let rc = unsafe { libc::write(fd_fd, bytes.as_ptr() as *const libc::c_void, bytes.len()) };
        if rc < 0 {
            match std::io::Error::last_os_error().raw_os_error() {
                Some(libc::EAGAIN) => {
                    let mut pfd = libc::pollfd {
                        fd: fd_fd,
                        events: libc::POLLOUT,
                        revents: 0,
                    };
                    unsafe { libc::poll(&mut pfd, 1, 200) };
                    continue;
                }
                Some(libc::EINTR) => continue,
                _ => return false,
            }
        }
        bytes = &bytes[rc as usize..];
    }
    true
}

/// Non-blocking reap for a raw pid (fd-level, no lock held).
pub fn try_wait_pid(pid: i32) -> Option<i32> {
    loop {
        let mut status: i32 = 0;
        let rc = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if rc == pid {
            return Some(decode_status(status));
        }
        if rc == 0 {
            return None; // still running
        }
        // A transient EINTR must not be mistaken for process death: Android
        // delivers signals constantly, and a spurious EINTR here used to
        // terminate live sessions (reported as exit -1 while the shell ran).
        let err = std::io::Error::last_os_error().raw_os_error();
        if err == Some(libc::EINTR) {
            continue;
        }
        return Some(-1); // ECHILD / EINVAL: treat as reaped/unknown death
    }
}

impl PtyHandle {
    pub fn open_and_spawn<S: AsRef<str>>(
        cols: u16,
        rows: u16,
        cmd: &str,
        args: &[S],
        env: &[S],
        cwd: &str,
    ) -> io::Result<PtyHandle> {
        let (master, slave) = open_pty(cols, rows)?;
        match spawn(master, slave, cmd, args, env, cwd) {
            Ok(pid) => Ok(PtyHandle { master, pid }),
            Err(e) => {
                unsafe {
                    libc::close(master);
                    libc::close(slave);
                }
                Err(e)
            }
        }
    }

    /// Poll for readability (or EOF) with timeout in ms. Returns true when
    /// there is data (or the peer closed).
    pub fn wait_readable(&self, timeout_ms: i32) -> bool {
        wait_readable_fd(self.master, timeout_ms)
    }

    /// Read up to `buf.len()` bytes. Ok(0) = EOF, Err WouldBlock = no data.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let rc = unsafe {
            libc::read(
                self.master,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
            )
        };
        if rc < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::WouldBlock || e.raw_os_error() == Some(libc::EINTR) {
                return Err(e);
            }
            return Err(e);
        }
        Ok(rc as usize)
    }

    /// Write all bytes, retrying on EAGAIN/EINTR with poll backpressure.
    pub fn write_all(&self, mut bytes: &[u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            let rc = unsafe {
                libc::write(
                    self.master,
                    bytes.as_ptr() as *const libc::c_void,
                    bytes.len(),
                )
            };
            if rc < 0 {
                let e = io::Error::last_os_error();
                match e.raw_os_error() {
                    Some(libc::EAGAIN) => {
                        let mut pfd = libc::pollfd {
                            fd: self.master,
                            events: libc::POLLOUT,
                            revents: 0,
                        };
                        unsafe { libc::poll(&mut pfd, 1, 200) };
                        continue;
                    }
                    Some(libc::EINTR) => continue,
                    _ => return Err(e),
                }
            }
            bytes = &bytes[rc as usize..];
        }
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        resize_fd(self.master, cols, rows)?;
        // Deliver SIGWINCH to the foreground process group.
        unsafe {
            libc::kill(-self.pid, libc::SIGWINCH);
        }
        Ok(())
    }

    pub fn signal(&self, sig: i32) {
        unsafe {
            libc::kill(self.pid, sig);
            libc::kill(-self.pid, sig);
        }
    }

    /// Non-blocking reaping: Some(exit_code) when the child has exited.
    pub fn try_wait(&self) -> Option<i32> {
        try_wait_pid(self.pid)
    }

    /// Blocking wait for exit.
    pub fn wait(&self) -> i32 {
        let mut status: i32 = 0;
        loop {
            let rc = unsafe { libc::waitpid(self.pid, &mut status, 0) };
            if rc == self.pid {
                return decode_status(status);
            }
            if rc < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
                return -1;
            }
        }
    }

    pub fn close(&self) {
        unsafe {
            libc::close(self.master);
        }
    }

    /// Politely terminate: SIGHUP, then SIGKILL after `grace_ms`.
    pub fn terminate(&self, grace_ms: u32) {
        self.signal(libc::SIGHUP);
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(grace_ms as u64);
        while std::time::Instant::now() < deadline {
            if self.try_wait().is_some() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        self.signal(libc::SIGKILL);
        let _ = self.wait();
    }
}

fn decode_status(status: i32) -> i32 {
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status)
    } else if libc::WIFSIGNALED(status) {
        128 + libc::WTERMSIG(status)
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_pty_sane() {
        let (m, s) = open_pty(80, 24).unwrap();
        assert!(m > 0 && s > 0);
        unsafe {
            libc::close(m);
            libc::close(s);
        }
    }

    #[test]
    fn resize_sets_winsize() {
        let (m, s) = open_pty(80, 24).unwrap();
        resize_fd(m, 100, 40).unwrap();
        let mut ws = libc::winsize {
            ws_col: 0,
            ws_row: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            assert_eq!(libc::ioctl(m, libc::TIOCGWINSZ, &mut ws), 0);
        }
        assert_eq!((ws.ws_col, ws.ws_row), (100, 40));
        unsafe {
            libc::close(m);
            libc::close(s);
        }
    }

    #[test]
    fn spawn_echo_roundtrip() {
        let h = PtyHandle::open_and_spawn(
            80,
            24,
            "/bin/sh",
            &["-c", "echo marker-$FOO; exit 3"],
            &["FOO=bar"],
            "/",
        )
        .unwrap();
        let mut buf = [0u8; 4096];
        let mut all = Vec::new();
        for _ in 0..100 {
            if h.wait_readable(100) {
                match h.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => all.extend_from_slice(&buf[..n]),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }
            }
            if h.try_wait().is_some() {
                // drain a bit more
                if let Ok(n) = h.read(&mut buf) {
                    all.extend_from_slice(&buf[..n]);
                }
                break;
            }
        }
        let out = String::from_utf8_lossy(&all);
        assert!(out.contains("marker-bar"), "got {out:?}");
        let code = h.wait();
        assert_eq!(code, 3, "exit code propagated");
        h.close();
    }

    #[test]
    fn exec_failure_returns_127() {
        let h = PtyHandle::open_and_spawn(
            80,
            24,
            "/nonexistent/binary-xyz",
            &[] as &[&str],
            &[] as &[&str],
            "/",
        )
        .unwrap();
        let code = h.wait();
        assert_eq!(code, 127);
        h.close();
    }

    #[test]
    fn resize_sends_sigwinch() {
        // Child traps SIGWINCH and prints; retry the resize a few times to
        // stay robust under load (the trap needs a moment to install).
        let h = PtyHandle::open_and_spawn(
            80,
            24,
            "/bin/sh",
            &["-c", "trap 'echo WINCH' WINCH; sleep 5"],
            &[],
            "/",
        )
        .unwrap();
        let mut buf = [0u8; 4096];
        let mut got = false;
        'outer: for _ in 0..5 {
            h.resize(40, 10).unwrap();
            for _ in 0..20 {
                if h.wait_readable(100) {
                    let n = h.read(&mut buf).unwrap_or(0);
                    if String::from_utf8_lossy(&buf[..n]).contains("WINCH") {
                        got = true;
                        break 'outer;
                    }
                }
            }
        }
        h.signal(libc::SIGKILL);
        let _ = h.wait();
        h.close();
        assert!(got, "SIGWINCH must reach the child");
    }

    #[test]
    fn terminate_kills_stubborn_child() {
        let h = PtyHandle::open_and_spawn(
            80,
            24,
            "/bin/sh",
            &["-c", "trap '' HUP; sleep 60"],
            &[],
            "/",
        )
        .unwrap();
        let t0 = std::time::Instant::now();
        h.terminate(300);
        assert!(
            t0.elapsed() < std::time::Duration::from_secs(30),
            "SIGKILL fallback must be quick"
        );
    }

    #[test]
    fn term_env_defaults_injected() {
        // If env lacks TERM, spawn injects TERM=xterm-256color.
        let h = PtyHandle::open_and_spawn(80, 24, "/bin/sh", &["-c", "echo T=$TERM"], &[], "/")
            .unwrap();
        let mut buf = [0u8; 1024];
        let mut all = Vec::new();
        for _ in 0..50 {
            if h.wait_readable(100) {
                match h.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => all.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }
            if h.try_wait().is_some() {
                break;
            }
        }
        let out = String::from_utf8_lossy(&all);
        assert!(out.contains("T=xterm-256color"), "got {out:?}");
        let _ = h.wait();
        h.close();
    }

    #[test]
    fn cwd_is_honored() {
        let h = PtyHandle::open_and_spawn(80, 24, "/bin/sh", &["-c", "pwd"], &[], "/tmp").unwrap();
        let mut buf = [0u8; 1024];
        let mut all = Vec::new();
        for _ in 0..50 {
            if h.wait_readable(100) {
                match h.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => all.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }
            if h.try_wait().is_some() {
                break;
            }
        }
        let out = String::from_utf8_lossy(&all);
        assert!(out.contains("/tmp"), "got {out:?}");
        let _ = h.wait();
        h.close();
    }
}
