use super::Traveler;
use super::constant_pool::ConstantPool;
use bytecode::atom::*;

pub type Attributes = Vec<Attribute>;

pub struct Attribute {
    pub name: String,
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
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Attributes
    where
        I: Iterator<Item = u8>,
    {
        let attribute_count = U2::read(seq, None) as usize;
        let mut attributes = Vec::<Attribute>::with_capacity(attribute_count);
        for _x in 0..attribute_count {
            attributes.push(Attribute::read(seq, constants));
        }
        attributes
    }
}

impl Traveler<Attribute> for Attribute {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> Attribute
    where
        I: Iterator<Item = u8>,
    {
        let name_index = U2::read(seq, None);
        let length = U4::read(seq, None) as usize;
        let mut content = Vec::<U1>::with_capacity(length);
        for _x in 0..length {
            content.push(U1::read(seq, None));
        }
        if let Some(pool) = constants {
            return Attribute {
                name: pool.get_str(name_index).to_string(),
                content: content,
            };
        }
        panic!("need constant pool to resolve attributes");
    }
}
