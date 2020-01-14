use bytecode::class::Class;
use chashmap::CHashMap;
use classpath::Classpath;
use regex::Regex;

#[macro_use]
pub mod heap;
#[macro_use]
pub mod metaspace;
pub mod stack;

// pub const PTR_SIZE: usize = std::mem::size_of::<usize>();

// 32-bit vm
pub const PTR_SIZE: usize = 4;
pub const NULL: Slot = [0x00; PTR_SIZE];

pub type Slot = [u8; PTR_SIZE];
pub type WideSlot = (Slot, Slot);
pub type Word = [u8; PTR_SIZE];


#[derive(Copy, Clone, Debug)]
pub enum Value {
    Word(Word),
    DWord(Word, Word),
}

pub trait Memorizable<T> {
    fn memorized(&self) -> T;
}

impl Memorizable<Slot> for i32 {
    fn memorized(&self) -> Slot {
        unsafe { std::mem::transmute::<i32, [u8; 4]>(*self) }
    }
}

impl Memorizable<WideSlot> for i64 {
    fn memorized(&self) -> WideSlot {
        unsafe {
            let bs = std::mem::transmute::<i64, [u8; 8]>(*self);
            let lower = [0u8; PTR_SIZE];
            let higher = [0u8; PTR_SIZE];
            (lower, higher)
        }
    }
}

impl Memorizable<Slot> for f32 {
    fn memorized(&self) -> Slot {
        unsafe { std::mem::transmute::<f32, [u8; 4]>(*self) }
    }
}

impl Memorizable<WideSlot> for f64 {
    fn memorized(&self) -> WideSlot {
        unsafe {
            let bs = std::mem::transmute::<f64, [u8; 8]>(*self);
            let lower = [0u8; PTR_SIZE];
            let higher = [0u8; PTR_SIZE];
            (lower, higher)
        }
    }
}

pub struct ObjectHeader {
    head: Word,
    klass: std::sync::Arc<metaspace::Klass>,
    array_info: Option<Word>,
    // payload: Vec<u8>,
}

#[test]
fn memorize_i32() {
    let i = (256 as i32).memorized();
    assert_eq!(&i, &[0u8, 1, 0, 0]);
}
