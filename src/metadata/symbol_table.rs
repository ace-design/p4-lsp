use crate::metadata::ast::{Ast, NodeKind, Visitable};
use crate::metadata::types::Type;
use crate::utils;
use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::{Position, Range};

#[derive(Debug, Default)]
pub struct SymbolTable {
    arena: Arena<ScopeSymbolTable>,
    root_id: Option<NodeId>,
}

pub trait SymbolTableActions {
    fn get_symbols_in_scope(&self, position: Position) -> Option<Symbols>;
    fn get_top_level_symbols(&self) -> Option<Symbols>;
    fn get_symbol_at_pos(&self, symbol: String, position: Position) -> Option<Symbol>;
}

impl SymbolTableActions for SymbolTable {
    fn get_symbols_in_scope(&self, position: Position) -> Option<Symbols> {
        let mut current_scope_id = self.root_id?;
        let mut symbols = self.arena.get(current_scope_id)?.get().symbols.clone();

        let mut subscope_exists = true;
        while subscope_exists {
            subscope_exists = false;

            for child_id in current_scope_id.children(&self.arena) {
                let scope = self.arena.get(child_id)?.get();
                if scope.range.start < position && position < scope.range.end {
                    current_scope_id = child_id;
                    subscope_exists = true;
                    symbols.add(scope.symbols.clone(), position);
                    break;
                }
            }
        }

        Some(symbols)
    }

    fn get_top_level_symbols(&self) -> Option<Symbols> {
        Some(self.arena.get(self.root_id?)?.get().symbols.clone())
    }

    fn get_symbol_at_pos(&self, symbol: String, position: Position) -> Option<Symbol> {
        todo!()
    }
}

impl SymbolTable {
    pub fn new(ast: &Ast) -> SymbolTable {
        let mut table = SymbolTable::default();

        table.root_id = Some(table.parse_scope(ast.get_root_id(), ast));

        table
    }

    fn parse_scope(&mut self, scope_node_id: NodeId, ast: &Ast) -> NodeId {
        let table = ScopeSymbolTable::parse(scope_node_id, ast);

        debug!("{:?}", table);
        let node_id = self.arena.new_node(table);

        for subscope_id in ast.get_subscope_ids(scope_node_id) {
            let subtable = self.parse_scope(subscope_id, ast);
            node_id.append(subtable, &mut self.arena);
        }

        node_id
    }
}

#[derive(Debug, Default, Clone)]
pub struct Symbols {
    pub types: Vec<Symbol>,
    pub constants: Vec<Symbol>,
    pub variables: Vec<Symbol>,
    pub functions: Vec<Symbol>,
}

impl Symbols {
    fn position_filter(&mut self, position: Position) {
        self.types.retain(|s| s.def_position.end < position);
        self.constants.retain(|s| s.def_position.end < position);
        self.variables.retain(|s| s.def_position.end < position);
        self.functions.retain(|s| s.def_position.end < position);
    }

    pub fn add(&mut self, mut other: Symbols, position: Position) {
        other.position_filter(position);

        self.types.append(&mut other.types);
        self.constants.append(&mut other.constants);
        self.variables.append(&mut other.variables);
        self.functions.append(&mut other.functions);
    }
}

#[derive(Debug, Default)]
struct ScopeSymbolTable {
    range: Range,
    symbols: Symbols,
}

impl ScopeSymbolTable {
    fn parse(scope_node_id: NodeId, ast: &Ast) -> ScopeSymbolTable {
        let mut table = ScopeSymbolTable {
            range: ast.get_node(scope_node_id).range,
            ..Default::default()
        };

        for node_id in ast.get_child_ids(scope_node_id) {
            let node = ast.get_node(node_id);

            match &node.kind {
                NodeKind::ConstantDec => {
                    let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                    let name = ast.get_node(name_node_id).content.clone();

                    let type_ = ast.get_type(node_id);

                    let symbol = Symbol::new(name, node.range, type_);

                    table.symbols.constants.push(symbol);
                }
                NodeKind::VariableDec => {
                    let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                    let name = ast.get_node(name_node_id).content.clone();

                    let type_ = ast.get_type(node_id);

                    let symbol = Symbol::new(name, node.range, type_);

                    table.symbols.variables.push(symbol);
                }
                NodeKind::TypeDec(_type_dec_type) => {
                    let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                    let name = ast.get_node(name_node_id).content.clone();

                    let type_ = ast.get_type(node_id);

                    table
                        .symbols
                        .types
                        .push(Symbol::new(name, node.range, type_));
                }
                _ => {}
            }
        }

        table
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,
    def_position: Range,
    type_: Option<Type>,
    usages: Vec<Range>,
}

impl Symbol {
    pub fn new(name: String, def_position: Range, type_: Option<Type>) -> Symbol {
        Symbol {
            name,
            def_position,
            type_,
            usages: vec![],
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
