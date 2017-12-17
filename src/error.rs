use std;
use std::io::{self, Write};
use lalrpop_util;
use ansi_term::Colour::{Red};
use lexer::{self, Location, Tok};

use super::FileContents;

#[derive(Debug)]
pub enum OuterError {
    ReadInput,
    ParseError,
    OutputError,
    FailedInitAnsiTerm(u64),
}
pub type OuterResult<T> = std::result::Result<T, OuterError>;
pub type ParseError<'input> = lalrpop_util::ParseError<
    Location<'input>, Tok<'input>, lexer::Error<'input>
>;

macro_rules! error {
    ( $out:expr, $fmt:expr ) => {
        writeln!($out, concat!("{}: ", $fmt), Red.paint("error"));
    };
    ( $out:expr, $fmt:expr, $( $arg:expr ),* ) => {
        writeln!($out, concat!("{}: ", $fmt), Red.paint("error"), $( $arg, )*);
    };
}

pub fn write_parse_error(f: &mut Write, err: ParseError, contents: &FileContents) -> OuterResult<()> {
    write_parse_error_inner(f, err, contents).map_err(|_| OuterError::OutputError )
}

fn write_parse_error_inner(f: &mut Write, err: ParseError, contents: &FileContents) -> io::Result<()> {
    use lalrpop_util::ParseError::*;
    match err {
        InvalidToken{ ref location } => {
            error!(f, "Invalid token")?;
            write_location(f, location, contents)?;
        },
        UnrecognizedToken{ ref token, ref expected } => {
            match *token {
                Some((ref start, ref token, ref end)) => {
                    error!(f, "Unrecognized token {:?}", token)?;
                    write_locations(f, start, end, contents)?;
                }
                None => error!(f, "Unrecognized EOF")?,
            }
            if !expected.is_empty() {
                for (i, e) in expected.iter().enumerate() {
                    let sep = match i {
                        0 => "Expected one of",
                        _ if i < expected.len() - 1 => ",",
                        // Last expected message to be written
                        _ => " or",
                    };
                    writeln!(f, "{} {}", sep, e)?;
                }
            }
        }
        ExtraToken { token: (ref start, ref token, ref end) } => {
            error!(f, "Extra token {:?}", token)?;
            write_locations(f, start, end, contents)?;

        }
        User { ref error } => {
            error!(f, "{}", error.kind)?;
            write_location(f, &error.location, contents)?;
        }
    }
    Ok(())
}

fn write_location(f: &mut Write, location: &Location, contents: &FileContents) -> io::Result<()> {
    writeln!(f, "At: {}:{}", location.file_name, location.line)?;
    if let Some(file_content) = contents.get(location.file_name) {
        let line_start = if let Some(mut prev_line_end) = file_content[..location.file_offset_bytes].rfind('\n') {
            let line_start = prev_line_end + 1;
            if file_content[..prev_line_end].ends_with('\r') {
                prev_line_end -= 1;
            }
            let prev_line_start = file_content[..prev_line_end].rfind('\n').map(|v| v + 1).unwrap_or(0);
            writeln!(f, "{} |{}", location.line - 1, &file_content[prev_line_start..prev_line_end])?;
            line_start
        } else {
            0
        };
        let line_end = file_content[location.file_offset_bytes..]
            .find(|c| c == '\n' || c == '\r')
            .map(|v| location.file_offset_bytes + v)
            .unwrap_or(file_content.len());
        writeln!(
            f, "{} |{}{}{}",
            location.line,
            &file_content[line_start..location.file_offset_bytes],
            Red.bold().paint(&file_content[location.file_offset_bytes..location.file_offset_bytes+1]),
            &file_content[location.file_offset_bytes+1..line_end],
        )?;
    }
    Ok(())
}

fn write_locations(f: &mut Write, start: &Location, end: &Location, contents: &FileContents) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::super::lexer::Matcher;
    use super::super::grammar::parse_Programme;
    use super::*;

    macro_rules! test_parse_error {
        ($test_name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $test_name() {
                let text = $input;
                let file_name = "test.sl";
                let contents = {
                    let mut contents = HashMap::new();
                    contents.insert(file_name, text);
                    contents
                };
                let matcher = Matcher::new(file_name, text);
                let parse_err = parse_Programme(matcher).unwrap_err();
                let mut output = Vec::new();
                write_parse_error(&mut output, parse_err, &contents).unwrap();
                assert_eq!(String::from_utf8(output).unwrap(), $expected)
            }
        };
    }

    test_parse_error!{eof_in_string_literal, "\
function main() {
    let a = \"sdsds
}
", format!("\
{}: Found end of file whilst looking for end of string literal
At: test.sl:2
1 |function main() {{
2 |    let a = {}sdsds
", Red.paint("error"), Red.bold().paint("\""))}

    test_parse_error!{windows_line_endings, "\
function main() {\r
    let a = \"sdsds\r
}\r
", format!("\
{}: Found end of file whilst looking for end of string literal
At: test.sl:2
1 |function main() {{
2 |    let a = {}sdsds
", Red.paint("error"), Red.bold().paint("\""))}


}