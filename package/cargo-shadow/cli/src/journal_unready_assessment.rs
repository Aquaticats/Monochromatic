//! What:
//! The assessment of one journal's containment records:
//! what they show about the writers of a killed run.
//! Why:
//! Split out of `journal_unready.rs`,
//! which recovers the journals this assessment clears,
//! to keep that file under the code-line limit.
//! The record and the context it was made in were split out of this file into
//! `journal_unready_record_context.rs`,
//! for the same reason.
//! The items are re-exported from `journal_unready`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function assess(directory: string): Finding | undefined
//! ```

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, test_kill_process_group}` for this file.
use rustix::process::{
    Pid,
    test_kill_process_group,
};

/// Imports the control group records and queries for this file.
use crate::control_group::{
    LauncherState,
    Membership,
    Record,
    parse_record,
    query,
};

/// Imports the finding,
/// the record context and the reader of that context,
/// which live in their own file (split out of this module),
/// and re-exports them,
/// so every caller keeps the path it had.
pub(crate) use crate::journal_unready_record_context::{
    Finding,
    RecordContext,
    record_and_context,
};

/// Imports the verdict on a journal that has no ready record for this file.
use crate::journal_unready_verdict::{
    launcher_verdict_of,
    note_uncertain,
};
#[cfg(test)]
thread_local! {
    /// A hook that runs inside `assess` after every record was queried and before the unsettled scopes are judged,
    /// so a test can make a launcher create its scope in the gap between the two observations (round 10,
    /// 9a D1).
    static AFTER_QUERY: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub(crate) fn after_query_for_test(hook: Option<Rc<dyn Fn()>>)` sets the hook that runs between the queries of a check and its
/// judgement of unsettled scopes.
/// Why:
/// The schedule of 9a D1 puts a launcher's scope creation exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function afterQueryForTest(hook?: () => void): void
/// ```
#[cfg(test)]
pub(crate) fn after_query_for_test(hook: Option<std::rc::Rc<dyn Fn()>>) {
    AFTER_QUERY.with(|slot| {
        *slot.borrow_mut() = hook;
    });
}

/// What:
/// `fn acknowledge_hint(directory: &Path) -> String` is the end of every message that an acknowledgement can resolve.
/// Why:
/// The message names the journal and the command that acknowledges it,
/// and what the person asserts by running it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function acknowledgeHint(directory: string): string
/// ```
fn acknowledge_hint(directory: &Path) -> String {
    let quoted: String = crate::arrival::shell_quote(&directory.display().to_string());
    return format!(
        "the journal {} was kept. When every process or worker of that run that can modify the workspace has stopped, run: cargo-shadow acknowledge {}",
        directory.display(),
        quoted
    );
}

/// What:
/// `fn assess(directory: &Path) -> Option<Finding>` reads the records of one journal and says what they show about writers.
/// Why:
/// A record is written before every child starts,
/// so an empty record,
/// or a directory with no files at all (a begin cut off right after its creation),
/// shows that no child started;
/// a directory that holds other files but lost its record is uncertainty,
/// never "no children"
/// (round 7,
/// 6b2 2:
/// neither the ready record nor the absence of one stands in for it).
/// A journal that holds records is also checked for the mount and inode view it was made in (contract assumption c),
/// which `record_and_context` answers before the records are judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assess(directory: string): Finding | undefined
/// ```
pub(crate) fn assess(directory: &Path) -> Option<Finding> {
    let hint: String = acknowledge_hint(directory);
    let context: RecordContext = match record_and_context(directory, &hint) {
        Ok(found) => found,
        Err(answer) => return answer,
    };
    let text: String = context.text;
    let mut uncertain: Option<String> = context.uncertain;
    // A "starting" line of the weaker path that no process group line follows: a child may run that no record names.
    let mut announced_without_group: bool = false;
    let mut weaker_path: bool = false;
    // The scopes of runs that are not settled and have no launcher record that shows the launcher gone: the launcher of such a run may
    // not have finished entering its scope.
    let mut unsettled: Vec<PathBuf> = Vec::new();
    // The whole record is read first: the launcher line of a scope comes after the scope line, and a scope is judged by it.
    // A launcher line that the record ends without a line feed may be cut short (a torn write), so it is not understood (11b 2.6).
    let line_total: usize = text.lines().count();
    let ends_whole: bool = text.ends_with('\n');
    let mut records: Vec<Result<Record, String>> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if !ends_whole && index + 1 == line_total && line.starts_with("launcher ") {
            records.push(Err(
                "the launcher line is not ended by a line feed, so it may be cut short".to_string(),
            ));
        } else {
            records.push(parse_record(line));
        }
    }
    let mut settled_paths: Vec<&PathBuf> = Vec::new();
    let mut scope_paths: Vec<&PathBuf> = Vec::new();
    for record in &records {
        match record {
            Ok(Record::Settled {
                path: settled_path,
            }) => settled_paths.push(settled_path),
            Ok(Record::Scope {
                path: listed_path,
                ..
            }) => scope_paths.push(listed_path),
            _ => {}
        }
    }
    for record in &records {
        if let Ok(Record::Launcher {
            path: noted_path,
            ..
        }) = record
            && !scope_paths.contains(&noted_path)
        {
            note_uncertain(
                &mut uncertain,
                format!(
                    "{}: the record notes a launcher for the scope {}, which the journal does not list, so what it shows cannot be used; {hint}",
                    directory.display(),
                    noted_path.display()
                ),
            );
        }
    }
    for record in records.iter().cloned() {
        match record {
            Err(reason) => note_uncertain(
                &mut uncertain,
                format!(
                    "{}: the record of the containment of an earlier run holds a line that is not understood ({reason}), so a child of that run may still be writing; {hint}",
                    directory.display()
                ),
            ),
            Ok(
                Record::Settled {
                    ..
                }
                | Record::Launcher {
                    ..
                },
            ) => {}
            Ok(Record::Scope {
                namespace,
                parent_view,
                path: scope_path,
            }) => {
                // The launcher is asked about before the scope is read (round 11): a launcher that is gone can never join the scope, so
                // what the scope shows afterwards is final; one that is alive keeps blocking, and no acknowledgement overrides it.
                let settled: bool = settled_paths.contains(&&scope_path);
                let mut launcher: Option<(i32, u64, u64)> = None;
                for other in records.iter().rev() {
                    if let Ok(Record::Launcher {
                        pid,
                        start,
                        pid_namespace,
                        path: noted_path,
                    }) = other
                        && *noted_path == scope_path
                    {
                        launcher = Some((*pid, *start, *pid_namespace));
                        break;
                    }
                }
                if !settled {
                    let launcher_verdict: Option<(i32, LauncherState)> =
                        launcher.map(launcher_verdict_of);
                    match launcher_verdict {
                        None => unsettled.push(scope_path.clone()),
                        Some((pid, LauncherState::Alive)) => {
                            return Some(Finding::Live(format!(
                                "{}: the launcher (pid {pid}, which becomes the program) of the scope {} of an earlier run is still running; the journal was kept for a later run",
                                directory.display(),
                                scope_path.display()
                            )));
                        }
                        Some((pid, LauncherState::Unknown(reason))) => note_uncertain(
                            &mut uncertain,
                            format!(
                                "{}: whether the launcher (pid {pid}) of the scope {} is gone cannot be told ({reason}), so it may still join the scope; {hint}",
                                directory.display(),
                                scope_path.display()
                            ),
                        ),
                        Some((_, LauncherState::Gone)) => {}
                    }
                }
                match query(namespace, parent_view, &scope_path) {
                    Membership::Gone => {}
                    Membership::Alive => {
                        return Some(Finding::Live(format!(
                            "{}: a process in the scope {} of an earlier run is still running; the journal was kept for a later run",
                            directory.display(),
                            scope_path.display()
                        )));
                    }
                    Membership::Unknown(reason) => note_uncertain(
                        &mut uncertain,
                        format!(
                            "{}: whether the scope of that run still has a member cannot be told ({reason}), so a child of that run may still be writing; {hint}",
                            directory.display()
                        ),
                    ),
                }
            }
            Ok(Record::ProcessGroupStarting) => {
                weaker_path = true;
                announced_without_group = true;
            }
            Ok(Record::ProcessGroup(leader)) => {
                weaker_path = true;
                announced_without_group = false;
                let alive: bool = match Pid::from_raw(leader).map(test_kill_process_group) {
                    Some(Ok(())) => true,
                    Some(Err(code)) => code != Errno::SRCH,
                    None => true,
                };
                if alive {
                    return Some(Finding::Live(format!(
                        "{}: a process group of an earlier run (pid {leader}) is still running; the journal was kept for a later run",
                        directory.display()
                    )));
                }
            }
        }
    }
    #[cfg(test)]
    if let Some(hook) = AFTER_QUERY.with(|slot| {
        return slot.borrow().clone();
    }) {
        hook();
    }
    for open in unsettled {
        // The run did not record its end and no record shows its launcher gone, so nothing shows that the launcher finished entering
        // the scope: an absent scope may not be made yet, and one that exists and is empty may be waiting for the launcher to be
        // attached. A look at the scope, however late, does not turn this into a completed lifecycle (round 10, 9a D1); a positively
        // populated scope has already blocked.
        note_uncertain(
            &mut uncertain,
            format!(
                "{}: the run that was to make the scope {} did not record its end, so its launcher may not have finished entering the scope \
             (whether the scope is absent or empty now) and a program may still start in it; {hint}",
                directory.display(),
                open.display()
            ),
        );
    }
    if announced_without_group {
        note_uncertain(
            &mut uncertain,
            format!(
                "{}: an earlier run announced a child (the weaker path, no delegated control group) and never recorded its process group, so that child may still be writing; {hint}",
                directory.display()
            ),
        );
    }
    if weaker_path {
        note_uncertain(
            &mut uncertain,
            format!(
                "{}: an earlier run had no delegated control group, so a process that left its process group cannot be found and the end of that group does not show that the run's writers stopped; {hint}",
                directory.display()
            ),
        );
    }
    return uncertain.map(Finding::Uncertain);
}
