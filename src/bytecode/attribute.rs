use super::Traveler;
use bytecode::atom::*;

pub type Attributes = Vec<Attribute>;

pub struct Attribute {
    pub name_index: U2,
    pub content: Vec<U1>,
}

pub enum AttributeInfo {
    ConstantValue(Vec<U1>),
    Code(Vec<U1>),
    StackMapTable(Vec<U1>),
    Exceptions(Vec<U1>),
    BootstrapMethods(Vec<U1>),
    // above for JVM
    InnerClasses(Vec<U1>),
    EnclosingMethod(Vec<U1>),
    Synthetic(Vec<U1>),
    Signature(Vec<U1>),
    RuntimeVisibleAnnotations(Vec<U1>),
    RuntimeInvisibleAnnotations(Vec<U1>),
    RuntimeVisibleParameterAnnotations(Vec<U1>),
    RuntimeInvisibleParameterAnnotations(Vec<U1>),
    RuntimeVisibleTypeAnnotations(Vec<U1>),
    RuntimeInvisibleTypeAnnotations(Vec<U1>),
    AnnotationDefault(Vec<U1>),
    MethodParameters(Vec<U1>),
    // above for Java SE
}

impl Traveler<Attributes> for Attributes {
    fn read<I>(seq: &mut I) -> Attributes
    where
        I: Iterator<Item = u8>,
    {
        let attribute_count = U2::read(seq) as usize;
        let mut attributes = Vec::<Attribute>::with_capacity(attribute_count);
        for _x in 0..attribute_count {
            attributes.push(Attribute::read(seq));
        }
        attributes
    }
}

impl Traveler<Attribute> for Attribute {
    fn read<I>(seq: &mut I) -> Attribute
    where
        I: Iterator<Item = u8>,
    {
        let name_index = U2::read(seq);
        let length = U4::read(seq) as usize;
        let mut content = Vec::<U1>::with_capacity(length);
        for _x in 0..length {
            content.push(U1::read(seq));
        }
        Attribute {
            name_index: name_index,
            content: content,
        }
    }
}
