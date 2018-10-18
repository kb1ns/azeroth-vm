pub mod atom;
pub mod attribute;
pub mod class;
pub mod constant_pool;
pub mod field;
pub mod interface;
pub mod method;

use super::mem::*;
use self::constant_pool::ConstantPool;

trait Traveler<T> {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> T
    where
        I: Iterator<Item = u8>;
}
