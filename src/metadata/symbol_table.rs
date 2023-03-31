use crate::metadata::ast::{Ast, NodeKind, TypeDecType, Visitable};
use crate::metadata::types::Type;
use indextree::{Arena, NodeId};
use std::{error, fmt};
use tower_lsp::lsp_types::Range;

#[derive(Debug, Default)]
pub struct SymbolTable {
    arena: Arena<ScopeSymbolTable>,
    root_id: Option<NodeId>,
}

impl SymbolTable {
    pub fn new(ast: &Ast) -> SymbolTable {
        let mut table = SymbolTable::default();

        table.root_id = Some(table.parse_scope(ast.get_root_id(), ast));

        table
    }

    fn parse_scope(&mut self, scope_node_id: NodeId, ast: &Ast) -> NodeId {
        let table = ScopeSymbolTable {
            range: ast.get_node(scope_node_id).range,
            types: self.parse_type_decs(scope_node_id, ast),
            constants: self.parse_consts(scope_node_id, ast),
            variables: self.parse_vars(scope_node_id, ast),
            ..Default::default()
        };
        debug!("{:?}", table);
        let node_id = self.arena.new_node(table);

        for subscope_id in ast.get_subscope_ids(scope_node_id) {
            let subtable = self.parse_scope(subscope_id, ast);
            node_id.append(subtable, &mut self.arena);
        }

        node_id
    }

    fn parse_type_decs(
        &self,
        scope_node_id: NodeId,
        ast: &Ast,
    ) -> Vec<Result<Symbol, SymbolError>> {
        let mut types: Vec<Result<Symbol, SymbolError>> = vec![];

        for node_id in ast.get_child_ids(scope_node_id) {
            let node = ast.get_node(node_id);

            if let NodeKind::TypeDec(type_dec_type) = &node.kind {
                let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                let name = ast.get_node(name_node_id).content.clone();

                let type_ = ast.get_type(node_id);

                let symbol = match type_dec_type {
                    TypeDecType::TypeDef => Ok(Symbol {
                        name,
                        def_position: node.range,
                        type_,
                    }),
                    _ => Err(SymbolError::Unknown),
                };

                types.push(symbol);
            }
        }

        types
    }

    fn parse_vars(&self, scope_node_id: NodeId, ast: &Ast) -> Vec<Result<Symbol, SymbolError>> {
        let mut variables: Vec<Result<Symbol, SymbolError>> = vec![];

        for node_id in ast.get_child_ids(scope_node_id) {
            let node = ast.get_node(node_id);

            if NodeKind::VariableDec == node.kind {
                let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                let name = ast.get_node(name_node_id).content.clone();

                let type_ = ast.get_type(node_id);

                let symbol = Ok(Symbol {
                    name,
                    def_position: node.range,
                    type_,
                });

                variables.push(symbol);
            }
        }

        variables
    }

    fn parse_consts(&self, scope_node_id: NodeId, ast: &Ast) -> Vec<Result<Symbol, SymbolError>> {
        let mut constants: Vec<Result<Symbol, SymbolError>> = vec![];

        for node_id in ast.get_child_ids(scope_node_id) {
            let node = ast.get_node(node_id);

            if NodeKind::ConstantDec == node.kind {
                let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                let name = ast.get_node(name_node_id).content.clone();

                let type_ = ast.get_type(node_id);

                let symbol = Ok(Symbol {
                    name,
                    def_position: node.range,
                    type_,
                });

                constants.push(symbol);
            }
        }

        constants
    }
}

#[derive(Debug, Default)]
struct ScopeSymbolTable {
    range: Range,
    types: Vec<Result<Symbol, SymbolError>>,
    constants: Vec<Result<Symbol, SymbolError>>,
    variables: Vec<Result<Symbol, SymbolError>>,
    functions: Vec<Result<Symbol, SymbolError>>,
}

#[derive(Debug)]
struct Symbol {
    name: String,
    def_position: Range,
    type_: Option<Type>,
}

#[derive(Debug)]
enum SymbolError {
    InvalidType,
    Unknown,
}

impl error::Error for SymbolError {}

impl fmt::Display for SymbolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            SymbolError::InvalidType => "Invalid type.",
            SymbolError::Unknown => "Unknown error.",
        };

        write!(f, "{}", message)
    }
}
