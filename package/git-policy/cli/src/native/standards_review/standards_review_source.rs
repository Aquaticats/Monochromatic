//! What:
//!  The Git reads of the standards-review shadow run,
//! in as few real-Git starts as the data allows:
//! one `git show`
//!  for the patch and one `git cat-file --batch -z` for the new-side text of the excerpted code files.
//! Why:
//!  The root `mise.toml` task `shadow-review` starts Git twenty-odd times per landing (`rev-parse`,
//! `log`,
//! `show`,
//!  `diff-tree`,
//! then `show` and `diff -U0` for each of up to eight files).
//! The landing already knows the commit name,
//!  its message and the changed names,
//! so the subject and the names need no start,
//! the zero-context ranges come out of
//!  the patch,
//! and the file texts come in one batch.
//! A commit with no code files costs no start at all.
//! A changed path is carried as the bytes Git printed for it:
//! a path is not text,
//! and the name Git is asked about must be the name it reported,
//! or it answers
//!  with another file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const patch = git('show', '--format=', '--patch', sha); const blobs = catFileBatchNul(codeFiles.map((name) => concat(sha, ':', name.bytes, 0)));
//! ```

use crate::standards_review_input::{ChangedPath, LandedCommit, MAX_EXCERPT_FILES};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;

/// The most patch bytes read,
/// the script's `maxBuffer` of 64 MiB.
pub const MAX_PATCH_BYTES: usize = 1 << 26;

/// What:
///  Whether one byte is the line feed that ends a `git cat-file --batch` header.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeedByte = (byte: number) => byte === '\n'.charCodeAt(0);
/// ```
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one byte is the NUL byte that ends each name of `git diff-tree -z`.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Read the patch of `sha` with one `git show`.
/// Why:
///   Color and external diff drivers are switched off so the text does not depend on the account's configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git('show', '--format=', '--patch', sha)
/// ```
fn read_patch(context: &GitContext, cwd: &Path, sha: &str) -> Result<String, String> {
    let request: GitRequest = GitRequest::new(cwd, &["show", "--format=", "--patch", "--no-color", "--no-ext-diff", sha]);
    let output = match run_git(context, &request) {
        Ok(done) => done,
        Err(error) => return Err(format!("starting git show failed: {error}")),
    };
    if !output.succeeded() {
        return Err(format!("git show {sha} failed: {}", output.error_text()));
    }
    if output.stdout.len() > MAX_PATCH_BYTES {
        return Err(format!("the patch of {sha} is over {MAX_PATCH_BYTES} bytes"));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).into_owned());
}

/// What:
///  Split `git cat-file --batch` output for the requested names into texts;
/// a name Git reports `missing` has none.
///  Every byte of the output must belong to one of the requested names,
/// so a reply nothing asked for is a failure,
/// not text
///  hung on the last name.
/// Why:
///   A file the commit deleted has no new-side text,
/// and a size says where content ends,
/// never a delimiter;
/// a reply count that differs from the request count means the request named
///   more or fewer objects than the review meant to ask about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseBatch({ output, names }): Map<ChangedPath, string>
/// ```
pub fn parse_batch(output: &[u8], names: &[ChangedPath]) -> Result<BTreeMap<ChangedPath, String>, String> {
    let mut texts: BTreeMap<ChangedPath, String> = BTreeMap::new();
    let mut offset: usize = 0;
    for name in names {
        let Some(relative) = output[offset..].iter().position(is_line_feed_byte) else {
            return Err(format!("git cat-file omitted the header for {name}"));
        };
        let header: String = String::from_utf8_lossy(&output[offset..offset + relative]).into_owned();
        offset += relative + 1;
        let fields: Vec<&str> = header.split(' ').collect();
        if fields.len() == 2 && fields[1] == "missing" {
            continue;
        }
        let announced_size: Option<usize> = match fields.get(2) {
            Some(text) => text.parse::<usize>().ok(),
            None => None,
        };
        let size: usize = match (fields.len(), fields.get(1), announced_size) {
            (3, Some(&"blob"), Some(size)) => size,
            _ => return Err(format!("git cat-file returned an unexpected header for {name}: {header}")),
        };
        // The content must end before the output does, and a line feed must follow it.
        let end: usize = match offset.checked_add(size) {
            Some(end) if end < output.len() && output[end] == b'\n' => end,
            _ => return Err(format!("git cat-file returned truncated content for {name}")),
        };
        texts.insert(name.clone(), String::from_utf8_lossy(&output[offset..end]).into_owned());
        offset = end + 1;
    }
    if offset != output.len() {
        return Err(format!("git cat-file answered more objects than the {} the review asked for", names.len()));
    }
    return Ok(texts);
}

/// What:
///  The names `git diff-tree` reports for the landed commit,
/// read raw (NUL separated,
/// so no name is quoted) in one start.
///  A root commit and a merge report none.
/// Why:
///   The service has only the commit name;
/// the landing's own list is not in the work item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git('diff-tree', '--no-commit-id', '--name-only', '-r', '-z', sha).split('\0').filter(Boolean)
/// ```
pub fn read_names(context: &GitContext, cwd: &Path, sha: &str) -> Result<Vec<ChangedPath>, String> {
    let request: GitRequest = GitRequest::new(cwd, &["diff-tree", "--no-commit-id", "--name-only", "-r", "-z", sha]);
    let output = match run_git(context, &request) {
        Ok(done) => done,
        Err(error) => return Err(format!("starting git diff-tree failed: {error}")),
    };
    if !output.succeeded() {
        return Err(format!("git diff-tree {sha} failed: {}", output.error_text()));
    }
    let mut names: Vec<ChangedPath> = Vec::new();
    for name in output.stdout.split(is_nul_byte) {
        if !name.is_empty() {
            names.push(ChangedPath::from_bytes(name.to_vec()));
        }
    }
    return Ok(names);
}

/// What:
///  The commit message of `sha`,
/// read in one start.
/// Why:
///   The subject is folded from it in-process (`commit_subject`),
/// as `git log --format=%s` folds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git('show', '--no-patch', '--format=%B', sha)
/// ```
pub fn read_message(context: &GitContext, cwd: &Path, sha: &str) -> Result<String, String> {
    let request: GitRequest = GitRequest::new(cwd, &["show", "--no-patch", "--format=%B", sha]);
    let output = match run_git(context, &request) {
        Ok(done) => done,
        Err(error) => return Err(format!("starting git show failed: {error}")),
    };
    if !output.succeeded() {
        return Err(format!("git show {sha} failed: {}", output.error_text()));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).into_owned());
}

/// What:
///  One `git cat-file --batch -z` request body for the objects of `sha` at `names`:
/// each spec is the commit name,
/// a colon,
/// the path's own bytes,
/// and the NUL byte that ends it.
/// Why:
///   A path may contain a line feed,
/// which would end the request in the middle of a name and let the rest of the name be read as
///  another object,
/// and a NUL byte cannot appear in a path,
/// so NUL is the only framing no name can
///  reach into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const specs = names.map((n) => Buffer.concat([utf8(sha), colon, n.bytes, nul]));
/// ```
fn batch_request(sha: &str, names: &[ChangedPath]) -> Vec<u8> {
    let mut specs: Vec<u8> = Vec::new();
    for name in names {
        specs.extend_from_slice(sha.as_bytes());
        specs.push(b':');
        specs.extend_from_slice(name.as_bytes());
        specs.push(0);
    }
    return specs;
}

/// What:
///  Read the patch and the excerpted files' new-side texts of a landed commit.
///  `code_files` are the code names in name order;
/// only the first `MAX_EXCERPT_FILES` are fetched,
/// and each is fetched by its own bytes.
/// Why:
///   It is the whole of the review's Git use:
/// two starts.
/// A name Git has no object for (a deleted file,
/// a name this repository's tree does not hold) is
///  absent from the texts,
/// never another object's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commit = await readLandedCommit({ context, cwd, sha, subject, codeFiles });
/// ```
pub fn read_landed_commit(context: &GitContext, cwd: &Path, sha: &str, subject: &str, code_files: &[ChangedPath]) -> Result<LandedCommit, String> {
    let patch: String = read_patch(context, cwd, sha)?;
    let wanted: Vec<ChangedPath> = code_files.iter().take(MAX_EXCERPT_FILES).cloned().collect();
    let mut request: GitRequest = GitRequest::new(cwd, &[OsString::from("cat-file"), OsString::from("--batch"), OsString::from("-z")]);
    request.input = Some(batch_request(sha, wanted.as_slice()));
    let output = match run_git(context, &request) {
        Ok(done) => done,
        Err(error) => return Err(format!("starting git cat-file failed: {error}")),
    };
    if !output.succeeded() {
        return Err(format!("git cat-file failed: {}", output.error_text()));
    }
    let contents: BTreeMap<ChangedPath, String> = parse_batch(output.stdout.as_slice(), wanted.as_slice())?;
    return Ok(LandedCommit { sha: String::from(sha), subject: String::from(subject), patch, contents });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_source_tests.rs"]
mod tests;
