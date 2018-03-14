use bytecode::atom::*;
use bytecode::*;

pub struct Class {}

impl Class {
    pub fn from_vec(bytes: Vec<u8>) -> Class {
        let ite = &mut bytes.into_iter();
        assert_eq!(255, U1::read(ite));
        assert_eq!(257, U4::read(ite));
        assert_eq!(512, U2::read(ite));
        Class {}
    }
}

#[test]
fn class_test() {
    let clz = class::Class::from_vec(vec![255, 0, 0, 1, 1, 2, 0]);
}
