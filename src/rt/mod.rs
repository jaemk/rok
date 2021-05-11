use crate::errors::Result;
use crate::rt::proc::Processor;
use rustyline::error::ReadlineError;
use std::path;

pub mod proc {
    use crate::asm::Op;
    use crate::errors::Result;

    pub struct Processor {
        registers: [u64; 64],
        program: Vec<u8>,
        pc: usize,
        cond: bool,
        // chan: std::sync::mpsc
    }
    impl Processor {
        pub fn new<P: Into<Vec<u8>>>(program: P) -> Self {
            Self {
                registers: [0; 64],
                program: program.into(),
                pc: 0,
                cond: false,
            }
        }

        fn take_16(buf: &[u8]) -> u16 {
            let data = (buf[0] as u16) << 8;
            data | (buf[1] as u16)
        }

        fn take_32(buf: &[u8]) -> u32 {
            let data = (Self::take_16(buf) as u32) << 16;
            data | (Self::take_16(&buf[2..]) as u32)
        }

        fn take_64(buf: &[u8]) -> u64 {
            let data = (Self::take_32(buf) as u64) << 32;
            data | (Self::take_32(&buf[4..]) as u64)
        }

        fn exec(&mut self, op: Op, args: [u8; 6]) -> Result<()> {
            println!("{:?}: {:?}", op, args);
            use Op::*;
            match op {
                HLT => Err(se!("halt"))?,
                IGL => Err(se!("illegal"))?,
                Reg => println!("Registers\n{:?}", self.registers),
                Set => {
                    let dest = args[0] as usize;
                    let val = (args[1] as u64) << 40
                        | (args[2] as u64) << 32
                        | (args[3] as u64) << 24
                        | (args[4] as u64) << 16
                        | (args[5] as u64);
                    self.registers[dest] = val;
                }
                Add => {
                    let a = args[0] as usize;
                    let b = args[1] as usize;
                    let dest = args[2] as usize;
                    self.registers[dest] = self.registers[a] + self.registers[b];
                }
                Sub => (),
            }
            Ok(())
        }

        pub fn run_code(&mut self, program: &[u8]) -> Result<()> {
            println!("prog: {:?}", program);
            let mut pc = 0;
            loop {
                let buf = &program[pc..];
                if buf.is_empty() {
                    break;
                }
                let two = Self::take_16(&program);
                let op = Op::from(two);
                pc += 2;
                let buf = &program[pc..];
                let args = [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]];
                self.exec(op, args)?;
                pc += 6;
            }
            Ok(())
        }
        pub fn run_to_completion(&mut self) -> Result<()> {
            let prog = self.program.clone();
            self.run_code(&prog)?;
            Ok(())
        }
    }
}

pub struct Runtime {
    procs: Vec<Processor>,
    code: Vec<u8>,
}
impl Runtime {
    pub fn new<P: Into<Vec<u8>>>(program: P) -> Self {
        Self {
            procs: vec![Processor::new(program)],
            code: vec![],
        }
    }
    pub fn run_to_completion(&mut self) -> Result<()> {
        let mut p = &mut self.procs[0];
        p.run_to_completion()
    }
    pub fn run_asm(&mut self, asm: &str) -> Result<()> {
        let asm = crate::asm::parse(&asm)?;
        let code = crate::asm::translate(&asm)?;
        let mut p = &mut self.procs[0];
        p.run_code(&code)
    }
}

pub fn read_eval(s: &str) -> Result<()> {
    let asm = crate::asm::parse(&s)?;
    let code = crate::asm::translate(&asm)?;
    let mut r = crate::rt::Runtime::new(code);
    Ok(r.run_to_completion()?)
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
        let mut runtime = crate::rt::Runtime::new(vec![]);
        loop {
            let line = rl.readline(">>> ");
            match line {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());
                    let res = match runtime.run_asm(&line) {
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
                std::fs::File::create(history_path).ok();
            }
            rl.save_history(history_path).ok();
        }
        Ok(())
    }
}
