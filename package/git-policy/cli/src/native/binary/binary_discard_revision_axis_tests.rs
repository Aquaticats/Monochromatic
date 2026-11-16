//! What:
//!  Every spelling of a revision `gitrevisions(7)` documents,
//!  as the revision word of every guarded form that takes one,
//!  each row real Git beside the wrapper on the same fixture:
//!  a range with two dots and one with three,
//!  a path form `<rev>:<path>` and a tree form `<rev>:`,
//!  `:/text`,
//!  `@{…}` forms,
//!  `^{…}` peels,
//!  a name that is at once a tag, a branch and a remote branch,
//!  and `HEAD`.
//! Why:
//!  A revision is Git's own grammar, and a guard that reads it by a rule of its own meets a new spelling
//!  in each review. The eleventh review found `A...B`, `:/text` and `<rev>:<path>` refused although Git
//!  runs them and nothing is at risk, and its own count of this axis was that only 14 of 56 rows were rows
//!  where Git replaces the file: the other 42 proved nothing.
//!  So the rows are planted at a name the *resolved* tree writes and the branch tips do not, and the axis
//!  has a second half: the same spellings in a worktree holding nothing of the person's, which the wrapper
//!  must run rather than refuse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const spelling of documented) for (const form of forms) expect(wrappedKept(spelling, form)).toBe(true);
//! // and the same spellings with nothing planted: expect(refused).toBe(false);
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
///  One repository whose names are all the spellings need:
///  `main` and `work` share a base commit that `work` moves past by deleting `mb.txt`,
///  `other` moves past it by adding `new.txt`, `sub2/s2.txt` and changing `sub/s.txt`,
///  a tag `other` sits on `main`,
///  and a linked worktree is on `side`.
///  `&Fixture` borrows the disposable fixture.
/// Why:
///  A row proves something only when the tree the resolved word writes holds an entry no branch tip holds:
///  the merge base holds `mb.txt` and neither tip does,
///  `:/work` names `work`'s commit and writes `w.txt`,
///  `other:` names `other`'s tree and writes `new.txt`,
///  `other:sub2` names that subtree and writes `sub2/s2.txt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linked = prepared(fixture, 'repo');
/// ```
fn prepared(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    let linked: PathBuf = fixture.root.join("repo-linked");
    std::fs::create_dir_all(repo.join("sub")).expect("a tracked directory");
    std::fs::write(repo.join("a.txt"), b"a-main\n").expect("a tracked file");
    std::fs::write(repo.join("mb.txt"), b"mb-base\n").expect("the file the merge base keeps");
    std::fs::write(repo.join("sub/s.txt"), b"s-base\n").expect("a tracked file below a directory");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "mb.txt", "sub/s.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["branch", "work"]);
    // `work` removes `mb.txt`, so the merge base of `work` and `other` holds it and neither tip does.
    git(fixture, repo.as_path(), &["checkout", "--quiet", "work"]);
    std::fs::remove_file(repo.join("mb.txt")).expect("the file goes on work");
    std::fs::write(repo.join("w.txt"), b"w\n").expect("a file only work holds");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=work"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("new.txt"), b"n-other\n").expect("a file only the branch holds");
    std::fs::write(repo.join("sub/s.txt"), b"s-other\n").expect("the other tree's s");
    std::fs::create_dir_all(repo.join("sub2")).expect("a directory only the branch holds");
    std::fs::write(repo.join("sub2/s2.txt"), b"s2\n").expect("a file only the branch holds");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    git(fixture, repo.as_path(), &["tag", "other", "main"]);
    let linked_text: &str = linked.to_str().expect("a fixture path is text");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked_text]);
    return linked;
}

/// What:
///  The one file the person holds for one row, at a name the row's resolved tree writes and no branch tip
///  does, so a build that lets the command run loses it and a build that refuses keeps it.
///  `&str` is the revision word.
/// Why:
///  A row planted at a path a branch also tracks is a tracked edit, which the older wrapper refuses for
///  another reason; the eleventh review named that as why this axis could not see its first finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planted = (revision) => revision.includes('..') ? 'mb.txt' : revision === ':/work' ? 'w.txt' : revision.startsWith('other:sub2') ? 'sub2/s2.txt' : 'new.txt';
/// ```
fn planted(revision: &str) -> &'static str {
    if revision.contains("..") {
        return "mb.txt";
    }
    if revision == ":/work" {
        return "w.txt";
    }
    if revision.starts_with("other:sub2") {
        return "sub2/s2.txt";
    }
    return "new.txt";
}

/// Whether the person's file is still theirs.
fn kept(linked: &Path, name: &str) -> bool {
    return std::fs::read(linked.join(name)).is_ok_and(|bytes| return bytes == PERSON);
}

/// The person's untracked file, written where the row's tree writes it.
fn plant(linked: &Path, name: &str) {
    let path: PathBuf = linked.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("a directory on the way");
    }
    std::fs::write(path.as_path(), PERSON).expect("the person's untracked file");
}

/// What:
///  Every spelling `gitrevisions(7)` documents that a guarded form can take.
///  `&str` pairs are the row's name and the word.
/// Why:
///  The grammar is Git's, so the list is the documentation's, not a list of the cases a review found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spellings = [['three-dot range', 'work...other'], ['path form', 'other:new.txt'], ...];
/// ```
fn documented_spellings() -> Vec<(&'static str, &'static str)> {
    return vec![
        ("a branch", "other"),
        ("a name that is a tag, a branch and nothing else", "other"),
        ("a ref spelled with its prefix", "heads/other"),
        ("a ref spelled in full", "refs/heads/other"),
        ("an annotated peel to a commit", "other^{commit}"),
        ("an empty peel", "other^{}"),
        ("a three-dot range", "work...other"),
        ("a two-dot range", "work..other"),
        ("a commit named by its message", ":/work"),
        ("a tree form of a branch", "other:"),
        ("a subtree of a branch", "other:sub2"),
        ("a reflog entry", "other@{0}"),
        ("the upstream of a branch", "other@{upstream}"),
        ("HEAD", "HEAD"),
    ];
}

/// What:
///  Every guarded form that takes a revision word, with the words around it.
///  `&str` triples are the row's name, the words before the revision and the words after it.
/// Why:
///  Two of these forms move `HEAD` and two name a tree explicitly,
///  and the file they write is what says whether the person's entry was in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forms = [['checkout -f', ['checkout', '-f'], []], ['checkout with a path', ['checkout'], ['--', 'new.txt']], ...];
/// ```
fn forms() -> Vec<(&'static str, Vec<&'static str>, Vec<&'static str>)> {
    return vec![
        ("checkout -f", vec!["checkout", "-f"], vec![]),
        ("switch -f --detach", vec!["switch", "-f", "--detach"], vec![]),
        ("checkout with a path", vec!["checkout"], vec!["--", "new.txt"]),
        ("restore --source", vec!["restore", "--source"], vec!["--", "new.txt"]),
    ];
}

/// One row in one arm: what the command exited with, whether the wrapper refused, and whether the
/// person's file is still theirs.
fn run(label: &str, before: &[&'static str], revision: &str, after: &[&'static str], plant_it: bool, wrapped_arm: bool) -> (Option<i32>, bool, bool) {
    let fixture: Fixture = fixture(label);
    let linked: PathBuf = prepared(&fixture);
    let watched: &str = planted(revision);
    if plant_it {
        plant(linked.as_path(), watched);
    }
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
    let answer: (Option<i32>, bool, bool) = (observed.code, refused(&observed), kept(linked.as_path(), watched));
    remove(&fixture);
    return answer;
}

/// What:
///  Every documented revision spelling, against every guarded form that takes one:
///  no row leaves the person's file replaced, and every row where Git replaces it is one the wrapper
///  refused.
/// Why:
///  The grammar is Git's, so the reading has to be Git's too;
///  the plant is at a name only the resolved tree writes, so a build that lets the command run loses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [, revision] of spellings) for (const [, before, after] of forms)
///   expect(run(revision, before, after, true, true).kept).toBe(true);
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
            let wrapped_arm: (Option<i32>, bool, bool) = run("revision-axis-wrapped", before.as_slice(), revision, after.as_slice(), true, true);
            let native_arm: (Option<i32>, bool, bool) = run("revision-axis-native", before.as_slice(), revision, after.as_slice(), true, false);
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

/// What:
///  The same spellings in a worktree that holds nothing of the person's:
///  the wrapper runs every one of them, as real Git does.
/// Why:
///  A command Git runs with nothing at risk is not refused by the wrapper, and a guard that refuses it is a
///  guard people learn to switch off. This is the eleventh review's first finding, which its own count of
///  this axis (42 of 56 rows proving nothing) could not show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [, revision] of spellings) for (const [, before, after] of forms)
///   expect(run(revision, before, after, false, true).refused).toBe(false);
/// ```
#[test]
fn no_documented_revision_spelling_is_refused_where_git_runs_it() {
    let mut rows: usize = 0;
    let mut failures: Vec<String> = Vec::<String>::new();
    for (spelling, revision) in documented_spellings() {
        for (form, before, after) in forms() {
            let name: String = format!("{spelling} / {form} / `{revision}`");
            let native_arm: (Option<i32>, bool, bool) = run("revision-axis-clean-native", before.as_slice(), revision, after.as_slice(), false, false);
            let wrapped_arm: (Option<i32>, bool, bool) = run("revision-axis-clean-wrapped", before.as_slice(), revision, after.as_slice(), false, true);
            rows += 1;
            // Only a row Git itself runs says anything about a refusal of a safe command.
            if native_arm.0 == Some(0) && wrapped_arm.1 {
                failures.push(format!("{name}: Git ran the form (exit 0) and the wrapper refused it"));
            }
        }
    }
    println!("rows: {rows}; rows where Git ran the form and the wrapper refused it: {}", failures.len());
    assert!(failures.is_empty(), "{} rows:\n{}", rows, failures.join("\n"));
}

/// What:
///  One row of the eighth finding's axis: what the arm exited with, whether it was refused, whether the
///  person's file is still theirs, and the whole of its standard error.
/// Why:
///  The row this test asks about is one the wrapper must *not* refuse, so the sentence the person reads has
///  to be Git's own, and that is only visible in the standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneRow = (label, before, revision, wrappedArm) => ({ code, refused, kept, stderr });
/// ```
fn one_row(label: &str, before: &[&'static str], revision: &str, wrapped_arm: bool) -> (Option<i32>, bool, bool, String) {
    let fixture: Fixture = fixture(label);
    let linked: PathBuf = prepared(&fixture);
    let watched: &str = planted(revision);
    plant(linked.as_path(), watched);
    let mut command: std::process::Command = if wrapped_arm { wrapped(&fixture) } else { direct(&fixture) };
    command.current_dir(linked.as_path()).args(before).arg(revision);
    let observed: Observed = observe(&mut command, b"");
    let answer: (Option<i32>, bool, bool, String) = (
        observed.code,
        refused(&observed),
        kept(linked.as_path(), watched),
        stderr_of(&observed).to_owned(),
    );
    remove(&fixture);
    return answer;
}

/// What:
///  A word Git's own command refuses as no commit, in front of a form that writes a whole tree:
///  the wrapper runs the command, Git's refusal answers, and the person's file is kept.
/// Why:
///  The twelfth review's eighth finding: `checkout -f other^{tree}`, `switch -f --detach other^{tree}`,
///  `checkout -f other:` and the same words through `reset --hard` were refused for the file the tree
///  holds, while real Git refuses the word before it writes anything. The guard's sentence then described
///  a loss that could not happen, and the person had to spend the opt-out to see Git's own answer.
///  The branch itself is the positive control: with the same plant, the wrapper must refuse that row and
///  keep the file, so a row that runs proves the wrapping, not a fixture that lets everything through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const revision of ['other^{tree}', 'other:']) for (const form of commitForms)
///   expect(nativeExit(revision)).not.toBe(0);
///   expect(wrappedRefused(revision)).toBe(false);
///   expect(wrappedKept(revision)).toBe(true);
/// for (const form of commitForms) expect(wrappedRefused('other')).toBe(true);
/// ```
#[test]
fn a_word_git_refuses_as_no_commit_is_left_to_git() {
    let commits: Vec<(&'static str, Vec<&'static str>)> = vec![
        ("checkout -f", vec!["checkout", "-f"]),
        ("switch -f --detach", vec!["switch", "-f", "--detach"]),
        ("reset --hard", vec!["reset", "--hard"]),
    ];
    let mut rows: usize = 0;
    let mut not_refused: usize = 0;
    let mut failures: Vec<String> = Vec::<String>::new();
    for revision in ["other^{tree}", "other:"] {
        for (form, before) in &commits {
            let name: String = format!("{form} / `{revision}`");
            let native_arm: (Option<i32>, bool, bool, String) = one_row("no-commit-native", before.as_slice(), revision, false);
            let wrapped_arm: (Option<i32>, bool, bool, String) = one_row("no-commit-wrapped", before.as_slice(), revision, true);
            rows += 1;
            if native_arm.0 == Some(0) {
                failures.push(format!("{name}: Git ran the form itself, so this row proves nothing"));
            }
            if wrapped_arm.1 {
                failures.push(format!("{name}: the wrapper refused a word Git refuses itself: {}", wrapped_arm.3));
            }
            if !wrapped_arm.2 {
                failures.push(format!("{name}: the person's file is gone"));
            }
            if !wrapped_arm.1 {
                not_refused += 1;
            }
            if !wrapped_arm.3.contains("non-commit") && !wrapped_arm.3.contains("not a commit") {
                failures.push(format!("{name}: Git's own answer did not reach the person: {}", wrapped_arm.3));
            }
        }
    }
    // The positive control: the branch's own tree holds the planted file, so the wrapper refuses and keeps it.
    //  The full ref is the word here, because `other` is also an annotated tag on `main` in this fixture and a
    //  `reset` takes the tag first, writing a tree that holds nothing of the person's.
    for (form, before) in &commits {
        let name: String = format!("control: {form} / `refs/heads/other`");
        let native_arm: (Option<i32>, bool, bool, String) = one_row("no-commit-control-native", before.as_slice(), "refs/heads/other", false);
        let wrapped_arm: (Option<i32>, bool, bool, String) = one_row("no-commit-control", before.as_slice(), "refs/heads/other", true);
        rows += 1;
        if native_arm.2 {
            failures.push(format!("{name}: Git kept the planted file, so the fixture cannot show a refusal: {native_arm:?}"));
        }
        if !wrapped_arm.1 {
            failures.push(format!("{name}: the wrapper did not refuse the branch's own tree: {wrapped_arm:?}"));
        }
        if !wrapped_arm.2 {
            failures.push(format!("{name}: the control's planted file is gone"));
        }
    }
    println!(
        "rows: {rows}; rows the wrapper did not refuse: {not_refused}; rows that failed: {}",
        failures.len()
    );
    assert!(failures.is_empty(), "{} rows:\n{}", rows, failures.join("\n"));
}
