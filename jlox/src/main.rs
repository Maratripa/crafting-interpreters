use std::{
    cell::RefCell,
    env, fs,
    io::{self, Error, Result, Write},
    rc::Rc,
};

pub mod ast;
pub mod class;
pub mod environment;
pub mod functions;
pub mod interpreter;
pub mod object;
pub mod parser;
pub mod resolver;
pub mod scanner;
pub mod token;
pub mod types;

use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;

pub struct Lox {
    interpreter: Rc<RefCell<Interpreter>>,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Rc::new(RefCell::new(Interpreter::new())),
        }
    }

    pub fn run(&mut self, bytes: String) -> std::result::Result<(), parser::Error> {
        let mut scanner = Scanner::new(&bytes);
        let tokens = scanner.scan_tokens();
        // println!("{tokens:?}");
        let mut parser = Parser::new(tokens);

        let statements = parser.parse()?;

        // println!("{statements:?}");

        let mut resolver = Resolver::new(self.interpreter.clone());

        if let Err(e) = resolver.resolve(&statements) {
            eprintln!("{e}");
            return Ok(());
        }

        if let Err(err) = self.interpreter.borrow_mut().interpret(statements) {
            eprintln!("Error: {err}");
        }

        Ok(())
    }

    pub fn run_file(&mut self, path: String) -> Result<()> {
        let bytes = fs::read_to_string(path)?;
        if let Err(_err) = self.run(bytes) {
            eprintln!("{:?}", _err);
            return Err(Error::from_raw_os_error(65));
        }

        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        loop {
            if let Err(err) = self.run(prompt()?) {
                eprintln!("Error: {err}");
            }
        }
    }
}

fn prompt() -> Result<String> {
    let mut line = String::new();
    print!("> ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut line)?;

    Ok(line)
}

fn main() -> Result<()> {
    let mut args = env::args();

    let _program_name = args.next();

    let mut program = Lox::new();

    if let Some(source_path) = args.next() {
        if let Some(_) = args.next() {
            eprintln!("Usage: jlox [script]");
            return Err(Error::from_raw_os_error(64));
        };

        program.run_file(source_path)?;
    } else {
        program.run_prompt()?;
    };

    Ok(())
}
