//! Virtual machine interpreter for Luma bytecode.
//!
//! This module implements a stack-based bytecode interpreter with support for:
//!
//! - **Stack management**: Values are pushed/popped from a stack during execution
//! - **Closures and upvalues**: Functions can capture variables from outer scopes
//! - **Operator overloading**: Custom types can define behavior for operators
//! - **Module system**: Import and caching of external modules
//! - **Native functions**: Built-in functions implemented in Rust
//! - **Call frames**: Function calls maintain their own execution context
//!
//! ## Execution Model
//!
//! The VM executes bytecode instructions in a loop, maintaining:
//! - A value stack for computation
//! - A call stack (frames) for function calls
//! - Global variables accessible from all scopes
//! - Upvalues for closure captures
//!
//! ## Error Handling
//!
//! Runtime errors are returned as `VmError` with optional source location information
//! for better error messages.

use super::native::*;
use super::value::{Upvalue, Value};
use super::{modules, operators};
use crate::ast::Span;
use crate::bytecode::ir::{Chunk, Constant, Instruction, UpvalueDescriptor};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Type alias for native function signatures
pub type NativeFunction = fn(&[Value]) -> Result<Value, String>;

/// Represents a runtime error with optional source location information
#[derive(Debug)]
pub struct VmError {
    pub message: String,
    pub span: Option<Span>,
    pub file: Option<String>,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(None))
    }
}

impl VmError {
    /// Create a simple runtime error without location info
    pub fn runtime(message: String) -> Self {
        VmError {
            message,
            span: None,
            file: None,
        }
    }

    /// Create a runtime error with location info
    pub fn with_location(message: String, span: Option<Span>, file: Option<String>) -> Self {
        VmError {
            message,
            span,
            file,
        }
    }

    /// Format the error with source location if available
    pub fn format(&self, source: Option<&str>) -> String {
        let mut result = String::new();

        if let (Some(file), Some(span)) = (&self.file, &self.span) {
            if let Some(src) = source {
                let loc = span.location(src);
                result.push_str(&format!(
                    "Runtime error at {}:{}:{}\n",
                    file, loc.line, loc.col
                ));

                // Show the line with the error
                let lines: Vec<&str> = src.lines().collect();
                if loc.line > 0 && loc.line <= lines.len() {
                    result.push_str(&format!("  {} | {}\n", loc.line, lines[loc.line - 1]));
                    result.push_str(&format!(
                        "  {} | {}{}\n",
                        " ".repeat(loc.line.to_string().len()),
                        " ".repeat(loc.col.saturating_sub(1)),
                        "^"
                    ));
                }
            } else {
                result.push_str(&format!("Runtime error at {}\n", file));
            }
        } else {
            result.push_str("Runtime error\n");
        }

        result.push_str(&self.message);
        result
    }
}

pub struct CallFrame {
    pub chunk: Chunk,
    pub ip: usize,
    pub base: usize,
    pub upvalues: Vec<Upvalue>, // Captured upvalues for this function
    // Captured locals for this frame: absolute stack index -> shared cell
    pub captured_locals: HashMap<usize, Upvalue>,
}

pub struct VM {
    pub stack: Vec<Value>,
    pub ip: usize,
    pub chunk: Chunk,
    pub globals: HashMap<String, Value>,
    pub base: usize,
    pub frames: Vec<CallFrame>,
    pub upvalues: Vec<Upvalue>, // Current function's upvalues
    // Captured locals for the current frame: absolute stack index -> shared cell
    pub captured_locals: HashMap<usize, Upvalue>,
    pub native_functions: HashMap<String, NativeFunction>,
    // Module system fields
    pub module_cache: Rc<RefCell<HashMap<String, Value>>>, // Shared cache of loaded modules
    pub loading_modules: Rc<RefCell<Vec<String>>>,         // Shared stack for circular detection
    pub current_file: Option<String>,                      // Current file being executed
    pub source: Option<String>,                            // Source code for error reporting
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
            upvalues: Vec::new(), // Initialize empty upvalues for top-level
            captured_locals: HashMap::new(),
            native_functions: HashMap::new(),
            module_cache: Rc::new(RefCell::new(HashMap::new())),
            loading_modules: Rc::new(RefCell::new(Vec::new())),
            current_file,
            source: None,
        };

        // Register native functions
        vm.register_native_function("cast", 2, native_cast);
        vm.register_native_function("isInstanceOf", 2, native_is_instance_of);
        vm.register_native_function("into", 2, native_into);
        vm.register_native_function("typeof", 1, native_typeof);
        vm.register_native_function("iter", 1, native_iter);
        vm.register_native_function("print", 0, native_print); // Variadic, arity 0 is placeholder

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

    /// Set the source code for error reporting
    pub fn set_source(&mut self, source: String) {
        self.source = Some(source);
    }

    /// Get the current span based on IP
    fn _current_span(&self) -> Option<Span> {
        self.chunk.get_span(self.ip)
    }

    /// Create a runtime error with current location
    fn _error(&self, message: String) -> VmError {
        VmError::with_location(message, self._current_span(), self.current_file.clone())
    }

    /// Load and execute the prelude (standard library)
    /// This is called automatically during VM initialization
    fn load_prelude(&mut self) -> Result<(), VmError> {
        // Include prelude source at compile time
        let prelude_source = include_str!("../prelude.luma");

        // Parse the prelude
        let ast = match crate::parser::parse(prelude_source, "<prelude>") {
            Ok(ast) => ast,
            Err(errors) => {
                let error_msg = errors
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(VmError::runtime(format!(
                    "Failed to parse prelude: {}",
                    error_msg
                )));
            }
        };

        // Note: Prelude typechecking is skipped as it uses advanced patterns
        // that the current typechecker doesn't fully support. The prelude is
        // manually verified for correctness during development.

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
            }
            Err(e) => Err(VmError::runtime(format!(
                "Failed to execute prelude: {:?}",
                e
            ))),
        }
    }

    fn register_native_function(
        &mut self,
        name: &str,
        arity: usize,
        _func: NativeFunction,
    ) {
        let native_val = Value::NativeFunction {
            name: name.to_string(),
            arity,
        };
        self.globals.insert(name.to_string(), native_val.clone());
        self.native_functions.insert(name.to_string(), _func);
    }

    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            if self.ip >= self.chunk.instructions.len() {
                return Err(self._error("IP out of bounds".into()));
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
                        None => return Err(self._error("Bad const index".into())),
                    };
                    self.stack.push(v);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::PopNPreserve(n) => {
                    // Preserve the top value, pop n items beneath, then restore the preserved value
                    let top = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("POPN_PRESERVE on empty stack".into()))?;
                    for _ in 0..n {
                        if self.stack.pop().is_none() {
                            return Err(self._error("POPN_PRESERVE underflow".into()));
                        }
                    }
                    self.stack.push(top);
                }
                Instruction::Dup => {
                    if let Some(v) = self.stack.last().cloned() {
                        self.stack.push(v);
                    } else {
                        return Err(self._error("DUP on empty stack".into()));
                    }
                }
                Instruction::Add => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("ADD right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("ADD left underflow".into()))?;

                    operators::execute_binary_op(self, a, b, "__add", |a, b| match (a, b) {
                        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
                        (Value::String(x), Value::String(y)) => {
                            Ok(Value::String(format!("{}{}", x, y)))
                        }
                        _ => Err("Type mismatch".to_string()),
                    })?;
                }
                Instruction::Sub => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SUB right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SUB left underflow".into()))?;

                    operators::execute_binary_op(self, a, b, "__sub", |a, b| match (a, b) {
                        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
                        _ => Err("Type mismatch".to_string()),
                    })?;
                }
                Instruction::Mul => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("MUL right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("MUL left underflow".into()))?;

                    operators::execute_binary_op(self, a, b, "__mul", |a, b| match (a, b) {
                        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),
                        _ => Err("Type mismatch".to_string()),
                    })?;
                }
                Instruction::Div => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("DIV right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("DIV left underflow".into()))?;

                    operators::execute_binary_op(self, a, b, "__div", |a, b| match (a, b) {
                        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x / y)),
                        _ => Err("Type mismatch".to_string()),
                    })?;
                }
                Instruction::Mod => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("MOD right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("MOD left underflow".into()))?;

                    operators::execute_binary_op(self, a, b, "__mod", |a, b| match (a, b) {
                        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x % y)),
                        _ => Err("Type mismatch".to_string()),
                    })?;
                }
                Instruction::Neg => {
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("NEG underflow".into()))?;

                    match &a {
                        Value::Number(x) => {
                            self.stack.push(Value::Number(-x));
                        }
                        _ => {
                            // Try operator overloading for __neg
                            if let Some(method) = operators::has_method(&a, "__neg") {
                                operators::call_overload_method(self, method, vec![a], 1, "__neg")?;
                            } else {
                                return Err(self._error(
                                    "NEG requires Number or __neg method".into(),
                                ));
                            }
                        }
                    }
                }
                Instruction::GetGlobal(idx) => {
                    let name = match self.chunk.constants.get(idx) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => {
                            return Err(self._error(
                                "GET_GLOBAL expects string constant".into(),
                            ));
                        }
                    };
                    if let Some(v) = self.globals.get(&name).cloned() {
                        self.stack.push(v);
                    } else {
                        return Err(self._error(format!("Undefined global '{}'", name)));
                    }
                }
                Instruction::SetGlobal(idx) => {
                    let name = match self.chunk.constants.get(idx) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => {
                            return Err(self._error(
                                "SET_GLOBAL expects string constant".into(),
                            ));
                        }
                    };
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_GLOBAL pop underflow".into()))?;
                    self.globals.insert(name, v);
                }
                Instruction::BuildList(n) => {
                    if self.stack.len() < n {
                        return Err(self._error("BUILD_LIST underflow".into()));
                    }
                    let mut tmp = Vec::with_capacity(n);
                    for _ in 0..n {
                        tmp.push(self.stack.pop().unwrap());
                    }
                    tmp.reverse();
                    self.stack.push(Value::List(Rc::new(RefCell::new(tmp))));
                }
                Instruction::BuildTable(n) => {
                    if self.stack.len() < n * 2 {
                        return Err(self._error("BUILD_TABLE underflow".into()));
                    }
                    let mut map: HashMap<String, Value> = HashMap::with_capacity(n);
                    for _ in 0..n {
                        let val = self.stack.pop().unwrap();
                        let key_v = self.stack.pop().unwrap();
                        let key = match key_v {
                            Value::String(s) => s,
                            _ => return Err(self._error("TABLE key must be string".into())),
                        };
                        map.insert(key, val);
                    }
                    self.stack.push(Value::Table(Rc::new(RefCell::new(map))));
                }
                Instruction::GetIndex => {
                    let index = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GET_INDEX index underflow".into()))?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GET_INDEX obj underflow".into()))?;
                    match (obj, index) {
                        (Value::List(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 {
                                return Err(self._error("List index negative".into()));
                            }
                            let i = i as usize;
                            let borrowed = arr.borrow();
                            match borrowed.get(i) {
                                Some(v) => self.stack.push(v.clone()),
                                None => {
                                    return Err(self._error(
                                        "List index out of bounds".into(),
                                    ));
                                }
                            }
                        }
                        (Value::Table(map), Value::String(k)) => {
                            let borrowed = map.borrow();
                            match borrowed.get(&k) {
                                Some(v) => self.stack.push(v.clone()),
                                None => return Err(self._error("Table key not found".into())),
                            }
                        }
                        _ => return Err(self._error("GET_INDEX type error".into())),
                    }
                }
                Instruction::GetProp(idx) => {
                    let name = match self.chunk.constants.get(idx) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => return Err(self._error("GET_PROP expects string const".into())),
                    };
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GET_PROP obj underflow".into()))?;
                    match obj {
                        Value::Table(map) => {
                            let borrowed = map.borrow();
                            match borrowed.get(&name) {
                                Some(v) => self.stack.push(v.clone()),
                                None => return Err(self._error("Property not found".into())),
                            }
                        }
                        _ => return Err(self._error("GET_PROP on non-table".into())),
                    }
                }
                Instruction::GetLen => {
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GET_LEN obj underflow".into()))?;
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
                        _ => {
                            return Err(self._error(
                                "GET_LEN requires list, table, or string".into(),
                            ));
                        }
                    }
                }
                Instruction::SetIndex => {
                    // Stack: value, index, object
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_INDEX value underflow".into()))?;
                    let index = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_INDEX index underflow".into()))?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_INDEX obj underflow".into()))?;

                    match (obj, index) {
                        (Value::List(arr), Value::Number(n)) => {
                            let i = n as i64;
                            if i < 0 {
                                return Err(self._error("List index negative".into()));
                            }
                            let i = i as usize;
                            let mut borrowed = arr.borrow_mut();
                            if i == borrowed.len() {
                                // Allow appending at exactly one past the last index
                                borrowed.push(value);
                            } else if i < borrowed.len() {
                                borrowed[i] = value;
                            } else {
                                return Err(self._error("List index out of bounds".into()));
                            }
                        }
                        (Value::Table(map), Value::String(k)) => {
                            let mut borrowed = map.borrow_mut();
                            borrowed.insert(k, value);
                        }
                        _ => return Err(self._error("SET_INDEX type error".into())),
                    }
                }
                Instruction::SetProp(idx) => {
                    // Stack: value, object
                    let name = match self.chunk.constants.get(idx) {
                        Some(Constant::String(s)) => s.clone(),
                        _ => return Err(self._error("SET_PROP expects string const".into())),
                    };
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_PROP value underflow".into()))?;
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_PROP obj underflow".into()))?;

                    match obj {
                        Value::Table(map) => {
                            let mut borrowed = map.borrow_mut();
                            borrowed.insert(name, value);
                        }
                        _ => return Err(self._error("SET_PROP on non-table".into())),
                    }
                }
                Instruction::GetLocal(slot) => {
                    let idx = self.base + slot;
                    if let Some(cell) = self.captured_locals.get(&idx) {
                        let v = cell.value.borrow().clone();
                        self.stack.push(v);
                    } else {
                        let v = self
                            .stack
                            .get(idx)
                            .cloned()
                            .ok_or_else(|| self._error("GET_LOCAL out of range".into()))?;
                        self.stack.push(v);
                    }
                }
                Instruction::SetLocal(slot) => {
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SET_LOCAL pop underflow".into()))?;
                    let idx = self.base + slot;
                    if let Some(cell) = self.captured_locals.get(&idx) {
                        *cell.value.borrow_mut() = v;
                    } else {
                        if idx >= self.stack.len() {
                            return Err(self._error("SET_LOCAL out of range".into()));
                        }
                        self.stack[idx] = v;
                    }
                }
                Instruction::SliceList(start_index) => {
                    let arr = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SLICE_LIST pop underflow".into()))?;
                    match arr {
                        Value::List(arr_ref) => {
                            let borrowed = arr_ref.borrow();
                            let len = borrowed.len();

                            // Create a slice from start_index to end
                            let slice_start = start_index.min(len);
                            let sliced: Vec<Value> = borrowed[slice_start..].to_vec();

                            self.stack.push(Value::List(Rc::new(RefCell::new(sliced))));
                        }
                        _ => return Err(self._error("SLICE_LIST requires a list".into())),
                    }
                }
                Instruction::Eq => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("EQ right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("EQ left underflow".into()))?;
                    operators::execute_eq_op(self, a, b)?;
                }
                Instruction::Ne => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("NE right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("NE left underflow".into()))?;
                    operators::execute_eq_op(self, a, b)?;
                    flip_bool(&mut self.stack)?;
                }
                Instruction::Lt => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("LT right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("LT left underflow".into()))?;
                    operators::execute_cmp_op(self, a, b, "__lt", |a, b| a < b)?;
                }
                Instruction::Le => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("LE right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("LE left underflow".into()))?;
                    operators::execute_cmp_op(self, a, b, "__le", |a, b| a <= b)?;
                }
                Instruction::Gt => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GT right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GT left underflow".into()))?;
                    operators::execute_cmp_op(self, a, b, "__gt", |a, b| a > b)?;
                }
                Instruction::Ge => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GE right underflow".into()))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("GE left underflow".into()))?;
                    operators::execute_cmp_op(self, a, b, "__ge", |a, b| a >= b)?;
                }
                Instruction::Not => {
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("NOT on empty stack".into()))?;
                    self.stack.push(Value::Boolean(!truthy(&v)));
                }
                Instruction::Jump(target) => {
                    self.ip = target;
                }
                Instruction::JumpIfFalse(target) => {
                    let v = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("JUMP_IF_FALSE pop underflow".into()))?;
                    if !truthy(&v) {
                        self.ip = target;
                    }
                }
                Instruction::MakeFunction(idx) => {
                    // Function constant already created during CONST; this is a no-op for now
                    // In a more complex implementation, we'd capture upvalues here
                    let v = match self.chunk.constants.get(idx) {
                        Some(Constant::Function(chunk)) => Value::Function {
                            chunk: chunk.clone(),
                            arity: chunk.local_count as usize,
                        },
                        _ => {
                            return Err(self._error(
                                "MAKE_FUNCTION expects function constant".into(),
                            ));
                        }
                    };
                    self.stack.push(v);
                }
                Instruction::Closure(idx) => {
                    // Create a closure by capturing upvalues from the current environment
                    let chunk = match self.chunk.constants.get(idx) {
                        Some(Constant::Function(chunk)) => chunk.clone(),
                        _ => {
                            return Err(self._error(
                                "CLOSURE expects function constant".into(),
                            ));
                        }
                    };

                    // Capture upvalues according to the function's upvalue descriptors
                    let mut upvalues = Vec::new();
                    for descriptor in &chunk.upvalue_descriptors {
                        let upvalue = match descriptor {
                            UpvalueDescriptor::Local(slot) => {
                                // Capture a local variable from the current stack frame using a shared cell
                                let abs = self.base + slot;
                                if let Some(cell) = self.captured_locals.get(&abs) {
                                    cell.clone()
                                } else {
                                    let value = self
                                        .stack
                                        .get(abs)
                                        .ok_or_else(|| {
                                            self._error(format!(
                                                "Upvalue capture: local slot {} out of bounds",
                                                slot
                                            ))
                                        })?
                                        .clone();
                                    let cell = Upvalue::new(value);
                                    self.captured_locals.insert(abs, cell.clone());
                                    cell
                                }
                            }
                            UpvalueDescriptor::Upvalue(upvalue_idx) => {
                                // Capture an upvalue from the current function's upvalues
                                self.upvalues
                                    .get(*upvalue_idx)
                                    .ok_or_else(|| {
                                        self._error(format!(
                                            "Upvalue capture: upvalue {} out of bounds",
                                            upvalue_idx
                                        ))
                                    })?
                                    .clone()
                            }
                        };
                        upvalues.push(upvalue);
                    }

                    let closure = Value::Closure {
                        chunk: chunk.clone(),
                        arity: chunk.local_count as usize, // Number of parameters
                        upvalues,
                    };
                    self.stack.push(closure);
                }
                Instruction::GetUpvalue(idx) => {
                    // Get value from upvalue at index
                    let upvalue = self.upvalues.get(idx).ok_or_else(|| {
                        self._error(format!("GetUpvalue: index {} out of bounds", idx))
                    })?;
                    let value = upvalue.value.borrow().clone();
                    self.stack.push(value);
                }
                Instruction::SetUpvalue(idx) => {
                    // Set value in upvalue at index
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("SetUpvalue: stack underflow".into()))?;
                    let upvalue = self.upvalues.get(idx).ok_or_else(|| {
                        self._error(format!("SetUpvalue: index {} out of bounds", idx))
                    })?;
                    *upvalue.value.borrow_mut() = value;
                }
                Instruction::Call(arity) => {
                    // Stack: [... callee arg1 arg2 ... argN]
                    // After call: [... result]
                    let callee_idx = self.stack.len() - arity - 1;
                    let callee = self
                        .stack
                        .get(callee_idx)
                        .cloned()
                        .ok_or_else(|| self._error("CALL callee underflow".into()))?;

                    match callee {
                        Value::Function {
                            chunk: fn_chunk,
                            arity: fn_arity,
                        } => {
                            if arity != fn_arity {
                                return Err(self._error(format!(
                                    "Arity mismatch: expected {}, got {}",
                                    fn_arity, arity
                                )));
                            }
                            // Save current frame
                            let frame = CallFrame {
                                chunk: self.chunk.clone(),
                                ip: self.ip,
                                base: self.base,
                                upvalues: self.upvalues.clone(),
                                captured_locals: std::mem::take(&mut self.captured_locals),
                            };
                            self.frames.push(frame);

                            // Set new base to point to first argument
                            self.base = callee_idx + 1;
                            // Switch to function chunk
                            self.chunk = fn_chunk;
                            self.ip = 0;
                            // Functions don't have upvalues and start with no captured locals
                            self.upvalues = Vec::new();
                            self.captured_locals = HashMap::new();
                        }
                        Value::Closure {
                            chunk: fn_chunk,
                            arity: fn_arity,
                            upvalues: fn_upvalues,
                        } => {
                            if arity != fn_arity {
                                return Err(self._error(format!(
                                    "Arity mismatch: expected {}, got {}",
                                    fn_arity, arity
                                )));
                            }
                            // Save current frame
                            let frame = CallFrame {
                                chunk: self.chunk.clone(),
                                ip: self.ip,
                                base: self.base,
                                upvalues: self.upvalues.clone(),
                                captured_locals: std::mem::take(&mut self.captured_locals),
                            };
                            self.frames.push(frame);

                            // Set new base to point to first argument
                            self.base = callee_idx + 1;
                            // Switch to function chunk
                            self.chunk = fn_chunk;
                            self.ip = 0;
                            // Set upvalues for the closure and clear captured locals for new frame
                            self.upvalues = fn_upvalues;
                            self.captured_locals = HashMap::new();
                        }
                        Value::NativeFunction {
                            name,
                            arity: fn_arity,
                        } => {
                            // Skip arity check for variadic functions (print)
                            if name != "print" && arity != fn_arity {
                                return Err(self._error(format!(
                                    "Arity mismatch: expected {}, got {}",
                                    fn_arity, arity
                                )));
                            }
                            // Collect arguments
                            let args: Vec<Value> = self.stack.drain(callee_idx + 1..).collect();
                            // Pop callee
                            self.stack.pop();

                            // Special dispatch for into(value, target_type)
                            if name == "into" {
                                if args.len() != 2 {
                                    return Err(self._error(format!(
                                        "into() expects 2 arguments, got {}",
                                        args.len()
                                    )));
                                }
                                let value = args[0].clone();
                                let target_type = args[1].clone();

                                // If value has __into, call it as a regular function
                                if let Some(method) = operators::has_method(&value, "__into") {
                                    // call_overload_method sets up the new frame and switches chunks
                                    operators::call_overload_method(
                                        self,
                                        method,
                                        vec![value, target_type],
                                        2,
                                        "__into",
                                    )?;
                                    // Do not push a result here; execution will continue in the method
                                } else {
                                    // Primitive fallback: allow simple String conversions
                                    match target_type {
                                        Value::Type(tmap) | Value::Table(tmap) => {
                                            let tb = tmap.borrow();
                                            let is_string_target =
                                                tb.contains_key("String") || tb.is_empty();
                                            if is_string_target {
                                                let converted = match value {
                                                    Value::Number(n) => {
                                                        Value::String(n.to_string())
                                                    }
                                                    Value::String(s) => Value::String(s),
                                                    Value::Boolean(b) => {
                                                        Value::String(b.to_string())
                                                    }
                                                    Value::Null => {
                                                        Value::String("null".to_string())
                                                    }
                                                    other => Value::String(format!("{}", other)),
                                                };
                                                self.stack.push(converted);
                                            } else {
                                                return Err(self._error("Type does not support conversion (no __into method)".to_string()));
                                            }
                                        }
                                        _ => {
                                            return Err(self._error(
                                                "Second argument to into() must be a type"
                                                    .to_string(),
                                            ));
                                        }
                                    }
                                }
                            } else {
                                // Call native function normally
                                let func = self.native_functions.get(&name).ok_or_else(|| {
                                    self._error(format!(
                                        "Native function '{}' not found",
                                        name
                                    ))
                                })?;
                                let result = func(&args).map_err(|e| self._error(e))?;
                                self.stack.push(result);
                            }
                        }
                        _ => return Err(self._error("CALL on non-function".into())),
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
                        self.captured_locals = frame.captured_locals;

                        // Push return value
                        self.stack.push(ret_val);
                    } else {
                        // Top-level return (shouldn't happen with well-formed code)
                        return Ok(ret_val);
                    }
                }
                Instruction::Import => {
                    let path_val = self
                        .stack
                        .pop()
                        .ok_or_else(|| self._error("IMPORT requires path on stack".into()))?;
                    let path = match path_val {
                        Value::String(s) => s,
                        _ => return Err(self._error("IMPORT requires String path".into())),
                    };

                    // Resolve the path relative to current file
                    let resolved_path =
                        modules::resolve_import_path(&path, self.current_file.as_ref())?;

                    // Check if module is already cached
                    if let Some(cached_value) =
                        self.module_cache.borrow().get(&resolved_path).cloned()
                    {
                        self.stack.push(cached_value);
                    } else {
                        // Check for circular dependencies
                        if self.loading_modules.borrow().contains(&resolved_path) {
                            let mut cycle = self.loading_modules.borrow().clone();
                            cycle.push(resolved_path.clone());
                            return Err(self._error(format!(
                                "Circular dependency detected: {}",
                                cycle.join(" -> ")
                            )));
                        }

                        // Load and evaluate the module
                        let module_value = modules::load_module(self, &resolved_path)?;
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
}

fn flip_bool(stack: &mut Vec<Value>) -> Result<(), VmError> {
    let v = stack
        .pop()
        .ok_or_else(|| VmError::runtime("flip bool underflow".into()))?;
    match v {
        Value::Boolean(b) => {
            stack.push(Value::Boolean(!b));
            Ok(())
        }
        _ => Err(VmError::runtime("flip bool type error".into())),
    }
}

fn truthy(v: &Value) -> bool {
    !matches!(v, Value::Boolean(false) | Value::Null)
}
