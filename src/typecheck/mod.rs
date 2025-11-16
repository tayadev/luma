use crate::ast::Program;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

pub type TypecheckResult<T> = Result<T, Vec<TypeError>>;

pub fn typecheck_program(_program: &Program) -> TypecheckResult<()> {
    // MVP stub: accept everything; fill in later with real checks
    Ok(())
}
