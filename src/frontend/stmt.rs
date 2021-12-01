use crate::frontend::expr::Expr;
use crate::frontend::error::Error;
use crate::frontend::tokens::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    FxFx{
        name:Token,
        params:Vec<Token>,
        body:Vec<Stmt>
    },
    Return{
        keyword:Token,
        value:Option<Expr>
    },
    Block {
        statements: Vec<Stmt>
    },
    Class {
        name:Token,
        superclass:Option<Expr>,
        methods: Vec<Stmt>
    },
    If {
        condition: Expr,
        else_branch: Box<Option<Stmt>>,
        then_branch: Box<Stmt>,
    },
    While {
        condition: Expr,
        statement: Box<Stmt>,
    },
    Expression {
        expr: Expr
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Print {
        expr: Expr
    },
    Nil,
}

impl Stmt {
    pub fn accept<R>(&self, v: &mut dyn Visitor<R>) -> Result<R, Error> {
        match self {
            Stmt::Block { statements } => v.visit_block_stmt(statements),
            Stmt::If { condition, else_branch, then_branch } => v.visit_if_stmt(condition, else_branch, then_branch),
            Stmt::While { condition, statement } => v.visit_while_stmt(condition, statement),
            Stmt::Expression { expr: expression } => v.visit_expression_stmt(expression),
            Stmt::Var { name, initializer } => v.visit_var_stmt(name, initializer),
            Stmt::Print { expr: expression } => v.visit_print_stmt(expression),
            Stmt::Nil => unimplemented!(),
            Stmt::FxFx { name, params, body } => v.visit_func_stmt(name,params,body),
            Stmt::Return { keyword,value } => v.visit_return_stmt(keyword,value),
            Stmt::Class {name, superclass, methods} => v.visit_class_stmt(name, superclass, methods)
        }
    }
}

pub trait Visitor<T> {
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<T, Error>;
    fn visit_func_stmt(&mut self, name:&Token, params:&Vec<Token>, body:&Vec<Stmt>)->Result<T,Error>;
    fn visit_class_stmt(&mut self, name:&Token, superclass:&Option<Expr>,methods:&Vec<Stmt>) -> Result<T, Error>;
    fn visit_return_stmt(&mut self, keyword:&Token , value:&Option<Expr>)->Result<T,Error>;
    fn visit_if_stmt(&mut self, condition: &Expr, else_branch: &Option<Stmt>, then_branch: &Stmt) -> Result<T, Error>;
    fn visit_while_stmt(&mut self, condition: &Expr, statement: &Stmt) -> Result<T, Error>;
    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<T, Error>;
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<T, Error>;
    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<T, Error>;
}