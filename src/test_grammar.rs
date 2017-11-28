
use super::grammar;

macro_rules! test_expr {
    ($text:expr, $ast:expr) => {
        assert_eq!(&format!("{:?}", grammar::parse_Expr($text).unwrap()), $ast);
    }
}

#[test]
fn nested_add_mul_expr() {
    test_expr!("22* 44 + 66", "BinaryOp(BinaryOp(22 * 44) + 66)");
}

#[test]
fn bracketed_add_expr() {
    test_expr!("22 * (44 + 66)", "BinaryOp(22 * BinaryOp(44 + 66))");
}

#[test]
fn or_test_expr() {
    test_expr!("0 + 1 or 1", "BinaryOp(BinaryOp(0 + 1) or 1)")
}

#[test]
fn and_test_expr() {
    test_expr!("0 and 1 + 0 or 0", "BinaryOp(BinaryOp(0 and BinaryOp(1 + 0)) or 0)")
}

#[test]
fn not_test_expr() {
    test_expr!("not 1", "UnaryOp(not 1)")
}

#[test]
fn double_not_expr() {
    test_expr!("not not 1", "UnaryOp(not UnaryOp(not 1))")
}

#[test]
fn nested_not_expr() {
    test_expr!("1 and not 0 + 1", "BinaryOp(1 and UnaryOp(not BinaryOp(0 + 1)))")
}

#[test]
fn comparision_op() {
    test_expr!("3 < 4", "BinaryOp(3 < 4)")
}

#[test]
fn chained_comparision_op_not_allowed() {
    assert!(grammar::parse_Expr("3 < 4 < 5").is_err())
}

#[test]
fn or_expr() {
    test_expr!("3 < 4 | 8", "BinaryOp(3 < BinaryOp(4 | 8))")
}

#[test]
fn and_and_xor_expr() {
    test_expr!("5 & 1 | 2 ^ 3", "BinaryOp(BinaryOp(5 & 1) | BinaryOp(2 ^ 3))")
}