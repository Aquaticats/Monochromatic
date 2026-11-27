//! What:
//! The order `install` works through the lock's tools,
//! and the tools it must install first because the others need them.
//! Why:
//! Installing `npm:wrangler` needs `pnpm`,
//! which needs `node`;
//! the order is a data table,
//! not a dependency solver,
//! because the repository's dependency chain is short and fixed.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const STAGING_IDS: string[];
//! export function syncOrder(tools: LockedTool[]): LockedTool[];
//! ```

/// Imports the locked tool the order is read from.
use super::super::model::LockedTool;
/// Imports the tool lookup.
use super::super::util::find_tool;
/// The tools installed first,
/// in this order,
/// because the others need them:
/// the `bin` downloader,
/// `uv`,
/// `mpm`,
/// the Rust
/// toolchain,
/// `cargo-binstall`,
/// Go,
/// Node and `pnpm`.
pub const STAGING_IDS: [&str; 9] = [
    "bin",
    "uv",
    "mpm",
    "rustup",
    "rust",
    "cargo-binstall",
    "go",
    "node",
    "pnpm",
];

/// What:
/// The tools in install order:
/// the staging tools that exist first (in staging order),
/// then the rest in lock
///           order.
/// Why:
/// Installing `npm:wrangler` needs `pnpm`,
/// which needs `node`;
/// the order is a data table,
/// not a dependency
///           solver,
/// because the repository's dependency chain is short and fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncOrder(tools: LockedTool[]): LockedTool[]
/// ```
pub fn sync_order(tools: &[LockedTool]) -> Vec<&LockedTool> {
    let mut ordered: Vec<&LockedTool> = Vec::new();
    for staging_id in STAGING_IDS {
        if let Some(found) = find_tool(tools, staging_id) {
            ordered.push(found);
        }
    }
    for tool in tools {
        if !STAGING_IDS.contains(&tool.declaration.id.as_str()) {
            ordered.push(tool);
        }
    }
    return ordered;
}
