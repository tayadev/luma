use crate::bytecode::ir::{Chunk, Instruction, Constant};
use super::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub enum VmError {
    Runtime(String),
}

pub struct VM {
    stack: Vec<Value>,
    ip: usize,
    chunk: Chunk,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        VM { stack: Vec::new(), ip: 0, chunk, globals: HashMap::new() }
    }

    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            if self.ip >= self.chunk.instructions.len() {
                return Err(VmError::Runtime("IP out of bounds".into()));
            }
            let instr = self.chunk.instructions[self.ip].clone();
            self.ip += 1;
            match instr {
                Instruction::Const(idx) => {
                    let v = match self.chunk.constants.get(idx) {
                        Some(Constant::Number(n)) => Value::Number(*n),
                        Some(Constant::String(s)) => Value::String(s.clone()),
                        Some(Constant::Boolean(b)) => Value::Boolean(*b),
                        Some(Constant::Null) => Value::Null,
                        None => return Err(VmError::Runtime("Bad const index".into())),
                    };
                    self.stack.push(v);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Dup => {
                    if let Some(v) = self.stack.last().cloned() {
                        self.stack.push(v);
                    } else {
                        return Err(VmError::Runtime("DUP on empty stack".into()));
                    }
                }
                Instruction::Add => {
                    let (b, a) = (self.stack.pop(), self.stack.pop());
                    match (a, b) {
                        (Some(Value::Number(x)), Some(Value::Number(y))) => self.stack.push(Value::Number(x + y)),
                        (Some(Value::String(x)), Some(Value::String(y))) => self.stack.push(Value::String(x + &y)),
                        (Some(Value::String(x)), Some(v)) => self.stack.push(Value::String(format!("{}{:?}", x, v))),
                        (Some(v), Some(Value::String(y))) => self.stack.push(Value::String(format!("{:?}{}", v, y))),
                        _ => return Err(VmError::Runtime("ADD type error".into())),
                    }
                }
                Instruction::Sub => bin_num(&mut self.stack, |a,b| a-b)?,
                Instruction::Mul => bin_num(&mut self.stack, |a,b| a*b)?,
                Instruction::Div => bin_num(&mut self.stack, |a,b| a/b)?,
                Instruction::Mod => bin_num(&mut self.stack, |a,b| a%b)?,
                Instruction::GetGlobal(idx) => {
                    let name = match self.chunk.constants.get(idx) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => return Err(VmError::Runtime("GET_GLOBAL expects string constant".into())),
                    };
                    if let Some(v) = self.globals.get(&name).cloned() {
                        self.stack.push(v);
                    } else {
                        return Err(VmError::Runtime(format!("Undefined global '{}'", name)));
                    }
                }
                Instruction::SetGlobal(idx) => {
                    let name = match self.chunk.constants.get(idx) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => return Err(VmError::Runtime("SET_GLOBAL expects string constant".into())),
                    };
                    let v = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_GLOBAL pop underflow".into()))?;
                    self.globals.insert(name, v);
                }
                Instruction::Eq => bin_eq(&mut self.stack)?,
                Instruction::Ne => { bin_eq(&mut self.stack)?; flip_bool(&mut self.stack)?; }
                Instruction::Lt => bin_cmp(&mut self.stack, |a,b| a<b)?,
                Instruction::Le => bin_cmp(&mut self.stack, |a,b| a<=b)?,
                Instruction::Gt => bin_cmp(&mut self.stack, |a,b| a>b)?,
                Instruction::Ge => bin_cmp(&mut self.stack, |a,b| a>=b)?,
                Instruction::Not => {
                    let v = self.stack.pop().ok_or_else(|| VmError::Runtime("NOT on empty stack".into()))?;
                    self.stack.push(Value::Boolean(!truthy(&v)));
                }
                Instruction::Jump(target) => {
                    self.ip = target;
                }
                Instruction::JumpIfFalse(target) => {
                    let v = self.stack.pop().ok_or_else(|| VmError::Runtime("JUMP_IF_FALSE pop underflow".into()))?;
                    if !truthy(&v) { self.ip = target; }
                }
                Instruction::Halt => {
                    return Ok(self.stack.pop().unwrap_or(Value::Null));
                }
            }
        }
    }
}

fn bin_num<F>(stack: &mut Vec<Value>, f: F) -> Result<(), VmError>
where F: FnOnce(f64,f64)->f64 {
    let (b, a) = (stack.pop(), stack.pop());
    match (a, b) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => { stack.push(Value::Number(f(x,y))); Ok(()) }
        _ => Err(VmError::Runtime("Numeric op type error".into())),
    }
}

fn bin_eq(stack: &mut Vec<Value>) -> Result<(), VmError> {
    let (b, a) = (stack.pop(), stack.pop());
    match (a, b) {
        (Some(x), Some(y)) => { stack.push(Value::Boolean(x == y)); Ok(()) }
        _ => Err(VmError::Runtime("EQ underflow".into())),
    }
}

fn bin_cmp<F>(stack: &mut Vec<Value>, f: F) -> Result<(), VmError>
where F: FnOnce(f64,f64)->bool {
    let (b, a) = (stack.pop(), stack.pop());
    match (a, b) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => { stack.push(Value::Boolean(f(x,y))); Ok(()) }
        _ => Err(VmError::Runtime("Comparison type error".into())),
    }
}

fn flip_bool(stack: &mut Vec<Value>) -> Result<(), VmError> {
    let v = stack.pop().ok_or_else(|| VmError::Runtime("flip bool underflow".into()))?;
    match v {
        Value::Boolean(b) => { stack.push(Value::Boolean(!b)); Ok(()) }
        _ => Err(VmError::Runtime("flip bool type error".into())),
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Boolean(false) => false,
        Value::Null => false,
        _ => true,
    }
}
