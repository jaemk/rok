pub mod token;
pub mod value;

use crate::errors::Result;
use rustyline::error::ReadlineError;
use std::{fs, path};
use value::Value;

pub struct Scope {}

pub fn read_eval(s: &str, scope: &mut Scope) -> Result<Value> {
    // lex to tokens
    let tokens = token::lex(s)?;
    // println!("tokens: {}", tokens);
    // parse to forms
    let values = value::parse_file(tokens)?;
    // todo: eval
    Ok(Value::List(values))
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
        let path = home::home_dir().unwrap_or_else(|| path::PathBuf::from("."));
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
        let mut scope = Scope {};
        loop {
            let line = rl.readline(">>> ");
            match line {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());
                    let res = match read_eval(&line, &mut scope) {
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                        Ok(t) => t,
                    };
                    println!("{:?}", res);
                }
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                    break;
                }
                Err(e) => return Err(se!("unknown: {}", e).into()),
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

trait RockAlphabetic {
    fn is_rok_alphabetic(&self) -> bool;
}

impl RockAlphabetic for char {
    fn is_rok_alphabetic(&self) -> bool {
        self.is_alphabetic() || *self == '.' || *self == '-' || *self == '_' || *self == '?'
    }
}
