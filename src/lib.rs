#![feature(weak_into_raw)]
#![feature(map_first_last)]
pub mod bytecode;
pub mod classpath;
#[macro_use]
pub mod mem;
#[macro_use]
pub mod interpreter;
pub mod gc;
