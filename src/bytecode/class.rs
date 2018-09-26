use bytecode::atom::*;
use bytecode::constant_pool::*;
use bytecode::interface::*;
use bytecode::field::*;
use bytecode::method::*;
use bytecode::attribute::*;
use bytecode::*;

pub struct Class {
    magic_number: U4,
    minor_version: U2,
    major_version: U2,
    constant_pool: ConstantPool,
    access_flag: U2,
    this_class: U2,
    super_class: U2,
    interfaces: Interfaces,
    fields: Fields,
    methods: Methods,
    attributes: Attributes,
}

impl Class {
    pub fn from_vec(bytes: Vec<u8>) -> Class {
        let seq = &mut bytes.into_iter();
        Class {
            magic_number: U4::read(seq),
            minor_version: U2::read(seq),
            major_version: U2::read(seq),
            constant_pool: ConstantPool::read(seq),
            access_flag: U2::read(seq),
            this_class: U2::read(seq),
            super_class: U2::read(seq),
            interfaces: Interfaces::read(seq),
            fields: Fields::read(seq),
            methods: Methods::read(seq),
            attributes: Attributes::read(seq),
        }
    }
}
