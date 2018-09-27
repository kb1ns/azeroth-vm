pub mod class;
pub mod atom;
pub mod constant_pool;
pub mod interface;
pub mod field;
pub mod method;
pub mod attribute;

use self::constant_pool::ConstantPool;

trait Traveler<T> {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> T
    where
        I: Iterator<Item = u8>;
}
