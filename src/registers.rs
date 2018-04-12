use libc::user_regs_struct;
use std::fmt;
use std::result::Result;

use debugger::Debugger;
use error::DebugError;

#[derive(Copy, Clone)]
pub struct Registers {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub orig_rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub eflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub fs_base: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

#[allow(non_camel_case_types)]
pub enum Register {
    r15,
    r14,
    r13,
    r12,
    rbp,
    rbx,
    r11,
    r10,
    r9,
    r8,
    rax,
    rcx,
    rdx,
    rsi,
    rdi,
    orig_rax,
    rip,
    cs,
    eflags,
    rsp,
    ss,
    fs_base,
    gs_base,
    ds,
    es,
    fs,
    gs,
}

impl Registers {
    pub fn from_process(dbg: &Debugger, args: Vec<u64>) -> Result<Self, DebugError> {


        let mut regs = dbg.process.getregs()?;

        match args.len() {
            0 => { Ok(regs) },
            1 => {
                regs.rdi = args[0];

                Ok(regs)
            },
            2 => {
                regs.rdi = args[0];
                regs.rsi = args[1];

                Ok(regs)
            },
            3 => {
                regs.rdi = args[0];
                regs.rsi = args[1];
                regs.rdx = args[2];

                Ok(regs)
            },
            4 => {
                regs.rdi = args[0];
                regs.rsi = args[1];
                regs.rdx = args[2];
                regs.rcx = args[3];

                Ok(regs)
            },
            5 => {
                regs.rdi = args[0];
                regs.rsi = args[1];
                regs.rdx = args[2];
                regs.rcx = args[3];
                regs.r8 = args[4];

                Ok(regs)
            },
            6 => {
                regs.rdi = args[0];
                regs.rsi = args[1];
                regs.rdx = args[2];
                regs.rcx = args[3];
                regs.r8 = args[4];
                regs.r9 = args[5];

                Ok(regs)
            },
            _ => {
                regs.rdi = args[0];
                regs.rsi = args[1];
                regs.rdx = args[2];
                regs.rcx = args[3];
                regs.r8 = args[4];
                regs.r9 = args[5];

                for rem in &args[6..] {
                    dbg.process.push(*rem)?;
                }

                Ok(regs)
            }
        }
    }
}

impl From<user_regs_struct> for Registers {
    fn from(other: user_regs_struct) -> Self {
        Registers {
            r15: other.r15,
            r14: other.r14,
            r13: other.r13,
            r12: other.r12,
            rbp: other.rbp,
            rbx: other.rbx,
            r11: other.r11,
            r10: other.r10,
            r9: other.r9,
            r8: other.r8,
            rax: other.rax,
            rcx: other.rcx,
            rdx: other.rdx,
            rsi: other.rsi,
            rdi: other.rdi,
            orig_rax: other.orig_rax,
            rip: other.rip,
            cs: other.cs,
            eflags: other.eflags,
            rsp: other.rsp,
            ss: other.ss,
            fs_base: other.fs_base,
            gs_base: other.gs_base,
            ds: other.ds,
            es: other.es,
            fs: other.fs,
            gs: other.gs,
        }
    }
}

impl Into<user_regs_struct> for Registers {
    fn into(self) -> user_regs_struct {
        user_regs_struct {
            r15: self.r15,
            r14: self.r14,
            r13: self.r13,
            r12: self.r12,
            rbp: self.rbp,
            rbx: self.rbx,
            r11: self.r11,
            r10: self.r10,
            r9: self.r9,
            r8: self.r8,
            rax: self.rax,
            rcx: self.rcx,
            rdx: self.rdx,
            rsi: self.rsi,
            rdi: self.rdi,
            orig_rax: self.orig_rax,
            rip: self.rip,
            cs: self.cs,
            eflags: self.eflags,
            rsp: self.rsp,
            ss: self.ss,
            fs_base: self.fs_base,
            gs_base: self.gs_base,
            ds: self.ds,
            es: self.es,
            fs: self.fs,
            gs: self.gs,
        }
    }
}

impl<'a> Into<user_regs_struct> for &'a Registers {
    fn into(self) -> user_regs_struct {
        user_regs_struct {
            r15: self.r15,
            r14: self.r14,
            r13: self.r13,
            r12: self.r12,
            rbp: self.rbp,
            rbx: self.rbx,
            r11: self.r11,
            r10: self.r10,
            r9: self.r9,
            r8: self.r8,
            rax: self.rax,
            rcx: self.rcx,
            rdx: self.rdx,
            rsi: self.rsi,
            rdi: self.rdi,
            orig_rax: self.orig_rax,
            rip: self.rip,
            cs: self.cs,
            eflags: self.eflags,
            rsp: self.rsp,
            ss: self.ss,
            fs_base: self.fs_base,
            gs_base: self.gs_base,
            ds: self.ds,
            es: self.es,
            fs: self.fs,
            gs: self.gs,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rax: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            orig_rax: 0,
            rip: 0,
            cs: 0,
            eflags: 0,
            rsp: 0,
            ss: 0,
            fs_base: 0,
            gs_base: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /* TODO include flags */
        write!(f,
"\trip: 0x{:x}
\trsp: 0x{:x}
\trbp: 0x{:x}
\trax: 0x{:x}
\trbx: 0x{:x}
\trcx: 0x{:x}
\trdx: 0x{:x}
\trsi: 0x{:x}
\trdi: 0x{:x}",
            self.rip,
            self.rsp,
            self.rbp,
            self.rax,
            self.rbx,
            self.rcx,
            self.rdx,
            self.rsi,
            self.rdi
        )
    }
}
