#[macro_use]
extern crate rdb;

use rdb::debugger::{Debugger,LogLevel};
use rdb::memory::{Memory, MemoryPack, QuerySize, Endianness};

fn main() {

    let file = "./bin/test".to_string();

    let mut dbg = Debugger::new(file.clone(), vec![file, "/bin/sh".to_string()])
        .expect("Could not start binary");

    dbg.log = LogLevel::Breakpoints | LogLevel::Commands;

    let main_addr = 0x4005d0;
    let loop_cmp = 0x4005ef;
    let main_loop = 0x4005e1;

    bp!(dbg, main_addr, name: "main::entry", enabled: true);
    bp!(dbg, loop_cmp,  name: "main::compare", enabled: true);
    bp!(dbg, main_loop, name: "main::loop", enabled: true);

    /* force program to infinite loop by always setting loop counter to 1 */
    dbg.register_action_at(loop_cmp, |dbg| {
        let regs = dbg.process.getregs()
            .expect("failed to get registers");
        let addr = regs.rbp - 0x4;

        dbg.process.poke_bits(addr, 0xdeadbeef, 4*8)
            .expect("failed to write to memory");

        println!("Set loop counter to 1");
        /* auto continue */
        cont!(dbg);
    });

    let pid = dbg.child.id() as usize;

    dbg.run()
        .expect("couldnt run");
    cont!(dbg);

    /* loop for a bit */
    cont!(dbg);
    cont!(dbg);

    let mut mem = Memory::load(pid)
        .expect("Failed to load memory");

    /* lowest address mapped in memory */
    let min = mem.min();
    /* highest address mapped in memory */
    let max = mem.max();

    let bp = dbg.process.getregs().
        expect("failed to getregs").rbp - 4;

    /* read loop counter using memory module */
    println!("Memory read: {:?}", mem.read(bp as usize, 4));

    let query = 0xdeadbeef_u32.pack(QuerySize::Length, Endianness::LittleEndian);
    println!("Query is: {:?}", query);
    let results = mem.search(min, max, query);
    for addr in results {
        println!("'0xdeadbeef' @ offset 0x{:x} (0x{:x}) in '{}'",
            addr.offset, addr.address, addr.region.pathname.unwrap_or("".to_string()));
    }

    cont!(dbg);
    cont!(dbg);

    /* search memory for set of u8 values */
    /* search values vector takes a vec![AsRef<[u8]>] */
    let results = mem.search(min, max, b"/bin/sh\x00");

    for binsh in results {
        println!("'/bin/sh' @ offset 0x{:x} (0x{:x}) in '{}'",
            binsh.offset, binsh.address, binsh.region.pathname.unwrap_or("".to_string()));
    }

    dbg.clear_actions_at(loop_cmp);

    cont!(dbg);
    cont!(dbg);
}
