use super::Traveler;
use bytecode::atom::*;

pub type Interfaces = Vec<U2>;

impl Traveler<Interfaces> for Interfaces {
    fn read<I>(seq: &mut I) -> Interfaces
    where
        I: Iterator<Item = u8>,
    {
        let size = U2::read(seq);
        let mut interfaces = Vec::<U2>::with_capacity(size as usize);
        for _x in 0..size {
            interfaces.push(U2::read(seq));
        }
        interfaces
    }
}
