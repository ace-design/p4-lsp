use std::ops::Range;

use tower_lsp::lsp_types::Position;
use tree_sitter::Point;

pub fn pos_to_point(pos: Position) -> Point {
    Point {
        row: pos.line as usize,
        column: pos.character as usize,
    }
}

pub fn pos_to_byte(pos: Position, text: &str) -> usize {
    let mut total_bytes = 0;
    let lines = &text.lines().collect::<Vec<&str>>()[..(pos.line as usize)];

    for line in lines {
        total_bytes += line.len() + 1; // WARNING: Could break if line break char is not 1 byte
    }
    total_bytes + pos.character as usize
}

pub fn calculate_end_point(start: Point, new_content: &str) -> Point {
    let new_lines: Vec<&str> = new_content.lines().collect();
    let nb_lines = if new_lines.is_empty() {
        1
    } else {
        new_lines.len()
    };

    let column = match new_lines.len() {
        0 => start.column,
        1 => start.column + new_content.len(),
        _ => new_lines.last().unwrap().len(),
    };

    Point {
        column,
        row: start.row + nb_lines,
    }
}

pub fn ts_range_to_std_range(range: tree_sitter::Range) -> Range<usize> {
    return range.start_byte..range.end_byte;
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Position;

    use super::pos_to_byte;

    #[test]
    fn test_pos_to_byte() {
        let string = "this\nis\na test\nfor this function";

        assert_eq!(
            pos_to_byte(
                Position {
                    character: 3,
                    line: 2
                },
                string
            ),
            11
        );
        assert_eq!(
            pos_to_byte(
                Position {
                    character: 5,
                    line: 0
                },
                string
            ),
            5
        );
    }
}
