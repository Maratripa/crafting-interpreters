use phf::phf_map;
use thiserror::Error;

use crate::{
    ast::Literal,
    token::{Token, TokenType},
};

#[derive(Error, Debug)]
enum Error {
    #[error("Unexpected character.")]
    UnexpectedChar,

    #[error("Undetermined String")]
    UndeterminedString,
}

type Result<T, E = Error> = std::result::Result<T, E>;

type TT = TokenType;

static KEYWORDS: phf::Map<&'static str, TT> = phf_map! {
    "and" => TT::And,
    "class" => TT::Class,
    "else" => TT::Else,
    "false" => TT::False,
    "for" => TT::For,
    "fun" => TT::Fun,
    "if" => TT::If,
    "nil" => TT::Nil,
    "or" => TT::Or,
    "print" => TT::Print,
    "return" => TT::Return,
    "super" => TT::Super,
    "this" => TT::This,
    "true" => TT::True,
    "var" => TT::Var,
    "while" => TT::While,
};

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub struct Scanner<'a> {
    source: &'a [u8],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            let _ = self.scan_token();
        }

        self.tokens.push(Token::new(TT::EOF, "", None, self.line));

        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<()> {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(TT::LeftParen, None),
            ')' => self.add_token(TT::RightParen, None),
            '{' => self.add_token(TT::LeftBrace, None),
            '}' => self.add_token(TT::RightBrace, None),
            ',' => self.add_token(TT::Comma, None),
            '.' => self.add_token(TT::Dot, None),
            '-' => self.add_token(TT::Minus, None),
            '+' => self.add_token(TT::Plus, None),
            ';' => self.add_token(TT::Semicolon, None),
            '*' => self.add_token(TT::Star, None),
            '!' => self.check_next('=', TT::BangEqual, TT::Bang),
            '=' => self.check_next('=', TT::EqualEqual, TT::Equal),
            '<' => self.check_next('=', TT::LessEqual, TT::Less),
            '>' => self.check_next('=', TT::GreaterEqual, TT::Greater),
            '/' => {
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TT::Slash, None);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string()?,
            c => {
                if is_digit(c) {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    return Err(Error::UnexpectedChar);
                }
            }
        }

        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = std::str::from_utf8(&self.source[self.start..self.current]).unwrap();

        if let Some(ttype) = KEYWORDS.get(text).cloned() {
            self.add_token(ttype, None);
        } else {
            self.add_token(TT::Identifier, Some(Literal::String(text.to_owned())));
        };
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.add_token(
            TokenType::Number,
            Some(Literal::Number(
                std::str::from_utf8(&self.source[self.start..self.current])
                    .unwrap()
                    .parse()
                    .unwrap(),
            )),
        )
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Error::UndeterminedString);
        }

        // The closing "
        self.advance();

        // Trim the surrounding quotes
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(
            TT::String,
            Some(Literal::String(
                String::from_utf8(value.to_owned()).unwrap(),
            )),
        );

        Ok(())
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1] as char
    }

    fn add_token(&mut self, token_type: TT, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token_type,
            std::str::from_utf8(text).unwrap(),
            literal,
            self.line,
        ));
    }

    fn check_next(&mut self, c: char, left: TT, right: TT) {
        if self.is_at_end() {
            self.add_token(right, None);
        } else if self.source[self.current] as char != c {
            self.add_token(right, None);
        } else {
            self.current += 1;
            self.add_token(left, None);
        }
    }

    fn match_next(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] as char != c {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.current] as char
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source[self.current + 1] as char
    }
}
