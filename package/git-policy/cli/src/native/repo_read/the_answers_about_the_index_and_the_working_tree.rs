//! What:
//!  The answers about the index and the working tree:
//! `ls-files --stage`,
//!  `ls-files --unmerged`,
//! `diff-index --cached`
//!  and `diff --cached --name-only`.
//! Why:
//!  Each is one start of Git for a fact the index and the object database already hold,
//!  and each pairs the index with the tree a revision names or with the index alone.
//!  The cache-tree names the directories the index has not changed,
//!  so these answers read only the trees along the paths that did change.
//!  The entry that dispatches them is `repo_read_query.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staged = listFiles(context, parsed); // undefined: start Git
//! const changed = diffCached(context, parsed, porcelain);
//! ```

#![cfg(unix)]

use crate::index_cache_tree::CacheTree;
use crate::repo_read_diff::{
    PathFilter,
    diff_sorted,
    name_lines,
    parse_diff_filter,
    raw_lines,
    select,
};
use crate::repo_read_object::to_hex;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_request_answer_and_context::{
    Answer,
    Context,
    Parsed,
    flags_are,
};
use crate::repo_read_tree::{
    FlatEntry,
    flatten_tree_shared,
};
use crate::repo_read_tree_cached::changes_with_cache;
use std::rc::Rc;

/// `ls-files --stage -z` and `ls-files --unmerged -z`.
pub(crate) fn list_files(
    context: &Context,
    parsed: &Parsed<'_>,
) -> Option<Answer> {
    let unmerged: bool = parsed.flags.contains(&"--unmerged");
    let required: [&str; 2] = [if unmerged { "--unmerged" } else { "--stage" }, "-z"];
    if !flags_are(&parsed.flags, &required, &["--full-name"])
        || !parsed.operands.is_empty()
        || !context.layout.prefix.is_empty()
    {
        return None;
    }
    let filter: PathFilter = PathFilter::parse(parsed.specs.as_slice(), true)?;
    let mut stdout: Vec<u8> = Vec::new();
    for entry in context.index()? {
        if (unmerged && entry.stage == 0) || !filter.matches(entry.path.as_slice()) {
            continue;
        }
        stdout.extend_from_slice(
            format!(
                "{:06o} {} {}\t",
                entry.mode,
                to_hex(entry.id.as_slice()),
                entry.stage
            )
            .as_bytes(),
        );
        stdout.extend_from_slice(entry.path.as_slice());
        stdout.push(0);
    }
    return Some(Answer {
        stdout,
        stderr: Vec::new(),
        code: 0,
    });
}

/// `diff-index --cached` and `diff --cached --name-only`.
pub(crate) fn diff_cached(
    context: &Context,
    parsed: &Parsed<'_>,
    porcelain: bool,
) -> Option<Answer> {
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
            Some(letters) if !porcelain && statuses.is_none() => {
                statuses = Some(parse_diff_filter(letters)?)
            }
            Some(_) => return None,
            _ => others.push(flag),
        }
    }
    let allowed: &[&str] = if porcelain { &[] } else { &["--no-renames"] };
    if (porcelain && raw)
        || !flags_are(&others, &required, allowed)
        || parsed.operands.len() != 1
        || (porcelain && !parsed.specs.is_empty())
    {
        return None;
    }
    let filter: PathFilter =
        PathFilter::parse(parsed.specs.as_slice(), context.layout.prefix.is_empty())?;
    let database: ObjectDatabase = context.database()?;
    let tree: Vec<u8> = context.tree_of(&database, parsed.operands[0])?;
    let (index, cache): (Vec<FlatEntry>, Option<CacheTree>) = context.resolved_index_with_tree()?;
    // The cache-tree names the directories the index has not changed, and those are left unread; without one, or when its walk
    //  declines, the whole tree is read.
    let cached = match cache.as_ref() {
        Some(nodes) => changes_with_cache(
            &database,
            tree.as_slice(),
            context.layout.object_format,
            nodes,
            index.as_slice(),
        ),
        None => None,
    };
    let changes = match cached {
        Some(found) => found,
        None => {
            let old: Rc<Vec<FlatEntry>> =
                flatten_tree_shared(&database, tree.as_slice(), context.layout.object_format)?;
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
    let stdout: Vec<u8> = if raw {
        raw_lines(kept.as_slice(), context.zero().as_str())
    } else {
        name_lines(kept.as_slice())
    };
    return Some(Answer {
        stdout,
        stderr: Vec::new(),
        code: 0,
    });
}
