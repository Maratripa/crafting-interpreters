use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

use crate::ast::{Expr, ExprVisitor, Literal, Stmt, StmtVisitor};
use crate::class::Class;
use crate::environment::Environment;
use crate::functions::{Callable, Clock, LoxFunction};
use crate::object::Object;
use crate::token::{Token, TokenType};

pub type Number = f64;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unsupported operation between {op} and {right:?}")]
    UnsupportedUnaryOp { op: Token, right: Rc<Object> },

    #[error("Unsupported addition between {left:?} and {right:?}")]
    UnsupportedAddOp { left: Rc<Object>, right: Rc<Object> },

    #[error("Unsupported operation: {left:?} {op} {right:?}")]
    UnsupportedBinaryOp {
        left: Rc<Object>,
        op: Token,
        right: Rc<Object>,
    },

    #[error("Cast conversion failed: {value} is not a number")]
    NaN { value: String },

    #[error("Division by zero")]
    ZeroDivision,

    #[error("Environment error: {error:?}")]
    EnvironmentError { error: crate::environment::Error },

    #[error("Object is not callable: {obj:?}")]
    NotCallable { obj: Rc<Object> },

    #[error("Expected {arity} arguments but got {size}.")]
    ArityError { arity: usize, size: usize },

    #[error("Forgot to handle return statement, this should not happen.")]
    Return { value: Rc<Object> },

    #[error("{name} Only instances have properties.")]
    PropertyAccessError { name: Token },

    #[error("Undefined property '{name}'")]
    UndefinedProperty { name: String },

    #[error("{name} Only instances have fields.")]
    FieldAccessError { name: Token },

    #[error("{stmt:?} is not a function statement.")]
    MethodNotFunction { stmt: Stmt },

    #[error("{name}: Superclass must be a class.")]
    SuperClassNotClass { name: Token },
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn n(&self) -> Result<f64, Error> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err(Error::NaN {
                value: self.to_string(),
            }),
        }
    }
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    locals: HashMap<Token, usize>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));

        (*globals).borrow_mut().define(
            "clock".to_owned(),
            Rc::new(Object::Function(Rc::new(Clock {}))),
        );

        Self {
            globals: globals.clone(),
            locals: HashMap::new(),
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: Rc<Vec<Stmt>>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
        let previous = (&self.environment).clone();
        // println!("Before: {previous:?}");
        self.environment = environment;

        for stmt in statements.iter() {
            if let Err(return_type) = self.execute(stmt.clone()) {
                self.environment = previous;
                return Err(return_type);
            }
        }

        // println!("After: {previous:?}");
        self.environment = previous;

        Ok(())
    }

    pub fn copy_globals(&mut self) -> Rc<RefCell<Environment>> {
        self.globals.clone()
    }

    pub fn resolve(&mut self, name: &Token, depth: usize) {
        self.locals.insert(name.clone(), depth);
    }

    fn look_up_variable(&mut self, name: Token) -> Result<Rc<Object>, Error> {
        if let Some(distance) = self.locals.get(&name) {
            match self
                .environment
                .borrow_mut()
                .get_at(*distance, &name.lexeme)
            {
                Ok(something) => return Ok(something),
                Err(e) => return Err(Error::EnvironmentError { error: e }),
            }
        } else {
            match self.globals.borrow_mut().get(&name.lexeme) {
                Ok(something) => return Ok(something),
                Err(e) => return Err(Error::EnvironmentError { error: e }),
            }
        }
    }
}

impl ExprVisitor<Object> for Interpreter {
    type E = Error;

    fn visit_assign_expr(&mut self, name: Token, value: Box<Expr>) -> Result<Rc<Object>, Self::E> {
        let val = self.evaluate(*value)?;

        if let Some(distance) = self.locals.get(&name) {
            if let Err(e) = self
                .environment
                .borrow_mut()
                .assign_at(*distance, name, val.clone())
            {
                return Err(Error::EnvironmentError { error: e });
            };
        } else {
            if let Err(e) = self.globals.borrow_mut().assign(name, val.clone()) {
                return Err(Error::EnvironmentError { error: e });
            }
        }

        Ok(val)
    }

    fn visit_binary_expr(
        &mut self,
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    ) -> Result<Rc<Object>, Error> {
        let l = self.evaluate(*left)?;
        let r = self.evaluate(*right)?;

        match op.token_type {
            TokenType::Minus => Ok(Rc::new(Object::Number(l.n()? - r.n()?))),
            TokenType::Slash => {
                let divisor = r.n()?;
                if divisor == 0.0 {
                    return Err(Error::ZeroDivision);
                }

                Ok(Rc::new(Object::Number(l.n()? / divisor)))
            }
            TokenType::Star => Ok(Rc::new(Object::Number(l.n()? * r.n()?))),

            TokenType::Plus => match (&*l, &*r) {
                (Object::Number(n), Object::Number(m)) => Ok(Rc::new(Object::Number(n + m))),
                (Object::String(s), Object::String(t)) => {
                    Ok(Rc::new(Object::String(format!("{s}{t}"))))
                }
                (_, _) => Err(Error::UnsupportedAddOp { left: l, right: r }),
            },

            TokenType::Greater => Ok(Rc::new(Object::Bool(l.n()? > r.n()?))),
            TokenType::GreaterEqual => Ok(Rc::new(Object::Bool(l.n()? >= r.n()?))),
            TokenType::Less => Ok(Rc::new(Object::Bool(l.n()? < r.n()?))),
            TokenType::LessEqual => Ok(Rc::new(Object::Bool(l.n()? <= r.n()?))),

            TokenType::BangEqual => Ok(Rc::new(Object::Bool(!(l == r)))),
            TokenType::EqualEqual => Ok(Rc::new(Object::Bool(l == r))),

            _ => Err(Error::UnsupportedBinaryOp {
                left: l,
                op,
                right: r,
            }),
        }
    }

    fn visit_call_expr(
        &mut self,
        callee: Box<Expr>,
        _paren: Token,
        arguments: Vec<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        let callee = self.evaluate(*callee)?;

        let mut args: Vec<Rc<Object>> = Vec::new();

        for argument in arguments {
            args.push(self.evaluate(argument)?)
        }

        match &*callee {
            Object::Function(f) => {
                if f.arity() != args.len() {
                    return Err(Error::ArityError {
                        arity: f.arity(),
                        size: args.len(),
                    });
                }
                f.call(self, args)
            }
            Object::Class(klass) => {
                if klass.borrow().arity() != args.len() {
                    return Err(Error::ArityError {
                        arity: klass.borrow().arity(),
                        size: args.len(),
                    });
                }
                klass.borrow().call(self, args)
            }
            _ => Err(Error::NotCallable { obj: callee }),
        }
    }

    fn visit_get_expr(&mut self, object: Box<Expr>, name: Token) -> Result<Rc<Object>, Self::E> {
        let obj = self.evaluate(*object)?;

        match &*obj {
            Object::Instance(inst) => inst.borrow().get(name),
            _ => Err(Error::PropertyAccessError { name }),
        }
    }

    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> Result<Rc<Object>, Error> {
        self.evaluate(*expr)
    }

    fn visit_literal_expr(&mut self, literal: Literal) -> Result<Rc<Object>, Error> {
        match literal {
            Literal::Nil => Ok(Rc::new(Object::Nil)),
            Literal::True => Ok(Rc::new(Object::Bool(true))),
            Literal::False => Ok(Rc::new(Object::Bool(false))),
            Literal::Number(n) => Ok(Rc::new(Object::Number(n))),
            Literal::String(s) => Ok(Rc::new(Object::String(s))),
        }
    }

    fn visit_logical_expr(
        &mut self,
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        let left = self.evaluate(*left)?;

        if op.token_type == TokenType::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else {
            if !left.is_truthy() {
                return Ok(left);
            }
        }

        self.evaluate(*right)
    }

    fn visit_set_expr(
        &mut self,
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        let obj = self.evaluate(*object)?;

        match &*obj {
            Object::Instance(inst) => {
                let val = self.evaluate(*value)?;
                inst.borrow_mut().set(name, val.clone());
                Ok(val)
            }
            _ => Err(Error::FieldAccessError { name }),
        }
    }

    fn visit_super_expr(&mut self, keyword: Token, method: Token) -> Result<Rc<Object>, Self::E> {
        let distance = *self
            .locals
            .get(&keyword)
            .expect("Expect keyword to be in locals.");

        let superclass = self
            .environment
            .borrow()
            .get_at(distance, "super")
            .map_err(|e| Error::EnvironmentError { error: e })?;

        let Object::Class(superclass) = &*superclass else {
            unreachable!()
        };

        let object = self
            .environment
            .borrow()
            .get_at(distance - 1, "this")
            .map_err(|e| Error::EnvironmentError { error: e })?;

        let Object::Instance(object) = &*object else {
            unreachable!()
        };

        let m = superclass.borrow().find_method(&method.lexeme);
        let Some(method) = m else {
            return Err(Error::UndefinedProperty {
                name: method.lexeme,
            });
        };

        Ok(Rc::new(Object::Function(Rc::new(
            method.bind(object.clone()),
        ))))
    }

    fn visit_this_expr(&mut self, keyword: Token) -> Result<Rc<Object>, Self::E> {
        self.look_up_variable(keyword)
    }

    fn visit_variable_expr(&mut self, name: Token) -> Result<Rc<Object>, Self::E> {
        self.look_up_variable(name)
    }

    fn visit_unary_expr(&mut self, op: Token, right: Box<Expr>) -> Result<Rc<Object>, Error> {
        let r = self.evaluate(*right)?;

        match op.token_type {
            TokenType::Minus => Ok(Rc::new(Object::Number(-r.n()?))),
            TokenType::Bang => Ok(Rc::new(Object::Bool(!r.is_truthy()))),
            _ => Err(Error::UnsupportedUnaryOp { op, right: r }),
        }
    }
}

impl StmtVisitor<()> for Interpreter {
    type E = Error;

    fn visit_block_stmt(&mut self, statements: Vec<Stmt>) -> Result<(), Self::E> {
        let reference = (&self.environment).clone();
        self.execute_block(
            Rc::new(statements),
            Rc::new(RefCell::new(Environment::new(Some(reference)))),
        )?;
        Ok(())
    }

    fn visit_class_stmt(
        &mut self,
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    ) -> Result<(), Self::E> {
        let sklass = if let Some(sclass) = superclass {
            let superclass = self.evaluate(sclass)?;
            if let Object::Class(klass) = &*superclass {
                Some(klass.clone())
            } else {
                return Err(Error::SuperClassNotClass { name });
            }
        } else {
            None
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Rc::new(Object::Nil));

        if let Some(superclass) = &sklass {
            let mut environment = Environment::new(Some(self.environment.clone()));
            environment.define(
                "super".to_string(),
                Rc::new(Object::Class(superclass.clone())),
            );
            self.environment = Rc::new(RefCell::new(environment));
        }

        let mut methods_map = HashMap::new();

        for method in methods {
            match method {
                Stmt::Function { name, params, body } => {
                    let function = LoxFunction::new(
                        name.lexeme.clone(),
                        self.environment.clone(),
                        params.into_iter().map(|e| e.lexeme).collect(),
                        Rc::new(body),
                        &name.lexeme == "init",
                    );
                    methods_map.insert(name.lexeme, function);
                }
                _ => return Err(Error::MethodNotFunction { stmt: method }),
            };
        }

        if sklass.is_some() {
            let enclosing = self
                .environment
                .borrow()
                .enclosing
                .clone()
                .expect("Expect enclosig to exist.");
            self.environment = enclosing;
        }

        let klass = Class::new(name.lexeme.clone(), sklass, methods_map);

        if let Err(e) = self
            .environment
            .borrow_mut()
            .assign(name, Rc::new(Object::Class(Rc::new(RefCell::new(klass)))))
        {
            return Err(Error::EnvironmentError { error: e });
        };

        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: Expr) -> Result<(), Error> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Result<(), Self::E> {
        let function = LoxFunction::new(
            name.lexeme.clone(),
            (&self.environment).clone(),
            params.into_iter().map(|t| t.lexeme).collect(),
            Rc::new(body),
            false,
        );

        self.environment
            .borrow_mut()
            .define(name.lexeme, Rc::new(Object::Function(Rc::new(function))));
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    ) -> Result<(), Self::E> {
        if self.evaluate(condition)?.is_truthy() {
            self.execute(*then_branch)?;
        } else if let Some(bexpr) = else_branch {
            self.execute(*bexpr)?;
        }

        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: Expr) -> Result<(), Error> {
        let value = self.evaluate(expr)?;
        println!("{value:?}");
        Ok(())
    }

    fn visit_return_stmt(&mut self, _keyword: Token, value: Option<Expr>) -> Result<(), Self::E> {
        let mut val: Rc<Object> = Rc::new(Object::Nil);

        if let Some(a) = value {
            val = self.evaluate(a)?;
        }

        Err(Error::Return { value: val })
    }

    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Result<(), Self::E> {
        let mut value = Rc::new(Object::Nil);
        if let Some(expr) = initializer {
            value = self.evaluate(expr)?;
        }

        self.environment.borrow_mut().define(name.lexeme, value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Result<(), Self::E> {
        while self.evaluate(condition.clone())?.is_truthy() {
            self.execute(*body.clone())?;
        }

        Ok(())
    }
}
