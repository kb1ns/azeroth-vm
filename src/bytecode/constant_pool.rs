use super::Traveler;
use bytecode::atom::*;

pub struct ConstantPool {
    items: Vec<ConstantItem>,
}

pub enum ConstantItem {
    utf(u8),
    integer(u8),
    float(u8),
    long(u8),
    double(u8),
    class(u8),
    string(u8),
    field_ref(u8),
    method_ref(u8),
    interface_method_ref(u8),
    name_and_type(u8),
    method_handle(u8),
    method_type(u8),
    invoke_dynamic(u8),
    empty,
}

impl Traveler<ConstantPool> for ConstantPool {
    fn read<I>(seq: &mut I) -> ConstantPool
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut v = Vec::<ConstantItem>::with_capacity(size as usize);
        v.push(ConstantItem::empty);
        ConstantPool { items: v }
    }
}

impl Traveler<ConstantItem> for ConstantItem {
    fn read<I>(seq: &mut I) -> ConstantItem
    where
        I: Iterator<Item = u8>,
    {
        let length = U2::read(seq);
        ConstantItem::empty
    }
}
