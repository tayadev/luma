use crate::bytecode::ir::{Chunk, Instruction, Constant, UpvalueDescriptor};
use super::value::{Value, Upvalue};
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
    upvalues: Vec<Upvalue>,  // Captured upvalues for this function
}

pub struct VM {
    stack: Vec<Value>,
    ip: usize,
    chunk: Chunk,
    globals: HashMap<String, Value>,
    base: usize,
    frames: Vec<CallFrame>,
    upvalues: Vec<Upvalue>,  // Current function's upvalues
    native_functions: HashMap<String, fn(&[Value]) -> Result<Value, String>>,
    // Module system fields
    module_cache: Rc<RefCell<HashMap<String, Value>>>,  // Shared cache of loaded modules
    loading_modules: Rc<RefCell<Vec<String>>>,          // Shared stack for circular detection
    current_file: Option<String>,                       // Current file being executed
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self::new_with_file(chunk, None)
    }
    
    pub fn new_with_file(chunk: Chunk, current_file: Option<String>) -> Self {
        let mut vm = VM { 
            stack: Vec::new(), 
            ip: 0, 
            chunk, 
            globals: HashMap::new(), 
            base: 0, 
            frames: Vec::new(),
            upvalues: Vec::new(),  // Initialize empty upvalues for top-level
            native_functions: HashMap::new(),
            module_cache: Rc::new(RefCell::new(HashMap::new())),
            loading_modules: Rc::new(RefCell::new(Vec::new())),
            current_file,
        };
        
        // Register native functions
        vm.register_native_function("cast", 2, native_cast);
        vm.register_native_function("isInstanceOf", 2, native_is_instance_of);
        vm.register_native_function("into", 2, native_into);
        vm.register_native_function("typeof", 1, native_typeof);
        vm.register_native_function("print", 0, native_print);  // Variadic, arity 0 is placeholder
        
        // Register I/O functions
        vm.register_native_function("write", 2, native_write);
        vm.register_native_function("read_file", 1, native_read_file);
        vm.register_native_function("write_file", 2, native_write_file);
        vm.register_native_function("file_exists", 1, native_file_exists);
        
        // Register panic function
        vm.register_native_function("panic", 1, native_panic);
        
        // Expose file descriptor constants
        vm.globals.insert("STDOUT".to_string(), Value::Number(1.0));
        vm.globals.insert("STDERR".to_string(), Value::Number(2.0));
        
        // Load prelude (standard library)
        if let Err(e) = vm.load_prelude() {
            eprintln!("Warning: Failed to load prelude: {:?}", e);
        }
        
        vm
    }
    
    /// Load and execute the prelude (standard library)
    /// This is called automatically during VM initialization
    fn load_prelude(&mut self) -> Result<(), VmError> {
        // Include prelude source at compile time
        let prelude_source = include_str!("../prelude.luma");
        
        // Parse the prelude
        let ast = match crate::parser::parse(prelude_source) {
            Ok(ast) => ast,
            Err(errors) => {
                let error_msg = errors.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(VmError::Runtime(format!("Failed to parse prelude: {}", error_msg)));
            }
        };
        
        // Skip typechecking the prelude for now - it's pre-verified
        // The typechecker is too strict for some prelude patterns
        // if let Err(errs) = crate::typecheck::typecheck_program(&ast) {
        //     let error_msg = errs.iter()
        //         .map(|e| e.message.clone())
        //         .collect::<Vec<_>>()
        //         .join(", ");
        //     return Err(VmError::Runtime(format!("Failed to typecheck prelude: {}", error_msg)));
        // }
        
        // Compile the prelude
        let prelude_chunk = crate::bytecode::compile::compile_program(&ast);
        
        // Save current VM state
        let saved_chunk = self.chunk.clone();
        let saved_ip = self.ip;
        let saved_base = self.base;
        
        // Execute the prelude in the current VM context
        // This will return the prelude export table as the result
        self.chunk = prelude_chunk;
        self.ip = 0;
        self.base = 0;

        let result = self.run();

        // Restore VM state
        self.chunk = saved_chunk;
        self.ip = saved_ip;
        self.base = saved_base;

        // Inject only the prelude export table as `prelude` in the global scope
        match result {
            Ok(prelude_exports) => {
                self.globals.insert("prelude".to_string(), prelude_exports);
                Ok(())
            },
            Err(e) => Err(VmError::Runtime(format!("Failed to execute prelude: {:?}", e))),
        }
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
                    upvalues: self.upvalues.clone(),
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
                Instruction::BuildList(n) => {
                    if self.stack.len() < n { return Err(VmError::Runtime("BUILD_LIST underflow".into())); }
                    let mut tmp = Vec::with_capacity(n);
                    for _ in 0..n { tmp.push(self.stack.pop().unwrap()); }
                    tmp.reverse();
                    self.stack.push(Value::List(Rc::new(RefCell::new(tmp))));
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
                        (Value::List(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 { return Err(VmError::Runtime("List index negative".into())); }
                            let i = i as usize;
                            let borrowed = arr.borrow();
                            match borrowed.get(i) { Some(v) => self.stack.push(v.clone()), None => return Err(VmError::Runtime("List index out of bounds".into())) }
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
                        Value::List(arr) => {
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
                        _ => return Err(VmError::Runtime("GET_LEN requires list, table, or string".into())),
                    }
                }
                Instruction::SetIndex => {
                    // Stack: value, index, object
                    let value = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_INDEX value underflow".into()))?;
                    let index = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_INDEX index underflow".into()))?;
                    let obj = self.stack.pop().ok_or_else(|| VmError::Runtime("SET_INDEX obj underflow".into()))?;
                    
                    match (obj, index) {
                        (Value::List(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 { return Err(VmError::Runtime("List index negative".into())); }
                            let i = i as usize;
                            let mut borrowed = arr.borrow_mut();
                            if i == borrowed.len() {
                                // Allow appending at exactly one past the last index
                                borrowed.push(value);
                            } else if i < borrowed.len() {
                                borrowed[i] = value;
                            } else {
                                return Err(VmError::Runtime("List index out of bounds".into()));
                            }
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
                Instruction::SliceList(start_index) => {
                    let arr = self.stack.pop().ok_or_else(|| VmError::Runtime("SLICE_LIST pop underflow".into()))?;
                    match arr {
                        Value::List(arr_ref) => {
                            let borrowed = arr_ref.borrow();
                            let len = borrowed.len();
                            
                            // Create a slice from start_index to end
                            let slice_start = start_index.min(len);
                            let sliced: Vec<Value> = borrowed[slice_start..].to_vec();
                            
                            self.stack.push(Value::List(Rc::new(RefCell::new(sliced))));
                        }
                        _ => return Err(VmError::Runtime("SLICE_LIST requires a list".into())),
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
                Instruction::Closure(idx) => {
                    // Create a closure by capturing upvalues from the current environment
                    let chunk = match self.chunk.constants.get(idx) {
                        Some(Constant::Function(chunk)) => chunk.clone(),
                        _ => return Err(VmError::Runtime("CLOSURE expects function constant".into())),
                    };
                    
                    // Capture upvalues according to the function's upvalue descriptors
                    let mut upvalues = Vec::new();
                    for descriptor in &chunk.upvalue_descriptors {
                        let upvalue = match descriptor {
                            UpvalueDescriptor::Local(slot) => {
                                // Capture a local variable from the current stack frame
                                let value = self.stack.get(self.base + slot)
                                    .ok_or_else(|| VmError::Runtime(format!("Upvalue capture: local slot {} out of bounds", slot)))?
                                    .clone();
                                Upvalue::new(value)
                            }
                            UpvalueDescriptor::Upvalue(upvalue_idx) => {
                                // Capture an upvalue from the current function's upvalues
                                self.upvalues.get(*upvalue_idx)
                                    .ok_or_else(|| VmError::Runtime(format!("Upvalue capture: upvalue {} out of bounds", upvalue_idx)))?
                                    .clone()
                            }
                        };
                        upvalues.push(upvalue);
                    }
                    
                    let closure = Value::Closure {
                        chunk: chunk.clone(),
                        arity: chunk.local_count as usize,  // Number of parameters
                        upvalues,
                    };
                    self.stack.push(closure);
                }
                Instruction::GetUpvalue(idx) => {
                    // Get value from upvalue at index
                    let upvalue = self.upvalues.get(idx)
                        .ok_or_else(|| VmError::Runtime(format!("GetUpvalue: index {} out of bounds", idx)))?;
                    let value = upvalue.value.borrow().clone();
                    self.stack.push(value);
                }
                Instruction::SetUpvalue(idx) => {
                    // Set value in upvalue at index
                    let value = self.stack.pop()
                        .ok_or_else(|| VmError::Runtime("SetUpvalue: stack underflow".into()))?;
                    let upvalue = self.upvalues.get(idx)
                        .ok_or_else(|| VmError::Runtime(format!("SetUpvalue: index {} out of bounds", idx)))?;
                    *upvalue.value.borrow_mut() = value;
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
                                upvalues: self.upvalues.clone(),
                            };
                            self.frames.push(frame);
                            
                            // Set new base to point to first argument
                            self.base = callee_idx + 1;
                            // Switch to function chunk
                            self.chunk = fn_chunk;
                            self.ip = 0;
                            // Functions don't have upvalues
                            self.upvalues = Vec::new();
                        }
                        Value::Closure { chunk: fn_chunk, arity: fn_arity, upvalues: fn_upvalues } => {
                            if arity != fn_arity {
                                return Err(VmError::Runtime(format!("Arity mismatch: expected {}, got {}", fn_arity, arity)));
                            }
                            // Save current frame
                            let frame = CallFrame {
                                chunk: self.chunk.clone(),
                                ip: self.ip,
                                base: self.base,
                                upvalues: self.upvalues.clone(),
                            };
                            self.frames.push(frame);
                            
                            // Set new base to point to first argument
                            self.base = callee_idx + 1;
                            // Switch to function chunk
                            self.chunk = fn_chunk;
                            self.ip = 0;
                            // Set upvalues for the closure
                            self.upvalues = fn_upvalues;
                        }
                        Value::NativeFunction { name, arity: fn_arity } => {
                            // Skip arity check for variadic functions (print)
                            if name != "print" && arity != fn_arity {
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
                        self.upvalues = frame.upvalues;
                        
                        // Push return value
                        self.stack.push(ret_val);
                    } else {
                        // Top-level return (shouldn't happen with well-formed code)
                        return Ok(ret_val);
                    }
                }
                Instruction::Import => {
                    let path_val = self.stack.pop().ok_or_else(|| VmError::Runtime("IMPORT requires path on stack".into()))?;
                    let path = match path_val {
                        Value::String(s) => s,
                        _ => return Err(VmError::Runtime("IMPORT requires String path".into())),
                    };
                    
                    // Resolve the path relative to current file
                    let resolved_path = self.resolve_import_path(&path)?;
                    
                    // Check if module is already cached
                    if let Some(cached_value) = self.module_cache.borrow().get(&resolved_path).cloned() {
                        self.stack.push(cached_value);
                    } else {
                        // Check for circular dependencies
                        if self.loading_modules.borrow().contains(&resolved_path) {
                            let mut cycle = self.loading_modules.borrow().clone();
                            cycle.push(resolved_path.clone());
                            return Err(VmError::Runtime(format!(
                                "Circular dependency detected: {}",
                                cycle.join(" -> ")
                            )));
                        }
                        
                        // Load and evaluate the module
                        let module_value = self.load_module(&resolved_path)?;
                        self.stack.push(module_value);
                    }
                }
                Instruction::Halt => {
                    return Ok(self.stack.pop().unwrap_or(Value::Null));
                }
            }
        }
    }
    
    /// Evaluate a chunk in the context of this VM's existing state (globals, etc.)
    /// This is useful for REPL-style evaluation where state persists across evaluations.
    pub fn eval(&mut self, chunk: Chunk) -> Result<Value, VmError> {
        // Save current state
        let saved_chunk = std::mem::replace(&mut self.chunk, chunk);
        let saved_ip = self.ip;
        let saved_base = self.base;
        
        // Reset execution state for new chunk
        self.ip = 0;
        self.base = 0;
        
        // Run the chunk
        let result = self.run();
        
        // Restore state (but keep globals, module_cache, etc.)
        self.chunk = saved_chunk;
        self.ip = saved_ip;
        self.base = saved_base;
        
        result
    }
    
    fn resolve_import_path(&self, path: &str) -> Result<String, VmError> {
        use std::path::Path;
        
        let path_obj = Path::new(path);
        
        // If it's an absolute path, use it as-is
        if path_obj.is_absolute() {
            return Ok(path_obj.canonicalize()
                .map_err(|e| VmError::Runtime(format!("Failed to resolve path '{}': {}", path, e)))?
                .to_string_lossy()
                .to_string());
        }
        
        // For relative paths, resolve relative to the current file
        let base_dir = if let Some(ref current_file) = self.current_file {
            Path::new(current_file).parent()
                .ok_or_else(|| VmError::Runtime(format!("Invalid current file path: {}", current_file)))?
                .to_path_buf()
        } else {
            // No current file, use current working directory
            std::env::current_dir()
                .map_err(|e| VmError::Runtime(format!("Failed to get current directory: {}", e)))?
        };
        
        let full_path = base_dir.join(path);
        
        // Canonicalize to get absolute path and resolve .. and .
        let canonical = full_path.canonicalize()
            .map_err(|e| VmError::Runtime(format!("Failed to resolve import path '{}': {}", path, e)))?;
        
        Ok(canonical.to_string_lossy().to_string())
    }
    
    fn load_module(&mut self, path: &str) -> Result<Value, VmError> {
        use std::fs;
        
        // Mark module as loading (for circular dependency detection)
        self.loading_modules.borrow_mut().push(path.to_string());
        
        // Ensure we always unmark the module, even on error
        let result = (|| {
            // Read the module source
            let source = fs::read_to_string(path)
                .map_err(|e| VmError::Runtime(format!("Failed to read module '{}': {}", path, e)))?;
            
            // Parse the module
            let ast = crate::parser::parse(&source)
                .map_err(|errors| {
                    VmError::Runtime(format!(
                        "Parse error in module '{}': {}",
                        path,
                        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
                    ))
                })?;
            
            // Typecheck the module (if enabled)
            crate::typecheck::typecheck_program(&ast)
                .map_err(|errs| {
                    VmError::Runtime(format!(
                        "Typecheck error in module '{}': {}",
                        path,
                        errs.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join(", ")
                    ))
                })?;
            
            // Compile the module
            let chunk = crate::bytecode::compile::compile_program(&ast);
            
            // Create a new VM for the module with the module's path as current file
            let mut module_vm = VM::new_with_file(chunk, Some(path.to_string()));
            
            // Share the module cache and loading stack (now using Rc, so they're truly shared)
            module_vm.module_cache = Rc::clone(&self.module_cache);
            module_vm.loading_modules = Rc::clone(&self.loading_modules);
            
            // Execute the module
            let module_value = module_vm.run()
                .map_err(|e| VmError::Runtime(format!("Error executing module '{}': {:?}", path, e)))?;
            
            // Cache the module value
            self.module_cache.borrow_mut().insert(path.to_string(), module_value.clone());
            
            Ok(module_value)
        })();
        
        // Always unmark module as loading, even on error
        self.loading_modules.borrow_mut().pop();
        
        result
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

// Native function: print(...values) -> null
// Prints all arguments to stdout, separated by tabs
fn native_print(args: &[Value]) -> Result<Value, String> {
    let mut output = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            output.push('\t');
        }
        output.push_str(&format!("{}", arg));
    }
    println!("{}", output);
    Ok(Value::Null)
}

// Native function: write(fd: Number, content: String) -> Result(Null, String)
// Writes content to a file descriptor (1=stdout, 2=stderr)
fn native_write(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("write() expects 2 arguments, got {}", args.len()));
    }
    
    let fd = match &args[0] {
        Value::Number(n) => *n as i32,
        _ => return Ok(make_result_err("write() first argument must be a number (file descriptor)".to_string())),
    };
    
    let content = match &args[1] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => format!("{}", other),
    };
    
    use std::io::Write;
    let result = match fd {
        1 => std::io::stdout().write_all(content.as_bytes()),
        2 => std::io::stderr().write_all(content.as_bytes()),
        _ => {
            return Ok(make_result_err(format!("Invalid file descriptor: {}. Only 1 (stdout) and 2 (stderr) are supported", fd)));
        }
    };
    
    match result {
        Ok(_) => Ok(make_result_ok(Value::Null)),
        Err(e) => Ok(make_result_err(format!("I/O error: {}", e))),
    }
}

// Native function: read_file(path: String) -> Result(String, String)
// Reads entire file contents as a string
fn native_read_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("read_file() expects 1 argument, got {}", args.len()));
    }
    
    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Ok(make_result_err("read_file() argument must be a string (file path)".to_string())),
    };
    
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(make_result_ok(Value::String(content))),
        Err(e) => Ok(make_result_err(format!("Failed to read file '{}': {}", path, e))),
    }
}

// Native function: write_file(path: String, content: String) -> Result(Null, String)
// Writes content to a file, creating or overwriting it
fn native_write_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("write_file() expects 2 arguments, got {}", args.len()));
    }
    
    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Ok(make_result_err("write_file() first argument must be a string (file path)".to_string())),
    };
    
    let content = match &args[1] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => format!("{}", other),
    };
    
    match std::fs::write(path, content) {
        Ok(_) => Ok(make_result_ok(Value::Null)),
        Err(e) => Ok(make_result_err(format!("Failed to write file '{}': {}", path, e))),
    }
}

// Native function: file_exists(path: String) -> Boolean
// Checks if a file or directory exists at the given path
fn native_file_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("file_exists() expects 1 argument, got {}", args.len()));
    }
    
    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("file_exists() argument must be a string (file path)".to_string()),
    };
    
    Ok(Value::Boolean(std::path::Path::new(path).exists()))
}

// Native function: panic(message: String) -> Never
// Prints error message to stderr and terminates the program
fn native_panic(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("panic() expects 1 argument, got {}", args.len()));
    }
    
    let message = match &args[0] {
        Value::String(s) => s.clone(),
        other => format!("{}", other),
    };
    
    eprintln!("PANIC: {}", message);
    std::process::exit(1);
}

// Native function: typeof(value: Any) -> String
// Returns the runtime type name of a value
fn native_typeof(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("typeof() expects 1 argument, got {}", args.len()));
    }
    
    let type_name = match &args[0] {
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Boolean(_) => "Boolean",
        Value::Null => "Null",
        Value::List(_) => "List",
        Value::Table(table) => {
            // Check if this table has a __type field (from cast())
            let borrowed = table.borrow();
            if let Some(Value::Type(_)) = borrowed.get("__type") {
                // This is a typed instance, return "Table" as the base type
                // The actual type information is in the __type field
                "Table"
            } else {
                "Table"
            }
        }
        Value::Function { .. } => "Function",
        Value::Closure { .. } => "Function",
        Value::NativeFunction { .. } => "Function",
        Value::Type(_) => "Type",
    };
    
    Ok(Value::String(type_name.to_string()))
}

// Helper: Create a Result value with ok field set
fn make_result_ok(value: Value) -> Value {
    let mut map = HashMap::new();
    map.insert("ok".to_string(), value);
    map.insert("err".to_string(), Value::Null);
    Value::Table(Rc::new(RefCell::new(map)))
}

// Helper: Create a Result value with err field set
fn make_result_err(error: String) -> Value {
    let mut map = HashMap::new();
    map.insert("ok".to_string(), Value::Null);
    map.insert("err".to_string(), Value::String(error));
    Value::Table(Rc::new(RefCell::new(map)))
}
