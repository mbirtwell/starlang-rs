use std;
use std::io::{self, Write};
use lalrpop_util;
use lexer::{self, Location, Tok};

pub enum OuterError {
    ReadInput,
    ParseError,
    OutputError,
}
pub type OuterResult<T> = std::result::Result<T, OuterError>;
pub type ParseError<'input> = lalrpop_util::ParseError<Location, Tok<'input>, lexer::Error>;

pub fn write_parse_error(f: &mut Write, err: ParseError) -> OuterResult<()> {
    write_parse_error_inner(f, err).map_err(|_| OuterError::OutputError )
}

fn write_parse_error_inner(f: &mut Write, err: ParseError) -> io::Result<()> {
    use lalrpop_util::ParseError::*;
    match err {
        InvalidToken{ ref location } => {
            writeln!(f, "error: Invalid token").unwrap();
            write_location(f, location)?;
        },
        UnrecognizedToken{ ref token, ref expected } => {
            match *token {
                Some((ref start, ref token, ref end)) => {
                    write!(f, "error: Unrecognized token {:?}", token).unwrap();
                    write_locations(f, start, end)?;
                }
                None => writeln!(f, "error: Unrecognized EOF").unwrap(),
            }
            if !expected.is_empty() {
                for (i, e) in expected.iter().enumerate() {
                    let sep = match i {
                        0 => "Expected one of",
                        _ if i < expected.len() - 1 => ",",
                        // Last expected message to be written
                        _ => " or",
                    };
                    writeln!(f, "{} {}", sep, e).unwrap();
                }
            }
        }
        ExtraToken { token: (ref start, ref token, ref end) } => {
            writeln!(f, "error: Extra token {:?}", token).unwrap();
            write_locations(f, start, end)?;

        }
        User { ref error } => {
            writeln!(f, "error: {}", error.kind)?;
            write_location(f, &error.location)?;
        }
    }
    Ok(())
}

fn write_location(f: &mut Write, location: &lexer::Location) -> io::Result<()> {
    Ok(())
}

fn write_locations(f: &mut Write, start: &lexer::Location, end: &lexer::Location) -> io::Result<()> {
    Ok(())
}
