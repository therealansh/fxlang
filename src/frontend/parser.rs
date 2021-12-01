use crate::frontend::tokens::{Token, TokenType};
use crate::frontend::expr::{Expr, LiteralValue};
use crate::frontend::error::{Error, parser_error};
use crate::frontend::expr::Expr::Literal;
use crate::frontend::stmt::Stmt;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error>{
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, Error>{
        if self.t_match(&[TokenType::Var]){
            self.var_decl()
        }
        else if self.t_match(&[TokenType::Fn]){
            self.function("function")
        }
        else if self.t_match(&[TokenType::Class]) {
            self.class_decl()
        }
        else{
            self.statement()
        }
    }

    fn var_decl(&mut self) -> Result<Stmt, Error>{
        let name = self.consume(TokenType::Identifier, "Expect a variable name.")?;
        let init = if self.t_match(&[TokenType::Equal]){
            Some(self.expression()?)
        }else{
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration");
        Ok(Stmt::Var{name:name, initializer:init})
    }

    fn statement(&mut self) -> Result<Stmt, Error>{
        if self.t_match(&[TokenType::Print]){
            self.print_statement()
        }
        else if self.t_match((&[TokenType::Return])) {
            self.return_stmt()
        }
        else if self.t_match(&[TokenType::If]) {
            self.if_stmt()
        }
        else if self.t_match(&[TokenType::While]) {
            self.while_stmt()
        }
        else if self.t_match(&[TokenType::For]) {
            self.for_stmt()
        }
        else if self.t_match(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block {
                statements:self.block()?
            })
        }
        else{
            self.expr_statement()
        }
    }

    fn block(&mut self)->Result<Vec<Stmt>,Error>{
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end(){
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace,"Expect '}' after block")?;
        Ok(statements)
    }

    fn if_stmt(&mut self)-> Result<Stmt, Error>{
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        let condition = self.expression()?;
        self.consume(TokenType::RightParen,"Expect ')' after if condition");
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.t_match(&[TokenType::Else]){
            Box::new(Some(self.statement()?))
        }else{
            Box::new(None)
        };

        Ok(Stmt::If {
            condition, else_branch, then_branch
        })
    }

    fn return_stmt(&mut self)->Result<Stmt,Error>{
        let keyword:Token = self.previous().clone();
        let val= if !self.check(TokenType::Semicolon){
            Some(self.expression()?)
        }else{
            None
        };
        self.consume(TokenType::Semicolon,"Expect ';' after return.")?;
        Ok(Stmt::Return {keyword,value:val})
    }

    fn function(&mut self, kind:&str)->Result<Stmt, Error>{
        let name = self.consume(TokenType::Identifier, format!("Expect {} name.", kind).as_str())?;
        self.consume(TokenType::LeftParen, format!("Expect '(' after {} name.", kind).as_str())?;
        let mut params:Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen){
            loop {
                if params.len()>=255 {
                    self.error(self.peek(), "Cannot have more than 255 params");
                }
                params.push(self.consume(TokenType::Identifier, "Expect param name,")?);
                if !self.t_match(&[TokenType::Comma]){
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen,"Expect ')' after params.")?;
        self.consume(TokenType::Gives, "Expected -> after fn declaration")?;
        self.consume(TokenType::LeftBrace, format!("Expect '{{' before {} body.", kind).as_str())?;
        let body = self.block()?;
        Ok(Stmt::FxFx {name,params,body})
    }

    fn class_decl(&mut self)->Result<Stmt,Error>{
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        let superclass = if self.t_match(&[TokenType::Less]){
            self.consume(TokenType::Identifier, "Expect superclass name.")?;
            Some(self.previous().clone())
        }else{
            None
        };
        self.consume(TokenType::Gives, "Expected -> after class declaration.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }
        self.consume(TokenType::RightBrace,"Expect '}' after class body")?;
        Ok(Stmt::Class {name, superclass: superclass.map(|name| Expr::Variable {name}),methods})
    }

    fn while_stmt(&mut self)->Result<Stmt,Error>{
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        let condition = self.expression()?;
        self.consume(TokenType::RightParen,"Expect ')' after condition.");
        let body = Box::new(self.statement()?);
        Ok(Stmt::While {condition,statement:body})
    }

    fn for_stmt(&mut self) -> Result<Stmt,Error>{
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'");
        let init = if self.t_match(&[TokenType::Semicolon]){
            None
        }else if self.t_match(&[TokenType::Var]){
            Some(self.var_decl()?)
        }else{
            Some(self.expr_statement()?)
        };

        let condition = if !self.t_match(&[TokenType::Semicolon]){
            Some(self.expression()?)
        }else{
            None
        };
        self.consume(TokenType::Semicolon,"Expeect ';' after loop condition");

        //TODO sure to name it increment??
        let increment = if !self.check(TokenType::RightParen){
            Some(self.expression()?)
        }else{
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if let Some(inc) = increment {
            let inc_stmt = Stmt::Expression {expr:inc};
            body = Stmt::Block {statements:vec![body, inc_stmt]}
        }

        body = Stmt::While {
            condition:condition.unwrap_or(Expr::Literal {val:LiteralValue::Boolean(true)}),
            statement: Box::new(body)
        };

        if let Some(init_stmt) = init{
            body = Stmt::Block {
                statements:vec![init_stmt, body]
            }
        }

        Ok(body)

    }

    fn print_statement(&mut self) -> Result<Stmt, Error>{
        let value = self.expression()?;
        self.consume(TokenType::Semicolon,"Expect ';' after value")?;
        Ok(Stmt::Print { expr: value })
    }

    fn expr_statement(&mut self) -> Result<Stmt, Error>{
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expr: expr })
    }

    fn t_match(&mut self, token_type: &[TokenType]) -> bool {
        for tt in token_type {
            if self.check(tt.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, t_type: TokenType) -> bool {
        if(self.is_at_end()) {
            return false;
        }
        t_type == self.peek().token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token{
        if(!self.is_at_end()){
            self.current+=1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current-1]
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error>{
        let mut expr = self.or_()?;
        if self.t_match(&[TokenType::Equal]){
            let val = Box::new(self.assignment()?);
            if let Expr::Variable {name} = expr {
                return Ok(Expr::Assign { name, val });
            }else if let Expr::Get {object, name} = expr{
                return Ok(Expr::Set {object,name, value: val});
            }
            let equals = self.previous();
            self.error(equals, "Invalid assignment target.");
        }
        Ok(expr)
    }

    fn or_(&mut self)-> Result<Expr,Error> {
        let mut expr = self.and_()?;
        while self.t_match(&[TokenType::Or]) {
            let op = (*self.previous()).clone();
            let right = self.and_()?;
            expr = Expr::Logical {
                lhs:Box::new(expr),
                rhs:Box::new(right),
                op
            }
        }
        Ok(expr)
    }

    fn and_(&mut self) -> Result<Expr,Error>{
        let mut expr = self.equality()?;
        while self.t_match(&[TokenType::And]){
            let op = (*self.previous()).clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                lhs:Box::new(expr),
                op,
                rhs:Box::new(right)
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        while self.t_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().clone();
            let rhs = self.comparison()?;
            expr = Expr::Binary {
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                op: op,
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr,Error> {
        let mut expr = self.term()?;
        while self.t_match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ]) {
            let op = self.previous().clone();
            let rhs = self.term()?;
            expr = Expr::Binary {
                lhs:Box::new(expr),
                op,
                rhs:Box::new(rhs)
            }
        }
        Ok(expr)
    }

    fn term(&mut self)->Result<Expr, Error> {
        let mut expr = self.factor()?;
        while self.t_match(&[
            TokenType::Minus,
            TokenType::Plus
        ]){
            let op = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                lhs:Box::new(expr),
                op,
                rhs:Box::new(right)
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr,Error> {
        let mut expr = self.unary()?;
        while self.t_match(&[TokenType::Slash,TokenType::Star]){
            let op = self.previous().clone();
            let rhs = self.unary()?;
            expr = Expr::Binary {
                lhs:Box::new(expr),
                op,
                rhs:Box::new(rhs)
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error>{
        if self.t_match(&[
            TokenType::Bang,
            TokenType::Minus
        ]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {op,rhs:Box::new(right)})
        }else{
            self.call()
        }
    }

    fn call(&mut self)-> Result<Expr, Error>{
        let mut expr = self.primary()?;
        loop {
            if self.t_match(&[TokenType::LeftParen]){
                expr = self.finish_call(expr)?;
            }
            else if self.t_match(&[TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expect prop name after '.'")?;
                expr = Expr::Get {
                    object:Box::new(expr),
                    name
                }
            }
            else{
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self,callee:Expr)->Result<Expr,Error>{
        //C has 127 argument limit
        let mut arguments:Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen){
            loop {
                //Inc/Dec your parameter len here
                if arguments.len() >= 255 {
                    self.error(self.peek(), "Cannot have more than 255 arguements.");
                }
                arguments.push(self.expression()?);
                if !self.t_match(&[TokenType::Comma]){
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments" )?;
        Ok(Expr::Call {
            callee:Box::new(callee),
            paren,
            arguments
        })
    }

    fn primary(&mut self)->Result<Expr,Error>{
        let expr = match &self.peek().token_type {
            TokenType::False => Expr::Literal { val:LiteralValue::Boolean(false) },
            TokenType::True => Expr::Literal { val:LiteralValue::Boolean(true) },
            TokenType::Nil => Expr::Literal {val:LiteralValue::Nil},
            TokenType::String {literal} => Expr::Literal {val:LiteralValue::String(literal.clone())},
            TokenType::Number {literal} => Expr::Literal {val:LiteralValue::Number(*literal)},
            TokenType::LeftParen => {
                self.advance();
                let expression = self.expression()?;
                self.consume(TokenType::RightParen,"Expect ')' after expression")?;
                return Ok(Expr::Grouping {
                    expr: Box::new(expression)
                });
            },
            TokenType::Super => {
                let keyword = self.advance().clone();
                self.consume(TokenType::Dot,"Expect '.' after 'super'")?;
                let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;
                return Ok(Expr::Super {keyword, method});
            },
            TokenType::This => Expr::This {keyword:self.peek().clone()},
            TokenType::Identifier => Expr::Variable {name: self.peek().clone()},
            _ => return Err(self.error(self.peek(),"Expect expression."))
        };
        self.advance();
        Ok(expr)
    }

    fn consume(&mut self,t_type:TokenType,msg:&str)->Result<Token,Error>{
        if self.check(t_type) {
            Ok(self.advance().clone())
        }else{
            Err(self.error(self.peek(),msg))
        }
    }

    fn error(&self, token:&Token, msg:&str)->Error{
        parser_error(token,msg);
        Error::Parse
    }

    fn sync(&mut self){
        self.advance();
        while !self.is_at_end() {
            if(self.previous().token_type==TokenType::Semicolon) { return; }
            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fn
                | TokenType::Var
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }
}

//TODO write unit tests for Parser