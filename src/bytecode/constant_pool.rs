use super::Traveler;
use std::mem;
use bytecode::atom::*;

pub type ConstantPool = Vec<ConstantItem>;

#[derive(Debug)]
pub enum ConstantItem {
    UTF8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(u16),
    String(u16),
    FieldRef(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
    NameAndType(u16, u16),
    MethodHandle(u8, u16),
    MethodType(u16),
    InvokeDynamic(u16, u16),
}

impl Traveler<ConstantPool> for ConstantPool {
    fn read<I>(seq: &mut I) -> ConstantPool
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut v = Vec::<ConstantItem>::with_capacity(size as usize - 1);
        let mut offset = 1;
        while offset < size {
            let tag = U1::read(seq);
            offset = offset + 1;
            let ele = match tag {
                INVOKEDYNAMIC_TAG => {
                    let bootstrap_method_attr_idx = U2::read(seq);
                    let name_and_type_idx = U2::read(seq);
                    ConstantItem::InvokeDynamic(bootstrap_method_attr_idx, name_and_type_idx)
                }
                METHODTYPE_TAG => {
                    let desc_idx = U2::read(seq);
                    ConstantItem::MethodType(desc_idx)
                }
                METHODHANDLE_TAG => {
                    let ref_type = U1::read(seq);
                    let ref_idx = U2::read(seq);
                    ConstantItem::MethodHandle(ref_type, ref_idx)
                }
                INTERFACEMETHODREF_TAG => {
                    let class_idx = U2::read(seq);
                    let name_and_type_idx = U2::read(seq);
                    ConstantItem::InterfaceMethodRef(class_idx, name_and_type_idx)
                }
                STRING_TAG => {
                    let string_idx = U2::read(seq);
                    ConstantItem::String(string_idx)
                }
                CLASS_TAG => {
                    let name_idx = U2::read(seq);
                    ConstantItem::Class(name_idx)
                }
                METHODREF_TAG => {
                    let class_idx = U2::read(seq);
                    let name_and_type_idx = U2::read(seq);
                    ConstantItem::MethodRef(class_idx, name_and_type_idx)
                }
                FIELDREF_TAG => {
                    let class_idx = U2::read(seq);
                    let name_and_type_idx = U2::read(seq);
                    ConstantItem::FieldRef(class_idx, name_and_type_idx)
                }
                //TODO
                UTF8_TAG => {
                    let length = U2::read(seq);
                    let mut buf = Vec::<u8>::with_capacity(length as usize);
                    for _x in 0..length {
                        buf.push(U1::read(seq));
                    }
                    ConstantItem::UTF8("".to_string())
                }
                INTEGER_TAG => {
                    let v = U4::read(seq);
                    let i: i32 = unsafe { mem::transmute::<u32, i32>(v) };
                    ConstantItem::Integer(i)
                }
                FLOAT_TAG => {
                    let v = U4::read(seq);
                    let i: f32 = unsafe { mem::transmute::<u32, f32>(v) };
                    ConstantItem::Float(i)
                }
                LONG_TAG => {
                    let v = U8::read(seq);
                    let i: i64 = unsafe { mem::transmute::<u64, i64>(v) };
                    offset = offset + 1;
                    ConstantItem::Long(i)
                }
                DOUBLE_TAG => {
                    let v = U8::read(seq);
                    let i: f64 = unsafe { mem::transmute::<u64, f64>(v) };
                    offset = offset + 1;
                    ConstantItem::Double(i)
                }
                NAMEANDTYPE_TAG => {
                    let name_idx = U2::read(seq);
                    let desc_idx = U2::read(seq);
                    ConstantItem::NameAndType(name_idx, desc_idx)
                }
                _ => panic!("invalid classfile"),
            };
            v.push(ele);
        }
        v
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


// impl Traveler<ConstantItem> for ConstantItem {
//     fn read<I>(seq: &mut I) -> ConstantItem
//     where
//         I: Iterator<Item = u8>,
//     {
//         let tag = U1::read(seq);
//     }
// }
