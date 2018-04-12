use spawn_ptrace::CommandPtraceSpawn;

use std::process::{Child,Stdio,Command};
use std::collections::HashMap;
use std::result::Result;
use std::boxed::Box;

use std::rc::Rc;
use std::cell::RefCell;

use breakpoint::Breakpoint;
use process::Process;
use status::Status;
use registers::Registers;
use error::DebugError;

#[macro_export]
macro_rules! pc {
    ($child:expr) => {
        {
            let regs = $child.getregs()
                .expect("pc! failed to get registers");

            regs.rip
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


type CellMap<K,V> = Rc<RefCell<HashMap<K,V>>>;
type CellVec<T> = Rc<RefCell<Vec<T>>>;
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
    pub process: Process,
    pub breakpoints: HashMap<u64,Breakpoint>,
    pub file: String,
    pub args: Vec<String>,
    pub child: Child,
    pub pc: OptionCell<u64>,
    pub log: LogLevel,
    actions_at: CellMap<u64,Vec<BoxedDebuggerFn>>,
    actions: CellVec<BoxedDebuggerFn>,
    init_state: bool,
}

impl Debugger {

    pub fn new(binary: String, args: Vec<String>)
        -> Result<Self,DebugError>
    {

        let child = Debugger::spawn(binary.clone(), args.clone())?;
        let process = Process::new(child.id());
        let pc = pc!(process);

        let d = Debugger {
            process: process,
            breakpoints: HashMap::new(),
            actions: Rc::new(RefCell::new(vec![])),
            file: binary,
            args: args,
            child: child,
            init_state: false,
            log: LogLevel::Silent,
            pc: Rc::new(RefCell::new(Some(pc))),
            actions_at: Rc::new(RefCell::new(HashMap::new())),
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

    fn spawn(fcn: String, args: Vec<String>) -> Result<Child, DebugError> {

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

        let name = self.breakpoints.len().to_string();
        let bp = Breakpoint::new(name, pid, addr)?;

        self.log_command(&format!("set breakpoint {} @ 0x{:x}", self.breakpoints.len(), addr));
        self.breakpoints.insert(addr+1, bp);

        Ok(self.breakpoint_at_mut(addr+1).unwrap())
    }

    pub fn cont(&self) -> Result<Option<u64>, DebugError> {

        self.log_command("continue");

        let pc = {
            let bp = self.current_breakpoint()?;

            match bp.cont() {
                Ok(pc) => pc,
                Err(DebugError::Status(stat)) => {
                    return self.handle_status(stat);
                },
                Err(x) => {
                    return Err(x);
                }
            }
        };


        self.set_pc(pc);
        self.log_breakpoint();
        self.on_break();

        Ok(Some(pc))
    }


    pub fn phantom_call(&mut self, addr: u64, args: Vec<u64>, exits: Vec<u64>)
        -> Result<Option<u64>, DebugError>
    {

        self.log_command(&format!("phantom call function @ 0x{:x}", addr));

        let arg_regs = Registers::from_process(self, args)?;

        for ex in exits {
            self.breakpoint(ex)
                .expect("failed to set breakpoint")
                .name("<phantom_call cleanup>");

            self.register_action_at(ex, |dbg| {
                dbg.cont()
                    .expect("failed to continue from breakpoint");
            });
        }

        /* set args */
        self.process.setregs(&arg_regs)?;

        let pc = self.current_breakpoint()?
            .phantom_call(addr)?;

        self.set_pc(pc);
        self.log_breakpoint();
        self.on_break();

        Ok(Some(pc))
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

        self.breakpoints.get(&addr)
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

        self.breakpoints.get_mut(&addr)
    }

    fn log_breakpoint(&self) {
        if self.log.contains(LogLevel::Breakpoints) {
            if let Ok(hit) = self.current_breakpoint() {
                println!("0x{:x}: Encountered breakpoint {}", self.pc.borrow().unwrap(), hit.name);
            }
        }
    }

    fn log_command<'actions_at>(&self, cmd: &'actions_at str) {
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
        let pc = match *self.pc.borrow() {
            Some(pc) => pc.clone(),
            None => { return; },
        };

        match self.actions_at.borrow().get(&pc) {
            Some(functions) => {
                for fct in functions {
                    fct(&self);
                }
            },
            None => {},
        };

        for fct in self.actions.borrow().iter() {
            fct(&self);
        }
    }

    pub fn register_action_at<F>(&self, addr: u64, fct: F)
        where F: Fn(&Debugger),
        F: 'static
    {
            self.actions_at.borrow_mut().entry(addr+1)
                .or_insert(vec![])
                .push(Box::new(fct));
    }

    pub fn register_action<F>(&self, fct: F)
        where F: Fn(&Debugger),
        F: 'static
    {
            self.actions.borrow_mut().push(Box::new(fct));
    }
}

