
pub fn strip_comments(mut input: &str) -> String {
    let mut out = String::new();
    loop {
        let mut in_string = false;
        let mut in_char = false;
        match input.find(|c| {
            if c == '"' {
                in_string = !in_string;
                false
            } else if c == '\'' {
                in_char = !in_char;
                false
            } else {
                !in_string && !in_char && c == '#'
            }
        }) {
            Some(comment_start) => {
                out.push_str(&input[..comment_start]);
                input = &input[comment_start..];
                match input.find('\n') {
                    Some(new_line) => input = &input[new_line..],
                    None => break,
                }
            },
            None => {
                out.push_str(input);
                break;
            },
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_comment_less_code() {
        assert_eq!(
            strip_comments(indoc!("\
                function main(args) {
                    return 1;
                }
            ")), indoc!("\
                function main(args) {
                    return 1;
                }
            ")
        )
    }

    #[test]
    fn strips_line_comment() {
        assert_eq!(
            strip_comments(indoc!("\
                function a() {}
                # A comment
                function main(args) {
                    return 1;
                }
            ")), indoc!("\
                function a() {}

                function main(args) {
                    return 1;
                }
            ")
        )
    }

    #[test]
    fn strips_trailing_comment() {
        assert_eq!(
            strip_comments(indoc!("\
                function main(args) {
                    return 1;# A comment
                }
            ")), indoc!("\
                function main(args) {
                    return 1;
                }
            ")
        )
    }

    #[test]
    fn ignores_hash_character_in_string() {
        assert_eq!(
            strip_comments(indoc!(r#"
                function main(args) {
                    let a = "a # char";
                }
            "#)), indoc!(r#"
                function main(args) {
                    let a = "a # char";
                }
            "#)
        )
    }

    #[test]
    fn stips_comment_after_string() {
        assert_eq!(
            strip_comments(indoc!(r#"
                function main(args) {
                    let a = "a # char";# A comment
                }
            "#)), indoc!(r#"
                function main(args) {
                    let a = "a # char";
                }
            "#)
        )
    }

    #[test]
    fn skips_hash_in_char() {
        assert_eq!(
            strip_comments(indoc!(r#"
                function main(args) {
                    let a = '#';# A comment
                }
            "#)), indoc!(r#"
                function main(args) {
                    let a = '#';
                }
            "#)
        )
    }
}