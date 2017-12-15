use std::process::exit;
use std::io::{self,Read,Write};
use std::fs;

extern crate lalrpop_util;
#[macro_use]
extern crate indoc;

extern crate argparse;
use argparse::{ArgumentParser, Store, Collect};

pub mod ast;
pub mod grammar;
mod comment_stripper;
mod exec_tree;
mod lexer;
use lexer::Matcher;
mod error;
use error::{OuterResult, OuterError, write_parse_error};

#[cfg(test)]
mod test_grammar;

fn main() {
    let mut stdlib_path = "stdlib.sl".to_string();
    let mut script_path = String::new();
    let mut args: Vec<String> = Vec::new();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Mike's first StarLang iterpreter using an executable AST");
        parser.refer(&mut stdlib_path).add_option(
            &["--stdlib"], Store, "path to the standard library to load. Default stdlib.sl",
        );
        parser.refer(&mut script_path).add_argument(
            "script_path", Store, "path to the script to run",
        );
        parser.refer(&mut args).add_argument(
            "args", Collect, "The args passed to the script",
        );
        parser.parse_args_or_exit()
    }
    args.insert(0, script_path.clone());
    let exit_status = match run(&stdlib_path, &script_path, args) {
        Ok(n) => n,
        Err(_) => 254,
    };
    exit(exit_status);
}

fn run<'filename>(stdlib_path: &'filename str, script_path: &'filename str, args: Vec<String>) -> OuterResult<i32> {
    let stdlib_contents = read_file(stdlib_path)?;
    let script_contents = read_file(script_path)?;
    let mut programme = parse_file(stdlib_path, &stdlib_contents)?;
    programme.extend(parse_file(script_path, &script_contents)?);
    let stdin = io::stdin();
    let stdout = io::stdout();
    {
        let mut stdin_lock = stdin.lock();
        let mut stdout_lock = stdout.lock();
        Ok(exec_tree::exec(&programme, args, &mut stdin_lock, &mut stdout_lock))
    }
}

fn read_file_inner(path: &str) -> io::Result<String> {
    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn read_file(path: &str) -> OuterResult<String> {
    match read_file_inner(path) {
        Ok(rv) => Ok(rv),
        Err(err) => {
            writeln!(io::stderr(), "error: Failed to read file '{}': {}", path, err).unwrap();
            Err(OuterError::ReadInput)
        }
    }
}

fn parse_file(path: &str, contents: &str) -> OuterResult<Vec<ast::Function>> {
    let lexer = Matcher::new(path, &contents);
    match grammar::parse_Programme(lexer) {
        Ok(rv) => Ok(rv),
        Err(err) => {
            let stderr = io::stderr();
            write_parse_error(&mut stderr.lock(), err)?;
            Err(OuterError::ParseError)
        },
    }
}
