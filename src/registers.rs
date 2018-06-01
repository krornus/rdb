use std::fmt;
use std::convert::Into;
use std::result::Result;

use libc::user_regs_struct;

use debugger::Debugger;
use error::DebugError;

pub trait Cast<T> {
    fn cast(self) -> T;
}

impl Cast<usize> for u64 {
    fn cast(self) -> usize {
        self as usize
    }
}

impl Cast<u64> for usize {
    fn cast(self) -> u64 {
        self as u64
    }
}


pub trait Register {
    type Size where
        Self::Size: Cast<usize>,
        usize: Cast<Self::Size>,
        Self: fmt::Display;

    fn ip(&self) -> Self::Size;
    fn sp(&self) -> Self::Size;
    fn bp(&self) -> Self::Size;
    fn set_ip(&mut self, ptr: Self::Size);
    fn set_sp(&mut self, ptr: Self::Size);
    fn set_bp(&mut self, ptr: Self::Size);
    fn mask(&self, reg: Self) -> Self;
    fn stack_offset(&self, offset: Self::Size) -> Self::Size;
    fn size_from(n: i64) -> Self::Size;
}

/* x86-64 */
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub struct x86_64_Registers {
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
pub enum x86_64_Register {
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

impl Register for x86_64_Registers {

    type Size = u64;

    fn ip(&self) -> Self::Size {
        self.rip
    }

    fn sp(&self) -> Self::Size {
        self.rsp
    }

    fn bp(&self) -> Self::Size {
        self.rbp
    }

    fn set_ip(&mut self, ptr: Self::Size) {
        self.rip = ptr;
    }

    fn set_sp(&mut self, ptr: Self::Size) {
        self.rsp = ptr;
    }

    fn set_bp(&mut self, ptr: Self::Size) {
        self.rbp = ptr;
    }

    fn mask(&self, mut reg: Self) -> Self {
        reg.rip = self.rip;
        reg.rsp = self.rsp;
        reg.rbp =  self.rbp;

        reg
    }

    fn stack_offset(&self, offset: Self::Size) -> Self::Size {
        self.rsp + offset
    }

    fn size_from(n: i64) -> Self::Size {
        n as Self::Size
    }
}

impl x86_64_Registers {
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

impl From<user_regs_struct> for x86_64_Registers {
    fn from(other: user_regs_struct) -> Self {
        x86_64_Registers {
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

impl Into<user_regs_struct> for x86_64_Registers {
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

impl<'a> Into<user_regs_struct> for &'a x86_64_Registers {
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

impl Default for x86_64_Registers {
    fn default() -> Self {
        x86_64_Registers {
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

impl fmt::Display for x86_64_Registers {
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
