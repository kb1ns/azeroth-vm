use super::Traveler;
use super::constant_pool;
use super::constant_pool::ConstantPool;
use bytecode::atom::*;

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
                interfaces.push(constant_pool::get_str(pool, idx).to_string());
            }
            return interfaces;
        }
        panic!("need constant pool to resolve interfaces");
    }
}
