#[macro_use]
extern crate rdb;

use rdb::debugger::{Debugger,LogLevel};
use rdb::processio::ProcessIO;


fn main() {

    let file = "./bin/test"
        .to_string();

    /* create debugger for file, ran with arguments [file] */
    let mut dbg = Debugger::new(file.clone(), vec![file])
        .expect("Could not start binary");

    /* set log level to show commands and breakpoints */
    dbg.log = LogLevel::Commands | LogLevel::Breakpoints;

    let main_addr = 0x4005d0;
    let foo_addr = 0x400592;
    let foo_exit = 0x4005a6;

    /* add a breakpoint in main */
    dbg.breakpoint(main_addr)
        .expect("failed to set breakpoint")
        .name("main::entry");

    /* run the debugged process */
    dbg.run()
        .expect("couldnt run");

    /* we are now at the first and only breakpoint */
    /* get the address of the string used in the function "foo" */
    dbg.phantom_call(foo_addr, vec![], vec![foo_exit])
        .expect("failed to execute phantom call");

    /* continue */
    cont!(dbg);

    /* print output */
    if let Some(o) = dbg.child.stdout() {
        println!("{}", o);
    }
}
