//! What:
//!  What makes a name no plain slot name,
//!  the one rule the store,
//!  the walk and the per-slot claim share.
//! Why:
//!  A slot is a folder `git worktree add` made below `.claude/worktrees`,
//!  whose name is a branch name's last component;
//!  Git refuses a name that starts with a dot,
//!  so a leading dot here is a folder this route could only reach outside the
//!  add.
//!  The rule lives apart from the store's own line grammar and its lock,
//!  because the project decided on 2026-11-21 that the store becomes one file
//!  per slot
//!  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`),
//!  and a caller that only judges a name must not move with that format.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = slotNameRefusal('two words'); // 'it holds a space...'
//! ```

/// What makes `name` no plain slot name,
/// or nothing when it is one.
pub(crate) fn slot_name_refusal(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some(String::from("it is empty"));
    }
    if name == "." || name == ".." || name.contains('/') {
        return Some(String::from("it holds a path separator or names a folder above"));
    }
    if name.starts_with('.') {
        return Some(String::from("it starts with a dot, and no worktree Git adds is named that way"));
    }
    if name.chars().any(char::is_whitespace) || name.chars().any(char::is_control) {
        return Some(String::from("it holds a space, a tab or another control character"));
    }
    return None;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_name_tests.rs"]
mod tests;
