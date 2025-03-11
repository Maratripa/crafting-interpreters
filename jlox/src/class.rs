use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    functions::{Callable, LoxFunction},
    interpreter::Interpreter,
    object::Object,
    token::Token,
};

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    superclass: Option<Rc<RefCell<Class>>>,
    methods: HashMap<String, LoxFunction>,
}

impl Class {
    pub fn new(
        name: String,
        superclass: Option<Rc<RefCell<Class>>>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        } else if let Some(superclass) = &self.superclass {
            return superclass.borrow().find_method(name);
        } else {
            return None;
        }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Callable for Class {
    type E = crate::interpreter::Error;

    fn arity(&self) -> usize {
        let initializer = self.find_method("init");

        if let Some(init) = initializer {
            init.arity()
        } else {
            0
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Rc<Object>>,
    ) -> Result<Rc<Object>, Self::E> {
        let instance = Rc::new(RefCell::new(Instance::new(Rc::new(RefCell::new(
            self.clone(),
        )))));

        let initializer = self.find_method("init");

        if let Some(init) = initializer {
            init.bind(instance.clone()).call(interpreter, arguments)?;
        }

        Ok(Rc::new(Object::Instance(instance)))
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    klass: Rc<RefCell<Class>>,
    fields: HashMap<String, Rc<Object>>,
}

impl Instance {
    pub fn new(klass: Rc<RefCell<Class>>) -> Self {
        Self {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: Token) -> Result<Rc<Object>, crate::interpreter::Error> {
        if self.fields.contains_key(&name.lexeme) {
            return Ok(self.fields.get(&name.lexeme).unwrap().clone());
        }

        if let Some(method) = self.klass.borrow().find_method(&name.lexeme) {
            return Ok(Rc::new(Object::Function(Rc::new(
                method.bind(Rc::new(RefCell::new(self.clone()))),
            ))));
        }

        Err(crate::interpreter::Error::UndefinedProperty { name: name.lexeme })
    }

    pub fn set(&mut self, name: Token, value: Rc<Object>) {
        self.fields.insert(name.lexeme, value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.borrow().to_string())
    }
}
