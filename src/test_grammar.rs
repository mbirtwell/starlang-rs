
use super::grammar;

macro_rules! test_expr {
    ($text:expr, $ast:expr) => {
        assert_eq!(&format!("{:?}", grammar::parse_Expr($text).unwrap()), $ast);
    }
}

#[test]
fn nested_add_mul_expr() {
    test_expr!("22* 44 + 66", "Expr(Expr(22 * 44) + 66)");
}

#[test]
fn bracketed_add_expr() {
    test_expr!("22 * (44 + 66)", "Expr(22 * Expr(44 + 66))");
}

#[test]
fn or_test_expr() {
    test_expr!("0 + 1 or 1", "Expr(Expr(0 + 1) or 1)")
}

#[test]
fn and_test_expr() {
    test_expr!("0 and 1 + 0 or 0", "Expr(Expr(0 and Expr(1 + 0)) or 0)")
}