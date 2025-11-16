use crate::bytecode::ir::{Chunk, Instruction, Constant};
use super::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub enum VmError {
    Runtime(String),
}

struct CallFrame {
    chunk: Chunk,
    ip: usize,
    base: usize,
}

pub struct VM {
    stack: Vec<Value>,
    ip: usize,
    chunk: Chunk,
    globals: HashMap<String, Value>,
    base: usize,
    frames: Vec<CallFrame>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        VM { stack: Vec::new(), ip: 0, chunk, globals: HashMap::new(), base: 0, frames: Vec::new() }
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
                        Some(Constant::Function(chunk)) => Value::Function {
                            chunk: chunk.clone(),
                            arity: chunk.local_count as usize,
                        },
                        None => return Err(VmError::Runtime("Bad const index".into())),
                    };
                    self.stack.push(v);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::PopNPreserve(n) => {
                    // Preserve the top value, pop n items beneath, then restore the preserved value
                    let top = self.stack.pop().ok_or_else(|| VmError::Runtime("POPN_PRESERVE on empty stack".into()))?;
                    for _ in 0..n {
                        if self.stack.pop().is_none() {
                            return Err(VmError::Runtime("POPN_PRESERVE underflow".into()));
                        }
                    }
                    self.stack.push(top);
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
                Instruction::BuildArray(n) => {
                    if self.stack.len() < n { return Err(VmError::Runtime("BUILD_ARRAY underflow".into())); }
                    let mut tmp = Vec::with_capacity(n);
                    for _ in 0..n { tmp.push(self.stack.pop().unwrap()); }
                    tmp.reverse();
                    self.stack.push(Value::Array(tmp));
                }
                Instruction::BuildTable(n) => {
                    if self.stack.len() < n * 2 { return Err(VmError::Runtime("BUILD_TABLE underflow".into())); }
                    let mut map: HashMap<String, Value> = HashMap::with_capacity(n);
                    for _ in 0..n {
                        let val = self.stack.pop().unwrap();
                        let key_v = self.stack.pop().unwrap();
                        let key = match key_v { Value::String(s) => s, _ => return Err(VmError::Runtime("TABLE key must be string".into())) };
                        map.insert(key, val);
                    }
                    self.stack.push(Value::Table(map));
                }
                Instruction::GetIndex => {
                    let index = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_INDEX index underflow".into()))?;
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_INDEX obj underflow".into()))?;
                    match (obj, index) {
                        (Value::Array(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 { return Err(VmError::Runtime("Array index negative".into())); }
                            let i = i as usize;
                            match arr.get(i) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("Array index out of bounds".into())) }
                        }
                        (Value::Table(map), Value::String(k)) => {
                            match map.get(&k) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("Table key not found".into())) }
                        }
                        _ => return Err(VmError::Runtime("GET_INDEX type error".into())),
                    }
                }
                Instruction::GetProp(idx) => {
                    let name = match self.chunk.constants.get(idx) { Some(Constant::String(s)) => s.clone(), _ => return Err(VmError::Runtime("GET_PROP expects string const".into())) };
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_PROP obj underflow".into()))?;
                    match obj {
                        Value::Table(map) => match map.get(&name) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("Property not found".into())) },
                        _ => return Err(VmError::Runtime("GET_PROP on non-table".into())),
                    }
                }
                Instruction::GetLocal(slot) => {
                    let idx = self.base + slot;
                    let v = self.stack.get(idx).cloned().ok_or_else(|| VmError::Runtime("GET_LOCAL out of range".into()))?;
                    self.stack.push(v);
                }
                Instruction::SetLocal(slot) => {
                    let v = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_LOCAL pop underflow".into()))?;
                    let idx = self.base + slot;
                    if idx >= self.stack.len() { return Err(VmError::Runtime("SET_LOCAL out of range".into())); }
                    self.stack[idx] = v;
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
                Instruction::MakeFunction(idx) => {
                    // Function constant already created during CONST; this is a no-op for now
                    // In a more complex implementation, we'd capture upvalues here
                    let v = match self.chunk.constants.get(idx) {
                        Some(Constant::Function(chunk)) => Value::Function {
                            chunk: chunk.clone(),
                            arity: chunk.local_count as usize,
                        },
                        _ => return Err(VmError::Runtime("MAKE_FUNCTION expects function constant".into())),
                    };
                    self.stack.push(v);
                }
                Instruction::Call(arity) => {
                    // Stack: [... callee arg1 arg2 ... argN]
                    // After call: [... result]
                    let callee_idx = self.stack.len() - arity - 1;
                    let callee = self.stack.get(callee_idx).cloned()
                        .ok_or_else(|| VmError::Runtime("CALL callee underflow".into()))?;
                    
                    match callee {
                        Value::Function { chunk: fn_chunk, arity: fn_arity } => {
                            if arity != fn_arity {
                                return Err(VmError::Runtime(format!("Arity mismatch: expected {}, got {}", fn_arity, arity)));
                            }
                            // Save current frame
                            let frame = CallFrame {
                                chunk: self.chunk.clone(),
                                ip: self.ip,
                                base: self.base,
                            };
                            self.frames.push(frame);
                            
                            // Set new base to point to first argument
                            self.base = callee_idx + 1;
                            // Switch to function chunk
                            self.chunk = fn_chunk;
                            self.ip = 0;
                        }
                        _ => return Err(VmError::Runtime("CALL on non-function".into())),
                    }
                }
                Instruction::Return => {
                    // Pop return value
                    let ret_val = self.stack.pop().unwrap_or(Value::Null);
                    
                    // Pop all locals and arguments (everything from base onwards)
                    self.stack.truncate(self.base - 1); // Keep everything before the callee
                    
                    // Restore previous frame
                    if let Some(frame) = self.frames.pop() {
                        self.chunk = frame.chunk;
                        self.ip = frame.ip;
                        self.base = frame.base;
                        
                        // Push return value
                        self.stack.push(ret_val);
                    } else {
                        // Top-level return (shouldn't happen with well-formed code)
                        return Ok(ret_val);
                    }
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
