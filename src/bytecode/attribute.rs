use super::constant_pool::ConstantPool;
use super::Traveler;
use bytecode::atom::*;

pub type Attributes = Vec<Attribute>;

pub struct ExceptionHandler {
    pub start_pc: U2,
    pub end_pc: U2,
    pub handler_pc: U2,
    pub catch_type: Option<String>,
}

pub enum Attribute {
    ConstantValue(Vec<U1>),
    Code(U2, U2, Vec<u8>, Vec<ExceptionHandler>, Attributes),
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

pub const CONSTANT_VALUE: &'static str = "ConstantValue";
pub const CODE: &'static str = "Code";
pub const STACK_MAP_TABLE: &'static str = "StackMapTable";
pub const EXCEPTIONS: &'static str = "Exceptions";
pub const BOOTSTRAP_METHODS: &'static str = "BootstrapMethods";
pub const INNER_CLASSES: &'static str = "InnerClasses";
pub const ENCLOSING_METHOD: &'static str = "EnclosingMethod";
pub const SYNTHETIC: &'static str = "Synthetic";
pub const SIGNATURE: &'static str = "Signature";
pub const RUNTIME_VISIBLE_ANNOTATIONS: &'static str = "RuntimeVisibleAnnotations";
pub const RUNTIME_INVISIBLE_ANNOTATIONS: &'static str = "RuntimeInvisibleAnnotations";
pub const RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &'static str =
    "RuntimeVisibleParameterAnnotations";
pub const RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &'static str =
    "RuntimeInvisibleParameterAnnotations";
pub const RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &'static str = "RuntimeVisibleTypeAnnotations";
pub const RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &'static str = "RuntimeInvisibleTypeAnnotations";
pub const ANNOTATION_DEFAULT: &'static str = "AnnotationDefault";
pub const METHOD_PARAMETERS: &'static str = "MethodParameters";

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
        let name_idx = U2::read(seq, None);
        let length = U4::read(seq, None) as usize;
        if let Some(pool) = constants {
            match pool.get_str(name_idx) {
                CODE => {
                    let max_stacks = U2::read(seq, None);
                    let max_locals = U2::read(seq, None);
                    let code_length = U4::read(seq, None);
                    let mut code = Vec::<u8>::with_capacity(code_length as usize);
                    for _x in 0..code_length {
                        code.push(U1::read(seq, None));
                    }
                    let exception_handler_count = U2::read(seq, None);
                    let mut exception_handlers =
                        Vec::<ExceptionHandler>::with_capacity(exception_handler_count as usize);
                    for _x in 0..exception_handler_count {
                        let start_pc = U2::read(seq, None);
                        let end_pc = U2::read(seq, None);
                        let handler_pc = U2::read(seq, None);
                        let catch_type_idx = U2::read(seq, Some(pool));
                        let catch_type = match catch_type_idx {
                            0 => None,
                            _ => Some(pool.get_str(catch_type_idx).to_string()),
                        };
                        exception_handlers.push(ExceptionHandler {
                            start_pc: start_pc,
                            end_pc: end_pc,
                            handler_pc: handler_pc,
                            catch_type: catch_type,
                        })
                    }
                    return Attribute::Code(
                        max_stacks,
                        max_locals,
                        code,
                        exception_handlers,
                        Attributes::read(seq, Some(pool)),
                    );
                }
                _ => {
                    let mut content = Vec::<U1>::with_capacity(length);
                    for _x in 0..length {
                        content.push(U1::read(seq, None));
                    }
                    return Attribute::ConstantValue(content);
                }
            }
        }
        panic!("need constant pool to resolve attributes");
    }
}
