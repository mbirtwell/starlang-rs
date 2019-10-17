use std::io::{self, Write};
use std::process::exit;

extern crate ansi_term;
extern crate lalrpop_util;
#[allow(unused_imports)] // used by tests
#[macro_use]
extern crate indoc;

extern crate argparse;
use argparse::{ArgumentParser, Collect, Store};

pub mod ast;
#[rustfmt::skip]
#[allow(unused_parens)]
pub mod grammar;
mod exec_tree;
mod lexer;
use lexer::Matcher;
mod error;
use error::*;
use file_data::{FileData, FileHandle};

mod file_data;

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
            &["--stdlib"],
            Store,
            "path to the standard library to load. Default stdlib.sl",
        );
        parser.refer(&mut script_path).add_argument(
            "script_path",
            Store,
            "path to the script to run",
        );
        parser
            .refer(&mut args)
            .add_argument("args", Collect, "The args passed to the script");
        parser.parse_args_or_exit()
    }
    args.insert(0, script_path.clone());
    let exit_status = match run(stdlib_path, script_path, args) {
        Ok(n) => n,
        Err(OuterError::FailedInitAnsiTerm(err_code)) => {
            writeln!(
                io::stderr(),
                "Failed to initialise ansi terminal support. err code: {}",
                err_code,
            )
            .unwrap();
            254
        }
        Err(_) => 254,
    };
    exit(exit_status);
}

fn run(stdlib_path: String, script_path: String, args: Vec<String>) -> OuterResult<i32> {
    ansi_term::enable_ansi_support().map_err(|e| OuterError::FailedInitAnsiTerm(e))?;
    let mut files = FileData::new();
    let stdlib_handle = files.read(stdlib_path)?;
    let script_handle = files.read(script_path)?;
    {
        let mut programme = parse_file(stdlib_handle, &files)?;
        programme.extend(parse_file(script_handle, &files)?);
        let stdin = io::stdin();
        let stdout = io::stdout();
        {
            let mut stdin_lock = stdin.lock();
            let mut stdout_lock = stdout.lock();
            match exec_tree::exec(&programme, args, &mut stdin_lock, &mut stdout_lock) {
                Err(err) => {
                    let stderr = io::stderr();
                    write_exec_error(&mut stderr.lock(), &err, &files)?;
                    Err(err.into())
                }
                Ok(i) => Ok(i),
            }
        }
    }
}

fn parse_file(file: FileHandle, files: &FileData) -> OuterResult<Vec<ast::Function<'_>>> {
    let lexer = Matcher::new(file, files.get_contents(file));
    match grammar::parse_Programme(lexer) {
        Ok(rv) => Ok(rv),
        Err(err) => {
            let stderr = io::stderr();
            write_parse_error(&mut stderr.lock(), err, files)?;
            Err(OuterError::ParseError)
        }
    }
}
