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
use comment_stripper::strip_comments;
mod exec_tree;

#[cfg(test)]
mod test_grammar;

type Error = io::Error;
type Result<T> = std::result::Result<T, Error>;

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
        Err(ref err) => {
            writeln!(io::stderr(), "Failed: {}", err).unwrap();
            1
        }
    };
    exit(exit_status);
}

fn run(stdlib_path: &str, script_path: &str, args: Vec<String>) -> Result<i32> {
    let mut programme = parse_file(stdlib_path)?;
    programme.extend(parse_file(script_path)?);
    let stdin = io::stdin();
    let stdout = io::stdout();
    {
        let mut stdin_lock = stdin.lock();
        let mut stdout_lock = stdout.lock();
        Ok(exec_tree::exec(&programme, args, &mut stdin_lock, &mut stdout_lock))
    }
}

fn parse_file(path: &str) -> Result<Vec<ast::Function>> {
    let mut file = fs::OpenOptions::new().read(true).open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    contents = strip_comments(&contents);
    Ok(grammar::parse_Programme(&contents).unwrap())
}
