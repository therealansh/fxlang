use crate::frontend::error::Error;
use crate::frontend::tokens::{TokenType, Token};
use crate::frontend::fxunit::FxUnit;
use crate::frontend::expr::*;
use crate::frontend::{expr, stmt};
use crate::frontend::stmt::Stmt;
use std::rc::Rc;
use std::cell::RefCell;
use crate::frontend::env::Environment;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::frontend::fxfx::FxFx;
use std::collections::HashMap;
use std::io;
use crate::frontend::fxclass::{FxClass, FxClassInstance};

pub struct Interpreter {
    pub globals:Rc<RefCell<Environment>>,
    env: Rc<RefCell<Environment>>,
    locals:HashMap<Token, usize>  //TODO Fix this locals fucks up the for loop init
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        //Native Clock Func
        let clock:FxUnit = FxUnit::Callable(
            FxFx::Native{
                arity:0,
                body: Box::new(|args:&Vec<FxUnit>|{
                    FxUnit::Number(
                        SystemTime::now().duration_since(UNIX_EPOCH).expect("Could not get time.").as_secs_f64() as f64
                    )
                })
            }
        );
        globals.borrow_mut().define("clock".to_string(), clock);

        //Native IO
        pub fn get_input() -> String {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).expect("Failed to get input fxfx hung.");
            buffer
        }

        let readNum:FxUnit = FxUnit::Callable(
            FxFx::Native {
                arity:0,
                body: Box::new(|args:&Vec<FxUnit>|{
                    FxUnit::Number(
                        get_input().trim().parse().unwrap()
                    )
                })
            }
        );
        globals.borrow_mut().define("readNum".to_string(),readNum);
        //TODO can we refactor this IO mod??
        let readString:FxUnit = FxUnit::Callable(
            FxFx::Native {
                arity:0,
                body:Box::new(|args:&Vec<FxUnit>|{
                    FxUnit::String(
                        get_input().trim().to_string()
                    )
                })
            }
        );
        globals.borrow_mut().define("readString".to_string(),readString);

        Interpreter {
            globals:Rc::clone(&globals),
            env: Rc::clone(&globals),
            locals:HashMap::new()
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    pub fn exec_block(&mut self, statements: &Vec<Stmt>, environment: Rc<RefCell<Environment>>) -> Result<(), Error> {
        //Stores the previous env
        let prev = self.env.clone();
        //Exec the block statement
        let steps = || -> Result<(),Error>{
            self.env = environment;
            for stmt in statements {
                self.execute(stmt)?
            }
            Ok(())
        };
        //Restore the prev env for global vars
        let res = steps();
        self.env = prev;
        res
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), Error> {
        statement.accept(self)
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<FxUnit, Error> {
        expression.accept(self)
    }

    fn is_truthy(&self, fxunit: &FxUnit) -> bool {
        match fxunit {
            FxUnit::Nil => false,
            FxUnit::Boolean(b) => b.clone(),
            _ => true,
        }
    }

    fn is_equal(&self, l_unit: &FxUnit, r_unit: &FxUnit) -> bool {
        l_unit.equals(r_unit)
    }

    fn num_op_error<R>(&self, op: &Token) -> Result<R, Error> {
        Err(Error::Runtime { token: op.clone(), message: "Operand must be a number".to_string() })
    }

    fn stringify(&self, fxunit: FxUnit) -> String {
        match fxunit {
            FxUnit::Boolean(b) => b.to_string(),
            FxUnit::Callable(f) => f.to_string(),
            FxUnit::Nil => "NIL".to_string(),
            FxUnit::Number(n) => n.to_string(),
            FxUnit::Class(c) => c.borrow().name.clone(),
            FxUnit::Instance(i) => format!("{} instance", i.borrow().class.borrow().name),
            FxUnit::String(s) => s
        }
    }

    pub fn resolve(&mut self, name:&Token,depth:usize){
        self.locals.insert(name.clone(), depth);
    }

    fn look_up_var(&self, name:&Token)->Result<FxUnit,Error>{
        if let Some(dist) = self.locals.get(name){
            self.env.borrow().get_at(*dist,&name.lexeme)
        }else{
            self.globals.borrow().get(name)
        }
    }
}

impl expr::Visitor<FxUnit> for Interpreter {
    fn visit_assign_expr(&mut self, name: &Token, val: &Expr) -> Result<FxUnit, Error> {
        let value = self.evaluate(val)?;
        if let Some(dist) = self.locals.get(name){
            self.env.borrow_mut().assign_at(*dist, name, value.clone())?;
        }else{
            self.env.borrow_mut().assign(name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_binary_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<FxUnit, Error> {
        let l = self.evaluate(lhs)?;
        let r = self.evaluate(rhs)?;
        match &op.token_type {
            TokenType::Minus => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Number(l_num - r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::Slash => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Number(l_num / r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::Star => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Number(l_num * r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::Plus => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Number(l_num + r_num)),
                (FxUnit::String(l_str), FxUnit::String(r_str)) => Ok(FxUnit::String(l_str.clone() + &r_str.clone())),
                _ => Err(Error::Runtime { token: op.clone(), message: "Operands must be numbers or strings".to_string() })
            },
            TokenType::Greater => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Boolean(l_num > r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::GreaterEqual => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Boolean(l_num >= r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::Less => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Boolean(l_num < r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::LessEqual => match (l, r) {
                (FxUnit::Number(l_num), FxUnit::Number(r_num)) => Ok(FxUnit::Boolean(l_num <= r_num)),
                _ => self.num_op_error(op)
            },
            TokenType::BangEqual => Ok(FxUnit::Boolean(!self.is_equal(&l, &r))),
            TokenType::EqualEqual => Ok(FxUnit::Boolean(self.is_equal(&l, &r))),
            _ => unreachable!()
        }
    }

    fn visit_call_expr(&mut self, callee: &Expr, paren: &Token, arguments: &Vec<Expr>) -> Result<FxUnit, Error> {
        let callee = self.evaluate(callee)?;
        let args_vals:Result<Vec<FxUnit>,Error> = arguments.into_iter().map(|expr| self.evaluate(expr)).collect();
        let args = args_vals?;
        match callee {
            FxUnit::Callable(func) => {
                let args_size = args.len();
                if args_size!=func.arity() {
                    Err(Error::Runtime {
                        token:paren.clone(),
                        message:format!("Expected {} args but found {}.", func.arity(),args_size)
                    })
                }else{
                    func.call(self,&args)
                }
            },
            FxUnit::Class(ref class )=> {
                let args_size = args.len();
                let instance = FxClassInstance::new(class);
                if let Some(init) = class.borrow().find_method("init"){
                    if args_size!=init.arity(){
                        return Err(Error::Runtime {
                            token:paren.clone(),
                            message:format!("Expected {} args but found {}.", init.arity(),args_size)
                        })
                    }else{
                        init.bind(instance.clone()).call(self,&args)?;
                    }
                }
                Ok(instance)
            }
            _ => Err(Error::Runtime {token:paren.clone(), message:"Can only call funcs and classes.".to_string()})
        }
    }

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<FxUnit, Error> {
        let object = self.evaluate(object)?;
        if let FxUnit::Instance(ref ins) = object {
            ins.borrow().get(name, &object)
        }else{
            Err(Error::Runtime {
                token:name.clone(),
                message:"Only instances can have props.".to_string()
            })
        }
    }

    fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<FxUnit, Error> {
        let obj = self.evaluate(object)?;
        if let FxUnit::Instance(ref instance) = obj {
            let value = self.evaluate(value)?;
            instance.borrow_mut().set(name, value);
            let r = FxUnit::Instance(Rc::clone(instance));
            Ok(r)
        }else {
            Err(Error::Runtime {
                token:name.clone(),
                message:"Only instances have fields.".to_string()
            })
        }
    }

    fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> Result<FxUnit, Error> {
        let dist= self.locals.get(keyword).expect("No Local distance for 'super'");
        let superclass = self.env.borrow().get_at(*dist, "super")?;
        let instance = self.env.borrow().get_at(*dist-1, "this")?;
        if let FxUnit::Class(ref superclass) = superclass {
            if let Some(method) = superclass.borrow().find_method(&method.lexeme){
                Ok(FxUnit::Callable(method.bind(instance)))
            }else{
                Err(Error::Runtime {
                    token:method.clone(),
                    message:format!("Undefined prop:{}", method.lexeme)
                })
            }
        }else{
            unreachable!()
        }
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<FxUnit, Error> {
        self.look_up_var(keyword)
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<FxUnit, Error> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&mut self, val: &LiteralValue) -> Result<FxUnit, Error> {
        match val {
            LiteralValue::Boolean(b) => Ok(FxUnit::Boolean(b.clone())),
            LiteralValue::Nil => Ok(FxUnit::Nil),
            LiteralValue::Number(n) => Ok(FxUnit::Number(n.clone())),
            LiteralValue::String(s) => Ok(FxUnit::String(s.clone()))
        }
    }

    fn visit_logical_expr(&mut self, lhs: &Expr, rhs: &Expr, op: &Token) -> Result<FxUnit, Error> {
        let left = self.evaluate(lhs)?;
        if op.token_type==TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        }else{
            if !self.is_truthy(&left){
                return Ok(left)
            }
        }
        self.evaluate(rhs)
    }

    fn visit_unary_expr(&mut self, op: &Token, rhs: &Expr) -> Result<FxUnit, Error> {
        let right = self.evaluate(rhs)?;
        match &op.token_type {
            TokenType::Minus => match right {
                FxUnit::Number(n) => Ok(FxUnit::Number(-n.clone())),
                _ => self.num_op_error(op)
            },
            TokenType::Bang => Ok(FxUnit::Boolean(!self.is_truthy(&right))),
            _ => unreachable!()
        }
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<FxUnit, Error> {
        self.look_up_var(name)
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        self.exec_block(
            statements,
            Rc::new(RefCell::new(Environment::from(&self.env))),
        );
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), Error> {
        let val = self.evaluate(expr)?;
        println!("{}", self.stringify(val));
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        let value: FxUnit = initializer.as_ref().map(|i| self.evaluate(i)).unwrap_or(Ok(FxUnit::Nil))?;
        self.env.borrow_mut().define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_if_stmt(&mut self, condition: &Expr, else_branch: &Option<Stmt>, then_branch: &Stmt) -> Result<(), Error> {
        let cond = self.evaluate(condition)?;
        if self.is_truthy(&cond){
            self.execute(then_branch)?;
        }else if let Some(other) = else_branch{
            self.execute(other)?;
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, statement: &Stmt) -> Result<(), Error> {
        let mut cond = self.evaluate(condition)?;
        while self.is_truthy(&cond) {
            self.execute(statement)?;
            cond = self.evaluate(condition)?;
        }
        Ok(())
    }

    fn visit_func_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) -> Result<(), Error> {
        let func = FxFx::User{
            name:name.clone(),
            params:params.clone(),
            body:body.clone(),
            closure:Rc::clone(&self.env),
            is_init:false
        };
        self.env.borrow_mut().define(name.lexeme.clone(),FxUnit::Callable(func));
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) -> Result<(), Error> {
        let return_val = value.as_ref().map(|v| self.evaluate(v)).unwrap_or(Ok(FxUnit::Nil))?;
        Err(Error::Return {value:return_val})
    }

    fn visit_class_stmt(&mut self, name: &Token, superclass:&Option<Expr>, methods: &Vec<Stmt>) -> Result<(), Error> {
        let s_class:Option<Rc<RefCell<FxClass>>> = superclass.as_ref().map(|expr|{
            if let FxUnit::Class(ref fx_class) = self.evaluate(expr)? {
                Ok(Rc::clone(fx_class))
            }else if let Expr::Variable {name} = expr {
                Err(Error::Runtime {
                    token:name.clone(),
                    message:"Superclass must be a class.".to_string(),
                })
            }else{
                unreachable!()
            }
        }).transpose()?;
        self.env.borrow_mut().define(name.lexeme.clone(), FxUnit::Nil);

        if let Some(ref class) = s_class {
            self.env = Rc::new(RefCell::new(Environment::from(&self.env)));
            self.env.borrow_mut().define("super".to_string(), FxUnit::Class(Rc::clone(class)))
        }

        let mut class_methods:HashMap<String,FxFx> = HashMap::new();
        for method in methods{
            if let Stmt::FxFx {name,params,body} = method {
                let function = FxFx::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: Rc::clone(&self.env),
                    is_init:name.lexeme=="init"
                };
                class_methods.insert(name.lexeme.clone(),function);
            }else{
                unreachable!()
            }
        }

        let fx_class = FxClass{
            name:name.lexeme.clone(),
            superclass:s_class.clone(),
            methods:class_methods
        };
        let class = FxUnit::Class(Rc::new(RefCell::new(fx_class)));
        if s_class.is_some(){
            let parent = self.env.borrow().enclosing.clone().expect("Superclass env has no parent.");
            self.env= parent;
        }
        self.env.borrow_mut().assign(name, class);
        Ok(())
    }
}

//TODO write unit tests for interpreter