//! What:
//!  What a removal deleted of the files Git ignores,
//!  of the files a worktree nested inside it held,
//!  written where a person reads it afterwards.
//! Why:
//!  The owner's decision of 2026-11-12T13:04:14
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "How far the protection of a worktree being removed reaches") is "Keep Git work only":
//!  a removal keeps the work Git names and takes everything else a person put there,
//!  and each thing it takes is logged.
//!  That is the files Git ignores,
//!  also whole ignored directories,
//!  and the untracked and ignored files of a worktree nested inside the one being removed.
//!  Files Git ignores are ordinary state:
//!  build output,
//!  a local environment file,
//!  a folder of downloaded packages.
//!  A nested worktree's files are invisible to real Git,
//!  whose own ignore rules of the outer worktree hide the whole nested directory.
//!  A removal takes both and no command afterwards can say what was there,
//!  so the record is made before the deletion and names them.
//!  One entry is one line of the record, except that a directory is one entry:
//!  a `node_modules` or a `target` holds tens of thousands of files,
//!  a line for each would bury the one file a person wrote,
//!  and the name of the directory is what a person needs to know that its whole content went.
//!  The names the record does not show are counted, never dropped silently,
//!  and the list is bounded,
//!  so the record stays one readable line whatever the worktree held.
//!  The record is the wrapper's own log,
//!  `monochromatic-git-policy-cli-worktree-deletions.jsonl`,
//!  in the repository's common Git directory beside the audit log,
//!  with the same rotation and the same older generations
//!  (`log_lines_appended_without_a_lock.rs`, `log_generations_kept_to_a_bound.rs`).
//!  A record that cannot be written costs the record and never the removal:
//!  a removal that stopped because its log could not be appended would be a worse defect than
//!  the silence the log exists to end.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listed = RemovalDeletions.of(git, commonDir, worktree);
//! listed.record(commonDir, worktree, ['worktree', 'remove', worktree]); // throws nothing
//! ```

use crate::diagnostic_log::debug;
use crate::iso_time::format_iso_milliseconds;
use crate::json_record::{quote, string_array};
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use crate::path_bytes_in_records::hex_of_path;
use crate::service_protocol::RELEASE;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_ladder_target::checkout_environment;
use crate::worktree_landed_registrations::registrations_of;
use crate::worktree_removal_second_look::worktrees_inside;
use crate::worktree_remove_registry::registration_of;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalDeletions";

/// The name of the log a removal's deleted files are written to,
/// inside the repository's common Git directory.
/// Why:
///  This record is the first of its kind,
///  so an earlier release wrote it under no other name and there is nothing to rename
///  (`wrapper_names_old_and_new.rs`).
pub const DELETIONS_LOG: &str = "monochromatic-git-policy-cli-worktree-deletions.jsonl";

/// The query that names every entry Git ignores in a worktree,
/// each whole ignored directory as one name.
/// Why:
///  These are the words `git clean` itself deletes by,
///  so what this record names is what a removal takes,
///  and the ignore rules are Git's own and not a second reading of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IGNORED_WORDS = ['ls-files', '--others', '--ignored', '--exclude-standard', '--directory', '--no-empty-directory', '-z'];
/// ```
const IGNORED_WORDS: [&str; 7] = ["ls-files", "--others", "--ignored", "--exclude-standard", "--directory", "--no-empty-directory", "-z"];

/// The query that names every file a worktree holds that Git does not track,
/// the files its ignore rules call ignored included,
/// each whole directory as one name.
/// Why:
///  A nested worktree goes with the worktree around it,
///  and its own files are exactly what no rule of the outer worktree ever listed:
///  `--exclude-standard` is left out on purpose,
///  because a nested worktree's build output is untracked *and* ignored there
///  and the record has to name it all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNTRACKED_WORDS = ['ls-files', '--others', '--directory', '--no-empty-directory', '-z'];
/// ```
const UNTRACKED_WORDS: [&str; 5] = ["ls-files", "--others", "--directory", "--no-empty-directory", "-z"];

/// How many entries one record names for one worktree before it counts the rest.
/// Why:
///  A person reads one line,
///  and a worktree can hold thousands of ignored files beside the folders that hold most of them.
///  Twenty still shows a whole ordinary set of local files (an environment file, a database, a log, a cache folder),
///  and a set larger than that is a fact about the worktree rather than a list anyone reads line by line.
///  The entries past the bound are counted in the same record,
///  so the reader learns the size of what is left out.
pub const SHOWN_ENTRIES: usize = 20;

/// What one listing of a worktree found,
/// and how far it was shown.
/// Why:
///   A listing is read before the removal and written after it,
///  so the answer travels as a value the caller records when the files are gone.
///  The total is kept apart from the shown names because a record that showed twenty of three
///  says something the total alone does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeletedEntries = { shown: string[]; total: number; unreadable?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeletedEntries {
    /// The entry names Git listed,
    /// at most `SHOWN_ENTRIES` of them,
    /// each with the separator that ends a whole directory.
    pub shown: Vec<String>,
    /// How many entries Git listed,
    /// the shown ones included.
    pub total: usize,
    /// Why the listing could not be read,
    /// when it could not:
    /// the removal then goes on,
    /// and the record says the names are unknown rather than empty.
    pub unreadable: Option<String>,
}

impl DeletedEntries {
    /// What:
    ///  The worktree's ignored entries as Git names them.
    /// Why:
    ///   The question is Git's,
    ///  so a file the person's own `.gitignore` ignores is one entry and a file Git tracks is none.
    ///  A listing that cannot be read is not a failure of the removal:
    ///  the caller removes anyway, and the record says the names were not read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// DeletedEntries.ignoredOf(git, worktree, admin)
    /// ```
    pub fn ignored_of(git: &GitContext, worktree: &Path, admin: Option<&Path>) -> DeletedEntries {
        return list(git, worktree, admin, IGNORED_WORDS.as_slice(), "the files it ignores");
    }

    /// What:
    ///  Every file one worktree holds that Git does not track,
    ///  the ones its own ignore rules hide included.
    /// Why:
    ///   A nested worktree's files are the ones a removal of the worktree around it takes unseen,
    ///  and they are untracked and ignored alike.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// DeletedEntries.untrackedAndIgnoredOf(git, nested, itsAdmin)
    /// ```
    pub fn untracked_and_ignored_of(git: &GitContext, worktree: &Path, admin: Option<&Path>) -> DeletedEntries {
        return list(git, worktree, admin, UNTRACKED_WORDS.as_slice(), "the files Git does not track");
    }

    /// What:
    ///  The entries of one `-z` listing,
    ///  the first `SHOWN_ENTRIES` of them kept.
    /// Why:
    ///   The listing is split on NUL because a path may hold any byte but that one,
    ///  and an empty name is the end of the output rather than an entry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// DeletedEntries.fromListing('build/\0.env\0') // { shown: ['build/', '.env'], total: 2 }
    /// ```
    fn from_listing(listing: &[u8]) -> DeletedEntries {
        let mut found: DeletedEntries = DeletedEntries::default();
        for name in listing.split(|byte: &u8| return *byte == 0).filter(|name: &&[u8]| return !name.is_empty()) {
            found.total += 1;
            if found.shown.len() < SHOWN_ENTRIES {
                found.shown.push(String::from_utf8_lossy(name).into_owned());
            }
        }
        return found;
    }

    /// What:
    ///  The names this record leaves out,
    ///  which is none for a worktree small enough to show whole.
    /// Why:
    ///   A reader learns how much of the set it did not see,
    ///  and a record with nothing left out says nothing about being incomplete.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries.more() // 0 | n
    /// ```
    pub fn more(&self) -> usize {
        return self.total.saturating_sub(self.shown.len());
    }
}

/// What:
///  One worktree nested inside the worktree being removed,
///  and the files of it that go.
/// Why:
///  The registration is what says a worktree is there,
///  and its files are the ones no rule of the outer worktree ever listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NestedWorktree = { worktree: string; entries: DeletedEntries };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedWorktree {
    /// The nested worktree's directory.
    pub worktree: PathBuf,
    /// Its untracked and ignored files.
    pub entries: DeletedEntries,
}

/// What:
///  Everything one removal deletes that the repository's own rules do not name.
/// Why:
///  The record is made of these two groups:
///  what Git ignores in the worktree itself,
///  and what each worktree nested inside it held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalDeletions = { ignored: DeletedEntries; nested: NestedWorktree[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RemovalDeletions {
    /// The worktree's own ignored entries.
    pub ignored: DeletedEntries,
    /// The worktrees this repository registers inside the one being removed.
    pub nested: Vec<NestedWorktree>,
}

impl RemovalDeletions {
    /// What:
    ///  Everything one removal is about to delete,
    ///  read before it deletes anything.
    /// Why:
    ///   Nothing can be listed after the worktree is gone,
    ///  and a registration list that cannot be read is not a failure of the removal:
    ///  the record then says the nested worktrees were not looked for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// RemovalDeletions.of(git, commonDir, worktree)
    /// ```
    pub fn of(git: &GitContext, common_dir: &Path, worktree: &Path) -> RemovalDeletions {
        // The worktree's own Git directory, which a linked worktree keeps in the common directory rather than at its
        //  root: the listing asks Git inside the worktree, so it is told which repository that is.
        let admin: Option<PathBuf> = registration_of(common_dir, worktree, worktree.as_os_str()).ok().map(|found| return found.admin);
        let mut found: RemovalDeletions = RemovalDeletions { ignored: DeletedEntries::ignored_of(git, worktree, admin.as_deref()), nested: Vec::new() };
        match registrations_of(common_dir) {
            Ok(registered) => {
                for nested in worktrees_inside(worktree, registered.as_slice()) {
                    let entries: DeletedEntries = DeletedEntries::untracked_and_ignored_of(git, nested.worktree.as_path(), Some(nested.admin.as_path()));
                    found.nested.push(NestedWorktree { worktree: nested.worktree.clone(), entries });
                }
            }
            Err(reason) => {
                found.ignored.unreadable = Some(match found.ignored.unreadable {
                    Some(already) => format!("{already}; and the registrations could not be read ({reason}), so no worktree nested inside this one was looked for"),
                    None => format!("the registrations could not be read ({reason}), so no worktree nested inside this one was looked for"),
                });
            }
        }
        return found;
    }

    /// What:
    ///  Whether there is anything to record at all.
    /// Why:
    ///   An ordinary clean removal deleted nothing a person could miss,
    ///  and a record of nothing would fill the log with lines nobody needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listed.anything() // ignored.total > 0 || nested.length > 0 || ignored.unreadable !== undefined
    /// ```
    pub fn anything(&self) -> bool {
        return !self.nested.is_empty() || self.ignored.total > 0 || self.ignored.unreadable.is_some();
    }
}

/// What:
///  One `-z` listing of a worktree,
///  from the words the caller gives.
/// Why:
///   The two listings differ in their words and in nothing else,
///  so the split, the bound and the excuse for a listing that could not be read are written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// list(git, worktree, admin, IGNORED_WORDS, 'the files it ignores')
/// ```
fn list(git: &GitContext, worktree: &Path, admin: Option<&Path>, words: &[&str], what: &str) -> DeletedEntries {
    let request: GitRequest = GitRequest {
        cwd: worktree.to_path_buf(),
        arguments: words.iter().map(OsString::from).collect(),
        without_prefix: true,
        environment: admin.map_or_else(Vec::new, |git_dir| return checkout_environment(git_dir, worktree)),
        ..GitRequest::default()
    };
    let output: GitOutput = match run_git(git, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return DeletedEntries {
                unreadable: Some(format!("Git could not be started to list {what} in {} ({error})", worktree.display())),
                ..DeletedEntries::default()
            };
        }
    };
    if output.code != Some(0) {
        let said: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
        return DeletedEntries {
            unreadable: Some(format!("Git could not list {what} in {} ({said})", worktree.display())),
            ..DeletedEntries::default()
        };
    }
    return DeletedEntries::from_listing(output.stdout.as_slice());
}

/// What:
///  Write one record of what a removal deleted.
/// Why:
///   The record answers four questions a person asks afterwards:
///  what was deleted,
///  from which worktree,
///  when,
///  and by which command.
///  The worktree is written as text and as the bytes it is,
///  because a path that is not UTF-8 would otherwise reach a reader changed.
///  A command that is not UTF-8 is written the same way.
///  Nothing here changes the outcome of the removal:
///  a failed write is a debug line, and the files are gone either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordRemovalDeletions({ commonDir, worktree, command, deletions });
/// ```
pub fn record_removal_deletions(common_dir: &Path, worktree: &Path, command: &[OsString], deletions: &RemovalDeletions) {
    if !deletions.anything() {
        return;
    }
    let line: String = deletions_line(worktree, command, deletions, now_millis());
    let target: PathBuf = common_dir.join(DELETIONS_LOG);
    if let Err(error) = append_line(target.as_path(), line.as_bytes(), &UNPINNED_LOG) {
        debug(TAG, format!("the record of the files {} deleted could not be written to {}: {error}", worktree.display(), target.display()).as_str());
    }
}

/// What:
///  The words of one removal,
///  which every record of a deletion names.
/// Why:
///   The wrapper deletes a worktree by `git worktree remove` in one route and by a destroy that stands for it in
///  another,
///  so the command a person reads is the same one in both,
///  with the `--force` that was given when one was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removalWords('/x', false) // ['worktree', 'remove', '/x']
/// ```
pub fn removal_words(target: &Path, force: bool) -> Vec<OsString> {
    let mut words: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("remove")];
    if force {
        words.push(OsString::from("--force"));
    }
    words.push(target.to_path_buf().into_os_string());
    return words;
}

/// What:
///  One record's line,
///  with the clock given by its caller so a control can pin it.
/// Why:
///   One builder for the line keeps every record the same shape,
///  and a control that wrote a clock of its own would test the clock rather than the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deletionsLine(worktree, command, deletions, timeMillis) === '{"schemaVersion":1,...}\n'
/// ```
pub(crate) fn deletions_line(worktree: &Path, command: &[OsString], deletions: &RemovalDeletions, time_millis: u128) -> String {
    let mut line: String = String::with_capacity(512);
    line.push_str("{\"schemaVersion\":1");
    let stamp: i64 = i64::try_from(time_millis).unwrap_or(i64::MAX);
    let _ = write!(line, ",\"time\":{},\"release\":{}", quote(format_iso_milliseconds(stamp).as_str()), quote(RELEASE));
    let _ = write!(line, ",\"kind\":{},\"worktree\":{}", quote("worktree-removal"), quote(worktree.to_string_lossy().as_ref()));
    let _ = write!(line, ",\"worktreeHex\":{}", quote(hex_of_path(worktree).as_str()));
    let words: Vec<String> = command.iter().map(|one| return one.to_string_lossy().into_owned()).collect();
    let _ = write!(line, ",\"command\":{}", string_array(words.as_slice()));
    let _ = write!(line, ",\"ignored\":{},\"ignoredShown\":{},\"ignoredTotal\":{}", string_array(&deletions.ignored.shown), deletions.ignored.shown.len(), deletions.ignored.total);
    let _ = write!(line, ",\"nested\":[{}]", nested_objects(&deletions.nested));
    if let Some(reason) = &deletions.ignored.unreadable {
        let _ = write!(line, ",\"unreadable\":{}", quote(reason.as_str()));
    }
    line.push_str("}\n");
    return line;
}

/// What:
///  The nested worktrees of one record as JSON objects,
///  each with its path and its own entries.
/// Why:
///   A person reads which nested worktree went and what it held,
///  and the entries follow the same bound and the same counting as the outer worktree's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestedObjects([{ worktree: '/x/n', entries: { shown: ['a.txt'], total: 1 } }])
/// ```
fn nested_objects(nested: &[NestedWorktree]) -> String {
    let mut objects: Vec<String> = Vec::with_capacity(nested.len());
    for one in nested {
        let mut object: String = String::with_capacity(256);
        let _ = write!(
            object,
            "{{\"worktree\":{},\"worktreeHex\":{},\"files\":{},\"filesShown\":{},\"filesTotal\":{}",
            quote(one.worktree.to_string_lossy().as_ref()),
            quote(hex_of_path(one.worktree.as_path()).as_str()),
            string_array(&one.entries.shown),
            one.entries.shown.len(),
            one.entries.total
        );
        if let Some(reason) = &one.entries.unreadable {
            let _ = write!(object, ",\"unreadable\":{}", quote(reason.as_str()));
        }
        object.push('}');
        objects.push(object);
    }
    return objects.join(",");
}

/// The clock a record carries,
/// in milliseconds since the Unix epoch,
/// or zero where the clock cannot say.
/// Why:
///   A machine with a clock before 1970 writes the same record with no time rather than failing the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now()
/// ```
fn now_millis() -> u128 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_) => 0,
    };
}

/// Controls of what a removal deleted stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_deletions_tests.rs"]
mod tests;
