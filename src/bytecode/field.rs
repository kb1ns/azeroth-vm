use super::Traveler;
use bytecode::atom::*;

pub type Fields = Vec<Field>;

pub struct Field {
    
}

impl Traveler<Fields> for Fields {

    fn read<I>(seq: &mut I) -> Fields
    where
        I: Iterator<Item = u8>,
    {
        Vec::<Field>::with_capacity(1)
    }
}
