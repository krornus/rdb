use process::Process;
use error::DebugError;
use registers::Register;

#[derive(Debug,Clone)]
pub struct Breakpoint {
    pub addr: u64,
    pub name: String,
    enabled: bool,
    process: Process,
    restore: u64,
}

impl Breakpoint {
    pub fn new(name: String, pid: u32, addr: u64) -> Result<Breakpoint, DebugError> {

        let process = Process::new(pid);

        let mut bp = Breakpoint {
            process: process,
            addr: addr,
            restore: 0,
            enabled: true,
            name: name,
        };

        let data = bp.trap()?;

        bp.restore = data;

        Ok(bp)
    }

    #[inline]
    pub fn name(&mut self, name: &'static str) -> &mut Breakpoint {
        self.name = name.to_string();
        self
    }

    pub fn trap(&self) -> Result<u64, DebugError> {
        let data = self.process.peek(self.addr)?;
        let trap = (data & !0xff) | 0xcc;
        self.process.poke(self.addr,trap)?;

        Ok(data)
    }

    pub fn enabled(&mut self, e: bool) -> Result<&mut Breakpoint, DebugError> {

        if e {
            self.trap()?;
            self.enabled = true;
        } else {
            self.process.poke(self.addr, self.restore)?;
            self.enabled = false;
        }

        Ok(self)
    }

    pub fn restore(&self) -> Result<u64, DebugError> {

        self.process.poke(self.addr, self.restore)?;
        self.process.setreg(Register::rip, self.addr)?;

        Ok(self.addr)
    }

    pub fn cont(&self) -> Result<u64, DebugError> {

        /*
         *  Do a restore, step, trap, continue
         *  return Status if the program stops
         */

        /* restore instruction, set pc to pc - 1 */
        self.restore()?;

        /* execute restored instruction */
        /* the process will be sigtrapped */
        self.process.step()?;
        self.process.wait_stop()?;

        /* re-trap instruction */
        self.trap()?;

        /* continue */
        self.process.cont()?;
        self.process.wait_stop()
    }

    pub fn restore_to(&self, addr: u64) -> Result<u64, DebugError> {

        self.process.poke(self.addr, self.restore)?;
        self.process.setreg(Register::rip, addr)?;

        Ok(addr)
    }

    pub fn jump_to(&self, addr: u64) -> Result<u64, DebugError> {

        self.restore_to(addr)?;
        self.process.cont()?;

        Ok(addr)
    }

    pub fn phantom_call(&self, addr: u64) -> Result<u64, DebugError> {

        let pc = self.addr;

        /* push return address (this breakpoint again) */
        self.process.push(pc)?;
        /* jump to function */
        self.process.setreg(Register::rip, addr)?;

        /* continue */
        self.process.cont()?;
        self.process.wait_stop()
    }

}


