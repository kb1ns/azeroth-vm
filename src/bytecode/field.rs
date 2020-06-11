use super::{atom::*, attribute::*, constant_pool::ConstantPool, Traveler};
use crate::mem::Value;
use std::cell::Cell;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::sync::Arc;

pub type Fields = Vec<Arc<Field>>;

const STATIC: u16 = 0x0008;

pub struct Field {
    pub access_flag: U2,
    pub name: String,
    pub descriptor: String,
    pub attributes: Attributes,
    pub value: Cell<Option<Value>>,
}

impl Field {
    /// B = Z = 1 < C = S = 2 < I = F = L = [ = 4 < J = D = 8
    pub fn memory_size(&self) -> usize {
        let ch = self.descriptor.chars().next().expect("");
        match ch {
            'B' | 'Z' => 1,
            'C' | 'S' => 2,
            'I' | 'F' | 'L' | '[' => 4,
            'J' | 'D' => 8,
            _ => {
                panic!("Illegal descriptor");
            }
        }
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> Ordering {
        self.memory_size().cmp(&other.memory_size())
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Field {}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.access_flag == other.access_flag
            && self.name == other.name
            && self.descriptor == other.descriptor
    }
}

impl Traveler<Fields> for Fields {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Fields
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq, None);
        let mut fields = Vec::<Arc<Field>>::with_capacity(size as usize);
        for _x in 0..size {
            fields.push(Arc::new(Field::read(seq, constants)));
        }
        fields
    }
}

fn init_value(access_flag: u16, descriptor: &str) -> Option<Value> {
    if access_flag & STATIC == STATIC {
        if descriptor == "D" || descriptor == "J" {
            Some(Value::DWord(0))
        } else {
            Some(Value::Word(0))
        }
    } else {
        None
    }
}

impl Traveler<Field> for Field {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Field
    where
        I: Iterator<Item = u8>,
    {
        let access_flag = U2::read(seq, None);
        let name_idx = U2::read(seq, None);
        let descriptor_idx = U2::read(seq, None);
        if let Some(pool) = constants {
            let descriptor = pool.get_str(descriptor_idx).to_string();
            return Field {
                access_flag: access_flag,
                name: pool.get_str(name_idx).to_string(),
                value: Cell::new(None),
                descriptor: descriptor,
                attributes: Attributes::read(seq, Some(pool)),
            };
        }
        panic!("need constant pool to resolve fields")
    }
}
