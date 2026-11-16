//! What:
//!  A check of the crate's own source text that fails when a storage call of the repository location read
//!  sits outside the one job `LOCATION_READ_CEILING` bounds.
//! Why:
//!  Every guarded command asks that read (`repo_read/repository_facts.rs`),
//!  and each of its calls waits in the kernel on a mount that stopped answering:
//!  a `canonicalize` or a `symlink_metadata` there holds a command with no end,
//!  which is what `repo_read/repo_read_layout_within.rs` exists to prevent (`LOCATION_READ_CEILING`).
//!  The owner's rule is "nothing should ever wait forever or indefinitely".
//!  The package's other scan of this kind (`blocking_code_off_the_runtime_tests.rs`) reads storage calls on
//!  tasks the runtime polls,
//!  so it sees none of them here:
//! every function of this read is synchronous,
//!  and its calls are held by a thread the caller waits for under a ceiling rather than by a worker.
//!  This scan covers that place:
//!  every storage call in the files the read is written in must be inside a function the ceiling job runs,
//!  and the entry points a caller uses must read nothing themselves,
//!  so a storage call added where no ceiling covers it fails a test before a review.
//!
//! The scan is textual and says where it is coarse:
//!  it follows no call,
//!  so a storage call added in a callee of another file is not a finding here
//!  (it is still inside the job,
//! because the job is the whole read);
//!  it cannot see that a listed function is only called from the job,
//!  which is what the dynamic controls of `repo_read/repo_read_layout_within_tests.rs` prove for the entry
//!  points,
//!  where a read is given no thread and every entry point answers nothing;
//!  and it reads one file at a time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/storage_reads_within_a_ceiling_tests.ts: one check over the read's own text.
//! ```

use super::blocking_code_off_the_runtime_tests::{Function, functions_in, holds_any};
use crate::source_scan_tests::{real_roots, source_root};
use std::path::Path;

/// The files the location read is written in,
/// as paths relative to `src/native`,
/// `/`-separated.
/// Why:
///  A file added to the read belongs here,
/// and a file renamed here fails the scan instead of passing it.
pub(super) const THE_READ: &[&str] = &[
    "repo_read/repo_read_layout.rs",
    "repo_read/repo_read_layout_within.rs",
    "repo_read/repo_read_layout_without.rs",
    "repo_read/repo_read_layout_disk_and_directory_facts.rs",
];

/// The functions a caller uses to ask the read,
/// which must read nothing on the caller's own thread.
/// Why:
///  Each of them hands the question to the ceiling's job,
///  and a storage call put back into one of them is exactly the defect the ceiling exists to prevent.
pub(super) const THE_ENTRY_POINTS: &[&str] =
    &["read_layout", "read_layout_with", "read_layout_in", "read_layout_after_success", "read_layout_after_success_with", "read_layout_after_success_for", "read_layout_without", "git_path"];

/// One function whose storage calls the ceiling's job carries.
pub(super) struct Carry {
    /// Its file,
    /// as a path relative to `src/native`.
    pub(super) file: &'static str,
    /// The name the signature gives it.
    pub(super) function: &'static str,
    /// Why its storage calls run inside the job,
    /// and so which part of the read it is.
    pub(super) why: &'static str,
}

/// The calls in these files that wait on the file system:
/// every `std::fs` call of the read,
/// the path questions a directory entry answers,
/// and the two readers of a file's bytes.
/// Why:
///  Each one can wait in the kernel with no end on a mount that stopped answering,
///  and none of them belongs on a caller's thread.
pub(super) const STORAGE_CALLS_OF_THE_READ: &[&str] =
    &["std::fs::", ".exists()", ".is_dir()", ".is_file()", "read_regular_file(", "read_following(", "File::open(", "File::create("];

/// The functions the ceiling's job runs.
/// Remove an entry when its code goes;
/// the test fails on an entry that matches no storage call.
pub(super) const THE_JOB: &[Carry] = &[
    Carry {
        file: "repo_read/repo_read_layout.rs",
        function: "dot_git_target",
        why: "the `.git` entry of one directory, read on the walk up",
    },
    Carry {
        file: "repo_read/repo_read_layout.rs",
        function: "worktree_entries",
        why: "the per-worktree configuration file, read once the format turns it on",
    },
    Carry {
        file: "repo_read/repo_read_layout.rs",
        function: "finish",
        why: "the checks of one discovered Git directory and the files it names",
    },
    Carry {
        file: "repo_read/repo_read_layout_within.rs",
        function: "from_a_directory",
        why: "the ceiling's own job for the callers that name a directory",
    },
    Carry {
        file: "repo_read/repo_read_layout_within.rs",
        function: "is_a_link_at",
        why: "the ceiling's own job for the one lookup `git_path` makes",
    },
    Carry {
        file: "repo_read/repo_read_layout_disk_and_directory_facts.rs",
        function: "start_directory",
        why: "the directory a command starts in, with its `-C` options",
    },
    Carry {
        file: "repo_read/repo_read_layout_disk_and_directory_facts.rs",
        function: "small_file_bytes",
        why: "a small file of the layout, such as `HEAD` or a `gitdir:` file",
    },
    Carry {
        file: "repo_read/repo_read_layout_disk_and_directory_facts.rs",
        function: "has_git_directory_entries",
        why: "the three entries a bare Git directory holds",
    },
    Carry {
        file: "repo_read/repo_read_layout_disk_and_directory_facts.rs",
        function: "owned_by_current_user",
        why: "the owner of the worktree and of its Git directory",
    },
    Carry {
        file: "repo_read/repo_read_layout_disk_and_directory_facts.rs",
        function: "device_of",
        why: "the device of a directory, which stops the walk where Git stops",
    },
    Carry {
        file: "repo_read/repo_read_layout_disk_and_directory_facts.rs",
        function: "has_git_directory_entries_split",
        why: "the three entries of a worktree whose Git directory and common directory differ",
    },
];

/// One storage call of the read,
/// and how the scan judges it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Judgement {
    /// A function the ceiling's job runs.
    Carried,
    /// A function a caller uses,
    /// which must not read anything itself.
    AnEntryPoint,
    /// A function no ceiling covers.
    OutsideTheJob,
}

/// One storage call of the read:
/// the file as a path relative to the scanned root,
/// the function that holds it,
/// the line,
/// and how the scan judges it.
pub(super) type Finding = (String, String, usize, Judgement);

/// Whether `file` and `function` are one of the entry points.
fn is_an_entry_point(function: &str) -> bool {
    return THE_ENTRY_POINTS.contains(&function);
}

/// Whether `file` and `function` are one the ceiling's job runs.
fn is_carried(file: &str, function: &str) -> bool {
    return THE_JOB.iter().any(|carry| {
        return carry.file == file && carry.function == function;
    });
}

/// Every storage call of the read under `root`,
/// judged by where it sits.
fn findings_in(root: &Path) -> Result<Vec<Finding>, String> {
    let mut found: Vec<Finding> = Vec::new();
    for file in THE_READ {
        let text: String = std::fs::read_to_string(root.join(file)).map_err(|error| return format!("cannot read {file}: {error}"))?;
        let functions: Vec<Function> = functions_in(text.as_str()).map_err(|error| return format!("{file}: {error}"))?;
        for function in functions {
            for (number, line, _joined) in &function.task_lines {
                if !holds_any(line.as_str(), STORAGE_CALLS_OF_THE_READ) {
                    continue;
                }
                let judgement: Judgement = if is_an_entry_point(function.name.as_str()) {
                    Judgement::AnEntryPoint
                } else if is_carried(file, function.name.as_str()) {
                    Judgement::Carried
                } else {
                    Judgement::OutsideTheJob
                };
                found.push((String::from(*file), function.name.clone(), *number, judgement));
            }
        }
    }
    return Ok(found);
}

/// The storage calls of this crate's own read,
/// judged.
fn findings() -> Vec<Finding> {
    return findings_in(&source_root()).expect("the crate's own sources of the location read");
}

/// The findings the scan refuses,
/// each as a `file:line [function] reason` text.
fn unexplained(found: &[Finding]) -> Vec<String> {
    return found
        .iter()
        .filter_map(|(file, function, number, judgement)| {
            return match judgement {
                Judgement::Carried => None,
                Judgement::AnEntryPoint => Some(format!(
                    "{file}:{number} [{function}] an entry point of the read reads the file system on its caller's thread; \
                     the question belongs to the ceiling's job in repo_read/repo_read_layout_within.rs"
                )),
                Judgement::OutsideTheJob => Some(format!(
                    "{file}:{number} [{function}] a storage call no ceiling covers; it is the read's work, so it runs inside \
                     the job LOCATION_READ_CEILING bounds, or it is named in THE_JOB with the part of the read it is"
                )),
            };
        })
        .collect();
}

/// The entries of the job that match no storage call,
/// each as a `file [function]: why` text.
fn stale(found: &[Finding]) -> Vec<String> {
    return THE_JOB
        .iter()
        .filter(|carry| {
            return !found.iter().any(|(file, function, _, judgement)| {
                return *judgement == Judgement::Carried && file == carry.file && function == carry.function;
            });
        })
        .map(|carry| format!("{} [{}]: {}", carry.file, carry.function, carry.why))
        .collect();
}

/// The read of a repository's location keeps every storage call inside the one job its ceiling bounds,
/// and its entry points read nothing themselves.
#[test]
fn the_location_read_keeps_every_storage_call_inside_its_ceiling_job() {
    let found: Vec<Finding> = findings();
    let refused: Vec<String> = unexplained(found.as_slice());
    assert!(
        refused.is_empty(),
        "the read of a repository location runs on every guarded command, so a storage call it makes without the ceiling \
         holds a command on a mount that stopped answering (LOCATION_READ_CEILING); fix it or name it in THE_JOB with the \
         part of the read it is:\n{}",
        refused.join("\n")
    );
}

/// Every entry of the job still matches a storage call,
/// so the list shrinks as the read changes.
#[test]
fn every_listed_part_of_the_job_still_matches_code() {
    let found: Vec<Finding> = findings();
    let gone: Vec<String> = stale(found.as_slice());
    assert!(gone.is_empty(), "remove these entries from THE_JOB, their code is gone:\n{}", gone.join("\n"));
}

/// The scan of the real crate answers findings and never an empty pass,
/// so a file it could not read or a rule that stopped matching fails here.
#[test]
fn the_scan_of_the_real_read_answers_findings_and_never_an_empty_pass() {
    let found: Vec<Finding> = findings();
    assert!(!found.is_empty(), "the read holds storage calls the scan did not see");
    assert!(found.iter().all(|(file, function, number, _)| {
        return !file.is_empty() && !function.is_empty() && *number > 0;
    }));
}

/// The crate's own targets reach these files,
/// so a read moved out of the module tree cannot escape the scan.
#[test]
fn the_read_is_part_of_this_crate() {
    let files: Vec<String> = crate::source_scan_tests::classified_sources(&source_root(), &real_roots())
        .expect("the crate's sources")
        .into_iter()
        .map(|source| return source.path)
        .collect();
    for file in THE_READ {
        assert!(files.iter().any(|path| return path == file), "{file} is not a source of this crate");
    }
}

/// The label the scan refuses on a line it must catch.
fn judged(text: &str, file: &str, function: &str) -> Judgement {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-read-within-one-file");
    crate::source_scan_tests::write_tree(root.as_path(), &[(file, text)]);
    for other in THE_READ {
        if *other != file {
            crate::source_scan_tests::write_tree(root.as_path(), &[(other, "pub fn nothing_here() {}\n")]);
        }
    }
    let found: Vec<Finding> = findings_in(root.as_path()).expect("the fixture's read");
    crate::test_support::remove(root.as_path());
    let judged: Vec<Judgement> = found
        .iter()
        .filter(|(_, name, _, _)| return name == function)
        .map(|(_, _, _, judgement)| return *judgement)
        .collect();
    assert_eq!(judged.len(), 1, "the scan judged {function} {} times: {found:?}", judged.len());
    let Some(one) = judged.first() else {
        panic!("the scan saw no storage call in {function}");
    };
    return *one;
}

/// The rules fire on the shapes they exist for,
/// so a scan that stopped matching would fail here.
#[test]
fn each_judgement_fires_on_its_own_source_text() {
    let entry: &str = "pub fn read_layout(prefix: &[std::ffi::OsString]) -> Option<()> {\n    let _ = std::fs::canonicalize(\"/a\");\n    let _ = prefix;\n    return None;\n}\n";
    let outside: &str = "pub fn a_new_step(directory: &std::path::Path) -> Option<()> {\n    let _ = std::fs::symlink_metadata(directory);\n    return None;\n}\n";
    let carried: &str = "pub fn dot_git_target(worktree: &std::path::Path) -> Option<()> {\n    let _ = std::fs::symlink_metadata(worktree);\n    return None;\n}\n";
    assert_eq!(judged(entry, "repo_read/repo_read_layout.rs", "read_layout"), Judgement::AnEntryPoint);
    assert_eq!(judged(outside, "repo_read/repo_read_layout.rs", "a_new_step"), Judgement::OutsideTheJob);
    assert_eq!(judged(carried, "repo_read/repo_read_layout.rs", "dot_git_target"), Judgement::Carried);
}
