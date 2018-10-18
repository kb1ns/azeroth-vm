use super::bytecode::class::Class;
use super::classpath::Classpath;
use super::regex::Regex;
use std;

pub mod heap;
pub mod metaspace;
pub mod stack;

pub const PTR_SIZE: usize = std::mem::size_of::<usize>();
//const PTR_SIZE: usize = 4;

pub const NULL: Slot = [0x00; PTR_SIZE];

pub type Slot = [u8; PTR_SIZE];
// pub type WideSlot = [u8; PTR_SIZE * 2];
pub type WideSlot = (Slot, Slot);
pub type Word = [u8; PTR_SIZE];

// pub fn split_wide_slot(w: WideSlot) -> (Slot, Slot) {
//     let mut higher = [0u8; PTR_SIZE];
//     higher.copy_from_slice(&w[0..PTR_SIZE]);
//     let mut lower = [0u8; PTR_SIZE];
//     lower.copy_from_slice(&w[PTR_SIZE..]);
//     (higher, lower)
// }

pub trait Memorizable<T> {

    fn memorized(&self) -> T;
}

impl Memorizable<Slot> for i32 {
    fn memorized(&self) -> Slot {
        unsafe {
            let bs = std::mem::transmute::<i32, [u8; 4]>(*self);
            let mut s = [0u8; PTR_SIZE];
            &s[0..4].copy_from_slice(&bs);
            s
        }
    }
}

impl Memorizable<WideSlot> for i64 {
    fn memorized(&self) -> WideSlot {
        unsafe {
            let bs = std::mem::transmute::<i64, [u8; 8]>(*self);
            let padding = [0u8; PTR_SIZE];
            (bs, padding)
        }
    }
}

impl Memorizable<Slot> for f32 {
    fn memorized(&self) -> Slot {
        unsafe {
            let bs = std::mem::transmute::<f32, [u8; 4]>(*self);
            let mut s = [0u8; PTR_SIZE];
            &s[0..4].copy_from_slice(&bs);
            s
        }
    }
}

impl Memorizable<WideSlot> for f64 {
    fn memorized(&self) -> WideSlot {
        unsafe {
            let bs = std::mem::transmute::<f64, [u8; 8]>(*self);
            let padding = [0u8; PTR_SIZE];
            (bs, padding)
        }
    }
}

pub trait Viewable {

    fn view<T>(self) -> T;
}

impl Viewable for Slot {

    fn view<T>(self) -> T {
        unsafe {
            let mut t = [0u8; 4];
            t.copy_from_slice(&self[0..4]);
            std::mem::transmute_copy::<[u8; 4], T>(&t)
        }
    }
}

impl Viewable for WideSlot {

    fn view<T>(self) -> T {
        unsafe {
            std::mem::transmute::<[u8; 8], T>(self.0)
        }
    }
}

pub struct Object {
    head: Word,
    klass: std::sync::Arc<self::metaspace::Klass>,
    array_info: Option<Word>,
    handle: Word,
    payload: Vec<u8>,
}

impl Object {
    fn padding(&mut self) {}
}

#[test]
fn memorize_i32() {
    let i = (256 as i32).memorized();
    assert_eq!(&i, &[0u8, 1, 0, 0, 0, 0, 0, 0]);
}
