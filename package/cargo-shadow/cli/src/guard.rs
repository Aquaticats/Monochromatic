//! What:
//! The loop guard:
//! an environment value that one shadow instance leaves for the instance that its own `cargo` candidate may
//! start,
//! so a link that leads back to the shadow is skipped instead of run again.
//! Why:
//! The toolchain links `cargo` through a launcher,
//! an executable of its own that then starts the shadow.
//! The launcher's
//! resolved path is not the shadow's own executable path,
//! so the PATH search could not tell that this candidate is the
//! shadow,
//! picked it as the real cargo and started the shadow again without end.
//! Each instance records the resolved path
//! of the candidate it picks;
//! the instance that candidate starts skips every recorded path,
//! so each hop shrinks the
//! candidate list and the chain ends.
//!
//! The guard must never reach the real cargo's own children:
//! a script that cargo runs may call `cargo` again,
//! and a guard
//! that skipped the real cargo there would make that call fail.
//! So the guard is bound to a process.
//! After `exec` the process
//! id is unchanged,
//! so an exec guard is honored only by the process that has that id;
//! a spawn guard is honored only by
//! the direct child of the process that wrote it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // guard.ts
//! export function parseGuard(text: string | undefined): Guard | undefined { ... }
//! export function formatGuard(guard: Guard): string { ... }
//! ```

/// Imports the borrowed and owned path types.
use std::path::PathBuf;

/// The environment variable that carries the guard from one shadow instance to the next.
pub const GUARD_VARIABLE: &str = "CARGO_SHADOW_GUARD";

/// How many hops the guard allows before the shadow refuses to start anything:
/// the candidates on one PATH are few,
/// so a
/// longer chain means the guard did not do its job,
/// and a message is better than a loop.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open; not a measurement.
pub const MAX_HOPS: u32 = 8;

/// What:
/// `pub enum GuardKind { Exec, Spawn }` is a type with two possible values.
/// `Exec` means the writer replaced itself,
/// so the
/// reader is the same process;
/// `Spawn` means the writer started a child,
/// so the reader is that child.
/// Why:
/// The two ways of starting cargo need two different proofs that the guard is meant for this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type GuardKind = 'exec' | 'spawn';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardKind {
    /// The writer replaced itself with the candidate;
    /// the reader has the writer's process id.
    Exec,
    /// The writer started the candidate as a child;
    /// the reader's parent has the writer's process id.
    Spawn,
}

/// What:
/// `pub struct Guard { ... }` is a record of one guard value:
/// how it was passed,
/// the writer's process id,
/// how many hops
/// it has made,
/// and the resolved paths already tried (`Vec<PathBuf>`,
/// a growable list of owned paths).
/// Why:
/// It is what `parse_guard` reads and `format_guard` writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type Guard = { kind: GuardKind; pid: number; hops: number; skipped: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Guard {
    /// How the writer started the next instance.
    pub kind: GuardKind,
    /// The process id of the writer.
    pub pid: u32,
    /// How many shadow instances the chain has passed through,
    /// the writer included.
    pub hops: u32,
    /// The resolved paths of the candidates the chain has already picked.
    pub skipped: Vec<PathBuf>,
}

/// What:
/// `pub fn format_guard(guard: &Guard) -> String` writes a guard as text:
/// `e` or `s`,
/// the process id,
/// the hop count,
/// then
/// the paths,
/// separated by newlines.
/// Why:
/// A newline cannot occur in a path built from PATH entries,
/// which are split on `:`,
/// and an environment value may hold one,
/// so no path is ever cut in two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function formatGuard(guard: Guard): string { return [kind, pid, hops, ...skipped].join('\n'); }
/// ```
pub fn format_guard(guard: &Guard) -> String {
    let kind_letter: &str = match guard.kind {
        GuardKind::Exec => "e",
        GuardKind::Spawn => "s",
    };
    let mut lines: Vec<String> = vec![kind_letter.to_string(), guard.pid.to_string(), guard.hops.to_string()];
    for path in &guard.skipped {
        lines.push(path.to_string_lossy().to_string());
    }
    return lines.join("\n");
}

/// What:
/// `pub fn parse_guard(text: &str) -> Option<Guard>` reads what `format_guard` wrote;
/// any text that is not that shape is
/// `None`.
/// Why:
/// A value left by anything else,
/// or a corrupted one,
/// must never be believed:
/// it is ignored,
/// and the shadow behaves as if
/// it had been started fresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseGuard(text: string): Guard | undefined
/// ```
pub fn parse_guard(text: &str) -> Option<Guard> {
    let mut lines = text.split('\n');
    let kind: GuardKind = match lines.next()? {
        "e" => GuardKind::Exec,
        "s" => GuardKind::Spawn,
        _ => return None,
    };
    // `?` after `.parse()` returns `None` from the function when the text is not a number.
    let pid: u32 = lines.next()?.parse().ok()?;
    let hops: u32 = lines.next()?.parse().ok()?;
    let mut skipped: Vec<PathBuf> = Vec::new();
    for line in lines {
        if !line.is_empty() {
            skipped.push(PathBuf::from(line));
        }
    }
    return Some(Guard { kind, pid, hops, skipped });
}

/// What:
/// `pub fn is_meant_for(guard: &Guard, own_pid: u32, parent_pid: u32) -> bool` says whether this process is the one the
/// guard was left for.
/// Why:
/// An exec guard is for the process that has the writer's id;
/// a spawn guard is for the writer's direct child.
/// Anyone else,
/// such as a script that the real cargo runs,
/// gets no guard and finds the real cargo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isMeantFor(guard: Guard, ownPid: number, parentPid: number): boolean
/// ```
pub fn is_meant_for(guard: &Guard, own_pid: u32, parent_pid: u32) -> bool {
    match guard.kind {
        GuardKind::Exec => return guard.pid == own_pid,
        GuardKind::Spawn => return guard.pid == parent_pid,
    }
}
