//! What:
//!  One reading of a repository's kept references:
//!  which of them there are,
//!  what each names,
//!  which of their commits a branch or a remote has since reached,
//!  and what the objects only they reach weigh.
//! Why:
//!  The pass and the listing judge a repository from the same reading,
//!  so what the person is told is what the next pass does,
//!  and neither asks Git a question the other does not.
//!  Four Git starts answer everything:
//!  a `for-each-ref` of the folder,
//!  a `rev-list` of the objects only the kept references reach,
//!  and,
//! when the pass has to weigh a set of them,
//! the same `rev-list` for that set,
//!  and a `cat-file` that sums the objects it names.
//!  The measurement the limits rest on was taken with these commands
//!  against this repository on 2026-11-12,
//!  read-only:
//!  `git for-each-ref refs/cli-git/ --format='%(refname) %(objectname) %(creatordate:iso-strict)'`
//!  printed five references,
//! all of them `refs/cli-git/kept/<number>`,
//!  written between 2026-11-07T13:01:46-05:00 and 2026-11-07T13:48:16-05:00,
//!  and nothing else lived under `refs/cli-git/`;
//!  `git rev-list --count <commit> --not --branches --remotes --tags`
//!  answered one for each of them,
//!  so none had reached a branch;
//!  and the reading below named 27 objects only they reach
//!  (5 commits,
//! 13 trees,
//! 9 blobs),
//!  7,573 bytes on disk and 258,055 bytes before compression.
//!  The commands and their output are in
//!  `doc/troubleshooting/cli-git-kept-references-are-pruned-by-size-and-age.md`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = await readTheKeptReferences(git, commonDir); // { kept, bytesOnlyTheyReach }
//! ```

use crate::kept_reference_name::batch_and_rescued_of;
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::collections::HashSet;
use std::path::Path;

/// The folder the reading asks about,
/// which is also the only one a recorded name may come from.
const KEPT_FOLDER: &str = "refs/cli-git/kept/";

/// The pattern that leaves every kept reference out of the refs a question excludes,
/// so what the reading weighs is what only the kept references reach.
const NOT_THE_KEPT_ONES: &str = "--exclude=refs/cli-git/kept/*";

/// The references a kept commit must be reachable from to hold nothing:
/// a local branch,
/// a remote-tracking branch and a tag.
/// Why:
///  Every other reference is a holder in flight or a holder of its own:
///  the landing's `refs/worktree/cli-git-service/<batch>`,
///  the carried work of an isolated merge,
///  a private fetch of a push,
///  a person's stash.
///  A pass that read any of them as a branch would remove the reference of a commit nothing else
///  would keep once that holder goes,
///  and say that nothing was lost.
const THE_HOLDERS_THAT_SETTLE_A_COMMIT: [&str; 3] = ["--branches", "--remotes", "--tags"];

/// What:
///  One kept reference of the repository.
/// Why:
///   The pass and the listing need its four answers together:
///  the commit it names,
///  the batch and the form its name carries,
///  and whether a branch has since reached that commit,
///  which is what makes it worth nothing to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OneKeptReference = { reference: string; commit: string; batch: bigint; rescued: boolean; reachedABranch: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OneKeptReference {
    /// The full reference name.
    pub reference: String,
    /// The commit it names.
    pub commit: String,
    /// The batch its name carries.
    pub batch: u64,
    /// Whether it is the name beside the batch's own.
    pub rescued: bool,
    /// Whether a branch,
    /// a remote-tracking reference or a tag has since reached the commit,
    /// so the reference holds nothing any more.
    pub reached_a_branch: bool,
}

/// What:
///  What one reading of the repository says.
/// Why:
///  Every question the pass asks is answered from it,
///  so no two steps of one pass can read two different repositories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheReading = { kept: OneKeptReference[]; bytesOnlyTheyReach: number };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TheReading {
    /// The kept references,
    /// in the order Git lists them.
    pub kept: Vec<OneKeptReference>,
    /// The bytes the objects reachable only through kept references weigh on disk.
    pub bytes_only_they_reach: u64,
}

/// What:
///  An exit status as a person reads it,
///  or the words for a command a signal ended.
/// Why:
///   A failure of a question is reported with what Git answered,
///  and a step a signal ended answered no status at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exitStatusWords = (code) => code === null ? 'no exit status' : String(code);
/// ```
fn exit_status_words(code: Option<i32>) -> String {
    return match code {
        Some(number) => number.to_string(),
        None => String::from("no exit status"),
    };
}

/// What:
///  The commits of a set of kept references.
/// Why:
///  A reading asks Git about what the references name,
///  and the names themselves are read beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitsOf = (ones) => ones.map((one) => one.commit);
/// ```
fn commits_of(ones: &[OneKeptReference]) -> Vec<String> {
    let mut commits: Vec<String> = Vec::with_capacity(ones.len());
    for one in ones {
        commits.push(one.commit.clone());
    }
    return commits;
}

/// What:
///  One Git question of the reading,
///  answered as text or as the reason it gave no answer.
/// Why:
///  Every step of the pass reports a failed question the same way,
///  and nothing is removed on a question nobody answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = async (request) => output.code === 0 ? output.stdout : throw new Error(words);
/// ```
async fn answered_as_text(git: &GitContext, request: &GitRequest) -> Result<Vec<u8>, String> {
    let output: GitOutput = match run_git_awaiting(git, request).await {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git could not be run ({}): {error}", crate::transaction_git::command_text(request.arguments.as_slice()))),
    };
    if output.code != Some(0) {
        return Err(format!(
            "`{}` answered {}: {}",
            crate::transaction_git::command_text(request.arguments.as_slice()),
            exit_status_words(output.code),
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    return Ok(output.stdout);
}

/// What:
///  The references the repository holds under the kept folder,
///  each with the commit it names.
/// Why:
///  One `for-each-ref` answers every name and every value,
///  and a name that is no kept reference the wrapper writes is left out here,
///  so nothing else under `refs/cli-git/` can reach the pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = (text) => text.split('\n').flatMap((line) => isKept(line.split('\t')[0]) ? [{ reference, commit }] : []);
/// ```
async fn listed_kept_references(git: &GitContext, common_dir: &Path) -> Result<Vec<OneKeptReference>, String> {
    let request: GitRequest =
        GitRequest::new(common_dir, &["for-each-ref", "--format=%(refname)%09%(objectname)", KEPT_FOLDER]);
    let text: String = String::from_utf8_lossy(answered_as_text(git, &request).await?.as_slice()).into_owned();
    let mut found: Vec<OneKeptReference> = Vec::new();
    for line in text.lines() {
        let Some((reference, commit)) = line.split_once('\t') else {
            continue;
        };
        let Some((batch, rescued)) = batch_and_rescued_of(reference) else {
            continue;
        };
        found.push(OneKeptReference { reference: String::from(reference), commit: String::from(commit), batch, rescued, reached_a_branch: false });
    }
    return Ok(found);
}

/// What:
///  The objects reachable from `commits` and from no reference outside the kept folder,
///  asked in one `rev-list`.
/// Why:
///  The tips are given on standard input,
///  so a repository holding thousands of kept references asks no command line longer than its limit,
///  and `--not --exclude=<the kept folder> --all` is what leaves every other reference out.
///  This is the weight of what the kept references hold:
///  an object in the answer stays only while a kept reference reaches it,
///  and it is what the disk gives back when the reference goes.
///  It is not the answer to whether a branch reaches a commit,
///  which `objects_a_branch_does_not_reach` asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const onlyTheseReach = (commits) => revList({ stdin: commits.join('\n') + '\n', args: ['rev-list', '--objects', '--stdin', '--not', '--exclude=refs/cli-git/kept/*', '--all'] });
/// ```
async fn objects_only_these_reach(git: &GitContext, common_dir: &Path, commits: &[String]) -> Result<Vec<String>, String> {
    if commits.is_empty() {
        return Ok(Vec::new());
    }
    let mut input: String = String::new();
    for commit in commits {
        input.push_str(commit.as_str());
        input.push('\n');
    }
    let mut request: GitRequest = GitRequest::new(common_dir, &["rev-list", "--objects", "--stdin", "--not", NOT_THE_KEPT_ONES, "--all"]);
    request.input = Some(input.into_bytes());
    let text: String = String::from_utf8_lossy(answered_as_text(git, &request).await?.as_slice()).into_owned();
    return Ok(objects_of(text.as_str()));
}

/// What:
///  The objects reachable from `commits` and from no local branch,
///  remote-tracking branch or tag,
///  asked in one `rev-list`.
/// Why:
///  A kept commit that appears in this answer is reached by none of them,
///  so the reference that holds it holds the only handle on it;
///  one that does not appear is reached by one of them and holds nothing,
///  which is the one rule that removes a reference whatever its age.
///  The holders in flight (a landing's scratch reference,
///  the carried work of a merge,
///  a private fetch,
///  a stash) are left out of the question on purpose:
///  each of them goes away by itself,
/// and a reference removed as "reached" because of one
///  would lose the commit when that holder goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aBranchDoesNotReach = (commits) => revList({ stdin: commits.join('\n') + '\n', args: ['rev-list', '--objects', '--stdin', '--not', ...THE_HOLDERS_THAT_SETTLE_A_COMMIT] });
/// ```
async fn objects_a_branch_does_not_reach(git: &GitContext, common_dir: &Path, commits: &[String]) -> Result<Vec<String>, String> {
    if commits.is_empty() {
        return Ok(Vec::new());
    }
    let mut input: String = String::new();
    for commit in commits {
        input.push_str(commit.as_str());
        input.push('\n');
    }
    let mut arguments: Vec<&str> = vec!["rev-list", "--objects", "--stdin", "--not"];
    arguments.extend(THE_HOLDERS_THAT_SETTLE_A_COMMIT.iter().copied());
    let mut request: GitRequest = GitRequest::new(common_dir, arguments.as_slice());
    request.input = Some(input.into_bytes());
    let text: String = String::from_utf8_lossy(answered_as_text(git, &request).await?.as_slice()).into_owned();
    return Ok(objects_of(text.as_str()));
}

/// What:
///  The object names of `rev-list --objects` output.
/// Why:
///  A line is the object and,
/// below a tree,
/// a tab and the path it is at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectsOf = (text) => text.split('\n').filter(Boolean).map((line) => line.split('\t')[0]);
/// ```
fn objects_of(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some((object, _path)) = line.split_once('\t') {
            found.push(String::from(object));
        } else if !line.is_empty() {
            found.push(String::from(line));
        }
    }
    return found;
}

/// What:
///  Whether a local branch,
///  a remote-tracking branch or a tag reaches `commit`.
/// Why:
///   A pass asks this again immediately before it removes a reference for that reason,
///  because a branch can move off the commit between the listing and the removal,
///  and the conditional removal would not see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aBranchReaches = async (commit) => (await git(['rev-list', '--count', commit, '--not', ...THE_HOLDERS_THAT_SETTLE_A_COMMIT])).trim() === '0';
/// ```
pub async fn a_branch_reaches(git: &GitContext, common_dir: &Path, commit: &str) -> Result<bool, String> {
    let mut arguments: Vec<&str> = vec!["rev-list", "--count", commit, "--not"];
    arguments.extend(THE_HOLDERS_THAT_SETTLE_A_COMMIT.iter().copied());
    let request: GitRequest = GitRequest::new(common_dir, arguments.as_slice());
    let text: String = String::from_utf8_lossy(answered_as_text(git, &request).await?.as_slice()).trim().to_owned();
    // A count of nothing is every commit of the question being where a branch, a remote-tracking reference or a tag
    //  already holds it.
    return match text.parse::<u64>() {
        Ok(count) => Ok(count == 0),
        Err(_not_a_count) => Ok(false),
    };
}

/// What:
///  The bytes those objects weigh on disk,
///  as `%(objectsize:disk)` reports each of them.
/// Why:
///  The disk is what the bound is about:
///  the same objects cost 7,573 bytes on disk and 258,055 bytes before compression in the measurement above,
///  and the on-disk figure is what a repository gives up when the reference goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytesOf = (objects) => batchCheck({ stdin: objects.join('\n'), format: '%(objectsize:disk)' }).sum();
/// ```
async fn bytes_of(git: &GitContext, common_dir: &Path, objects: &[String]) -> Result<u64, String> {
    if objects.is_empty() {
        return Ok(0);
    }
    let mut input: String = String::new();
    for object in objects {
        input.push_str(object.as_str());
        input.push('\n');
    }
    let mut request: GitRequest = GitRequest::new(common_dir, &["cat-file", "--batch-check=%(objectsize:disk)"]);
    request.input = Some(input.into_bytes());
    let text: String = String::from_utf8_lossy(answered_as_text(git, &request).await?.as_slice()).into_owned();
    let mut total: u64 = 0;
    for line in text.lines() {
        // An object Git cannot read answers with `missing` in place of a size, which adds nothing.
        total = total.saturating_add(line.trim().parse::<u64>().unwrap_or(0));
    }
    return Ok(total);
}

/// What:
///  The bytes the objects reachable from `commits` and from no reference outside the kept folder weigh.
/// Why:
///   The size rule asks this again for the references that would survive,
///  so the pass weighs what it would leave behind and not what it started from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytesOnlyTheseReach = async (commits) => bytesOf(await onlyTheseReach(commits));
/// ```
pub async fn bytes_only_these_reach(git: &GitContext, common_dir: &Path, commits: &[String]) -> Result<u64, String> {
    let objects: Vec<String> = objects_only_these_reach(git, common_dir, commits).await?;
    return bytes_of(git, common_dir, objects.as_slice()).await;
}

/// What:
///  One reading of the repository's kept references.
/// Why:
///  The pass and the listing start from it,
///  and a kept commit's absence from the objects only kept references reach is what says a branch has reached it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = { kept: listed.map((one) => ({ ...one, reachedABranch: !unreached.includes(one.commit) })), bytesOnlyTheyReach };
/// ```
pub async fn read_the_kept_references(git: &GitContext, common_dir: &Path) -> Result<TheReading, String> {
    let mut kept: Vec<OneKeptReference> = listed_kept_references(git, common_dir).await?;
    if kept.is_empty() {
        return Ok(TheReading { kept, bytes_only_they_reach: 0 });
    }
    let commits: Vec<String> = commits_of(kept.as_slice());
    let unreached: HashSet<String> = objects_a_branch_does_not_reach(git, common_dir, commits.as_slice()).await?.into_iter().collect();
    for one in &mut kept {
        one.reached_a_branch = !unreached.contains(&one.commit);
    }
    let held: Vec<String> = objects_only_these_reach(git, common_dir, commits.as_slice()).await?;
    let bytes: u64 = bytes_of(git, common_dir, held.as_slice()).await?;
    return Ok(TheReading { kept, bytes_only_they_reach: bytes });
}
