pub mod atom;
pub mod attribute;
pub mod class;
pub mod constant_pool;
pub mod field;
pub mod interface;
pub mod method;

use bytecode::constant_pool::ConstantPool;
use bytecode::atom::*;

trait Traveler<T> {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> T
    where
        I: Iterator<Item = u8>;
}

pub const JVM_BYTE: char = 'B';
pub const JVM_CHAR: char = 'C';
pub const JVM_DOUBLE: char = 'D';
pub const JVM_FLOAT: char = 'F';
pub const JVM_INT: char = 'I';
pub const JVM_LONG: char = 'J';
pub const JVM_SHORT: char = 'S';
pub const JVM_BOOLEAN: char = 'Z';
pub const JVM_REF: char = 'L';
pub const JVM_ARRAY: char = '[';

pub const METHOD_ACC_STATIC: U2 = 0x0008;
