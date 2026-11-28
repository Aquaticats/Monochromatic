//! What:
//!  The path Git's own reading resolves one command operand to,
//!  following every symbolic link it meets and allowing the last component to
//!  be missing.
//! Why:
//!  Git finds the worktree a `git worktree remove` names by matching the last
//!  components of its registered paths first and by resolving the operand as a
//!  path second,
//!  and the resolution follows links and tolerates one missing last component
//!  (`find_worktree` and `strbuf_realpath` in Git's `worktree.c` and
//!  `abspath.c` at `v2.56.1`).
//!  A slot whose folder a person moved aside or deleted keeps its registration
//!  and its binding record,
//!  and a symbolic link a person kept pointing at that folder names it still,
//!  so the wrapper's own reading has to resolve the operand the way Git does or
//!  the removal Git would run is read as naming nothing
//!  (`SPEC.md`,
//!  "Removal of a ZFS worktree clone").
//!  The resolution is Git's and not `std::fs::canonicalize`'s:
//!  a canonicalise call answers nothing once any component is gone,
//!  while Git reads a link whose target is gone and answers the path the link
//!  names.
//!  The one difference from Git's own reading is deliberate and is the reading a
//!  folder that stands already had:
//!  a trailing separator names the same directory,
//!  where Git's own component reader treats the separator after the last name as
//!  one more component and refuses the path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const path = pathGitResolvesAnOperandTo('/repo/.claude/worktrees/gone'); // '/repo/.claude/worktrees/gone'
//! ```

use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::{
    Component,
    Path,
    PathBuf,
};

/// How many symbolic links Git follows before it refuses a path.
/// Why:
///  Git's own reader counts the link that overflows against `MAXSYMLINKS`,
///  which is 32 in its `abspath.c`,
///  so it follows 33 links and refuses the 34th;
///  the same count here refuses the same path Git refuses,
///  and no chain shorter than that is answered differently.
const SYMBOLIC_LINKS_GIT_FOLLOWS: usize = 32;

/// What:
///  One path's components as Git's reader walks them:
///  the root left out,
///  `.` dropped,
///  and `..` kept as a component of its own.
/// Why:
///  Git resolves `.` by doing nothing and `..` against the path it has resolved so far,
///  which is what a reader that kept either as a name could not do,
///  and the root is where the resolved path starts rather than a name to push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = componentsGitReads('/repo/./a/../b'); // ['repo', 'a', '..', 'b']
/// ```
fn components_git_reads(path: &Path) -> VecDeque<OsString> {
    let mut words: VecDeque<OsString> = VecDeque::new();
    for part in path.components() {
        match part {
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => words.push_back(OsString::from("..")),
            Component::Normal(name) => words.push_back(name.to_os_string()),
            // What: A prefix is a Windows drive or share, which this reading never meets.
            // Why: The removal plan and the registry are built for Unix only (`worktree/mod.rs`), so a prefix here would
            //      be a shape nothing calls this with.
            Component::Prefix(_) => return VecDeque::new(),
        }
    }
    return words;
}

/// What:
///  The path Git's own reading of `path` answers,
///  or nothing when Git's reading answers nothing.
/// Why:
///  A missing component is allowed only when it is the last one,
///  so a path whose parent is gone answers nothing and Git's own refusal of it
///  stands;
///  a symbolic link is read and its target's components take its place,
///  which is how a person's link to a slot whose folder is gone still names
///  that slot;
///  and `..` after a link resolves against what the link resolved to.
///  The cause is never reported:
///  the caller asks whether the path names a worktree and hands everything else
///  to Git,
///  which words its own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolved = pathGitResolvesAnOperandTo(operandWithItsNearestStandingFolder);
/// ```
pub(crate) fn path_git_resolves_an_operand_to(path: &Path) -> Option<PathBuf> {
    let mut remaining: VecDeque<OsString> = components_git_reads(path);
    let mut resolved: PathBuf = match path.is_absolute() {
        true => PathBuf::from("/"),
        false => std::env::current_dir().ok()?,
    };
    let mut links: usize = 0;
    while let Some(next) = remaining.pop_front() {
        if next == ".." {
            resolved.pop();
            continue;
        }
        resolved.push(next.as_os_str());
        match std::fs::symlink_metadata(resolved.as_path()) {
            // Git's reader stops at a missing component only when no component follows it, and reads the path it has.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && remaining.is_empty() => {}
            Err(_) => return None,
            Ok(found) if found.file_type().is_symlink() => {
                links = links.saturating_add(1);
                if links > SYMBOLIC_LINKS_GIT_FOLLOWS + 1 {
                    return None;
                }
                let target: PathBuf = std::fs::read_link(resolved.as_path()).ok()?;
                resolved.pop();
                if target.is_absolute() {
                    resolved = PathBuf::from("/");
                }
                let mut spliced: VecDeque<OsString> = components_git_reads(target.as_path());
                spliced.append(&mut remaining);
                remaining = spliced;
            }
            Ok(_) => {}
        }
    }
    return Some(resolved);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "path_git_resolves_an_operand_to_tests.rs"]
mod tests;
