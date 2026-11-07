//! What:
//!  Answers,
//! without a Git process,
//! to the exact read-only Git command lines the wrapper itself issues:
//!  `ls-files --stage`,
//! `ls-files --unmerged`,
//! `diff-index --cached`,
//! `diff --cached --name-only`,
//! `diff-tree --root -r -m`
//!  and `show --no-patch --format=%s`.
//! Why:
//!  Each is a start of Git for a fact two in-process readers (the index,
//! the object database) can produce byte for byte.
//!  The reader takes the same argument list the caller would give Git and returns Git's standard output and exit code,
//!  or `None` for any command line,
//! repository or configuration it does not fully understand,
//! so a caller tries it first
//!  and starts Git on `None`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = answerInProcess({ args, directory, variables }); // undefined: start Git
//! ```

#![cfg(unix)]

use crate::repo_read_config_scan::{ScopeEnvironment, content_key, entries_avoid_keys, file_avoids_keys, last_variable, outer_scopes_avoid_keys};
use crate::repo_read_config_text::{CONFIG_TEXT_LIMIT, parse_config_text};
use crate::repo_read_diff::{PathFilter, diff_sorted, name_lines, parse_diff_filter, raw_lines, select};
use crate::repo_read_layout::{RepoLayout, env_config_avoids, read_layout_with};
use crate::repo_read_object::{ObjectKind, from_hex, name_length, to_hex};

use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_refs::{config_list, for_each_ref, remote_url, rev_parse, symbolic_ref};
use crate::index_cache_tree::CacheTree;
use crate::repo_read_tree_cached::changes_with_cache;
use crate::repo_read_tree::{FlatEntry, flatten_pair, flatten_tree_shared, parse_commit, subject_of};
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;
pub use crate::repo_read_query_request_answer_and_context::{Answer, Query};
use crate::repo_read_query_request_answer_and_context::{Context, Parsed, flags_are, parse};

/// Whether `byte` is a newline.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether a configuration key,
/// in any letter case,
/// is one that could change the output of a content command.
fn content_key_of_any_case(key: &str) -> bool {
    return content_key(key.to_ascii_lowercase().as_str());
}

/// The environment Git would see,
/// without the two variables this reader handles itself.
fn environment(
    base: Vec<(OsString, OsString)>,
    variables: &[(OsString, OsString)],
) -> (Vec<(OsString, OsString)>, Option<OsString>, Option<OsString>) {
    let mut all: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in base {
        let mut replaced: bool = false;
        for (added, _) in variables {
            if *added == name {
                replaced = true;
                break;
            }
        }
        if !replaced {
            all.push((name, value));
        }
    }
    all.extend(variables.iter().cloned());
    // The last assignment wins, as it does for the child's environment.
    let index: Option<OsString> = last_variable(all.as_slice(), "GIT_INDEX_FILE");
    let objects: Option<OsString> = last_variable(all.as_slice(), "GIT_OBJECT_DIRECTORY");
    let mut kept: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in all {
        if name != "GIT_INDEX_FILE" && name != "GIT_OBJECT_DIRECTORY" {
            kept.push((name, value));
        }
    }
    return (kept, index, objects);
}

/// Whether no configuration file Git reads for these queries sets a key that could change the output.
fn content_config_is_plain(layout: &RepoLayout, scope: &ScopeEnvironment) -> Option<bool> {
    let path: PathBuf = layout.common_dir.join("config");
    let bytes: Vec<u8> = crate::private_storage::read_regular_file(path.as_path(), CONFIG_TEXT_LIMIT as u64).ok()?;
    let entries = parse_config_text(bytes.as_slice())?;
    if !entries_avoid_keys(entries.as_slice(), path.as_path(), scope.home.as_deref(), 0, &content_key)? {
        return Some(false);
    }
    // The per-worktree file is where `git sparse-checkout` writes `core.sparseCheckout`; it is scanned whenever it
    // exists, also when the extension that makes Git read it is off, which only makes the readers decline more often.
    if !file_avoids_keys(layout.git_dir.join("config.worktree").as_path(), scope.home.as_deref(), 0, &content_key)? {
        return Some(false);
    }
    return outer_scopes_avoid_keys(scope, &content_key);
}

/// `ls-files --stage -z` and `ls-files --unmerged -z`.
fn list_files(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    let unmerged: bool = parsed.flags.contains(&"--unmerged");
    let required: [&str; 2] = [if unmerged { "--unmerged" } else { "--stage" }, "-z"];
    if !flags_are(&parsed.flags, &required, &["--full-name"]) || !parsed.operands.is_empty() || !context.layout.prefix.is_empty() {
        return None;
    }
    let filter: PathFilter = PathFilter::parse(parsed.specs.as_slice(), true)?;
    let mut stdout: Vec<u8> = Vec::new();
    for entry in context.index()? {
        if (unmerged && entry.stage == 0) || !filter.matches(entry.path.as_slice()) {
            continue;
        }
        stdout.extend_from_slice(format!("{:06o} {} {}\t", entry.mode, to_hex(entry.id.as_slice()), entry.stage).as_bytes());
        stdout.extend_from_slice(entry.path.as_slice());
        stdout.push(0);
    }
    return Some(Answer { stdout, stderr: Vec::new(), code: 0 });
}

/// `diff-index --cached` and `diff --cached --name-only`.
fn diff_cached(context: &Context, parsed: &Parsed<'_>, porcelain: bool) -> Option<Answer> {
    // The exit status alone, for the checks of the wrapper that print nothing.
    if !porcelain && parsed.flags.contains(&"--quiet") {
        return crate::repo_read_query_quiet::diff_index_quiet(context, parsed);
    }
    let raw: bool = parsed.flags.contains(&"--raw");
    let mut required: Vec<&str> = vec!["--cached", "-z"];
    required.push(if raw { "--raw" } else { "--name-only" });
    if raw {
        required.push("--no-abbrev");
    }
    let mut statuses: Option<Vec<u8>> = None;
    let mut others: Vec<&str> = Vec::new();
    for flag in &parsed.flags {
        match flag.strip_prefix("--diff-filter=") {
            // A second `--diff-filter` adds its letters in Git; the reader leaves that spelling to it.
            Some(letters) if !porcelain && statuses.is_none() => statuses = Some(parse_diff_filter(letters)?),
            Some(_) => return None,
            _ => others.push(flag),
        }
    }
    let allowed: &[&str] = if porcelain { &[] } else { &["--no-renames"] };
    if (porcelain && raw) || !flags_are(&others, &required, allowed) || parsed.operands.len() != 1 || (porcelain && !parsed.specs.is_empty()) {
        return None;
    }
    let filter: PathFilter = PathFilter::parse(parsed.specs.as_slice(), context.layout.prefix.is_empty())?;
    let database: ObjectDatabase = context.database()?;
    let tree: Vec<u8> = context.tree_of(&database, parsed.operands[0])?;
    let (index, cache): (Vec<FlatEntry>, Option<CacheTree>) = context.resolved_index_with_tree()?;
    // The cache-tree names the directories the index has not changed, and those are left unread; without one, or when its walk
    //  declines, the whole tree is read.
    let cached = match cache.as_ref() {
        Some(nodes) => changes_with_cache(&database, tree.as_slice(), context.layout.object_format, nodes, index.as_slice()),
        None => None,
    };
    let changes = match cached {
        Some(found) => found,
        None => {
            let old: Rc<Vec<FlatEntry>> = flatten_tree_shared(&database, tree.as_slice(), context.layout.object_format)?;
            diff_sorted(old.as_slice(), index.as_slice())?
        }
    };
    // The porcelain pairs an added and a deleted path as a rename; plain lists cannot say which pairs it would choose.
    if porcelain {
        let mut added: bool = false;
        let mut deleted: bool = false;
        for change in &changes {
            added = added || change.status == b'A';
            deleted = deleted || change.status == b'D';
        }
        if added && deleted {
            return None;
        }
    }
    let kept = select(changes, &filter, statuses.as_deref());
    let stdout: Vec<u8> = if raw { raw_lines(kept.as_slice(), context.zero().as_str()) } else { name_lines(kept.as_slice()) };
    return Some(Answer { stdout, stderr: Vec::new(), code: 0 });
}

/// `diff-tree --root --no-commit-id -r -z -m --no-renames --no-abbrev <commit>` for a commit with at most one parent.
fn diff_tree(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    let required: [&str; 7] = ["--root", "--no-commit-id", "-r", "-z", "-m", "--no-renames", "--no-abbrev"];
    if !flags_are(&parsed.flags, &required, &[]) || parsed.operands.len() != 1 || !parsed.specs.is_empty() {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let name: Vec<u8> = from_hex(parsed.operands[0], context.layout.object_format)?;
    let object = database.read(name.as_slice())?;
    if object.kind != ObjectKind::Commit {
        return None;
    }
    let commit = parse_commit(object.bytes.as_slice(), context.layout.object_format)?;
    if commit.parents.len() > 1 {
        return None;
    }
    let changes = match commit.parents.first() {
        Some(parent) => {
            let parent_object = database.read(parent.as_slice())?;
            if parent_object.kind != ObjectKind::Commit {
                return None;
            }
            let parent_tree = parse_commit(parent_object.bytes.as_slice(), context.layout.object_format)?.tree;
            // Only the entries that differ are listed, so a subtree both trees share is not read: a commit that changes one
            //  file of a large checkout reads the few trees along the path, not every directory of both sides.
            let (before, after): (Vec<FlatEntry>, Vec<FlatEntry>) = flatten_pair(&database, parent_tree.as_slice(), commit.tree.as_slice(), context.layout.object_format)?;
            diff_sorted(before.as_slice(), after.as_slice())?
        }
        None => {
            let new: Rc<Vec<FlatEntry>> = flatten_tree_shared(&database, commit.tree.as_slice(), context.layout.object_format)?;
            diff_sorted(&[], new.as_slice())?
        }
    };
    return Some(Answer { stdout: raw_lines(changes.as_slice(), context.zero().as_str()), stderr: Vec::new(), code: 0 });
}

/// `diff-tree -r --name-only -z [--no-renames] <old> <new>` and `diff-tree --no-commit-id --name-only -r -z <commit>`:
/// the paths
///  that differ between two trees or commits,
/// or between a commit with exactly one parent and that parent.
fn diff_tree_names(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !parsed.specs.is_empty() {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let (old, new): (Vec<u8>, Vec<u8>) = match parsed.operands.as_slice() {
        [pair_old, pair_new] => {
            if !flags_are(&parsed.flags, &["-r", "--name-only", "-z"], &["--no-renames"]) {
                return None;
            }
            (context.tree_of(&database, pair_old)?, context.tree_of(&database, pair_new)?)
        }
        [single] => {
            if !flags_are(&parsed.flags, &["--no-commit-id", "--name-only", "-r", "-z"], &[]) {
                return None;
            }
            let name: Vec<u8> = from_hex(single, context.layout.object_format)?;
            let object = database.read(name.as_slice())?;
            if object.kind != ObjectKind::Commit {
                return None;
            }
            let commit = parse_commit(object.bytes.as_slice(), context.layout.object_format)?;
            // A root commit and a merge commit print nothing or per-parent lists in Git, which the reader leaves to it.
            let [parent] = commit.parents.as_slice() else { return None };
            let parent_object = database.read(parent.as_slice())?;
            if parent_object.kind != ObjectKind::Commit {
                return None;
            }
            (parse_commit(parent_object.bytes.as_slice(), context.layout.object_format)?.tree, commit.tree)
        }
        _ => return None,
    };
    let (before, after): (Vec<FlatEntry>, Vec<FlatEntry>) = flatten_pair(&database, old.as_slice(), new.as_slice(), context.layout.object_format)?;
    let changes = diff_sorted(before.as_slice(), after.as_slice())?;
    return Some(Answer { stdout: name_lines(changes.as_slice()), stderr: Vec::new(), code: 0 });
}

/// `show --no-patch --format=%s <commit>`.
fn show_subject(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--no-patch", "--format=%s"], &[]) || parsed.operands.len() != 1 || !parsed.specs.is_empty() {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let name: Vec<u8> = from_hex(parsed.operands[0], context.layout.object_format)?;
    let object = database.read(name.as_slice())?;
    if object.kind != ObjectKind::Commit {
        return None;
    }
    let commit = parse_commit(object.bytes.as_slice(), context.layout.object_format)?;
    let mut stdout: Vec<u8> = subject_of(commit.message.as_slice())?;
    stdout.push(b'\n');
    return Some(Answer { stdout, stderr: Vec::new(), code: 0 });
}

/// `cat-file --batch` fed full object names,
/// one per line:
/// for each,
/// `<name> <kind> <size>`,
/// the content and a line feed.
fn batch_objects(context: &Context, parsed: &Parsed<'_>, input: Option<&[u8]>) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--batch"], &[]) || !parsed.operands.is_empty() || !parsed.specs.is_empty() {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let mut stdout: Vec<u8> = Vec::new();
    let text: &[u8] = input?.strip_suffix(b"\n")?;
    for line in text.split(is_newline) {
        let name: Vec<u8> = from_hex(line, context.layout.object_format)?;
        // A name the store lacks is Git's `missing` line, which the reader leaves to it.
        let object = database.read(name.as_slice())?;
        stdout.extend_from_slice(format!("{} {} {}\n", to_hex(name.as_slice()), object.kind.word(), object.bytes.len()).as_bytes());
        stdout.extend_from_slice(object.bytes.as_slice());
        stdout.push(b'\n');
    }
    return Some(Answer { stdout, stderr: Vec::new(), code: 0 });
}

/// What:
///  The answer to `query`,
/// or `None` when Git must be started.
/// Why:
///   This is the one entry point;
/// every reason to decline (an unknown command line,
/// a repository or configuration
///  the readers do not fully understand,
/// a missing object) is the same `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = answer(query); // undefined: start Git
/// ```
pub fn answer(query: &Query<'_>) -> Option<Answer> {
    let mut base: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    base.extend(query.variables.iter().cloned());
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(base.as_slice());
    return answer_with(query, std::env::vars_os().collect(), &scope);
}

/// The same with the process environment and the configuration scope facts given,
/// so a control does not depend on its host.
pub fn answer_with(query: &Query<'_>, base: Vec<(OsString, OsString)>, scope: &ScopeEnvironment) -> Option<Answer> {
    let mut start: PathBuf = match query.directory {
        Some(directory) => directory.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };
    let parsed: Parsed<'_> = parse(query.arguments, &mut start)?;
    if !matches!(parsed.command, "ls-files" | "diff-index" | "diff" | "diff-tree" | "show" | "config" | "remote" | "rev-parse" | "for-each-ref" | "symbolic-ref" | "cat-file") {
        return None;
    }
    let (variables, index_variable, objects_variable) = environment(base, query.variables);
    let layout: RepoLayout = read_layout_with(start.as_path(), variables.as_slice(), scope)?;
    let index_path: PathBuf = match index_variable {
        Some(file) => start.join(file),
        None => layout.git_dir.join("index"),
    };
    let mut objects: Option<PathBuf> = None;
    if let Some(directory) = objects_variable {
        objects = Some(start.join(directory));
    }
    let content_command: bool = matches!(parsed.command, "ls-files" | "diff-index" | "diff" | "diff-tree" | "show");
    if content_command && !content_config_is_plain(&layout, scope)? {
        return None;
    }
    // Entries injected through the environment are configuration too.
    if content_command && !env_config_avoids(variables.as_slice(), &content_key_of_any_case) {
        return None;
    }
    let context: Context = Context { layout, index_path, objects, scope: scope.clone(), variables };
    if name_length(context.layout.object_format) == 0 {
        return None;
    }
    return match parsed.command {
        "ls-files" => list_files(&context, &parsed),
        "diff-index" => diff_cached(&context, &parsed, false),
        "diff" => diff_cached(&context, &parsed, true),
        "diff-tree" if parsed.flags.contains(&"--root") => diff_tree(&context, &parsed),
        "diff-tree" => diff_tree_names(&context, &parsed),
        "show" => show_subject(&context, &parsed),
        "config" => config_list(&context, &parsed),
        "remote" => remote_url(&context, &parsed),
        "for-each-ref" => for_each_ref(&context, &parsed),
        "rev-parse" => rev_parse(&context, &parsed),
        "symbolic-ref" => symbolic_ref(&context, &parsed),
        "cat-file" => batch_objects(&context, &parsed, query.input),
        _ => None,
    };
}

#[cfg(test)]
#[path = "repo_read_query_tests.rs"]
pub(crate) mod tests;

/// Controls for the tree-to-tree listings stay out of the release executable.
#[cfg(test)]
#[path = "repo_read_pair_tests.rs"]
mod pair_tests;
