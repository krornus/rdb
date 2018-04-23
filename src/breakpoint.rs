use std::rc::Rc;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};

use process::Process;
use error::DebugError;
use registers::{Register,x86_64_Registers};

#[derive(Debug,Clone)]
pub struct Breakpoint {
    pub addr: u64,
    pub name: String,
    enabled: Rc<RefCell<bool>>,
    temporary: bool,
    process: Process<x86_64_Registers>,
    restore: u64,
}

impl Hash for Breakpoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
        self.name.hash(state);
        self.process.pid.hash(state);
    }
}

impl PartialEq for Breakpoint {
    fn eq(&self, other: &Breakpoint) -> bool {
        self.addr == other.addr &&
        self.name == other.name &&
        self.process.pid == other.process.pid
    }
}
impl Eq for Breakpoint { }

impl Drop for Breakpoint {
    fn drop(&mut self) {
        if *self.enabled.borrow() && self.process.status().running() {
            match self.restore() {
                Ok(_) => println!("dropped breakpoint '{}' @ 0x{:x}", self.name, self.addr),
                Err(_) => println!("failed to drop breakpoint '{}' @ 0x{:x}", self.name, self.addr),
            }
        }
    }
}

impl Breakpoint {
    pub fn new(name: String, pid: u32, addr: u64) -> Result<Breakpoint, DebugError> {

        let process = Process::new(pid);

        let mut bp = Breakpoint {
            process: process,
            addr: addr,
            restore: 0,
            enabled: Rc::new(RefCell::new(true)),
            temporary: false,
            name: name,
        };

        let data = bp.trap()?;
        bp.restore = data;

        Ok(bp)
    }

    pub fn finish(&mut self) -> &Breakpoint {
        self
    }

    #[inline]
    pub fn name(&mut self, name: &'static str) -> &mut Breakpoint {
        self.name = name.to_string();
        self
    }

    pub fn trap(&self) -> Result<u64, DebugError> {

        let data = self.process.peek(self.addr)?;

        self.process.poke_bits(self.addr,0xcc,8)?;
        *self.enabled.borrow_mut() = true;

        Ok(data)
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.borrow()
    }

    pub fn is_temporary(&self) -> bool {
        self.temporary
    }

    pub fn enabled(&mut self, e: bool) -> Result<&mut Breakpoint, DebugError> {

        if e {
            self.trap()?;
        } else {
            self.process.poke(self.addr, self.restore)?;
        }

        *self.enabled.borrow_mut() = e;

        Ok(self)
    }

    pub fn temporary(&mut self, e: bool) -> &mut Breakpoint {
        self.temporary = e;

        if self.is_enabled() && e {
            *self.enabled.borrow_mut() = false;
        }

        self
    }

    pub fn restore(&self) -> Result<u64, DebugError> {

        self.process.poke(self.addr, self.restore)?;
        let addr = self.addr;
        self.set_ip(addr)?;
        *self.enabled.borrow_mut() = false;

        Ok(self.addr)
    }

    pub fn cont(&self) -> Result<u64, DebugError> {

        /*
         *  Do a restore, step, trap, continue
         *  return Status if the program stops
         */

        /* restore instruction, set pc to pc - 1 */
        self.restore()?;

        if !self.temporary {
            /* execute restored instruction */
            /* the process will be sigtrapped */
            self.process.step()?;
            self.process.wait_stop()?;

            /* re-trap instruction */
            self.trap()?;
        }

        /* continue */
        self.process.cont()?;
        self.process.wait_stop()
    }

    pub fn restore_to(&self, addr: u64) -> Result<u64, DebugError> {

        self.process.poke(self.addr, self.restore)?;
        self.set_ip(addr)?;

        Ok(addr)
    }

    pub fn jump_to(&self, addr: u64) -> Result<u64, DebugError> {

        self.restore_to(addr)?;
        self.process.cont()?;

        Ok(addr)
    }

    pub fn phantom_call(&self, addr: u64) -> Result<u64, DebugError> {

        /* jump to function */
        self.set_ip(addr)?;

        /* continue */
        self.process.cont()?;
        self.process.wait_stop()
    }

    fn set_ip(&self, addr: u64) -> Result<u64, DebugError> {
        let mut regs = self.process.getregs()?;
        regs.set_ip(addr);
        self.process.setregs(&regs)?;

        Ok(regs.ip())
    }
}


