//! What:
//!  What both rungs that fill a worktree themselves need to know about the target of a `git worktree add` once real Git
//!  ran without a checkout:
//! the administrative directory,
//! Git's pointer,
//! the commit and tree `HEAD` names,
//! whether the
//!  tree differs from the source's,
//! the line Git's checkout would print,
//! and the step that applies the difference.
//! Why:
//!  The snapshot and clone rungs differ in how the source's files reach the destination and agree on everything else,
//! so
//!  the shared reading of Git's result and the single `git read-tree -m -u` step live in one place.
//! The first version keeps
//!  Git for the two-tree update because it owns the conversion filters (`doc/planning/cli-git-btrfs-worktree.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const target = prepareTarget(source, plan, context); // throws a reason
//! await applyDifference(source, target, workTree, runGit);
//! ```

use crate::config_snapshot::{ConfigEntry, read_config_snapshot};
use crate::landed_summary::short_name;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::from_hex;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{FlatEntry, flatten_tree, parse_commit, subject_of};
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clean::head_commit;
use crate::worktree_copy::{GitRunner, GitStatus, GitStep};
use crate::fs_beneath::handle_path;
use crate::worktree_add_binding::{Bound, path_identity};
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The target of one `git worktree add` as real Git left it.
/// Why:
///   The rungs read these facts once and use them in turn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { admin: string; pointer: Uint8Array; commit: string; differs: boolean; line: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// The administrative directory of the new worktree.
    pub admin: PathBuf,
    /// The bytes of the `.git` file Git wrote in the destination.
    pub pointer: Vec<u8>,
    /// The commit the new `HEAD` names,
    /// as hexadecimal.
    pub commit: String,
    /// Whether the tree of that commit differs from the source's.
    pub differs: bool,
    /// The line Git's checkout prints (empty for `--quiet`).
    pub line: String,
}

/// What:
///  The `.gitattributes` files of a tree as path and object name.
/// Why:
///   Unchanged paths keep the source's bytes,
/// so the conversion rules of the two trees must agree for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributeFiles(tree: FlatEntry[]): [Uint8Array, Uint8Array][];
/// ```
fn attribute_files(tree: &[FlatEntry]) -> Vec<(&[u8], &[u8])> {
    let mut files: Vec<(&[u8], &[u8])> = Vec::new();
    for entry in tree {
        if entry.path == b".gitattributes" || entry.path.ends_with(b"/.gitattributes") {
            files.push((entry.path.as_slice(), entry.id.as_slice()));
        }
    }
    return files;
}

/// What:
///  Whether the two trees carry the same `.gitattributes` files.
/// Why:
///   A different rule could change how a path both trees share is written,
/// which only Git's checkout gets right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributesAgree(database: ObjectDatabase, from: Uint8Array, to: Uint8Array, format: ObjectFormat): boolean;
/// ```
fn attributes_agree(database: &ObjectDatabase, from: &[u8], to: &[u8], format: ObjectFormat) -> Result<(), String> {
    let (Some(old), Some(new)) = (flatten_tree(database, from, format), flatten_tree(database, to, format)) else { return Err(String::from("a tree is unreadable")) };
    return if attribute_files(old.as_slice()) == attribute_files(new.as_slice()) { Ok(()) } else { Err(String::from("the .gitattributes files differ between the source and the new HEAD")) };
}

/// The configuration keys,
/// besides the `filter.` section,
/// that change how a file is written to or read from a worktree.
const CONVERSION_KEYS: &[&str] = &[
    "core.autocrlf",
    "core.eol",
    "core.safecrlf",
    "core.attributesfile",
    "core.symlinks",
    "core.filemode",
    "core.ignorecase",
    "core.precomposeunicode",
    "core.protectntfs",
    "core.protecthfs",
    "core.checkroundtripencoding",
    "core.sparsecheckout",
    "core.sparsecheckoutcone",
];

/// What:
///  The settings of a configuration listing that decide how files are converted:
/// `filter.*`,
/// `extensions.*` and the
///  conversion keys,
/// in the order the listing gives them,
/// duplicates kept.
/// Why:
///   For most keys the last value wins,
/// so the order is part of the meaning;
/// two listings with the same values in another
///  order are different configurations,
/// and a set would call them equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conversionSettings(entries: ConfigEntry[]): [string, string | undefined][];
/// ```
fn conversion_settings(entries: &[ConfigEntry]) -> Vec<(String, Option<Vec<u8>>)> {
    let mut settings: Vec<(String, Option<Vec<u8>>)> = Vec::new();
    for entry in entries {
        if entry.key.starts_with("filter.") || entry.key.starts_with("extensions.") || CONVERSION_KEYS.contains(&entry.key.as_str()) {
            settings.push((entry.key.clone(), entry.value.clone()));
        }
    }
    return settings;
}

/// What:
///  Check that the new worktree sees the same conversion settings as the source,
/// in the same order,
/// read in the new
///  worktree's own context.
/// Why:
///   A conditional include (`onbranch`,
/// `gitdir`,
/// `hasconfig`) can give the new branch other filters than the source's,
///  and then the source's bytes are not what Git's checkout would write.
/// Git evaluates the conditions for the new
///  worktree,
/// so comparing its listing with the source's covers every conditional without parsing one;
/// the sequence,
/// not
///  the set,
/// is compared because the last value of a key wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conversionAgrees(source: ConfigEntry[], target: ConfigEntry[]): void; // throws the first difference
/// ```
fn conversion_agrees(source: &[ConfigEntry], target: &[ConfigEntry]) -> Result<(), String> {
    let (old, new): (Vec<(String, Option<Vec<u8>>)>, Vec<(String, Option<Vec<u8>>)>) = (conversion_settings(source), conversion_settings(target));
    if old == new {
        return Ok(());
    }
    // The first position where the two lists differ; the loop stops there, as `find` did.
    let mut first: usize = 0;
    for index in 0..old.len().max(new.len()) {
        if old.get(index) != new.get(index) {
            first = index;
            break;
        }
    }
    let named: Option<&(String, Option<Vec<u8>>)> = match old.get(first) {
        Some(setting) => Some(setting),
        None => new.get(first),
    };
    let key: String = match named {
        Some((name, _value)) => name.clone(),
        None => String::new(),
    };
    return Err(format!("the conversion setting {key} differs between the source and the new worktree, or comes in another order"));
}

/// The attribute names a line of an attributes file applies as a filter,
/// one per `filter=<name>` token.
fn filters_named(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        for token in line.split_whitespace() {
            if let Some(name) = token.strip_prefix("filter=") {
                names.push(name.to_owned());
            }
        }
    }
    return names;
}

/// The standard definition of the Git LFS filter,
/// which converts a file by its content alone.
const LFS_FILTER: [(&str, &str); 4] = [
    ("filter.lfs.clean", "git-lfs clean -- %f"),
    ("filter.lfs.smudge", "git-lfs smudge -- %f"),
    ("filter.lfs.process", "git-lfs filter-process"),
    ("filter.lfs.required", "true"),
];

/// What:
///  Bytes as text,
/// with anything that is not UTF-8 replaced.
/// Why:
///   Configuration values are compared as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder().decode(bytes)
/// ```
fn lossy_text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).into_owned();
}

/// What:
///  Refuse a conversion that equal settings do not prove equal:
/// an attribute that selects a filter other than the standard
///  Git LFS one,
/// an attributes file named by a relative path,
/// or LFS configured beyond its standard definition.
/// Why:
///   A filter command may behave by branch,
/// directory or environment,
/// `core.attributesFile` may mean another file in
///  another worktree,
/// and LFS extensions change the pointer a clean produces;
/// none of that is visible in equal strings.
///  The standard LFS filter is a function of the file's content and the same program,
/// so it is allowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function externalConversionRefusal(settings: ConfigEntry[], attributeTexts: string[]): string | undefined;
/// ```
/// What:
///  The last value the configuration gives `key`,
/// as text,
/// if any entry has the key.
/// Why:
///   Git reads later entries over earlier ones,
/// so the search runs from the end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.findLast((entry) => entry.key === key)?.value?.toString()
/// ```
fn last_value_of(entries: &[ConfigEntry], key: &str) -> Option<String> {
    for entry in entries.iter().rev() {
        if entry.key == key {
            return entry.value.as_deref().map(lossy_text);
        }
    }
    return None;
}

/// What:
///  Whether the configuration holds the four settings of the standard Git LFS filter,
/// each with its standard value.
/// Why:
///   Only the standard definition is proven to give the same result in both worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// LFS_FILTER.every(([key, expected]) => lastValueOf(entries, key) === expected)
/// ```
fn lfs_filter_is_standard(entries: &[ConfigEntry]) -> bool {
    for (key, expected) in LFS_FILTER {
        if last_value_of(entries, key).as_deref() != Some(expected) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether the configuration has any `lfs.extension.*` entry.
/// Why:
///   Extensions change what the LFS filter does,
/// so they are not proven equal in both worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.some((entry) => entry.key.startsWith('lfs.extension.'))
/// ```
fn has_lfs_extension(entries: &[ConfigEntry]) -> bool {
    for entry in entries {
        if entry.key.starts_with("lfs.extension.") {
            return true;
        }
    }
    return false;
}

fn external_conversion_refusal(entries: &[ConfigEntry], attribute_texts: &[String]) -> Option<String> {
    // The first entry that names a relative attributes file; the loop stops there, as `find` did.
    for entry in entries {
        let absolute: bool = match entry.value.as_deref() {
            Some(value) => value.starts_with(b"/"),
            None => false,
        };
        if entry.key == "core.attributesfile" && !absolute {
            return Some(format!("core.attributesFile is {:?}, which is not an absolute path", entry.value.as_deref().map(String::from_utf8_lossy)));
        }
    }
    let mut used: Vec<String> = Vec::new();
    for text in attribute_texts {
        used.extend(filters_named(text.as_str()));
    }
    for name in &used {
        if name != "lfs" {
            return Some(format!("an attribute applies the filter {name}, whose result is not proven equal in both worktrees"));
        }
        if !lfs_filter_is_standard(entries) {
            return Some(String::from("the lfs filter is not the standard Git LFS definition"));
        }
        if has_lfs_extension(entries) {
            return Some(String::from("Git LFS extensions are configured"));
        }
    }
    return None;
}

/// The text of every attributes file that applies to both worktrees besides the tracked ones:
/// the repository's
///  `info/attributes`,
/// the file `core.attributesFile` names and the default global file.
fn untracked_attribute_texts(source: &SnapshotSource, entries: &[ConfigEntry]) -> Vec<String> {
    let mut paths: Vec<PathBuf> = vec![source.common_dir.join("info/attributes")];
    // The last entry that names an attributes file decides, even when its value is empty; the loop stops there.
    for entry in entries.iter().rev() {
        if entry.key == "core.attributesfile" {
            if let Some(value) = entry.value.as_deref() {
                paths.push(PathBuf::from(String::from_utf8_lossy(value).into_owned()));
            }
            break;
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".config/git/attributes"));
    }
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(config).join("git/attributes"));
    }
    let mut texts: Vec<String> = Vec::new();
    for path in &paths {
        if let Ok(bytes) = std::fs::read(path.as_path()) {
            texts.push(String::from_utf8_lossy(bytes.as_slice()).into_owned());
        }
    }
    return texts;
}

/// What:
///  Run one Git command through the lifecycle's runner and require status zero.
/// Why:
///   Every Git run of the ladder inherits the caller's streams and reports only a status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runStep(runGit, args, env);
/// ```
pub(crate) fn run_step(run_git: &mut dyn GitRunner, arguments: &[OsString], environment: &[(OsString, OsString)]) -> Result<GitStatus, String> {
    // A step of the ladder runs after the add's binding is recorded, so its Git needs no name in a record.
    return match run_git.run(&GitStep::Other { arguments, environment }, &mut |_pid: u32| {}) {
        Ok(status) => Ok(status),
        Err(error) => Err(error.0),
    };
}

/// What:
///  The environment Git gives the commands of its own checkout:
/// its git directory and work tree.
/// Why:
///   The ladder's runs name the same two so that they see the new worktree whatever the caller's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const env = { GIT_DIR: gitDir, GIT_WORK_TREE: workTree };
/// ```
pub(crate) fn checkout_environment(git_dir: &Path, work_tree: &Path) -> Vec<(OsString, OsString)> {
    return vec![(OsString::from("GIT_DIR"), git_dir.as_os_str().to_os_string()), (OsString::from("GIT_WORK_TREE"), work_tree.as_os_str().to_os_string())];
}

/// What:
///  Read the result of the `--no-checkout` run:
/// the pointer,
/// the administrative directory,
/// the new `HEAD` and the line.
///  The error is the reason the rungs cannot serve the command.
/// Why:
///   A destination that holds more than Git's pointer,
/// a `HEAD` the readers cannot resolve,
/// a commit whose subject needs
///  Git to print,
/// or `.gitattributes` files that differ between the two trees leave the checkout to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTarget(source: SnapshotSource, plan: AddPlan, context: GitContext): Target; // throws a reason
/// ```
pub fn prepare_target(source: &SnapshotSource, plan: &AddPlan, context: &GitContext, bound: &Bound) -> Result<Target, String> {
    let destination: &Path = plan.destination.as_path();
    if bound.binding.destination != destination {
        return Err(String::from("the add is bound to another destination"));
    }
    let pointer: Vec<u8> = bound.binding.pointer.clone();
    let admin: PathBuf = bound.binding.admin.clone();
    // The destination as bound must still be the one directory Git made, holding nothing but Git's pointer.
    if path_identity(destination) != Some(bound.binding.destination_id) {
        return Err(String::from("the destination is no longer the directory Git made"));
    }
    let held_listing = match std::fs::read_dir(handle_path(&bound.destination)) {
        Ok(listing) => listing,
        Err(error) => return Err(format!("the destination is unreadable ({error})")),
    };
    let held_entries: usize = held_listing.count();
    if held_entries != 1 {
        return Err(String::from("the destination holds more than Git's .git file"));
    }
    let Some(target_config) = read_config_snapshot(context, destination) else {
        return Err(String::from("the new worktree's configuration could not be read"));
    };
    conversion_agrees(source.config.entries(), target_config.entries())?;
    let format: ObjectFormat = if source.clean.hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(commit) = head_commit(admin.as_path(), source.common_dir.as_path(), format) else {
        return Err(String::from("the new HEAD is not a plain ref to a commit"));
    };
    let Some(database) = ObjectDatabase::open(source.common_dir.as_path(), None, format) else {
        return Err(String::from("the object database is not plain"));
    };
    let Some(name) = from_hex(commit.as_bytes(), format) else {
        return Err(String::from("the new HEAD is not an object name"));
    };
    let Some(raw) = database.read(name.as_slice()) else {
        return Err(String::from("the new HEAD commit is unreadable"));
    };
    let Some(fields) = parse_commit(raw.bytes.as_slice(), format) else {
        return Err(String::from("the new HEAD commit has a layout the reader does not know"));
    };
    let differs: bool = fields.tree != source.clean.head_tree;
    if differs {
        attributes_agree(&database, source.clean.head_tree.as_slice(), fields.tree.as_slice(), format)?;
    }
    // The tracked attributes files are the same in both trees by now, so the source's stand for both.
    let mut attribute_texts: Vec<String> = untracked_attribute_texts(source, target_config.entries());
    for (path, id) in &source.clean.attribute_blobs {
        let Some(blob) = database.read(id.as_slice()) else {
            return Err(format!("{} is unreadable", String::from_utf8_lossy(path)));
        };
        attribute_texts.push(String::from_utf8_lossy(blob.bytes.as_slice()).into_owned());
    }
    if let Some(reason) = external_conversion_refusal(target_config.entries(), attribute_texts.as_slice()) {
        return Err(reason);
    }
    let line: String = if plan.quiet {
        String::new()
    } else {
        let Some(subject) = subject_of(fields.message.as_slice()) else {
            return Err(String::from("the subject of the new HEAD needs Git to print"));
        };
        format!("HEAD is now at {} {}\n", short_name(context, destination, source.common_dir.join("objects").as_path(), Some(&source.config), commit.as_str()), String::from_utf8_lossy(subject.as_slice()))
    };
    return Ok(Target { admin, pointer, commit, differs, line });
}

/// What:
///  Apply the difference between the source's tree and the new `HEAD` to `work_tree`,
/// whose index is the source's:
///  nothing when the trees are equal,
/// else one `git read-tree -m -u`.
/// Why:
///   Git owns the conversion filters,
/// so the paths that differ are written by Git and the unchanged ones keep the bytes
///  the source had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await applyDifference(source, target, workTree, runGit); // throws a reason
/// ```
pub fn apply_difference(source: &SnapshotSource, target: &Target, work_tree: &Path, run_git: &mut dyn GitRunner) -> Result<(), String> {
    if !target.differs {
        return Ok(());
    }
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        work_tree.as_os_str().to_os_string(),
        OsString::from("read-tree"),
        OsString::from("-m"),
        OsString::from("-u"),
        OsString::from(source.clean.head_commit.as_str()),
        OsString::from(target.commit.as_str()),
    ];
    let status: GitStatus = run_step(run_git, arguments.as_slice(), checkout_environment(target.admin.as_path(), work_tree).as_slice())?;
    return if status.succeeded() { Ok(()) } else { Err(format!("`git read-tree -m -u` ended with {:?}", status.code)) };
}

/// Ladder target controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_target_tests.rs"]
mod tests;
