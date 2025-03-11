use std::{cell::RefCell, collections::HashMap, rc::Rc};

use thiserror::Error;

use crate::{
    ast::{Expr, ExprVisitor, Literal, Stmt, StmtVisitor},
    interpreter::Interpreter,
    object::Object,
    token::Token,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{expr}: Can't read local variable in its own initializer.")]
    ReadInitializer { expr: Token },

    #[error("{name}: Already a variable with this name in this scope.")]
    DoubleVariable { name: String },

    #[error("{keyword}: Can't return from top-level code.")]
    BadReturn { keyword: Token },

    #[error("{stmt:?}: Method statement is not a function.")]
    MethodStmtNotFunction { stmt: Stmt },

    #[error("{keyword}: Can't use 'this' outside of a class.")]
    ThisOutsideClass { keyword: Token },

    #[error("{keyword}: Can't return a value from an initializer.")]
    ReturnInitializer { keyword: Token },

    #[error("{keyword}: A class can't inherit from itself.")]
    ClassBootstrap { keyword: Token },

    #[error("{keyword}: Can't use 'super' outside of a class.")]
    SuperOutsideClass { keyword: Token },

    #[error("{keyword}: Can't use 'super' in a class with no superclass.")]
    SuperNoSubClass { keyword: Token },
}

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_fn: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_fn: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements.into_iter() {
            self.resolve_stmt(statement)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
        self.execute(stmt.clone())?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: Expr) -> Result<(), Error> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("Popped an empty scopes stack");
    }

    fn declare(&mut self, name: &Token) -> Result<(), Error> {
        if self.scopes.is_empty() {
            return Ok(());
        }

        let scope = self
            .scopes
            .last_mut()
            .expect("Scopes stack is empty when peeking");

        if scope.contains_key(&name.lexeme) {
            return Err(Error::DoubleVariable {
                name: name.lexeme.clone(),
            });
        }

        scope.insert(name.lexeme.to_owned(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self
            .scopes
            .last_mut()
            .expect("Scopes stack is empty when peeking (2)");
        scope.insert(name.lexeme.to_owned(), true);
    }

    fn resolve_local(&mut self, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .borrow_mut()
                    .resolve(name, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: Vec<Token>,
        body: Vec<Stmt>,
        fn_type: FunctionType,
    ) -> Result<(), Error> {
        let enclosing_function = self.current_fn;
        self.current_fn = fn_type;

        self.begin_scope();

        for param in params {
            self.declare(&param)?;
            self.define(&param);
        }

        self.resolve(&body)?;
        self.end_scope();
        self.current_fn = enclosing_function;

        Ok(())
    }
}

impl ExprVisitor<Object> for Resolver {
    type E = Error;

    fn visit_variable_expr(&mut self, name: Token) -> Result<Rc<Object>, Self::E> {
        if !self.scopes.is_empty() && !(self.scopes.last().unwrap().get(&name.lexeme).unwrap()) {
            return Err(Error::ReadInitializer { expr: name });
        }

        self.resolve_local(&name);

        Ok(Rc::new(Object::Nil))
    }

    fn visit_assign_expr(&mut self, name: Token, value: Box<Expr>) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*value)?;
        self.resolve_local(&name);

        Ok(Rc::new(Object::Nil))
    }

    fn visit_binary_expr(
        &mut self,
        left: Box<Expr>,
        _op: Token,
        right: Box<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*left)?;
        self.resolve_expr(*right)?;

        Ok(Rc::new(Object::Nil))
    }

    fn visit_call_expr(
        &mut self,
        callee: Box<Expr>,
        _paren: Token,
        arguments: Vec<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*callee)?;

        for argument in arguments {
            self.resolve_expr(argument)?;
        }

        Ok(Rc::new(Object::Nil))
    }

    fn visit_get_expr(&mut self, object: Box<Expr>, _name: Token) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*object)?;

        Ok(Rc::new(Object::Nil))
    }

    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*expr)?;

        Ok(Rc::new(Object::Nil))
    }

    fn visit_literal_expr(&mut self, _literal: Literal) -> Result<Rc<Object>, Self::E> {
        Ok(Rc::new(Object::Nil))
    }

    fn visit_logical_expr(
        &mut self,
        left: Box<Expr>,
        _op: Token,
        right: Box<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*left)?;
        self.resolve_expr(*right)?;

        Ok(Rc::new(Object::Nil))
    }

    fn visit_set_expr(
        &mut self,
        object: Box<Expr>,
        _name: Token,
        value: Box<Expr>,
    ) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*value)?;
        self.resolve_expr(*object)?;

        Ok(Rc::new(Object::Nil))
    }

    fn visit_super_expr(&mut self, keyword: Token, _method: Token) -> Result<Rc<Object>, Self::E> {
        if self.current_class == ClassType::None {
            return Err(Error::SuperOutsideClass { keyword });
        } else if self.current_class != ClassType::SubClass {
            return Err(Error::SuperNoSubClass { keyword });
        }

        self.resolve_local(&keyword);

        Ok(Rc::new(Object::Nil))
    }

    fn visit_this_expr(&mut self, keyword: Token) -> Result<Rc<Object>, Self::E> {
        if self.current_class == ClassType::None {
            return Err(Error::ThisOutsideClass { keyword });
        }

        self.resolve_local(&keyword);

        Ok(Rc::new(Object::Nil))
    }

    fn visit_unary_expr(&mut self, _op: Token, right: Box<Expr>) -> Result<Rc<Object>, Self::E> {
        self.resolve_expr(*right)?;

        Ok(Rc::new(Object::Nil))
    }
}

impl StmtVisitor<Object> for Resolver {
    type E = Error;

    fn visit_block_stmt(&mut self, statements: Vec<Stmt>) -> Result<Object, Self::E> {
        self.begin_scope();
        self.resolve(&statements)?;
        self.end_scope();

        Ok(Object::Nil)
    }

    fn visit_class_stmt(
        &mut self,
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    ) -> Result<Object, Self::E> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&name)?;
        self.define(&name);

        let there_is_superclass = superclass.is_some();
        if let Some(sclass) = superclass {
            if let Expr::Variable { name: sname } = &sclass {
                if &sname.lexeme == &name.lexeme {
                    return Err(Error::ClassBootstrap { keyword: name });
                }
            }

            self.current_class = ClassType::SubClass;
            self.resolve_expr(sclass)?;

            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("super".to_string(), true);
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_string(), true);

        for method in methods {
            let declaration = if &name.lexeme == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };

            match method {
                Stmt::Function {
                    name: _,
                    params,
                    body,
                } => self.resolve_function(params, body, declaration)?,
                _ => return Err(Error::MethodStmtNotFunction { stmt: method }),
            };
        }

        self.end_scope();

        if there_is_superclass {
            self.end_scope();
        }

        self.current_class = enclosing_class;

        Ok(Object::Nil)
    }

    fn visit_var_stmt(
        &mut self,
        name: Token,
        initializer: Option<Expr>,
    ) -> Result<Object, Self::E> {
        self.declare(&name)?;
        if let Some(init) = initializer {
            self.evaluate(init)?;
        }
        self.define(&name);

        Ok(Object::Nil)
    }

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Result<Object, Self::E> {
        self.declare(&name)?;
        self.define(&name);

        self.resolve_function(params, body, FunctionType::Function)?;

        Ok(Object::Nil)
    }

    fn visit_expression_stmt(&mut self, expr: Expr) -> Result<Object, Self::E> {
        self.resolve_expr(expr)?;

        Ok(Object::Nil)
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    ) -> Result<Object, Self::E> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(&*then_branch)?;

        if let Some(else_part) = else_branch {
            self.resolve_stmt(&*else_part)?;
        }

        Ok(Object::Nil)
    }

    fn visit_print_stmt(&mut self, expr: Expr) -> Result<Object, Self::E> {
        self.resolve_expr(expr)?;

        Ok(Object::Nil)
    }

    fn visit_return_stmt(
        &mut self,
        keyword: Token,
        value: Option<Expr>,
    ) -> Result<Object, Self::E> {
        if self.current_fn == FunctionType::None {
            return Err(Error::BadReturn { keyword });
        }

        if let Some(val) = value {
            if self.current_fn == FunctionType::Initializer {
                return Err(Error::ReturnInitializer { keyword });
            }
            self.resolve_expr(val)?;
        }

        Ok(Object::Nil)
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Result<Object, Self::E> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(&*body)?;

        Ok(Object::Nil)
    }
}
