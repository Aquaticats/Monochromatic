//! What:
//!  Group folders of the folder rule:
//!  a folder that holds child folders holds nothing else,
//!  and its `uses` folder summarizes the other groups and one-folder entries its child folders use.
//! Why:
//!  The tests of the group checks are in place before the checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findGroups(tree: ModuleTree): Set<Path>;
//! ```

/// Controls of the group checks stay outside release artifacts.
#[cfg(test)]
#[path = "layout_group_folders_tests.rs"]
mod tests;
