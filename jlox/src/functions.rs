use std::{
    cell::RefCell,
    fmt::Display,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::Stmt,
    class::Instance,
    environment::Environment,
    interpreter::{Error, Interpreter},
    object::Object,
};

pub trait Callable {
    type E;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, Self::E>;

    fn arity(&self) -> usize;
}

impl std::fmt::Debug for dyn Callable<E = Error> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lox function>")
    }
}

pub struct Clock;

impl Callable for Clock {
    type E = Error;

    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Ok(Rc::new(Object::Number(now as f64)))
    }
}

impl std::fmt::Debug for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function>")
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: String,
    closure: Rc<RefCell<Environment>>,
    params: Vec<String>,
    body: Rc<Vec<Stmt>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: String,
        closure: Rc<RefCell<Environment>>,
        params: Vec<String>,
        body: Rc<Vec<Stmt>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            closure,
            params,
            body,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Self {
        let mut environment = Environment::new(Some(self.closure.clone()));
        environment.define("this".to_string(), Rc::new(Object::Instance(instance)));
        Self::new(
            self.name.clone(),
            Rc::new(RefCell::new(environment)),
            self.params.clone(),
            self.body.clone(),
            self.is_initializer,
        )
    }
}

impl Callable for LoxFunction {
    type E = Error;

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, Error> {
        let environment = Rc::new(RefCell::new(Environment::new(Some(
            (&self.closure).clone(),
        ))));

        // println!("Before: {environment:?}");

        for (i, arg) in arguments.into_iter().enumerate() {
            environment
                .borrow_mut()
                .define(self.params[i].to_owned(), arg);
        }

        // println!("After: {environment:?}");

        match interpreter.execute_block(self.body.clone(), environment) {
            Ok(_) => {
                if self.is_initializer {
                    self.closure
                        .borrow()
                        .get_at(0, "this")
                        .map_err(|e| Error::EnvironmentError { error: e })
                } else {
                    Ok(Rc::new(Object::Nil))
                }
            }
            Err(Error::Return { value }) => {
                if self.is_initializer {
                    self.closure
                        .borrow()
                        .get_at(0, "this")
                        .map_err(|e| Error::EnvironmentError { error: e })
                } else {
                    Ok(value)
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", &self.name)
    }
}
