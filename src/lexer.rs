
#[derive(PartialEq, Debug)]
pub enum Tok {
//    Identifier(String),
    // Literals
    Integer(i32),
//    Char(char),
//    String(String),
    // Key words
//    Function,
//    Return,
//    Let,
//    If,
//    While,

//    And,
//    Not,
//    Or,
    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    SemiColon,
    Comma,
    Equal,
    LessThan,
    MoreThan,
    LessThanOrEqual,
    MoreThanOrEqual,
    DoubleEqual,
    NotEqual,
    Ampersand,
    Pipe,
    Caret,
    Plus,
    Minus,
    Tilde,
    LeftShift,
    RightShift,
    Asterisk,
    Percent,
    ForwardSlash,
}

#[derive(PartialEq, Debug)]
pub enum Error {
//    IllegalChar(char, Location),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Location {
    line: usize,
    line_offset_chars: usize,
    file_offset_bytes: usize,
}

impl Location {
    fn new(line: usize, line_offset_chars: usize, file_offset_bytes: usize) -> Location {
        Location {
            line: line,
            line_offset_chars: line_offset_chars,
            file_offset_bytes: file_offset_bytes
        }
    }
    fn new_line(&mut self) {
        self.line += 1;
        self.line_offset_chars = 0;
    }
}

struct FindTokenStartResult {
    offset: usize,
    state: FindTokenStartState,
}

enum FindTokenStartState {
    WholeToken(Tok),
    PunctuationStart,
    NumberStart,
    EndOfFile,
}

pub struct Matcher<'input> {
    text: &'input str,
    location: Location,
}

impl<'input> Matcher<'input> {
    pub fn new(text: &'input str) -> Matcher {
        Matcher {
            text: text,
            location: Location::new(1, 0, 0),
        }
    }

    fn find_token_start(&mut self) -> FindTokenStartResult {
        use self::Tok::*;
        use self::FindTokenStartState::*;
//        let mut expect_line_feed = false;
        for (offset, c) in self.text.char_indices() {
            macro_rules! result {
                ( $state:expr ) => {
                    return FindTokenStartResult { offset: offset, state: $state }
                }
            }
            macro_rules! wt {
                ( $t:ident) => {
                    result!(WholeToken($t))
                }
            }
            match c {
                _ if c == ' ' || c == '\t' => self.location.line_offset_chars += 1,
                '\n' => self.location.new_line(),
                '(' => wt!(LeftParen),
                ')' => wt!(RightParen),
                '{' => wt!(LeftBrace),
                '}' => wt!(RightBrace),
                ']' => wt!(LeftBracket),
                '[' => wt!(RightBracket),
                ';' => wt!(SemiColon),
                ',' => wt!(Comma),
                '&' => wt!(Ampersand),
                '|' => wt!(Pipe),
                '^' => wt!(Caret),
                '+' => wt!(Plus),
                '-' => wt!(Minus),
                '~' => wt!(Tilde),
                '*' => wt!(Asterisk),
                '%' => wt!(Percent),
                '/' => wt!(ForwardSlash),
                '='|'<'|'>'|'!' => result!(PunctuationStart),
                '0'...'9' => result!(NumberStart),
                _ => {
                    panic!("IllegalChar");
                }
            }
        }
        FindTokenStartResult { offset: self.text.len(), state: EndOfFile }
    }
    fn consume(&mut self, bytes: usize) {
        self.location.file_offset_bytes += bytes;
        self.location.line_offset_chars += bytes;
        self.text = &self.text[bytes..];
    }
    fn token(&mut self, token: Tok, size: usize) -> MatcherItem {
        let start = self.location;
        self.consume(size);
        Ok((start, token, self.location))
    }
    fn extract_punctuation(&mut self) -> MatcherItem {
        use self::Tok::*;
        let mut chars = self.text.chars();
        macro_rules! second_char {
            ( $default:expr, $( $ch:tt => $tok:ident ),+ ) => {{
                match chars.next() {
                    $(
                        Some($ch) => self.token($tok, 2),
                    )*
                    _ => self.token($default, 1),
                }
            }};
        }
        match chars.next() {
            Some(char1) => {
                match char1 {
                    '=' => second_char!{Equal,
                        '=' => DoubleEqual
                    },
                    '<' => second_char!{LessThan,
                        '=' => LessThanOrEqual,
                        '<' => LeftShift
                    },
                    '>' => second_char!(MoreThan,
                        '=' => MoreThanOrEqual,
                        '>' => RightShift
                    ),
                    '!' => {
                        match chars.next() {
                            Some('=') => self.token(NotEqual, 2),
                            _ => panic!("IllegalChar ! without ="),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }
    fn extract_number(&mut self) -> MatcherItem {
        use std::str::FromStr;
        let number_length = self.text.find(|c| {c < '0' || c > '9'}).unwrap_or(self.text.len());
        let tok = Tok::Integer(i32::from_str(&self.text[..number_length]).unwrap());
        self.token(tok, number_length)
    }
}

type MatcherItem = Result<(Location, Tok, Location), Error>;

impl<'input> Iterator for Matcher<'input> {
    type Item = MatcherItem;

    fn next(&mut self) -> Option<Self::Item> {
        use self::FindTokenStartState::*;
        let token_start = self.find_token_start();
        self.location.file_offset_bytes += token_start.offset;
        self.text = &self.text[token_start.offset..];
        match token_start.state {
            WholeToken(token) => Some(self.token(token, 1)),
            PunctuationStart => Some(self.extract_punctuation()),
            NumberStart => Some(self.extract_number()),
            EndOfFile => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Tok::*;

    macro_rules! test_lex {
        ( $test_name:ident, $input:expr, $expected:expr ) => {
            #[test]
             fn $test_name() {
                let matcher = Matcher::new($input);
                let output = matcher.collect::<Vec<_>>();
                assert_eq!(output, $expected)
             }
        }
    }

    fn tok(t: Tok, line: usize, start_line_offset: usize, start_file_offst: usize, bytes: usize)
        -> MatcherItem {
        Ok(
            (Location::new(line, start_line_offset, start_file_offst),
             t,
             Location::new(line, start_line_offset + bytes, start_file_offst + bytes),
            )
        )
    }

    test_lex!{empty_string_ends_immediately, "", vec![]}
    test_lex!{extract_a_simple_plus, "+", vec![
        tok(Plus, 1, 0, 0, 1),
    ]}
    test_lex!{extract_two_pluses, "++", vec![
        tok(Plus, 1, 0, 0, 1),
        tok(Plus, 1, 1, 1, 1),
    ]}
    test_lex!{extract_two_pluses_on_different_lines, "+\n+\n", vec![
        tok(Plus, 1, 0, 0, 1),
        tok(Plus, 2, 0, 2, 1),
    ]}
    test_lex!{extract_equal, "=", vec![
        tok(Equal, 1, 0, 0, 1),
    ]}
    test_lex!{extract_double_equal, "==", vec![
        tok(DoubleEqual, 1, 0, 0, 2),
    ]}
    test_lex!{extract_equal_when_followed_by_space, "= ", vec![
        tok(Equal, 1, 0, 0, 1),
    ]}
    test_lex!{extract_less_than, "<", vec![
        tok(LessThan, 1, 0, 0, 1),
    ]}
    test_lex!{extract_less_than_or_equal, "<=", vec![
        tok(LessThanOrEqual, 1, 0, 0, 2),
    ]}
    test_lex!{extract_left_shift, "<<", vec![
        tok(LeftShift, 1, 0, 0, 2),
    ]}
    test_lex!{extract_more_than, ">", vec![
        tok(MoreThan, 1, 0, 0, 1),
    ]}
    test_lex!{extract_more_than_or_equal, ">=", vec![
        tok(MoreThanOrEqual, 1, 0, 0, 2),
    ]}
    test_lex!{extract_right_shift, ">>", vec![
        tok(RightShift, 1, 0, 0, 2),
    ]}
    test_lex!{extract_not_equal, "!=", vec![
        tok(NotEqual, 1, 0, 0, 2),
    ]}
    test_lex!{extract_integer, "123", vec![
        tok(Integer(123), 1, 0, 0, 3),
    ]}
}