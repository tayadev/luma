//! Instruction execution logic for the VM
//!
//! This module contains the main instruction dispatch loop and handlers for
//! all bytecode instructions.

use super::errors::VmError;
use super::frames::CallFrame;
use super::interpreter::VM;
use super::stack::{flip_bool, truthy};
use super::value::{Upvalue, Value};
use super::{modules, operators};
use crate::bytecode::ir::{Constant, Instruction, UpvalueDescriptor};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl VM {
    /// Execute bytecode instructions until completion
    pub fn execute(&mut self) -> Result<Value, VmError> {
        loop {
            if self.ip >= self.chunk.instructions.len() {
                return Err(self._error("IP out of bounds".into()));
            }
            let instr = self.chunk.instructions[self.ip].clone();
            self.ip += 1;

            match instr {
                Instruction::Const(idx) => self.exec_const(idx)?,
                Instruction::Pop => self.exec_pop()?,
                Instruction::PopNPreserve(n) => self.exec_pop_n_preserve(n)?,
                Instruction::Dup => self.exec_dup()?,
                Instruction::Add => self.exec_add()?,
                Instruction::Sub => self.exec_sub()?,
                Instruction::Mul => self.exec_mul()?,
                Instruction::Div => self.exec_div()?,
                Instruction::Mod => self.exec_mod()?,
                Instruction::Neg => self.exec_neg()?,
                Instruction::GetGlobal(idx) => self.exec_get_global(idx)?,
                Instruction::SetGlobal(idx) => self.exec_set_global(idx)?,
                Instruction::BuildList(n) => self.exec_build_list(n)?,
                Instruction::BuildTable(n) => self.exec_build_table(n)?,
                Instruction::GetIndex => self.exec_get_index()?,
                Instruction::GetProp(idx) => self.exec_get_prop(idx)?,
                Instruction::GetLen => self.exec_get_len()?,
                Instruction::SetIndex => self.exec_set_index()?,
                Instruction::SetProp(idx) => self.exec_set_prop(idx)?,
                Instruction::GetLocal(slot) => self.exec_get_local(slot)?,
                Instruction::SetLocal(slot) => self.exec_set_local(slot)?,
                Instruction::SliceList(start_index) => self.exec_slice_list(start_index)?,
                Instruction::Eq => self.exec_eq()?,
                Instruction::Ne => self.exec_ne()?,
                Instruction::Lt => self.exec_lt()?,
                Instruction::Le => self.exec_le()?,
                Instruction::Gt => self.exec_gt()?,
                Instruction::Ge => self.exec_ge()?,
                Instruction::Not => self.exec_not()?,
                Instruction::Jump(target) => self.exec_jump(target)?,
                Instruction::JumpIfFalse(target) => self.exec_jump_if_false(target)?,
                Instruction::MakeFunction(idx) => self.exec_make_function(idx)?,
                Instruction::Closure(idx) => self.exec_closure(idx)?,
                Instruction::GetUpvalue(idx) => self.exec_get_upvalue(idx)?,
                Instruction::SetUpvalue(idx) => self.exec_set_upvalue(idx)?,
                Instruction::Call(arity) => self.exec_call(arity)?,
                Instruction::Return => {
                    if let Some(ret_val) = self.exec_return()? {
                        return Ok(ret_val);
                    }
                }
                Instruction::Import => self.exec_import()?,
                Instruction::Halt => return Ok(self.stack.pop().unwrap_or(Value::Null)),
            }
        }
    }

    // Stack operations
    fn exec_const(&mut self, idx: usize) -> Result<(), VmError> {
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
        Ok(())
    }

    fn exec_pop(&mut self) -> Result<(), VmError> {
        self.stack.pop();
        Ok(())
    }

    fn exec_pop_n_preserve(&mut self, n: usize) -> Result<(), VmError> {
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
        Ok(())
    }

    fn exec_dup(&mut self) -> Result<(), VmError> {
        if let Some(v) = self.stack.last().cloned() {
            self.stack.push(v);
            Ok(())
        } else {
            Err(self._error("DUP on empty stack".into()))
        }
    }

    // Arithmetic operations
    fn exec_add(&mut self) -> Result<(), VmError> {
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
            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{x}{y}"))),
            _ => Err("Type mismatch".to_string()),
        })
    }

    fn exec_sub(&mut self) -> Result<(), VmError> {
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
        })
    }

    fn exec_mul(&mut self) -> Result<(), VmError> {
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
        })
    }

    fn exec_div(&mut self) -> Result<(), VmError> {
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
        })
    }

    fn exec_mod(&mut self) -> Result<(), VmError> {
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
        })
    }

    fn exec_neg(&mut self) -> Result<(), VmError> {
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("NEG underflow".into()))?;

        match &a {
            Value::Number(x) => {
                self.stack.push(Value::Number(-x));
                Ok(())
            }
            _ => {
                // Try operator overloading for __neg
                if let Some(method) = operators::has_method(&a, "__neg") {
                    operators::call_overload_method(self, method, vec![a], 1, "__neg")
                } else {
                    Err(self._error("NEG requires Number or __neg method".into()))
                }
            }
        }
    }

    // Global variable operations
    fn exec_get_global(&mut self, idx: usize) -> Result<(), VmError> {
        let name = match self.chunk.constants.get(idx) {
            Some(Constant::String(s)) => s.clone(),
            _ => {
                return Err(self._error("GET_GLOBAL expects string constant".into()));
            }
        };
        if let Some(v) = self.globals.get(&name).cloned() {
            self.stack.push(v);
            Ok(())
        } else {
            Err(self._error(format!("Undefined global '{name}'")))
        }
    }

    fn exec_set_global(&mut self, idx: usize) -> Result<(), VmError> {
        let name = match self.chunk.constants.get(idx) {
            Some(Constant::String(s)) => s.clone(),
            _ => {
                return Err(self._error("SET_GLOBAL expects string constant".into()));
            }
        };
        let v = self
            .stack
            .pop()
            .ok_or_else(|| self._error("SET_GLOBAL pop underflow".into()))?;
        self.globals.insert(name, v);
        Ok(())
    }

    // Collection operations
    fn exec_build_list(&mut self, n: usize) -> Result<(), VmError> {
        if self.stack.len() < n {
            return Err(self._error("BUILD_LIST underflow".into()));
        }
        let mut tmp = Vec::with_capacity(n);
        for _ in 0..n {
            tmp.push(self.stack.pop().unwrap());
        }
        tmp.reverse();
        self.stack.push(Value::List(Rc::new(RefCell::new(tmp))));
        Ok(())
    }

    fn exec_build_table(&mut self, n: usize) -> Result<(), VmError> {
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
        Ok(())
    }

    fn exec_get_index(&mut self) -> Result<(), VmError> {
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
                    Some(v) => {
                        self.stack.push(v.clone());
                        Ok(())
                    }
                    None => Err(self._error("List index out of bounds".into())),
                }
            }
            (Value::Table(map), Value::String(k)) => {
                let borrowed = map.borrow();
                match borrowed.get(&k) {
                    Some(v) => {
                        self.stack.push(v.clone());
                        Ok(())
                    }
                    None => Err(self._error("Table key not found".into())),
                }
            }
            _ => Err(self._error("GET_INDEX type error".into())),
        }
    }

    fn exec_get_prop(&mut self, idx: usize) -> Result<(), VmError> {
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
                    Some(v) => {
                        self.stack.push(v.clone());
                        Ok(())
                    }
                    None => Err(self._error("Property not found".into())),
                }
            }
            _ => Err(self._error("GET_PROP on non-table".into())),
        }
    }

    fn exec_get_len(&mut self) -> Result<(), VmError> {
        let obj = self
            .stack
            .pop()
            .ok_or_else(|| self._error("GET_LEN obj underflow".into()))?;
        match obj {
            Value::List(arr) => {
                let borrowed = arr.borrow();
                self.stack.push(Value::Number(borrowed.len() as f64));
                Ok(())
            }
            Value::Table(map) => {
                let borrowed = map.borrow();
                self.stack.push(Value::Number(borrowed.len() as f64));
                Ok(())
            }
            Value::String(s) => {
                self.stack.push(Value::Number(s.len() as f64));
                Ok(())
            }
            _ => Err(self._error("GET_LEN requires list, table, or string".into())),
        }
    }

    fn exec_set_index(&mut self) -> Result<(), VmError> {
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
                    borrowed.push(value);
                } else if i < borrowed.len() {
                    borrowed[i] = value;
                } else {
                    return Err(self._error("List index out of bounds".into()));
                }
                Ok(())
            }
            (Value::Table(map), Value::String(k)) => {
                let mut borrowed = map.borrow_mut();
                borrowed.insert(k, value);
                Ok(())
            }
            _ => Err(self._error("SET_INDEX type error".into())),
        }
    }

    fn exec_set_prop(&mut self, idx: usize) -> Result<(), VmError> {
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
                Ok(())
            }
            _ => Err(self._error("SET_PROP on non-table".into())),
        }
    }

    // Local variable operations
    fn exec_get_local(&mut self, slot: usize) -> Result<(), VmError> {
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
        Ok(())
    }

    fn exec_set_local(&mut self, slot: usize) -> Result<(), VmError> {
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
        Ok(())
    }

    fn exec_slice_list(&mut self, start_index: usize) -> Result<(), VmError> {
        let arr = self
            .stack
            .pop()
            .ok_or_else(|| self._error("SLICE_LIST pop underflow".into()))?;
        match arr {
            Value::List(arr_ref) => {
                let borrowed = arr_ref.borrow();
                let len = borrowed.len();
                let slice_start = start_index.min(len);
                let sliced: Vec<Value> = borrowed[slice_start..].to_vec();
                self.stack.push(Value::List(Rc::new(RefCell::new(sliced))));
                Ok(())
            }
            _ => Err(self._error("SLICE_LIST requires a list".into())),
        }
    }

    // Comparison operations
    fn exec_eq(&mut self) -> Result<(), VmError> {
        let b = self
            .stack
            .pop()
            .ok_or_else(|| self._error("EQ right underflow".into()))?;
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("EQ left underflow".into()))?;
        operators::execute_eq_op(self, a, b)
    }

    fn exec_ne(&mut self) -> Result<(), VmError> {
        let b = self
            .stack
            .pop()
            .ok_or_else(|| self._error("NE right underflow".into()))?;
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("NE left underflow".into()))?;
        operators::execute_eq_op(self, a, b)?;
        flip_bool(&mut self.stack)
    }

    fn exec_lt(&mut self) -> Result<(), VmError> {
        let b = self
            .stack
            .pop()
            .ok_or_else(|| self._error("LT right underflow".into()))?;
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("LT left underflow".into()))?;
        operators::execute_cmp_op(self, a, b, "__lt", |a, b| a < b)
    }

    fn exec_le(&mut self) -> Result<(), VmError> {
        let b = self
            .stack
            .pop()
            .ok_or_else(|| self._error("LE right underflow".into()))?;
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("LE left underflow".into()))?;
        operators::execute_cmp_op(self, a, b, "__le", |a, b| a <= b)
    }

    fn exec_gt(&mut self) -> Result<(), VmError> {
        let b = self
            .stack
            .pop()
            .ok_or_else(|| self._error("GT right underflow".into()))?;
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("GT left underflow".into()))?;
        operators::execute_cmp_op(self, a, b, "__gt", |a, b| a > b)
    }

    fn exec_ge(&mut self) -> Result<(), VmError> {
        let b = self
            .stack
            .pop()
            .ok_or_else(|| self._error("GE right underflow".into()))?;
        let a = self
            .stack
            .pop()
            .ok_or_else(|| self._error("GE left underflow".into()))?;
        operators::execute_cmp_op(self, a, b, "__ge", |a, b| a >= b)
    }

    fn exec_not(&mut self) -> Result<(), VmError> {
        let v = self
            .stack
            .pop()
            .ok_or_else(|| self._error("NOT on empty stack".into()))?;
        self.stack.push(Value::Boolean(!truthy(&v)));
        Ok(())
    }

    // Control flow operations
    fn exec_jump(&mut self, target: usize) -> Result<(), VmError> {
        self.ip = target;
        Ok(())
    }

    fn exec_jump_if_false(&mut self, target: usize) -> Result<(), VmError> {
        let v = self
            .stack
            .pop()
            .ok_or_else(|| self._error("JUMP_IF_FALSE pop underflow".into()))?;
        if !truthy(&v) {
            self.ip = target;
        }
        Ok(())
    }

    // Function operations
    fn exec_make_function(&mut self, idx: usize) -> Result<(), VmError> {
        let v = match self.chunk.constants.get(idx) {
            Some(Constant::Function(chunk)) => Value::Function {
                chunk: chunk.clone(),
                arity: chunk.local_count as usize,
            },
            _ => {
                return Err(self._error("MAKE_FUNCTION expects function constant".into()));
            }
        };
        self.stack.push(v);
        Ok(())
    }

    fn exec_closure(&mut self, idx: usize) -> Result<(), VmError> {
        let chunk = match self.chunk.constants.get(idx) {
            Some(Constant::Function(chunk)) => chunk.clone(),
            _ => {
                return Err(self._error("CLOSURE expects function constant".into()));
            }
        };

        let mut upvalues = Vec::new();
        for descriptor in &chunk.upvalue_descriptors {
            let upvalue = match descriptor {
                UpvalueDescriptor::Local(slot) => {
                    let abs = self.base + slot;
                    if let Some(cell) = self.captured_locals.get(&abs) {
                        cell.clone()
                    } else {
                        let value = self
                            .stack
                            .get(abs)
                            .ok_or_else(|| {
                                self._error(format!(
                                    "Upvalue capture: local slot {slot} out of bounds"
                                ))
                            })?
                            .clone();
                        let cell = Upvalue::new(value);
                        self.captured_locals.insert(abs, cell.clone());
                        cell
                    }
                }
                UpvalueDescriptor::Upvalue(upvalue_idx) => self
                    .upvalues
                    .get(*upvalue_idx)
                    .ok_or_else(|| {
                        self._error(format!(
                            "Upvalue capture: upvalue {upvalue_idx} out of bounds"
                        ))
                    })?
                    .clone(),
            };
            upvalues.push(upvalue);
        }

        let closure = Value::Closure {
            chunk: chunk.clone(),
            arity: chunk.local_count as usize,
            upvalues,
        };
        self.stack.push(closure);
        Ok(())
    }

    fn exec_get_upvalue(&mut self, idx: usize) -> Result<(), VmError> {
        let upvalue = self
            .upvalues
            .get(idx)
            .ok_or_else(|| self._error(format!("GetUpvalue: index {idx} out of bounds")))?;
        let value = upvalue.value.borrow().clone();
        self.stack.push(value);
        Ok(())
    }

    fn exec_set_upvalue(&mut self, idx: usize) -> Result<(), VmError> {
        let value = self
            .stack
            .pop()
            .ok_or_else(|| self._error("SetUpvalue: stack underflow".into()))?;
        let upvalue = self
            .upvalues
            .get(idx)
            .ok_or_else(|| self._error(format!("SetUpvalue: index {idx} out of bounds")))?;
        *upvalue.value.borrow_mut() = value;
        Ok(())
    }

    fn exec_call(&mut self, arity: usize) -> Result<(), VmError> {
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
                    return Err(
                        self._error(format!("Arity mismatch: expected {fn_arity}, got {arity}"))
                    );
                }
                let frame = CallFrame {
                    chunk: self.chunk.clone(),
                    ip: self.ip,
                    base: self.base,
                    upvalues: self.upvalues.clone(),
                    captured_locals: std::mem::take(&mut self.captured_locals),
                };
                self.frames.push(frame);

                self.base = callee_idx + 1;
                self.chunk = fn_chunk;
                self.ip = 0;
                self.upvalues = Vec::new();
                self.captured_locals = HashMap::new();
                Ok(())
            }
            Value::Closure {
                chunk: fn_chunk,
                arity: fn_arity,
                upvalues: fn_upvalues,
            } => {
                if arity != fn_arity {
                    return Err(
                        self._error(format!("Arity mismatch: expected {fn_arity}, got {arity}"))
                    );
                }
                let frame = CallFrame {
                    chunk: self.chunk.clone(),
                    ip: self.ip,
                    base: self.base,
                    upvalues: self.upvalues.clone(),
                    captured_locals: std::mem::take(&mut self.captured_locals),
                };
                self.frames.push(frame);

                self.base = callee_idx + 1;
                self.chunk = fn_chunk;
                self.ip = 0;
                self.upvalues = fn_upvalues;
                self.captured_locals = HashMap::new();
                Ok(())
            }
            Value::NativeFunction {
                name,
                arity: fn_arity,
            } => {
                // FFI functions have variable arity, skip arity check for them
                let is_ffi_dispatch = name.starts_with("ffi.")
                    && ![
                        "ffi.def",
                        "ffi.new_cstr",
                        "ffi.nullptr",
                        "ffi.is_null",
                        "ffi.call",
                        "ffi.free_cstr",
                    ]
                    .contains(&name.as_str());

                if name != "print" && !is_ffi_dispatch && arity != fn_arity {
                    return Err(
                        self._error(format!("Arity mismatch: expected {fn_arity}, got {arity}"))
                    );
                }
                let args: Vec<Value> = self.stack.drain(callee_idx + 1..).collect();
                self.stack.pop();

                if name == "into" {
                    self.exec_native_into(args)
                } else if is_ffi_dispatch {
                    // Dispatch to FFI function handler
                    let result = super::native::native_ffi_dispatch(&name, &args)
                        .map_err(|e| self._error(e))?;
                    self.stack.push(result);
                    Ok(())
                } else {
                    let func = self.native_functions.get(&name).ok_or_else(|| {
                        self._error(format!("Native function '{name}' not found"))
                    })?;
                    let result = func(&args).map_err(|e| self._error(e))?;
                    self.stack.push(result);
                    Ok(())
                }
            }
            _ => Err(self._error("CALL on non-function".into())),
        }
    }

    fn exec_native_into(&mut self, args: Vec<Value>) -> Result<(), VmError> {
        if args.len() != 2 {
            return Err(self._error(format!("into() expects 2 arguments, got {}", args.len())));
        }
        let value = args[0].clone();
        let target_type = args[1].clone();

        if let Some(method) = operators::has_method(&value, "__into") {
            operators::call_overload_method(self, method, vec![value, target_type], 2, "__into")
        } else {
            match target_type {
                Value::Type(tmap) | Value::Table(tmap) => {
                    let tb = tmap.borrow();
                    let is_string_target = tb.contains_key("String") || tb.is_empty();
                    if is_string_target {
                        let converted = match value {
                            Value::Number(n) => Value::String(n.to_string()),
                            Value::String(s) => Value::String(s),
                            Value::Boolean(b) => Value::String(b.to_string()),
                            Value::Null => Value::String("null".to_string()),
                            other => Value::String(format!("{other}")),
                        };
                        self.stack.push(converted);
                        Ok(())
                    } else {
                        Err(self._error(
                            "Type does not support conversion (no __into method)".to_string(),
                        ))
                    }
                }
                _ => Err(self._error("Second argument to into() must be a type".to_string())),
            }
        }
    }

    fn exec_return(&mut self) -> Result<Option<Value>, VmError> {
        let ret_val = self.stack.pop().unwrap_or(Value::Null);
        self.stack.truncate(self.base - 1);

        if let Some(frame) = self.frames.pop() {
            self.chunk = frame.chunk;
            self.ip = frame.ip;
            self.base = frame.base;
            self.upvalues = frame.upvalues;
            self.captured_locals = frame.captured_locals;
            self.stack.push(ret_val);
            Ok(None)
        } else {
            Ok(Some(ret_val))
        }
    }

    // Module operations
    fn exec_import(&mut self) -> Result<(), VmError> {
        let path_val = self
            .stack
            .pop()
            .ok_or_else(|| self._error("IMPORT requires path on stack".into()))?;
        let path = match path_val {
            Value::String(s) => s,
            _ => return Err(self._error("IMPORT requires String path".into())),
        };

        let resolved_path = modules::resolve_import_path(&path, self.current_file.as_ref())?;

        if let Some(cached_value) = self.module_cache.borrow().get(&resolved_path).cloned() {
            self.stack.push(cached_value);
        } else {
            if self.loading_modules.borrow().contains(&resolved_path) {
                let mut cycle = self.loading_modules.borrow().clone();
                cycle.push(resolved_path.clone());
                return Err(self._error(format!(
                    "Circular dependency detected: {}",
                    cycle.join(" -> ")
                )));
            }
            let module_value = modules::load_module(self, &resolved_path)?;
            self.stack.push(module_value);
        }
        Ok(())
    }
}
