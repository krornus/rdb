use libc::{user_regs_struct,c_void};
use libc::ptrace;
use libc::{
    PTRACE_GETREGS,
    PTRACE_SETREGS,
    PTRACE_CONT,
    PTRACE_PEEKTEXT,
    PTRACE_POKETEXT,
    PTRACE_SINGLESTEP
};
use nix::{errno,Errno};

use std::ptr;
use std::result::Result;
use std::cell::Cell;

use status::Status;
use error::DebugError;
use registers::{Register,Registers};

#[derive(Debug,Clone)]
pub struct Process {
    pub pid: u32,
    pub status: Cell<Status>,
}

impl Process {

    pub fn new(pid: u32) -> Self {
        Process {
            pid: pid,
            status: Cell::new(Status::new(0)),
        }
    }

    pub fn status(&self) -> Status {
        self.status.get().clone()
    }

    pub fn wait_stop(&self) -> Result<u64, DebugError> {
        let stat = self.wait();

        if stat.stopped() {
            let ip = self.getregs()
                .expect("failed to get registers of process");
            Ok(ip.rip)
        } else {
            Err(DebugError::from(stat))
        }
    }

    pub fn wait(&self) -> Status {
        let status = Status::wait(self.pid as i32);
        self.status.set(status);

        status
    }

    pub fn step(&self) -> Result<i64, DebugError> {
        unsafe {
            Errno::clear();

            let res = ptrace(PTRACE_SINGLESTEP, self.pid, ptr::null::<c_void>(), ptr::null::<c_void>());

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(res)
            }
        }
    }

    pub fn peek(&self, addr: u64) -> Result<u64, DebugError> {
        unsafe {
            Errno::clear();

            let res = ptrace(PTRACE_PEEKTEXT, self.pid, addr, ptr::null::<c_void>());

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(res as u64)
            }
        }
    }

    pub fn setregs_user(&self, mut regs: Registers) -> Result<i64, DebugError> {
        let cregs = self.getregs()?;
        regs.rip = cregs.rip;
        regs.rsp = cregs.rsp;
        regs.rbp = cregs.rbp;

        self.setregs(&mut regs)
    }

    pub fn stack(&self, offset: u64) -> Result<u64, DebugError> {
        let regs = self.getregs()?;
        let sp = regs.rsp + offset;

        self.peek(sp)
    }

    pub fn push(&self, data: u64) -> Result<u64, DebugError> {
        let mut regs = self.getregs()?;
        regs.rsp -= 0x8;

        self.setregs(&regs)?;
        self.poke(regs.rsp, data)?;

        Ok(regs.rsp)
    }

    pub fn retn(&self) -> Result<u64, DebugError> {
        let regs = self.getregs()?;
        let retn = regs.rbp + 0x8;

        self.peek(retn)
    }

    pub fn poke(&self, addr: u64, word: u64) -> Result<i64, DebugError> {
        unsafe {
            Errno::clear();

            let res = ptrace(PTRACE_POKETEXT, self.pid, addr, word);

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(res)
            }
        }
    }

    pub fn getregs(&self) -> Result<Registers, DebugError> {

        unsafe {
            let mut regs: user_regs_struct = Registers::default().into();
            Errno::clear();

            ptrace(PTRACE_GETREGS, self.pid, ptr::null::<c_void>(), &mut regs);

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(Registers::from(regs))
            }
        }
    }

    pub fn cont(&self) -> Result<i64, DebugError> {

        let ret = unsafe {
            Errno::clear();

            let ret = ptrace(PTRACE_CONT, self.pid, ptr::null::<c_void>(), ptr::null::<c_void>());

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(ret)
            }
        };

        ret
    }

    pub fn setregs(&self, regs: &Registers) -> Result<i64, DebugError> {

        let urs: user_regs_struct = regs.into();
        unsafe {
            Errno::clear();

            let ret = ptrace(PTRACE_SETREGS, self.pid, ptr::null::<c_void>(), &urs);

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(ret)
            }
        }
    }

    pub fn setreg(&self, reg: Register, value: u64) -> Result<i64, DebugError> {

        let mut register = self.getregs()?;

        match reg {
            Register::r15 => {
                register.r15 = value;
            },
            Register::r14 => {
                register.r14 = value;
            },
            Register::r13 => {
                register.r13 = value;
            },
            Register::r12 => {
                register.r12 = value;
            },
            Register::rbp => {
                register.rbp = value;
            },
            Register::rbx => {
                register.rbx = value;
            },
            Register::r11 => {
                register.r11 = value;
            },
            Register::r10 => {
                register.r10 = value;
            },
            Register::r9 => {
                register.r9 = value;
            },
            Register::r8 => {
                register.r8 = value;
            },
            Register::rax => {
                register.rax = value;
            },
            Register::rcx => {
                register.rcx = value;
            },
            Register::rdx => {
                register.rdx = value;
            },
            Register::rsi => {
                register.rsi = value;
            },
            Register::rdi => {
                register.rdi = value;
            },
            Register::orig_rax => {
                register.orig_rax = value;
            },
            Register::rip => {
                register.rip = value;
            },
            Register::cs => {
                register.cs = value;
            },
            Register::eflags => {
                register.eflags = value;
            },
            Register::rsp => {
                register.rsp = value;
            },
            Register::ss => {
                register.ss = value;
            },
            Register::fs_base => {
                register.fs_base = value;
            },
            Register::gs_base => {
                register.gs_base = value;
            },
            Register::ds => {
                register.ds = value;
            },
            Register::es => {
                register.es = value;
            },
            Register::fs => {
                register.fs = value;
            },
            Register::gs => {
                register.gs = value;
            },
        }

        self.setregs(&register)
    }
}
