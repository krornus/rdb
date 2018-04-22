#[macro_use]
extern crate rdb;

use rdb::debugger::{Debugger,LogLevel};
use rdb::memory;
use rdb::processio::ProcessIO;

fn main() {

    let file = "./bin/test"
        .to_string();

    let mut dbg = Debugger::new(file.clone(), vec![file])
        .expect("Could not start binary");

    dbg.log = LogLevel::Commands | LogLevel::Breakpoints;

    let main_addr = 0x4005d0;

    bp!(dbg, main_addr, name: "main::entry", enabled: true);

    dbg.run()
        .expect("couldnt run");

    let pid = dbg.child.id();
    let mut mem = memory::Memory::load(pid as usize)
        .expect("Failed to load memory");

    mem.write(0x4006ce, b"rust!\x00")
        .expect("failed to write to memory");
    /* main::entry */
    cont!(dbg);



    if let Some(o) = dbg.child.stdout() {
        println!("{}", o);
    }
}
