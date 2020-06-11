use super::{atom::*, constant_pool::ConstantPool, Traveler};

pub type Interfaces = Vec<String>;

impl Traveler<Interfaces> for Interfaces {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Interfaces
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq, None);
        let mut interfaces = Vec::<String>::with_capacity(size as usize);
        if let Some(pool) = constants {
            for _x in 0..size {
                let idx = U2::read(seq, None);
                interfaces.push(pool.get_str(idx).to_string());
            }
            return interfaces;
        }
        panic!("need constant pool to resolve interfaces");
    }
}
