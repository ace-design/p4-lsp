use tower_lsp::lsp_types::Position;
use tree_sitter::Point;

pub fn pos_to_point(pos: Position) -> Point {
    Point {
        row: pos.line as usize,
        column: pos.character as usize,
    }
}

pub fn pos_to_byte(pos: Position, text: &str) -> usize {
    todo!()
}
