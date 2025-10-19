use std::env;
use std::io;
use std::process;

enum Token {
    Literal(char),
    Digit,
    Word,
    Class(String),
    NegClass(String),
}

fn tokenize_pattern(pattern: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
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

    tokens
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let tokens = tokenize_pattern(pattern);
    let input_chars: Vec<char> = input_line.chars().collect();

    'input_loop: for i in 0..input_line.len() {
        for (j, token) in tokens.iter().enumerate() {
            if i + j >= input_chars.len() || !matchone(input_chars[i + j], token) {
                continue 'input_loop;
            }
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

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
