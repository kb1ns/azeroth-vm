pub mod class;
pub mod atom;

trait Traveler<T> {
    fn read<I>(seq: &mut I) -> T
    where
        I: Iterator<Item = u8>;
}
