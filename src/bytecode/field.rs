use super::Traveler;
use bytecode::atom::*;
use bytecode::attribute::*;

pub type Fields = Vec<Field>;

pub struct Field {
    pub access_flag: U2,
    pub name_index: U2,
    pub descriptor_index: U2,
    pub attributes: Vec<Attribute>,
}

impl Traveler<Fields> for Fields {
    fn read<I>(seq: &mut I) -> Fields
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut fields = Vec::<Field>::with_capacity(size as usize);
        for _x in 0..size {
            fields.push(Field::read(seq));
        }
        fields
    }
}

impl Traveler<Field> for Field {
    fn read<I>(seq: &mut I) -> Field
    where
        I: Iterator<Item = u8>,
    {
        Field {
            access_flag: U2::read(seq),
            name_index: U2::read(seq),
            descriptor_index: U2::read(seq),
            attributes: Attributes::read(seq),
        }
    }
}

impl Field {
    pub fn is_public(&self) -> bool {
        false
    }
}
