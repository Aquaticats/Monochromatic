//! What:
//!  The commit-only finding for a staged deletion that an explicit-path commit cannot record because the path is
//!  still on disk.
//! Why:
//!  An explicit pathspec selects worktree content,
//! never index state,
//! so a deletion staged with `git rm --cached`
//!  for a path that stays on disk (an ignored editor file,
//! for example) is not the content the pathspec selects.
//!  Without this finding Git answers "nothing to commit",
//! or a private `git add` fails with a hint to force-add the
//!  ignored path,
//! which would re-track the file instead of recording its removal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const present = stagedDeletions.filter((path) => existsSync(join(repositoryRoot, path)));
//! ```

use crate::commit_selection_staging::{ITEMS_BYTES_LIMIT, excludes_paths};
use crate::path_bytes::{os_from_path_bytes, path_message_text};
use crate::private_index_request::caller_pathspec_request;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
use std::ffi::OsString;
use std::path::Path;

/// The core of the finding;
/// the same core as the other commit-only findings.
pub const STAGED_DELETION_CORE_ID: &str = "commit-only";

/// The code of the finding,
/// reported as `commit-only/staged-deletion-worktree-present`.
pub const STAGED_DELETION_CODE: &str = "staged-deletion-worktree-present";

/// What:
///  The selected paths whose deletion is staged in the real index while an entry still exists at that worktree path.
/// Why:
///   Only these paths make an explicit-path commit disagree with the staged state;
/// the read touches no index,
///  object store or worktree file.
///  The elements are the selection's own,
///  whether the caller wrote them on the command line or in the file `--pathspec-from-file` names,
///  and they reach Git with the pathspec modes the caller set:
/// `--literal-pathspecs commit -- 'st*r.txt'` asks about the file of that name and never about a glob,
///  so this reading answers for the same selection the staging stages.
///  One start carries at most `ITEMS_BYTES_LIMIT` of elements,
/// and every subtracting element rides in each of them,
///  so a pathspec file past one command line is still answered,
/// and an exclusion narrows the paths of every start rather than one.
///  The paths come back in Git's own order:
/// byte order,
/// whichever start listed them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const chunk of elementChunks(elements)) git(['diff-index', '--cached', '--name-only', '--diff-filter=D', '-z', base, '--', ...chunk], { index: realIndex });
/// ```
pub fn staged_deletions_present_on_disk(
    context: &GitContext,
    cwd: &Path,
    repository_root: &Path,
    real_index: &Path,
    base_revision: &str,
    elements: &[OsString],
) -> Result<Vec<Vec<u8>>, GitFailure> {
    let mut remaining: Vec<Vec<u8>> = Vec::new();
    for chunk in element_chunks(elements) {
        let mut arguments: Vec<OsString> = ["diff-index", "--cached", "--name-only", "--diff-filter=D", "-z", base_revision, "--"]
            .iter()
            .map(OsString::from)
            .collect();
        arguments.extend(chunk.iter().cloned());
        let output: GitOutput = run_git_checked(context, &caller_pathspec_request(context, cwd, arguments.as_slice(), Some(real_index), None))?;
        for path in output.stdout.split(is_nul_byte) {
            if path.is_empty() {
                continue;
            }
            // A path whose file is still on disk keeps its staged deletion, as the earlier filter kept it.
            if repository_root.join(os_from_path_bytes(path)).symlink_metadata().is_ok() {
                remaining.push(path.to_vec());
            }
        }
    }
    remaining.sort();
    remaining.dedup();
    return Ok(remaining);
}

/// What:
///  The elements one `diff-index` start carries:
/// every element that subtracts paths,
/// and as many elements that name them as one command line takes.
/// Why:
///   A start that is missing a subtracting element would list a path the caller excluded,
/// and a start that names one path would miss the others,
///  so each start carries all of the subtracting elements and a bounded share of the naming ones.
///  A list of nothing but subtracting elements is one start,
/// because that is native Git's reading of it (every path except these),
/// and each such element is short enough to ride alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const elementChunks = (elements) => group(elements.filter((each) => !excludesPaths(each)), ITEMS_BYTES_LIMIT).map((share) => [...elements.filter(excludesPaths), ...share]);
/// ```
fn element_chunks(elements: &[OsString]) -> Vec<Vec<OsString>> {
    let mut subtracting: Vec<OsString> = Vec::new();
    let mut subtracting_bytes: u64 = 0;
    for element in elements {
        if excludes_paths(element.as_os_str()) {
            subtracting_bytes += bytes_of(element);
            subtracting.push(element.clone());
        }
    }
    let mut chunks: Vec<Vec<OsString>> = Vec::new();
    let mut naming: Vec<OsString> = Vec::new();
    let mut bytes: u64 = 0;
    for element in elements {
        if excludes_paths(element.as_os_str()) {
            continue;
        }
        let size: u64 = bytes_of(element);
        if !naming.is_empty() && subtracting_bytes + bytes + size > ITEMS_BYTES_LIMIT {
            let mut chunk: Vec<OsString> = subtracting.clone();
            chunk.append(&mut naming);
            chunks.push(chunk);
            bytes = 0;
        }
        bytes += size;
        naming.push(element.clone());
    }
    if !naming.is_empty() {
        let mut chunk: Vec<OsString> = subtracting;
        chunk.append(&mut naming);
        chunks.push(chunk);
    } else if subtracting_bytes > 0 {
        chunks.push(subtracting);
    }
    return chunks;
}

/// The bytes one element takes on a command line,
/// with the byte that separates it from the next.
fn bytes_of(element: &OsString) -> u64 {
    return element.as_encoded_bytes().len() as u64 + 1;
}

/// What:
///  Whether one byte is the NUL byte that ends each record of a `-z` listing.
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
///  The message of the finding:
/// the paths,
/// why a pathspec cannot record them,
/// and each valid way forward.
/// Why:
///   Both intents are legitimate (commit the removal,
/// or re-track the file),
/// and the diagnostic must not pick one;
///  forcing an add of an ignored path belongs only to the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git commit with explicit paths selects worktree content, so it cannot record the staged deletion of ...`
/// ```
pub fn staged_deletion_message(paths: &[Vec<u8>]) -> String {
    let mut names: Vec<String> = Vec::new();
    for path in paths {
        names.push(path_message_text(path.as_slice()));
    }
    let listed: String = serde_json::to_string(&names).unwrap_or_default();
    return format!(
        "monochromatic-git-policy-cli: git commit with explicit paths selects worktree content, so it cannot record the staged deletion of \
{listed}: the path stays on disk. Choose explicitly: to commit the deletion and keep the file, pass --no-only without \
paths (this commits every staged change, not only these paths); to commit the file's current content instead, stage it \
again with git add first (an ignored path also needs git add --force, which tracks the file again)."
    );
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_staged_deletions_tests.rs"]
mod tests;
