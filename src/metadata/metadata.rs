use super::symbol_table::SymbolTable;
use super::Ast;

use crate::metadata::{Symbol, Symbols};
use tower_lsp::lsp_types::Position;

use crate::metadata::symbol_table::SymbolTableActions;

pub trait AstEditor {
    fn open(content: &str, syntax_tree: tree_sitter::Tree);
    fn update(new_content: &str); // Take in a vector of changes instead?
}

pub trait AstQuery {}

pub trait SymbolTableEditor {}

pub trait SymbolTableQuery {
    fn get_symbols_at_pos(&self, position: Position) -> Option<Symbols>;
    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol>;
}

pub struct Metadata {
    pub ast: Ast,
    pub symbol_table: SymbolTable,
}

impl Metadata {
    pub fn new(source_code: &str, syntax_tree: tree_sitter::Tree) -> Option<Metadata> {
        let ast = Ast::new(source_code, syntax_tree)?;
        let symbol_table = SymbolTable::new(&ast);
        debug!("\nAST:\n{}\nSymbol Table:\n{}", ast, symbol_table);

        Some(Metadata { ast, symbol_table })
    }
}

impl SymbolTableQuery for Metadata {
    fn get_symbols_at_pos(&self, position: Position) -> Option<Symbols> {
        self.symbol_table.get_symbols_in_scope(position)
    }

    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol> {
        self.symbol_table.get_symbol_at_pos(name, position)
    }
}
