use std::cell::RefCell;
use std::rc::Rc;
use crate::frontend::fxunit::FxUnit;
use std::collections::HashMap;
use crate::frontend::tokens::Token;
use crate::frontend::error::Error;
use crate::frontend::fxfx::FxFx;

#[derive(Debug)]
pub struct FxClass{
    pub name:String,
    pub superclass: Option<Rc<RefCell<FxClass>>>,
    pub methods:HashMap<String,FxFx>
}

impl FxClass{
    pub fn find_method(&self, name:&str)->Option<FxFx>{
        if self.methods.contains_key(name){
            self.methods.get(name).map(|f| f.clone())
        }else{
            if let Some(ref superclass) = self.superclass{
                superclass.borrow().find_method(name)
            }else{
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct FxClassInstance{
    pub class:Rc<RefCell<FxClass>>,
    fields:HashMap<String, FxUnit>
}

impl FxClassInstance {
    pub fn new(class:&Rc<RefCell<FxClass>>) -> FxUnit {
        let instance = FxClassInstance{
            class:Rc::clone(class),
            fields:HashMap::new()
        };
        FxUnit::Instance(Rc::new(RefCell::new(instance)))
    }

    pub fn get(&self, name:&Token, instance:&FxUnit)->Result<FxUnit,Error>{
        if let Some(field) = self.fields.get(&name.lexeme){
            Ok(field.clone())
        }
        else if let Some(method) = self.class.borrow().find_method(&name.lexeme) {
            Ok(FxUnit::Callable(method.bind(instance.clone())))
        }
        else{
            Err(Error::Runtime {
                token:name.clone(),
                message:format!("Undefined prop '{}'.", name.lexeme)
            })
        }
    }

    pub fn set(&mut self, name:&Token, value:FxUnit){
        self.fields.insert(name.lexeme.clone(),value);
    }
}