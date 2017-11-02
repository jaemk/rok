
use std::io;
use rustyline;

error_chain! {
    foreign_links {
        Io(io::Error);
        ReadLine(rustyline::error::ReadlineError);
    }
    errors {
        ParseError(s: String) {
            description("Error encountered while parsing tokens")
            display("ParseError: {}", s)
        }
    }
}

