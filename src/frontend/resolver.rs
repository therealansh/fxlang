use crate::frontend::interpreter::Interpreter;
use crate::frontend::expr::{Visitor, Expr, LiteralValue};
use crate::frontend::{stmt, expr};
use crate::frontend::error::{Error, parser_error, report};
use crate::frontend::tokens::{Token, TokenType};
use crate::frontend::stmt::Stmt;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer
}

#[derive(Debug, Clone)]
enum ClassType {
    None,
    Class,
    SubClass
}

pub struct Resolver<'a> {
    interpreter:&'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_func:FunctionType,
    current_class:ClassType,
    pub had_error:bool
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver { interpreter, scopes: Vec::new(), current_func:FunctionType::None, current_class:ClassType::None, had_error:false }
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<Stmt>) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        stmt.accept(self);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        expr.accept(self);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self){
        self.scopes.pop();
    }

    fn declare(&mut self, name:&Token){
        let mut already_defined:bool = false;
        match self.scopes.last_mut() {
            Some(ref mut scope) => {
                already_defined = scope.contains_key(&name.lexeme);
                scope.insert(name.lexeme.clone(),false);
            },
            None => ()
        }
        if already_defined{
            self.error(name, "Variable with this name already declared.")
        }
    }

    fn define(&mut self, name:&Token){
        match self.scopes.last_mut() {
            Some(ref mut scope) => {
                scope.insert(name.lexeme.clone(),true);
            },
            None => ()
        }
    }

    fn resolve_local(&mut self, name:&Token){
        for (i,scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme){
                self.interpreter.resolve(name,i);
            }
        }
    }

    fn resolve_func(&mut self,params: &Vec<Token>, body: &Vec<Stmt>, fx_type: FunctionType ){
        let enclosing_func = self.current_func.clone();
        self.current_func = fx_type;
        self.begin_scope();
        for param in params{
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();
        self.current_func = enclosing_func;
    }

    fn error(&mut self, token:&Token, msg:&str){
        if token.token_type==TokenType::Eof{
            report(token.line, " at end", msg);
        }else{
            report(token.line, &format!(" at '{}'", token.lexeme), msg);
        }
        self.had_error = true;
    }

}

impl<'a> stmt::Visitor<()> for Resolver<'a> {
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        self.begin_scope();
        self.resolve_stmts(statements);
        self.end_scope();
        Ok(())
    }

    fn visit_func_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) -> Result<(), Error> {
        self.declare(name);
        self.define(name);
        self.resolve_func(params,body, FunctionType::Function);
        Ok(())
    }

    fn visit_class_stmt(&mut self, name: &Token, superclass:&Option<Expr>, methods: &Vec<Stmt>) -> Result<(), Error> {
        let enclosing_class = mem::replace(&mut self.current_class,ClassType::Class);
        self.declare(name);
        self.define(name);

        if let Some(Expr::Variable {name:superclass_name}) = superclass{
            if name.lexeme == superclass_name.lexeme {
                self.error(superclass_name, "A class cannot inherit from itself.");
            }
            self.current_class = ClassType::SubClass;
            self.resolve_local(superclass_name);
            self.begin_scope();
            self.scopes.last_mut().expect("Scope is empty.").insert("super".to_owned(),true);
        }

        self.begin_scope();
        self.scopes.last_mut().expect("Scope is empty.").insert("this".to_owned(),true);

        for method in methods{
            if let Stmt::FxFx {name,params,body} = method {
                let declaration = if name.lexeme=="init"{
                    FunctionType::Initializer
                }else{
                    FunctionType::Method
                };
                self.resolve_func(params,body,declaration);
            }else{
                unreachable!()
            }
        }
        if superclass.is_some(){
            self.end_scope()
        }
        self.end_scope();
        self.current_class = enclosing_class;
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) -> Result<(), Error> {
        if let FunctionType::None = self.current_func {
            self.error(
                keyword,
                "Cannot return from top-level code."
            );
        }
        if let Some(return_val) = value {
            self.resolve_expr(return_val);
        }
        Ok(())
    }

    fn visit_if_stmt(&mut self, condition: &Expr, else_branch: &Option<Stmt>, then_branch: &Stmt) -> Result<(), Error> {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch);
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, statement: &Stmt) -> Result<(), Error> {
        self.resolve_expr(condition);
        self.resolve_stmt(statement);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        self.resolve_expr(expr);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve_expr(init);
        }
        self.define(name);
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        self.resolve_expr(expr);
        Ok(())
    }
}

impl<'a> expr::Visitor<()> for Resolver<'a>{
    fn visit_assign_expr(&mut self, name: &Token, val: &Expr) -> Result<(), Error> {
        self.resolve_expr(val);
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<(), Error> {
        self.resolve_expr(lhs);
        self.resolve_expr(rhs);
        Ok(())
    }

    fn visit_call_expr(&mut self, callee: &Expr, paren: &Token, arguments: &Vec<Expr>) -> Result<(), Error> {
        self.resolve_expr(callee);
        for arg in arguments {
            self.resolve_expr(arg);
        }
        Ok(())
    }

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<(), Error> {
        self.resolve_expr(object);
        Ok(())
    }

    fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<(),Error>{
        self.resolve_expr(value);
        self.resolve_expr(object);
        Ok(())
    }

    fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> Result<(), Error> {
        match self.current_class {
            ClassType::None => self.error(keyword, "Cannot use super outside of a class."),
            ClassType::Class => self.error(keyword, "Cannot use 'super' in the base class."),
            _ => self.resolve_local(keyword)
        }
        Ok(())
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<(), Error> {
        if let ClassType::None = self.current_class {
            self.error(keyword, "Cannot use 'this' outside of class");
        }else{
            self.resolve_local(keyword);
        }
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<(), Error> {
        self.resolve_expr(expr);
        Ok(())
    }

    fn visit_literal_expr(&mut self, val: &LiteralValue) -> Result<(), Error> {
        Ok(())
    }

    fn visit_logical_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<(), Error> {
        self.resolve_expr(lhs);
        self.resolve_expr(rhs);
        Ok(())
    }

    fn visit_unary_expr(&mut self, op: &Token, rhs: &Expr) -> Result<(), Error> {
        self.resolve_expr(rhs);
        Ok(())
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<(), Error> {
        if let Some(scope) = self.scopes.last() {
            if let Some(flag) = scope.get(&name.lexeme){
                if *flag == false {
                    self.error(name, "Cannot read local var in its own initializer.");
                }
            }
        };
        self.resolve_local(name);
        Ok(())
    }
}