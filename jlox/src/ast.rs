use std::fmt::Display;
use std::rc::Rc;

use crate::{token::Token, types::Number};

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping {
        ex: Box<Expr>,
    },
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

#[derive(PartialEq, Clone, Debug)]
pub enum Literal {
    Number(Number),
    String(String),
    True,
    False,
    Nil,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Assign { name, value } => f.write_fmt(format_args!("{name} = {value}")),
            Self::Binary { left, op, right } => {
                f.write_fmt(format_args!("({} {} {})", left, op, right))
            }
            Self::Call {
                callee,
                paren: _,
                arguments,
            } => f.write_fmt(format_args!("{callee}({arguments:?})")),
            Self::Get { object, name } => f.write_fmt(format_args!("{object}.{name}")),
            Self::Grouping { ex } => f.write_fmt(format_args!("({})", ex)),
            Self::Literal(Literal::Number(n)) => n.fmt(f),
            Self::Literal(Literal::String(s)) => s.fmt(f),
            Self::Literal(Literal::True) => true.fmt(f),
            Self::Literal(Literal::False) => false.fmt(f),
            Self::Literal(Literal::Nil) => f.write_str("nil"),
            Self::Logical { left, op, right } => {
                f.write_fmt(format_args!("({} {} {})", left, op, right))
            }
            Self::Set {
                object,
                name,
                value,
            } => f.write_fmt(format_args!("{}.{} = {}", object, name, value)),
            Self::Super { keyword, method } => f.write_fmt(format_args!("{keyword}.{method}")),
            Self::This { keyword: _ } => f.write_str("this"),
            Self::Unary { op, right } => f.write_fmt(format_args!("({}{})", op, right)),
            Self::Variable { name } => f.write_fmt(format_args!("{}", name)),
        }
    }
}

pub trait ExprVisitor<T> {
    type E;

    fn evaluate(&mut self, expr: Expr) -> Result<Rc<T>, Self::E> {
        match expr {
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            Expr::Binary { left, op, right } => self.visit_binary_expr(left, op, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.visit_call_expr(callee, paren, arguments),
            Expr::Get { object, name } => self.visit_get_expr(object, name),
            Expr::Grouping { ex } => self.visit_grouping_expr(ex),
            Expr::Literal(literal) => self.visit_literal_expr(literal),
            Expr::Logical { left, op, right } => self.visit_logical_expr(left, op, right),
            Expr::Set {
                object,
                name,
                value,
            } => self.visit_set_expr(object, name, value),
            Expr::Super { keyword, method } => self.visit_super_expr(keyword, method),
            Expr::This { keyword } => self.visit_this_expr(keyword),
            Expr::Unary { op, right } => self.visit_unary_expr(op, right),
            Expr::Variable { name } => self.visit_variable_expr(name),
        }
    }

    fn visit_assign_expr(&mut self, name: Token, value: Box<Expr>) -> Result<Rc<T>, Self::E>;
    fn visit_binary_expr(
        &mut self,
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    ) -> Result<Rc<T>, Self::E>;
    fn visit_call_expr(
        &mut self,
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    ) -> Result<Rc<T>, Self::E>;
    fn visit_get_expr(&mut self, object: Box<Expr>, name: Token) -> Result<Rc<T>, Self::E>;
    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> Result<Rc<T>, Self::E>;
    fn visit_literal_expr(&mut self, literal: Literal) -> Result<Rc<T>, Self::E>;
    fn visit_logical_expr(
        &mut self,
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    ) -> Result<Rc<T>, Self::E>;
    fn visit_set_expr(
        &mut self,
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    ) -> Result<Rc<T>, Self::E>;
    fn visit_super_expr(&mut self, keyword: Token, method: Token) -> Result<Rc<T>, Self::E>;
    fn visit_this_expr(&mut self, keyword: Token) -> Result<Rc<T>, Self::E>;
    fn visit_unary_expr(&mut self, op: Token, right: Box<Expr>) -> Result<Rc<T>, Self::E>;
    fn visit_variable_expr(&mut self, name: Token) -> Result<Rc<T>, Self::E>;
}

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    },
    Expression {
        expr: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expr: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

pub trait StmtVisitor<T> {
    type E;

    fn execute(&mut self, stmt: Stmt) -> Result<T, Self::E> {
        match stmt {
            Stmt::Block { statements } => self.visit_block_stmt(statements),
            Stmt::Class {
                name,
                superclass,
                methods,
            } => self.visit_class_stmt(name, superclass, methods),
            Stmt::Expression { expr } => self.visit_expression_stmt(expr),
            Stmt::Function { name, params, body } => self.visit_function_stmt(name, params, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print { expr } => self.visit_print_stmt(expr),
            Stmt::Return { keyword, value } => self.visit_return_stmt(keyword, value),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
        }
    }

    fn visit_block_stmt(&mut self, statements: Vec<Stmt>) -> Result<T, Self::E>;
    fn visit_class_stmt(
        &mut self,
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    ) -> Result<T, Self::E>;
    fn visit_expression_stmt(&mut self, expr: Expr) -> Result<T, Self::E>;
    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Result<T, Self::E>;
    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    ) -> Result<T, Self::E>;
    fn visit_print_stmt(&mut self, expr: Expr) -> Result<T, Self::E>;
    fn visit_return_stmt(&mut self, keyword: Token, value: Option<Expr>) -> Result<T, Self::E>;
    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Result<T, Self::E>;
    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Result<T, Self::E>;
}
