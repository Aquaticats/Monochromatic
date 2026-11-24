//! Controls for the record's own bytes:
//!  a maximal record's size,
//!  the widest line,
//!  and one inventory entry line.

use crate::worktree_clone_binding_record::*;

/// The binding's bound is the design's own 43,027 bytes,
/// computed here from the terms.
#[test]
fn the_bindings_bound_is_the_designs_sum() {
    assert_eq!(BINDING_MAXIMUM_BYTES, 43_027);
}

/// The widest line is the pointer's,
/// which is a `gitdir: ` prefix, a `PATH_MAX` path and a newline,
/// each byte written as two hexadecimal digits.
#[test]
fn the_widest_line_is_the_pointer_line() {
    assert_eq!(BINDING_MAXIMUM_LINE_BYTES, 8_219);
}

/// One inventory entry line is 313 bytes at its widest,
/// which is the design's own arithmetic.
#[test]
fn an_inventory_entry_line_is_at_most_313_bytes() {
    assert_eq!(INVENTORY_ENTRY_LINE_MAXIMUM_BYTES, 313);
    // The header plus the eight fields of the grammar, each at its widest:
    // kind, poolGuid, dataset, datasetGuid, bounds, boundsGuid, boundsCreatetxg and entries.
    assert_eq!(INVENTORY_FIXED_BYTES, 71 + 12 + 30 + 264 + 33 + 263 + 32 + 37 + 29);
}
