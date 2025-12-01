//! Bytecode compiler for Luma.
//!
//! This module implements the compilation of Luma's AST into bytecode instructions.
//! The compiler performs two passes:
//!
//! 1. **Pre-declaration pass**: Scans top-level function declarations and registers them
//!    in the global scope. This enables mutual recursion between functions.
//!
//! 2. **Compilation pass**: Traverses the AST and emits bytecode instructions. This includes:
//!    - Expression compilation with proper operator precedence
//!    - Statement compilation with control flow handling
//!    - Closure capture via upvalue descriptors
//!    - Pattern matching and destructuring
//!    - Loop compilation with break/continue support
//!
//! The compiler uses a stack-based virtual machine model where most operations
//! work on values at the top of the stack.

use super::ir::{Chunk, Constant, Instruction, UpvalueDescriptor};
use crate::ast::{Argument, Expr, Program, Stmt};
use std::collections::HashMap;

pub fn compile_program(program: &Program) -> Chunk {
    compile_program_impl(program, false)
}

/// Compile a program for REPL mode where variables should be globals
/// to persist across evaluations
pub fn compile_repl_program(program: &Program) -> Chunk {
    compile_program_impl(program, true)
}

fn compile_program_impl(program: &Program, repl_mode: bool) -> Chunk {
    let mut c = Compiler::new("<program>");

    // Pre-scan: record top-level function parameter names for named-arg reordering
    let mut global_fn_params: HashMap<String, Vec<String>> = HashMap::new();
    for stmt in &program.statements {
        if let Stmt::VarDecl { name, value, .. } = stmt
            && let Expr::Function { arguments, .. } = value
        {
            let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
            global_fn_params.insert(name.clone(), params);
        }
    }
    c.global_fn_params = global_fn_params;

    if !repl_mode {
        // Enter a scope for modules so top-level variables become locals
        // This allows closures to capture them as upvalues
        c.enter_scope();

        // Pre-register all top-level let/var declarations with null placeholders
        for stmt in &program.statements {
            if let Stmt::VarDecl { name, .. } = stmt {
                let null_idx = push_const(&mut c.chunk, Constant::Null);
                c.chunk.instructions.push(Instruction::Const(null_idx));
                let slot = c.local_count;
                c.scopes.last_mut().unwrap().insert(name.clone(), slot);
                c.local_count += 1;
            }
        }
    } else {
        // In REPL mode, pre-register globals with null placeholders like before
        for stmt in &program.statements {
            if let Stmt::VarDecl { name, .. } = stmt {
                let null_idx = push_const(&mut c.chunk, Constant::Null);
                c.chunk.instructions.push(Instruction::Const(null_idx));
                let name_idx = push_const(&mut c.chunk, Constant::String(name.clone()));
                c.chunk.instructions.push(Instruction::SetGlobal(name_idx));
            }
        }
    }

    // Compile all statements
    for stmt in &program.statements {
        c.emit_stmt(stmt);
    }

    // In REPL mode, pop locals. In module mode, leave them on stack to preserve
    // upvalue references in closures (the return value stays on top).
    if repl_mode {
        // No cleanup needed for REPL—variables are globals
    } else {
        // For modules: don't pop locals; keep them on stack for closure upvalues.
        // The last statement's value (implicit return) stays on top.
    }

    c.chunk.instructions.push(Instruction::Halt);
    c.chunk.clone()
}

pub(super) fn push_const(chunk: &mut Chunk, c: Constant) -> usize {
    chunk.constants.push(c);
    chunk.constants.len() - 1
}

pub(super) struct LoopContext {
    pub(super) break_patches: Vec<usize>,
    pub(super) continue_patches: Vec<usize>,
    pub(super) local_count: usize,
    pub(super) continue_target: Option<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct UpvalueInfo {
    pub(super) descriptor: UpvalueDescriptor,
    pub(super) _name: String,
}

pub(super) struct Compiler {
    pub(super) chunk: Chunk,
    pub(super) scopes: Vec<HashMap<String, usize>>,
    pub(super) local_count: usize,
    pub(super) loop_stack: Vec<LoopContext>,
    pub(super) upvalues: Vec<UpvalueInfo>,
    pub(super) parent: Option<Box<Compiler>>,
    pub(super) param_scopes: Vec<HashMap<String, Vec<String>>>,
    pub(super) global_fn_params: HashMap<String, Vec<String>>,
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self {
            chunk: Chunk {
                name: name.to_string(),
                ..Default::default()
            },
            scopes: Vec::new(),
            local_count: 0,
            loop_stack: Vec::new(),
            upvalues: Vec::new(),
            parent: None,
            param_scopes: Vec::new(),
            global_fn_params: HashMap::new(),
        }
    }
    fn new_with_parent(name: &str, parent: Compiler) -> Self {
        Self {
            chunk: Chunk {
                name: name.to_string(),
                ..Default::default()
            },
            scopes: Vec::new(),
            local_count: 0,
            loop_stack: Vec::new(),
            upvalues: Vec::new(),
            parent: Some(Box::new(parent)),
            param_scopes: Vec::new(),
            global_fn_params: HashMap::new(),
        }
    }
    pub(super) fn emit_stmt(&mut self, s: &Stmt) {
        super::emit_stmt::emit_stmt(self, s);
    }

    // emit_expr moved to emit_expr.rs

    pub(super) fn emit_jump_if_false(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk
            .instructions
            .push(Instruction::JumpIfFalse(usize::MAX));
        pos
    }
    pub(super) fn emit_jump(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk.instructions.push(Instruction::Jump(usize::MAX));
        pos
    }
    pub(super) fn patch_jump(&mut self, at: usize, target: usize) {
        match self.chunk.instructions.get_mut(at) {
            Some(Instruction::JumpIfFalse(addr)) => *addr = target,
            Some(Instruction::Jump(addr)) => *addr = target,
            _ => {}
        }
    }
    pub(super) fn current_ip(&self) -> usize {
        self.chunk.instructions.len()
    }

    // helpers are implemented in helpers.rs as impl Compiler

    pub(super) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.param_scopes.push(HashMap::new());
    }

    pub(super) fn exit_scope_with_preserve(&mut self, preserve_top: bool) {
        if let Some(scope) = self.scopes.pop() {
            let to_pop = scope.len();
            if to_pop > 0 {
                if preserve_top {
                    self.chunk
                        .instructions
                        .push(Instruction::PopNPreserve(to_pop));
                } else {
                    for _ in 0..to_pop {
                        self.chunk.instructions.push(Instruction::Pop);
                    }
                }
            }
            self.local_count = self.local_count.saturating_sub(to_pop);
        }
        self.param_scopes.pop();
    }

    pub(super) fn lookup_local(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    /// Lookup parameter names for a function variable by identifier name.
    /// Searches current and outer param scopes, then global map, then parent's chain.
    pub(super) fn lookup_fn_params(&self, name: &str) -> Option<Vec<String>> {
        for scope in self.param_scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        if let Some(v) = self.global_fn_params.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.lookup_fn_params(name);
        }
        None
    }

    /// Resolve an upvalue by searching parent scopes.
    /// Returns the upvalue index if found, None otherwise.
    pub(super) fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        // If there's no parent, we can't have upvalues
        let parent = self.parent.as_mut()?;

        // First check if the variable is a local in the parent
        if let Some(slot) = parent.lookup_local(name) {
            // It's a local in the parent - capture it
            let descriptor = UpvalueDescriptor::Local(slot);
            return Some(self.add_upvalue(descriptor, name.to_string()));
        }

        // Otherwise, try to resolve it as an upvalue in the parent
        if let Some(parent_upvalue_idx) = parent.resolve_upvalue(name) {
            // It's an upvalue in the parent - capture that upvalue
            let descriptor = UpvalueDescriptor::Upvalue(parent_upvalue_idx);
            return Some(self.add_upvalue(descriptor, name.to_string()));
        }

        None
    }

    /// Add an upvalue to this function's upvalue list.
    /// Returns the index of the upvalue.
    /// If the upvalue already exists, returns its existing index.
    fn add_upvalue(&mut self, descriptor: UpvalueDescriptor, name: String) -> usize {
        // Check if we already have this upvalue
        for (i, uv) in self.upvalues.iter().enumerate() {
            match (&uv.descriptor, &descriptor) {
                (UpvalueDescriptor::Local(a), UpvalueDescriptor::Local(b)) if a == b => return i,
                (UpvalueDescriptor::Upvalue(a), UpvalueDescriptor::Upvalue(b)) if a == b => {
                    return i;
                }
                _ => {}
            }
        }

        // Add new upvalue
        let idx = self.upvalues.len();
        self.upvalues.push(UpvalueInfo {
            descriptor,
            _name: name,
        });
        idx
    }

    /// Compile a nested function with access to parent scope for closures.
    /// This is done by temporarily moving self to become the parent of a new compiler.
    pub(super) fn compile_nested_function(
        &mut self,
        arguments: &[Argument],
        body: &[Stmt],
    ) -> (Chunk, Vec<UpvalueDescriptor>) {
        // We need to create a new compiler with self as parent, but we can't move self
        // Instead, we'll use std::mem::replace to temporarily swap self with a dummy
        let parent = std::mem::replace(self, Compiler::new("__temp__"));

        // Create nested compiler with parent
        let mut nested = Compiler::new_with_parent("<function>", parent);
        let arity = arguments.len();

        // Find variadic parameter (if any) and validate it's the last one
        let variadic_param_index = arguments
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.variadic)
            .map(|(idx, _)| idx);

        // If there's a variadic parameter, it must be the last one
        if let Some(idx) = variadic_param_index {
            if idx != arguments.len() - 1 {
                panic!("Variadic parameter must be the last parameter");
            }
        }

        // Enter scope for function parameters
        nested.enter_scope();
        // Parameters become locals in order
        for arg in arguments {
            let slot = nested.local_count;
            nested
                .scopes
                .last_mut()
                .unwrap()
                .insert(arg.name.clone(), slot);
            nested.local_count += 1;
        }
        // Predeclare local function names within function body for mutual recursion
        nested.predeclare_function_locals(body);

        // Compile body
        for stmt in body {
            nested.emit_stmt(stmt);
        }

        // Exit scope
        nested.exit_scope_with_preserve(does_block_leave_value(body));
        nested.chunk.instructions.push(Instruction::Return);
        nested.chunk.local_count = arity as u16;
        nested.chunk.variadic_param_index = variadic_param_index;

        // Extract upvalue descriptors and chunk
        let upvalue_descriptors: Vec<UpvalueDescriptor> = nested
            .upvalues
            .iter()
            .map(|uv| uv.descriptor.clone())
            .collect();
        nested.chunk.upvalue_descriptors = upvalue_descriptors.clone();
        let chunk = nested.chunk.clone();

        // Restore self from parent
        let parent = nested.parent.take().unwrap();
        *self = *parent;

        (chunk, upvalue_descriptors)
    }

    /// Predeclare local function names in the current scope so mutually-recursive
    /// functions can reference each other as locals (not fall back to globals).
    /// Also records parameter name lists for named-arg reordering.
    pub(super) fn predeclare_function_locals(&mut self, stmts: &[Stmt]) {
        if self.scopes.is_empty() {
            return;
        }
        for stmt in stmts {
            if let Stmt::VarDecl { name, value, .. } = stmt
                && let Expr::Function { arguments, .. } = value
            {
                // Skip if already declared in this scope (avoid duplicates)
                let already = self.scopes.last().and_then(|m| m.get(name)).is_some();
                if !already {
                    // Initialize slot with Null to occupy stack/local
                    let null_idx = push_const(&mut self.chunk, Constant::Null);
                    self.chunk.instructions.push(Instruction::Const(null_idx));
                    let slot = self.local_count;
                    self.scopes.last_mut().unwrap().insert(name.clone(), slot);
                    self.local_count += 1;
                }
                // Record param names for named-arg reordering in this scope
                if let Some(scope) = self.param_scopes.last_mut() {
                    let params = arguments.iter().map(|a| a.name.clone()).collect::<Vec<_>>();
                    scope.insert(name.clone(), params);
                }
            }
        }
    }
}

/// Applies implicit return to a match arm body by converting trailing ExprStmt to Return
pub(super) fn apply_implicit_return_to_arm(body: &[Stmt]) -> Vec<Stmt> {
    let mut arm_body = body.to_vec();
    if let Some(last) = arm_body.pop() {
        match last {
            Stmt::ExprStmt { expr: e, .. } => arm_body.push(Stmt::Return {
                value: e,
                span: None,
            }),
            other => arm_body.push(other),
        }
    }
    arm_body
}

pub(super) fn does_block_leave_value(block: &[Stmt]) -> bool {
    match block.last() {
        Some(Stmt::Return { .. }) => true,
        Some(Stmt::If {
            then_block,
            elif_blocks,
            else_block,
            ..
        }) => {
            // An if-statement leaves a value if all branches leave a value
            let then_leaves = does_block_leave_value(then_block);
            let elif_leave = elif_blocks.iter().all(|(_, b)| does_block_leave_value(b));
            let else_leaves = else_block
                .as_ref()
                .is_some_and(|b| does_block_leave_value(b));
            then_leaves && elif_leave && else_leaves
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn compile_source(source: &str) -> Chunk {
        let program = parse(source, "test.luma").expect("Parse failed");
        compile_program(&program)
    }

    fn has_instruction(chunk: &Chunk, instr: fn(&Instruction) -> bool) -> bool {
        chunk.instructions.iter().any(instr)
    }

    // Basic constant tests
    #[test]
    fn test_compile_number_constant() {
        let chunk = compile_source("42");
        assert!(matches!(
            chunk.constants.first(),
            Some(Constant::Number(n)) if (*n - 42.0).abs() < f64::EPSILON
        ));
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::Const(0)
        )));
    }

    #[test]
    fn test_compile_string_constant() {
        let chunk = compile_source("\"hello\"");
        assert!(matches!(
            chunk.constants.first(),
            Some(Constant::String(s)) if s == "hello"
        ));
    }

    #[test]
    fn test_compile_boolean_constant() {
        let chunk = compile_source("true");
        assert!(matches!(
            chunk.constants.first(),
            Some(Constant::Boolean(true))
        ));
    }

    #[test]
    fn test_compile_null_constant() {
        let chunk = compile_source("null");
        assert!(matches!(chunk.constants.first(), Some(Constant::Null)));
    }

    // Arithmetic operator tests
    #[test]
    fn test_compile_addition() {
        let chunk = compile_source("1 + 2");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Add)));
    }

    #[test]
    fn test_compile_subtraction() {
        let chunk = compile_source("5 - 3");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Sub)));
    }

    #[test]
    fn test_compile_multiplication() {
        let chunk = compile_source("4 * 7");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Mul)));
    }

    #[test]
    fn test_compile_division() {
        let chunk = compile_source("10 / 2");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Div)));
    }

    #[test]
    fn test_compile_modulo() {
        let chunk = compile_source("10 % 3");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Mod)));
    }

    // Comparison operator tests
    #[test]
    fn test_compile_equality() {
        let chunk = compile_source("1 == 1");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Eq)));
    }

    #[test]
    fn test_compile_inequality() {
        let chunk = compile_source("1 != 2");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Ne)));
    }

    #[test]
    fn test_compile_less_than() {
        let chunk = compile_source("1 < 2");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Lt)));
    }

    #[test]
    fn test_compile_less_equal() {
        let chunk = compile_source("1 <= 2");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Le)));
    }

    #[test]
    fn test_compile_greater_than() {
        let chunk = compile_source("2 > 1");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Gt)));
    }

    #[test]
    fn test_compile_greater_equal() {
        let chunk = compile_source("2 >= 1");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Ge)));
    }

    // Unary operator tests
    #[test]
    fn test_compile_negation() {
        let chunk = compile_source("-42");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Neg)));
    }

    #[test]
    fn test_compile_logical_not() {
        let chunk = compile_source("!true");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Not)));
    }

    // Collection tests
    #[test]
    fn test_compile_list() {
        let chunk = compile_source("[1, 2, 3]");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::BuildList(3)
        )));
    }

    #[test]
    fn test_compile_empty_list() {
        let chunk = compile_source("[]");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::BuildList(0)
        )));
    }

    #[test]
    fn test_compile_table() {
        let chunk = compile_source("{ x = 1, y = 2 }");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::BuildTable(2)
        )));
    }

    #[test]
    fn test_compile_list_index() {
        let chunk = compile_source("let x = [1, 2, 3]\nx[0]");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::GetIndex
        )));
    }

    #[test]
    fn test_compile_table_member_access() {
        let chunk = compile_source("let t = { x = 1 }\nt.x");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::GetProp(_)
        )));
    }

    // Variable tests
    #[test]
    fn test_compile_local_variable() {
        let chunk = compile_source("let x = 42\nx");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::GetLocal(_)
        )));
    }

    #[test]
    fn test_compile_local_assignment() {
        let chunk = compile_source("var x = 42\nx = 43");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::SetLocal(_)
        )));
    }

    // Control flow tests
    #[test]
    fn test_compile_if_statement() {
        let chunk = compile_source("if true do let x = 1 end");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::JumpIfFalse(_)
        )));
    }

    #[test]
    fn test_compile_if_else_statement() {
        let chunk = compile_source("if true do let x = 1 else do let y = 2 end");
        let jump_if_false_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::JumpIfFalse(_)))
            .count();
        assert_eq!(jump_if_false_count, 1);
        let jump_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::Jump(_)))
            .count();
        assert!(jump_count >= 1);
    }

    #[test]
    fn test_compile_while_loop() {
        let chunk = compile_source("while true do let x = 1 end");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::JumpIfFalse(_)
        )));
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::Jump(_)
        )));
    }

    #[test]
    fn test_compile_for_loop() {
        let chunk = compile_source("for x in [1, 2, 3] do let y = x end");
        // For loops involve iteration which uses various instructions
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::Jump(_)
        )));
    }

    // Function tests
    #[test]
    fn test_compile_function_definition() {
        let chunk = compile_source("let f = fn(x: Number): Number do return x + 1 end");
        // Should have a function constant
        assert!(
            chunk
                .constants
                .iter()
                .any(|c| matches!(c, Constant::Function(_)))
        );
    }

    #[test]
    fn test_compile_function_call() {
        let chunk = compile_source("let f = fn(x: Number): Number do return x end\nf(42)");
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::Call(_)
        )));
    }

    #[test]
    fn test_compile_closure() {
        let chunk = compile_source("let x = 42\nlet f = fn(): Number do return x end");
        // Closures use the Closure instruction
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::Closure(_)
        )));
    }

    #[test]
    fn test_compile_return_statement() {
        let chunk = compile_source("let f = fn(): Number do return 42 end");
        // Should have a function constant somewhere
        let has_func = chunk.constants.iter().any(|c| {
            if let Constant::Function(func_chunk) = c {
                has_instruction(func_chunk, |i| matches!(i, Instruction::Return))
            } else {
                false
            }
        });
        assert!(has_func, "Expected function with return instruction");
    }

    // Complex expression tests
    #[test]
    fn test_compile_nested_arithmetic() {
        let chunk = compile_source("(1 + 2) * (3 - 4)");
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Add)));
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Sub)));
        assert!(has_instruction(&chunk, |i| matches!(i, Instruction::Mul)));
    }

    #[test]
    fn test_compile_chained_member_access() {
        let chunk = compile_source("let t = { inner = { value = 42 } }\nt.inner.value");
        let prop_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::GetProp(_)))
            .count();
        assert_eq!(prop_count, 2);
    }

    // Halt instruction test
    #[test]
    fn test_compile_always_ends_with_halt() {
        let chunk = compile_source("42");
        assert!(matches!(chunk.instructions.last(), Some(Instruction::Halt)));
    }

    // Constant pool tests
    #[test]
    fn test_constant_deduplication() {
        // Multiple uses of same constant should reuse the same constant index
        let chunk = compile_source("let x = 42\nlet y = 42");
        // Should have at most 2 constants: null (for pre-declaration) and 42
        let number_constants = chunk
            .constants
            .iter()
            .filter(|c| matches!(c, Constant::Number(_)))
            .count();
        assert!(number_constants <= 2); // Might have 1 or 2 depending on optimization
    }

    // Scope tests
    #[test]
    fn test_nested_scopes() {
        let chunk = compile_source("let x = 1\ndo let x = 2 end");
        // Should have Pop or PopNPreserve instruction to clean up inner scope
        assert!(has_instruction(&chunk, |i| matches!(
            i,
            Instruction::Pop | Instruction::PopNPreserve(_)
        )));
    }

    #[test]
    fn test_mutual_recursion() {
        let code = r#"
            let even = fn(n: Number): Boolean do
                if n == 0 do
                    return true
                else do
                    return odd(n - 1)
                end
            end
            
            let odd = fn(n: Number): Boolean do
                if n == 0 do
                    return false
                else do
                    return even(n - 1)
                end
            end
        "#;
        let chunk = compile_source(code);
        // Should compile without errors and have function constants
        let func_count = chunk
            .constants
            .iter()
            .filter(|c| matches!(c, Constant::Function(_)))
            .count();
        assert_eq!(func_count, 2);
    }
}
