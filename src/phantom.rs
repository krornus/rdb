use libc::{user_regs_struct};

use registers::{Register,Cast};
use process::Process;
use error::DebugError;
use breakpoint::Breakpoint;

struct PhantomCall<T> {
    restore: T,
    exits: Vec<Breakpoint>,
}

pub struct PhantomManager<T> {
    stack: Vec<PhantomCall<T>>,
    pid: u64,
}

impl<T> PhantomManager<T>
    where T: Register + Default + Clone,
            T: ::std::fmt::Display,
            T: From<user_regs_struct> + Into<user_regs_struct>,
            <T as Register>::Size: Cast<usize>,
            usize: Cast<<T as Register>::Size>,
{
    pub fn new(pid: u64) -> Self {
        PhantomManager {
            stack: vec![],
            pid: pid,
        }
    }

    pub fn push(&mut self, restore: T, exits: Vec<u64>) {

        let breakpoints = exits.iter().map(|&ex| {
            let mut bp = Breakpoint::new(
                format!("<phantom_call_cleanup @ 0x{:x}>",ex).to_string(), self.pid as u32, ex
            ).expect("failed to set phantom breakpoint");

            bp.temporary(true);
            bp
        }).collect::<Vec<Breakpoint>>();

        let call = PhantomCall {
            exits: breakpoints,
            restore: restore,
        };

        self.stack.push(call);
    }

    pub fn clean(&mut self, process: &Process<T>) -> Result<<T as Register>::Size, DebugError> {

        if !self.is_exit(process) {
            return Err("Not at exit for phantom call".into());
        }

        /* dropped scope causes restore on instruction */
        /* don't need to waste a for-loop */
        let mut call = self.stack.pop().unwrap();

        let ip = call.restore.ip();
        call.restore.set_ip((ip.cast()-1).cast());

        process.setregs(&call.restore)?;

        process.cont()?;
        process.wait_stop()
    }

    /* FIXME: needs way more sofistication, what about recursion etc.. */
    /* thats why this takes process and not an address */
    pub fn is_exit(&self, process: &Process<T>) -> bool {

        if self.stack.len() == 0 {
            false
        } else if let Ok(regs) = process.getregs() {
            let pc = regs.ip().cast() as u64 - 1;

            let call = self.stack.last().unwrap();
            call.exits.iter().find(|&x| pc == x.addr).is_some()
        } else {
            false
        }

    }
}

