use super::file_data::FileHandle;
use super::grammar;
use super::lexer::Matcher;

macro_rules! test_expr {
    ($text:expr, $ast:expr) => {
        assert_eq!(
            &format!(
                "{:?}",
                grammar::parse_Expr(Matcher::new(FileHandle::dummy(), $text)).unwrap()
            ),
            $ast
        );
    };
}

macro_rules! test_stmt {
    ($text:expr, $ast:expr) => {
        assert_eq!(
            &format!(
                "{:?}",
                grammar::parse_Statement(Matcher::new(FileHandle::dummy(), $text)).unwrap()
            ),
            $ast
        );
    };
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
    test_expr!(
        "0 and 1 + 0 or 0",
        "BinaryOp(BinaryOp(0 and BinaryOp(1 + 0)) or 0)"
    )
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
    test_expr!(
        "1 and not 0 + 1",
        "BinaryOp(1 and UnaryOp(not BinaryOp(0 + 1)))"
    )
}

#[test]
fn comparision_op() {
    test_expr!("3 < 4", "BinaryOp(3 < 4)")
}

#[test]
fn chained_comparision_op_not_allowed() {
    assert!(grammar::parse_Expr(Matcher::new(FileHandle::dummy(), "3 < 4 < 5")).is_err())
}

#[test]
fn or_expr() {
    test_expr!("3 < 4 | 8", "BinaryOp(3 < BinaryOp(4 | 8))")
}

#[test]
fn and_and_xor_expr() {
    test_expr!(
        "5 & 1 | 2 ^ 3",
        "BinaryOp(BinaryOp(5 & 1) | BinaryOp(2 ^ 3))"
    )
}

#[test]
fn shift_operators() {
    test_expr!(
        "4 + 5 << 3 & 2 >> 1",
        "BinaryOp(BinaryOp(BinaryOp(4 + 5) << 3) & BinaryOp(2 >> 1))"
    )
}

#[test]
fn u_expr() {
    test_expr!("~4 % -3", "BinaryOp(UnaryOp(~ 4) % UnaryOp(- 3))")
}

#[test]
fn call() {
    test_expr!(
        "func(1, 2, 3)",
        "Call(function: func, arguments: [1, 2, 3])"
    )
}

#[test]
fn identifier_expr() {
    test_expr!("3 + var1", "BinaryOp(3 + Identifier(var1))")
}

#[test]
fn subscription() {
    test_expr!(
        "array[1 + 2]",
        "Subscription(array_expr: Identifier(array), subscript_expr: BinaryOp(1 + 2))"
    )
}

#[test]
fn array_literal() {
    test_expr!("[1, 2, var1]", "Array([1, 2, Identifier(var1)])")
}

#[test]
fn character_literal() {
    test_expr!("'a'", "Char('a')")
}

#[test]
fn string_literal() {
    test_expr!("\"str\"", "String(\"str\")")
}

#[test]
fn return_statement() {
    test_stmt!("return 4;", "Return(4)")
}

#[test]
fn expression_statement() {
    test_stmt!(
        "func(arr, 1);",
        "Expr(Call(function: func, arguments: [Identifier(arr), 1]))"
    )
}

#[test]
fn assignment_statement() {
    test_stmt!(
        "var = 2 + var2;",
        "Assign(target: Identifier(var), expr: BinaryOp(2 + Identifier(var2)))"
    )
}

#[test]
fn declaration_statement() {
    test_stmt!("let var = 3;", "Declare(identifier: var, expr: 3)")
}

#[test]
fn if_statemenet() {
    test_stmt!(
        "if a == 2 {
            f();
        }",
        "If(test: BinaryOp(Identifier(a) == 2), block: [Expr(Call(function: f, arguments: []))])"
    )
}

#[test]
fn while_statemenet() {
    test_stmt!(
        "while a == 2 {
            f();
        }",
        "While(test: BinaryOp(Identifier(a) == 2), block: [Expr(Call(function: f, arguments: []))])"
    )
}

#[test]
fn function_definition() {
    let text = "\
        function fname(arg1) {
            return 1;
        }
    ";
    let actual = &format!(
        "{:?}",
        grammar::parse_Function(Matcher::new(FileHandle::dummy(), text)).unwrap()
    );
    let expected = "Function(name: fname, arguments: [arg1], stmts: [Return(1)])";
    assert_eq!(actual, expected);
}

#[test]
fn programme() {
    let text = "\
        function fname(arg1) {
            return 1;
        }

        function main(args) {
            let a = fname(args[0]);
            return a + 1;
        }
    ";
    let actual = &format!(
        "{:?}",
        grammar::parse_Programme(Matcher::new(FileHandle::dummy(), text)).unwrap()
    );
    let expected = "[\
                    Function(name: fname, arguments: [arg1], stmts: [Return(1)]), \
                    Function(name: main, arguments: [args], stmts: [\
                    Declare(identifier: a, expr: Call(function: fname, arguments: [Subscription(\
                    array_expr: Identifier(args), \
                    subscript_expr: 0\
                    )])), \
                    Return(BinaryOp(Identifier(a) + 1))\
                    ])]";
    assert_eq!(actual, expected);
}
