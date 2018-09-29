use super::Traveler;
use super::constant_pool::ConstantPool;
use bytecode::atom::*;
use bytecode::attribute::*;

pub type Fields = Vec<Field>;

pub struct Field {
    pub access_flag: U2,
    pub name: String,
    pub descriptor: String,
    pub attributes: Vec<Attribute>,
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

impl Traveler<Field> for Field {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Field
    where
        I: Iterator<Item = u8>,
    {
        let access_flag = U2::read(seq, None);
        let name_idx = U2::read(seq, None);
        let descriptor_idx = U2::read(seq, None);
        if let Some(pool) = constants {
            return Field {
                access_flag: access_flag,
                name: pool.get_str(name_idx).to_string(),
                descriptor: pool.get_str(descriptor_idx).to_string(),
                attributes: Attributes::read(seq, Some(pool)),
            };
        }
        panic!("need constant pool to resolve fields")
    }
}
