use super::Traveler;
use bytecode::atom::*;

pub type Methods = Vec<Method>;

pub struct Method {}

impl Traveler<Methods> for Methods {
    fn read<I>(seq: &mut I) -> Methods
    where
        I: Iterator<Item = u8>,
    {
        Vec::<Method>::with_capacity(1)
    }
}
