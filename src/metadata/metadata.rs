use super::symbol_table::SymbolTable;
use super::Ast;

pub struct Metadata {
    pub ast: Ast,
    pub symbol_table: SymbolTable,
}

impl Metadata {
    pub fn new(source_code: &str, syntax_tree: tree_sitter::Tree) -> Option<Metadata> {
        let ast = Ast::new(source_code, syntax_tree)?;
        debug!("\nAST:\n{}", ast);
        let symbol_table = SymbolTable::new(&ast);
        debug!("\nSymbol Table:\n{}", symbol_table);

        Some(Metadata { ast : ast, symbol_table : symbol_table })
    }
}
