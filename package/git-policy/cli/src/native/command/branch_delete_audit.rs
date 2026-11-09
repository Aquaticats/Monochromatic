//! What:
//!  The facts one audit line carries about a deleted branch:
//!  its name,
//!  the commit it pointed at,
//!  and
//!       whether that commit is reachable from the `HEAD` the command ran beside.
//! Why:
//!  The owner chose "Allow,
//! add audit line" for `git branch -D` from the main worktree,
//!  so the line has to
//!       carry enough to restore the branch:
//!  a name plus the commit is exactly `git branch <name> <commit>`.
//!  The merged answer is one
//!       `git merge-base --is-ancestor` call,
//!  and it tells a reader whether the commit was already safe elsewhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // branchFacts({ context, name }) => [{ name: 'branch', value: 'gone' }, { name: 'commit', value: 'abc...' }]
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::git_metadata::{MetadataOutput, run_metadata_git};
/// The bytes of a name as the record keeps them beside their text.
use crate::path_bytes_in_records::hex_of_bytes;
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes,
///  `&OsStr` its borrowed form and `Path` a borrowed path.
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What:
///  The global options a query of this repository carries:
///  a `-C` naming the worktree when one is known,
///  else the
///       invocation's own options,
///  which already select it.
/// Why:
///   The audit reads the repository the command targets,
///  never the directory the wrapper happens to run in,
///  and
///       no second `-C` is put in front of one the invocation gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prefix = root ? ['-C', root] : context.globalPrefix;
/// ```
fn prefix_for(context: &GitContext, root: Option<&Path>) -> Vec<OsString> {
    let Some(directory) = root else {
        return context.global_prefix.clone();
    };
    return vec![
        OsString::from("-C"),
        directory.as_os_str().to_os_string(),
    ];
}

/// The reference namespace a local branch lives in.
pub const HEADS: &str = "refs/heads/";

/// The reference namespace a remote-tracking branch lives in.
pub const REMOTES: &str = "refs/remotes/";

/// What:
///  The full reference a deletion name points at in the namespace it deletes from.
/// Why:
///   `git branch -D gone` deletes `refs/heads/gone` and `git branch -r -D origin/gone` deletes
///  `refs/remotes/origin/gone`:
///  the same name can exist in both,
/// and the record has to name the one Git took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reference = (remotes ? 'refs/remotes/' : 'refs/heads/') + name;
/// ```
pub fn reference_of(name: &OsStr, remotes: bool) -> OsString {
    let mut reference: OsString = OsString::from(if remotes { REMOTES } else { HEADS });
    reference.push(name);
    return reference;
}

/// What:
///  The commit one branch points at,
/// or nothing when the repository has no such reference.
/// Why:
///   A deletion of a name that does not exist deletes nothing,
///  and Git answers that itself;
///  the record
///       then has no branch to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit([...global, 'rev-parse', '--verify', '--quiet', reference]);
/// ```
pub fn branch_commit(context: &GitContext, root: Option<&Path>, name: &OsStr, remotes: bool) -> Option<String> {
    let (reference, _short): (OsString, String) = deleted_reference(context, root, name, remotes)?;
    return commit_of(context, root, reference.as_os_str());
}

/// What:
///  One query of the repository through a Git subcommand that reads only,
///  or nothing when Git cannot be asked at all.
/// Why:
///   Every question here is one such run,
/// and a query that cannot start answers nothing rather than
///  failing the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rev_parse({ context, root, arguments }): MetadataOutput | undefined;
/// ```
fn rev_parse(context: &GitContext, root: Option<&Path>, arguments: &[OsString]) -> Option<MetadataOutput> {
    let mut spelled: Vec<OsString> = prefix_for(context, root);
    spelled.extend(arguments.iter().cloned());
    return run_metadata_git(context.real_git.as_path(), spelled.as_slice(), context.overlay.as_slice()).ok();
}

/// What:
///  One Git word and a reference as the arguments of a query.
///  `&[&str]` are the words;
///  `&OsStr` is the reference,
///  passed byte for byte.
/// Why:
///   A reference name is bytes and not text (Git accepts a branch name that is not UTF-8),
///  so it reaches
///  Git unchanged where a text conversion would empty it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arguments = [...words.map(w => Buffer.from(w)), Buffer.from(reference)];
/// ```
fn query_arguments(words: &[&str], reference: &OsStr) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = Vec::<OsString>::with_capacity(words.len() + 1);
    for word in words {
        arguments.push(OsString::from(word));
    }
    arguments.push(reference.to_os_string());
    return arguments;
}

/// What:
///  The commit a reference points at,
/// or nothing when there is no such reference.
/// Why:
///   A deletion of a reference that does not exist deletes nothing,
/// and Git answers that itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit([...global, 'rev-parse', '--verify', '--quiet', reference]);
/// ```
fn commit_of(context: &GitContext, root: Option<&Path>, reference: &OsStr) -> Option<String> {
    let query: Vec<OsString> = query_arguments(&["rev-parse", "--verify", "--quiet"], reference);
    let output: MetadataOutput = rev_parse(context, root, query.as_slice())?;
    if !output.success {
        return None;
    }
    let commit: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    if commit.is_empty() {
        return None;
    }
    return Some(commit);
}

/// What:
///  The reference a deletion name means and the short name a reader restores from.
///  `Option<(OsString, String)>` is "the reference and its name,
/// or nothing".
/// Why:
///   A name is not always the branch it names:
///  Git expands `@{-1}` to the branch the worktree visited before the
///       current one,
///  so a record built from the typed name would look for `refs/heads/@{-1}`,
///  find
///       nothing and write no line at all,
///  which is the line the owner asked for.
///  The typed reference is asked about
///  first,
///  so a plain name costs the one query it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deletedReference({ context, root, name, remotes }): [string, string] | undefined;
/// ```
fn deleted_reference(context: &GitContext, root: Option<&Path>, name: &OsStr, remotes: bool) -> Option<(OsString, String)> {
    let intended: OsString = reference_of(name, remotes);
    // The typed name is the branch itself for every spelling but an expression of Git's own.
    if commit_of(context, root, intended.as_os_str()).is_some() {
        return Some((intended, name.to_string_lossy().into_owned()));
    }
    let expanded: OsString = expanded_branch_name(context, root, name, remotes)?;
    let reference: OsString = reference_of(expanded.as_os_str(), remotes);
    return Some((reference, expanded.to_string_lossy().into_owned()));
}

/// What:
///  The branch name a deletion operand expands to in its own namespace,
///  or nothing when Git expands it to no branch name there.
///  `Option<OsString>` is "the expanded name or nothing".
/// Why:
///   `git branch` expands its operand with Git's `copy_branchname` under the kind of the
///  namespace it deletes from,
///  and the two kinds differ:
///  `INTERPRET_BRANCH_LOCAL` reads `@{-1}` as the branch the worktree visited before and
///  refuses a mark that names a remote-tracking reference,
///  where `INTERPRET_BRANCH_REMOTE` reads the upstream and push marks and reads no
///       previous branch at all.
///  Each kind is asked through the one plumbing that performs it,
///  so a revision that is no branch name (`HEAD`,
///  `@`,
///  `main@{1}`) expands to nothing and the line is not written,
///  where general revision resolution answers `refs/heads/main` for `HEAD` and would name
///  a branch the deletion never took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = remotes
///   ? await runGit([...global, 'rev-parse', '--symbolic-full-name', '--verify', '--quiet', name])
///   : await runGit([...global, 'check-ref-format', '--branch', name]);
/// ```
fn expanded_branch_name(context: &GitContext, root: Option<&Path>, name: &OsStr, remotes: bool) -> Option<OsString> {
    if remotes {
        return expanded_remote_tracking_name(context, root, name);
    }
    let query: Vec<OsString> = query_arguments(&["check-ref-format", "--branch"], name);
    let output: MetadataOutput = rev_parse(context, root, query.as_slice())?;
    if !output.success {
        return None;
    }
    // A reference name holds no whitespace, so what Git ended with its line break is the name itself.
    let first: &[u8] = output.stdout.split(is_line_break).next().unwrap_or_default();
    let reference: &[u8] = first.trim_ascii();
    if reference.is_empty() {
        return None;
    }
    return Some(os_string_of(reference));
}

/// What:
///  The name a remote-tracking deletion operand expands to,
///  or nothing when Git's remote-tracking expansion reads none.
/// Why:
///   `git check-ref-format --branch` performs the local kind,
///  which refuses every mark that a remote-tracking deletion reads,
///  so this namespace asks `rev-parse --symbolic-full-name`,
///  which reads the marks of both kinds.
///  Only a reference in the remote-tracking namespace is an answer there,
///  which is exactly what `branch_interpret_allowed` leaves of a mark for
///       `INTERPRET_BRANCH_REMOTE`:
///  `@{-1}` and `HEAD` resolve to a local branch and are no remote-tracking name.
///  A branch mark always spells `@{`,
///  so an operand without it is no mark,
///  and answering it here would name a reference Git prefixes again and never deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!name.includes('@{')) return undefined;
/// const output = await runGit([...global, 'rev-parse', '--symbolic-full-name', '--verify', '--quiet', name]);
/// ```
fn expanded_remote_tracking_name(context: &GitContext, root: Option<&Path>, name: &OsStr) -> Option<OsString> {
    let bytes: &[u8] = name.as_encoded_bytes();
    let mark: bool = bytes.windows(2).any(is_branch_mark_start);
    if !mark {
        return None;
    }
    let query: Vec<OsString> = query_arguments(&["rev-parse", "--symbolic-full-name", "--verify", "--quiet"], name);
    let output: MetadataOutput = rev_parse(context, root, query.as_slice())?;
    if !output.success {
        return None;
    }
    let first: &[u8] = output.stdout.split(is_line_break).next().unwrap_or_default();
    let reference: &[u8] = first.trim_ascii();
    let short: &[u8] = reference.strip_prefix(REMOTES.as_bytes())?;
    if short.is_empty() {
        return None;
    }
    return Some(os_string_of(short));
}

/// What:
///  Whether two bytes of an operand open a branch mark,
///  which Git's own mark reading always spells `@{`.
/// Why:
///   A named function keeps the reading of one operand free of a closure that captures
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (pair: Buffer) => pair.equals(Buffer.from('@{'))
/// ```
fn is_branch_mark_start(pair: &[u8]) -> bool {
    return pair == b"@{".as_slice();
}

/// What:
///  Whether one byte ends the line a query's answer prints.
/// Why:
///   A named function keeps the split of one answer free of a closure that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a || byte === 0x0d
/// ```
fn is_line_break(byte: &u8) -> bool {
    return *byte == b'\n' || *byte == b'\r';
}

/// What:
///  A reference's exact bytes as operating-system text.
/// Why:
///   A branch name is bytes,
/// so a name that is not UTF-8 reaches the record as the name Git holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = Buffer.from(bytes);
/// ```
fn os_string_of(bytes: &[u8]) -> OsString {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        return OsString::from_vec(bytes.to_vec());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(bytes).into_owned());
    }
}

/// What:
///  Whether a commit is an ancestor of the command's `HEAD`,
/// which is what "merged" means to Git's own `git branch -d`.
///  `Option<bool>` is "the answer,
/// or nothing when Git cannot say".
/// Why:
///   A merged branch can be brought back from the branch it is merged into,
/// and an unmerged one cannot,
///  so
///       this is the one word that says how much the deletion costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit([...global, 'merge-base', '--is-ancestor', commit, 'HEAD']);
/// ```
pub fn merged_into_head(context: &GitContext, root: Option<&Path>, commit: &str) -> Option<bool> {
    let mut arguments: Vec<OsString> = prefix_for(context, root);
    for token in ["merge-base", "--is-ancestor"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(OsString::from(commit));
    arguments.push(OsString::from("HEAD"));
    let output: MetadataOutput =
        run_metadata_git(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice()).ok()?;
    // 0 is "yes", 1 is "no", and anything else is a repository Git could not read, which is no answer.
    return match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        Some(_) | None => None,
    };
}

/// What:
///  The facts one record of a deleted branch carries,
/// best effort,
/// one record per branch.
/// Why:
///   `git branch -D one two` deletes two refs,
///  and each needs its own commit if either is to be restored,
///  so
///       each gets its own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deletionFacts({ context, name }): [string, AuditFact[]] | undefined;
/// ```
pub fn deletion_facts(context: &GitContext, root: Option<&Path>, name: &OsStr, remotes: bool) -> Option<Vec<AuditFact>> {
    let (reference, branch): (OsString, String) = deleted_reference(context, root, name, remotes)?;
    let commit: String = commit_of(context, root, reference.as_os_str())?;
    let mut facts: Vec<AuditFact> = vec![
        AuditFact::Text("branch", branch),
        // The text field is the display name, and `referenceHex` is the identity, byte for byte, the way the
        //  record's own `gitDirHex`, `cwdHex` and `argumentsHex` keep theirs: a name that is not UTF-8 would
        //  otherwise restore a branch spelled with replacement characters.
        AuditFact::Text("reference", reference.to_string_lossy().into_owned()),
        AuditFact::Text("referenceHex", hex_of_bytes(reference.as_encoded_bytes())),
        AuditFact::Text("commit", commit.clone()),
    ];
    if let Some(merged) = merged_into_head(context, root, commit.as_str()) {
        facts.push(AuditFact::Flag("merged", merged));
    }
    return Some(facts);
}

/// What:
///  Restore advice a person can run,
///  as one text fact,
///  so a reader of the line does not have to look up the command.
///  `&[u8]`/`&str` borrow the full reference and the commit.
/// Why:
///   The whole point of the line is that the branch comes back,
///  and `git update-ref` restores a
///       local branch and a remote-tracking reference alike,
///  where `git branch` restores only the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git update-ref ${reference} ${commit}`
/// ```
pub fn restore_command(reference: &OsStr, commit: &str) -> String {
    return format!("git update-ref {} {commit}", reference.to_string_lossy());
}

/// The facts and their real-Git controls stay out of the release executable.
#[cfg(test)]
#[path = "branch_delete_audit_tests.rs"]
mod tests;
