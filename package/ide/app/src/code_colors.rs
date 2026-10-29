//! Source ink:
//!  the palette values code painting needs,
//!  and the translation of semantic paint roles into them.

/// Markdown roles share the tuned syntax hues.
use crate::source_style::{
    BOLD_ROLE, HEADING_ROLE, ITALIC_ROLE, LINK_ROLE, LINK_URL_ROLE, LIST_ROLE, QUOTE_ROLE,
    RAW_INLINE_ROLE, STRIKETHROUGH_ROLE,
};

/// Native palette values needed by code painting.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CodeColors {
    /// Default source ink from the system palette.
    pub foreground: [u8; 4],
    /// Selected source ink from the system palette.
    pub selected: [u8; 4],
    /// Select the corresponding syntax-color variants.
    pub dark: bool,
}

/// Translate semantic paint roles into the selected system-theme palette;
///  the minimap's picture takes its glyph colors from the same function.
pub fn ink(role: u32, colors: CodeColors) -> [u8; 4] {
    if role == 64 {
        return colors.selected;
    }
    // The Markdown roles reuse the six tuned hues; their weight or italic face is the second channel.
    if role == 1 || role == 8 || role == 10 || role == ITALIC_ROLE {
        if colors.dark {
            return [199, 146, 234, 255];
        }
        return [118, 54, 164, 255];
    }
    if role == 2 || role == RAW_INLINE_ROLE {
        if colors.dark {
            return [168, 204, 140, 255];
        }
        return [50, 101, 32, 255];
    }
    if role == 3 || role == STRIKETHROUGH_ROLE || role == QUOTE_ROLE {
        if colors.dark {
            return [160, 168, 176, 255];
        }
        return [85, 92, 101, 255];
    }
    if role == 4 || role == 13 || role == LINK_URL_ROLE || role == LIST_ROLE {
        if colors.dark {
            return [240, 188, 122, 255];
        }
        return [139, 77, 4, 255];
    }
    if role == 5 || role == 12 || role == HEADING_ROLE || role == LINK_ROLE {
        if colors.dark {
            return [130, 190, 235, 255];
        }
        return [20, 86, 127, 255];
    }
    if role == 6 || role == 11 || role == BOLD_ROLE {
        if colors.dark {
            return [225, 212, 157, 255];
        }
        return [113, 83, 16, 255];
    }
    return colors.foreground;
}
