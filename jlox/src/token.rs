use std::{fmt::Display, hash::Hash};

use crate::ast::Literal;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum TokenType {
    //Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    line: usize,
}

impl Eq for Token {}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: Option<Literal>, line: usize) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
            literal,
            line,
        }
    }

    fn to_string(&self) -> String {
        format!("{:?} {}", self.token_type, self.lexeme)
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::LeftParen => f.write_str("("),
            Self::RightParen => f.write_str(")"),
            Self::LeftBrace => f.write_str("["),
            Self::RightBrace => f.write_str("]"),
            Self::Comma => f.write_str(","),
            Self::Dot => f.write_str("."),
            Self::Minus => f.write_str("-"),
            Self::Plus => f.write_str("+"),
            Self::Semicolon => f.write_str(";"),
            Self::Slash => f.write_str("/"),
            Self::Star => f.write_str("*"),
            Self::Bang => f.write_str("!"),
            Self::BangEqual => f.write_str("!="),
            Self::Equal => f.write_str("="),
            Self::EqualEqual => f.write_str("=="),
            Self::Greater => f.write_str(">"),
            Self::GreaterEqual => f.write_str(">="),
            Self::Less => f.write_str("<"),
            Self::LessEqual => f.write_str("<="),
            Self::Identifier => f.write_str("IDENT"),
            Self::String => f.write_str("STR"),
            Self::Number => f.write_str("NUM"),
            Self::And => f.write_str("and"),
            Self::Class => f.write_str("class"),
            Self::Else => f.write_str("else"),
            Self::False => f.write_str("false"),
            Self::Fun => f.write_str("fun"),
            Self::For => f.write_str("for"),
            Self::If => f.write_str("if"),
            Self::Nil => f.write_str("nil"),
            Self::Or => f.write_str("or"),
            Self::Print => f.write_str("print"),
            Self::Return => f.write_str("return"),
            Self::Super => f.write_str("super"),
            Self::This => f.write_str("this"),
            Self::True => f.write_str("true"),
            Self::Var => f.write_str("var"),
            Self::While => f.write_str("while"),
            Self::EOF => f.write_str("\\d"),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:#?} {} {:#?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.token_type.hash(state);
        self.lexeme.hash(state);
    }
}
