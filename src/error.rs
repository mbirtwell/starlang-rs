use ansi_term::Colour::Red;
use exec_tree::error::*;
use lalrpop_util;
use lexer::{self, Location, Tok};
use std;
use std::io::{self, Write};

use file_data::FileData;

#[derive(Debug)]
pub enum OuterError {
    ReadInput,
    ParseError,
    StaticAnalysisFailed,
    OutputError,
    FailedInitAnsiTerm(u64),
}
pub type OuterResult<T> = std::result::Result<T, OuterError>;
pub type ParseError<'input> = lalrpop_util::ParseError<Location, Tok<'input>, lexer::Error>;

impl<'a> From<ExecError<'a>> for OuterError {
    fn from(value: ExecError<'a>) -> OuterError {
        match value {
            ExecError::StaticAnalysisFailed(_) => OuterError::StaticAnalysisFailed,
        }
    }
}

macro_rules! error {
    ( $out:expr, $fmt:expr ) => {
        writeln!($out, concat!("{}: ", $fmt), Red.paint("error"));
    };
    ( $out:expr, $fmt:expr, $( $arg:expr ),* ) => {
        writeln!($out, concat!("{}: ", $fmt), Red.paint("error"), $( $arg, )*);
    };
}

pub fn write_parse_error(
    f: &mut dyn Write,
    err: ParseError,
    contents: &FileData,
) -> OuterResult<()> {
    write_parse_error_inner(f, err, contents).map_err(|_| OuterError::OutputError)
}

fn write_parse_error_inner(
    f: &mut dyn Write,
    err: ParseError,
    contents: &FileData,
) -> io::Result<()> {
    use lalrpop_util::ParseError::*;
    match err {
        InvalidToken { ref location } => {
            error!(f, "Invalid token")?;
            write_location(f, location, contents)?;
        }
        UnrecognizedToken {
            ref token,
            ref expected,
        } => {
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
        ExtraToken {
            token: (ref start, ref token, ref end),
        } => {
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

pub fn write_exec_error<'a>(
    f: &mut dyn Write,
    err: &'a ExecError,
    contents: &FileData,
) -> OuterResult<()> {
    write_exec_error_inner(f, err, contents).map_err(|_| OuterError::OutputError)
}

fn write_exec_error_inner<'a>(
    f: &mut dyn Write,
    err: &'a ExecError,
    contents: &FileData,
) -> io::Result<()> {
    match *err {
        ExecError::StaticAnalysisFailed(ref errs) => {
            for static_err in errs {
                write_static_analysis_err(f, static_err, contents)?;
                writeln!(f)?;
            }
        }
    }
    Ok(())
}

fn write_static_analysis_err<'a>(
    f: &mut dyn Write,
    err: &'a StaticAnalysisError,
    contents: &FileData,
) -> io::Result<()> {
    match *err {
        StaticAnalysisError::CallUnknownFunction(fname, start, end) => {
            error!(f, "Call to unknown function {:?}", fname)?;
            write_locations(f, &start, &end, contents)?;
        }
    }
    Ok(())
}

fn write_location(f: &mut dyn Write, location: &Location, contents: &FileData) -> io::Result<()> {
    write_location_at(f, location, contents)?;
    if let Some(file) = location.file {
        let file_content = contents.get_contents(file);
        let line_start = find_line_start(location, file_content);
        write_previous_line(f, location, line_start, file_content)?;
        let line_end = find_line_end(location, file_content);
        write_single_line_error(f, location, line_start, line_end, 1, file_content)?;
        write_next_line(f, location, line_end, file_content)?;
    }
    Ok(())
}

fn write_locations(
    f: &mut dyn Write,
    start: &Location,
    end: &Location,
    contents: &FileData,
) -> io::Result<()> {
    write_location_at(f, start, contents)?;
    if let Some(file) = start.file {
        let file_content = contents.get_contents(file);
        let line_start = find_line_start(start, file_content);
        write_previous_line(f, start, line_start, file_content)?;
        let end_line_end = find_line_end(end, file_content);
        if start.line == end.line {
            write_single_line_error(
                f,
                start,
                line_start,
                end_line_end,
                end.file_offset_bytes - start.file_offset_bytes,
                file_content,
            )?;
        } else {
            let end_start_line = find_line_end(start, file_content);
            let start_end_line = find_line_start(end, file_content);
            writeln!(
                f,
                "{} |{}{}",
                start.line,
                &file_content[line_start..start.file_offset_bytes],
                Red.bold()
                    .paint(&file_content[start.file_offset_bytes..end_start_line]),
            )?;
            if start.line + 1 != end.line {
                for (lineno, line) in file_content[end_start_line..start_end_line]
                    .lines()
                    .skip(1)
                    .enumerate()
                {
                    writeln!(f, "{} |{}", start.line + lineno + 1, Red.bold().paint(line),)?;
                }
            }
            writeln!(
                f,
                "{} |{}{}",
                end.line,
                Red.bold()
                    .paint(&file_content[start_end_line..end.file_offset_bytes]),
                &file_content[end.file_offset_bytes..end_line_end],
            )?;
        }
        write_next_line(f, end, end_line_end, file_content)?;
    }
    Ok(())
}

fn write_location_at(f: &mut dyn Write, location: &Location, files: &FileData) -> io::Result<()> {
    writeln!(
        f,
        "At: {}:{}",
        location
            .file
            .map(|f| files.get_name(f))
            .unwrap_or("unknown"),
        location.line
    )
}

fn find_line_start(location: &Location, file_content: &str) -> usize {
    file_content[..location.file_offset_bytes]
        .rfind('\n')
        .map(|v| v + 1)
        .unwrap_or(0)
}

fn find_line_end(location: &Location, file_content: &str) -> usize {
    file_content[location.file_offset_bytes..]
        .find(|c| c == '\n' || c == '\r')
        .map(|v| location.file_offset_bytes + v)
        .unwrap_or(file_content.len())
}

fn write_previous_line(
    f: &mut dyn Write,
    location: &Location,
    line_start: usize,
    file_content: &str,
) -> io::Result<()> {
    if let Some(prev_line) = file_content[..line_start].lines().next_back() {
        writeln!(f, "{} |{}", location.line - 1, prev_line)?;
    }
    Ok(())
}

fn write_next_line(
    f: &mut dyn Write,
    location: &Location,
    line_end: usize,
    file_content: &str,
) -> io::Result<()> {
    if let Some(next_line) = file_content[line_end..].lines().skip(1).next() {
        writeln!(f, "{} |{}", location.line + 1, next_line)?;
    }
    Ok(())
}

fn write_single_line_error(
    f: &mut dyn Write,
    start: &Location,
    line_start: usize,
    line_end: usize,
    len: usize,
    file_content: &str,
) -> io::Result<()> {
    writeln!(
        f,
        "{} |{}{}{}",
        start.line,
        &file_content[line_start..start.file_offset_bytes],
        Red.bold()
            .paint(&file_content[start.file_offset_bytes..start.file_offset_bytes + len]),
        &file_content[start.file_offset_bytes + len..line_end],
    )
}

#[cfg(test)]
mod tests {
    use super::super::grammar::parse_Programme;
    use super::super::lexer::{Location, Matcher};
    use super::*;
    use std::collections::HashMap;

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

    test_parse_error! {eof_in_string_literal, "\
function main() {
    let a = \"sdsds
}
", format!("\
{}: Found end of file whilst looking for end of string literal
At: test.sl:2
1 |function main() {{
2 |    let a = {}sdsds
3 |}}
", Red.paint("error"), Red.bold().paint("\""))}

    test_parse_error! {windows_line_endings, "\
function main() {\r
    let a = \"sdsds\r
}\r
", format!("\
{}: Found end of file whilst looking for end of string literal
At: test.sl:2
1 |function main() {{
2 |    let a = {}sdsds
3 |}}
", Red.paint("error"), Red.bold().paint("\""))}

    test_parse_error! {no_surrounding_lines, "\
function main() { let a = \"sdsds }
", format!("\
{}: Found end of file whilst looking for end of string literal
At: test.sl:1
1 |function main() {{ let a = {}sdsds }}
", Red.paint("error"), Red.bold().paint("\""))}

    test_parse_error! {missing_curly, "\
function main()
    return 1 + 3;
}
", format!("\
{}: Unrecognized token Return
At: test.sl:2
1 |function main()
2 |    {} 1 + 3;
3 |}}
Expected one of \"{{\"
", Red.paint("error"), Red.bold().paint("return"))}

    macro_rules! test_write_lines {
        ($test_name:ident, $content:expr,
            ($start_line:expr, $start_line_offset:expr, $start_offset:expr),
            ($end_line:expr, $end_line_offset:expr, $end_offset:expr),
            $expected:expr) => {
            #[test]
            fn $test_name() {
                let file_contents = $content;
                let mut contents = HashMap::new();
                contents.insert("test.sl", file_contents);
                let mut output = Vec::new();
                write_locations(
                    &mut output,
                    &Location::new("test.sl", $start_line, $start_line_offset, $start_offset),
                    &Location::new("test.sl", $end_line, $end_line_offset, $end_offset),
                    &contents,
                )
                .unwrap();
                assert_eq!(String::from_utf8(output).unwrap(), $expected);
            }
        };
    }

    test_write_lines! {error_across_two_lines, "\
if something() +
    somethingelse() {
    # code here
}
",
    (1, 3, 3),
    (2, 18, 36), format!("\
At: test.sl:1
1 |if {}
2 |{} {{
3 |    # code here
", Red.bold().paint("something() +"), Red.bold().paint("    somethingelse()"))
        }

    test_write_lines! {error_across_four_lines, "\
# X
if a and
    b and
    c and
    d {
    # code here
}
",
    (2, 3, 7),
    (5, 5, 38), format!("\
At: test.sl:2
1 |# X
2 |if {}
3 |{}
4 |{}
5 |{} {{
6 |    # code here
", Red.bold().paint("a and"), Red.bold().paint("    b and"),
        Red.bold().paint("    c and"), Red.bold().paint("    d"))
        }

    test_write_lines! {error_surrounded_by_blank_lines, "\
# a

if something() {

    # code here
}
",
    (3, 3, 8),
    (3, 14, 19), format!("\
At: test.sl:3
2 |
3 |if {} {{
4 |
", Red.bold().paint("something()"))
        }

}
