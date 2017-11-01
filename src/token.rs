use std::fmt;
use std::ops;
use itertools;
use itertools::structs::PutBackN;
use errors::*;


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
    While,
    Loop,
    If,
    Else,
    And,
    Or,
    True,
    False,
    Let,
    Fun,
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


pub fn parse(s: &str) -> Result<TokenStream> {
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


    while let Some(kar) = chars.next() {
        let next = chars.next();
        if let Some(next) = next { chars.put_back(next); }
        use self::TokenKind::*;
        let (kind, lex) = match kar {
            c if c.is_whitespace() => {
                col_no += 1;
                if c == '\n' { line_no += 1; }
                continue
            }
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
            c @ '!' => {
                if match_next(&mut chars, '=') {
                    let mut s = c.to_string();
                    s.push(chars.next().unwrap());
                    (BangEqual, s)
                }
                else { (Bang, c.to_string()) }
            }
            c @ '=' => {
                if match_next(&mut chars, '=') {
                    let mut s = c.to_string();
                    s.push(chars.next().unwrap());
                    (EqualEqual, s)
                }
                else { (Equal, c.to_string()) }
            }
            c @ '<' => {
                if match_next(&mut chars, '=') {
                    let mut s = c.to_string();
                    s.push(chars.next().unwrap());
                    (LessEqual, s)
                }
                else { (Less, c.to_string()) }
            }
            c @ '>' => {
                if match_next(&mut chars, '=') {
                    let mut s = c.to_string();
                    s.push(chars.next().unwrap());
                    (GreaterEqual, s)
                }
                else { (Greater, c.to_string()) }
            }
            c @ '/' => {
                if match_next(&mut chars, '/') {
                    chars.next();                                                   // consume the second comment slash
                    let comment = drain_until_including(&mut chars, |c| c == '\n'); // collect the comment
                    (Comment, comment.trim_right_matches("\n").to_owned())
                } else {
                    (Slash, c.to_string())
                }
            }
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
            d if d.is_digit(10) => {
                let mut s = d.to_string() + &drain_until(&mut chars, |c| !c.is_digit(10));
                // let next = chars.peek().map(|c| c.to_owned());
                // if let Some(next) = next {
                //     if next.is_digit(10) {
                //         s.push_str(&drain_until)
                //     }
                // }
                (Num, s)
            }
            _ => bail_fmt!(ErrorKind::ParseError, "Unexpected character: {:?} at {}, col {}", kar, line_no, col_no),
        };

        let token = Token::new(kind.clone(), &lex, line_no, col_no);
        tokens.push(token);
        match kind {
            TokenKind::Comment => {
                line_no += 1;
                col_no += 1;
            }
            TokenKind::Str => {
                col_no += lex.len() as u32 + 2; // account for 2 double quotes
            }
            _ => {
                col_no += lex.len() as u32;
            }
        }
    }
    Ok(tokens)
}
