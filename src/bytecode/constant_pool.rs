use super::Traveler;
use bytecode::atom::*;
use std::mem;
use std::str;

pub struct ConstantPool(Vec<ConstantItem>);

#[derive(Debug)]
pub enum ConstantItem {
    UTF8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(U2),
    String(U2),
    FieldRef(U2, U2),
    MethodRef(U2, U2),
    InterfaceMethodRef(U2, U2),
    NameAndType(U2, U2),
    MethodHandle(U1, U2),
    MethodType(U2),
    InvokeDynamic(U2, U2),
    NIL,
}

impl ConstantPool {
    pub fn get_integer(&self, idx: U2) -> i32 {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::Integer(i) => {
                    return i;
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }

    pub fn get_float(&self, idx: U2) -> f32 {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::Float(f) => {
                    return f;
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }

    pub fn get_long(&self, idx: U2) -> i64 {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::Long(l) => {
                    return l;
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }

    pub fn get_double(&self, idx: U2) -> f64 {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::Double(d) => {
                    return d;
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }

    pub fn get_name_and_type(&self, idx: U2) -> (&str, &str) {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::NameAndType(n_idx, t_idx) => {
                    return (&self.get_str(n_idx), &self.get_str(t_idx));
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }

    pub fn get_javaref(&self, idx: U2) -> (&str, (&str, &str)) {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::InterfaceMethodRef(c_idx, nt_idx) => {
                    return (&self.get_str(c_idx), self.get_name_and_type(nt_idx));
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }

    pub fn get_str(&self, idx: U2) -> &str {
        if let Some(item) = self.0.get(idx as usize) {
            match item {
                &ConstantItem::String(offset) => {
                    return self.get_str(offset);
                }
                &ConstantItem::UTF8(ref s) => {
                    return s;
                }
                &ConstantItem::Class(offset) => {
                    return self.get_str(offset);
                }
                _ => {
                    panic!("invalid class file");
                }
            }
        }
        panic!("invalid class file");
    }
}

impl Traveler<ConstantPool> for ConstantPool {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> ConstantPool
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq, None);
        let mut pool = Vec::<ConstantItem>::with_capacity(size as usize);
        pool.push(ConstantItem::NIL);
        let mut offset = 1;
        while offset < size {
            let tag = U1::read(seq, None);
            offset = offset + 1;
            let ele = match tag {
                INVOKEDYNAMIC_TAG => {
                    let bootstrap_method_attr_idx = U2::read(seq, None);
                    let name_and_type_idx = U2::read(seq, None);
                    ConstantItem::InvokeDynamic(bootstrap_method_attr_idx, name_and_type_idx)
                }
                METHODTYPE_TAG => {
                    let desc_idx = U2::read(seq, None);
                    ConstantItem::MethodType(desc_idx)
                }
                METHODHANDLE_TAG => {
                    let ref_type = U1::read(seq, None);
                    let ref_idx = U2::read(seq, None);
                    ConstantItem::MethodHandle(ref_type, ref_idx)
                }
                INTERFACEMETHODREF_TAG => {
                    let class_idx = U2::read(seq, None);
                    let name_and_type_idx = U2::read(seq, None);
                    ConstantItem::InterfaceMethodRef(class_idx, name_and_type_idx)
                }
                STRING_TAG => {
                    let string_idx = U2::read(seq, None);
                    ConstantItem::String(string_idx)
                }
                CLASS_TAG => {
                    let name_idx = U2::read(seq, None);
                    ConstantItem::Class(name_idx)
                }
                METHODREF_TAG => {
                    let class_idx = U2::read(seq, None);
                    let name_and_type_idx = U2::read(seq, None);
                    ConstantItem::MethodRef(class_idx, name_and_type_idx)
                }
                FIELDREF_TAG => {
                    let class_idx = U2::read(seq, None);
                    let name_and_type_idx = U2::read(seq, None);
                    ConstantItem::FieldRef(class_idx, name_and_type_idx)
                }
                UTF8_TAG => {
                    let length = U2::read(seq, None);
                    let mut buf = Vec::<u8>::with_capacity(length as usize);
                    for _x in 0..length {
                        buf.push(U1::read(seq, None));
                    }
                    // TODO MUTF-8 encode
                    let s = str::from_utf8(&buf).unwrap();
                    ConstantItem::UTF8(s.to_string())
                }
                INTEGER_TAG => {
                    let v = U4::read(seq, None);
                    let i: i32 = unsafe { mem::transmute::<u32, i32>(v) };
                    ConstantItem::Integer(i)
                }
                FLOAT_TAG => {
                    let v = U4::read(seq, None);
                    let i: f32 = unsafe { mem::transmute::<u32, f32>(v) };
                    ConstantItem::Float(i)
                }
                LONG_TAG => {
                    let v = U8::read(seq, None);
                    let i: i64 = unsafe { mem::transmute::<u64, i64>(v) };
                    offset = offset + 1;
                    pool.push(ConstantItem::NIL);
                    ConstantItem::Long(i)
                }
                DOUBLE_TAG => {
                    let v = U8::read(seq, None);
                    let i: f64 = unsafe { mem::transmute::<u64, f64>(v) };
                    offset = offset + 1;
                    pool.push(ConstantItem::NIL);
                    ConstantItem::Double(i)
                }
                NAMEANDTYPE_TAG => {
                    let name_idx = U2::read(seq, None);
                    let desc_idx = U2::read(seq, None);
                    ConstantItem::NameAndType(name_idx, desc_idx)
                }
                _ => panic!("invalid classfile"),
            };
            pool.push(ele);
        }
        ConstantPool(pool)
    }
}

const UTF8_TAG: u8 = 1;
const INTEGER_TAG: u8 = 3;
const FLOAT_TAG: u8 = 4;
const LONG_TAG: u8 = 5;
const DOUBLE_TAG: u8 = 6;
const CLASS_TAG: u8 = 7;
const STRING_TAG: u8 = 8;
const FIELDREF_TAG: u8 = 9;
const METHODREF_TAG: u8 = 10;
const INTERFACEMETHODREF_TAG: u8 = 11;
const NAMEANDTYPE_TAG: u8 = 12;
const METHODHANDLE_TAG: u8 = 15;
const METHODTYPE_TAG: u8 = 16;
const INVOKEDYNAMIC_TAG: u8 = 18;
