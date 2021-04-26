use crate::errors::Result;
use crate::value::Value;

pub struct Scope {}

pub fn eval(values: Vec<Value>, scope: &mut Scope) -> Result<Value> {
    todo!()
}
