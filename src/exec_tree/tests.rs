use super::main::exec;
use super::super::grammar::parse_Programme;

struct ProgResult {
    status_code: i32,
}

fn compile_and_run_programme(text: &str) -> ProgResult {
    let prog = parse_Programme(text).unwrap();
    ProgResult { status_code: exec(&prog) }
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
fn declare_and_reurn() {
    let result = compile_and_run_programme("\
            function main (args) {
                let a = 42;
                return a;
            }
        ");
    assert_eq!(result.status_code, 42);
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