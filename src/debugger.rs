use spawn_ptrace::CommandPtraceSpawn;

use std::process::{Child,Stdio,Command};
use std::collections::HashMap;
use std::result::Result;
use std::cell::RefCell;
use std::rc::Rc;
use std::boxed::Box;
use std::ffi::OsStr;

use breakpoint::Breakpoint;
use process::Process;
use status::Status;
use registers::{Register,x86_64_Registers};
use error::DebugError;
use phantom::PhantomManager;

#[macro_export]
macro_rules! pc {
    ($child:expr) => {
        {
            let regs = $child.getregs()
                .expect("pc! failed to get registers");

            regs.ip()
        }
    }
}

#[macro_export]
macro_rules! bp {
    ($dbg: expr, $addr:expr $(, name: $name:expr)* $(, enabled: $enabled:expr)*) => {
        $dbg.breakpoint($addr)
            .expect("failed to set breakpoint")
            $(.name($name))*
            $(.enabled($enabled)
            .expect("failed to disable breakpoint"))*;
    }
}

#[macro_export]
macro_rules! cont {
    ($dbg: expr) => {
            $dbg.cont()
                .expect("failed to continue");
    }
}

type OptionCell<T> = Rc<RefCell<Option<T>>>;
type BoxedDebuggerFn = Box<Fn(&Debugger)>;

bitflags! {
    pub struct LogLevel: u32 {
        #[allow(non_upper_case_globals)]
        const Breakpoints = 0b01;
        #[allow(non_upper_case_globals)]
        const Commands = 0b10;
        #[allow(non_upper_case_globals)]
        const Silent = 0b00;
    }
}


pub struct Debugger {
    pub process: Process<x86_64_Registers>,
    pub breakpoints: HashMap<u64,Breakpoint>,
    pub file: String,
    pub args: Vec<String>,
    pub child: Child,
    pub pc: OptionCell<u64>,
    pub log: LogLevel,
    phantom_mgr: Rc<RefCell<PhantomManager<x86_64_Registers>>>,
    actions_at: HashMap<u64,Vec<BoxedDebuggerFn>>,
    actions: Vec<BoxedDebuggerFn>,
    init_state: bool,
}

impl Debugger {

    pub fn new<T: AsRef<OsStr> + Clone>(binary: T, args: Vec<T>)
        -> Result<Self,DebugError>
    {

        let child = Debugger::spawn(binary.clone(), args.clone())?;
        let pid = child.id();
        let process = Process::<x86_64_Registers>::new(pid);
        let pc = pc!(process);

        let args = args.into_iter().map(|s|
            s.as_ref().to_string_lossy().into_owned()
        ).collect();

        let d = Debugger {
            process: process,
            breakpoints: HashMap::new(),
            actions: vec![],
            file: binary.as_ref().to_string_lossy().into_owned(),
            args: args,
            child: child,
            phantom_mgr: Rc::new(RefCell::new(PhantomManager::new(pid.into()))),
            init_state: false,
            log: LogLevel::Silent,
            pc: Rc::new(RefCell::new(Some(pc))),
            actions_at: HashMap::new(),
        };

        Ok(d)
    }

    pub fn run(&mut self) -> Result<String, DebugError> {

        self.log_command(&format!("running binary '{}' with argc {}", self.file, self.args.len()));

        self.process.cont()?;

        let pc = {
            /* returns pc on success */
            match self.process.wait_stop() {
                Ok(x) => x,
                Err(DebugError::Status(stat)) => {
                    if stat.exited() {
                        /* ran and exited without hitting breakpoints */
                        return Ok(self.file.clone());
                    } else {
                        /* ran and errored out (segfault?) */
                        return Err(DebugError::from(stat));
                    }
                },
                Err(x) => {
                    return Err(x);
                }
            }
        };

        self.set_pc(pc);

        self.log_breakpoint();
        self.on_break();
        self.init_state = false;

        Ok(self.file.clone())
    }

    fn set_pc(&self, pc: u64) {
        *self.pc.borrow_mut() = Some(pc);
    }

    fn spawn<T: AsRef<OsStr>>(fcn: T, args: Vec<T>) -> Result<Child, DebugError> {

        match Command::new(fcn)
            .args(&args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn_ptrace() {

            Ok(s) => {
                Ok(s)
            },
            Err(e) => {
                Err(DebugError::from(e))
            }
        }

    }

    pub fn breakpoint(&mut self, addr: u64) -> Result<&mut Breakpoint, DebugError> {

        let pid = self.process.pid;

        let name = (self.breakpoints.len()+1).to_string();
        let bp = Breakpoint::new(name, pid, addr)?;

        self.log_command(&format!("set breakpoint {} @ 0x{:x}", self.breakpoints.len(), addr));
        self.breakpoints.insert(addr+1, bp);

        Ok(self.breakpoint_at_mut(addr+1).unwrap())
    }

    pub fn tmp_breakpoint(&mut self, addr: u64) -> Result<&mut Breakpoint, DebugError> {

        let pid = self.process.pid;

        let name = self.breakpoints.len().to_string();
        let bp = Breakpoint::new(name, pid, addr)?;

        self.log_command(&format!("set breakpoint {} @ 0x{:x}", self.breakpoints.len(), addr));
        self.breakpoints.insert(addr+1, bp);

        Ok(self.breakpoint_at_mut(addr+1).unwrap())
    }

    pub fn cont(&self) -> Result<Option<u64>, DebugError> {

        self.log_command("continue");

        let pc = match self.current_breakpoint() {
            Ok(bp) => {
                match bp.cont() {
                    Ok(pc) => {
                        pc
                    },
                    Err(DebugError::Status(stat)) => {
                        return self.handle_status(stat);
                    },
                    Err(x) => {
                        return Err(x);
                    }
                }
            },
            Err(_) => {
                match self.non_bp_cont()? {
                    Some(ip) => ip,
                    None => { return Ok(None); }
                }
            }
        };

        self.set_pc(pc);
        self.log_breakpoint();
        self.on_break();

        Ok(Some(pc))
    }

    fn non_bp_cont(&self) -> Result<Option<u64>, DebugError> {
        println!(" (not at breakpoint)");
        self.process.cont()?;
        let ip = self.process.wait_stop()?;

        Ok(Some(ip))
    }

    pub fn single_step(&self) -> Result<Option<u64>, DebugError> {
        self.log_command("single step");

        self.process.step()?;
        let ip = self.process.wait_stop()?;
        self.log_breakpoint();
        self.on_break();
        Ok(Some(ip))
    }

    pub fn phantom_call(&mut self, addr: u64, args: Vec<u64>, exits: Vec<u64>)
        -> Result<Option<u64>, DebugError>
    {

        self.log_command(&format!("phantom call function @ 0x{:x}", addr));

        let arg_regs = x86_64_Registers::from_process(self, args)?;
        let reset = self.process.getregs()?;

        self.phantom_mgr.borrow_mut().push(reset, exits);

        /* set args */
        self.process.setregs_user(&arg_regs)?;

        let pc = self.current_breakpoint()?
            .phantom_call(addr)?;

        self.set_pc(pc);
        self.log_breakpoint();
        /* callbacks might continue */
        self.on_break();


        Ok(Some(pc))
    }

    pub fn at_breakpoint(&self) -> bool {
        if let Some(pc) = *self.pc.borrow() {
            match self.breakpoint_at(pc) {
                Some(_) => true,
                None => false,
            }
        } else {
            false
        }
    }

    pub fn current_breakpoint(&self) -> Result<&Breakpoint, DebugError> {

        if let Some(pc) = *self.pc.borrow() {
            match self.breakpoint_at(pc) {
                Some(x) => Ok(x),
                None => Err(DebugError::from("No breakpoint found at given address")),
            }
        } else {
            Err(DebugError::from("program is not currently running"))
        }
    }

    pub fn breakpoint_at(&self, addr: u64) -> Option<&Breakpoint> {

        if let Some(bp) = self.breakpoints.get(&addr) {
            if bp.is_enabled() || bp.is_temporary() {
                Some(bp)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn current_breakpoint_mut(&mut self) -> Result<&mut Breakpoint, String> {

        let ip = self.pc.borrow().clone();

        if let Some(pc) = ip {
            match self.breakpoint_at_mut(pc) {
                Some(x) => Ok(x),
                None => Err("No breakpoint found at given address".to_string()),
            }
        } else {
            Err("program is not currently running".to_string())
        }
    }

    pub fn breakpoint_at_mut(&mut self, addr: u64) -> Option<&mut Breakpoint> {

        if let Some(bp) = self.breakpoints.get_mut(&addr) {
            if bp.is_enabled() {
                Some(bp)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn log_breakpoint(&self) {
        if self.log.contains(LogLevel::Breakpoints) {
            if let Ok(hit) = self.current_breakpoint() {
                println!("0x{:x}: Encountered breakpoint {}", hit.addr, hit.name);
            }
        }
    }

    fn log_command<'a>(&self, cmd: &'a str) {
        if self.log.contains(LogLevel::Commands) {
            println!("{}",cmd);
        }
    }

    fn handle_status(&self, status: Status) -> Result<Option<u64>, DebugError> {

        if status.exited() {
            Ok(None)
        } else if status.trapped() {
            println!("at 0x{:x}, available breakpoints:", pc!(self.process));
            for (_,bp) in self.breakpoints.iter() {
                println!("\t'{}' -> 0x{:x}", bp.name, bp.addr);
            }
            Err(DebugError::from("recieved SIGTRAP but not at actions_at breakpoint!"))
        } else {
            Err(DebugError::from("uknown error"))
        }

    }

    fn on_break(&self) {
        let process = &self.process;

        if self.phantom_mgr.borrow().is_exit(process) {
            let pc =self.phantom_mgr.borrow_mut().clean(process)
                .expect("Failed to reset process after phantom call");
            self.set_pc(pc);
        }

        let pc = match *self.pc.borrow() {
            Some(pc) => pc.clone(),
            None => { return; },
        };

        match self.actions_at.get(&pc) {
            Some(functions) => {
                for fct in functions {
                    fct(&self);
                }
            },
            None => {},
        };

        for fct in self.actions.iter() {
            fct(&self);
        }
    }

    pub fn register_action_at<F>(&mut self, addr: u64, fct: F)
        where F: Fn(&Debugger),
        F: 'static
    {
            self.actions_at.entry(addr+1)
                .or_insert(vec![])
                .push(Box::new(fct));
    }

    pub fn clear_actions_at(&mut self, addr: u64) {
            self.actions_at.remove(&(addr+1));
    }

    pub fn register_action<F>(&mut self, fct: F)
        where F: Fn(&Debugger),
        F: 'static
    {
            self.actions.push(Box::new(fct));
    }
}

