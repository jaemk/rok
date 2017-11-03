use std::collections::BTreeMap;
use num;


#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Ident {
    name: String,
}


#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Object {
    id: usize,
    ident: Ident,
    value: Value,
}


#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub enum Value {
    Num(num::rational::BigRational),
    Str(String),
    List(Vec<Value>),
    Map(BTreeMap<Value, Value>),
    Func(Function),
}


#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Expression;


#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Function {
    id: usize,
    ident: Ident,
    args: Vec<Object>,
    body: Expression,
}

