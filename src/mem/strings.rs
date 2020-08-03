use std::collections::HashMap;
use super::Ref;


pub type Strings = HashMap<String, Ref>;

pub static mut STRINGS: Strings = HashMap::with_capacity(4096);

impl Strings {

    pub fn get(string: &str) -> Ref {
        0
    }
}
