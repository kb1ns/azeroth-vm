
use super::constant_pool::ConstantPool;
use super::Traveler;
use bytecode::atom::*;
use bytecode::attribute::*;

pub type Methods = Vec<Method>;

pub struct Method {
    pub access_flag: U2,
    pub name: String,
    pub descriptor: String,
    pub attributes: Attributes,
}

impl Traveler<Method> for Method {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Method
    where
        I: Iterator<Item = u8>,
    {
        let access_flag = U2::read(seq, None);
        if let Some(pool) = constants {
            return Method {
                access_flag: access_flag,
                name: pool.get_str(U2::read(seq, None)).to_string(),
                descriptor: pool.get_str(U2::read(seq, None)).to_string(),
                attributes: Attributes::read(seq, Some(pool)),
            };
        }
        panic!("need constant pool to resolve methods");
    }
}

impl Method {
    pub fn get_code(&self) -> Option<&Attribute> {
        for attr in &self.attributes {
            if let &Attribute::Code(_, _, _, _, _) = attr {
                return Some(attr);
            }
        }
        return None;
    }
}

impl Traveler<Methods> for Methods {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Methods
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq, None);
        let mut methods = Vec::<Method>::with_capacity(size as usize);
        for _x in 0..size {
            methods.push(Method::read(seq, constants));
        }
        methods
    }
}
