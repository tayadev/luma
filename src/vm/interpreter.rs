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
    // Module system fields
    module_cache: HashMap<String, Value>,  // Cache of loaded modules (absolute path -> value)
    loading_modules: Vec<String>,          // Stack of currently loading modules (for circular detection)
    current_file: Option<String>,          // Current file being executed (for relative path resolution)
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
            native_functions: HashMap::new(),
            module_cache: HashMap::new(),
            loading_modules: Vec::new(),
            current_file,
        };
        
        // Register native functions
        vm.register_native_function("cast", 2, native_cast);
        vm.register_native_function("isInstanceOf", 2, native_is_instance_of);
        
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
                        _ => return Err(VmError::Runtime("ADD requires (Number, Number) or (String, String)".into())),
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
                Instruction::Import => {
                    let path_val = self.stack.pop().ok_or_else(|| VmError::Runtime("IMPORT requires path on stack".into()))?;
                    let path = match path_val {
                        Value::String(s) => s,
                        _ => return Err(VmError::Runtime("IMPORT requires String path".into())),
                    };
                    
                    // Resolve the path relative to current file
                    let resolved_path = self.resolve_import_path(&path)?;
                    
                    // Check if module is already cached
                    if let Some(cached_value) = self.module_cache.get(&resolved_path).cloned() {
                        self.stack.push(cached_value);
                    } else {
                        // Check for circular dependencies
                        if self.loading_modules.contains(&resolved_path) {
                            let mut cycle = self.loading_modules.clone();
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
        self.loading_modules.push(path.to_string());
        
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
        
        // Share the module cache and loading stack to prevent re-loading
        module_vm.module_cache = self.module_cache.clone();
        module_vm.loading_modules = self.loading_modules.clone();
        
        // Execute the module
        let module_value = module_vm.run()
            .map_err(|e| VmError::Runtime(format!("Error executing module '{}': {:?}", path, e)))?;
        
        // Cache the module value
        self.module_cache.insert(path.to_string(), module_value.clone());
        
        // Unmark module as loading
        self.loading_modules.pop();
        
        Ok(module_value)
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
