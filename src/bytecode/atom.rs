use super::Traveler;
use std::mem;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;
pub type U8 = u64;

impl Traveler<U1> for U1 {
    fn read<I>(seq: &mut I) -> U1
    where
        I: Iterator<Item = u8>,
    {
        seq.next().unwrap()
    }
}

impl Traveler<U2> for U2 {
    fn read<I>(seq: &mut I) -> U2
    where
        I: Iterator<Item = u8>,
    {
        let u0 = seq.next().unwrap();
        let u1 = seq.next().unwrap();
        let u = [u1, u0];
        unsafe {
            mem::transmute::<[u8; 2], u16>(u)
        }
    }
}

impl Traveler<U4> for U4 {
    fn read<I>(seq: &mut I) -> U4
    where
        I: Iterator<Item = u8>,
    {
        let u0 = seq.next().unwrap();
        let u1 = seq.next().unwrap();
        let u2 = seq.next().unwrap();
        let u3 = seq.next().unwrap();
        let u = [u3, u2, u1, u0];
        unsafe {
            mem::transmute::<[u8; 4], u32>(u)
        }
    }
}

impl Traveler<U8> for U8 {
    fn read<I>(seq: &mut I) -> U8
    where
        I: Iterator<Item = u8>,
    {
        let u0 = seq.next().unwrap();
        let u1 = seq.next().unwrap();
        let u2 = seq.next().unwrap();
        let u3 = seq.next().unwrap();
        let u4 = seq.next().unwrap();
        let u5 = seq.next().unwrap();
        let u6 = seq.next().unwrap();
        let u7 = seq.next().unwrap();
        let u = [u7, u6, u5, u4, u3, u2, u1, u0];
        unsafe {
            mem::transmute::<[u8; 8], u64>(u)
        }
    }
}
