use super::Traveler;
use bytecode::atom::*;

pub struct ConstantPool {
    items: Vec<ConstantItem>,
}

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
    MethodHandle(u8),
    MethodType(u16),
    InvokeDynamic(u8),
    Empty,
}

impl Traveler<ConstantPool> for ConstantPool {
    fn read<I>(seq: &mut I) -> ConstantPool
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut v = Vec::<ConstantItem>::with_capacity(size as usize + 1);
        v.push(ConstantItem::Empty);
        for _i in 0..size {
            v.push(ConstantItem::read(seq));
        }
        ConstantPool { items: v }
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


impl Traveler<ConstantItem> for ConstantItem {
    fn read<I>(seq: &mut I) -> ConstantItem
    where
        I: Iterator<Item = u8>,
    {
        let tag = U1::read(seq);
        match tag {
            //TODO
            UTF8_TAG => {
                let length = U2::read(seq);
                for _x in 0..length {
                    seq.next();
                }
                ConstantItem::UTF8("".to_string())
            }
            INTEGER_TAG => {
                U4::read(seq);
                ConstantItem::Integer(0)
            }
            _ => panic!("invalid classfile"),
        }
    }
}
