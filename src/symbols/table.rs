use super::constant::Constant;
use super::function::Function;
use super::r#type::Type;
use super::variable::Variable;
use crate::ast::Ast;

trait Symbol {}

#[derive(Debug, Default)]
pub struct SymbolTable {
    types: Vec<Type>,
    constants: Vec<Constant>,
    variables: Vec<Variable>,
    function: Vec<Function>,
}

impl SymbolTable {
    pub fn new(source_code: &str, ast: Ast) -> SymbolTable {
        SymbolTable::default()
    }
}
