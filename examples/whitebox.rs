#[macro_use]
extern crate rdb;

use std::fs::File;
use std::io::Read;
use std::env;

use rdb::debugger::{Debugger,LogLevel};
use rdb::memory;
use rdb::processio::ProcessIO;

fn main() {

    let file = "./bin/whitebox"
        .to_string();

    let mut table = Vec::new();

    File::open("./bin/whitebox.table")
        .expect("Failed to open table")
        .read_to_end(&mut table)
        .expect("Failed to read table");

    let args = env::args().skip(1).collect();

    let mut dbg = Debugger::new(file.clone(), args)
        .expect("Could not start binary");

    let load_addr = 0x400823;
    bp!(dbg, load_addr, name: "main::entry", enabled: true);

    dbg.run()
        .expect("couldnt run");

    let pid = dbg.child.id();

    let mut mem = memory::Memory::load(pid as usize)
        .expect("Failed to load memory");
    mem.write(0x6650c0, &table)
        .expect("failed to write to memory");

    /* main::entry */
    cont!(dbg);

    if let Some(o) = dbg.child.stdout() {
        println!("{}", o.trim());
    }
}
