use super::{atom::*, attribute::*, constant_pool::ConstantPool, Traveler};
use crate::mem::Value;
use std::cell::Cell;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::sync::Arc;

pub type Fields = Vec<Arc<Field>>;

const ACC_STATIC: u16 = 0x0008;

const ACC_PUBLIC: u16 = 0x0001;

const ACC_PROTECTED: u16 = 0x0004;

const ACC_PRIVATE: u16 = 0x0002;

const ACC_FINAL: u16 = 0x0010;

pub struct Field {
    pub access_flag: U2,
    pub name: String,
    pub descriptor: String,
    pub attributes: Attributes,
    pub value: Cell<Option<Value>>,
}

impl Field {
    pub fn memory_size(&self) -> usize {
        match self.descriptor.as_str() {
            "J" | "D" => 8,
            _ => 4,
        }
    }

    pub fn is_static(&self) -> bool {
        self.access_flag & ACC_STATIC == ACC_STATIC
    }

    pub fn is_public(&self) -> bool {
        self.access_flag & ACC_PUBLIC == ACC_PUBLIC
    }

    pub fn is_protected(&self) -> bool {
        self.access_flag & ACC_PROTECTED == ACC_PROTECTED
    }

    pub fn is_private(&self) -> bool {
        self.access_flag & ACC_PRIVATE == ACC_PRIVATE
    }

    pub fn is_final(&self) -> bool {
        self.access_flag & ACC_FINAL == ACC_FINAL
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

// TODO
fn init_value(access_flag: u16, descriptor: &str) -> Option<Value> {
    if access_flag & ACC_STATIC == ACC_STATIC {
        match descriptor {
            "D" | "J" => Some(Value::DWord(0)),
            _ => Some(Value::Word(0)),
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
