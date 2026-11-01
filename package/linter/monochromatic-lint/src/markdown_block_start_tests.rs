//! Verify both block starters and lookalike prose without requiring a complete Markdown parse.

/// Import the production boundary classifier.
use super::starts_block_construct;

/// Real block markers are conservatively rejected.
#[test]
fn recognizes_the_incumbent_block_start_catalog() {
    for suffix in [
        " # h",
        " ###### h",
        " - item",
        " + item",
        " * item",
        " > quote",
        " 1) item",
        " 1. item",
        " ```rs",
        " ~~~",
        " <div>",
        " | row",
        " ---\n",
        " _ _ _",
        " ===",
        " #",
        " #\n",
        " #\r",
        " #\t heading",
        " 12.\r",
        " 9)\n",
        " 3)\t item",
        " 1)",
    ] {
        assert!(starts_block_construct(suffix, 0), "{suffix}");
    }
}

/// Negative controls distinguish actual markers from prefixes of ordinary words.
#[test]
fn leaves_non_markers_and_empty_lines_available() {
    for suffix in [
        "",
        "   ",
        "\n",
        "\r",
        " #word",
        " ####### h",
        " ~~ text",
        " -word",
        " ++ text",
        " --- prose",
        " 12word",
        " 12.x",
        " 123",
        " . next",
        " ) next",
        " ordinary",
        " 🚀",
    ] {
        assert!(!starts_block_construct(suffix, 0), "{suffix}");
    }
    assert!(starts_block_construct("🚀. # h", 5));
}
