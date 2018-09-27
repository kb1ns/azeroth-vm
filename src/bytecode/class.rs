use bytecode::atom::*;
use bytecode::constant_pool::*;
use bytecode::interface::*;
use bytecode::field::*;
use bytecode::method::*;
use bytecode::attribute::*;
use bytecode::*;

pub struct Class {
    pub constant_pool: ConstantPool,
    access_flag: U2,
    pub this_class_name: String,
    pub super_class_name: String,
    pub interfaces: Interfaces,
    pub fields: Fields,
    pub methods: Methods,
    pub attributes: Attributes,
}

impl Class {
    pub fn from_vec(bytes: Vec<u8>) -> Class {
        let seq = &mut bytes.into_iter();
        let magic_number = U4::read(seq, None);
        let minor_version = U2::read(seq, None);
        let major_version = U2::read(seq, None);
        let constant_pool = ConstantPool::read(seq, None);
        let access_flag = U2::read(seq, None);
        let this_class = U2::read(seq, None);
        let super_class = U2::read(seq, None);
        let this_class_name = constant_pool::get_str(&constant_pool, this_class).to_string();
        let super_class_name = constant_pool::get_str(&constant_pool, super_class).to_string();
        let interfaces = Interfaces::read(seq, Some(&constant_pool));
        let fields = Fields::read(seq, Some(&constant_pool));
        let methods = Methods::read(seq, Some(&constant_pool));
        let attributes = Attributes::read(seq, Some(&constant_pool));
        Class {
            constant_pool: constant_pool,
            access_flag: access_flag,
            this_class_name: this_class_name,
            super_class_name: super_class_name,
            interfaces: interfaces,
            fields: fields,
            methods: methods,
            attributes: attributes,
        }
    }

    pub fn debug_constants(&self) {
        for item in &self.constant_pool {
            println!("{:?}", item);
        }
    }

    pub fn get_method(&self, method_name: &str, descriptor: &str) {
    }
}
