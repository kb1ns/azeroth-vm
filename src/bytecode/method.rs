use super::Traveler;
use bytecode::atom::*;
use bytecode::attribute::*;

pub type Methods = Vec<Method>;

pub struct Method {
    pub access_flag: U2,
    pub name_index: U2,
    pub descriptor_index: U2,
    pub attributes: Vec<Attribute>,
}

impl Traveler<Methods> for Methods {
    fn read<I>(seq: &mut I) -> Methods
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut methods = Vec::<Method>::with_capacity(size as usize);
        for _x in 0..size {
            methods.push(Method::read(seq));
        }
        methods
    }
}

impl Traveler<Method> for Method {
    fn read<I>(seq: &mut I) -> Method
    where
        I: Iterator<Item = u8>,
    {
        Method {
            access_flag: U2::read(seq),
            name_index: U2::read(seq),
            descriptor_index: U2::read(seq),
            attributes: Attributes::read(seq),
        }
    }
}
