use super::Traveler;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;
pub type U8 = u64;

impl Traveler<U1> for U1 {
    fn read<I>(seq: &mut I) -> U1
    where
        I: Iterator<Item = u8>,
    {
        let u = seq.next();
        match u {
            None => panic!("invalid classfile"),
            Some(uu) => uu,
        }
    }
}

impl Traveler<U2> for U2 {
    fn read<I>(seq: &mut I) -> U2
    where
        I: Iterator<Item = u8>,
    {
        let u0 = seq.next();
        let u1 = seq.next();
        match u0 {
            None => panic!("invalid classfile"),
            Some(uu0) => match u1 {
                None => panic!("invalid classfile"),
                Some(uu1) => ((uu0 as u16) << 8) + (uu1 as u16),
            },
        }
    }
}

impl Traveler<U4> for U4 {
    fn read<I>(seq: &mut I) -> U4
    where
        I: Iterator<Item = u8>,
    {
        let u0 = seq.next();
        let u1 = seq.next();
        let u2 = seq.next();
        let u3 = seq.next();
        match u0 {
            None => panic!("invalid classfile"),
            Some(uu0) => match u1 {
                None => panic!("invalid classfile"),
                Some(uu1) => match u2 {
                    None => panic!("invalid classfile"),
                    Some(uu2) => match u3 {
                        None => panic!("invalid classfile"),
                        Some(uu3) => ((uu0 as u32) << 24) + ((uu1 as u32) << 16) + ((uu2 as u32) << 8) + (uu3 as u32),
                    },
                },
            },
        }
    }
}

impl Traveler<U8> for U8 {
    fn read<I>(seq: &mut I) -> U8
    where
        I: Iterator<Item = u8>,
    {
        let u0 = seq.next();
        let u1 = seq.next();
        let u2 = seq.next();
        let u3 = seq.next();
        let u4 = seq.next();
        let u5 = seq.next();
        let u6 = seq.next();
        let u7 = seq.next();
        match u0 {
            None => panic!("invalid classfile"),
            Some(uu0) => match u1 {
                None => panic!("invalid classfile"),
                Some(uu1) => match u2 {
                    None => panic!("invalid classfile"),
                    Some(uu2) => match u3 {
                        None => panic!("invalid classfile"),
                        Some(uu3) => match u4 {
                            None => panic!("invalid classfile"),
                            Some(uu4) => match u5 {
                                None => panic!("invalid classfile"),
                                Some(uu5) => match u6 {
                                    None => panic!("invalid classfile"),
                                    Some(uu6) => match u7 {
                                        None => panic!("invalid classfile"),
                                        Some(uu7) => ((uu0 as u64) << 56) + ((uu1 as u64) << 48) + ((uu2 as u64) << 40) + ((uu3 as u64) << 32) + ((uu4 as u64) << 24) + ((uu5 as u64) << 16) + ((uu6 as u64) << 8) + (uu7 as u64),
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }
}
