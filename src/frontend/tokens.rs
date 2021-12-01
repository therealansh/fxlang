use std::collections::HashMap;
use lazy_static::lazy_static;
use std::fmt;
use std::hash::{Hasher, Hash};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens
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

    // Literals - Encoded in the enum
    Identifier,
    String { literal: String },
    Number { literal: f64 },

    // Keywords
    And,
    Class,
    Else,
    False,
    Fn,
    Gives,
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

    Eof,
}

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str,TokenType> = {
        let mut map = HashMap::new();
        map.insert("and", TokenType::And);
        map.insert("class", TokenType::Class);
        map.insert("else", TokenType::Else);
        map.insert("false", TokenType::False);
        map.insert("for", TokenType::For);
        map.insert("fn", TokenType::Fn);
        map.insert("if", TokenType::If);
        map.insert("nil", TokenType::Nil);
        map.insert("or", TokenType::Or);
        map.insert("print", TokenType::Print);
        map.insert("return", TokenType::Return);
        map.insert("super", TokenType::Super);
        map.insert("this", TokenType::This);
        map.insert("true", TokenType::True);
        map.insert("var", TokenType::Var);
        map.insert("while", TokenType::While);
        map.insert("->", TokenType::Gives);
        map
    };
}

#[derive(Debug, Clone,PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, line: i32) -> Token {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token_type {
            TokenType::String { literal } => write!(f, "String {:?} {:?}",self.lexeme,literal),
            TokenType::Number {literal} => write!(f, "Number {:?} {:?}", self.lexeme, literal),
            _ => write!(f, "{:?} {:?}", self.token_type, self.lexeme)
        }
    }
}
//TODO add docs for hash, hasher and Eq
impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lexeme.hash(state);
        self.line.hash(state);
    }
}

impl Eq for Token {}