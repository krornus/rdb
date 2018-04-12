#[macro_use]
extern crate rdb;

use rdb::debugger::{Debugger,LogLevel};
use rdb::processio::ProcessIO;

fn main() {

    let file = "./bin/test".to_string();
    let mut dbg = Debugger::new(file.clone(), vec![file])
        .expect("Could not start binary");
    dbg.log = LogLevel::Breakpoints | LogLevel::Commands;

    dbg.register_action(|dbg| {
        println!("==========================================");
        println!("{}", dbg.process.getregs().unwrap());
        println!("==========================================");
    });

    let main_addr = 0x4005a7;

    let main_retn = 0x4005dc;
    let puts_addr = 0x4005fa;
    let main_loop =  0x4005b8;

    let bar_addr = 0x4005a7;
    let bar_exit = 0x4005cf;

    let foo_addr = 0x400592;
    let foo_str  = 0x40059b;
    let _foo_printf = 0x4005a0;
    let foo_exit = 0x4005a6;

    bp!(dbg, main_addr, name: "main::entry", enabled: false);
    bp!(dbg, main_retn, name: "main::retn", enabled: false);
    bp!(dbg, main_loop, name: "main::loop", enabled: false);
    bp!(dbg, foo_exit, name: "foo::exit", enabled: false);
    bp!(dbg, puts_addr, name: "main::puts", enabled: true);
    bp!(dbg, foo_addr, name: "foo::entry", enabled: true);
    bp!(dbg, foo_str, name: "foo::str", enabled: true);
    bp!(dbg, bar_addr, name: "bar::entry", enabled: true);

    dbg.run()
        .expect("couldnt run");

    /* lets get the address of the string used in foo */
    dbg.phantom_call(foo_addr, vec![], vec![foo_exit])
        .expect("failed to execute phantom call");

    /* foo::entry */
    cont!(dbg);

    let regs = dbg.process.getregs()
        .expect("failed to getregs");
    let foo_str_addr = regs.rdi;

    /* foo::str */
    cont!(dbg);

    /* cleanup registers before leaving phantom_call_cleanup */
    dbg.process.setregs_user(regs)
      .expect("failed to setregs");


    dbg.phantom_call(bar_addr, vec![0xdeadbeef, foo_str_addr], vec![bar_exit])
        .expect("failed to execute phantom call");

    /* bar::entry */
    cont!(dbg);

    /* cleanup registers before leaving phantom_call_cleanup */
    dbg.process.setregs_user(regs)
        .expect("failed to setregs");

    /* at breakpoint main::puts */
    cont!(dbg);

    if let Some(o) = dbg.child.stdout() {
        println!("{}", o);
    }

}
