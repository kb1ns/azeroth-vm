use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

use crate::bytecode::class::Class;
use crate::classpath::Classpath;

use chashmap::CHashMap;
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
            Value::Byte(vv) => [vv, 0, 0, 0],
            Value::DByte(vv) => {
                let most = ((vv & 0xf0u16) >> 8) as u8;
                let least = (vv & 0x0fu16) as u8;
                [most, least, 0, 0]
            }
            Value::Word(vv) => vv.to_le_bytes(),
            _ => panic!(""),
        }
    }

    pub fn of_w(v: Value) -> WideSlot {
        match v {
            Value::DWord(vv) => vv.to_le_bytes(),
            _ => panic!(""),
        }
    }

    pub fn eval(v: Slot, descriptor: &str) -> Value {
        let ch = descriptor.chars().next().unwrap();
        match ch {
            'D' | 'J' => {
                let mut vv = [0u8; 8];
                &vv[..].copy_from_slice(&v);
                Value::DWord(u64::from_le_bytes(vv))
            }
            'Z' | 'B' => Value::Byte(v[0]),
            'S' | 'C' => {
                let mut vv = [0u8; 2];
                &vv[..].copy_from_slice(&v);
                Value::DByte(u16::from_le_bytes(vv))
            }
            _ => {
                let mut vv = [0u8; 4];
                &vv[..].copy_from_slice(&v);
                Value::Word(u32::from_le_bytes(vv))
            }
        }
    }

    pub fn eval_w(v: WideSlot) -> Value {
        Value::DWord(u64::from_le_bytes(v))
    }
}

#[derive(Debug)]
pub struct RefKey {
    key: (String, String, String),
    key_ptr: ((*const u8, usize), (*const u8, usize), (*const u8, usize)),
}

impl RefKey {
    pub fn new(key0: String, key1: String, key2: String) -> Self {
        Self {
            key_ptr: (
                (key0.as_bytes().as_ptr(), key0.len()),
                (key1.as_bytes().as_ptr(), key1.len()),
                (key2.as_bytes().as_ptr(), key2.len()),
            ),
            key: (key0, key1, key2),
        }
    }
}

impl Hash for RefKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.0.hash(state);
        self.key.1.hash(state);
        self.key.2.hash(state);
    }
}

impl PartialEq for RefKey {
    fn eq(&self, other: &Self) -> bool {
        self.key.0 == other.key.0 && self.key.1 == other.key.1 && self.key.2 == other.key.2
    }
}

impl<'a> Borrow<(&'a str, &'a str, &'a str)> for RefKey
where
    Self: 'a,
{
    fn borrow(&self) -> &(&'a str, &'a str, &'a str) {
        unsafe { std::mem::transmute(&self.key_ptr) }
    }
}

impl Eq for RefKey {}

impl Clone for RefKey {
    fn clone(&self) -> Self {
        Self::new(self.key.0.clone(), self.key.1.clone(), self.key.2.clone())
    }
}
