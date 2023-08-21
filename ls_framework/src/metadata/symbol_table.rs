use crate::{language_def, metadata::NodeKind, utils};
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

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolId {
    symbol_table_id: NodeId,
    index: usize,
}

impl SymbolId {
    pub fn new(symbol_table_id: NodeId, index: usize) -> Self {
        Self {
            symbol_table_id,
            index,
        }
    }
}

pub trait SymbolTableActions {
    fn get_symbol(&self, id: SymbolId) -> Option<&Symbol>;
    fn get_symbol_mut(&mut self, id: SymbolId) -> Option<&mut Symbol>;
    fn get_all_symbols(&self) -> Vec<Symbol>;
    fn get_symbols_in_scope(&self, position: Position) -> Vec<Symbol>;
    fn get_variable_at_pos(&self, position: Position, source_code: &str) -> Option<Vec<Symbol>>;
    fn get_top_level_symbols(&self) -> Vec<Symbol>;
    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol>;
    fn rename_symbol(&mut self, id: usize, new_name: String);
}

impl SymbolTableActions for SymbolTable {
    fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        let scope_table = self.arena.get(id.symbol_table_id)?.get();
        scope_table.symbols.get(id.index)
    }

    fn get_symbol_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        let scope_table = self.arena.get_mut(id.symbol_table_id)?.get_mut();
        scope_table.symbols.get_mut(id.index)
    }

    fn get_symbols_in_scope(&self, position: Position) -> Vec<Symbol> {
        let mut current_scope_id = self.root_id.unwrap();
        let mut symbols: Vec<Symbol>;
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

                    let mut scope_symbols = scope.symbols.clone();
                    scope_symbols.retain(|s| s.def_position.end < position);
                    symbols.append(&mut scope_symbols);
                    break;
                }
            }
        }

        symbols
    }

    fn get_variable_at_pos(&self, position: Position, source_code_t: &str) -> Option<Vec<Symbol>> {
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

    fn get_top_level_symbols(&self) -> Vec<Symbol> {
        self.arena
            .get(self.root_id.unwrap())
            .unwrap()
            .get()
            .symbols
            .clone()
    }

    fn rename_symbol(&mut self, id: usize, new_name: String) {
        for scope in self.arena.iter_mut() {
            if let Some(symbol) = scope.get_mut().symbols.get_mut(id) {
                symbol.name = new_name;
                break;
            }
        }
    }

    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol> {
        let scope_id = self.get_scope_id(position)?;

        for pre_id in scope_id.predecessors(&self.arena) {
            let scope = self.arena.get(pre_id)?.get();

            if let Some(symbol) = scope.symbols.iter().find(|s| s.name == name) {
                return Some(symbol);
            }
        }

        None
    }

    fn get_all_symbols(&self) -> Vec<Symbol> {
        let mut symbols: Vec<Symbol> = Vec::new();

        for child_id in self.root_id.unwrap().descendants(&self.arena) {
            symbols.append(&mut self.arena.get(child_id).unwrap().get().symbols.clone());
        }

        symbols
    }
}

impl SymbolTable {
    pub fn new(ast: &mut Ast) -> SymbolTable {
        let mut table = SymbolTable::default();

        table.root_id = Some(table.parse_scope(ast.visit_root().get_id(), ast.get_arena()));
        table.parse_usages(ast.get_arena());
        table.parse_types(ast.visit_root().get_id(), ast.get_arena());

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

    fn parse_scope(&mut self, node_id: NodeId, ast_arena: &mut Arena<Node>) -> NodeId {
        let table = ScopeSymbolTable::new(ast_arena.get(node_id).unwrap().get().range);
        let current_table_node_id = self.arena.new_node(table);

        let mut queue: Vec<NodeId> = node_id.children(ast_arena).collect();

        while let Some(node_id) = queue.pop() {
            if let crate::language_def::Symbol::Init {
                kind,
                name_node,
                type_node: _,
            } = &ast_arena.get(node_id).unwrap().get().symbol
            {
                let name_node_id = node_id
                    .children(ast_arena)
                    .find(|id| {
                        ast_arena.get(*id).unwrap().get().kind == NodeKind::Node(name_node.clone())
                    })
                    .unwrap();

                let name_node = ast_arena.get(name_node_id).unwrap().get();

                let mut symbol =
                    Symbol::new(name_node.content.clone(), kind.clone(), name_node.range);

                for id in node_id.descendants(ast_arena) {
                    if let language_def::Symbol::Field { name_node } =
                        &ast_arena.get(id).unwrap().get().symbol
                    {
                        let field_name_node = ast_arena
                            .get(
                                id.children(ast_arena)
                                    .find(|child_id| {
                                        ast_arena.get(*child_id).unwrap().get().kind
                                            == NodeKind::Node(name_node.clone())
                                    })
                                    .unwrap(),
                            )
                            .unwrap()
                            .get();

                        symbol.add_field(Symbol::new(
                            field_name_node.content.clone(),
                            "Field".to_string(),
                            field_name_node.range,
                        ));
                    }
                }

                let symbols = &mut self
                    .arena
                    .get_mut(current_table_node_id)
                    .unwrap()
                    .get_mut()
                    .symbols;
                symbols.push(symbol);

                let index = symbols.len() - 1;
                ast_arena
                    .get_mut(name_node_id)
                    .unwrap()
                    .get_mut()
                    .link(current_table_node_id, index);
            }

            if ast_arena.get(node_id).unwrap().get().kind.is_scope_node() {
                let subtable = self.parse_scope(node_id, ast_arena);
                current_table_node_id.append(subtable, &mut self.arena);
            } else {
                queue.append(&mut node_id.children(ast_arena).collect());
            }
        }

        current_table_node_id
    }

    fn parse_usages(&mut self, arena: &mut Arena<Node>) {
        for node in arena
            .iter_mut()
            .filter(|node| matches!(node.get().symbol, language_def::Symbol::Usage))
        {
            let node = node.get_mut();
            let symbol_name = &node.content;

            let scope_id = self.get_scope_id(node.range.start).unwrap();
            let scope_ids: Vec<NodeId> = scope_id.predecessors(&self.arena).collect();

            let mut found = false;
            for id in scope_ids {
                if let Some(index) = self
                    .arena
                    .get(id)
                    .unwrap()
                    .get()
                    .symbols
                    .iter()
                    .position(|s| &s.name == symbol_name)
                {
                    let symbol = &mut self.arena.get_mut(id).unwrap().get_mut().symbols[index];
                    node.link(id, index);
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

    fn parse_types(&mut self, root_id: NodeId, ast_arena: &mut Arena<Node>) {
        for node_id in root_id.descendants(ast_arena) {
            if let language_def::Symbol::Init {
                kind: _,
                name_node,
                type_node: Some(type_node_query),
            } = ast_arena.get(node_id).unwrap().get().symbol.clone()
            {
                let type_node_id = node_id
                    .children(ast_arena)
                    .find(|id| {
                        ast_arena.get(*id).unwrap().get().kind
                            == NodeKind::Node(type_node_query.clone())
                    })
                    .unwrap();

                if let Some(symbol_id) = ast_arena
                    .get(type_node_id)
                    .unwrap()
                    .get()
                    .linked_symbol
                    .clone()
                {
                    let name_node_id = node_id
                        .children(ast_arena)
                        .find(|id| {
                            ast_arena.get(*id).unwrap().get().kind
                                == NodeKind::Node(name_node.clone())
                        })
                        .unwrap();

                    if let Some(name_symbol_id) = ast_arena
                        .get(name_node_id)
                        .unwrap()
                        .get()
                        .linked_symbol
                        .clone()
                    {
                        self.get_symbol_mut(name_symbol_id)
                            .unwrap()
                            .set_type_symbol(symbol_id);
                    }
                }
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
struct ScopeSymbolTable {
    range: Range,
    symbols: Vec<Symbol>,
}

impl ScopeSymbolTable {
    fn new(range: Range) -> ScopeSymbolTable {
        ScopeSymbolTable {
            range,
            ..Default::default()
        }
    }
}

impl fmt::Display for ScopeSymbolTable {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("\n");

        output.push_str(
            format!(
                "{0: <10} | {1: <15} | {2: <10} | {3: <10} | {4: <10}\n",
                "symbol", "name", "position", "usages", "fields"
            )
            .as_str(),
        );

        output.push_str("-".repeat(62).as_str());
        output.push('\n');

        for s in self.symbols.iter() {
            output.push_str(&s.to_string());
        }

        fmt.write_str(&output)
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    id: usize,
    name: String,
    kind: String,
    type_symbol: Option<SymbolId>,
    def_position: Range,
    usages: Vec<Range>,
    fields: Option<Vec<Symbol>>,
}

impl Symbol {
    pub fn new(name: String, kind: String, def_position: Range) -> Symbol {
        Symbol {
            id: get_id(),
            name,
            kind,
            type_symbol: None,
            def_position,
            usages: vec![],
            fields: None,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn set_type_symbol(&mut self, id: SymbolId) {
        self.type_symbol = Some(id)
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

    pub fn add_field(&mut self, field: Symbol) {
        if let Some(fields) = &mut self.fields {
            fields.push(field);
        } else {
            self.fields = Some(vec![field]);
        }
    }

    pub fn get_fields(&self) -> &Option<Vec<Symbol>> {
        &self.fields
    }

    pub fn contains_fields(&self, name: String) -> Option<Symbol> {
        if let Some(fields) = &self.fields {
            for field in fields {
                if field.get_name() == name {
                    return Some(field.clone());
                }
            }
        }
        None
    }

    pub fn get_kind(&self) -> String {
        self.kind.clone()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let fields = if let Some(fields) = &self.fields {
            fields.len()
        } else {
            0
        };

        fmt.write_str(
            format!(
                "{0: <10} | {1: <15} | {2: <10} | {3: <10} | {4: <10}\n",
                self.kind,
                self.name,
                format!(
                    "l:{} c:{}",
                    self.def_position.start.line, self.def_position.start.character
                ),
                self.usages.len(),
                fields
            )
            .as_str(),
        )
    }
}