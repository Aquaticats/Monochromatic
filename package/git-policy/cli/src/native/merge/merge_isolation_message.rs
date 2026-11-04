//! What:
//!  The text of an isolated merge's commit message and the name Git gives the merged commit in it.
//! Why:
//!  The merge commit must read exactly as native Git would write it: the default subject comes from `git
//!  fmt-merge-msg`, which `git merge` itself uses, fed the description line `git merge` builds for the commit-ish
//!  (`branch 'x'`, `tag 'x'`, `remote-tracking branch 'x'` or `commit 'x'`), and any message is cleaned the way a
//!  commit that opens no editor cleans it.
//!  A commit-ish whose description cannot be built the same way is not guessed: the caller forwards the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const description = await describeMergeSource({ reference, oid });
//! const message = await mergeMessage({ description, log, given });
//! ```

/// Running real Git on a request.
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};

/// What:
///  The line `git merge` hands to `git fmt-merge-msg` for one merged commit, or nothing when the name needs a form
///  this reader does not build.
/// Why:
///   `merge_name` in Git's `builtin/merge.c` resolves the name as a reference first: a branch, a tag or a
///  remote-tracking branch is named by its short name and `of .`, and a name that is no reference is a bare commit.
///  The `<name>^` and `<name>~<n>` forms have their own wording, so a name containing `^`, `~`, `@`, `:` or a space
///  and no full reference is left to native Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeMergeSource({ reference, fullName, oid }): string | undefined;
/// ```
pub fn describe_merge_source(reference: &str, full_name: &str, oid: &str) -> Option<String> {
    if let Some(name) = full_name.strip_prefix("refs/heads/") {
        return Some(format!("{oid}\t\tbranch '{name}' of .\n"));
    }
    if let Some(name) = full_name.strip_prefix("refs/tags/") {
        return Some(format!("{oid}\t\ttag '{name}' of .\n"));
    }
    if let Some(name) = full_name.strip_prefix("refs/remotes/") {
        return Some(format!("{oid}\t\tremote-tracking branch '{name}' of .\n"));
    }
    if full_name.is_empty()
        && !reference.is_empty()
        && reference.bytes().all(|byte| { return byte.is_ascii_hexdigit(); })
    {
        return Some(format!("{oid}\t\tcommit '{reference}'\n"));
    }
    return None;
}

/// What:
///  The object names that `.git/FETCH_HEAD` lists for merging: its lines whose name is followed by two tabs.
/// Why:
///   `handle_fetch_head` in Git's `builtin/merge.c` takes every line that starts with a full hexadecimal name and two tabs
///  as a head to merge and skips the lines marked `not-for-merge`; a pull merges `FETCH_HEAD`, and this wrapper completes
///  that merge only when exactly one head is listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fetchHeadMergeHeads(text: string): string[];
/// ```
pub fn fetch_head_merge_heads(text: &str) -> Vec<String> {
    // The list of names found, in file order.
    let mut heads: Vec<String> = Vec::new();
    // `lines()` splits at line feeds like `text.split('\n')`.
    for line in text.lines() {
        // A name is as long as the hash: 40 hexadecimal digits for SHA-1, 64 for SHA-256.
        let Some((name, rest)) = line.split_once('\t') else { continue };
        let is_name: bool = (name.len() == 40 || name.len() == 64) && name.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
        // Two tabs mean the head is for merging; a word after the first tab (`not-for-merge`) means it is not.
        if is_name && rest.starts_with('\t') {
            heads.push(String::from(name));
        }
    }
    return heads;
}

/// What:
///  The default merge message for a described commit: Git's own `fmt-merge-msg`, with the shortlog choice given.
/// Why:
///   The subject depends on the checked-out branch and on the `merge.log`, `merge.suppressDest` and
///  `merge.branchdesc` settings, which only Git applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function defaultMergeMessage({ context, description, log }): Promise<Uint8Array>;
/// ```
pub fn default_merge_message(
    context: &GitContext,
    description: &str,
    log: Option<bool>,
) -> Result<Vec<u8>, String> {
    let mut arguments: Vec<&str> = vec!["fmt-merge-msg"];
    match log {
        Some(true) => arguments.push("--log"),
        Some(false) => arguments.push("--no-log"),
        None => {}
    }
    let mut request: GitRequest = GitRequest::new(std::path::Path::new("."), &arguments);
    request.input = Some(description.as_bytes().to_vec());
    let output = run_git_checked(context, &request).map_err(|error| { return error.to_string(); })?;
    return Ok(output.stdout);
}

/// What:
///  A message cleaned as a commit that opens no editor cleans it: trailing space stripped, blank lines collapsed,
///  one final line feed.
/// Why:
///   `git merge` runs the message through the same cleanup `git commit --cleanup=whitespace` does when it opens no
///  editor; `git stripspace` is that cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cleanMessage({ context, message }): Promise<Uint8Array>;
/// ```
pub fn clean_message(context: &GitContext, message: &[u8]) -> Result<Vec<u8>, String> {
    let mut request: GitRequest = GitRequest::new(std::path::Path::new("."), &["stripspace"]);
    request.input = Some(message.to_vec());
    let output = run_git_checked(context, &request).map_err(|error| { return error.to_string(); })?;
    return Ok(output.stdout);
}

/// Description lines and message cleanup.
#[cfg(test)]
#[path = "merge_isolation_message_tests.rs"]
mod tests;
