//! What:
//!  The scan that keeps every storage step of the discarding guards under a ceiling.
//! Why:
//!  The guard's pass runs synchronously on the thread that forwarded the command
//!  (`invocation/wrapped_command.rs` calls the policy pass with no bound around it),
//!  so a `stat`, a read, a listing, a `canonicalize` or an open on a mount whose server stopped
//!  answering waits in the kernel with no end and holds the command with it.
//!  The owner's rule is "nothing should ever wait forever or indefinitely",
//!  and the guard's own design answers it with `work_within`: the storage step runs on a thread the
//!  pass waits for only until its ceiling, and no answer is a refusal.
//!  The package's `blocking_code_off_the_runtime` scan reads the bodies of `async fn`s, which is
//!  the code a runtime worker polls;
//!  this path is synchronous, so that scan never reaches it and this one does.
//!  Every storage call in the guard's files is named here with the ceiling it runs under,
//!  and a call that is not named fails the suite, as does an entry whose call is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every storage call in the guard's files appears in UNDER_A_CEILING with its ceiling
//! ```

use crate::source_scan_tests::{real_roots, source_root};
use std::path::PathBuf;

/// The files the discarding guard's own pass runs,
/// as `/`-separated paths relative to `src/native`.
/// Why:
///  These are the modules this branch adds or changes that decide whether a command may run:
///  the written-path reading, the dirty question, the clean's nested-repository reading, the save,
///  the two bounded metadata helpers and the policy itself.
///  A storage call added to one of them is one the guard takes on its own path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GUARD_FILES = ['command/command_clean_nested_repositories.rs', /* ... */];
/// ```
const GUARD_FILES: &[&str] = &[
    "command/command_clean.rs",
    "command/command_clean_nested_repositories.rs",
    "command/command_written_paths.rs",
    "command/rule_clean_interactive_nested_repositories.rs",
    "command/rule_linked_worktree.rs",
    "command/rule_worktree_dirty.rs",
    "conflict/discarded_nested_repository.rs",
    "foundation/metadata_of_a_path_within_a_ceiling.rs",
    "invocation/forwarded_command.rs",
    "invocation/guarded_forwarded_command.rs",
    "policy/policy_linked_worktree.rs",
    "policy/policy_linked_worktree_nested_repositories.rs",
    "policy/policy_linked_worktree_refused_overwrite.rs",
    "policy/policy_pass.rs",
    "policy/policy_transforms.rs",
    "repo_read/repository_facts_edits_the_status_hides.rs",
    "repo_read/repository_facts_paths_in_the_way.rs",
    "repo_read/repository_facts_written_paths.rs",
];

/// The calls that touch storage:
/// the standard library's file reads, writes, listings, metadata, renames and opens,
/// and the guard's own helpers that reach them.
/// Why:
///  A helper is the same wait with another name:
///  `fresh_saved_directory` makes a directory and `create_private_directory_all` makes them
///  all the way down,
///  and each is a kernel call that never returns on a stopped mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STORAGE_CALLS = ['std::fs::metadata(', 'File::open(', 'create_private_directory_all(', /* ... */];
/// ```
const STORAGE_CALLS: &[&str] = &[
    "std::fs::read(",
    "std::fs::read_to_string(",
    "std::fs::read_link(",
    "std::fs::read_dir(",
    "std::fs::write(",
    "std::fs::metadata(",
    "std::fs::symlink_metadata(",
    "std::fs::remove_file(",
    "std::fs::remove_dir(",
    "std::fs::remove_dir_all(",
    "std::fs::rename(",
    "std::fs::canonicalize(",
    "std::fs::create_dir(",
    "std::fs::create_dir_all(",
    "std::fs::copy(",
    "std::fs::hard_link(",
    "std::fs::try_exists(",
    "std::fs::OpenOptions",
    "std::fs::DirBuilder",
    "File::open(",
    "File::create(",
    "OpenOptions::new()",
    ".sync_all()",
    ".sync_data(",
    "create_private_directory_all(",
    "fresh_saved_directory(",
    "fresh_directory(",
    "sync_directory(",
    "read_plain_file",
    // The guard's own helpers that reach the calls above: a call to one is the same wait wherever it runs.
    "a_bare_repository_is(",
    "a_repository_shape(",
    "a_directory_is(",
    "directory_beneath(",
    "read_at_most(",
];

/// One storage call of the guard,
/// and the ceiling it runs under.
///  A `struct` is a record with named fields.
/// Why:
///  The file and the line's own text say where the call is;
///  the reason says which `work_within` job holds it, or which refusal a call that cannot wait
///  already answers with,
///  so a reader can check the claim without the scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnderACeiling = { file: string; call: string; why: string };
/// ```
struct UnderACeiling {
    /// The file, relative to `src/native`.
    file: &'static str,
    /// The line's code text, trimmed,
    ///  exactly as the scan reads it.
    call: &'static str,
    /// The ceiling the call runs under, and what turns no answer into a refusal.
    why: &'static str,
}

/// Every storage call of the guard, with the ceiling it runs under.
/// Why:
///  This is the list the brief asks for, kept where the scan reads it so it cannot drift from the
///  code:
///  a call that is not here fails, and an entry whose line changed fails too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNDER_A_CEILING: UnderACeiling[] = [{ file: '...', call: '...', why: '...' }];
/// ```
const UNDER_A_CEILING: &[UnderACeiling] = &[
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "let listing: std::fs::ReadDir = match std::fs::read_dir(directory.as_path()) {",
        why: "the walk job of `work_within` in `repositories_beneath_within_a_ceiling`, under `READ_CEILING`",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "if std::fs::symlink_metadata(directory.join(\".git\")).is_ok() {",
        why: "the same walk job: the shape of a repository is decided where it is found (`a_repository_shape`)",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "let Ok(head) = std::fs::symlink_metadata(directory.join(\"HEAD\")) else {",
        why: "the same walk job: a bare repository's `HEAD`, read as the entry itself (`a_bare_repository_is`)",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "let Ok(target) = std::fs::read_link(directory.join(\"HEAD\")) else {",
        why: "the same walk job: the target of a `HEAD` link, which Git reads before it calls a directory a repository (`a_head_git_reads`)",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "let Ok(found) = std::fs::metadata(path) else {",
        why: "the same walk job: a marker that must be a directory (`a_directory_is`)",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "let metadata: std::fs::Metadata = std::fs::symlink_metadata(current.as_path()).ok()?;",
        why: "the same walk job: every component of the path Git named (`directory_beneath`)",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "let Some(start_path) = directory_beneath(worktree_root, start) else {",
        why: "the same walk job: every component of the path Git named is read there",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "if let Some(bare) = a_repository_shape(directory.as_path()) {",
        why: "the same walk job: the shape of a repository is decided where the walk finds it",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "if a_bare_repository_is(directory) {",
        why: "the same walk job, through `a_repository_shape`: a bare repository's markers",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "if !a_directory_is(directory.join(\"objects\").as_path()) {",
        why: "the same walk job, through `a_bare_repository_is`: the `objects` marker",
    },
    UnderACeiling {
        file: "command/command_clean_nested_repositories.rs",
        call: "return a_directory_is(directory.join(\"refs\").as_path());",
        why: "the same walk job, through `a_bare_repository_is`: the `refs` marker",
    },
    UnderACeiling {
        file: "repo_read/repository_facts_written_paths.rs",
        call: "return std::fs::write(written_path.as_path(), handed);",
        why: "the job of `work_within` in `hand_the_bytes_to_git`, bounded by `WORKTREE_LOOKUP_CEILING`: the copy Git reads instead of the file the guard read",
    },
    UnderACeiling {
        file: "repo_read/repository_facts_written_paths.rs",
        call: "return std::fs::symlink_metadata(named.as_path());",
        why: "the job of `work_within` in `the_pathspec_copy_that_is_there`, bounded by `WORKTREE_LOOKUP_CEILING`: whether this invocation handed Git a copy at all",
    },
    UnderACeiling {
        file: "repo_read/repository_facts_written_paths.rs",
        call: "return read_at_most(opened.as_path(), PATHSPEC_FILE_READ_LIMIT);",
        why: "the job of `work_within` in `pathspecs_in_the_file`, bounded by `PATHSPEC_FILE_READ_CEILING`",
    },
    UnderACeiling {
        file: "command/rule_worktree_dirty.rs",
        call: "of_the_entry: std::sync::Arc::new(|path: &Path| return std::fs::symlink_metadata(path)),",
        why: "the job of `work_within` in `index_file_is_absent_read_with`, under `WORKTREE_LOOKUP_CEILING`; no answer refuses the command",
    },
    UnderACeiling {
        file: "command/rule_worktree_dirty.rs",
        call: "through_a_link: std::sync::Arc::new(|path: &Path| return std::fs::metadata(path)),",
        why: "the same `work_within` job: the reading `is_dir` makes of the Git directory",
    },
    UnderACeiling {
        file: "conflict/discarded_nested_repository.rs",
        call: "return std::fs::symlink_metadata(named.as_path());",
        why: "the job of `work_within` in `the_saved_copy_is_gone`, bounded by `WORKTREE_LOOKUP_CEILING`: a rename back's `ENOENT` is proved to be the copy's absence only by asking the copy",
    },
    UnderACeiling {
        file: "conflict/discarded_nested_repository.rs",
        call: "return fresh_saved_directory(common.as_path(), milliseconds);",
        why: "the job of `work_within` that makes the folder of saved copies, under `WORKTREE_LOOKUP_CEILING`; no answer stops the save naming the place",
    },
    UnderACeiling {
        file: "conflict/discarded_nested_repository.rs",
        call: "return create_private_directory_all(make.as_path());",
        why: "the job of `work_within` that makes the directory a copy goes in; no answer stops the save naming the repository",
    },
    UnderACeiling {
        file: "conflict/discarded_nested_repository.rs",
        call: "return std::fs::rename(source.as_path(), destination.as_path());",
        why: "the rename job of `work_within` that moves a repository into the saved copies; no answer leaves the move to the put-back, which says the repository may be at either place",
    },
    UnderACeiling {
        file: "conflict/discarded_nested_repository.rs",
        call: "return std::fs::rename(from.as_path(), to.as_path());",
        why: "the rename job of `work_within` that puts a saved repository back (`move_back_within_a_ceiling`); a destination that is not there is a move that never happened",
    },
    UnderACeiling {
        file: "conflict/discarded_nested_repository.rs",
        call: "return sync_directory(named.as_path());",
        why: "the job of `work_within` in `synced_within_a_ceiling`; no answer stops the save naming the directory",
    },
    UnderACeiling {
        file: "foundation/metadata_of_a_path_within_a_ceiling.rs",
        call: "return std::fs::symlink_metadata(named.as_path());",
        why: "the job of `work_within` in `symlink_metadata_within`, the bounded lookup every other reading uses",
    },
    UnderACeiling {
        file: "foundation/metadata_of_a_path_within_a_ceiling.rs",
        call: "return std::fs::metadata(named.as_path());",
        why: "the job of `work_within` in `metadata_through_links_within`, the bounded reading through a link",
    },
    UnderACeiling {
        file: "repo_read/repository_facts_paths_in_the_way.rs",
        call: "let Ok(metadata) = std::fs::symlink_metadata(full.as_path()) else {",
        why: "the naming job of `work_within` in `paths_in_the_way`, under `WORKTREE_LOOKUP_CEILING`",
    },
    UnderACeiling {
        file: "repo_read/repository_facts_paths_in_the_way.rs",
        call: "return std::fs::symlink_metadata(full.as_path()).is_ok_and(|metadata| return metadata.file_type().is_dir());",
        why: "the same `work_within` naming job: a collapsed directory's own entry",
    },
    UnderACeiling {
        file: "repo_read/repository_facts_written_paths.rs",
        call: "let file: std::fs::File = std::fs::File::open(path)?;",
        why: "the job of `work_within` in `pathspecs_in_the_file`, bounded by `PATHSPEC_FILE_READ_CEILING` and a byte limit",
    },
];

/// The code lines of one file,
/// trimmed,
/// with a line that is only a comment left out.
/// Why:
///  A storage call a comment or a doc says is text and runs nothing;
///  the guard's files carry many such mentions, and reading them as calls would reject code that
///  waits nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const codeLines = (text) => text.split('\n').map(l => l.trim()).filter(l => l && !l.startsWith('//'));
/// ```
fn code_lines(text: &str) -> Vec<String> {
    let mut code: Vec<String> = Vec::<String>::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        // A declaration names a call and makes none, and a line that is only a comment or an import runs nothing.
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("use ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("pub ")
            || trimmed.starts_with("async fn ")
        {
            continue;
        }
        code.push(trimmed.to_owned());
    }
    return code;
}

/// What:
///  Every storage call the guard's own files make, as `(file, line)`.
/// Why:
///  The set is what the entries are compared against;
///  a line that mentions a storage call in a comment never reaches it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const calls = () => GUARD_FILES.flatMap((file) => codeLines(read(file)).filter(isStorage).map((line) => [file, line]));
/// ```
fn calls() -> Vec<(&'static str, String)> {
    let root: PathBuf = source_root();
    let mut found: Vec<(&'static str, String)> = Vec::<(&'static str, String)>::new();
    for file in GUARD_FILES {
        let text: String = std::fs::read_to_string(root.join(file)).expect("a guard file");
        for line in code_lines(text.as_str()) {
            if STORAGE_CALLS.iter().any(|needle| return line.contains(needle)) {
                found.push((file, line));
            }
        }
    }
    return found;
}

/// What:
///  The calls no entry names,
///  each as `file: line`,
///  with the reason a new call has to be answered rather than ignored.
/// Why:
///  This is the finding a person reads: a storage step of the guard that no ceiling holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unexplained = (calls, entries) => calls.filter(([file, line]) => !entries.some((e) => e.file === file && e.call === line));
/// ```
fn unexplained(calls: &[(&'static str, String)], entries: &[UnderACeiling]) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    for (file, line) in calls {
        if !entries.iter().any(|entry| return entry.file == *file && entry.call == line.as_str()) {
            found.push(format!("{file}: {line}"));
        }
    }
    return found;
}

/// What:
///  The entries whose call the files no longer hold,
///  each as `file: call`.
/// Why:
///  An entry that matches nothing is a claim about code that is gone,
///  and the list must shrink with the code instead of describing a past reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = (calls, entries) => entries.filter((e) => !calls.some(([file, line]) => file === e.file && line === e.call));
/// ```
fn stale(calls: &[(&'static str, String)], entries: &[UnderACeiling]) -> Vec<String> {
    let mut gone: Vec<String> = Vec::<String>::new();
    for entry in entries {
        if !calls.iter().any(|(file, line)| return *file == entry.file && line.as_str() == entry.call) {
            gone.push(format!("{}: {} ({})", entry.file, entry.call, entry.why));
        }
    }
    return gone;
}

/// What:
///  Every storage call the guard's files make runs under a ceiling.
/// Why:
///  A call that is not named is one nobody has shown to be bounded,
///  and the guard's pass has no bound around it:
///  a `stat` on a mount that stopped answering would hold the command with no end.
///  The names of the files are a list in this test,
///  so a new guard file is added here with its calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unexplained(calls(), UNDER_A_CEILING)).toEqual([]);
/// ```
#[test]
fn every_storage_step_of_the_guard_runs_under_a_ceiling() {
    let found: Vec<(&'static str, String)> = calls();
    let unheld: Vec<String> = unexplained(found.as_slice(), UNDER_A_CEILING);
    assert!(
        unheld.is_empty(),
        "every storage step of the discarding guard runs under a ceiling (`work_within`, the ceiling in \
         `service/wait_ceilings.rs`), or is named in UNDER_A_CEILING with the ceiling that holds it:\n{}",
        unheld.join("\n")
    );
}

/// What:
///  Every entry names a call the guard still makes.
/// Why:
///  The list is the reading of the code, not a history of it;
///  an entry whose call changed or went is removed with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(stale(calls(), UNDER_A_CEILING)).toEqual([]);
/// ```
#[test]
fn every_entry_names_a_storage_call_the_guard_still_makes() {
    let found: Vec<(&'static str, String)> = calls();
    let gone: Vec<String> = stale(found.as_slice(), UNDER_A_CEILING);
    assert!(
        gone.is_empty(),
        "remove these entries from UNDER_A_CEILING, their code is gone:\n{}",
        gone.join("\n")
    );
}

/// What:
///  The files this scan reads are the ones the branch's guard lives in,
///  and the scan reads each of them.
/// Why:
///  A file list that has drifted from the code would pass while a guard file went unread;
///  every named file is read here, and a file that does not exist fails loudly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(GUARD_FILES.every((file) => exists(read(file)))).toBe(true);
/// ```
#[test]
fn the_scan_reads_every_guard_file() {
    let root: PathBuf = source_root();
    for file in GUARD_FILES {
        let text: String = std::fs::read_to_string(root.join(file)).expect("a guard file the scan names");
        assert!(!text.is_empty(), "{file} is empty");
    }
    // The crate's own roots classify these files, so a renamed module fails here rather than being skipped.
    let sources: Vec<String> = crate::source_scan_tests::classified_sources(&root, &real_roots())
        .expect("the crate's sources")
        .into_iter()
        .map(|file| return file.path)
        .collect();
    for file in GUARD_FILES {
        assert!(
            sources.iter().any(|path| return path == file),
            "{file} is not a release source of this crate, so the list names a file the compiler never reads"
        );
    }
}

/// What:
///  Every entry says which ceiling holds its call.
/// Why:
///  The entry is the claim a reader checks against the code,
///  and a reason that names no ceiling is no claim at all:
///  it would excuse a call nobody had shown to be bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(UNDER_A_CEILING.every((entry) => /CEILING|work_within|walk job/.test(entry.why))).toBe(true);
/// ```
#[test]
fn every_entry_states_the_ceiling_that_holds_it() {
    for entry in UNDER_A_CEILING {
        let states_a_ceiling: bool = entry.why.contains("CEILING")
            || entry.why.contains("work_within")
            || entry.why.contains("walk job");
        assert!(states_a_ceiling, "{}: {} ({})", entry.file, entry.call, entry.why);
    }
}
