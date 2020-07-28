pub mod atom;
pub mod attribute;
pub mod class;
pub mod constant_pool;
pub mod field;
pub mod interface;
pub mod method;

use self::atom::*;
use self::constant_pool::ConstantPool;
use crate::mem::*;

trait Traveler<T> {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> T
    where
        I: Iterator<Item = u8>;
}

pub const JVM_VOID: char = 'V';
pub const JVM_BYTE: char = 'B';
pub const JVM_CHAR: char = 'C';
pub const JVM_DOUBLE: char = 'D';
pub const JVM_FLOAT: char = 'F';
pub const JVM_INT: char = 'I';
pub const JVM_LONG: char = 'J';
pub const JVM_SHORT: char = 'S';
pub const JVM_BOOLEAN: char = 'Z';
pub const JVM_REF: char = 'L';
pub const JVM_ARRAY: char = '[';

pub const METHOD_ACC_STATIC: U2 = 0x0008;

pub fn literal_size(desc: &str) -> usize {
    match desc {
        "D" | "J" => 8,
        "B" | "Z" => 1,
        "S" => 2,
        "C" | "I" | "F" => 4,
        _ => 0,
    }
}

pub fn resolve_method_descriptor(descriptor: &str) -> (Vec<String>, String) {
    let t = descriptor
        .chars()
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut params: Vec<String> = vec![];
    let mut expect_type: bool = false;
    let mut expect_semicolon: bool = false;
    let mut token: String = String::new();
    for (i, ch) in descriptor.chars().enumerate() {
        if expect_semicolon {
            token.push(ch);
            if ch == ';' {
                expect_semicolon = false;
                expect_type = false;
                params.push(token.clone());
                token.clear();
            }
            continue;
        }
        match ch {
            '(' => {
                if expect_type {
                    panic!(format!("Illegal method descriptor: {}", descriptor));
                }
                continue;
            }
            ')' => {
                if expect_type {
                    panic!(format!("Illegal method descriptor: {}", descriptor));
                }
                return (params, t[i + 1..].join(""));
            }
            JVM_ARRAY => {
                expect_type = true;
                token.push('[');
            }
            JVM_REF => {
                expect_semicolon = true;
                token.push('L');
            }
            JVM_BYTE | JVM_CHAR | JVM_FLOAT | JVM_DOUBLE | JVM_INT | JVM_LONG | JVM_SHORT
            | JVM_BOOLEAN => {
                if expect_type {
                    token.push(ch);
                    params.push(token.clone());
                    token.clear();
                    expect_type = false;
                } else {
                    params.push(ch.to_string());
                }
            }
            _ => {
                if expect_semicolon {
                    token.push(ch);
                } else {
                    panic!(format!("Illegal method descriptor: {}", descriptor));
                }
            }
        }
    }
    panic!(format!("Illegal method descriptor: {}", descriptor));
}

#[test]
pub fn test_resolve_method() {
    let (params, ret) = resolve_method_descriptor("(Ljava/lang/String;IJ)V");
    assert_eq!(ret, "V");
    assert_eq!(params, vec!["Ljava/lang/String;", "I", "J"]);
    let (params, ret) = resolve_method_descriptor("([IJLjava/lang/String;)[Ljava/lang/String;");
    assert_eq!(ret, "[Ljava/lang/String;");
    assert_eq!(params, vec!["[I", "J", "Ljava/lang/String;"]);
    let (params, ret) = resolve_method_descriptor("([Ljava/lang/String;)V");
    assert_eq!(params, vec!["[Ljava/lang/String;"]);
    assert_eq!(ret, "V");
}
