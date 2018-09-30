use super::classpath::Classpath;
use super::bytecode::class::Class;
use super::bytecode::attribute::*;
use super::regex::Regex;

pub mod heap;
pub mod stack;
pub mod metaspace;

pub const NULL: Slot = [0x00, 0x00, 0x00, 0x00];

pub type Slot = [u8; 4];
pub type Slot2 = [u8; 8];

pub fn split_slot2(w: Slot2) -> (Slot, Slot) {
    let mut higher = [0u8; 4];
    higher.copy_from_slice(&w[0..4]);
    let mut lower = [0u8; 4];
    lower.copy_from_slice(&w[4..]);
    (higher, lower)
}
