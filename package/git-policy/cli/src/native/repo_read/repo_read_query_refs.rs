//! What:
//!  In-process answers for the reference and configuration command lines the wrapper issues:
//!  `config --list -z --show-scope`,
//! `remote get-url <name>`,
//! `rev-parse --verify --quiet <ref>^{commit}`,
//!  `for-each-ref --format=%(refname)%00%(objectname)%00%(symref)` and
//!  `rev-parse --abbrev-ref --symbolic-full-name @{upstream}`.
//! Why:
//!  Each reads only the configuration files,
//! `HEAD`,
//! the loose and packed refs and at most a few objects;
//!  every case where Git's output depends on more (a linked worktree,
//! a conditional include,
//! a rewritten URL,
//! an
//!  ambiguous short name,
//! a broken ref) is `None`,
//! which starts Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = referenceAnswer(context, parsed); // undefined: start Git
//! ```

#![cfg(unix)]

use crate::repo_read_config_list::{ListedEntry, list_config, listing_bytes};
use crate::repo_read_object::{ObjectKind, from_hex};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_request_answer_and_context::{Answer, Context, Parsed, flags_are};
use crate::repo_read_refs::{Lookup, RefValue, Refs, is_listable_ref_name};
use std::ffi::OsString;

/// The `--format` text of the one listing the wrapper asks for.
const LISTING_FORMAT: &str = "--format=%(refname)%00%(objectname)%00%(symref)";

/// How many refs a `for-each-ref` listing answers in process;
/// a repository above this count starts Git.
/// Why:
///   Every ref the listing prints is checked against the object database,
///  because Git drops a ref whose object is missing (`warning: ignoring broken ref`)
///  and the answer must drop it too,
///  and that check is one pack-index lookup per ref.
///   Measured on 2026-11-09 (release profile,
///  arms alternating in one binary,
///  this repository: 1442 refs,
///  four packs of 514k objects,
///  load average 61 to 84):
///  the answer took 7.1 ms at the median against 3.8 ms for `git for-each-ref`,
///  and 4.3 ms of it was that check,
///  after the check was changed to ask the packs before the loose paths.
///  The layout and the ref walk alone take 1.8 ms,
///  so the two meet near 490 refs by the measured per-ref cost (3.5 us) and the answer loses above them.
///   This limit sits well below that meeting point,
///  because a larger object database makes each check dearer than it is here,
///  and the Git start it is compared against is a constant.
const REF_LISTING_LIMIT: usize = 256;

/// How many tag levels are peeled.
const PEEL_LIMIT: usize = 8;

/// The refs of a context whose worktree owns the shared directory;
/// `None` for a linked worktree.
fn shared_refs(context: &Context) -> Option<Refs> {
    if context.layout.git_dir != context.layout.common_dir {
        return None;
    }
    return Some(Refs::new(context.layout.common_dir.as_path(), context.layout.object_format.hex_length()));
}

/// The listing,
/// or `None`.
fn listing(context: &Context) -> Option<Vec<ListedEntry>> {
    return list_config(context.layout.common_dir.as_path(), context.layout.git_dir.as_path(), &context.scope, context.variables.as_slice());
}

/// Every value of `key` among the listed entries,
/// in order.
fn values_of<'a>(listed: &'a [ListedEntry], key: &str) -> Vec<&'a Option<Vec<u8>>> {
    let mut values: Vec<&'a Option<Vec<u8>>> = Vec::new();
    for item in listed {
        if item.entry.key == key {
            values.push(&item.entry.value);
        }
    }
    return values;
}

/// The last non-empty text value of the variable `name` among `variables`,
/// or nothing.
fn text_variable(variables: &[(OsString, OsString)], name: &str) -> Option<String> {
    for (candidate, value) in variables.iter().rev() {
        if candidate == name {
            return match value.to_str() {
                Some(text) if !text.is_empty() => Some(text.to_string()),
                _ => None,
            };
        }
    }
    return None;
}

/// Whether a locale name is one whose messages are English:
/// `C`,
/// `POSIX` or starting with `en`.
fn is_english(value: &str) -> bool {
    return value == "C" || value == "POSIX" || value.starts_with("en");
}

/// Whether `byte` may appear in a remote name this reader takes:
/// a letter,
/// a digit,
/// `-`,
/// `_` or `.`.
fn is_remote_name_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.');
}

/// Whether `byte` is a newline.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether the symbolic ref target `name` is a plain,
/// non-symbolic ref or does not exist yet;
///  Git follows a chain of symbolic refs,
/// so a target that is itself symbolic is left to it.
fn is_terminal(refs: &Refs, name: &str) -> bool {
    return matches!(refs.lookup(name), Lookup::Missing | Lookup::Found(RefValue::Direct(_)));
}

/// Whether Git's messages are in English under these variables,
/// which the synthesized failures assume.
fn messages_are_english(variables: &[(OsString, OsString)]) -> bool {
    let mut locale: Option<String> = text_variable(variables, "LC_ALL");
    if locale.is_none() {
        locale = text_variable(variables, "LC_MESSAGES");
    }
    if locale.is_none() {
        locale = text_variable(variables, "LANG");
    }
    if let Some(name) = locale.as_deref()
        && !is_english(name)
    {
        return false;
    }
    if let Some(name) = text_variable(variables, "LANGUAGE")
        && !is_english(name.as_str())
    {
        return false;
    }
    return true;
}

/// `config --list -z --show-scope`.
pub(crate) fn config_list(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--list", "-z", "--show-scope"], &[]) || !parsed.operands.is_empty() || !parsed.specs.is_empty() {
        return None;
    }
    let listed: Vec<ListedEntry> = listing(context)?;
    return Some(Answer { stdout: listing_bytes(listed.as_slice()), stderr: Vec::new(), code: 0 });
}

/// `remote get-url <name>` for a remote with one plain URL and no rewriting rule anywhere.
pub(crate) fn remote_url(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !parsed.flags.is_empty() || !parsed.specs.is_empty() || parsed.operands.len() != 2 || parsed.operands[0] != b"get-url" {
        return None;
    }
    let name: &str = std::str::from_utf8(parsed.operands[1]).ok()?;
    if name.is_empty() || !name.bytes().all(is_remote_name_byte) {
        return None;
    }
    let listed: Vec<ListedEntry> = listing(context)?;
    for item in &listed {
        if item.entry.key.starts_with("url.") && (item.entry.key.ends_with(".insteadof") || item.entry.key.ends_with(".pushinsteadof")) {
            return None;
        }
    }
    let key: String = format!("remote.{name}.url");
    let mut urls: Vec<&ListedEntry> = Vec::new();
    for item in &listed {
        if item.entry.key == key {
            urls.push(item);
        }
    }
    if urls.is_empty() {
        // A remote that is configured nowhere, not even by a legacy `remotes/` or `branches/` file, is Git's own refusal.
        let prefix: String = format!("remote.{name}.");
        for directory in ["remotes", "branches"] {
            if std::fs::symlink_metadata(context.layout.common_dir.join(directory).join(name)).is_ok() {
                return None;
            }
        }
        for item in &listed {
            if item.entry.key.starts_with(prefix.as_str()) {
                return None;
            }
        }
        if !messages_are_english(context.variables.as_slice()) {
            return None;
        }
        let message: String = format!("error: No such remote '{name}'\n");
        return Some(Answer { stdout: Vec::new(), stderr: message.into_bytes(), code: 2 });
    }
    let [only] = urls.as_slice() else {
        return None;
    };
    let value: Vec<u8> = only.entry.value.clone()?;
    if value.is_empty() {
        return None;
    }
    let mut stdout: Vec<u8> = value;
    stdout.push(b'\n');
    return Some(Answer { stdout, stderr: Vec::new(), code: 0 });
}

/// The commit a ref name peels to,
/// through at most a few tags.
fn peel_to_commit(database: &ObjectDatabase, context: &Context, start: &str) -> Option<String> {
    let mut current: Vec<u8> = from_hex(start.as_bytes(), context.layout.object_format)?;
    for _ in 0..PEEL_LIMIT {
        let object = database.read(current.as_slice())?;
        match object.kind {
            ObjectKind::Commit => return Some(crate::repo_read_object::to_hex(current.as_slice())),
            ObjectKind::Tag => {
                let first: &[u8] = object.bytes.split(is_newline).next()?;
                current = from_hex(first.strip_prefix(b"object ")?, context.layout.object_format)?;
            }
            ObjectKind::Tree | ObjectKind::Blob => return None,
        }
    }
    return None;
}

/// `for-each-ref` with the one format the wrapper uses.
pub(crate) fn for_each_ref(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if parsed.flags != [LISTING_FORMAT] || !parsed.operands.is_empty() || !parsed.specs.is_empty() {
        return None;
    }
    let refs: Refs = shared_refs(context)?;
    // Past this count the answer takes longer than the Git start it saves, so Git answers.
    //  The packed file is counted before anything else,
    //  because opening the database and walking `refs/` are the costs this check exists to avoid.
    if refs.packed_ref_count()? > REF_LISTING_LIMIT {
        return None;
    }
    let database: ObjectDatabase = context.database()?;
    let listing: Vec<(String, RefValue)> = refs.list()?;
    // Loose refs are counted only now, because counting them means walking `refs/`;
    //  a repository whose count is this high because of loose refs is declined before one object is checked.
    if listing.len() > REF_LISTING_LIMIT {
        return None;
    }
    let mut stdout: Vec<u8> = Vec::new();
    for (name, value) in listing {
        let (object, symref): (String, &str) = match &value {
            RefValue::Direct(object) => (object.clone(), ""),
            RefValue::Symbolic(target) if is_terminal(&refs, target.as_str()) => (refs.resolve(name.as_str())?, target.as_str()),
            RefValue::Symbolic(_) => return None,
        };
        // Git skips, with a warning, a ref whose object is missing; that case is left to it.
        if !database.contains(from_hex(object.as_bytes(), context.layout.object_format)?.as_slice()) {
            return None;
        }
        stdout.extend_from_slice(format!("{name}\0{object}\0{symref}\n").as_bytes());
    }
    return Some(Answer { stdout, stderr: Vec::new(), code: 0 });
}

/// Whether some ref spelled `prefix` + `short` exists,
/// which would make the short name ambiguous.
fn shadowed(refs: &Refs, prefix: &str, short: &str) -> Option<bool> {
    let name: String = format!("{prefix}{short}");
    if !is_listable_ref_name(name.as_str()) {
        return Some(false);
    }
    return match refs.lookup(name.as_str()) {
        Lookup::Missing => Some(false),
        Lookup::Found(_) => Some(true),
        Lookup::Unsure => None,
    };
}

/// The tracking ref a fetch refspec maps `merge` to.
pub(crate) fn tracking_ref(listed: &[ListedEntry], remote: &str, merge: &str) -> Option<String> {
    let key: String = format!("remote.{remote}.fetch");
    for item in listed {
        if item.entry.key != key {
            continue;
        }
        let spec: &str = std::str::from_utf8(item.entry.value.as_deref()?).ok()?;
        let (source, destination) = spec.strip_prefix('+').unwrap_or(spec).split_once(':')?;
        match (source.split_once('*'), destination.split_once('*')) {
            (Some((head, tail)), Some((to_head, to_tail))) => {
                if let Some(rest) = merge.strip_prefix(head)
                    && let Some(middle) = rest.strip_suffix(tail)
                {
                    return Some(format!("{to_head}{middle}{to_tail}"));
                }
            }
            (None, None) if source == merge => return Some(destination.to_string()),
            _ => {}
        }
    }
    return None;
}

/// `rev-parse --abbrev-ref --symbolic-full-name @{upstream}` for a branch with a plain upstream,
/// or none at all.
fn upstream(context: &Context) -> Option<Answer> {
    let refs: Refs = shared_refs(context)?;
    let head = match refs.lookup_head(context.layout.git_dir.as_path()) {
        Some(RefValue::Symbolic(target)) => target,
        _ => return None,
    };
    let branch: &str = head.strip_prefix("refs/heads/")?;
    // A `HEAD` that names a branch which is itself symbolic follows the chain in Git.
    if !is_terminal(&refs, head.as_str()) {
        return None;
    }
    let listed: Vec<ListedEntry> = listing(context)?;
    let remotes = values_of(listed.as_slice(), &format!("branch.{branch}.remote"));
    let merges = values_of(listed.as_slice(), &format!("branch.{branch}.merge"));
    // More than one entry is a multi-valued setting whose choice Git makes; the reader declines it.
    let (remote_value, merge_value) = (remotes.as_slice(), merges.as_slice());
    let ([Some(remote)], [Some(merge)]) = (remote_value, merge_value) else {
        if remote_value.is_empty() && merge_value.is_empty() && messages_are_english(context.variables.as_slice()) {
            let message: String = format!("fatal: no upstream configured for branch '{branch}'\n");
            return Some(Answer { stdout: Vec::new(), stderr: message.into_bytes(), code: 128 });
        }
        return None;
    };
    let remote_name: &str = std::str::from_utf8(remote.as_slice()).ok()?;
    let merge_name: &str = std::str::from_utf8(merge.as_slice()).ok()?;
    if remote_name == "." || !is_listable_ref_name(merge_name) {
        return None;
    }
    let tracking: String = tracking_ref(listed.as_slice(), remote_name, merge_name)?;
    let short: &str = tracking.strip_prefix("refs/remotes/")?;
    // The tracking ref must exist and resolve, not be a symbolic ref that points nowhere.
    refs.resolve(tracking.as_str())?;
    // The first of Git's lookup rules is the bare name itself, a file in the Git directory.
    if std::fs::symlink_metadata(context.layout.common_dir.join(short)).is_ok() {
        return None;
    }
    for prefix in ["refs/", "refs/tags/", "refs/heads/"] {
        if shadowed(&refs, prefix, short)? {
            return None;
        }
    }
    if shadowed(&refs, "refs/remotes/", &format!("{short}/HEAD"))? {
        return None;
    }
    return Some(Answer { stdout: format!("{short}\n").into_bytes(), stderr: Vec::new(), code: 0 });
}

/// `symbolic-ref --quiet HEAD`:
/// the branch `HEAD` names,
/// or a silent failure for a detached `HEAD`.
pub(crate) fn symbolic_ref(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--quiet"], &[]) || parsed.operands != [b"HEAD".as_slice()] || !parsed.specs.is_empty() {
        return None;
    }
    let refs: Refs = shared_refs(context)?;
    return match refs.lookup_head(context.layout.git_dir.as_path())? {
        // Git follows a chain of symbolic refs and prints the last; a target that is symbolic is left to it.
        RefValue::Symbolic(target) if !is_terminal(&refs, target.as_str()) => None,
        RefValue::Symbolic(target) => Some(Answer { stdout: format!("{target}\n").into_bytes(), stderr: Vec::new(), code: 0 }),
        RefValue::Direct(_) => Some(Answer { stdout: Vec::new(), stderr: Vec::new(), code: 1 }),
    };
}

/// `rev-parse --verify --quiet <ref>^{commit}` and the upstream query.
pub(crate) fn rev_parse(context: &Context, parsed: &Parsed<'_>) -> Option<Answer> {
    if !parsed.specs.is_empty() || parsed.operands.len() != 1 {
        return None;
    }
    if parsed.operands[0] == b"@{upstream}" {
        if !flags_are(&parsed.flags, &["--abbrev-ref", "--symbolic-full-name"], &[]) {
            return None;
        }
        return upstream(context);
    }
    if !flags_are(&parsed.flags, &["--verify", "--quiet"], &[]) {
        return None;
    }
    let name: &str = std::str::from_utf8(parsed.operands[0]).ok()?.strip_suffix("^{commit}")?;
    let refs: Refs = shared_refs(context)?;
    let object: String = refs.resolve(name)?;
    let database: ObjectDatabase = context.database()?;
    let commit: String = peel_to_commit(&database, context, object.as_str())?;
    return Some(Answer { stdout: format!("{commit}\n").into_bytes(), stderr: Vec::new(), code: 0 });
}

#[cfg(test)]
#[path = "repo_read_query_refs_tests.rs"]
mod tests;
