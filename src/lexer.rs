
#[derive(PartialEq, Debug)]
pub enum Tok<'input> {
    Identifier(&'input str),
    // Literals
    Integer(i32),
    Char(char),
    String(&'input str),
    // Key words
    Function,
    Return,
    Let,
    If,
    While,

    And,
    Not,
    Or,
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

impl Default for Location {
    fn default() -> Location {
        Location::new(0, 0, 0)
    }
}

fn is_identifier_char(c: char) -> bool{
    match c {
        'a'...'z'|'A'...'Z'|'0'...'9'|'_' => true,
        _ => false,
    }
}

struct FindTokenStartResult<'input> {
    offset: usize,
    state: FindTokenStartState<'input>,
}

enum FindTokenStartState<'input> {
    WholeToken(Tok<'input>),
    PunctuationStart,
    NumberStart,
    CharStart,
    StringStart,
    IdentifierOrKeyWordStart,
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

    fn find_token_start(&mut self) -> FindTokenStartResult<'input> {
        use self::Tok::*;
        use self::FindTokenStartState::*;
//        let mut expect_line_feed = false;
        let mut in_comment = false;
        for (offset, c) in self.text.char_indices() {
            macro_rules! result {
                ( $state:expr ) => {
                    if in_comment {
                        self.location.line_offset_chars += 1;
                    } else {
                        return FindTokenStartResult { offset: offset, state: $state }
                    }
                }
            }
            macro_rules! wt {
                ( $t:ident) => {
                    result!(WholeToken($t))
                }
            }
            match c {
                _ if c == ' ' || c == '\t' => self.location.line_offset_chars += 1,
                '\n' => {
                    self.location.new_line();
                    in_comment = false;
                },
                '#' => {
                    self.location.line_offset_chars += 1;
                    in_comment = true;
                },
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
                '\'' => result!(CharStart),
                '"' => result!(StringStart),
                'a'...'z'|'A'...'Z'|'_' => result!(IdentifierOrKeyWordStart),
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
    fn token(&mut self, token: Tok<'input>, size: usize) -> <Self as Iterator>::Item {
        let start = self.location;
        self.consume(size);
        Ok((start, token, self.location))
    }
    fn extract_punctuation(&mut self) -> <Self as Iterator>::Item {
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
    fn extract_number(&mut self) -> <Self as Iterator>::Item {
        use std::str::FromStr;
        let number_length = self.text.find(|c| {c < '0' || c > '9'}).unwrap_or(self.text.len());
        let tok = Tok::Integer(i32::from_str(&self.text[..number_length]).unwrap());
        self.token(tok, number_length)
    }
    fn extract_char(&mut self) -> <Self as Iterator>::Item {
        let mut chars = self.text.chars();
        if chars.next() != Some('\'') {
            unreachable!();
        }
        let c = match chars.next() {
            Some(c) => c,
            None => panic!("EOF in char literal"),
        };
        match chars.next() {
            Some('\'') => {},
            Some(_) => panic!("Bad char literal"),
            None => panic!("EOF in char literal"),
        }
        let tok = Tok::Char(c);
        self.token(tok, 3)
    }
    fn extract_string(&mut self) -> <Self as Iterator>::Item {
        // TODO will break on unicode
        if let Some(string_length) = self.text[1..].find('"') {
            let s = &self.text[1..string_length + 1];
            self.token(Tok::String(s), string_length + 2)
        } else {
            panic!("EOF in string");
        }
    }
    fn extract_identifier_or_keyword(&mut self) -> <Self as Iterator>::Item {
        let len = self.text.find(|c| { !is_identifier_char(c) }).unwrap_or(self.text.len());
        let tok = match &self.text[..len] {
            "function" => Tok::Function,
            "return" => Tok::Return,
            "let" => Tok::Let,
            "if" => Tok::If,
            "while" => Tok::While,
            "and" => Tok::And,
            "not" => Tok::Not,
            "or" => Tok::Or,
            s => Tok::Identifier(s),
        };
        self.token(tok, len)
    }
}

impl<'input> Iterator for Matcher<'input> {
    type Item = Result<(Location, Tok<'input>, Location), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::FindTokenStartState::*;
        let token_start = self.find_token_start();
        self.location.file_offset_bytes += token_start.offset;
        self.text = &self.text[token_start.offset..];
        match token_start.state {
            WholeToken(token) => Some(self.token(token, 1)),
            PunctuationStart => Some(self.extract_punctuation()),
            NumberStart => Some(self.extract_number()),
            CharStart => Some(self.extract_char()),
            StringStart => Some(self.extract_string()),
            IdentifierOrKeyWordStart => Some(self.extract_identifier_or_keyword()),
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

    macro_rules! test_keywords {
        ( $( $test_name:ident: $keyword:expr => $tok:ident ),* ) => {
            $(
                test_lex!{$test_name, $keyword, vec![
                    tok($tok, 1, 0, 0, $keyword.len())
                ]}
            )*
        }
    }

    fn tok(t: Tok, line: usize, start_line_offset: usize, start_file_offst: usize, bytes: usize)
        -> <Matcher as Iterator>::Item {
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
    test_lex!{extract_integer_2, "923 - 03", vec![
        tok(Integer(923), 1, 0, 0, 3),
        tok(Minus, 1, 4, 4, 1),
        tok(Integer(3), 1, 6, 6, 2),
    ]}
    test_lex!{extract_char, "'s'", vec![
        tok(Char('s'), 1, 0, 0, 3),
    ]}
    test_lex!{extract_str, r#""hello""#, vec![
        tok(String("hello"), 1, 0, 0, 7),
    ]}
    test_lex!{extract_identifier, "bob", vec![
        tok(Identifier("bob"), 1, 0, 0, 3),
    ]}
    test_keywords!{
        extract_function: "function" => Function,
        extract_return: "return" => Return,
        extract_let: "let" => Let,
        extract_if: "if" => If,
        extract_while: "while" => While,
        extract_and: "and" => And,
        extract_not: "not" => Not,
        extract_or: "or" => Or
    }
    test_lex!{ignore_comments, indoc!("
            ident # A comment
            and
        "),
        vec![
            tok(Identifier("ident"), 1, 0, 0, 5),
            tok(And, 2, 0, 18, 3),
        ]
    }

}