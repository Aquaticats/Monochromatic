//! What:
//!  The checks that the new worktree converts files the way the source does:
//!  its conversion settings in the order the configuration gives them,
//!  its `.gitattributes` files against the source's,
//!  and every filter an attribute selects or the configuration extends.
//! Why:
//!   A clone or a snapshot keeps an unchanged path's bytes,
//!  so a filter the new worktree would apply differently writes a path the source's bytes do not hold;
//!  Git owns the conversion filters,
//!  so the rung may fill a worktree only where Git's own rule is the same in both,
//!  and every case this cannot prove leaves the whole checkout to Git.
//!  The value that names an attributes file is that file's path as bytes,
//!  so a path that is not text is still read as the file the worktree reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! conversionAgrees(sourceConfig, targetConfig); // throws the first setting that differs
//! externalConversionRefusal(targetConfig, attributeTexts); // the reason, or nothing
//! ```

use crate::config_snapshot::ConfigEntry;
/// A pathname from the bytes a configuration value holds.
use crate::hooks_path_values::path_answer;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{
    FlatEntry,
    flatten_tree,
};
use crate::worktree_snapshot_source::SnapshotSource;
use std::path::PathBuf;

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
pub(crate) fn attributes_agree(
    database: &ObjectDatabase,
    from: &[u8],
    to: &[u8],
    format: ObjectFormat,
) -> Result<(), String> {
    let (Some(old), Some(new)) = (
        flatten_tree(database, from, format),
        flatten_tree(database, to, format),
    ) else {
        return Err(String::from("a tree is unreadable"));
    };
    return if attribute_files(old.as_slice()) == attribute_files(new.as_slice()) {
        Ok(())
    } else {
        Err(String::from(
            "the .gitattributes files differ between the source and the commit the worktree is made at, so the rules the add read are not the rules Git writes with; commit the source's .gitattributes change, and the next add clones",
        ))
    };
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
        if entry.key.starts_with("filter.")
            || entry.key.starts_with("extensions.")
            || CONVERSION_KEYS.contains(&entry.key.as_str())
        {
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
pub(crate) fn conversion_agrees(
    source: &[ConfigEntry],
    target: &[ConfigEntry],
) -> Result<(), String> {
    let (old, new): (
        Vec<(String, Option<Vec<u8>>)>,
        Vec<(String, Option<Vec<u8>>)>,
    ) = (conversion_settings(source), conversion_settings(target));
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
    return Err(format!(
        "the conversion setting {key} differs between the source and the new worktree, or comes in another order"
    ));
}

/// The attribute names a line of an attributes file applies as a filter,
/// one per `filter=<name>` token.
/// Why:
///   An attributes file is bytes,
/// and a name it selects is compared with the standard one as those same bytes,
/// so a name that is not text is not folded onto a name that is.
pub(crate) fn filters_named(text: &[u8]) -> Vec<Vec<u8>> {
    let mut names: Vec<Vec<u8>> = Vec::new();
    for line in text.split(is_a_line_break) {
        if line.trim_ascii_start().starts_with(b"#") {
            continue;
        }
        for token in line.split(is_ascii_whitespace) {
            if let Some(name) = token.strip_prefix(b"filter=") {
                names.push(name.to_vec());
            }
        }
    }
    return names;
}

/// Whether one byte ends a line of an attributes file.
fn is_a_line_break(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether one byte separates two tokens of an attributes file.
fn is_ascii_whitespace(byte: &u8) -> bool {
    return byte.is_ascii_whitespace();
}

/// The standard definition of the Git LFS filter,
/// which converts a file by its content alone.
pub(crate) const LFS_FILTER: &[(&str, &str)] = &[
    ("filter.lfs.clean", "git-lfs clean -- %f"),
    ("filter.lfs.smudge", "git-lfs smudge -- %f"),
    ("filter.lfs.process", "git-lfs filter-process"),
    ("filter.lfs.required", "true"),
];

/// What:
///  The last value the configuration gives `key`,
/// as its own bytes,
/// if any entry has the key.
/// Why:
///   Git reads later entries over earlier ones,
/// so the search runs from the end,
/// and a value is the bytes the configuration file holds,
///  which a decoding would shorten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.findLast((entry) => entry.key === key)?.value
/// ```
fn last_value_of<'entries>(
    entries: &'entries [ConfigEntry],
    key: &str,
) -> Option<&'entries [u8]> {
    for entry in entries.iter().rev() {
        if entry.key == key {
            return entry.value.as_deref();
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
        if last_value_of(entries, key) != Some(expected.as_bytes()) {
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
/// none of that is visible in equal settings.
///  The standard LFS filter is a function of the file's content and the same program,
/// so it is allowed,
/// and a name is compared with the standard one as its own bytes,
///  so a name that is not text is not read as the standard name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function externalConversionRefusal(settings: ConfigEntry[], attributeTexts: Buffer[]): string | undefined;
/// ```
pub(crate) fn external_conversion_refusal(
    entries: &[ConfigEntry],
    attribute_texts: &[Vec<u8>],
) -> Option<String> {
    // The first entry that names a relative attributes file; the loop stops there, as `find` did.
    for entry in entries {
        let absolute: bool = match entry.value.as_deref() {
            Some(value) => value.starts_with(b"/"),
            None => false,
        };
        if entry.key == "core.attributesfile" && !absolute {
            return Some(format!(
                "core.attributesFile is {:?}, which is not an absolute path",
                entry.value.as_deref().map(String::from_utf8_lossy)
            ));
        }
    }
    let mut used: Vec<Vec<u8>> = Vec::new();
    for text in attribute_texts {
        used.extend(filters_named(text.as_slice()));
    }
    for name in &used {
        if name.as_slice() != b"lfs" {
            return Some(format!(
                "an attribute applies the filter {}, whose result is not proven equal in both worktrees",
                String::from_utf8_lossy(name)
            ));
        }
        if !lfs_filter_is_standard(entries) {
            return Some(String::from(
                "the lfs filter is not the standard Git LFS definition",
            ));
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
/// What:
///  The bytes of every attributes file that applies to both worktrees besides the tracked ones:
/// the repository's
///  `info/attributes`,
/// the file `core.attributesFile` names and the default global file.
/// Why:
///   The value that names a file is that file's path as bytes,
///  so a file whose own path is not UTF-8 is still the file this worktree reads,
/// and its rules are kept as the bytes they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirAttributes([commonDir + '/info/attributes', pathFromBytes(attributesFileValue), ...]).filter(found => found)
/// ```
pub(crate) fn untracked_attribute_texts(
    source: &SnapshotSource,
    entries: &[ConfigEntry],
) -> Vec<Vec<u8>> {
    let mut paths: Vec<PathBuf> = vec![source.common_dir.join("info/attributes")];
    // The last entry that names an attributes file decides, even when its value is empty; the loop stops there.
    for entry in entries.iter().rev() {
        if entry.key == "core.attributesfile" {
            if let Some(value) = entry.value.as_deref() {
                paths.push(path_answer(value));
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
    let mut texts: Vec<Vec<u8>> = Vec::new();
    for path in &paths {
        if let Ok(bytes) = std::fs::read(path.as_path()) {
            texts.push(bytes);
        }
    }
    return texts;
}
