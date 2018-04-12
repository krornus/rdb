#[macro_use]
extern crate rdb;

use rdb::debugger::Debugger;
use rdb::processio::ProcessIO;

fn main() {

    let file = "./bin/test"
        .to_string();

    let mut dbg = Debugger::new(file.clone(), vec![file])
        .expect("Could not start binary");

    let main_addr = 0x004005d0;
    let foo_addr = 0x400592;
    let foo_exit = 0x4005a6;

    bp!(dbg, main_addr, name: "main::entry", enabled: true);

    dbg.run()
        .expect("couldnt run");

    /* lets get the address of the string used in foo */
    dbg.phantom_call(foo_addr, vec![], vec![foo_exit])
        .expect("failed to execute phantom call");

    /* main::entry */
    cont!(dbg);

    if let Some(o) = dbg.child.stdout() {
        println!("{}", o);
    }
}
