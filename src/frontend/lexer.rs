use crate::frontend::tokens::{Token, TokenType, KEYWORDS};
use crate::frontend::error;

pub struct Lexer {
    src:String,
    token:Vec<Token>,
    start:usize,
    current:usize,
    line:i32
}

impl Lexer {
    pub fn new(src:String) -> Lexer{
        Lexer{
            src,
            token:Vec::new(),
            start:0,
            current:0,
            line:1,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.token.push(Token::new(TokenType::Eof, "", self.line));
        &self.token
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => {
                if self.n_match('>') {
                    self.add_token(TokenType::Gives)
                } else {
                    self.add_token(TokenType::Minus)
                }
            },
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.n_match('='){
                    self.add_token(TokenType::BangEqual)
                }else{
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.n_match('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.n_match('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.n_match('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.n_match('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => (), // Ignore whitespace
            '\n' => self.line += 1,
            '"' => self.string(),
            c => {
                if c.is_digit(10) {
                    self.number()
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier()
                } else {
                    error::error(self.line, "Unexpected character.")
                }
            }
        };
    }

    fn peek(&self) -> char {
        self.src.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.src.chars().nth(self.current+1).unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        let char_vec: Vec<char> = self.src.chars().collect();
        char_vec[self.current-1]
    }

    fn add_token(&mut self, token_type:TokenType) {
        let txt = self.src.get(self.start..self.current).expect("Source Token Empty");
        self.token.push(Token::new(token_type, txt, self.line))
    }

    fn identifier(&mut self){
        while self.peek().is_alphanumeric() || self.peek()=='_'{
            self.advance();
        }

        let txt = self.src.get(self.start..self.current).expect("Unexpected Termination");
        let t_type: TokenType = KEYWORDS.get(txt).cloned().unwrap_or(TokenType::Identifier);
        self.add_token(t_type);
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let n: f64 = self
            .src
            .get(self.start..self.current)
            .expect("Unexpected end.")
            .parse()
            .expect("Scanned number could not be parsed.");
        self.add_token(TokenType::Number { literal: n })
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error::error(self.line, "Unterminated string.");
            return;
        }

        self.advance();
        let literal = self
            .src
            .get((self.start + 1)..(self.current - 1))
            .expect("Unexpected end.")
            .to_string();

        self.add_token(TokenType::String { literal });
    }

    fn n_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self
            .src
            .chars()
            .nth(self.current)
            .expect("Unexpected end of source.")
            != expected
        {
            return false;
        }

        self.current += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //FIXME
    #[test]
    fn default_test() {
        let input = "var a = 5".to_string();
        let mut lexer = Lexer::new(input);
        let expected = vec![
            Token{ token_type:TokenType::Var, lexeme:"var".to_string(),line:1},
            // Token{ token_type:TokenType::Identifier{literal:"a".to_string()}, lexeme:"a".to_string(),line:1},
            Token{ token_type:TokenType::Equal, lexeme:"=".to_string(),line:1},
            Token{ token_type:TokenType::Number {literal:5.0}, lexeme:"5".to_string(),line:1},
            Token{ token_type:TokenType::Eof, lexeme:"".to_string(),line:1},
        ];
        let actual = lexer.scan_tokens();
        assert_eq!(expected.len(), actual.len());
        for i in 0..expected.len(){
            assert_eq!(expected[i].token_type, actual[i].token_type);
            assert_eq!(expected[i].lexeme, actual[i].lexeme);
            assert_eq!(expected[i].line, actual[i].line);
        }
    }
}