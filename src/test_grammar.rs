
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