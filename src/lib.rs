extern crate libc;
extern crate nix;
extern crate posix;
extern crate spawn_ptrace;
extern crate regex;
extern crate twox_hash;
extern crate memmap;
extern crate vm_info;
extern crate twoway;

#[macro_use] extern crate bitflags;

pub mod debugger;
pub mod registers;
pub mod error;
pub mod processio;
pub mod breakpoint;
pub mod process;
pub mod status;
pub mod memory;
//mod manager;
mod phantom;

#[cfg(test)]
mod tests {
}

