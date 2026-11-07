//! What:
//! The check that the launchers a launcher leads through never come back to it and never go in a circle.
//! Why:
//! A launcher replaces itself with the program its record names.
//! When that program is a launcher again (a record that names its own launcher,
//! or two records that name each other),
//! each one replaces itself with the other in the same process,
//! and nothing ever runs:
//! the owner's rule is "We should never invoke ourselves recursively",
//! and "nothing should ever wait forever or indefinitely".
//! A chain of launchers that ends at a program is fine (a launcher may name another launcher of a real program);
//! only a chain that revisits a launcher,
//! or runs past `MAX_LAUNCHER_CHAIN`,
//! is refused,
//! before anything is started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function chainProblem(own: string, program: string): string | undefined
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use super::launcher::{parse_record, record_path};

/// How many launchers in a row are followed before the chain counts as one that never reaches a program.
/// Delegated pick under the owner's rule that nothing waits without end (the same bound as the shadows' lookups,
/// `src/shadow_identity.rs`,
/// `MAX_LAUNCHER_HOPS`);
/// not a measurement.
pub const MAX_LAUNCHER_CHAIN: usize = 8;

/// What:
/// `pub fn chain_problem(own: &Path, program: &Path) -> Option<String>` is the reason a launcher must not start `program`,
/// when there is one.
/// Why:
/// It follows the records from `program` (a launcher is a program with a `.launch` record beside its resolved path) and compares every
/// resolved path with this launcher's own resolved path and with the paths already followed.
/// A program that does not resolve,
/// or a record that cannot be read,
/// ends the walk with no problem:
/// the start itself reports a missing program.
pub fn chain_problem(own: &Path, program: &Path) -> Option<String> {
    let own_resolved: PathBuf = fs::canonicalize(own).unwrap_or_else(|_| { return own.to_path_buf(); });
    let mut followed: Vec<PathBuf> = vec![own_resolved.clone()];
    let mut current: PathBuf = program.to_path_buf();
    for _hop in 0..=MAX_LAUNCHER_CHAIN {
        let Ok(resolved) = fs::canonicalize(&current) else {
            return None;
        };
        if resolved == own_resolved {
            return Some(format!(
                "the launcher {} leads back to itself: the program its record names, directly or through other launchers, is this launcher. \
                 Nothing was started. Repair the record {} (running `mpm install` in the tree writes it again)",
                own_resolved.display(),
                record_path(&own_resolved).display()
            ));
        }
        if followed.contains(&resolved) {
            return Some(format!(
                "the launchers {} and {} lead in a circle: their records name each other, directly or through other launchers, and no program ever starts. \
                 Nothing was started. Repair the records (running `mpm install` in the tree writes them again)",
                own_resolved.display(),
                resolved.display()
            ));
        }
        followed.push(resolved.clone());
        let Ok(text) = fs::read_to_string(record_path(&resolved)) else {
            return None;
        };
        let record = parse_record(&text)?;
        current = record.real;
    }
    return Some(format!(
        "the launcher {} leads through more than {MAX_LAUNCHER_CHAIN} launchers in a row without reaching a program. Nothing was started. \
         Repair the records (running `mpm install` in the tree writes them again)",
        own_resolved.display()
    ));
}

#[cfg(test)]
#[path = "launcher_chain_tests.rs"]
mod tests;
