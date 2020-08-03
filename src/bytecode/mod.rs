pub mod atom;
pub mod attribute;
pub mod class;
pub mod constant_pool;
pub mod field;
pub mod interface;
pub mod method;

use self::atom::*;
use self::constant_pool::ConstantPool;

trait Traveler<T> {
    fn read<I>(seq: &mut I, constants: Option<&ConstantPool>) -> T
    where
        I: Iterator<Item = u8>;
}

pub const METHOD_ACC_STATIC: U2 = 0x0008;

pub fn resolve_method_descriptor(
    descriptor: &str,
    access_flag: U2,
) -> (Vec<String>, usize, String) {
    let t = descriptor
        .chars()
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut params: Vec<String> = vec![];
    let mut expect_type: bool = false;
    let mut expect_semicolon: bool = false;
    let mut token: String = String::new();
    let instance = if access_flag & METHOD_ACC_STATIC == METHOD_ACC_STATIC {
        0
    } else {
        1
    };
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
                let slots = params
                    .iter()
                    .map(|x| match x.as_ref() {
                        "D" | "J" => 2,
                        _ => 1,
                    })
                    .sum::<usize>()
                    + instance;
                return (params, slots, t[i + 1..].join(""));
            }
            '[' => {
                expect_type = true;
                token.push('[');
            }
            'L' => {
                expect_semicolon = true;
                token.push('L');
            }
            'I' | 'J' | 'S' | 'B' | 'C' | 'F' | 'D' | 'Z' => {
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
    let (params, slots, ret) =
        resolve_method_descriptor("(Ljava/lang/String;IJ)V", METHOD_ACC_STATIC);
    assert_eq!(ret, "V");
    assert_eq!(slots, 4);
    assert_eq!(params, vec!["Ljava/lang/String;", "I", "J"]);
    let (params, slots, ret) =
        resolve_method_descriptor("([[I[J[[Ljava/lang/IString;)[Ljava/lang/String;", 0);
    assert_eq!(ret, "[Ljava/lang/String;");
    assert_eq!(slots, 4);
    assert_eq!(params, vec!["[[I", "[J", "[[Ljava/lang/IString;"]);
    let (params, slots, ret) =
        resolve_method_descriptor("([Ljava/lang/String;)V", METHOD_ACC_STATIC);
    assert_eq!(params, vec!["[Ljava/lang/String;"]);
    assert_eq!(slots, 1);
    assert_eq!(ret, "V");
}
