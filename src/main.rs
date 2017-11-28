extern crate lalrpop_util;

pub mod ast;

pub mod calculator5;

#[test]
fn calculator5() {
    assert_eq!(&format!("{:?}", calculator5::parse_Exprs("").unwrap()),
               "[]");
    assert_eq!(&format!("{:?}", calculator5::parse_Exprs("22 * 44 + 66").unwrap()),
               "[((22 * 44) + 66)]");
    assert_eq!(&format!("{:?}", calculator5::parse_Exprs("22 * 44 + 66,").unwrap()),
               "[((22 * 44) + 66)]");
    assert_eq!(&format!("{:?}", calculator5::parse_Exprs("22 * 44 + 66, 13*3").unwrap()),
               "[((22 * 44) + 66), (13 * 3)]");
    assert_eq!(&format!("{:?}", calculator5::parse_Exprs("22 * 44 + 66, 13*3,").unwrap()),
               "[((22 * 44) + 66), (13 * 3)]");
}


#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
