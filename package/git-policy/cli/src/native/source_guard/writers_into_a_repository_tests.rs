//! What:
//!  Guard for every place this crate hands work to a thread or a task of its own:
//! each one is on the inventory below with the reason it is what it is,
//! and one that writes a file under a Git repository's folders is counted by the starter a fixture removal waits on.
//! Why:
//!  `remove` in `testing/test_support.rs` removes a fixture only once the work this process started has ended
//! (`foundation/scoped_start_started_threads_test_support.rs`),
//! and a wait that counts some writers and not others answers "nothing writes" when it has not looked everywhere:
//! a landing's note was written on a thread of its own,
//! the wait never saw it,
//! and the note made its directories again under a fixture that had been removed
//! (the review of 2026-11-15,
//! `REPRO_REMOVE_RETURNED_WHILE_NOTE_WRITER_WAS_HELD`).
//!  So a start added later must be a decision recorded here,
//!  never a reflex:
//! a start that creates or removes a file under a repository belongs in the counted starter
//! (`Counted`),
//! and one that does not says what it writes instead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const site of startSites(releaseSources)) expect(inventoryEntryFor(site)).toBeDefined();
//! for (const entry of inventory) expect(sitesIn(entry.file, entry.marker).length).toBe(entry.count);
//! ```

use crate::source_scan_tests::{classified_sources, real_roots, source_root};
use std::path::PathBuf;

/// The texts of a line that make it a place this crate hands work over.
/// Why:
///  These are the five ways a thread or a task starts in this crate,
///  and the two starters every counted job goes through;
/// a packager's `Builder::new` is not one,
/// because it is not spelled with its own type prefix here.
const A_START_IS_SPELLED: &[&str] = &[
    "std::thread::Builder::new()",
    "std::thread::spawn(",
    "tokio::spawn(",
    "spawn_scoped(",
    "spawn_on_runtime(",
    "spawn_on_runtime_within(",
    "spawn_named(",
    "spawn_task(",
];

/// What the wait of a fixture removal does with one start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Waited {
    /// The start goes through a starter that counts it,
    /// so a fixture removal waits for it
    /// (`started_threads`).
    Counted,
    /// The line is the starter itself,
    /// the mechanism every counted job is started by,
    /// so it is not a job of its own.
    TheStarterItself,
    /// The caller's own scope or join waits for the work,
    /// so it cannot be running when the call that started it returns.
    JoinedByTheCaller,
    /// The work makes no file under a Git repository:
    /// it reads,
    /// or writes words to a stream,
    /// or samples this process,
    /// or repairs an installed package's files.
    NoFileUnderARepository,
    /// The work runs while the service runs and ends when the service stops,
    /// and it creates no file of its own under a repository.
    ServiceLifetime,
}

/// One place this crate hands work to a thread or a task,
/// by the file it stands in,
/// a text on its line,
/// how many lines of that file carry the text,
/// and what the fixture removal's wait makes of it.
struct Start {
    /// The file,
    /// `/`-separated,
    /// relative to `src/native`.
    file: &'static str,
    /// A text on the line that starts the work,
    /// so a line that moved is still found.
    marker: &'static str,
    /// How many lines of the file carry the marker.
    count: usize,
    /// What the wait makes of it.
    waited: Waited,
    /// The reason,
    /// in the words of what the work does.
    reason: &'static str,
}

/// The inventory.
/// Why:
///  Every entry is a decision a reviewer can read:
/// what the work is,
/// and why the wait does or does not see it.
const STARTS: &[Start] = &[
    Start {
        file: "foundation/scoped_start.rs",
        marker: "Builder::new().name(String::from(name))",
        count: 1,
        waited: Waited::TheStarterItself,
        reason: "this is `spawn_named`, the thread starter every counted job of this crate goes through",
    },
    Start {
        file: "foundation/scoped_start.rs",
        marker: "spawn_on_runtime(async move",
        count: 1,
        waited: Waited::TheStarterItself,
        reason: "this is `spawn_task`, the runtime-task starter every counted job of this crate goes through",
    },
    Start {
        file: "foundation/scoped_start.rs",
        marker: "spawn_scoped(scope, move ||",
        count: 1,
        waited: Waited::TheStarterItself,
        reason: "this is `start_or_run_with`, whose scope joins the thread before its caller returns",
    },
    Start {
        file: "foundation/lazy_runtime.rs",
        marker: "return spawn_on_runtime(future);",
        count: 1,
        waited: Waited::TheStarterItself,
        reason: "this is the line of `spawn_on_runtime_within` that hands a task to `spawn_on_runtime`; a caller of it is listed below",
    },
    Start {
        file: "foundation/lazy_runtime.rs",
        marker: "String::from(\"git-policy-runtime-build\")",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it builds this process's runtime and makes no file",
    },
    Start {
        file: "foundation/work_awaited_within_a_ceiling.rs",
        marker: "spawn_named(WORK_THREAD_NAME",
        count: 1,
        waited: Waited::Counted,
        reason: "the note a landing writes into its journal, and every other job bounded by a ceiling, is a counted job",
    },
    Start {
        file: "foundation/scratch_entry_removal.rs",
        marker: "spawn_named(\"scratch-removal\"",
        count: 1,
        waited: Waited::Counted,
        reason: "it removes a scratch entry, which is an entry under a repository's folders",
    },
    Start {
        file: "foundation/log_generations_kept_to_a_bound.rs",
        marker: "spawn_named(\"log-trim\"",
        count: 1,
        waited: Waited::Counted,
        reason: "it renames and removes the generations of a log inside a repository's data folder",
    },
    Start {
        file: "foundation/diagnostic_lines_off_the_calling_thread.rs",
        marker: "String::from(\"cli-git-diagnostic-lines\")",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it writes this process's diagnostic lines to the stream it was given, for as long as the process runs, and makes no file",
    },
    Start {
        file: "foundation/notice_stream_that_does_not_hold_its_writer.rs",
        marker: "WRITER_THREAD_NAME",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it writes a notice's words to a stream and makes no file",
    },
    Start {
        file: "service/service_shadow_pool_fill.rs",
        marker: "spawn_named(\"shadow-pool-fill\"",
        count: 1,
        waited: Waited::Counted,
        reason: "it makes a finished shadow, a directory tree under the repository's Git directory",
    },
    Start {
        file: "service/service_shadow_draw.rs",
        marker: "spawn_named(\"shadow-pool-draw\"",
        count: 1,
        waited: Waited::Counted,
        reason: "it draws a shadow into the worktree's drawn folder under the Git directory",
    },
    Start {
        file: "service/service_repository_scratch_sweep.rs",
        marker: "spawn_named(\"repository-scratch-sweep\"",
        count: 1,
        waited: Waited::Counted,
        reason: "it removes the entries a killed command left in the repository's scratch directory",
    },
    Start {
        file: "service/service_scratch_release_removal.rs",
        marker: "spawn_task(remove(path))",
        count: 1,
        waited: Waited::Counted,
        reason: "it removes a released scratch tree, which is a directory under the repository's folders",
    },
    Start {
        file: "service/service_log_lines_off_the_calling_thread.rs",
        marker: "String::from(\"service log lines\")",
        count: 1,
        waited: Waited::ServiceLifetime,
        reason: "it writes the service's log lines to the stream it was given, for as long as the process runs, and makes no file",
    },
    Start {
        file: "service/service_server_accept_connections_as_tasks.rs",
        marker: "tokio::spawn(handle_connection(",
        count: 1,
        waited: Waited::ServiceLifetime,
        reason: "one task per connection, which ends with the connection and answers it on its socket",
    },
    Start {
        file: "service/service_runtime_build_report.rs",
        marker: "spawn_on_runtime_within(RUNTIME_BUILD_PATIENCE",
        count: 1,
        waited: Waited::ServiceLifetime,
        reason: "the tasks a service cannot be without, such as its acceptor, which end when the service stops and make no file",
    },
    Start {
        file: "service/service_provision_slow_manager_notice.rs",
        marker: "let thread: Option<JoinHandle<()>>",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it writes the words of a slow manager's notice to a stream",
    },
    Start {
        file: "landing/service_work.rs",
        marker: "spawn_named(REMOVAL_THREAD_NAME",
        count: 1,
        waited: Waited::Counted,
        reason: "it removes a worktree or its subvolume, which are directories under a repository's folders",
    },
    Start {
        file: "landing/service_index_prestage.rs",
        marker: "spawn_task(run_blocking_job",
        count: 1,
        waited: Waited::Counted,
        reason: "it makes and flushes a marker file in the repository's data folder",
    },
    Start {
        file: "landing/service_scratch_renewal.rs",
        marker: "String::from(\"scratch-renewal\")",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it sets the modification time of a scratch tree and creates nothing; it ends when the value that started it is dropped",
    },
    Start {
        file: "landing/service_durable.rs",
        marker: "spawn_on_runtime(background(jobs))",
        count: 1,
        waited: Waited::ServiceLifetime,
        reason: "it flushes records the landing already wrote and makes no entry; it ends with the process",
    },
    Start {
        file: "landing/service_rules_precompile.rs",
        marker: "spawn_on_runtime(async move",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it reads a repository's rules and compiles them into this process's cache, making no file",
    },
    Start {
        file: "landing/service_claim_renewal_running.rs",
        marker: "spawn_on_runtime(async move",
        count: 1,
        waited: Waited::ServiceLifetime,
        reason: "it renews the claims of a running service and ends when the claim is settled or the service stops",
    },
    Start {
        file: "landing/service_work_slow_lane.rs",
        marker: "tokio::spawn(run_awaiting(",
        count: 1,
        waited: Waited::ServiceLifetime,
        reason: "it runs the service's slow lane of work items and ends when the service stops",
    },
    Start {
        file: "commit/native_commit_waiting_notice.rs",
        marker: "\"native-commit-notice",
        count: 2,
        waited: Waited::NoFileUnderARepository,
        reason: "they write a commit's waiting notice to a stream and make no file",
    },
    Start {
        file: "worktree/add_standing_notice.rs",
        marker: "std::thread::Builder::new()",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it writes a standing notice's words to a stream and ends when the value that started it is dropped",
    },
    Start {
        file: "push/bounded_query_input.rs",
        marker: "spawn_named(\"cli-git-second-pass-input\"",
        count: 2,
        waited: Waited::Counted,
        reason: "the writer of a command's input is a counted job, so a fixture removal waits for it",
    },
    Start {
        file: "push/stop_of_a_push_and_sweep_of_its_locks.rs",
        marker: "spawn_task(async move",
        count: 1,
        waited: Waited::Counted,
        reason: "it sweeps the lock files of a stopped push, which are entries under a repository's folders",
    },
    Start {
        file: "push/child_watch.rs",
        marker: "spawn_on_runtime(async move",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it reads a watched child's output and makes no file",
    },
    Start {
        file: "push/child_watch_output_watched_on_the_runtime.rs",
        marker: "spawn_on_runtime(async move",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it reads a watched child's output and makes no file",
    },
    Start {
        file: "push/child_watch_one_shot_run.rs",
        marker: "String::from(\"cli-git-left-running-step\")",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it watches a step the wrapper left running and makes no file",
    },
    Start {
        file: "push/manual_push_range.rs",
        marker: "String::from(\"cli-git-range-input\")",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it writes a range into a child's standard input and makes no file",
    },
    Start {
        file: "push/auto_push_reconcile.rs",
        marker: "std::thread::Builder::new()",
        count: 1,
        waited: Waited::JoinedByTheCaller,
        reason: "the thread that pushes what the reconcile's loop asks; the scope waits for it, so it cannot outlive its caller",
    },
    Start {
        file: "push/auto_push_reconcile.rs",
        marker: ".spawn_scoped(scope,",
        count: 1,
        waited: Waited::JoinedByTheCaller,
        reason: "the line that starts the scoped thread above, which the scope joins before its caller returns",
    },
    Start {
        file: "process/open_file_of_a_descriptor_in_a_child.rs",
        marker: "REAPER_THREAD_NAME",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it reads another process's open files and makes no file",
    },
    Start {
        file: "process/stuck_children_reaped_when_the_kernel_lets_go.rs",
        marker: "REAPER_THREAD_NAME",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it waits for a child the kernel has not released and makes no file",
    },
    Start {
        file: "process/thread_quiet.rs",
        marker: "let sampler: Option<std::thread::JoinHandle<Duration>>",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it samples a thread's processor time and makes no file",
    },
    Start {
        file: "repository_link_check/repository_link_check_walk.rs",
        marker: "String::from(\"repository-link-check\")",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it reads a drive's link and answers on a channel, and makes no file",
    },
    Start {
        file: "standards_review/standards_review_run.rs",
        marker: "String::from(\"standards-review\")",
        count: 2,
        waited: Waited::NoFileUnderARepository,
        reason: "the review runs in this process and writes its words to the sink its caller lends it, making no file",
    },
    Start {
        file: "markdown/markdown_linter_guard.rs",
        marker: "std::thread::Builder::new()",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it reads the linter's answer and writes notices to a sink, making no file",
    },
    Start {
        file: "repair/package_repair_command.rs",
        marker: "spawn_on_runtime(async move",
        count: 1,
        waited: Waited::NoFileUnderARepository,
        reason: "it repairs an installed package's files under the package manager's folders, not a Git repository's",
    },
];

/// The lines of one file that start work,
/// as `(line number, the line without its indentation)`.
fn start_lines(text: &str) -> Vec<(usize, String)> {
    let mut found: Vec<(usize, String)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed: &str = line.trim_start();
        if A_START_IS_SPELLED.iter().any(|spelling| { return trimmed.contains(spelling); }) {
            found.push((index + 1, String::from(trimmed)));
        }
    }
    return found;
}

/// What:
///  The start lines of `sources` that no entry on the inventory names,
///  each as `file:line` and the line.
/// Why:
///   A start added after this inventory was written is one whose writer kind nobody judged,
///  and that is the state this guard is for.
fn unnamed_starts(sources: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (file, text) in sources {
        for (number, line) in start_lines(text) {
            let named: bool = STARTS
                .iter()
                .any(|entry| { return entry.file == file.as_str() && line.contains(entry.marker); });
            if !named {
                found.push(format!("{file}:{number} starts work no entry on the inventory names: {line}"));
            }
        }
    }
    return found;
}

/// How many start lines of `sources` each entry names,
/// in the inventory's order.
fn matches(sources: &[(String, String)]) -> Vec<usize> {
    let mut matched: Vec<usize> = vec![0; STARTS.len()];
    for (file, text) in sources {
        for (_, line) in start_lines(text) {
            for (index, entry) in STARTS.iter().enumerate() {
                if entry.file == file.as_str() && line.contains(entry.marker) {
                    matched[index] += 1;
                }
            }
        }
    }
    return matched;
}

/// What:
///  The entries whose count of lines no longer equals what they state,
///  each saying whether the entry is stale or matches more than it names.
/// Why:
///   An entry that matches nothing is a decision about code that moved or went;
///  an entry that matches more lines than it states is a second start of the same shape nobody judged.
fn entries_out_of_step(matched: &[usize]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (index, entry) in STARTS.iter().enumerate() {
        match matched[index].cmp(&entry.count) {
            std::cmp::Ordering::Less => found.push(format!("{}: no line carries `{}` any more, so the entry for it is stale", entry.file, entry.marker)),
            std::cmp::Ordering::Greater => found.push(format!("{}: {} lines carry `{}` where the entry states {}", entry.file, matched[index], entry.marker, entry.count)),
            std::cmp::Ordering::Equal => {}
        }
    }
    return found;
}

/// The release sources of this crate,
/// as `(identity, text)`.
fn release_sources() -> Vec<(String, String)> {
    let root: PathBuf = source_root();
    let files = classified_sources(root.as_path(), real_roots().as_slice()).expect("the crate's own module tree");
    let mut sources: Vec<(String, String)> = Vec::new();
    for file in files {
        if file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str())).expect("a source of the crate");
        sources.push((file.path, text));
    }
    return sources;
}

/// What:
///  Every place the release sources hand work to a thread or a task is on the inventory,
///  and every entry still names the lines it was written for.
/// Why:
///   A wait that counts some writers and not others is the defect this inventory exists for;
/// a start added later fails here until its writer kind is judged.
#[test]
fn every_start_in_the_release_sources_is_on_the_inventory() {
    let sources: Vec<(String, String)> = release_sources();
    let mut found: Vec<String> = unnamed_starts(sources.as_slice());
    found.extend(entries_out_of_step(matches(sources.as_slice()).as_slice()));
    assert!(found.is_empty(), "the inventory of starts does not match the code:\n{}", found.join("\n"));
}

/// What:
///  The wait counts every start the inventory classes as counted,
///  and no other class names a counted starter.
/// Why:
///   A start that creates or removes a file under a repository and is classed as anything else
///  is a writer the wait does not see,
///  which is the finding this guard answers;
/// a counted start classed as something else would let a reviewer read the opposite of the code.
#[test]
fn the_starts_that_write_a_repository_are_the_ones_the_wait_counts() {
    let counted: Vec<&str> = STARTS
        .iter()
        .filter(|entry| { return entry.waited == Waited::Counted; })
        .map(|entry| { return entry.file; })
        .collect();
    for entry in STARTS {
        let starter: bool = entry.marker.starts_with("spawn_named(") || entry.marker.starts_with("spawn_task(");
        assert_eq!(
            starter,
            entry.waited == Waited::Counted,
            "{} `{}` is classed {:?}: a start through a counted starter is what the wait sees, and one that is not cannot be",
            entry.file,
            entry.marker,
            entry.waited
        );
    }
    assert!(!counted.is_empty(), "no start is counted, so the wait would see nothing");
}

/// What:
///  The scan finds a start written into a fixture of its own,
///  and names the file and the line,
///  so an empty pass cannot come from a scan that reads nothing.
/// Why:
///   A guard whose walk stops returning files passes every run
///  (`source_scan_tests` exists for that reason);
/// this control shows the finding the guard would print.
#[test]
fn the_scan_names_a_start_written_into_a_fixture() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("source-guard-writers")
        .join(format!("scan-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.as_path()).expect("the fixture directory");
    let text: &str = "fn work() {\n    let _started = std::thread::Builder::new().name(String::from(\"a-fixture-thread\")).spawn(move || {});\n}\n";
    let sources: Vec<(String, String)> = vec![(String::from("a_fixture.rs"), String::from(text))];
    let found: Vec<String> = unnamed_starts(sources.as_slice());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("a_fixture.rs:2 "), "{}", found[0]);
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// The inventory is a list of files this crate really has,
/// so a moved module fails here rather than passing by silence.
#[test]
fn every_listed_file_is_a_release_source() {
    let root: PathBuf = source_root();
    let files = classified_sources(root.as_path(), real_roots().as_slice()).expect("the crate's own module tree");
    let names: Vec<String> = files.iter().filter(|file| { return !file.test_only; }).map(|file| { return file.path.clone(); }).collect();
    for entry in STARTS {
        assert!(names.iter().any(|name| { return name == entry.file; }), "{} is on the inventory and not a release source", entry.file);
    }
}

/// What:
///  Every entry says in words what the work does,
///  so the inventory is a record a reviewer can weigh and not a list of names.
/// Why:
///   The reason is the whole of the decision:
///  a start classed `NoFileUnderARepository` is right only if the sentence beside it is true,
/// and a reader who finds it false knows which entry to send back.
#[test]
fn every_entry_says_what_its_work_does() {
    for entry in STARTS {
        let words: usize = entry.reason.split_whitespace().count();
        assert!(words >= 8, "{} `{}` gives a reason of {words} words: {}", entry.file, entry.marker, entry.reason);
        assert!(!entry.reason.ends_with('.'), "{}: {}", entry.file, entry.reason);
    }
}
