use libc::{waitpid, WIFSTOPPED, WIFEXITED, WIFSIGNALED, WSTOPSIG, WTERMSIG, WCOREDUMP};
use nix::sys::signal::Signal;

#[derive(Debug,Clone,Copy)]
pub struct Status {
    status: i32
}

impl Status {
    pub fn wait(pid: i32) -> Self {

        let mut s: i32 = 0;
        unsafe { waitpid(pid, &mut s, 0); }

        Status { status: s }
    }

    pub fn new(sig: i32) -> Self {
        Status { status: sig }
    }

    pub fn running(&self) -> bool {
        return !self.stopped()
            && !self.exited()
            /* allow for sigint */
            && (!self.signaled() && self.signal() != Some(Signal::SIGINT))
            && !self.coredumped()
    }

    pub fn stopped(&self) -> bool {
        unsafe { WIFSTOPPED(self.status) }
    }

    pub fn coredumped(&self) -> bool {
        unsafe { WCOREDUMP(self.status) }
    }

    pub fn exited(&self) -> bool {
        unsafe { WIFEXITED(self.status) }
    }

    pub fn signaled(&self) -> bool {
        unsafe { WIFSIGNALED(self.status) }
    }

    pub fn stopsig(&self) -> Option<i32> {
        if self.stopped() {
            Some(unsafe { WSTOPSIG(self.status) })
        } else {
            None
        }
    }

    pub fn termsig(&self) -> Option<i32> {
        if self.exited() {
            Some(unsafe { WTERMSIG(self.status) })
        } else {
            None
        }
    }

    pub fn trapped(&self) -> bool {
        self.signal() == Some(Signal::SIGTRAP)
    }

    pub fn signal(&self) -> Option<Signal> {
        let sig = if let Some(x) = self.termsig() {
            x
        } else if let Some(x) = self.stopsig() {
            x
        } else {
            return None;
        };

        /* TODO: apparently this can error? use result? */
        match Signal::from_c_int(sig) {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }
}

impl ToString for Status {
    fn to_string(&self) -> String {

        use self::Signal::*;
        match self.signal() {
            Some(SIGHUP) => { "SIGHUP".to_string() },
            Some(SIGINT) => { "SIGINT".to_string() },
            Some(SIGQUIT) => { "SIGQUIT".to_string() },
            Some(SIGILL) => { "SIGILL".to_string() },
            Some(SIGTRAP) => { "SIGTRAP".to_string() },
            Some(SIGABRT) => { "SIGABRT".to_string() },
            Some(SIGBUS) => { "SIGBUS".to_string() },
            Some(SIGFPE) => { "SIGFPE".to_string() },
            Some(SIGKILL) => { "SIGKILL".to_string() },
            Some(SIGUSR1) => { "SIGUSR1".to_string() },
            Some(SIGSEGV) => { "SIGSEGV".to_string() },
            Some(SIGUSR2) => { "SIGUSR2".to_string() },
            Some(SIGPIPE) => { "SIGPIPE".to_string() },
            Some(SIGALRM) => { "SIGALRM".to_string() },
            Some(SIGTERM) => { "SIGTERM".to_string() },
            Some(SIGSTKFLT) => { "SIGSTKFLT".to_string() },
            Some(SIGCHLD) => { "SIGCHLD".to_string() },
            Some(SIGCONT) => { "SIGCONT".to_string() },
            Some(SIGSTOP) => { "SIGSTOP".to_string() },
            Some(SIGTSTP) => { "SIGTSTP".to_string() },
            Some(SIGTTIN) => { "SIGTTIN".to_string() },
            Some(SIGTTOU) => { "SIGTTOU".to_string() },
            Some(SIGURG) => { "SIGURG".to_string() },
            Some(SIGXCPU) => { "SIGXCPU".to_string() },
            Some(SIGXFSZ) => { "SIGXFSZ".to_string() },
            Some(SIGVTALRM) => { "SIGVTALRM".to_string() },
            Some(SIGPROF) => { "SIGPROF".to_string() },
            Some(SIGWINCH) => { "SIGWINCH".to_string() },
            Some(SIGIO) => { "SIGIO".to_string() },
            Some(SIGPWR) => { "SIGPWR".to_string() },
            Some(SIGSYS) => { "SIGSYS".to_string() },
            None => { "UKNOWN".to_string() },
        }
    }
}
