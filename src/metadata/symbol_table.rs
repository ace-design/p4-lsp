use std::fmt;

use crate::metadata::ast::{Ast, NodeKind, VisitNode, Visitable};
use crate::metadata::types::Type;
use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::{Position, Range};

#[derive(Debug, Default)]
pub struct SymbolTable {
    arena: Arena<ScopeSymbolTable>,
    root_id: Option<NodeId>,
    undefined_list: Vec<Range>,
}

pub trait SymbolTableActions {
    fn get_symbols_in_scope(&self, position: Position) -> Option<Symbols>;
    fn get_top_level_symbols(&self) -> Option<Symbols>;
    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol>;
    fn get_symbol_at_pos_mut(&mut self, name: String, position: Position) -> Option<&mut Symbol>;
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

    fn get_symbol_at_pos_mut(&mut self, name: String, position: Position) -> Option<&mut Symbol> {
        let scope_id = self.get_scope_id(position)?;

        for pre_id in scope_id.predecessors(&self.arena) {
            let scope = self.arena.get(pre_id)?.get();

            if scope.symbols.contains(&name) {
                return self.arena.get_mut(pre_id)?.get_mut().symbols.get_mut(&name);
            }
        }

        None
    }

    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol> {
        let scope_id = self.get_scope_id(position)?;

        for pre_id in scope_id.predecessors(&self.arena) {
            let scope = self.arena.get(pre_id)?.get();

            if scope.symbols.contains(&name) {
                let scope = self.arena.get(pre_id)?.get();
                let symbols = scope.get_symbols();
                let symbol = symbols.get(&name)?;
                return Some(symbol);
            }
        }

        None
    }
}

impl SymbolTable {
    pub fn new(ast: &Ast) -> SymbolTable {
        let mut table = SymbolTable::default();

        table.root_id = Some(table.parse_scope(ast.visit_root()));
        table.parse_usages(ast.visit_root());

        table
    }

    fn get_scope_id(&self, position: Position) -> Option<NodeId> {
        self._get_scope_id(position, self.root_id?)
    }

    fn _get_scope_id(&self, position: Position, current: NodeId) -> Option<NodeId> {
        for child_scope_id in current.children(&self.arena) {
            let child_scope = self.arena.get(child_scope_id).unwrap().get();

            if position >= child_scope.range.start && position <= child_scope.range.end {
                if let Some(scope_id) = self._get_scope_id(position, child_scope_id) {
                    return Some(scope_id);
                } else {
                    return Some(child_scope_id);
                }
            }
        }
        self.root_id
    }

    fn parse_scope(&mut self, visit_node: VisitNode) -> NodeId {
        let table = ScopeSymbolTable::parse(visit_node.clone());

        let node_id = self.arena.new_node(table);

        for child_visit in visit_node
            .get_children()
            .into_iter()
            .filter(|n| n.get().kind.is_scope_node())
        {
            let subtable = self.parse_scope(child_visit);
            node_id.append(subtable, &mut self.arena);
        }

        node_id
    }

    fn parse_usages(&mut self, visit_node: VisitNode) {
        for child_visit in visit_node.get_descendants() {
            if let Some(type_node_visit) = child_visit.get_type_node() {
                let type_node = type_node_visit.get();
                let used_type = type_node_visit.get_type().unwrap();
                match used_type {
                    Type::Base(_) => {}
                    Type::Name => {
                        let name = type_node.content.clone();
                        let pos = type_node.range.start;

                        if let Some(symbol) = self.get_symbol_at_pos_mut(name, pos) {
                            symbol.usages.push(type_node.range);
                        } else {
                            self.undefined_list.push(type_node.range)
                        }
                    }
                    Type::Tuple => {}
                    Type::Header => {}
                    Type::Specialized => {}
                }
            }
        }
    }
}
impl fmt::Display for SymbolTable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();

        for node in self.arena.iter() {
            output.push_str(format!("{}\n", node.get()).as_str());
        }

        fmt.write_str(&output)
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

    pub fn contains(&self, name: &str) -> bool {
        for symbol in &self.types {
            if symbol.name == name {
                return true;
            }
        }
        for symbol in &self.constants {
            if symbol.name == name {
                return true;
            }
        }
        for symbol in &self.variables {
            if symbol.name == name {
                return true;
            }
        }
        for symbol in &self.functions {
            if symbol.name == name {
                return true;
            }
        }

        false
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for symbol in &self.types {
            if symbol.name == name {
                return Some(symbol);
            }
        }
        for symbol in &self.constants {
            if symbol.name == name {
                return Some(symbol);
            }
        }
        for symbol in &self.variables {
            if symbol.name == name {
                return Some(symbol);
            }
        }
        for symbol in &self.functions {
            if symbol.name == name {
                return Some(symbol);
            }
        }

        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for symbol in &mut self.types {
            if symbol.name == name {
                return Some(symbol);
            }
        }
        for symbol in &mut self.constants {
            if symbol.name == name {
                return Some(symbol);
            }
        }
        for symbol in &mut self.variables {
            if symbol.name == name {
                return Some(symbol);
            }
        }
        for symbol in &mut self.functions {
            if symbol.name == name {
                return Some(symbol);
            }
        }

        None
    }
}

#[derive(Debug, Default)]
struct ScopeSymbolTable {
    range: Range,
    symbols: Symbols,
}

impl fmt::Display for ScopeSymbolTable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("\n");

        output.push_str(
            format!(
                "{0: <8} | {1: <15} | {2: <10} | {3: <10} | {4: <10}\n",
                "symbol", "name", "position", "type", "usages"
            )
            .as_str(),
        );

        output.push_str("-".repeat(62).as_str());
        output.push('\n');

        for s in &self.symbols.types {
            output.push_str(format!("{: <8} | {}\n", "type", s.to_string()).as_str());
        }

        for s in &self.symbols.constants {
            output.push_str(format!("{: <8} | {}\n", "constant", s.to_string()).as_str());
        }

        for s in &self.symbols.variables {
            output.push_str(format!("{: <8} | {}\n", "variable", s.to_string()).as_str());
        }

        for s in &self.symbols.functions {
            output.push_str(format!("{: <8} | {}\n", "function", s.to_string()).as_str());
        }

        fmt.write_str(&output)
    }
}

impl ScopeSymbolTable {
    fn get_symbols(&self) -> &Symbols {
        &self.symbols
    }

    fn parse(root_visit_node: VisitNode) -> ScopeSymbolTable {
        let mut table = ScopeSymbolTable {
            range: root_visit_node.get().range,
            ..Default::default()
        };

        for child_visit_node in root_visit_node.get_children() {
            let child_node = child_visit_node.get();

            match &child_node.kind {
                NodeKind::ConstantDec => {
                    let name_node = child_visit_node.get_child_of_kind(NodeKind::Name).unwrap();
                    let name = name_node.get().content.clone();

                    let type_node = child_visit_node.get_type_node();
                    let type_ = if let Some(type_node) = type_node {
                        type_node.get_type()
                    } else {
                        None
                    };

                    let symbol = Symbol::new(name, name_node.get().range, type_);

                    table.symbols.constants.push(symbol);
                }
                NodeKind::VariableDec => {
                    let name_node = child_visit_node.get_child_of_kind(NodeKind::Name).unwrap();
                    let name = name_node.get().content.clone();

                    let type_node = child_visit_node.get_type_node();
                    let type_ = if let Some(type_node) = type_node {
                        type_node.get_type()
                    } else {
                        None
                    };

                    let symbol = Symbol::new(name, name_node.get().range, type_);

                    table.symbols.variables.push(symbol);
                }
                NodeKind::TypeDec(_type_dec_type) => {
                    let name_node = child_visit_node.get_child_of_kind(NodeKind::Name).unwrap();
                    let name = name_node.get().content.clone();

                    let type_node = child_visit_node.get_type_node();

                    let type_ = if let Some(type_node) = type_node {
                        type_node.get_type()
                    } else {
                        None
                    };

                    table
                        .symbols
                        .types
                        .push(Symbol::new(name, name_node.get().range, type_));
                }
                NodeKind::Params => {
                    for param_visit in child_visit_node.get_children() {
                        let param_node = param_visit.get();
                        if param_node.kind == NodeKind::Param {
                            let name_node = param_visit.get_child_of_kind(NodeKind::Name).unwrap();
                            let name = name_node.get().content.clone();

                            let type_node = param_visit.get_type_node();

                            let type_ = if let Some(type_node) = type_node {
                                type_node.get_type()
                            } else {
                                None
                            };

                            table.symbols.types.push(Symbol::new(
                                name,
                                name_node.get().range,
                                type_,
                            ));
                        }
                    }
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

impl fmt::Display for Symbol {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let type_str: String = if let Some(type_) = self.type_ {
            type_.to_string()
        } else {
            "None".into()
        };

        fmt.write_str(
            format!(
                "{0: <15} | {1: <10} | {2: <10} | {3: <10}",
                self.name,
                format!(
                    "l:{} c:{}",
                    self.def_position.start.line, self.def_position.start.character
                ),
                type_str,
                self.usages.len()
            )
            .as_str(),
        )
    }
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

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_definition_range(&self) -> Range {
        self.def_position
    }

    pub fn get_usages(&self) -> &Vec<Range> {
        &self.usages
    }
}
