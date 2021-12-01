use crate::frontend::error::Error;
use crate::frontend::tokens::Token;
use std::fmt;
use std::fmt::Formatter;

/*
TODO add docs for Visitor Design Pattern
 */
pub trait Visitor<T> {
    fn visit_assign_expr(&mut self, name: &Token, val: &Expr) -> Result<T, Error>;
    fn visit_binary_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<T, Error>;
    fn visit_call_expr(&mut self, callee:&Expr, paren:&Token, arguments:&Vec<Expr>)->Result<T,Error>;
    fn visit_get_expr(&mut self, object:&Expr,name:&Token)->Result<T,Error>;
    fn visit_set_expr(&mut self, object:&Expr, name:&Token, value:&Expr) -> Result<T,Error>;
    fn visit_super_expr(&mut self, keyword:&Token, method:&Token) -> Result<T,Error>;
    fn visit_this_expr(&mut self,keyword:&Token) -> Result<T,Error>;
    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<T, Error>;
    fn visit_literal_expr(&mut self, val: &LiteralValue) -> Result<T, Error>;
    fn visit_logical_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<T, Error>;
    fn visit_unary_expr(&mut self, op: &Token, rhs: &Expr) -> Result<T, Error>;
    fn visit_variable_expr(&mut self, name: &Token) -> Result<T, Error>;
}

impl Expr {
    //Generics and dyn for Traits
    pub fn accept<T>(&self, v: &mut dyn Visitor<T>) -> Result<T, Error> {
        match self {
            Expr::Assign { name, val } => v.visit_assign_expr(name, val),
            Expr::Binary { lhs, rhs, op } => v.visit_binary_expr(lhs, rhs, op),
            Expr::Call {callee,paren, arguments} => v.visit_call_expr(callee,paren,arguments),
            Expr::Get {object, name} => v.visit_get_expr(object, name),
            Expr::Set {object,name,value}=>v.visit_set_expr(object,name,value),
            Expr::Super {keyword, method} => v.visit_super_expr(keyword, method),
            Expr::This {keyword} => v.visit_this_expr(keyword),
            Expr::Grouping { expr } => v.visit_grouping_expr(expr),
            Expr::Literal { val } => v.visit_literal_expr(val),
            Expr::Logical { lhs, rhs, op } => v.visit_logical_expr(lhs, rhs, op),
            Expr::Unary { op, rhs } => v.visit_unary_expr(op, rhs),
            Expr::Variable { name } => v.visit_variable_expr(name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        val: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Call{
        callee:Box<Expr>,
        paren: Token,
        arguments:Vec<Expr>
    },
    Get{
        object:Box<Expr>,
        name:Token
    },
    Set{
        object:Box<Expr>,
        name:Token,
        value:Box<Expr>
    },
    Super{
        keyword:Token,
        method:Token
    },
    This{
        keyword:Token
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        val: LiteralValue,
    },
    Logical {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Unary {
        op: Token,
        rhs: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "{}", n),
            LiteralValue::String(s) => write!(f, "{}", s),
            LiteralValue::Boolean(b) => write!(f, "{}", b),
            LiteralValue::Nil => write!(f, "nil")
        }
    }
}

//Debugging AST
pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> Result<String, Error> {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: String, exprs: Vec<&Expr>) -> Result<String, Error> {
        let mut r = String::new();
        r.push('(');
        r.push_str(&name);
        for e in exprs {
            r.push(' ');
            r.push_str(&e.accept(self)?);
        }
        r.push(')');
        Ok(r)
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_assign_expr(&mut self, name: &Token, val: &Expr) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), vec![val])
    }

    fn visit_binary_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<String, Error> {
        self.parenthesize(op.lexeme.clone(), vec![lhs, rhs])
    }

    fn visit_call_expr(&mut self, callee: &Expr, paren: &Token, arguments: &Vec<Expr>) -> Result<String, Error> {
        todo!()
    }

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<String, Error> {
        todo!()
    }

    fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<String, Error> {
        todo!()
    }

    fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> Result<String, Error> {
        todo!()
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<String, Error> {
        todo!()
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<String, Error> {
        self.parenthesize("group".to_string(), vec![expr])
    }

    fn visit_literal_expr(&mut self, val: &LiteralValue) -> Result<String, Error> {
        Ok(val.to_string())
    }

    fn visit_logical_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<String, Error> {
        self.parenthesize(op.lexeme.clone(), vec![lhs, rhs])
    }

    fn visit_unary_expr(&mut self, op: &Token, rhs: &Expr) -> Result<String, Error> {
        self.parenthesize(op.lexeme.clone(), vec![rhs])
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<String, Error> {
        Ok(name.lexeme.clone())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::tokens::{TokenType, Token};

    #[test]
    fn default_test() {
        //expr = -420 * (421)
        let expression = Expr::Binary {
            lhs: Box::new(Expr::Unary {
                op: Token::new(TokenType::Minus, "-", 1),
                rhs: Box::new(Expr::Literal {
                    val: LiteralValue::Number(420f64),
                }),
            }),
            op: Token::new(TokenType::Star, "*", 1),
            rhs: Box::new(
                Expr::Grouping {
                    expr: Box::new(
                        Expr::Literal {
                            val: LiteralValue::Number(421f64)
                        }
                    )
                }
            ),
        };
        let mut printer = AstPrinter;
        assert_eq!(
            printer.print(expression).unwrap(),
            "(* (- 420) (group 421))"
        )
    }
}