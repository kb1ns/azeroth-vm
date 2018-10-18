use super::constant_pool::ConstantPool;
use super::Traveler;
use super::Value;
use super::NULL;
use bytecode::atom::*;
use bytecode::attribute::*;

pub type Fields = Vec<Field>;

const STATIC: u16 = 0x0008;

pub struct Field {
    pub access_flag: U2,
    pub name: String,
    pub descriptor: String,
    pub attributes: Vec<Attribute>,
    pub value: Option<Value>,
}

impl Traveler<Fields> for Fields {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Fields
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq, None);
        let mut fields = Vec::<Field>::with_capacity(size as usize);
        for _x in 0..size {
            fields.push(Field::read(seq, constants));
        }
        fields
    }
}

fn get_init_value(access_flag: u16, descriptor: &str) -> Option<Value> {
    if access_flag & STATIC == STATIC {
        if descriptor == "D" || descriptor == "J" {
            Some(Value::DWord(NULL, NULL))
        } else {
            Some(Value::Word(NULL))
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
                value: get_init_value(access_flag, &descriptor),
                descriptor: descriptor,
                attributes: Attributes::read(seq, Some(pool)),
            };
        }
        panic!("need constant pool to resolve fields")
    }
}
