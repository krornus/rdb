//#![feature(test)]
#![feature(generic_associated_types)]
extern crate libc;
extern crate nix;
extern crate posix;
extern crate spawn_ptrace;
extern crate regex;
extern crate twox_hash;
extern crate memmap;
extern crate vm_info;
extern crate twoway;
//extern crate test;

#[macro_use] extern crate bitflags;

pub mod debugger;
pub mod registers;
pub mod error;
pub mod processio;
pub mod breakpoint;
pub mod process;
pub mod status;
pub mod memory;
mod phantom;

/*
#[cfg(test)]
mod tests {

    use super::*;
    use test::Bencher;

    #[bench]
    fn memory(b: &mut Bencher) {

        let mut dbg = debugger::Debugger::new::<&str>(
            "./bin/test", vec!["./bin/test", "/bin/sh"]
        ).expect("Could not start binary");

        dbg.breakpoint(0x4005d0)
            .expect("Failed to set breakpoint");

        let pid = dbg.child.id() as usize;

        let mut mem = memory::Memory::load(pid)
            .expect("Failed to load memory");

        let min = mem.min();
        let max = mem.max();

        dbg.run()
            .expect("couldnt run");

        b.iter(|| mem.search(min, max, b"/bin/sh\x00"));
    }
}

*/
