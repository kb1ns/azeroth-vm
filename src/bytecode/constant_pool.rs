use super::Traveler;
use bytecode::atom::*;

pub struct ConstantPool {
    items: Vec<ConstantItem>,
}

pub enum ConstantItem {
    utf8(String),
    integer(i32),
    float(f32),
    long(i64),
    double(f64),
    class(u16),
    string(u16),
    field_ref(u16, u16),
    method_ref(u16, u16),
    interface_method_ref(u16, u16),
    name_and_type(u16, u16),
    method_handle(u8),
    method_type(u16),
    invoke_dynamic(u8),
    empty,
}

impl Traveler<ConstantPool> for ConstantPool {
    fn read<I>(seq: &mut I) -> ConstantPool
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut v = Vec::<ConstantItem>::with_capacity(size as usize + 1);
        v.push(ConstantItem::empty);
        for i in 0..size {
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
                for x in 0..length {
                    seq.next();
                }
                ConstantItem::utf8("".to_string())
            }
            INTEGER_TAG => {
                U4::read(seq);
                ConstantItem::integer(0)
            }
            _ => panic!("invalid classfile"),
        }
    }
}
