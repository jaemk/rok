#![recursion_limit = "1024"]

extern crate rok;
#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::fs;
use std::io::{self, Read};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn run() -> Result<()> {
    let matches = App::new("rok")
        .version(crate_version!())
        .about("rok!")
        .help("Run a file, string, or start an interpreter")
        .arg(
            Arg::with_name("file")
                .help("file to evaluate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("evaluate")
                .help("string to evaluate")
                .long("eval")
                .short("e")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    let src = if let Some(file) = matches.value_of("file") {
        println!("evaluating file: {}", file);
        let mut s = String::new();
        let mut f = fs::File::open(&file)?;
        f.read_to_string(&mut s)?;
        Some(s)
    } else if let Some(string) = matches.value_of("evaluate") {
        println!("evaluating str: {}", string);
        Some(string.to_owned())
    } else {
        None
    };

    if let Some(_src) = src {
        //let tokens = rok::parse(&src)?;
        //let res = rok::eval(&tokens)?;
        //println!("{:?}", res);
    } else {
        println!("Rok {}", crate_version!());
        rok::Repl::new().save_history(true).run()?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
