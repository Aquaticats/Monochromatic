//! What:
//!  The object questions a landing and its client ask Git when the in-process object reader declines the database:
//!  a commit's own bytes,
//!  whether the objects a request names are gone,
//!  and whether the trees it embeds are complete.
//! Why:
//!  The reader declines a store it cannot vouch for (a replace ref,
//!  a reftable ref store,
//!  a shallow or grafted history) rather than answer an object Git replaces,
//!  and every caller then asks Git,
//!  which reads the replacement itself or words the failure.
//!  A caller that took the decline as an answer lost what only the object holds:
//!  the message and parents a reflog row repeats,
//!  and content a prune removed was reported as present,
//!  so the import died on it and left a crash report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = await commitBytes(git, repo, name); const gone = await objectsMissing(git, repo, names); const one = await objectMissing(git, repo, name);
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::path::Path;

/// What:
///  The bytes `git cat-file commit <name>` prints:
/// the commit Git reads,
///  the replacement a replace ref names included,
///  or `None` when Git reads no commit of that name.
/// Why:
///  The bytes carry the parents and the message,
/// and a reflog row repeats both,
/// so one start answers the whole question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitBytes = (git, repo, name) => git(['cat-file', 'commit', name]).stdout;
/// ```
pub(crate) fn commit_bytes(git: &GitContext, repository: &Path, name: &str) -> Option<Vec<u8>> {
    // A landing reads one commit for its objects and again for the reflog row of a one-commit batch, and the bytes do not
    //  change between the two questions, so the second is answered from what the first read. A failed start is never
    //  remembered: the next question may start Git where this one could not.
    if let Some(read) = crate::landing_state_carried_by_its_task::commit_bytes_already_read(name) {
        return Some(read);
    }
    let output: GitOutput = run(git, repository, &["cat-file", "commit", name])?;
    let bytes: Vec<u8> = output.succeeded().then_some(output.stdout)?;
    crate::landing_state_carried_by_its_task::remember_commit_bytes(name, bytes.as_slice());
    return Some(bytes);
}

/// What:
///  Whether any of these objects,
///  or anything below the trees among them,
///  is gone from the object store Git reads.
/// Why:
///  `rev-list --objects --missing=print` prints `?<name>` for each object it cannot read and exits zero,
///  so absence is read from a line and never from how Git ended:
/// a Git a signal ended prints no such line,
///  and an object it fails for another reason is not called gone.
///  A commit name is not one of these:
///  walking a commit also walks its history,
///  which no question here asks.
///
/// The names travel on standard input rather than the command line,
///  so a request whose trees and blobs would not fit one command line is still asked instead of failing to start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = (git, repo, inputDir, names) => git(['rev-list', '--objects', '--no-object-names', '--missing=print', '--stdin'], names.join('\n')).stdout.some((line) => line.startsWith('?'));
/// ```
pub(crate) fn objects_missing(git: &GitContext, repository: &Path, input_directory: &Path, names: &[&str]) -> bool {
    if names.is_empty() {
        return false;
    }
    let length: usize = names.iter().map(|name| { return name.len() + 1; }).sum();
    let mut input: String = String::with_capacity(length);
    for name in names {
        input.push_str(name);
        input.push('\n');
    }
    let arguments: [&str; 5] = ["rev-list", "--objects", "--no-object-names", "--missing=print", "--stdin"];
    let mut request: GitRequest = GitRequest::new(repository, &arguments);
    request.input = Some(input.into_bytes());
    // The request's input file is made in the repository's own scratch directory, never the system temporary directory.
    let scoped: GitContext = GitContext { common_dir: Some(input_directory.to_path_buf()), ..git.clone() };
    let Some(output): Option<GitOutput> = run_git(&scoped, &request).ok() else { return false };
    return output.stdout.split(|byte| { return *byte == b'\n'; }).any(|line| { return line.starts_with(b"?"); });
}

/// What:
///  Whether the one object `name` is gone from the object store Git reads.
/// Why:
///  `cat-file -e` answers 1 for an object it cannot find and 128 for a name it cannot read at all,
///  and a signal leaves no code:
/// only the 1 is absence,
///  so a commit is asked this way and its history is never walked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = (git, repo, name) => git(['cat-file', '-e', name]).code === 1;
/// ```
pub(crate) fn object_missing(git: &GitContext, repository: &Path, name: &str) -> bool {
    return matches!(run(git, repository, &["cat-file", "-e", name]), Some(output) if output.code == Some(1));
}

/// What:
///  Whether any of the commits in `commits` is gone.
/// Why:
///  Presence alone is what the reader answered for a commit,
///  so the fallback asks presence alone,
///  one start each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = (git, repo, requests) => requests.some((r) => r.commit && objectMissing(git, repo, r.commit));
/// ```
pub(crate) fn built_commits_missing(git: &GitContext, repository: &Path, commits: &[&str]) -> bool {
    for commit in commits {
        if object_missing(git, repository, commit) {
            return true;
        }
    }
    return false;
}

/// One Git start,
/// or `None` when Git could not be started:
/// the question is then unanswered rather than answered with a guess.
fn run(git: &GitContext, repository: &Path, arguments: &[&str]) -> Option<GitOutput> {
    return run_git(git, &GitRequest::new(repository, arguments)).ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_object_questions_left_to_git_tests.rs"]
mod tests;
