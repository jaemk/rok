use crate::errors::Result;
use std::process::id;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Op {
    HLT,
    IGL,
    Add,
    Sub,
    Set,
    Reg,
}
impl Op {
    fn to_parts(&self) -> [u8; 2] {
        match *self {
            Op::HLT => [0, 0],
            Op::Add => [0, 1],
            Op::Sub => [0, 2],
            Op::Set => [0, 3],
            Op::Reg => [0, 100],
            _ => [0, 99],
        }
    }
}
impl From<u16> for Op {
    fn from(v: u16) -> Self {
        match v {
            0 => Op::HLT,
            1 => Op::Add,
            2 => Op::Sub,
            3 => Op::Set,
            100 => Op::Reg,
            _ => Op::IGL,
        }
    }
}
impl<'a> From<&'a str> for Op {
    fn from(v: &'a str) -> Self {
        match v.to_uppercase().as_str() {
            "HLT" => Op::HLT,
            "ADD" => Op::Add,
            "SUB" => Op::Sub,
            "SET" => Op::Set,
            "REG" => Op::Reg,
            _ => Op::IGL,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Operation {
    tag: Option<String>,
    code: Op,
    args: Vec<String>,
}

pub fn parse(s: &str) -> Result<Vec<Operation>> {
    let mut ops = vec![];
    for line in s.trim().lines() {
        let idents = line
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if idents.is_empty() {
            continue;
        }
        let op = Op::from(idents[0].as_str());
        let args = if idents.len() > 1 {
            idents[1..].to_vec()
        } else {
            vec![]
        };
        ops.push(Operation {
            tag: None,
            code: op,
            args,
        });
    }
    Ok(ops)
}

fn pack(program: &mut Vec<u8>, new: &[u8], count: usize) {
    const ZERO: [u8; 8] = [0; 8];
    let remainder = 8 - count;
    program.extend_from_slice(&new[0..count]);
    program.extend_from_slice(&ZERO[0..remainder]);
}

pub fn translate(ops: &[Operation]) -> Result<Vec<u8>> {
    let mut prog = vec![];
    let mut buf = vec![0; 8];
    for operation in ops {
        let code = &operation.code;
        let code_buf = code.to_parts();
        match code {
            Op::HLT => {
                buf[0] = code_buf[0];
                buf[1] = code_buf[1];
                pack(&mut prog, &buf, 2);
            }
            Op::Reg => {
                buf[0] = code_buf[0];
                buf[1] = code_buf[1];
                pack(&mut prog, &buf, 2);
            }
            Op::Set => {
                buf[0] = code_buf[0];
                buf[1] = code_buf[1];
                let dest = operation.args[0].parse::<u8>()?;
                let val = operation.args[1].parse::<u32>()?;
                buf[2] = dest;
                buf[4] = (val >> 24) as u8;
                buf[5] = (val >> 16) as u8;
                buf[6] = (val >> 8) as u8;
                buf[7] = val as u8;
                pack(&mut prog, &buf, 8);
            }
            Op::Add => {
                buf[0] = code_buf[0];
                buf[1] = code_buf[1];
                let a = operation.args[0].parse::<u8>()?;
                let b = operation.args[1].parse::<u8>()?;
                let c = operation.args[2].parse::<u8>()?;
                buf[2] = a;
                buf[3] = b;
                buf[4] = c;
                pack(&mut prog, &buf, 5);
            }
            Op::Sub => {
                buf[0] = code_buf[0];
                buf[1] = code_buf[1];
                let a = operation.args[0].parse::<u8>()?;
                let b = operation.args[1].parse::<u8>()?;
                let c = operation.args[2].parse::<u8>()?;
                buf[2] = a;
                buf[3] = b;
                buf[4] = c;
                pack(&mut prog, &buf, 5);
            }
            _ => {
                buf[0] = code_buf[0];
                buf[1] = code_buf[1];
                pack(&mut prog, &buf, 2);
            }
        };
    }
    Ok(prog)
}
