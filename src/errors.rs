
use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);
    }
    errors {
        ParseError(s: String) {
            description("Error encountered while parsing tokens")
            display("ParseError: {}", s)
        }
    }
}

