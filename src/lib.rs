#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
extern crate num;
extern crate itertools;
extern crate rustyline;

#[macro_use] mod macros;
pub mod errors;
pub mod token;

//use std::io::{self, Write}; //BufRead};
use std::path;
use std::fs;
use std::env;
use rustyline::error::ReadlineError;
use errors::*;
use token::Token;


pub fn eval(tokens: &[Token]) -> Result<()> {
    unimplemented!()
}


pub struct Repl {
    save_history: bool,
    history_path: Option<path::PathBuf>,
}
impl Repl {
    pub fn new() -> Self {
        Self {
            save_history: false,
            history_path: None,
        }
    }
    pub fn save_history(&mut self, b: bool) -> &mut Self {
        self.save_history = b;
        let path = env::home_dir().unwrap_or_else(|| path::PathBuf::from("."));
        self.history_path = Some(path.join(".rok_history"));
        self
    }
    pub fn history_path<P: AsRef<path::Path>>(&mut self, path: P) -> &mut Self {
        self.save_history = true;
        self.history_path = Some(path.as_ref().to_owned());
        self
    }
    pub fn run(&self) -> Result<()> {
        let mut rl = rustyline::Editor::<()>::new();
        if let Some(ref history_path) = self.history_path {
            rl.load_history(history_path).ok();
        }
        loop {
            let line = rl.readline(">>> ");
            match line {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());
                    let tokens = &line.parse::<TokenStream>()?;
                    println!("{}", tokens);
                }
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                    break;
                }
                Err(e) => bail!(e),
            }
        }
        if let Some(ref history_path) = self.history_path {
            if !history_path.exists() {
                fs::File::create(history_path).ok();
            }
            rl.save_history(history_path).ok();
        }
        Ok(())
    }
}

}

pub fn repl() -> Result<()> {
    loop {
        let line = prompt_line("> ")?;
        let tokens = token::parse(&line)?;
        println!("~ {}", tokens);
    }
    Ok(())
}

