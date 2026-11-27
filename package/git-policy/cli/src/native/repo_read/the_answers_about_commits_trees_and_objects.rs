//! What:
//!  The answers that read an object by name:
//! `diff-tree --root -r -m`,
//!  the `--name-only` listing between two trees,
//! `show --no-patch --format=%s`
//!  and `cat-file --batch`.
//! Why:
//!  Each is one start of Git for bytes the object database already holds.
//!  Two of them list only the trees along the paths that differ,
//!  and the subject needs the commit's message,
//!  so these share the commit's fields and the flat list of a tree.
//!  The entry that dispatches them is `repo_read_query.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const changes = diffTree(context, parsed); // undefined: start Git
//! const batch = batchObjects(context, parsed, input);
//! ```

#![cfg(unix)]

use crate::repo_read_diff::{
    diff_sorted,
    name_lines,
    raw_lines,
};
use crate::repo_read_object::{
    ObjectKind,
    from_hex,
    to_hex,
};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_request_answer_and_context::{
    Answer,
    Context,
    Parsed,
    flags_are,
};
use crate::repo_read_tree::{
    FlatEntry,
    flatten_pair,
    flatten_tree_shared,
    parse_commit,
    subject_of,
};
use std::rc::Rc;

/// Whether `byte` is a newline.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// `diff-tree --root --no-commit-id -r -z -m --no-renames --no-abbrev <commit>` for a commit with at most one parent.
pub(crate) fn diff_tree(
    context: &Context,
    parsed: &Parsed<'_>,
) -> Option<Answer> {
    let required: [&str; 7] = [
        "--root",
        "--no-commit-id",
        "-r",
        "-z",
        "-m",
        "--no-renames",
        "--no-abbrev",
    ];
    if !flags_are(&parsed.flags, &required, &[])
        || parsed.operands.len() != 1
        || !parsed.specs.is_empty()
    {
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
            let parent_tree =
                parse_commit(parent_object.bytes.as_slice(), context.layout.object_format)?.tree;
            // Only the entries that differ are listed, so a subtree both trees share is not read: a commit that changes one
            //  file of a large checkout reads the few trees along the path, not every directory of both sides.
            let (before, after): (Vec<FlatEntry>, Vec<FlatEntry>) = flatten_pair(
                &database,
                parent_tree.as_slice(),
                commit.tree.as_slice(),
                context.layout.object_format,
            )?;
            diff_sorted(before.as_slice(), after.as_slice())?
        }
        None => {
            let new: Rc<Vec<FlatEntry>> = flatten_tree_shared(
                &database,
                commit.tree.as_slice(),
                context.layout.object_format,
            )?;
            diff_sorted(&[], new.as_slice())?
        }
    };
    return Some(Answer {
        stdout: raw_lines(changes.as_slice(), context.zero().as_str()),
        stderr: Vec::new(),
        code: 0,
    });
}

/// `diff-tree -r --name-only -z [--no-renames] <old> <new>` and `diff-tree --no-commit-id --name-only -r -z <commit>`:
/// the paths
///  that differ between two trees or commits,
/// or between a commit with exactly one parent and that parent.
pub(crate) fn diff_tree_names(
    context: &Context,
    parsed: &Parsed<'_>,
) -> Option<Answer> {
    if !parsed.specs.is_empty() {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let (old, new): (Vec<u8>, Vec<u8>) = match parsed.operands.as_slice() {
        [pair_old, pair_new] => {
            if !flags_are(
                &parsed.flags,
                &["-r", "--name-only", "-z"],
                &["--no-renames"],
            ) {
                return None;
            }
            (
                context.tree_of(&database, pair_old)?,
                context.tree_of(&database, pair_new)?,
            )
        }
        [single] => {
            if !flags_are(
                &parsed.flags,
                &["--no-commit-id", "--name-only", "-r", "-z"],
                &[],
            ) {
                return None;
            }
            let name: Vec<u8> = from_hex(single, context.layout.object_format)?;
            let object = database.read(name.as_slice())?;
            if object.kind != ObjectKind::Commit {
                return None;
            }
            let commit = parse_commit(object.bytes.as_slice(), context.layout.object_format)?;
            // A root commit and a merge commit print nothing or per-parent lists in Git, which the reader leaves to it.
            let [parent] = commit.parents.as_slice() else {
                return None;
            };
            let parent_object = database.read(parent.as_slice())?;
            if parent_object.kind != ObjectKind::Commit {
                return None;
            }
            (
                parse_commit(parent_object.bytes.as_slice(), context.layout.object_format)?.tree,
                commit.tree,
            )
        }
        _ => return None,
    };
    let (before, after): (Vec<FlatEntry>, Vec<FlatEntry>) = flatten_pair(
        &database,
        old.as_slice(),
        new.as_slice(),
        context.layout.object_format,
    )?;
    let changes = diff_sorted(before.as_slice(), after.as_slice())?;
    return Some(Answer {
        stdout: name_lines(changes.as_slice()),
        stderr: Vec::new(),
        code: 0,
    });
}

/// `show --no-patch --format=%s <commit>`.
pub(crate) fn show_subject(
    context: &Context,
    parsed: &Parsed<'_>,
) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--no-patch", "--format=%s"], &[])
        || parsed.operands.len() != 1
        || !parsed.specs.is_empty()
    {
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
    return Some(Answer {
        stdout,
        stderr: Vec::new(),
        code: 0,
    });
}

/// `cat-file --batch` fed full object names,
/// one per line:
/// for each,
/// `<name> <kind> <size>`,
/// the content and a line feed.
pub(crate) fn batch_objects(
    context: &Context,
    parsed: &Parsed<'_>,
    input: Option<&[u8]>,
) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--batch"], &[])
        || !parsed.operands.is_empty()
        || !parsed.specs.is_empty()
    {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let mut stdout: Vec<u8> = Vec::new();
    let text: &[u8] = input?.strip_suffix(b"\n")?;
    for line in text.split(is_newline) {
        let name: Vec<u8> = from_hex(line, context.layout.object_format)?;
        // A name the store lacks is Git's `missing` line, which the reader leaves to it.
        let object = database.read(name.as_slice())?;
        stdout.extend_from_slice(
            format!(
                "{} {} {}\n",
                to_hex(name.as_slice()),
                object.kind.word(),
                object.bytes.len()
            )
            .as_bytes(),
        );
        stdout.extend_from_slice(object.bytes.as_slice());
        stdout.push(b'\n');
    }
    return Some(Answer {
        stdout,
        stderr: Vec::new(),
        code: 0,
    });
}
