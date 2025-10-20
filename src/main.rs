use std::env;
use std::io;
use std::process;

#[derive(Debug, Clone)]
enum Token {
    Literal(char),
    Digit,
    Word,
    Wildcard,
    Class(String),
    NegClass(String),
    OneOrMore(Box<Token>),
    ZeroOrOne(Box<Token>),
    Alternation(Vec<String>),
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
            '(' => {
                let mut alternation_content = String::new();
                while let Some(ch) = chars.next() {
                    if ch == ')' {
                        break;
                    }
                    alternation_content.push(ch);
                }
                let options: Vec<String> = alternation_content
                    .split('|')
                    .map(|s| s.to_string())
                    .collect();
                tokens.push(Token::Alternation(options));
            }
            _ => tokens.push(Token::Literal(c)),
        }
    }

    (anchor_start, anchor_end, tokens)
}

// see https://www.cs.princeton.edu/courses/archive/spr09/cos333/beautiful.html
fn match_pattern(text: &str, pattern: &str) -> bool {
    let (anchor_start, anchor_end, tokens) = tokenize_pattern(pattern);

    if anchor_start {
        return matchhere(text, &tokens, anchor_end);
    }

    for i in 0..text.len() {
        if matchhere(&text[i..], &tokens, anchor_end) {
            return true;
        }
    }

    false
}

fn matchhere(text: &str, tokens: &[Token], anchor_end: bool) -> bool {
    if tokens.is_empty() {
        return !anchor_end || text.is_empty();
    }

    match &tokens[0] {
        Token::OneOrMore(inner_token) => {
            matchoneormore(text, inner_token, &tokens[1..], anchor_end)
        }
        Token::ZeroOrOne(inner_token) => {
            if !text.is_empty() && matchone(text.chars().next().unwrap(), inner_token) {
                if matchhere(&text[1..], &tokens[1..], anchor_end) {
                    return true;
                }
            }
            matchhere(text, &tokens[1..], anchor_end)
        }
        Token::Alternation(options) => {
            for option in options {
                if text.starts_with(option)
                    && matchhere(&text[option.len()..], &tokens[1..], anchor_end)
                {
                    return true;
                }
            }
            false
        }
        _ => {
            if !text.is_empty() && matchone(text.chars().next().unwrap(), &tokens[0]) {
                matchhere(&text[1..], &tokens[1..], anchor_end)
            } else {
                false
            }
        }
    }
}

fn matchoneormore(text: &str, inner_token: &Token, tokens: &[Token], anchor_end: bool) -> bool {
    if text.is_empty() || !matchone(text.chars().next().unwrap(), inner_token) {
        return false;
    }
    for i in 1..text.len() {
        if matchhere(&text[i..], &tokens[1..], anchor_end) {
            return true;
        }
    }
    false
}

fn matchone(next_char: char, token: &Token) -> bool {
    match token {
        Token::Literal(c) => next_char == *c,
        Token::Digit => next_char.is_ascii_digit(),
        Token::Word => next_char.is_ascii_alphanumeric() || next_char == '_',
        Token::Wildcard => next_char != '\n',
        Token::Class(s) => s.chars().any(|c| next_char == c),
        Token::NegClass(s) => s.chars().all(|c| next_char != c),
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
