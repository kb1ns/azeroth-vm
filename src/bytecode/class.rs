use super::{atom::*, attribute::*, constant_pool::*, field::*, interface::*, method::*, *};
use std::sync::Arc;

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
        U4::read(seq, None);
        U2::read(seq, None);
        U2::read(seq, None);
        let constants = ConstantPool::read(seq, None);
        let access_flag = U2::read(seq, None);
        let this_class = U2::read(seq, None);
        let super_class = U2::read(seq, None);
        let this_class_name = constants.get_str(this_class).to_string();
        let super_class_name = constants.get_str(super_class).to_string();
        let interfaces = Interfaces::read(seq, Some(&constants));
        let fields = Fields::read(seq, Some(&constants));
        let methods = Methods::read(seq, Some(&constants));
        let attributes = Attributes::read(seq, Some(&constants));
        Class {
            constant_pool: constants,
            access_flag: access_flag,
            this_class_name: this_class_name,
            super_class_name: super_class_name,
            interfaces: interfaces,
            fields: fields,
            methods: methods,
            attributes: attributes,
        }
    }

    pub fn get_method(&self, method_name: &str, method_descriptor: &str) -> Option<Arc<Method>> {
        self.methods.find(method_name, method_descriptor)
    }

    pub fn get_field(&self, field_name: &str, field_descriptor: &str) -> Option<Arc<Field>> {
        for ref f in &self.fields {
            if f.name == field_name && f.descriptor == field_descriptor {
                return Some(Arc::clone(f));
            }
        }
        None
    }

    pub fn get_name(&self) -> &str {
        self.this_class_name.as_ref()
    }

    pub fn get_super_class(&self) -> &str {
        self.super_class_name.as_ref()
    }

    pub fn get_interfaces(&self) -> &Vec<String> {
        self.interfaces.as_ref()
    }
}
