use super::Ast;
use super::{symbol_table::SymbolTable, Field};

use crate::metadata::{Symbol, Symbols};
use tower_lsp::lsp_types::Position;

use crate::metadata::symbol_table::SymbolTableActions;

#[derive(Debug, Clone)]
pub enum SymbolTableEdit {
    Rename { symbol_id: usize, new_name: String },
}

pub trait SymbolTableEditor {
    fn new_edit(&mut self, edit: SymbolTableEdit);
    fn update(&mut self, ast: &Ast);
}

pub trait SymbolTableQuery {
    fn get_symbols_at_pos(&self, position: Position) -> Symbols;
    fn get_name_field(&self, position: Position, source_code: &str) -> Option<Vec<Field>>;
    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol>;
}

#[derive(Debug, Clone)]
pub struct SymbolTableManager {
    symbol_table: SymbolTable,
}

impl SymbolTableManager {
    pub fn new(ast: &Ast) -> SymbolTableManager {
        let symbol_table = SymbolTable::new(ast);
        debug!("\nSymbol Table:\n{symbol_table}");
        SymbolTableManager { symbol_table }
    }
}

impl SymbolTableQuery for SymbolTableManager {
    fn get_symbols_at_pos(&self, position: Position) -> Symbols {
        self.symbol_table.get_symbols_in_scope(position)
    }

    fn get_symbol_at_pos(&self, name: String, position: Position) -> Option<&Symbol> {
        self.symbol_table.get_symbol_at_pos(name, position)
    }

    fn get_name_field(&self, position: Position, source_code: &str) -> Option<Vec<Field>> {
        self.symbol_table.get_variable_at_pos(position, source_code)
    }
}

impl SymbolTableEditor for SymbolTableManager {
    fn new_edit(&mut self, edit: SymbolTableEdit) {
        match edit {
            SymbolTableEdit::Rename {
                symbol_id,
                new_name,
            } => self.symbol_table.rename_symbol(symbol_id, new_name),
        }
    }

    fn update(&mut self, ast: &Ast) {
        *self = SymbolTableManager::new(ast)
    }
}
