use crate::{
    ast::{Expr, Literal, Stmt},
    token::{
        Token,
        TokenType::{self, *},
    },
};
use std::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{msg} at {token}")]
    Bad { token: Token, msg: String },

    #[error("Invalid assignment target {token}.")]
    InvalidAssignment { token: Token },

    #[error("Maximum limit of arguments achieved.")]
    MaxArgs,
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn variant_eq(a: &TokenType, b: &TokenType) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let res = if self.check(&Class) {
            self.advance();
            self.class_declaration()
        } else if self.check(&Fun) {
            self.advance();
            self.function("function")
        } else if self.check(&Var) {
            self.advance();
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Ok(stmt) => return Ok(stmt),
            Err(err) => {
                self.synchronize();
                return Err(err);
            }
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(Identifier, "Expect class name.")?;

        let superclass = if self.check(&Greater) {
            self.advance();
            Some(Expr::Variable {
                name: self.consume(Identifier, "Expect superclass name.")?,
            })
        } else {
            None
        };

        self.consume(LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            superclass,
            methods,
        })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.check(&For) {
            self.advance();
            return self.for_statement();
        }

        if self.check(&If) {
            self.advance();
            return self.if_statement();
        }

        if self.check(&Print) {
            // println!("1) There is a print!");
            self.advance();
            return self.print_statement();
        }

        if self.check(&Return) {
            self.advance();
            return self.return_statement();
        }

        if self.check(&While) {
            self.advance();
            return self.while_statement();
        }

        if self.check(&LeftBrace) {
            self.advance();
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(LeftParen, "Expect '(' after 'for'.")?;

        let initializer: Option<Stmt>;
        if self.check(&Semicolon) {
            self.advance();
            initializer = None;
        } else if self.check(&Var) {
            self.advance();
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Expr> = None;
        if !self.check(&Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(Semicolon, "Expect ';' after loop condition.")?;

        let mut increment: Option<Expr> = None;
        if !self.check(&RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(expr) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression { expr }],
            };
        }

        let condition = condition.unwrap_or(Expr::Literal(Literal::True));
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![init, body],
            };
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(LeftParen, "Expect '(' after 'if'.")?;
        let condition: Expr = self.expression()?;
        self.consume(RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.check(&Else) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        // println!("2) Value is: {value:?}");
        self.consume(Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expr: value })
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous().clone();
        let mut value: Option<Expr> = None;

        if !self.check(&Semicolon) {
            value = Some(self.expression()?);
        }

        self.consume(Semicolon, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(Identifier, "Expect variable name.")?;

        let initializer = if self.check(&Equal) {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var { name, initializer })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expr })
    }

    fn function(&mut self, kind: &str) -> Result<Stmt> {
        let name = self.consume(Identifier, &format!("Expect {kind} name."))?;
        self.consume(LeftParen, &format!("Expect '(' after {kind} name."))?;

        let mut parameters: Vec<Token> = Vec::new();

        if !self.check(&RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(Error::MaxArgs);
                }

                parameters.push(self.consume(Identifier, "Expect parameter name.")?);

                if !self.check(&Comma) {
                    break;
                } else {
                    self.advance();
                }
            }
        }

        self.consume(RightParen, "Expect ')' after parameters.")?;
        self.consume(LeftBrace, &format!("Expect '{{' before {kind} body."))?;

        let body = self.block()?;
        return Ok(Stmt::Function {
            name,
            params: parameters,
            body,
        });
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.check(&Equal) {
            let equals = self.advance().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                Expr::Get { object, name } => {
                    return Ok(Expr::Set {
                        object,
                        name,
                        value: Box::new(value),
                    })
                }
                _ => return Err(Error::InvalidAssignment { token: equals }),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.check(&Or) {
            let op = self.advance().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.check(&And) {
            let op = self.advance().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        // println!("3) Expression: {expr:?}");

        while self.eval_tokens(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn eval_tokens(&mut self, types: &[TokenType]) -> bool {
        for ty in types.iter() {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, ty: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        variant_eq(&self.peek().token_type, ty)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == EOF
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        // println!("4) Expression: {expr:?}");

        while self.eval_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        // println!("5) Expr: {expr:?}");

        while self.eval_tokens(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        // println!("6) Expr: {expr:?}");

        while self.eval_tokens(&[Slash, Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.eval_tokens(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op: operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments: Vec<Expr> = Vec::new();

        if !self.check(&RightParen) {
            arguments.push(self.expression()?);
            while self.eval_tokens(&[Comma]) {
                if arguments.len() >= 255 {
                    return Err(Error::MaxArgs);
                }
                arguments.push(self.expression()?);
            }
        }

        let paren = self.consume(RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.eval_tokens(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.eval_tokens(&[Dot]) {
                let name = self.consume(Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
        // println!("Token from self.advance(): {:?}", self.advance());
        self.advance();

        let prev = self.previous();

        match prev.token_type {
            False => Ok(Expr::Literal(Literal::False)),
            True => Ok(Expr::Literal(Literal::True)),
            Nil => Ok(Expr::Literal(Literal::Nil)),
            Super => {
                let keyword = prev.clone();
                self.consume(Dot, "Expect '.' after 'super'.")?;
                let method = self.consume(Identifier, "Expect superclass method name.")?;
                Ok(Expr::Super { keyword, method })
            }
            This => Ok(Expr::This {
                keyword: prev.clone(),
            }),
            Number => {
                let n = prev
                    .clone()
                    .literal
                    .expect("Number token does not have a literal");
                Ok(Expr::Literal(n))
            }
            String => {
                let s = prev
                    .clone()
                    .literal
                    .expect("String token does not have a literal");
                Ok(Expr::Literal(s))
            }
            Identifier => Ok(Expr::Variable { name: prev.clone() }),
            LeftParen => {
                let expr = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping { ex: Box::new(expr) });
            }
            other => {
                println!("Token found that is wrong: {other}");
                return Err(Error::Bad {
                    token: self.peek().clone(),
                    msg: "Expect expression.".to_owned(),
                });
            }
        }
    }

    fn consume(&mut self, ty: TokenType, message: &str) -> Result<Token> {
        if self.check(&ty) {
            return Ok(self.advance().clone());
        }

        Err(Error::Bad {
            token: self.peek().clone(),
            msg: message.to_owned(),
        })
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }

            match self.peek().token_type {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}
