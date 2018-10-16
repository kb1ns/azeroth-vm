use super::bytecode::class::Class;
use super::classpath::Classpath;
use super::regex::Regex;
use std;

pub mod heap;
pub mod metaspace;
pub mod stack;

pub const NULL: Slot = [0x00, 0x00, 0x00, 0x00];

pub type Slot = [u8; 4];
pub type Slot2 = [u8; 8];
pub type Word = [u8; 4];
pub type DWord = [u8; 8];

pub fn split_slot2(w: Slot2) -> (Slot, Slot) {
    let mut higher = [0u8; 4];
    higher.copy_from_slice(&w[0..4]);
    let mut lower = [0u8; 4];
    lower.copy_from_slice(&w[4..]);
    (higher, lower)
}

pub struct Object {
    head: Word,
    klass: std::sync::Arc<self::metaspace::Klass>,
    array_info: Option<Word>,
    payload: Vec<u8>,
}

impl Object {
    fn padding(&mut self) {}
}
