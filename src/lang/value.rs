use crate::errors::Result;
use crate::lang::token::{Token, TokenKind, TokenStream};
use num;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Ident {
    name: String,
}
impl Ident {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Clone, Hash, PartialOrd, PartialEq)]
pub struct List {
    inner: Vec<Value>,
}
impl std::ops::Deref for List {
    type Target = Vec<Value>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::ops::DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
impl List {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub enum Value {
    Num(num::rational::BigRational),
    Str(String),
    List(List),
    Map(BTreeMap<Value, Value>),
    Func(Function),
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Function {
    id: usize,
    ident: Option<Ident>,
    full: List,
    args: List,
    body: List,
}

fn parse_list(tokens: &[Token]) -> Result<List> {
    println!("list tokens: {}", TokenStream(tokens.to_vec()));
    Ok(List::new())
}

fn find_list_end(tokens: &[Token]) -> Result<usize> {
    if tokens[0].kind == TokenKind::EndOfFile {
        return Ok(1);
    }
    assert_eq!(tokens[0].kind, TokenKind::LeftParen);
    let mut i = 1;
    let mut open_count = 0;
    loop {
        println!("** NEXT: {}", tokens[i]);
        if open_count == 0 && tokens[i].kind == TokenKind::RightParen {
            i += 1;
            break;
        }

        if tokens[i].kind == TokenKind::LeftParen {
            open_count += 1;
        } else if tokens[i].kind == TokenKind::RightParen {
            open_count -= 1
        }

        i += 1;
    }
    return Ok(i);
}

pub fn parse_file(tokens: TokenStream) -> Result<List> {
    let mut lists = List::new();
    let mut i = 0;
    loop {
        let start = i;
        // println!("parsing from {}", tokens[start]);
        let n = find_list_end(&tokens[i..])?;
        i = start + n;
        if i >= tokens.len() {
            break;
        }
        let res = parse_list(&tokens[start..i])?;
        lists.push(Value::List(res));
    }
    Ok(lists)
}
