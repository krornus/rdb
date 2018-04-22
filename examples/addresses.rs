#[macro_use]
extern crate rdb;

use rdb::debugger::{Debugger,LogLevel};
use rdb::processio::ProcessIO;

fn main() {

    let file = "./bin/test".to_string();

    let mut dbg = Debugger::new(file.clone(), vec![file])
        .expect("Could not start binary");

    dbg.log = LogLevel::Breakpoints | LogLevel::Commands;
    //dbg.log = LogLevel::Silent;

    //dbg.register_action(|dbg| {
    //    println!("==========================================");
    //    println!("{}", dbg.process.getregs().unwrap());
    //    println!("==========================================");
    //});

    let puts_addr = 0x4005fa;
    let main_exit = 0x400605;

    let bar_addr = 0x4005a7;
    let bar_exit = 0x4005cf;

    let foo_addr = 0x400592;
    let foo_str  = 0x40059b;
    let foo_exit = 0x4005a6;

    bp!(dbg, puts_addr, name: "main::puts");
    bp!(dbg, main_exit, name: "main::exit");
    bp!(dbg, foo_addr, name: "foo::entry");
    bp!(dbg, foo_str, name: "foo::str");
    bp!(dbg, bar_addr, name: "bar::entry");

    dbg.run()
        .expect("couldnt run");

    /* break main::puts */
    /* lets get the address of the string used in foo */
    dbg.phantom_call(foo_addr, vec![], vec![foo_exit])
        .expect("failed to execute phantom call");

    /* break foo::entry */
    cont!(dbg);

    let regs = dbg.process.getregs()
        .expect("failed to getregs");
    let foo_str_addr = regs.rdi;

    /* break foo::str */
    dbg.phantom_call(bar_addr, vec![0xdeadbeef, foo_str_addr], vec![bar_exit])
        .expect("failed to execute phantom call");

    /* bar::entry */
    cont!(dbg);

    /* cleanup phantom call */
    /* foo::str */
    cont!(dbg);
    /* cleanup phantom call */
    /* main::exit */
    cont!(dbg);
    cont!(dbg);


    if let Some(o) = dbg.child.stdout() {
        println!("{}",o);
    }

}
