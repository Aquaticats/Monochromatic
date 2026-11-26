//! The texts, row map, viewport and palette every test builds:
//!  a long line of ordinary code,
//!  ten lines whose wrapped rows and block are known,
//!  a viewport of the tile's size,
//!  a word builder,
//!  and the palette the paint tests read.

use ide_app::{
    code_colors::CodeColors,
    line_rows::LineRows,
    row_map::RowMap,
    shaped_text::Viewport,
    wrap_sign::SignCoverage,
};

/// Which symbols the fonts of a test show.
pub enum Coverage {
    /// Every symbol.
    All,
    /// Every symbol but this one.
    Without(
        /// The symbol the fonts lack.
        char,
    ),
    /// Both symbols of this pair and no other.
    Pair(
        /// The pair the fonts show.
        (char, char),
    ),
    /// This symbol alone.
    Only(
        /// The only symbol the fonts show.
        char,
    ),
    /// No symbol.
    None,
}

impl SignCoverage for Coverage {
    fn can_display(
        &mut self,
        character: char,
    ) -> bool {
        return match self {
            Coverage::All => true,
            Coverage::Without(lacking) => character != *lacking,
            Coverage::Pair(pair) => pair.0 == character || pair.1 == character,
            Coverage::Only(shown) => character == *shown,
            Coverage::None => false,
        };
    }
}

/// A long line of ordinary code that wraps at the widths used here.
pub const CODE: &str = "let value = compute_total(first_item, second_item); // a long comment that keeps going and going";

/// A map of ten lines where lines 2,
/// 3,
/// and 7 are wrapped to 3,
/// 2,
/// and 5 rows and line 3 owns a 26 px block.
pub fn wrapped_map() -> RowMap {
    let mut rows = LineRows::plain(10);
    rows.apply([(2, 3), (3, 2), (7, 5)]);
    return RowMap::new(10, &[(3, 26.0)]).with_rows(&rows);
}

/// A viewport of `count` lines from `first`,
/// wrapped at `wrap` logical pixels.
pub fn viewport(
    first: usize,
    count: usize,
    wrap: f32,
) -> Viewport {
    return Viewport {
        first,
        count,
        width: wrap + 24.0,
        scale: 1.0,
        wrap: Some(wrap),
        clip: None,
    };
}

/// Words of the shape `word12 ` until the text holds `length` characters,
/// in one line without a terminator.
pub fn words(length: usize) -> String {
    let mut text = String::new();
    let mut number = 0;
    while text.len() < length {
        text.push_str(&format!("word{number} "));
        number += 1;
    }
    text.truncate(length);
    return text;
}

/// The palette of the pixel checks.
pub const PALETTE: CodeColors = CodeColors {
    foreground: [0, 0, 0, 255],
    selected: [255, 255, 255, 255],
    dark: false,
};
