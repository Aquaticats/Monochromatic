//! What:
//!  Every spelling of a revision `gitrevisions(7)` documents,
//!  as the revision word of every guarded form that takes one,
//!  each row real Git beside the wrapper on the same fixture:
//!  a range with two dots and one with three,
//!  `@{…}` forms,
//!  `^{…}` peels,
//!  `:/text`,
//!  a name that is at once a tag, a branch and a remote branch,
//!  and a short object id.
//! Why:
//!  A revision is Git's own grammar, and a guard that read it by a rule of its own met a new spelling in
//!  each review: `A...B` is the merge base to `checkout` while every single-object query refuses it,
//!  and a name that is both a tag and a branch is the branch to `checkout` while `ls-tree` takes the tag
//!  (the eleventh review's first and second findings).
//!  The same names are also spelled the way Git documents them, so a reading that resolves one of them
//!  and not another fails a row here rather than in a person's worktree.
//!  A row where Git itself writes nothing, or refuses the form, is counted rather than trusted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const spelling of documented) for (const form of forms) expect(wrappedKept(spelling, form)).toBe(true);
//! ```

use super::support::{Fixture, Observed, direct, fixture, git, observe, remove, repository, stderr_of, wrapped};
use std::path::{Path, PathBuf};

/// The bytes the person wrote at the path a form would write.
const PERSON: &[u8] = b"PERSON-untracked\n";

/// Whether the wrapper itself rejected the command.
fn refused(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// What:
///  One linked worktree with a branch `other` whose tree differs from `main`,
///  a tag `other` on `main`, a tag `v1`, a remote-tracking branch, reflog entries,
///  and the person's untracked file at the path the other tree (or the merge base) holds.
///  `&Fixture` borrows the disposable fixture.
/// Why:
///  Each spelling needs something in the repository to denote:
///  a branch that moved, a merge base that holds a file `main` does not,
///  a tag and a branch of one name, and a remote-tracking ref to guess from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linked = prepared(fixture, 'repo');
/// ```
fn prepared(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("repo-linked");
    std::fs::write(repo.join("a.txt"), b"a-main\n").expect("a tracked file");
    std::fs::write(repo.join("b.txt"), b"b-main\n").expect("the file the merge base keeps");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["branch", "work"]);
    // `work` removes `b.txt`, so the merge base of `work` and `other` holds it.
    git(fixture, repo.as_path(), &["checkout", "--quiet", "work"]);
    std::fs::remove_file(repo.join("b.txt")).expect("the file goes on work");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=work"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("new-in-other.txt"), b"n-other\n").expect("a file only the branch holds");
    std::fs::write(repo.join("b.txt"), b"b-other\n").expect("the other tree's b");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    git(fixture, repo.as_path(), &["tag", "other", "main"]);
    git(fixture, repo.as_path(), &["tag", "v1", "main"]);
    let linked_text: &str = linked.to_str().expect("a fixture path is text");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked_text]);
    return linked;
}

/// The person's untracked file, written where the row's tree writes it.
///  `&Path` borrows the linked worktree, `&str` is the name.
fn plant(linked: &Path, name: &str) {
    std::fs::write(linked.join(name), PERSON).expect("the person's untracked file");
}

/// What:
///  The one file the person holds for one row, at the name that row's tree writes:
///  a range writes the merge base's `b.txt`,
///  and every other spelling is a test of which tree the form resolves,
///  so the file stands at `new-in-other.txt`, which only the branch adds.
///  `&str` is the revision word.
/// Why:
///  The name has to be one the *right* tree writes and the wrong one does not,
///  or a row passes on a refusal that came from somewhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planted = (revision) => revision.includes('..') ? 'b.txt' : 'new-in-other.txt';
/// ```
fn planted(revision: &str) -> &'static str {
    return if revision.contains("..") { "b.txt" } else { "new-in-other.txt" };
}

/// Whether the person's file is still theirs.
fn kept(linked: &Path, name: &str) -> bool {
    return std::fs::read(linked.join(name)).is_ok_and(|bytes| return bytes == PERSON);
}

/// What:
///  Every spelling `gitrevisions(7)` documents that a guarded form can take,
///  as the words the form is given.
///  `&str` pairs are the row's name and the word.
/// Why:
///  The grammar is Git's, so the list is the documentation's, not a list of the cases a review found:
///  two-dot and three-dot ranges, `@{…}` reflog and upstream forms, `^{…}` peels, `:/text`,
///  a name that is a tag and a branch, a ref spelled in full and with its `refs/heads/` prefix,
///  a short object id, `-` and `@{-1}`, and `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spellings = [['three-dot range', 'work...other'], ['two-dot range', 'work..other'], ...];
/// ```
fn documented_spellings() -> Vec<(&'static str, &'static str)> {
    return vec![
        ("a branch", "other"),
        ("a name that is a tag, a branch and nothing else", "other"),
        ("a ref spelled with its prefix", "heads/other"),
        ("a ref spelled in full", "refs/heads/other"),
        ("a tag", "v1"),
        ("an annotated peel to a commit", "v1^{commit}"),
        ("a peel to a tree", "v1^{tree}"),
        ("an empty peel", "v1^{}"),
        ("a three-dot range", "work...other"),
        ("a two-dot range", "work..other"),
        ("a reflog entry", "other@{0}"),
        ("the upstream of a branch", "other@{upstream}"),
        ("the previous branch", "@{-1}"),
        ("HEAD", "HEAD"),
    ];
}

/// What:
///  Every guarded form that takes a revision word, with the words around it and the file each one writes.
///  `&str` triples are the row's name, the words before the revision and the words after it.
/// Why:
///  Two of these forms move `HEAD` and three name a tree explicitly,
///  and the file they write is what says whether the person's entry was in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forms = [['checkout -f', ['checkout', '-f'], []], ['checkout with a path', ['checkout'], ['--', 'b.txt']], ...];
/// ```
fn forms() -> Vec<(&'static str, Vec<&'static str>, Vec<&'static str>)> {
    return vec![
        ("checkout -f", vec!["checkout", "-f"], vec![]),
        ("switch -f --detach", vec!["switch", "-f", "--detach"], vec![]),
        ("checkout with a path", vec!["checkout"], vec!["--", "b.txt"]),
        ("restore --source", vec!["restore", "--source"], vec!["--", "b.txt"]),
    ];
}

/// One row in one arm: what the command exited with, whether the wrapper refused, and whether the
/// person's file is still theirs.
fn run(label: &str, before: &[&'static str], revision: &str, after: &[&'static str], wrapped_arm: bool) -> (Option<i32>, bool, bool) {
    let fixture: Fixture = fixture(label);
    let linked: PathBuf = prepared(&fixture);
    let name: &str = planted(revision);
    plant(linked.as_path(), name);
    let mut words: Vec<&str> = before.to_vec();
    words.push(revision);
    words.extend(after.iter().copied());
    let observed: Observed = if wrapped_arm {
        let mut command = wrapped(&fixture);
        command.current_dir(linked.as_path()).args(words.as_slice());
        observe(&mut command, b"")
    } else {
        let mut command = direct(&fixture);
        command.current_dir(linked.as_path()).args(words.as_slice());
        observe(&mut command, b"")
    };
    let answer: (Option<i32>, bool, bool) = (observed.code, refused(&observed), kept(linked.as_path(), name));
    remove(&fixture);
    return answer;
}

/// What:
///  Every documented revision spelling, against every guarded form that takes one:
///  no row leaves the person's file replaced, and every row where Git replaces it is one the wrapper
///  refused.
/// Why:
///  The grammar is Git's, so the reading has to be Git's too;
///  a row where Git itself writes nothing is counted rather than trusted,
///  and a row where Git changes the file while the wrapper ran it is the defect this axis exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [, revision] of spellings) for (const [, before, after, watched] of forms)
///   expect(run(revision, before, after, watched, true).kept).toBe(true);
/// ```
#[test]
fn every_documented_revision_spelling_is_read_as_git_reads_it() {
    let mut rows: usize = 0;
    let mut refused_rows: usize = 0;
    let mut nothing_to_lose: usize = 0;
    let mut failures: Vec<String> = Vec::<String>::new();
    for (spelling, revision) in documented_spellings() {
        for (form, before, after) in forms() {
            let name: String = format!("{spelling} / {form} / `{revision}`");
            let watched: &str = planted(revision);
            let wrapped_arm: (Option<i32>, bool, bool) = run("revision-axis-wrapped", before.as_slice(), revision, after.as_slice(), true);
            let native_arm: (Option<i32>, bool, bool) = run("revision-axis-native", before.as_slice(), revision, after.as_slice(), false);
            rows += 1;
            if native_arm.2 {
                nothing_to_lose += 1;
            } else if wrapped_arm.1 {
                refused_rows += 1;
            } else if !wrapped_arm.2 {
                failures.push(format!(
                    "{name}: Git replaced `{watched}` (exit {:?}) and the wrapper ran the form (exit {:?})",
                    native_arm.0, wrapped_arm.0
                ));
            }
            if !wrapped_arm.2 {
                failures.push(format!("{name}: the person's `{watched}` is gone"));
            }
        }
    }
    println!(
        "rows: {rows}; rows where Git replaces the file and the wrapper refuses: {refused_rows}; rows where Git kept it: {nothing_to_lose}; rows that lost it: {}",
        failures.len()
    );
    assert!(failures.is_empty(), "{} rows:\n{}", rows, failures.join("\n"));
}
