use super::main::exec;
use super::super::grammar::parse_Programme;

struct ProgResult {
    status_code: i32,
    output: Vec<u8>,
}

fn compile_and_run_programme(text: &str) -> ProgResult {
    compile_and_run_programme_with_args_and_input(text, Vec::new(), &[])
}

fn compile_and_run_programme_with_args(text: &str, args: Vec<String>) -> ProgResult {
    compile_and_run_programme_with_args_and_input(text, args, &[])
}

fn compile_and_run_programme_with_input(text: &str, input: &'static [u8]) -> ProgResult {
    compile_and_run_programme_with_args_and_input(text, Vec::new(), input)
}

fn compile_and_run_programme_with_args_and_input(text: &str, args: Vec<String>, input: &'static [u8]) -> ProgResult {
    let prog = parse_Programme(text).unwrap();
    let mut output = Vec::new();
    let status_code = {
        exec(&prog, args, Box::new(input), &mut output)
    };
    ProgResult { status_code: status_code, output: output }
}

#[test]
fn noop_programme() {
    let result = compile_and_run_programme("\
            function main (args) {
            }
        ");
    assert_eq!(result.status_code, 0);
}

#[test]
fn main_return_status_code() {
    let result = compile_and_run_programme("\
            function main (args) {
                return 3;
            }
        ");
    assert_eq!(result.status_code, 3);
}

#[test]
fn return_expression() {
    let result = compile_and_run_programme("\
            function main (args) {
                return 2 + 3;
            }
        ");
    assert_eq!(result.status_code, 5);
}

#[test]
fn return_more_maths() {
    let result = compile_and_run_programme("\
            function main (args) {
                return (2 * 5 - 1) % 5 ;
            }
        ");
    assert_eq!(result.status_code, 4);
}

#[test]
fn return_division() {
    let result = compile_and_run_programme("\
            function main (args) {
                return 5 / 2;
            }
        ");
    assert_eq!(result.status_code, 2);
}

#[test]
fn return_bit_manipulation() {
    let result = compile_and_run_programme("\
            function main (args) {
                return 1 << 2 | 64 >> 3 | 255 & 64 | 255 - 32 ^ 255;
            }
        ");
    assert_eq!(result.status_code, 0x6c);
}

#[test]
fn declare_and_return() {
    let result = compile_and_run_programme("\
            function main (args) {
                let a = 42;
                return a;
            }
        ");
    assert_eq!(result.status_code, 42);
}

#[test]
fn declare_update_and_return() {
    let result = compile_and_run_programme("\
            function main (args) {
                let a = 42;
                a = a + 1;
                return a;
            }
        ");
    assert_eq!(result.status_code, 43);
}

#[test]
fn function_call() {
    let result = compile_and_run_programme("\
            function f1() {
                return 43;
            }
            function main (args) {
                return f1();
            }
        ");
    assert_eq!(result.status_code, 43);
}

#[test]
fn function_with_arguments_call() {
    let result = compile_and_run_programme("\
            function f1(a) {
                return a + 1;
            }
            function main (args) {
                return f1(40) + 1;
            }
        ");
    assert_eq!(result.status_code, 42);
}

#[test]
fn array_literal_and_subscription() {
    let result = compile_and_run_programme("\
            function main (args) {
                let a = [42];
                return a[0];
            }
        ");
    assert_eq!(result.status_code, 42);
}

#[test]
fn assigning_to_a_subscript() {
    let result = compile_and_run_programme("\
            function main (args) {
                let a = [42];
                a[0] = a[0] * 2;
                return a[0];
            }
        ");
    assert_eq!(result.status_code, 84);
}

#[test]
fn access_cmd_line_args() {
    let result = compile_and_run_programme_with_args("\
            function main (args) {
                return args[0][0];
            }
        ", vec!["a".to_string()]);
    assert_eq!(result.status_code, b'a' as i32)
}

#[test]
fn call_new_platform_func() {
    let result = compile_and_run_programme("\
            function main (args) {
                let a = new(3);
                a[2] = 24;
                return a[2] * 2;
            }
        ");
    assert_eq!(result.status_code, 48);
}

#[test]
fn call_getc_function_to_get_input() {
    let result = compile_and_run_programme_with_input("\
            function main (args) {
                return getc();
            }
        ", b"a");
    assert_eq!(result.status_code, b'a' as i32);
}

#[test]
fn call_putc_to_produce_output() {
    let result = compile_and_run_programme("\
            function main (args) {
                putc('a');
            }
        ");
    assert_eq!(result.output, b"a");

}

//    #[test]
//    fn comparisions() {
//        let result = compile_and_run_programme("\
//            function main (args) {
//                return
//                    (1 < 3) << 0
//                    (4 < 3) << 1
//                    (4 > 3) << 2
//                    (4 > 6) << 3
//                    (4 > 6) << 3
//                ;
//            }
//        ");
//        assert_eq!(result.status_code, 0x6c);
//    }
