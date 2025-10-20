use std::env;
use std::io;
use std::process;

#[derive(Debug, Clone)]
enum Token {
    Literal(char),
    Digit,
    Word,
    Class(String),
    NegClass(String),
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

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let (anchor_start, anchor_end, tokens) = tokenize_pattern(pattern);
    let input_chars: Vec<char> = input_line.chars().collect();

    'input_loop: for i in 0..input_line.len() {
        if (anchor_start && i != 0) || (i + tokens.len() > input_chars.len()) {
            break;
        }
        for (j, token) in tokens.iter().enumerate() {
            if i + j >= input_chars.len() || !matchone(input_chars[i + j], token) {
                continue 'input_loop;
            }
        }
        if anchor_end && i + tokens.len() != input_line.len() {
            continue;
        }
        return true;
    }
    false
}

fn matchone(input_char: char, token: &Token) -> bool {
    match token {
        Token::Literal(c) => input_char == *c,
        Token::Digit => input_char.is_ascii_digit(),
        Token::Word => input_char.is_ascii_alphanumeric() || input_char == '_',
        Token::Class(s) => s.chars().any(|c| input_char == c),
        Token::NegClass(s) => s.chars().all(|c| input_char != c),
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
