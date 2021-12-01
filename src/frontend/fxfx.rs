use crate::frontend::fxunit::FxUnit;
use crate::frontend::tokens::{Token, TokenType};
use crate::frontend::stmt::Stmt;
use crate::frontend::env::Environment;
use std::rc::Rc;
use std::cell::RefCell;
use crate::frontend::interpreter::Interpreter;
use crate::frontend::error::Error;
use std::fmt;

#[derive(Clone)]
pub enum FxFx{
    Native{
        arity:usize,
        body: Box<fn(&Vec<FxUnit>)->FxUnit>
    },

    User{
        name:Token,
        params:Vec<Token>,
        body:Vec<Stmt>,
        closure:Rc<RefCell<Environment>>,
        is_init:bool
    }
}

impl FxFx{
    pub fn call(&self, interpreter:&mut Interpreter, args:&Vec<FxUnit>)->Result<FxUnit,Error>{
        match self {
            FxFx::Native {body,..}=>{
                Ok(body(args))
            },
            FxFx::User {params,body,closure,is_init,..}=>{
                let mut env = Rc::new(RefCell::new(Environment::from(closure)));
                for (param,arg) in params.iter().zip(args.iter()){
                    env.borrow_mut().define(param.lexeme.clone(), arg.clone());
                }
                match interpreter.exec_block(body,env) {
                    Err(Error::Return{value}) => {
                        if *is_init {
                            Ok(closure.borrow().get_at(0,"this").expect("Initializer should return 'this'"))
                        }else{
                            Ok(value)
                        }
                    },
                    Err(other)=>Err(other),
                    Ok(..)=> {
                        if *is_init {
                            Ok(closure.borrow().get_at(0,"this").expect("Initializer should return 'this'"))
                        }else{
                            Ok(FxUnit::Nil)
                        }
                    }
                }
            }
        }
    }
    pub fn arity(&self)->usize{
        match self {
            FxFx::Native {arity,..}=>*arity,
            FxFx::User {params, ..}=> params.len()
        }
    }

    pub fn bind(&self, instance:FxUnit) -> Self {
        match self {
            FxFx::Native {body, ..} => unreachable!(),
            FxFx::User {name,params,body,closure, is_init} => {
                let mut env = Rc::new(RefCell::new(Environment::from(closure)));
                env.borrow_mut().define("this".to_string(),instance);
                FxFx::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: env,
                    is_init:*is_init
                }
            }
        }
    }
}

impl fmt::Debug for FxFx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxFx::Native { .. } => write!(f, "<native func>"),
            FxFx::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}

impl fmt::Display for FxFx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxFx::Native { .. } => write!(f, "<native func>"),
            FxFx::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}