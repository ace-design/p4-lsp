use crate::utils;
use std::fmt;

use crate::metadata::ast::{Ast, NodeKind, TypeDecType, VisitNode, Visitable};
use crate::metadata::types::Type;
use indextree::{Arena, NodeId};
use std::sync::atomic::{AtomicUsize, Ordering};
use tower_lsp::lsp_types::{Position, Range};

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
    fn get_variable_in_pos(&self, position: Position, source_code: &str) -> Option<Vec<Field>>;
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
                    symbols.add(scope.symbols.clone(), position);
                    break;
                }
            }
        }

        symbols
    }

    fn get_variable_in_pos(&self, position: Position, source_code_t: &str) -> Option<Vec<Field>> {
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
            if let Some(mut symbol) = symbols {
                if let Some(x) = symbol.type_.get_name() {
                    if x == Type::Name {
                        let node = symbol.type_.get_node()?;
                        let name = node.content.clone();
                        let pos = node.range.start;
                        match self.get_symbol_at_pos(name, pos) {
                            Some(x) => {
                                symbol = x;
                            }
                            None => {
                                return Some(vec![]);
                            }
                        }
                    }
                }

                for name in names.iter().take(names.len() - 1).skip(1) {
                    let fields = symbol.contains_fields(name.to_string());
                    if let Some(field) = fields {
                        if let Some(x) = field.type_.get_name() {
                            if x == Type::Name {
                                let node = field.type_.get_node()?;
                                let name = node.content.clone();
                                let pos = node.range.start;
                                match self.get_symbol_at_pos(name, pos) {
                                    Some(x) => {
                                        symbol = x;
                                    }
                                    None => {
                                        return Some(vec![]);
                                    }
                                }
                            }
                        }
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

        table.root_id = Some(table.parse_scope(ast.visit_root()).unwrap());
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

    fn parse_scope(&mut self, visit_node: VisitNode) -> Option<NodeId> {
        let table_option = ScopeSymbolTable::parse(visit_node);
        if let Some(table) = table_option {
            let node_id = self.arena.new_node(table);

            for child_visit in visit_node.get_children() {
                let child_visit_id = child_visit.get();
                let kind = &child_visit_id.kind;
                if kind.is_scope_node() {
                    let subtable = self.parse_scope(child_visit);
                    if let Some(x) = subtable {
                        node_id.append(x, &mut self.arena);
                    }
                }
            }

            return Some(node_id);
        }
        None
    }

    fn get_value_symbol(&mut self, child_value: VisitNode, symbol: Symbol) {
        // todo-issue
        if let Some(child_symbol_new) = child_value.get_value_symbol_node() {
            let value_node = child_symbol_new.get();
            let name: String = value_node.content.clone();
            if let Some(new_field) = symbol.contains_fields(name) {
                let name = new_field.get_name();
                let pos = new_field.get_definition_range().start;
                let mut a: Option<Symbol> = None;
                if let Some(symbol_parent) = self.get_symbol_at_pos_mut(name, pos) {
                    symbol_parent.usages.push(value_node.range);
                    a = Some(symbol_parent.clone());
                }
                if let Some(x) = a {
                    self.get_value_symbol(child_symbol_new, x);
                }
            }
        }
    }

    fn parse_usages(&mut self, visit_node: VisitNode) {
        for child_visit in visit_node.get_descendants() {
            for type_node_visit in child_visit.get_children().into_iter() {
                let type_node = type_node_visit.get();
                if matches!(type_node.kind, NodeKind::Type(_)) {
                    let used_type = type_node_visit.get_type().unwrap();
                    match used_type {
                        Type::Base(_) => {}
                        Type::Name => {
                            let name_symbol = type_node.content.clone();
                            let pos_symbol = type_node.range.start;

                            if let Some(symbol) =
                                self.get_symbol_at_pos_mut(name_symbol.clone(), pos_symbol)
                            {
                                symbol.usages.push(type_node.range);
                            } else {
                                self.undefined_list.push(type_node.range)
                            }

                            if let Some(value_node_visit) = child_visit.get_value_node() {
                                for child_value in value_node_visit.get_children() {
                                    let value_node = child_value.get();
                                    let name = value_node.content.clone();
                                    let pos = value_node.range.start;
                                    let symbol_tt = &self.get_symbol_at_pos(name, pos);

                                    if let Some(symbol_t) = *symbol_tt {
                                        let mut symbol = symbol_t.to_owned();
                                        symbol.usages.push(value_node.range);
                                        self.get_value_symbol(child_value, symbol);
                                    } else {
                                        self.undefined_list.push(value_node.range)
                                    }
                                }
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

    pub fn find(&self, name: &str) -> Option<&Symbol> {
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
        self.functions.iter().find(|&symbol| symbol.name == name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        if let Some(s) = self.types.iter_mut().find(|symbol| symbol.name == name) {
            return Some(s);
        }
        if let Some(s) = self.constants.iter_mut().find(|symbol| symbol.name == name) {
            return Some(s);
        }
        if let Some(s) = self.variables.iter_mut().find(|symbol| symbol.name == name) {
            return Some(s);
        }
        if let Some(s) = self.functions.iter_mut().find(|symbol| symbol.name == name) {
            return Some(s);
        }

        None
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Symbol> {
        if let Some(s) = self.types.iter_mut().find(|symbol| symbol.id == id) {
            return Some(s);
        }
        if let Some(s) = self.constants.iter_mut().find(|symbol| symbol.id == id) {
            return Some(s);
        }
        if let Some(s) = self.variables.iter_mut().find(|symbol| symbol.id == id) {
            return Some(s);
        }
        if let Some(s) = self.functions.iter_mut().find(|symbol| symbol.id == id) {
            return Some(s);
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
                "{0: <8} | {1: <15} | {2: <10} | {3: <10} | {4: <10}\n",
                "symbol", "name", "position", "type", "usages"
            )
            .as_str(),
        );

        output.push_str("-".repeat(62).as_str());
        output.push('\n');

        for s in &self.symbols.types {
            output.push_str(format!("{: <8} | {}\n", "type", s).as_str());
        }

        for s in &self.symbols.constants {
            output.push_str(format!("{: <8} | {}\n", "constant", s).as_str());
        }

        for s in &self.symbols.variables {
            output.push_str(format!("{: <8} | {}\n", "variable", s).as_str());
        }

        for s in &self.symbols.functions {
            output.push_str(format!("{: <8} | {}\n", "function", s).as_str());
        }

        fmt.write_str(&output)
    }
}

impl ScopeSymbolTable {
    fn get_symbols(&self) -> &Symbols {
        &self.symbols
    }

    fn parse(root_visit_node: VisitNode) -> Option<ScopeSymbolTable> {
        fn _create_symbol_for_parse(child_visit_node: VisitNode, kind: NodeKind) -> Option<Symbol> {
            if let Some(name_node) = child_visit_node.get_child_of_kind(kind) {
                let name = name_node.get().content.clone();

                let type_node = child_visit_node.get_type_node();
                let mut node: Option<super::Node> = None;
                let type_ = if let Some(type_node) = type_node {
                    node = Some(type_node.get().clone());
                    type_node.get_type()
                } else {
                    None
                };

                return Some(Symbol::new(
                    name,
                    name_node.get().range,
                    TypeSymbol::new(type_, node),
                    None,
                ));
            }
            None
        }

        fn do_loop_parse(root_visit_node: VisitNode, table: &mut ScopeSymbolTable) {
            for child_visit_node in root_visit_node.get_children() {
                let child_node = child_visit_node.get();

                match &child_node.kind {
                    NodeKind::ConstantDec => {
                        if let Some(x) = _create_symbol_for_parse(child_visit_node, NodeKind::Name)
                        {
                            table.symbols.constants.push(x);
                        }
                    }
                    NodeKind::VariableDec => {
                        if let Some(x) = _create_symbol_for_parse(child_visit_node, NodeKind::Name)
                        {
                            table.symbols.variables.push(x);
                        }
                    }
                    NodeKind::ParserDec
                    | NodeKind::ControlDec
                    | NodeKind::ControlAction
                    | NodeKind::Instantiation
                    | NodeKind::MatchKind
                    | NodeKind::ErrorCst
                    | NodeKind::PreprocInclude
                    | NodeKind::PreprocDefine
                    | NodeKind::PreprocUndef
                    | NodeKind::ParserState
                    | NodeKind::ValueSet
                    | NodeKind::ControlTable
                    | NodeKind::TableKw
                    | NodeKind::Row
                    | NodeKind::SwitchCase => {
                        if let Some(x) = _create_symbol_for_parse(child_visit_node, NodeKind::Name)
                        {
                            table.symbols.functions.push(x);
                        }
                    }
                    NodeKind::TypeDec(_type_dec_type) => {
                        fn get_fields_vec(
                            child_visit_node: VisitNode,
                            type1: NodeKind,
                            type2: NodeKind,
                        ) -> Vec<Field> {
                            let mut fields: Vec<Field> = vec![];
                            let fields_node: VisitNode =
                                match child_visit_node.get_child_of_kind(type1) {
                                    Some(x) => x,
                                    None => {
                                        return fields;
                                    }
                                };

                            for field_visit in fields_node.get_children() {
                                let param_node = field_visit.get();
                                if param_node.kind == type2 {
                                    let name_node =
                                        field_visit.get_child_of_kind(NodeKind::Name).unwrap();
                                    let name = name_node.get().content.clone();

                                    let type_node = field_visit.get_type_node();

                                    let mut node: Option<super::Node> = None;
                                    let type_ = if let Some(type_node) = type_node {
                                        node = Some(type_node.get().clone());
                                        type_node.get_type()
                                    } else {
                                        None
                                    };

                                    fields.push(Field::new(
                                        name,
                                        name_node.get().range,
                                        TypeSymbol::new(type_, node),
                                    ));
                                }
                            }
                            fields
                        }

                        let name_node = child_visit_node.get_child_of_kind(NodeKind::Name).unwrap();
                        let name = name_node.get().content.clone();

                        let type_node = child_visit_node.get_type_node();

                        let mut node: Option<super::Node> = None;
                        let type_ = if let Some(type_node) = type_node {
                            node = Some(type_node.get().clone());
                            type_node.get_type()
                        } else {
                            None
                        };
                        let mut fields: Vec<Field> = vec![];
                        match _type_dec_type {
                            TypeDecType::TypeDef => {}
                            TypeDecType::HeaderType
                            | TypeDecType::HeaderUnion
                            | TypeDecType::Struct => {
                                fields = get_fields_vec(
                                    child_visit_node,
                                    NodeKind::Fields,
                                    NodeKind::Field,
                                );
                            }
                            TypeDecType::Enum => {
                                fields = get_fields_vec(
                                    child_visit_node,
                                    NodeKind::Options,
                                    NodeKind::Option,
                                );
                            }
                            TypeDecType::Parser | TypeDecType::Control => {
                                fields = get_fields_vec(
                                    child_visit_node,
                                    NodeKind::Params,
                                    NodeKind::Param,
                                );
                            }
                            TypeDecType::Package => {}
                        }

                        let fields_symbol: Option<Vec<Field>> = if fields.is_empty() {
                            None
                        } else {
                            Some(fields)
                        };

                        table.symbols.types.push(Symbol::new(
                            name,
                            name_node.get().range,
                            TypeSymbol::new(type_, node),
                            fields_symbol,
                        ));
                    }
                    NodeKind::Params => {
                        for param_visit in child_visit_node.get_children() {
                            let param_node = param_visit.get();
                            if param_node.kind == NodeKind::Param {
                                let name_node =
                                    param_visit.get_child_of_kind(NodeKind::Name).unwrap();
                                let name = name_node.get().content.clone();

                                let type_node = param_visit.get_type_node();

                                let mut node: Option<super::Node> = None;
                                let type_ = if let Some(type_node) = type_node {
                                    node = Some(type_node.get().clone());
                                    type_node.get_type()
                                } else {
                                    None
                                };

                                table.symbols.types.push(Symbol::new(
                                    name,
                                    name_node.get().range,
                                    TypeSymbol::new(type_, node),
                                    None,
                                ));
                            }
                        }
                    }
                    NodeKind::Extern => {
                        if let Some(x) = _create_symbol_for_parse(child_visit_node, NodeKind::Name)
                        {
                            table.symbols.functions.push(x);
                        } else if let Some(fn_node) =
                            root_visit_node.get_child_of_kind(NodeKind::FunctionName)
                        {
                            if let Some(x) = _create_symbol_for_parse(fn_node, NodeKind::Name) {
                                table.symbols.functions.push(x);
                            }
                        }
                    }
                    NodeKind::Function => {
                        if let Some(fn_node) =
                            root_visit_node.get_child_of_kind(NodeKind::FunctionName)
                        {
                            if let Some(x) = _create_symbol_for_parse(fn_node, NodeKind::Name) {
                                table.symbols.functions.push(x);
                            }
                        }
                    }
                    NodeKind::Method => {
                        for child_child_visit_node in child_visit_node.get_children() {
                            if let Some(fn_node) =
                                child_child_visit_node.get_child_of_kind(NodeKind::FunctionName)
                            {
                                if let Some(x) = _create_symbol_for_parse(fn_node, NodeKind::Name) {
                                    table.symbols.functions.push(x);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let root_visit_node_id = root_visit_node.get();
        let mut table = ScopeSymbolTable {
            range: root_visit_node_id.range,
            ..Default::default()
        };

        if let NodeKind::Root
        | NodeKind::Block
        | NodeKind::ParserDec
        | NodeKind::ControlDec
        | NodeKind::Table
        | NodeKind::Switch
        | NodeKind::Obj
        | NodeKind::Function
        | NodeKind::Methods
        | NodeKind::ControlAction
        | NodeKind::Body
        | NodeKind::ParserState
        | NodeKind::ControlTable
        | NodeKind::Instantiation
        | NodeKind::SwitchCase = &root_visit_node_id.kind
        {
            do_loop_parse(root_visit_node, &mut table);
        } else {
            return None;
        }

        Some(table)
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    def_position: Range,
    type_: TypeSymbol,
    usages: Vec<Range>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    id: usize,
    name: String,
    def_position: Range,
    type_: TypeSymbol,
    usages: Vec<Range>,
    fields: Option<Vec<Field>>,
}
#[derive(Debug, Clone)]
pub struct TypeSymbol {
    name: Option<Type>,
    node: Option<super::Node>,
}

impl fmt::Display for Symbol {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let type_str: String = if let Some(type_) = self.type_.name {
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
    pub fn new(
        name: String,
        def_position: Range,
        type_: TypeSymbol,
        fields: Option<Vec<Field>>,
    ) -> Symbol {
        Symbol {
            id: get_id(),
            name,
            def_position,
            type_,
            usages: vec![],
            fields,
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
    pub fn new(name: String, def_position: Range, type_: TypeSymbol) -> Field {
        Field {
            name,
            def_position,
            type_,
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

impl TypeSymbol {
    pub fn new(name: Option<Type>, node: Option<super::Node>) -> TypeSymbol {
        TypeSymbol { name, node }
    }

    pub fn get_name(&self) -> Option<Type> {
        self.name
    }

    pub fn get_node(&self) -> Option<super::Node> {
        self.node.clone()
    }
}
