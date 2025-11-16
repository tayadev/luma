use crate::ast::{Program, Stmt, Expr, BinaryOp, UnaryOp, LogicalOp, AssignOp};
use super::ir::{Chunk, Instruction, Constant};

pub fn compile_program(program: &Program) -> Chunk {
    let mut c = Compiler::new("<program>");
    for stmt in &program.statements {
        c.emit_stmt(stmt);
    }
    c.chunk.instructions.push(Instruction::Halt);
    c.chunk.clone()
}

fn push_const(chunk: &mut Chunk, c: Constant) -> usize {
    chunk.constants.push(c);
    chunk.constants.len() - 1
}

struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self { chunk: Chunk { name: name.to_string(), ..Default::default() } }
    }

    fn emit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Return(expr) => {
                self.emit_expr(expr);
            }
            Stmt::If { condition, then_block, elif_blocks, else_block } => {
                // if cond then ... elif ... else ... end
                self.emit_expr(condition);
                let jf_main = self.emit_jump_if_false();

                // then block
                for st in then_block { self.emit_stmt(st); }
                let j_end = self.emit_jump();

                // else/elif chain
                let mut next_start = self.current_ip();
                self.patch_jump(jf_main, next_start);

                // elif blocks
                for (elif_cond, elif_body) in elif_blocks {
                    self.emit_expr(elif_cond);
                    let jf_elif = self.emit_jump_if_false();
                    for st in elif_body { self.emit_stmt(st); }
                    let _j_after_elif = self.emit_jump();
                    // patch jf_elif to the next block start
                    next_start = self.current_ip();
                    self.patch_jump(jf_elif, next_start);
                    // leave j_after_elif as placeholder to be patched to end later
                }

                // else block
                if let Some(else_body) = else_block {
                    for st in else_body { self.emit_stmt(st); }
                }

                // Patch final end for main then and all elif end jumps
                let end_ip = self.current_ip();
                self.patch_jump(j_end, end_ip);
                // Also patch any Jump(usize::MAX) left from elif bodies to end_ip
                // Walk back and patch immediate previous Jump placeholders
                for instr in &mut self.chunk.instructions {
                    if let Instruction::Jump(addr) = instr { if *addr == usize::MAX { *addr = end_ip; } }
                }
            }
            Stmt::While { condition, body } => {
                let loop_start = self.current_ip();
                self.emit_expr(condition);
                let jf_end = self.emit_jump_if_false();
                for st in body { self.emit_stmt(st); }
                // jump back to loop start
                let start = loop_start;
                self.chunk.instructions.push(Instruction::Jump(start));
                let end_ip = self.current_ip();
                self.patch_jump(jf_end, end_ip);
            }
            Stmt::VarDecl { name, value, .. } => {
                // globals-only MVP
                self.emit_expr(value);
                let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
            }
            Stmt::Assignment { target, op, value } => {
                if let Expr::Identifier(name) = target {
                    match op {
                        AssignOp::Assign => {
                            self.emit_expr(value);
                        }
                        AssignOp::AddAssign => {
                            self.emit_expr(&Expr::Identifier(name.clone()));
                            self.emit_expr(value);
                            self.chunk.instructions.push(Instruction::Add);
                        }
                        AssignOp::SubAssign => {
                            self.emit_expr(&Expr::Identifier(name.clone()));
                            self.emit_expr(value);
                            self.chunk.instructions.push(Instruction::Sub);
                        }
                        AssignOp::MulAssign => {
                            self.emit_expr(&Expr::Identifier(name.clone()));
                            self.emit_expr(value);
                            self.chunk.instructions.push(Instruction::Mul);
                        }
                        AssignOp::DivAssign => {
                            self.emit_expr(&Expr::Identifier(name.clone()));
                            self.emit_expr(value);
                            self.chunk.instructions.push(Instruction::Div);
                        }
                        AssignOp::ModAssign => {
                            self.emit_expr(&Expr::Identifier(name.clone()));
                            self.emit_expr(value);
                            self.chunk.instructions.push(Instruction::Mod);
                        }
                    }
                    let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                    self.chunk.instructions.push(Instruction::SetGlobal(name_idx));
                } else {
                    // Other assignment targets not yet supported
                }
            }
            // TODO: other statements in MVP
            _ => {
                // For now, ignore non-return statements
            }
        }
    }

    fn emit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Number(n) => {
                let idx = push_const(&mut self.chunk, Constant::Number(*n));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::String(s) => {
                let idx = push_const(&mut self.chunk, Constant::String(s.clone()));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Boolean(b) => {
                let idx = push_const(&mut self.chunk, Constant::Boolean(*b));
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Null => {
                let idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(idx));
            }
            Expr::Array(items) => {
                for item in items { self.emit_expr(item); }
                self.chunk.instructions.push(Instruction::BuildArray(items.len()));
            }
            Expr::Table(fields) => {
                for (k, v) in fields {
                    let k_idx = push_const(&mut self.chunk, Constant::String(k.clone()));
                    self.chunk.instructions.push(Instruction::Const(k_idx));
                    self.emit_expr(v);
                }
                self.chunk.instructions.push(Instruction::BuildTable(fields.len()));
            }
            Expr::MemberAccess { object, member } => {
                self.emit_expr(object);
                let name_idx = push_const(&mut self.chunk, Constant::String(member.clone()));
                self.chunk.instructions.push(Instruction::GetProp(name_idx));
            }
            Expr::Index { object, index } => {
                self.emit_expr(object);
                self.emit_expr(index);
                self.chunk.instructions.push(Instruction::GetIndex);
            }
            Expr::Identifier(name) => {
                let name_idx = push_const(&mut self.chunk, Constant::String(name.clone()));
                self.chunk.instructions.push(Instruction::GetGlobal(name_idx));
            }
            Expr::Unary { op, operand } => {
                match op {
                    UnaryOp::Neg => {
                        // 0 - operand
                        let zero = push_const(&mut self.chunk, Constant::Number(0.0));
                        self.chunk.instructions.push(Instruction::Const(zero));
                        self.emit_expr(operand);
                        self.chunk.instructions.push(Instruction::Sub);
                    }
                    UnaryOp::Not => {
                        self.emit_expr(operand);
                        self.chunk.instructions.push(Instruction::Not);
                    }
                }
            }
            Expr::Logical { left, op, right } => {
                match op {
                    LogicalOp::And => {
                        // left && right with short-circuit
                        self.emit_expr(left);
                        self.chunk.instructions.push(Instruction::Dup);
                        let jf = self.emit_jump_if_false();
                        // left truthy: discard left and eval right
                        self.chunk.instructions.push(Instruction::Pop);
                        self.emit_expr(right);
                        let end = self.current_ip();
                        self.patch_jump(jf, end);
                    }
                    LogicalOp::Or => {
                        // left || right with short-circuit using only JUMP_IF_FALSE + JUMP
                        self.emit_expr(left);
                        self.chunk.instructions.push(Instruction::Dup);
                        let jf = self.emit_jump_if_false();
                        let jend = self.emit_jump();
                        // Evaluate right when left is falsey
                        let after_jf = self.current_ip();
                        self.patch_jump(jf, after_jf);
                        self.chunk.instructions.push(Instruction::Pop);
                        self.emit_expr(right);
                        let end = self.current_ip();
                        self.patch_jump(jend, end);
                    }
                }
            }
            Expr::Binary { left, op, right } => {
                self.emit_expr(left);
                self.emit_expr(right);
                match op {
                    BinaryOp::Add => self.chunk.instructions.push(Instruction::Add),
                    BinaryOp::Sub => self.chunk.instructions.push(Instruction::Sub),
                    BinaryOp::Mul => self.chunk.instructions.push(Instruction::Mul),
                    BinaryOp::Div => self.chunk.instructions.push(Instruction::Div),
                    BinaryOp::Mod => self.chunk.instructions.push(Instruction::Mod),
                    BinaryOp::Eq => self.chunk.instructions.push(Instruction::Eq),
                    BinaryOp::Ne => self.chunk.instructions.push(Instruction::Ne),
                    BinaryOp::Lt => self.chunk.instructions.push(Instruction::Lt),
                    BinaryOp::Le => self.chunk.instructions.push(Instruction::Le),
                    BinaryOp::Gt => self.chunk.instructions.push(Instruction::Gt),
                    BinaryOp::Ge => self.chunk.instructions.push(Instruction::Ge),
                }
            }
            // Other expressions not yet supported in step-1
            _ => {
                let idx = push_const(&mut self.chunk, Constant::Null);
                self.chunk.instructions.push(Instruction::Const(idx));
            }
        }
    }

    fn emit_jump_if_false(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk.instructions.push(Instruction::JumpIfFalse(usize::MAX));
        pos
    }
    fn emit_jump(&mut self) -> usize {
        let pos = self.chunk.instructions.len();
        self.chunk.instructions.push(Instruction::Jump(usize::MAX));
        pos
    }
    fn patch_jump(&mut self, at: usize, target: usize) {
        match self.chunk.instructions.get_mut(at) {
            Some(Instruction::JumpIfFalse(addr)) => *addr = target,
            Some(Instruction::Jump(addr)) => *addr = target,
            _ => {}
        }
    }
    fn current_ip(&self) -> usize { self.chunk.instructions.len() }
}
