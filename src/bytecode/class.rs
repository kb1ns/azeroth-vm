use bytecode::atom::*;
use bytecode::constant_pool::ConstantPool;
use bytecode::*;

pub struct Class {
    magic_number: U4,
    minor_version: U2,
    major_version: U2,
    constant_pool: ConstantPool,
}

impl Class {
    pub fn from_vec(bytes: Vec<u8>) -> Class {
        let seq = &mut bytes.into_iter();
        Class {
            magic_number: U4::read(seq),
            minor_version: U2::read(seq),
            major_version: U2::read(seq),
            constant_pool: ConstantPool::read(seq),
        }
    }
}

#[test]
fn class_test() {
    let clz = class::Class::from_vec(vec![255, 0, 0, 1, 1, 2, 0, 255, 1, 1]);
}
