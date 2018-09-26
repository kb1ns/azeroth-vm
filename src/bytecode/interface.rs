use super::Traveler;
use bytecode::atom::*;

pub type Interfaces = Vec<Interface>;

pub struct Interface {}

impl Traveler<Interfaces> for Interfaces {
    fn read<I>(seq: &mut I) -> Interfaces
    where
        I: Iterator<Item = u8>,
    {
        Vec::<Interface>::with_capacity(1)
    }
}
