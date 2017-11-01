#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
extern crate num;
extern crate itertools;

#[macro_use] mod macros;
pub mod errors;
pub mod token;

use std::io::{self, Write, BufRead};

use errors::*;
use token::Token;


pub fn eval(tokens: &[Token]) -> Result<()> {
    unimplemented!()
}


pub fn prompt_line(prompt: &str) -> Result<String> {
    let mut s = String::new();
    print!("{}", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut s)?;
    let line = s.trim_right_matches("\n");
    Ok(line.to_owned())
}

pub fn repl() -> Result<()> {
    loop {
        let line = prompt_line("> ")?;
        let tokens = token::parse(&line)?;
        println!("~ {}", tokens);
    }
    Ok(())
}

