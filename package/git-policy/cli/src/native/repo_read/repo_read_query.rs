//! What:
//!  Answers, without a Git process, to the exact read-only Git command lines the wrapper itself issues:
//!  `ls-files --stage`, `ls-files --unmerged`, `diff-index --cached`, `diff --cached --name-only`, `diff-tree --root -r -m`
//!  and `show --no-patch --format=%s`.
//! Why:
//!  Each is a start of Git for a fact two in-process readers (the index, the object database) can produce byte for byte.
//!  The reader takes the same argument list the caller would give Git and returns Git's standard output and exit code,
//!  or `None` for any command line, repository or configuration it does not fully understand, so a caller tries it first
//!  and starts Git on `None`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = answerInProcess({ args, directory, variables }); // undefined: start Git
//! ```

#![cfg(unix)]

use crate::repo_read_config_scan::{ScopeEnvironment, content_key, entries_avoid_keys, file_avoids_keys, outer_scopes_avoid_keys};
use crate::repo_read_config_text::{CONFIG_TEXT_LIMIT, parse_config_text};
use crate::repo_read_diff::{PathFilter, diff_sorted, name_lines, parse_diff_filter, raw_lines, select};
use crate::repo_read_index::{INTENT_TO_ADD, IndexEntry, ParsedIndex, parse_index_with_tree};
use crate::repo_read_layout::{RepoLayout, env_config_avoids, read_layout_with};
use crate::repo_read_object::{ObjectKind, from_hex, name_length, to_hex};

use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_refs::{config_list, for_each_ref, remote_url, rev_parse, symbolic_ref};
use crate::index_cache_tree::CacheTree;
use crate::repo_read_tree_cached::changes_with_cache;
use crate::repo_read_tree::{FlatEntry, flatten_pair, flatten_tree_shared, parse_commit, subject_of};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The command line and surroundings of one query.
pub struct Query<'a> {
    /// The arguments given to Git, global options first.
    pub arguments: &'a [OsString],
    /// The directory Git would run in; `None` is the process directory.
    pub directory: Option<&'a Path>,
    /// The variables added to Git's environment for this run.
    pub variables: &'a [(OsString, OsString)],
    /// The bytes sent to Git's standard input, for the commands that read object names from it.
    pub input: Option<&'a [u8]>,
}

/// What Git would have printed and returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Answer {
    /// Standard output.
    pub stdout: Vec<u8>,
    /// Standard error.
    pub stderr: Vec<u8>,
    /// The exit code.
    pub code: i32,
}

/// The parts of a command line after its global options.
pub(crate) struct Parsed<'a> {
    /// The subcommand word.
    pub(crate) command: &'a str,
    /// Options before `--`.
    pub(crate) flags: Vec<&'a str>,
    /// Non-option arguments before `--`.
    pub(crate) operands: Vec<&'a [u8]>,
    /// Arguments after `--`.
    pub(crate) specs: Vec<&'a [u8]>,
}

/// Split the arguments: leading `-C <dir>` pairs move the start directory, `--no-literal-pathspecs` is Git's default.
fn parse<'a>(arguments: &'a [OsString], start: &mut PathBuf) -> Option<Parsed<'a>> {
    let mut index: usize = 0;
    while index < arguments.len() {
        let word: &OsStr = arguments[index].as_os_str();
        if word == "-C" {
            let target: &OsString = arguments.get(index + 1)?;
            if !target.is_empty() {
                *start = start.join(target);
            }
            // Git changes directory at each `-C` in turn and fails at the first one that is not a directory.
            if !std::fs::metadata(&*start).is_ok_and(|found| { return found.is_dir(); }) {
                return None;
            }
            index += 2;
        } else if word == "--no-literal-pathspecs" {
            index += 1;
        } else {
            break;
        }
    }
    let command: &str = arguments.get(index)?.to_str()?;
    let mut parsed: Parsed<'a> = Parsed { command, flags: Vec::new(), operands: Vec::new(), specs: Vec::new() };
    let mut after_separator: bool = false;
    for argument in &arguments[index + 1..] {
        let bytes: &[u8] = argument.as_encoded_bytes();
        if after_separator {
            parsed.specs.push(bytes);
        } else if bytes == b"--" {
            after_separator = true;
        } else if bytes.starts_with(b"-") {
            parsed.flags.push(argument.to_str()?);
        } else {
            parsed.operands.push(bytes);
        }
    }
    return Some(parsed);
}

/// Whether `flags` holds exactly the options of `allowed` (each at most once) and every one of `required`.
pub(crate) fn flags_are(flags: &[&str], required: &[&str], allowed: &[&str]) -> bool {
    return required.iter().all(|flag| { return flags.contains(flag); })
        && flags.iter().all(|flag| { return allowed.contains(flag) || required.contains(flag); })
        && flags.iter().enumerate().all(|(position, flag)| { return !flags[..position].contains(flag); });
}

/// The environment Git would see, without the two variables this reader handles itself.
fn environment(
    base: Vec<(OsString, OsString)>,
    variables: &[(OsString, OsString)],
) -> (Vec<(OsString, OsString)>, Option<OsString>, Option<OsString>) {
    let mut all: Vec<(OsString, OsString)> = base
        .into_iter()
        .filter(|(name, _)| { return !variables.iter().any(|(added, _)| { return added == name; }); })
        .collect();
    all.extend(variables.iter().cloned());
    let find = |wanted: &str| -> Option<OsString> {
        // The last assignment wins, as it does for the child's environment.
        return all.iter().rev().find(|(name, _)| { return name == wanted; }).map(|(_, value)| { return value.clone(); });
    };
    let index: Option<OsString> = find("GIT_INDEX_FILE");
    let objects: Option<OsString> = find("GIT_OBJECT_DIRECTORY");
    all.retain(|(name, _)| { return name != "GIT_INDEX_FILE" && name != "GIT_OBJECT_DIRECTORY"; });
    return (all, index, objects);
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

/// Everything one query needs, opened lazily.
pub(crate) struct Context {
    /// The repository.
    pub(crate) layout: RepoLayout,
    /// The index file Git would read.
    index_path: PathBuf,
    /// The object directory override, if any.
    objects: Option<PathBuf>,
    /// Where the system and global configuration files are.
    pub(crate) scope: ScopeEnvironment,
    /// The environment Git would see, without the index and object directory variables.
    pub(crate) variables: Vec<(OsString, OsString)>,
}

impl Context {
    /// The object database.
    pub(crate) fn database(&self) -> Option<ObjectDatabase> {
        return ObjectDatabase::open(self.layout.common_dir.as_path(), self.objects.as_deref(), self.layout.object_format);
    }

    /// The index entries.
    pub(crate) fn index(&self) -> Option<Vec<IndexEntry>> {
        return self.index_with_tree().map(|parsed| { return parsed.entries; });
    }

    /// The index entries and the cache-tree extension.
    pub(crate) fn index_with_tree(&self) -> Option<ParsedIndex> {
        let bytes: Vec<u8> = crate::private_storage::read_regular_file(self.index_path.as_path(), 1 << 30).ok()?;
        return parse_index_with_tree(bytes.as_slice(), self.layout.object_format);
    }

    /// The index as flat entries and its cache-tree, refusing unmerged or intent-to-add entries.
    fn resolved_index_with_tree(&self) -> Option<(Vec<FlatEntry>, Option<CacheTree>)> {
        let parsed: ParsedIndex = self.index_with_tree()?;
        if parsed.entries.iter().any(|entry| { return entry.stage != 0 || entry.extended & INTENT_TO_ADD != 0; }) {
            return None;
        }
        let flat: Vec<FlatEntry> = parsed.entries.into_iter().map(|entry| { return FlatEntry { path: entry.path, mode: entry.mode, id: entry.id }; }).collect();
        return Some((flat, parsed.tree));
    }

    /// The root tree of a commit or tree named by `hex`.
    fn tree_of(&self, database: &ObjectDatabase, hex: &[u8]) -> Option<Vec<u8>> {
        let name: Vec<u8> = from_hex(hex, self.layout.object_format)?;
        let object = database.read(name.as_slice())?;
        return match object.kind {
            ObjectKind::Tree => Some(name),
            ObjectKind::Commit => Some(parse_commit(object.bytes.as_slice(), self.layout.object_format)?.tree),
            ObjectKind::Blob | ObjectKind::Tag => None,
        };
    }

    /// The zero object name spelled out for raw lines.
    fn zero(&self) -> String {
        return "0".repeat(self.layout.object_format.hex_length());
    }
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
    let cached = cache.as_ref().and_then(|nodes| { return changes_with_cache(&database, tree.as_slice(), context.layout.object_format, nodes, index.as_slice()); });
    let changes = match cached {
        Some(found) => found,
        None => {
            let old: Rc<Vec<FlatEntry>> = flatten_tree_shared(&database, tree.as_slice(), context.layout.object_format)?;
            diff_sorted(old.as_slice(), index.as_slice())?
        }
    };
    // The porcelain pairs an added and a deleted path as a rename; plain lists cannot say which pairs it would choose.
    if porcelain && changes.iter().any(|change| { return change.status == b'A'; }) && changes.iter().any(|change| { return change.status == b'D'; }) {
        return None;
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

/// `diff-tree -r --name-only -z [--no-renames] <old> <new>` and `diff-tree --no-commit-id --name-only -r -z <commit>`: the paths
///  that differ between two trees or commits, or between a commit with exactly one parent and that parent.
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

/// `cat-file --batch` fed full object names, one per line: for each, `<name> <kind> <size>`, the content and a line feed.
fn batch_objects(context: &Context, parsed: &Parsed<'_>, input: Option<&[u8]>) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--batch"], &[]) || !parsed.operands.is_empty() || !parsed.specs.is_empty() {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let mut stdout: Vec<u8> = Vec::new();
    let text: &[u8] = input?.strip_suffix(b"\n")?;
    for line in text.split(|byte| { return *byte == b'\n'; }) {
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
///  The answer to `query`, or `None` when Git must be started.
/// Why:
///   This is the one entry point; every reason to decline (an unknown command line, a repository or configuration
///  the readers do not fully understand, a missing object) is the same `None`.
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

/// The same with the process environment and the configuration scope facts given, so a control does not depend on its host.
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
    let objects: Option<PathBuf> = objects_variable.map(|directory| { return start.join(directory); });
    let content_command: bool = matches!(parsed.command, "ls-files" | "diff-index" | "diff" | "diff-tree" | "show");
    if content_command && !content_config_is_plain(&layout, scope)? {
        return None;
    }
    // Entries injected through the environment are configuration too.
    if content_command && !env_config_avoids(variables.as_slice(), &|key: &str| { return content_key(key.to_ascii_lowercase().as_str()); }) {
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
