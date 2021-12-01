use std::collections::HashMap;
use crate::frontend::fxunit::FxUnit;
use crate::frontend::tokens::Token;
use crate::frontend::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Environment {
    //Parent Pointer Tree
    pub(crate) enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, FxUnit>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn from(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: FxUnit) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<FxUnit, Error> {
        let key = &*name.lexeme;
        if let Some(val) = self.values.get(key) {
            Ok((*val).clone())
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow().get(name)
            } else {
                Err(Error::Runtime { token: name.clone(), message: format!("Undefined variable '{}'.", key) })
            }
        }
    }

    pub fn assign(&mut self, name: &Token, val: FxUnit) -> Result<(), Error> {
        let key = &*name.lexeme;
        if self.values.contains_key(key) {
            self.values.insert(name.lexeme.clone(), val);
            Ok(())
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow_mut().assign(name, val)
            } else {
                Err(Error::Runtime { token: name.clone(), message: format!("Undefined variable '{}'.", key) })
            }
        }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let parent = self.enclosing.clone().expect(&format!("No enclosing environment at {}", 1));
        let mut environment = Rc::clone(&parent);
        for i in 1..distance {
            let parent = environment.borrow().enclosing.clone().expect(&format!("No enclosing environment at {}", i));
            environment = Rc::clone(&parent);
        }
        environment
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<FxUnit, Error> {
        if distance > 0 {
            Ok(self.ancestor(distance).borrow().values.get(name).expect(&format!("Undefined variable '{}'", name)).clone())
        } else {
            Ok(self.values.get(name).expect(&format!("Undefined variable '{}'", name)).clone())
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: FxUnit) -> Result<(), Error> {
        if distance > 0 {
            self.ancestor(distance).borrow_mut().values.insert(name.lexeme.clone(), value);
        } else {
            self.values.insert(name.lexeme.clone(), value);
        }
        Ok(())
    }
}