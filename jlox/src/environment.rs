use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::object::Object;
use crate::token::Token;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Undefined variable '{name}'.")]
    UndefinedVariable { name: String },

    #[error("Environment does not have an enclosing")]
    EnclosingError,
}

#[derive(Debug)]
pub struct Environment {
    pub values: HashMap<String, Rc<Object>>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: String, value: Rc<Object>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Rc<Object>, Error> {
        if self.values.contains_key(name) {
            return Ok(self.values.get(name).unwrap().clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().get(name);
        }

        Err(Error::UndefinedVariable {
            name: name.to_string(),
        })
    }
    pub fn assign(&mut self, name: Token, value: Rc<Object>) -> Result<(), Error> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(Error::UndefinedVariable { name: name.lexeme })
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Rc<Object>, Error> {
        if distance == 0 {
            return Ok(self.get(name)?);
        } else {
            let ancestor = self.ancestor(distance)?;
            return Ok(ancestor.borrow_mut().get(name)?);
        }
    }

    fn ancestor(&self, distance: usize) -> Result<Rc<RefCell<Self>>, Error> {
        if let Some(enclosing) = &self.enclosing {
            let mut env = enclosing.clone();

            for _ in 1..distance {
                let temp = env.borrow_mut().enclosing.clone();
                if let Some(enc) = temp {
                    env = enc;
                } else {
                    return Err(Error::EnclosingError);
                }
            }

            return Ok(env);
        }

        Err(Error::EnclosingError)
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: Token,
        value: Rc<Object>,
    ) -> Result<(), Error> {
        if distance == 0 {
            self.assign(name, value)?;
        } else {
            self.ancestor(distance)?.borrow_mut().assign(name, value)?;
        }

        Ok(())
    }
}
