use crate::{metadata::NodeKind, utils};
use std::collections::HashMap;
use std::fmt;

use crate::metadata::ast::{Ast, Visitable};
use indextree::{Arena, NodeId};
use std::sync::atomic::{AtomicUsize, Ordering};
use tower_lsp::lsp_types::{Position, Range};

use super::Node;

fn get_id() -> usize {
    static SYMBOL_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
    SYMBOL_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    arena: Arena<ScopeSymbolTable>,
    root_id: Option<NodeId>,
    undefined_list: Vec<Range>,
}

pub trait SymbolTableActions {
    fn get_symbols_in_scope(&self, position: Position) -> Symbols;
    fn get_variable_at_pos(&self, position: Position, source_code: &str) -> Option<Vec<Field>>;
    fn get_top_level_symbols(&self) -> Option<Symbols>;
    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol>;
    fn get_symbol_at_pos_mut(&mut self, name: String, position: Position) -> Option<&mut Symbol>;
    fn rename_symbol(&mut self, id: usize, new_name: String);
}

impl SymbolTableActions for SymbolTable {
    fn get_symbols_in_scope(&self, position: Position) -> Symbols {
        let mut current_scope_id = self.root_id.unwrap();
        let mut symbols: Symbols;
        symbols = self
            .arena
            .get(current_scope_id)
            .unwrap()
            .get()
            .symbols
            .clone();

        let mut subscope_exists = true;
        while subscope_exists {
            subscope_exists = false;

            for child_id in current_scope_id.children(&self.arena) {
                let scope = self.arena.get(child_id).unwrap().get();
                if scope.range.start < position && position < scope.range.end {
                    current_scope_id = child_id;
                    subscope_exists = true;
                    symbols.merge(scope.symbols.clone(), position);
                    break;
                }
            }
        }

        symbols
    }

    fn get_variable_at_pos(&self, position: Position, source_code_t: &str) -> Option<Vec<Field>> {
        let mut source_code = source_code_t.to_string();
        let pos = utils::pos_to_byte(position, &source_code);
        let _ = source_code.split_off(pos);
        let mut index = source_code.len();
        let mut text: String;
        loop {
            index -= 1;
            let chara = source_code.chars().nth(index).unwrap();
            if !(chara.is_ascii_alphanumeric()
                || chara == '.'
                || chara == '_'
                || chara.is_ascii_whitespace())
            {
                text = source_code.split_off(index + 1);
                break;
            }
        }
        text = text.split_whitespace().collect::<Vec<&str>>().join("");
        if text.contains('.') {
            let t: Vec<&str> = source_code.split('\n').collect::<Vec<&str>>();
            let l = t.len();
            let position_start = Position {
                line: l as u32,
                character: (t[l - 1].len() + 1) as u32,
            };
            let names: Vec<&str> = text.split('.').collect();

            let symbols: &Option<&Symbol> =
                &self.get_symbol_at_pos(names[0].to_string(), position_start);
            if let Some(symbol) = symbols {
                // if let Some(x) = symbol.type_.get_name() {
                //     if x == Type::Name {
                //         let node = symbol.type_.get_node()?;
                //         let name = node.content.clone();
                //         let pos = node.range.start;
                //         match self.get_symbol_at_pos(name, pos) {
                //             Some(x) => {
                //                 symbol = x;
                //             }
                //             None => {
                //                 return Some(vec![]);
                //             }
                //         }
                //     }
                // }
                //
                for name in names.iter().take(names.len() - 1).skip(1) {
                    let fields = symbol.contains_fields(name.to_string());
                    if let Some(_field) = fields {
                        // if let Some(x) = field.type_.get_name() {
                        //     if x == Type::Name {
                        //         let node = field.type_.get_node()?;
                        //         let name = node.content.clone();
                        //         let pos = node.range.start;
                        //         match self.get_symbol_at_pos(name, pos) {
                        //             Some(x) => {
                        //                 symbol = x;
                        //             }
                        //             None => {
                        //                 return Some(vec![]);
                        //             }
                        //         }
                        //     }
                        // }
                    } else {
                        return Some(vec![]);
                    }
                }

                match symbol.get_fields() {
                    Some(fields) => {
                        return Some(fields.to_owned());
                    }
                    None => {}
                }
            }

            Some(vec![]) //Some(name_fields)
        } else {
            None
        }
    }

    fn get_top_level_symbols(&self) -> Option<Symbols> {
        Some(self.arena.get(self.root_id?)?.get().symbols.clone())
    }

    fn rename_symbol(&mut self, id: usize, new_name: String) {
        for scope in self.arena.iter_mut() {
            if let Some(symbol) = scope.get_mut().symbols.get_mut(id) {
                symbol.name = new_name;
                break;
            }
        }
    }

    fn get_symbol_at_pos_mut(&mut self, name: String, position: Position) -> Option<&mut Symbol> {
        let scope_id = self.get_scope_id(position)?;

        for pre_id in scope_id.predecessors(&self.arena) {
            let scope = self.arena.get(pre_id)?.get();

            if scope.symbols.contains(&name) {
                return self
                    .arena
                    .get_mut(pre_id)?
                    .get_mut()
                    .symbols
                    .find_mut(&name);
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
                let symbol = symbols.find(&name)?;
                return Some(symbol);
            }
        }

        None
    }
}

impl SymbolTable {
    pub fn new(ast: &Ast) -> SymbolTable {
        let mut table = SymbolTable::default();

        table.root_id = Some(table.parse_scope(ast.visit_root().get_id(), &ast.get_arena()));
        table.parse_usages(&mut ast.get_arena());

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
                    if scope_id.to_string() == self.root_id?.to_string() {
                        return Some(child_scope_id);
                    } else {
                        return Some(scope_id);
                    }
                } else {
                    return Some(child_scope_id);
                }
            }
        }
        self.root_id
    }

    fn parse_scope(&mut self, node_id: NodeId, arena: &Arena<Node>) -> NodeId {
        let table = ScopeSymbolTable::new(arena.get(node_id).unwrap().get().range);
        let current_table_node_id = self.arena.new_node(table);

        let mut queue: Vec<NodeId> = node_id.children(arena).collect();

        while let Some(node_id) = queue.pop() {
            let node = arena.get(node_id).unwrap().get();
            if node.kind.is_scope_node() {
                let subtable = self.parse_scope(node_id, arena);
                current_table_node_id.append(subtable, &mut self.arena);
            } else {
                queue.append(&mut node_id.children(arena).collect());
            }

            if let crate::language_def::Symbol::Init { kind, name_node } = node.symbol.clone() {
                let name_node_id = node_id
                    .children(arena)
                    .find(|id| {
                        arena.get(*id).unwrap().get().kind == NodeKind::Node(name_node.clone())
                    })
                    .unwrap();

                let name_node = arena.get(name_node_id).unwrap().get();

                let symbol = Symbol::new(name_node.content.clone(), name_node.range);
                self.arena
                    .get_mut(current_table_node_id)
                    .unwrap()
                    .get_mut()
                    .symbols
                    .add(kind, symbol);
            }
        }

        current_table_node_id
    }

    fn parse_usages(&mut self, arena: &mut Arena<Node>) {
        for node in arena
            .iter()
            .filter(|node| matches!(node.get().symbol, crate::language_def::Symbol::Usage))
        {
            let node = node.get();
            let symbol_name = &node.content;

            let scope_id = self.get_scope_id(node.range.start).unwrap();
            let scope_ids: Vec<NodeId> = scope_id.predecessors(&self.arena).collect();

            let mut found = false;
            for id in scope_ids {
                if let Some(symbol) = self
                    .arena
                    .get_mut(id)
                    .unwrap()
                    .get_mut()
                    .symbols
                    .find_mut(symbol_name)
                {
                    found = true;
                    symbol.add_usage(node.range);
                    break;
                }
            }

            if !found {
                self.undefined_list.push(node.range);
            }
        }
    }
}
impl fmt::Display for SymbolTable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();

        let mut sorted = self.arena.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.get().range.start.cmp(&b.get().range.start));

        for node in sorted {
            output.push_str(format!("{}\n", node.get()).as_str());
        }

        fmt.write_str(&output)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Symbols {
    symbols: HashMap<String, Vec<Symbol>>,
}

impl Symbols {
    pub fn add(&mut self, symbol_type_name: String, symbol: Symbol) {
        if !self.symbols.contains_key(&symbol_type_name) {
            self.symbols.insert(symbol_type_name.clone(), Vec::new());
        }

        self.symbols
            .get_mut(&symbol_type_name)
            .unwrap()
            .push(symbol);
    }

    pub fn get_type(&self, symbol_type_name: String) -> Option<&Vec<Symbol>> {
        self.symbols.get(&symbol_type_name)
    }

    fn position_filter(&mut self, position: Position) {
        for list in self.symbols.values_mut() {
            list.retain(|s| s.def_position.end < position)
        }
    }

    pub fn merge(&mut self, mut other: Symbols, position: Position) {
        other.position_filter(position);
        self.symbols.extend(other.symbols);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.symbols
            .values()
            .any(|list| list.iter().any(|s| s.name == name))
    }

    pub fn find(&self, name: &str) -> Option<&Symbol> {
        for list in self.symbols.values() {
            let maybe_symbol = list.iter().find(|&symbol| symbol.name == name);

            if let Some(symbol) = maybe_symbol {
                return Some(symbol);
            }
        }
        None
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for list in self.symbols.values_mut() {
            let maybe_symbol = list.iter_mut().find(|symbol| symbol.name == name);

            if let Some(symbol) = maybe_symbol {
                return Some(symbol);
            }
        }
        None
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Symbol> {
        for list in self.symbols.values_mut() {
            let maybe_symbol = list.iter_mut().find(|symbol| symbol.id == id);

            if let Some(symbol) = maybe_symbol {
                return Some(symbol);
            }
        }
        None
    }
}

#[derive(Debug, Default, Clone)]
struct ScopeSymbolTable {
    range: Range,
    symbols: Symbols,
}

impl fmt::Display for ScopeSymbolTable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("\n");

        output.push_str(
            format!(
                "{0: <8} | {1: <15} | {2: <10} | {3: <10}\n",
                "symbol", "name", "position", "usages"
            )
            .as_str(),
        );

        output.push_str("-".repeat(62).as_str());
        output.push('\n');

        for (symbol_type, list) in self.symbols.symbols.iter() {
            for s in list {
                output.push_str(format!("{: <8} | {}\n", symbol_type, s).as_str());
            }
        }

        fmt.write_str(&output)
    }
}

impl ScopeSymbolTable {
    fn new(range: Range) -> ScopeSymbolTable {
        ScopeSymbolTable {
            range,
            ..Default::default()
        }
    }

    fn get_symbols(&self) -> &Symbols {
        &self.symbols
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    def_position: Range,
    usages: Vec<Range>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    id: usize,
    name: String,
    def_position: Range,
    usages: Vec<Range>,
    fields: Option<Vec<Field>>,
}

impl fmt::Display for Symbol {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(
            format!(
                "{0: <15} | {1: <10} | {2: <10}",
                self.name,
                format!(
                    "l:{} c:{}",
                    self.def_position.start.line, self.def_position.start.character
                ),
                self.usages.len()
            )
            .as_str(),
        )
    }
}

impl Symbol {
    pub fn new(name: String, def_position: Range) -> Symbol {
        Symbol {
            id: get_id(),
            name,
            def_position,
            usages: vec![],
            fields: None,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
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

    pub fn add_usage(&mut self, range: Range) {
        self.usages.push(range);
    }

    pub fn get_usages(&self) -> &Vec<Range> {
        &self.usages
    }

    pub fn get_fields(&self) -> &Option<Vec<Field>> {
        &self.fields
    }

    pub fn contains_fields(&self, name: String) -> Option<Field> {
        match &self.fields {
            Some(x) => {
                for y in x {
                    if y.get_name() == name {
                        return Some(y.clone());
                    }
                }
                None
            }
            None => None,
        }
    }
}

impl Field {
    pub fn new(name: String, def_position: Range) -> Field {
        Field {
            name,
            def_position,
            usages: vec![],
        }
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
