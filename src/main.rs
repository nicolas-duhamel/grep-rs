use std::env;
use std::io;
use std::process;

enum Token {
    Literal(char),
    Digit,
    Word,
    Wildcard,
    Class(String),
    NegClass(String),
    OneOrMore(Box<Token>),
    ZeroOrOne(Box<Token>),
}

fn tokenize_pattern(mut pattern: &str) -> (bool, bool, Vec<Token>) {
    let mut tokens = Vec::new();
    let mut anchor_start = false;
    let mut anchor_end = false;

    if pattern.starts_with('^') {
        anchor_start = true;
        pattern = &pattern[1..];
    }

    if pattern.ends_with('$') {
        anchor_end = true;
        pattern = &pattern[..pattern.len() - 1];
    }

    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '+' => {
                if let Some(last_token) = tokens.pop() {
                    tokens.push(Token::OneOrMore(Box::new(last_token)));
                } else {
                    panic!("'+' cannot be the first token");
                }
            }
            '?' => {
                if let Some(last_token) = tokens.pop() {
                    tokens.push(Token::ZeroOrOne(Box::new(last_token)));
                } else {
                    panic!("'?' cannot be the first token");
                }
            }
            '.' => {
                tokens.push(Token::Wildcard);
            }
            '\\' => {
                if let Some(next) = chars.next() {
                    match next {
                        'd' => tokens.push(Token::Digit),
                        'w' => tokens.push(Token::Word),
                        '\\' => tokens.push(Token::Literal('\\')),
                        _ => panic!("Unhandled escape: \\{}", next),
                    }
                } else {
                    panic!("Escape character at end of pattern");
                }
            }
            '[' => {
                let mut class_content = String::new();
                let mut negated = false;

                if let Some(&'^') = chars.peek() {
                    negated = true;
                    chars.next();
                }

                while let Some(ch) = chars.next() {
                    if ch == ']' {
                        break;
                    }
                    class_content.push(ch);
                }

                if negated {
                    tokens.push(Token::NegClass(class_content));
                } else {
                    tokens.push(Token::Class(class_content));
                }
            }
            _ => tokens.push(Token::Literal(c)),
        }
    }

    (anchor_start, anchor_end, tokens)
}

// see https://www.cs.princeton.edu/courses/archive/spr09/cos333/beautiful.html
fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let (anchor_start, anchor_end, tokens) = tokenize_pattern(pattern);
    let input_chars: Vec<char> = input_line.chars().collect();

    if anchor_start {
        return matchhere(&input_chars, &tokens, anchor_end);
    }

    for i in 0..input_chars.len() {
        if matchhere(&input_chars[i..], &tokens, anchor_end) {
            return true;
        }
    }

    false
}

fn matchhere(input_chars: &[char], tokens: &[Token], anchor_end: bool) -> bool {
    if tokens.is_empty() {
        return !anchor_end || input_chars.is_empty();
    }

    match &tokens[0] {
        Token::OneOrMore(inner_token) => {
            matchoneormore(input_chars, inner_token, &tokens[1..], anchor_end)
        }
        Token::ZeroOrOne(inner_token) => {
            if !input_chars.is_empty() && matchone(input_chars[0], inner_token) {
                if matchhere(&input_chars[1..], &tokens[1..], anchor_end) {
                    return true;
                }
            }
            matchhere(input_chars, &tokens[1..], anchor_end)
        }
        _ => {
            if !input_chars.is_empty() && matchone(input_chars[0], &tokens[0]) {
                matchhere(&input_chars[1..], &tokens[1..], anchor_end)
            } else {
                false
            }
        }
    }
}

fn matchoneormore(
    input_chars: &[char],
    inner_token: &Token,
    tokens: &[Token],
    anchor_end: bool,
) -> bool {
    if input_chars.is_empty() || !matchone(input_chars[0], inner_token) {
        return false;
    }
    for i in 1..input_chars.len() {
        if matchhere(&input_chars[i..], &tokens[1..], anchor_end) {
            return true;
        }
    }
    false
}

fn matchone(input_char: char, token: &Token) -> bool {
    match token {
        Token::Literal(c) => input_char == *c,
        Token::Digit => input_char.is_ascii_digit(),
        Token::Word => input_char.is_ascii_alphanumeric() || input_char == '_',
        Token::Wildcard => input_char != '\n',
        Token::Class(s) => s.chars().any(|c| input_char == c),
        Token::NegClass(s) => s.chars().all(|c| input_char != c),
        _ => panic!("Quantifier token should be handled in matchhere"),
    }
}

fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    input_line = input_line.trim_end().to_string();
    if match_pattern(&input_line, &pattern) {
        println!("This is a match");
        process::exit(0);
    } else {
        println!("This is not a match");
        process::exit(1);
    }
}
