use crate::bytecode::ir::{Chunk, Instruction, Constant};
use super::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

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
    native_functions: HashMap<String, fn(&[Value]) -> Result<Value, String>>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        let mut vm = VM { 
            stack: Vec::new(), 
            ip: 0, 
            chunk, 
            globals: HashMap::new(), 
            base: 0, 
            frames: Vec::new(),
            native_functions: HashMap::new(),
        };
        
        // Register native functions
        vm.register_native_function("cast", 2, native_cast);
        vm.register_native_function("isInstanceOf", 2, native_is_instance_of);
        vm.register_native_function("into", 2, native_into);
        
        vm
    }
    
    fn register_native_function(&mut self, name: &str, arity: usize, _func: fn(&[Value]) -> Result<Value, String>) {
        let native_val = Value::NativeFunction { 
            name: name.to_string(), 
            arity 
        };
        self.globals.insert(name.to_string(), native_val.clone());
        self.native_functions.insert(name.to_string(), _func);
    }
    
    /// Helper method to check if a value is a table with a specific method
    /// Checks both the value itself and its type definition (if it has __type metadata)
    fn has_method(value: &Value, method_name: &str) -> Option<Value> {
        match value {
            Value::Table(table) => {
                let borrowed = table.borrow();
                
                // First check if the method exists directly on the value
                if let Some(method) = borrowed.get(method_name) {
                    return Some(method.clone());
                }
                
                // If not, check the type definition (if value has __type metadata)
                // __type can be either Value::Type (created by cast()) or Value::Table (user-defined)
                if let Some(Value::Type(type_table) | Value::Table(type_table)) = borrowed.get("__type") {
                    let type_borrowed = type_table.borrow();
                    if let Some(method) = type_borrowed.get(method_name) {
                        return Some(method.clone());
                    }
                }
                
                None
            }
            _ => None,
        }
    }
    
    /// Helper method to set up and execute a method call for operator overloading
    fn call_overload_method(
        &mut self,
        method: Value,
        args: Vec<Value>,
        expected_arity: usize,
        method_name: &str,
    ) -> Result<(), VmError> {
        match &method {
            Value::Function { chunk, arity } => {
                if *arity != expected_arity {
                    return Err(VmError::Runtime(format!(
                        "{} method must have arity {}, got {}", method_name, expected_arity, arity
                    )));
                }
                
                // Save current frame
                let frame = CallFrame {
                    chunk: self.chunk.clone(),
                    ip: self.ip,
                    base: self.base,
                };
                self.frames.push(frame);
                
                // Set up stack for function call
                self.stack.push(method.clone());
                for arg in args {
                    self.stack.push(arg);
                }
                
                // Set new base to point to first argument
                self.base = self.stack.len() - expected_arity;
                // Switch to method chunk
                self.chunk = chunk.clone();
                self.ip = 0;
                Ok(())
            }
            _ => Err(VmError::Runtime(format!("{} must be a function", method_name))),
        }
    }
    
    /// Helper method to execute a binary operator with optional operator overloading
    fn execute_binary_op(
        &mut self,
        a: Value,
        b: Value,
        method_name: &str,
        default_op: impl FnOnce(&Value, &Value) -> Result<Value, String>,
    ) -> Result<(), VmError> {
        // Try default operation first
        match default_op(&a, &b) {
            Ok(result) => {
                self.stack.push(result);
                Ok(())
            }
            Err(_) => {
                // Try operator overloading
                if let Some(method) = Self::has_method(&a, method_name) {
                    self.call_overload_method(method, vec![a, b], 2, method_name)
                } else {
                    Err(VmError::Runtime(format!(
                        "Operation requires compatible types or {} method", method_name
                    )))
                }
            }
        }
    }
    
    /// Helper for equality comparison with operator overloading
    fn execute_eq_op(&mut self, a: Value, b: Value) -> Result<(), VmError> {
        // Try operator overloading first for tables
        if let Some(method) = Self::has_method(&a, "__eq") {
            self.call_overload_method(method, vec![a, b], 2, "__eq")
        } else {
            // Default equality
            self.stack.push(Value::Boolean(a == b));
            Ok(())
        }
    }
    
    /// Helper for comparison operations with operator overloading
    fn execute_cmp_op(
        &mut self,
        a: Value,
        b: Value,
        method_name: &str,
        default_cmp: impl FnOnce(f64, f64) -> bool,
    ) -> Result<(), VmError> {
        // Try default numeric comparison
        match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => {
                self.stack.push(Value::Boolean(default_cmp(*x, *y)));
                Ok(())
            }
            _ => {
                // Try operator overloading
                if let Some(method) = Self::has_method(&a, method_name) {
                    self.call_overload_method(method, vec![a, b], 2, method_name)
                } else {
                    Err(VmError::Runtime(format!(
                        "Comparison requires Number or {} method", method_name
                    )))
                }
            }
        }
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
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("ADD right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("ADD left underflow".into()))?;
                    
                    self.execute_binary_op(a, b, "__add", |a, b| {
                        match (a, b) {
                            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
                            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
                            _ => Err("Type mismatch".to_string()),
                        }
                    })?;
                }
                Instruction::Sub => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("SUB right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("SUB left underflow".into()))?;
                    
                    self.execute_binary_op(a, b, "__sub", |a, b| {
                        match (a, b) {
                            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
                            _ => Err("Type mismatch".to_string()),
                        }
                    })?;
                }
                Instruction::Mul => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("MUL right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("MUL left underflow".into()))?;
                    
                    self.execute_binary_op(a, b, "__mul", |a, b| {
                        match (a, b) {
                            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),
                            _ => Err("Type mismatch".to_string()),
                        }
                    })?;
                }
                Instruction::Div => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("DIV right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("DIV left underflow".into()))?;
                    
                    self.execute_binary_op(a, b, "__div", |a, b| {
                        match (a, b) {
                            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x / y)),
                            _ => Err("Type mismatch".to_string()),
                        }
                    })?;
                }
                Instruction::Mod => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("MOD right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("MOD left underflow".into()))?;
                    
                    self.execute_binary_op(a, b, "__mod", |a, b| {
                        match (a, b) {
                            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x % y)),
                            _ => Err("Type mismatch".to_string()),
                        }
                    })?;
                }
                Instruction::Neg => {
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("NEG underflow".into()))?;
                    
                    match &a {
                        Value::Number(x) => {
                            self.stack.push(Value::Number(-x));
                        }
                        _ => {
                            // Try operator overloading for __neg
                            if let Some(method) = Self::has_method(&a, "__neg") {
                                self.call_overload_method(method, vec![a], 1, "__neg")?;
                            } else {
                                return Err(VmError::Runtime(
                                    "NEG requires Number or __neg method".into()
                                ));
                            }
                        }
                    }
                }
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
                    self.stack.push(Value::Array(Rc::new(RefCell::new(tmp))));
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
                    self.stack.push(Value::Table(Rc::new(RefCell::new(map))));
                }
                Instruction::GetIndex => {
                    let index = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_INDEX index underflow".into()))?;
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_INDEX obj underflow".into()))?;
                    match (obj, index) {
                        (Value::Array(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 { return Err(VmError::Runtime("Array index negative".into())); }
                            let i = i as usize;
                            let borrowed = arr.borrow();
                            match borrowed.get(i) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("Array index out of bounds".into())) }
                        }
                        (Value::Table(map), Value::String(k)) => {
                            let borrowed = map.borrow();
                            match borrowed.get(&k) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("Table key not found".into())) }
                        }
                        _ => return Err(VmError::Runtime("GET_INDEX type error".into())),
                    }
                }
                Instruction::GetProp(idx) => {
                    let name = match self.chunk.constants.get(idx) { Some(Constant::String(s)) => s.clone(), _ => return Err(VmError::Runtime("GET_PROP expects string const".into())) };
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_PROP obj underflow".into()))?;
                    match obj {
                        Value::Table(map) => {
                            let borrowed = map.borrow();
                            match borrowed.get(&name) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("Property not found".into())) }
                        }
                        _ => return Err(VmError::Runtime("GET_PROP on non-table".into())),
                    }
                }
                Instruction::GetLen => {
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("GET_LEN obj underflow".into()))?;
                    match obj {
                        Value::Array(arr) => {
                            let borrowed = arr.borrow();
                            self.stack.push(Value::Number(borrowed.len() as f64));
                        }
                        Value::Table(map) => {
                            let borrowed = map.borrow();
                            self.stack.push(Value::Number(borrowed.len() as f64));
                        }
                        Value::String(s) => {
                            self.stack.push(Value::Number(s.len() as f64));
                        }
                        _ => return Err(VmError::Runtime("GET_LEN requires array, table, or string".into())),
                    }
                }
                Instruction::SetIndex => {
                    // Stack: value, index, object
                    let value = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_INDEX value underflow".into()))?;
                    let index = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_INDEX index underflow".into()))?;
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_INDEX obj underflow".into()))?;
                    
                    match (obj, index) {
                        (Value::Array(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 { return Err(VmError::Runtime("Array index negative".into())); }
                            let i = i as usize;
                            let mut borrowed = arr.borrow_mut();
                            if i >= borrowed.len() {
                                return Err(VmError::Runtime("Array index out of bounds".into()));
                            }
                            borrowed[i] = value;
                        }
                        (Value::Table(map), Value::String(k)) => {
                            let mut borrowed = map.borrow_mut();
                            borrowed.insert(k, value);
                        }
                        _ => return Err(VmError::Runtime("SET_INDEX type error".into())),
                    }
                }
                Instruction::SetProp(idx) => {
                    // Stack: value, object
                    let name = match self.chunk.constants.get(idx) { 
                        Some(Constant::String(s)) => s.clone(), 
                        _ => return Err(VmError::Runtime("SET_PROP expects string const".into())) 
                    };
                    let value = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_PROP value underflow".into()))?;
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_PROP obj underflow".into()))?;
                    
                    match obj {
                        Value::Table(map) => {
                            let mut borrowed = map.borrow_mut();
                            borrowed.insert(name, value);
                        }
                        _ => return Err(VmError::Runtime("SET_PROP on non-table".into())),
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
                Instruction::SliceArray(start_index) => {
                    let arr = self.stack.pop().ok_or_else(|| VmError::Runtime("SLICE_ARRAY pop underflow".into()))?;
                    match arr {
                        Value::Array(arr_ref) => {
                            let borrowed = arr_ref.borrow();
                            let len = borrowed.len();
                            
                            // Create a slice from start_index to end
                            let slice_start = start_index.min(len);
                            let sliced: Vec<Value> = borrowed[slice_start..].to_vec();
                            
                            self.stack.push(Value::Array(Rc::new(RefCell::new(sliced))));
                        }
                        _ => return Err(VmError::Runtime("SLICE_ARRAY requires an array".into())),
                    }
                }
                Instruction::Eq => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("EQ right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("EQ left underflow".into()))?;
                    self.execute_eq_op(a, b)?;
                }
                Instruction::Ne => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("NE right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("NE left underflow".into()))?;
                    self.execute_eq_op(a, b)?;
                    flip_bool(&mut self.stack)?;
                }
                Instruction::Lt => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("LT right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("LT left underflow".into()))?;
                    self.execute_cmp_op(a, b, "__lt", |a, b| a < b)?;
                }
                Instruction::Le => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("LE right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("LE left underflow".into()))?;
                    self.execute_cmp_op(a, b, "__le", |a, b| a <= b)?;
                }
                Instruction::Gt => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("GT right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("GT left underflow".into()))?;
                    self.execute_cmp_op(a, b, "__gt", |a, b| a > b)?;
                }
                Instruction::Ge => {
                    let b = self.stack.pop().ok_or_else(|| VmError::Runtime("GE right underflow".into()))?;
                    let a = self.stack.pop().ok_or_else(|| VmError::Runtime("GE left underflow".into()))?;
                    self.execute_cmp_op(a, b, "__ge", |a, b| a >= b)?;
                }
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
                        Value::NativeFunction { name, arity: fn_arity } => {
                            if arity != fn_arity {
                                return Err(VmError::Runtime(format!("Arity mismatch: expected {}, got {}", fn_arity, arity)));
                            }
                            // Collect arguments
                            let args: Vec<Value> = self.stack.drain(callee_idx + 1..).collect();
                            // Pop callee
                            self.stack.pop();
                            
                            // Call native function
                            let func = self.native_functions.get(&name)
                                .ok_or_else(|| VmError::Runtime(format!("Native function '{}' not found", name)))?;
                            let result = func(&args)
                                .map_err(|e| VmError::Runtime(e))?;
                            self.stack.push(result);
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

fn flip_bool(stack: &mut Vec<Value>) -> Result<(), VmError> {
    let v = stack.pop().ok_or_else(|| VmError::Runtime("flip bool underflow".into()))?;
    match v {
        Value::Boolean(b) => { stack.push(Value::Boolean(!b)); Ok(()) }
        _ => Err(VmError::Runtime("flip bool type error".into())),
    }
}

fn truthy(v: &Value) -> bool {
    !matches!(v, Value::Boolean(false) | Value::Null)
}

// Helper to extract type definition from a Value (either Table or Type)
fn get_type_map(value: &Value) -> Option<Rc<RefCell<HashMap<String, Value>>>> {
    match value {
        Value::Type(t) => Some(t.clone()),
        Value::Table(t) => Some(t.clone()),
        _ => None,
    }
}

// Helper function to check if a value has all required fields for a type (for trait matching)
fn has_required_fields(value: &Value, type_def: &HashMap<String, Value>) -> Result<bool, String> {
    match value {
        Value::Table(table) => {
            let borrowed = table.borrow();
            
            // Check all required fields in the type definition
            for (field_name, field_type) in type_def.iter() {
                // Skip special fields like __parent and methods (functions)
                if field_name.starts_with("__") {
                    continue;
                }
                
                // If the field type is a function, it's a method - skip validation for methods
                if matches!(field_type, Value::Function { .. } | Value::NativeFunction { .. }) {
                    continue;
                }
                
                // Check if the field exists in the value
                if !borrowed.contains_key(field_name) {
                    return Ok(false);
                }
                
                // For now, we do structural matching - just check if field exists
                // Full type checking would require recursively validating field types
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

// Helper function to check if a value is compatible for casting
// For casting, we allow missing fields (they'll be filled with defaults)
fn is_castable(value: &Value) -> bool {
    matches!(value, Value::Table(_))
}

// Helper function to merge parent fields into child
fn merge_parent_fields(type_def: &HashMap<String, Value>) -> HashMap<String, Value> {
    let mut merged = type_def.clone();
    
    // Check if there's a __parent field
    if let Some(parent_val) = type_def.get("__parent") {
        if let Some(parent_map) = get_type_map(parent_val) {
            let parent_borrowed = parent_map.borrow();
            
            // Recursively merge parent's fields
            let parent_merged = merge_parent_fields(&parent_borrowed);
            
            // Add parent fields that don't exist in child
            for (key, value) in parent_merged.iter() {
                if !merged.contains_key(key) {
                    merged.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    merged
}

// Native function: cast(type, value) -> typed_value
fn native_cast(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("cast() expects 2 arguments, got {}", args.len()));
    }
    
    let type_def = match &args[0] {
        Value::Table(t) => t.clone(),
        Value::Type(t) => t.clone(),
        _ => return Err("cast() first argument must be a type (table)".to_string()),
    };
    
    let value = &args[1];
    
    // Check that the value is a table (castable)
    if !is_castable(value) {
        return Err("cast() can only cast table values".to_string());
    }
    
    // Merge inherited fields from parent types
    let type_borrowed = type_def.borrow();
    let merged_type = merge_parent_fields(&type_borrowed);
    
    // At this point, value is guaranteed to be a Table
    match value {
        Value::Table(table) => {
            let value_borrowed = table.borrow();
            let mut new_table = value_borrowed.clone();
            
            // Merge inherited fields from parent if any
            for (key, val) in merged_type.iter() {
                if !key.starts_with("__") && !new_table.contains_key(key) {
                    // Don't copy methods, only data fields
                    if !matches!(val, Value::Function { .. } | Value::NativeFunction { .. }) {
                        new_table.insert(key.clone(), val.clone());
                    }
                }
            }
            
            // Attach the type definition as metadata
            new_table.insert("__type".to_string(), Value::Type(type_def.clone()));
            
            Ok(Value::Table(Rc::new(RefCell::new(new_table))))
        }
        _ => unreachable!("is_castable ensures value is a Table"),
    }
}

// Native function: isInstanceOf(value, type) -> boolean
fn native_is_instance_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("isInstanceOf() expects 2 arguments, got {}", args.len()));
    }
    
    let value = &args[0];
    let type_def = match &args[1] {
        Value::Table(t) => t.clone(),
        Value::Type(t) => t.clone(),
        _ => return Err("isInstanceOf() second argument must be a type (table)".to_string()),
    };
    
    // Check if the value is a table with __type metadata
    if let Value::Table(table) = value {
        let borrowed = table.borrow();
        
        // Check direct type match
        if let Some(Value::Type(value_type)) = borrowed.get("__type") {
            // Compare type references
            if Rc::ptr_eq(value_type, &type_def) {
                return Ok(Value::Boolean(true));
            }
            
            // Check if value_type inherits from type_def via __parent chain
            let mut current_type = value_type.clone();
            loop {
                let has_parent = {
                    let current_borrowed = current_type.borrow();
                    current_borrowed.get("__parent").cloned()
                };
                
                if let Some(parent_val) = has_parent {
                    if let Some(parent_map) = get_type_map(&parent_val) {
                        // Check if this parent matches the target type
                        if Rc::ptr_eq(&parent_map, &type_def) {
                            return Ok(Value::Boolean(true));
                        }
                        current_type = parent_map;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        // Fallback to structural matching (trait-like behavior)
        let type_borrowed = type_def.borrow();
        if has_required_fields(value, &type_borrowed)? {
            return Ok(Value::Boolean(true));
        }
    }
    
    Ok(Value::Boolean(false))
}

// Native function: into(value, target_type) -> converted_value
// Calls the __into method on the value with the target type
fn native_into(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("into() expects 2 arguments, got {}", args.len()));
    }
    
    let value = &args[0];
    let target_type = &args[1];
    
    // Check if value has __into method
    match value {
        Value::Table(table) => {
            let borrowed = table.borrow();
            if let Some(_into_method) = borrowed.get("__into") {
                // We need to call the __into method with (self, target_type)
                // But we can't easily call it from here without access to the VM
                // For now, return an error suggesting that __into calls must happen in VM context
                return Err(format!(
                    "Type conversions via __into are not fully implemented yet. \
                     For now, use explicit conversion methods or wait for v2. \
                     See GC_HOOKS.md for details."
                ));
            } else {
                return Err(format!("Type does not support conversion (no __into method)"));
            }
        }
        _ => {
            // For non-table values, provide default conversions
            match target_type {
                Value::Type(type_map) | Value::Table(type_map) => {
                    let type_borrowed = type_map.borrow();
                    // Check if target is String type (basic heuristic)
                    // TODO: Improve type matching logic
                    if type_borrowed.contains_key("String") || type_borrowed.is_empty() {
                        // Default string conversion for primitive types
                        match value {
                            Value::Number(n) => Ok(Value::String(n.to_string())),
                            Value::String(s) => Ok(Value::String(s.clone())),
                            Value::Boolean(b) => Ok(Value::String(b.to_string())),
                            Value::Null => Ok(Value::String("null".to_string())),
                            _ => Err("Cannot convert value to String".to_string()),
                        }
                    } else {
                        Err(format!("Unsupported conversion target"))
                    }
                }
                _ => Err("Second argument to into() must be a type".to_string()),
            }
        }
    }
}
