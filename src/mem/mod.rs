use bytecode::class::Class;
use chashmap::CHashMap;
use classpath::Classpath;
use regex::Regex;

#[macro_use]
pub mod heap;
#[macro_use]
pub mod metaspace;
pub mod klass;
pub mod stack;

// pub const PTR_SIZE: usize = std::mem::size_of::<usize>();

// 32-bit vm
pub const PTR_SIZE: usize = 4;

pub const NULL: Slot = [0x00; PTR_SIZE];
pub const LONG_NULL: WideSlot = [0x00; PTR_SIZE * 2];

pub type Slot = [u8; PTR_SIZE];
pub type WideSlot = [u8; PTR_SIZE * 2];

pub type Ref = u32;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Byte(u8),
    DByte(u16),
    Word(u32),
    DWord(u64),
}

impl Value {
    pub fn of(v: Value) -> Slot {
        match v {
            Value::Byte(vv) => {
                if cfg!(target_endian = "big") {
                    [0, 0, 0, vv]
                } else {
                    [vv, 0, 0, 0]
                }
            }
            Value::DByte(vv) => {
                let high = ((vv & 0xf0u16) >> 8) as u8;
                let low = (vv & 0x0fu16) as u8;
                if cfg!(target_endian = "big") {
                    [high, low, 0, 0]
                } else {
                    [0, 0, low, high]
                }
            }
            Value::Word(vv) => vv.to_ne_bytes(),
            _ => panic!(""),
        }
    }

    pub fn of_w(v: Value) -> WideSlot {
        match v {
            Value::DWord(vv) => vv.to_ne_bytes(),
            _ => panic!(""),
        }
    }

    pub fn eval(v: Slot, descriptor: &str) -> Value {
        let ch = descriptor.chars().next().unwrap();
        match ch {
            'D' | 'J' => {
                let mut vv = [0u8; 8];
                &vv[..].copy_from_slice(&v);
                Value::DWord(u64::from_ne_bytes(vv))
            }
            'Z' | 'B' => Value::Byte(v[0]),
            'S' | 'C' => {
                let mut vv = [0u8; 2];
                &vv[..].copy_from_slice(&v);
                Value::DByte(u16::from_ne_bytes(vv))
            }
            _ => {
                let mut vv = [0u8; 4];
                &vv[..].copy_from_slice(&v);
                Value::Word(u32::from_ne_bytes(vv))
            }
        }
    }

    pub fn eval_w(v: WideSlot) -> Value {
        Value::DWord(u64::from_ne_bytes(v))
    }
}

pub trait Memorizable<T> {
    fn memorized(&self) -> T;
}

impl Memorizable<Slot> for i32 {
    fn memorized(&self) -> Slot {
        self.to_ne_bytes()
    }
}

impl Memorizable<WideSlot> for i64 {
    fn memorized(&self) -> WideSlot {
        self.to_ne_bytes()
    }
}

impl Memorizable<Slot> for u32 {
    fn memorized(&self) -> Slot {
        self.to_ne_bytes()
    }
}

impl Memorizable<WideSlot> for u64 {
    fn memorized(&self) -> WideSlot {
        self.to_ne_bytes()
    }
}

impl Memorizable<Slot> for f32 {
    fn memorized(&self) -> Slot {
        unsafe { std::mem::transmute::<f32, Slot>(*self) }
    }
}

impl Memorizable<WideSlot> for f64 {
    fn memorized(&self) -> WideSlot {
        unsafe { std::mem::transmute::<f64, WideSlot>(*self) }
    }
}

#[test]
fn memorize_i32() {
    let i = (256 as i32).memorized();
    if cfg!(target_endian = "big") {
        assert_eq!(&i, &[0, 0, 1, 0]);
    } else {
        assert_eq!(&i, &[0, 1, 0, 0]);
    }
}
