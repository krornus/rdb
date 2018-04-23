use std::collections::HashMap;

use debugger::Debugger;
use error::DebugError;
use breakpoint::Breakpoint;

pub struct Manager {
    pid: usize,
    pc: Option<u64>,
    global_actions: Vec<fn(&Debugger)>,
    bp_actions: HashMap<u64, Vec<fn(&Debugger)>>,
    breakpoints: HashMap<u64, Breakpoint>,
    phantom_stack: Vec<Vec<u64>>,
}

impl Manager {

    pub fn new(pid: usize, pc: Option<u64>) -> Self {
        Manager {
            pid: pid,
            pc: pc,
            global_actions: vec![],
            bp_actions: HashMap::new(),
            breakpoints: HashMap::new(),
            phantom_stack: vec![],
        }
    }

    pub fn set_pc(&mut self, pc: u64) {
        self.pc = Some(pc);
    }

    pub fn clear_pc(&mut self) {
        self.pc = None;
    }

    pub fn phantom_push(&mut self, exits: Vec<u64>) {
        self.phantom_stack.push(exits);
    }

    pub fn phantom_cleanup(&mut self, exits: Vec<u64>) {
        self.phantom_stack.push(exits);
    }

    pub fn add(&mut self, addr: u64) -> Result<&mut Breakpoint, DebugError> {

        let name = format!("breakpoint {}", self.breakpoints.len());
        let bp = Breakpoint::new(name, self.pid as u32, addr)?;

        if self.breakpoints.contains_key(&addr) {
            Err(DebugError::from("Breakpoint already exists at given address"))
        } else {
            self.breakpoints.insert(addr, bp);
            Ok(self.at_mut(addr).unwrap())
        }
    }

    pub fn current(&self) -> Result<Option<&Breakpoint>, DebugError> {
        if let Some(pc) = self.pc {
            match self.at(pc-1) {
                Some(bp) => Ok(Some(bp)),
                None => Err(DebugError::from("No breakpoint at current address"))
            }
        } else {
            Err(DebugError::from("Program is not currently running"))
        }
    }

    pub fn current_mut(&mut self) -> Result<Option<&mut Breakpoint>, DebugError> {
        if let Some(pc) = self.pc {
            match self.at_mut(pc-1) {
                Some(bp) => Ok(Some(bp)),
                None => Err(DebugError::from("No breakpoint at current address"))
            }
        } else {
            Err(DebugError::from("Program is not currently running"))
        }
    }

    pub fn at(&self, addr: u64) -> Option<&Breakpoint> {
        self.breakpoints.get(&addr)
    }

    pub fn at_mut(&mut self, addr: u64) -> Option<&mut Breakpoint> {
        self.breakpoints.get_mut(&addr)
    }

    pub fn register_action(&mut self, fcn: fn(&Debugger)) {
        self.global_actions.push(fcn);
    }

    pub fn register_action_at(&mut self, bp: u64, fcn: fn(&Debugger)) {
        self.bp_actions.entry(bp).or_insert(vec![]).push(fcn);
    }

    pub fn actions(&self, dbg: &Debugger) {

        if let Ok(Some(bp)) = self.current() {
            self.bp_actions.get(&bp.addr)
                .unwrap_or(&vec![]).iter().map(|fct| {
                    fct(dbg);
                }).collect::<Vec<_>>();
        }

        for fct in self.global_actions.iter() {
            fct(dbg);
        }

    }
}
