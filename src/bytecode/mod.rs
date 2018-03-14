pub mod class;
pub mod atom;
pub mod constant_pool;

trait Traveler<T> {
    fn read<I>(seq: &mut I) -> T
    where
        I: Iterator<Item = u8>;
}
