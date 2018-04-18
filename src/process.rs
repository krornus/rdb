use libc::{user_regs_struct,c_void};
use libc::ptrace;
use libc::{
    PTRACE_GETREGS,
    PTRACE_SETREGS,
    PTRACE_CONT,
    PTRACE_PEEKTEXT,
    PTRACE_POKETEXT,
    PTRACE_SINGLESTEP,
    PTRACE_ATTACH,
    PTRACE_DETACH,
};
use nix::{errno,Errno};

use std::ptr;
use std::mem;
use std::result::Result;
use std::cell::Cell;
use std::marker::PhantomData;

use status::Status;
use error::DebugError;
use registers::{Register,Cast};

#[derive(Debug,Clone)]
pub struct Process<T> {
    pub pid: u32,
    pub status: Cell<Status>,
    marker: PhantomData<T>,
}

macro_rules! rsize {
    ($x: ty) => {
        <T as Register>::Size
    }
}

impl<T> Process<T>
    where
        T: Register + Default + Clone,
        T: From<user_regs_struct> + Into<user_regs_struct>,
        <T as Register>::Size: Cast<usize>,
        usize: Cast<<T as Register>::Size>,
    {

    pub fn new(pid: u32) -> Self {
        Process {
            pid: pid,
            status: Cell::new(Status::new(0)),
            marker: PhantomData,
        }
    }

    pub fn status(&self) -> Status {
        self.status.get().clone()
    }

    pub fn wait_stop(&self) -> Result<rsize!(T), DebugError> {
        let stat = self.wait();

        if stat.stopped() {
            let regs = self.getregs()
                .expect("failed to get registers of process");

            Ok(regs.ip())
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

    pub fn peek(&self, addr: rsize!(T)) -> Result<rsize!(T), DebugError> {
        unsafe {
            Errno::clear();

            let res = ptrace(PTRACE_PEEKTEXT, self.pid, addr, ptr::null::<c_void>());

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(T::size_from(res))
            }
        }
    }

    pub fn setregs_user(&self, regs: T) -> Result<i64, DebugError> {

        let mregs = self.getregs()?;

        self.setregs(&mut mregs.mask(regs))
    }

    pub fn stack(&self, offset: rsize!(T)) -> Result<rsize!(T), DebugError> {
        let sp = self.getregs()?
            .stack_offset(offset);

        self.peek(sp)
    }

    pub fn push(&self, data: rsize!(T)) -> Result<rsize!(T), DebugError> {
        let mut regs = self.getregs()?;
        let ip = regs.ip();
        regs.set_ip(
            (ip.cast() - mem::size_of::<rsize!(T)>()).cast()
        );

        self.setregs(&regs)?;
        self.poke(regs.sp(), data)?;

        Ok(regs.sp())
    }

    pub fn retn(&self) -> Result<rsize!(T), DebugError> {
        let regs = self.getregs()?;
        let retn = regs.bp().cast() + 0x8;

        self.peek(retn.cast())
    }

    pub fn poke(&self, addr: rsize!(T), word: rsize!(T)) -> Result<i64, DebugError> {
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

    pub fn attach(&self) -> Result<i64, DebugError> {
        unsafe {
            Errno::clear();

            let res = ptrace(PTRACE_ATTACH, self.pid, ptr::null::<c_void>(), ptr::null::<c_void>());

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(res)
            }
        }
    }

    pub fn detach(&self) -> Result<i64, DebugError> {
        unsafe {
            Errno::clear();

            let res = ptrace(PTRACE_DETACH, self.pid, ptr::null::<c_void>(), ptr::null::<c_void>());

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(res)
            }
        }
    }

    pub fn getregs(&self) -> Result<T, DebugError> {

        unsafe {
            let mut regs: user_regs_struct = T::default().into();
            Errno::clear();

            ptrace(PTRACE_GETREGS, self.pid, ptr::null::<c_void>(), &mut regs);

            if errno::errno() != 0 {
                Err(DebugError::from(Errno::last()))
            } else {
                Ok(T::from(regs))
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

    pub fn setregs(&self, regs: &T) -> Result<i64, DebugError> {

        let urs: user_regs_struct = regs.clone().into();
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
}
