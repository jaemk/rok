use std::fmt;
use std::ops;
use std::str;
use itertools;
use itertools::structs::PutBackN;
use {RockAlphabetic};
use errors::*;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    //literal: ??
    pub source_line: u32,
    pub source_column: u32,
}
impl Token {
    pub fn new(kind: TokenKind, lexeme: &str, source_line: u32, source_column: u32) -> Self {
        Self {
            kind,
            lexeme: lexeme.to_owned(),
            source_line,
            source_column,
        }
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{:?}: {:?} at (l:{}, c:{})>", self.kind, self.lexeme, self.source_line, self.source_column)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // -- syntax --
    LeftParen,
    RightParen,

    LeftBrace,
    RightBrace,

    LeftBracket,
    RightBracket,
    Map,

    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Colon,
    SemiColon,
    Slash,

    Bang,
    Equal,
    Greater,
    Less,
    BangEqual,
    EqualEqual,
    GreaterEqual,
    LessEqual,

    // -- keywords --
    For,
    In,
    While,
    Loop,
    If,
    Else,
    And,
    Or,
    True,
    False,
    Let,
    Func,
    Nil,
    Return,

    // -- literals --
    Ident,
    Str,
    Num,

    // -- Misc --
    Comment,
    EndOfFile
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream(Vec<Token>);
impl TokenStream {
    pub fn new() -> Self {
        TokenStream(vec![])
    }
}
impl ops::Deref for TokenStream {
    type Target = Vec<Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for TokenStream {
    fn deref_mut(&mut self) -> &mut Vec<Token> {
        &mut self.0
    }
}
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let size = self.0.len();
        for (i, token) in self.0.iter().enumerate() {
            write!(f, "{}", token)?;
            if i < size - 1 {
                write!(f, ",\n")?;
            }
        }
        write!(f, "]")
    }
}
impl str::FromStr for TokenStream {
    type Err = Error;

    fn from_str(s: &str) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();
        let mut chars = itertools::put_back_n(s.chars());
        let mut line_no = 1;
        let mut col_no = 1;

        fn match_next<T>(source: &mut PutBackN<T>, want: char) -> bool
            where T: Iterator<Item=char>
        {
            let next = source.next();
            if let Some(next) = next {
                source.put_back(next);
            }
            next.map(|c| c == want).unwrap_or(false)
        }

        fn get_next<T>(source: &mut PutBackN<T>) -> Option<char>
            where T: Iterator<Item=char>
        {
            let next = source.next();
            if let Some(next) = next {
                source.put_back(next);
            }
            next
        }

        fn drain_until<T, F>(source: &mut PutBackN<T>, func: F) -> String
            where T: Iterator<Item=char>,
                  F: Fn(char) -> bool
        {
            let mut s = String::new();
            while let Some(next) = source.next() {
                if func(next) {
                    source.put_back(next);
                    return s
                }
                s.push(next);
            }
            s
        }

        fn drain_until_including<T, F>(source: &mut T, func: F) -> String
            where T: Iterator<Item=char>,
                  F: Fn(char) -> bool
        {
            let mut s = String::new();
            while let Some(next) = source.next() {
                s.push(next);
                if func(next) { break; }
            }
            s
        }

        fn drain_to_required_including<T, F>(source: &mut T, func: F) -> Result<String>
            where T: Iterator<Item=char>,
                  F: Fn(char) -> bool
        {
            let mut s = String::new();
            while let Some(next) = source.next() {
                s.push(next);
                if func(next) { return Ok(s) }
            }
            bail!("Unterminated")
        }

        /// Parse the tail (fractional part) of a digit.
        ///
        /// This will return the tail chars (including the dot) if the tail is a valid
        /// sequence of digits. In the case of a trailing dot (e.g. "1234."), "." will be returned.
        /// If the tail is a sequence of alphabetic chars, then an empty tail will be returned
        /// and the dot should be treated as a dot operator.
        ///
        /// A ParseError will be returned if the tail has an invalid sequence of digits
        /// e.g. "1234.12a"
        ///
        /// E.g. we currently have "1234" with a `PutBackN` containing ['.', char, char, ...]
        ///      valid numbers can have a trailing dot
        fn get_digit_tail<T>(mut source: &mut PutBackN<T>) -> Result<String>
            where T: Iterator<Item=char>
        {
            let mut s = String::new();
            let next1 = source.next();  // the dot
            let next2 = source.next();  // char after the dot
            if let Some(next2) = next2 {
                source.put_back(next2);

                if next2.is_whitespace() {
                    s.push('.');
                } else if next2.is_rok_alphabetic() {
                    source.put_back(next1.unwrap());
                } else if next2.is_digit(10) {
                    s.push(next1.unwrap()); // push the dot
                    let digit_tail = drain_until(&mut source, |c| !c.is_digit(10));
                    let next = get_next(&mut source).unwrap_or(' ');
                    if next.is_rok_alphabetic() {
                        bail_fmt!(ErrorKind::ParseError, "Unexpected character: {:?}. Found alphabetic trailing a digit", next)
                    }
                    s.push_str(&digit_tail);
                }
            } else {
                s.push('.');
            }
            Ok(s)
        }

        while let Some(kar) = chars.next() {
            use self::TokenKind::*;
            let (kind, lex) = match kar {
                // ignore whitespace
                c if c.is_whitespace() => {
                    if c == '\n' {
                        line_no += 1;
                        col_no = 1;   // reset column
                    } else {
                        col_no += 1;
                    }
                    continue
                }

                // handle signle character tokens
                c @ '(' => (LeftParen,      c.to_string()),
                c @ ')' => (RightParen,     c.to_string()),
                c @ '[' => (LeftBrace,      c.to_string()),
                c @ ']' => (RightBrace,     c.to_string()),
                c @ '{' => (LeftBracket,    c.to_string()),
                c @ '}' => (RightBracket,   c.to_string()),
                c @ ',' => (Comma,          c.to_string()),
                c @ '.' => (Dot,            c.to_string()),
                c @ '+' => (Plus,           c.to_string()),
                c @ '*' => (Star,           c.to_string()),
                c @ ':' => (Colon,          c.to_string()),
                c @ ';' => (SemiColon,      c.to_string()),

                // handle the possibly double character tokens
                c @ '!' => {
                    if match_next(&mut chars, '=') {
                        let mut s = c.to_string();
                        s.push(chars.next().unwrap());
                        (BangEqual, s)
                    } else { (Bang, c.to_string()) }
                }
                c @ '=' => {
                    if match_next(&mut chars, '=') {
                        let mut s = c.to_string();
                        s.push(chars.next().unwrap());
                        (EqualEqual, s)
                    } else { (Equal, c.to_string()) }
                }
                c @ '<' => {
                    if match_next(&mut chars, '=') {
                        let mut s = c.to_string();
                        s.push(chars.next().unwrap());
                        (LessEqual, s)
                    } else { (Less, c.to_string()) }
                }
                c @ '>' => {
                    if match_next(&mut chars, '=') {
                        let mut s = c.to_string();
                        s.push(chars.next().unwrap());
                        (GreaterEqual, s)
                    } else { (Greater, c.to_string()) }
                }

                // hashmap literal
                c @ '#' => {
                    if match_next(&mut chars, '{') {
                        let mut s = c.to_string();
                        s.push(chars.next().unwrap());
                        (Map, s)
                    } else {
                        let s = c.to_string() + &drain_until(&mut chars, |c| !c.is_rok_alphabetic());
                        (Ident, s)
                    }
                }

                // handle comments (or slashes)
                c @ '/' => {
                    if match_next(&mut chars, '/') {
                        chars.next();                                                   // consume the second comment slash
                        let comment = drain_until_including(&mut chars, |c| c == '\n'); // collect the comment
                        (Comment, comment.trim_right_matches("\n").to_owned())
                    } else {
                        (Slash, c.to_string())
                    }
                }

                // handle string literals
                '"' => {
                    let s = drain_to_required_including(&mut chars, |c| c == '"')
                        .map_err(|_| format_err!(ErrorKind::ParseError, "Unterminated string at line {}, col {}", line_no, col_no))?;
                    (Str, s.trim_right_matches("\"").to_owned())
                }
                '\'' => {
                    let s = drain_to_required_including(&mut chars, |c| c == '\'')
                        .map_err(|_| format_err!(ErrorKind::ParseError, "Unterminated string at line {}, col {}", line_no, col_no))?;
                    (Str, s.trim_right_matches("\'").to_owned())
                }

                // handle numbers (trailing dot allowed)
                d if d.is_digit(10) => {
                    let mut s = d.to_string() + &drain_until(&mut chars, |c| !c.is_digit(10));
                    let next = get_next(&mut chars).unwrap_or(' ');
                    if next == '.' {
                        let tail = get_digit_tail(&mut chars)?;
                        s.push_str(&tail);
                    } else if next.is_rok_alphabetic() {
                        bail_fmt!(ErrorKind::ParseError,
                                  "Unexpected character: {:?} at line {}, col {}. Found alphabetic trailing a digit",
                                  next, line_no, col_no)
                    }
                    (Num, s)
                }

                // handle keywords
                c => {
                    let s = c.to_string() + &drain_until(&mut chars, |c| !c.is_rok_alphabetic());
                    match s.as_str() {
                        "for"       => (For, s),
                        "in"        => (In, s),
                        "while"     => (While, s),
                        "loop"      => (Loop, s),
                        "if"        => (If, s),
                        "else"      => (Else, s),
                        "and"       => (And, s),
                        "or"        => (Or, s),
                        "true"      => (True, s),
                        "false"     => (False, s),
                        "let"       => (Let, s),
                        "fn"        => (Func, s),
                        "nil"       => (Nil, s),
                        "return"    => (Return, s),
                        _ => (Ident, s),
                    }
                }
            };

            let token = Token::new(kind.clone(), &lex, line_no, col_no);
            tokens.push(token);
            match kind {
                // account for double char token
                EqualEqual | BangEqual | LessEqual | GreaterEqual => col_no += 2,

                // comments are the end of lines
                Comment => {
                    line_no += 1;
                    col_no += 1;
                }

                // account for surrounding qoutes
                Str => {
                    col_no += lex.len() as u32 + 2; // account for 2 double quotes
                }

                _ => {
                    col_no += lex.len() as u32;
                }
            }
        }
        tokens.push(Token::new(TokenKind::EndOfFile, "", line_no, col_no));
        Ok(tokens)
    }
}

