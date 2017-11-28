extern crate lalrpop_util;

pub mod ast;

pub mod grammar;

#[test]
fn calculator5() {
    assert_eq!(&format!("{:?}", grammar::parse_Exprs("").unwrap()),
               "[]");
    assert_eq!(&format!("{:?}", grammar::parse_Exprs("22 * 44 + 66").unwrap()),
               "[((22 * 44) + 66)]");
    assert_eq!(&format!("{:?}", grammar::parse_Exprs("22 * 44 + 66,").unwrap()),
               "[((22 * 44) + 66)]");
    assert_eq!(&format!("{:?}", grammar::parse_Exprs("22 * 44 + 66, 13*3").unwrap()),
               "[((22 * 44) + 66), (13 * 3)]");
    assert_eq!(&format!("{:?}", grammar::parse_Exprs("22 * 44 + 66, 13*3,").unwrap()),
               "[((22 * 44) + 66), (13 * 3)]");
}


#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
