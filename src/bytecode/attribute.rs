use super::Traveler;
use bytecode::atom::*;

pub type Attributes = Vec<Attribute>;

pub struct Attribute {}

impl Traveler<Attributes> for Attributes {
    fn read<I>(seq: &mut I) -> Attributes
    where
        I: Iterator<Item = u8>,
    {
        Vec::<Attribute>::with_capacity(1)
    }
}
