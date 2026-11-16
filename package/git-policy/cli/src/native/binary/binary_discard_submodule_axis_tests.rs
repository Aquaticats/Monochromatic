//! What:
//!  The form that recurses into submodules, crossed with what a submodule's own worktree holds:
//!  an uncommitted tracked edit,
//!  nothing at all,
//!  and the same edit with the recursion switched off by the option or by `submodule.recurse`.
//! Why:
//!  A submodule's worktree holds a person's files as much as the outer one,
//!  and a recursive checkout, switch or reset moves each submodule to the commit the target tree
//!  records, replacing the work its own repository does not hold.
//!  `submodule.<name>.ignore` keeps that work out of every status the outer repository answers,
//!  so the outer reading cannot see it
//!  (`git checkout -f --recurse-submodules other` replaced a submodule edit,
//!  the ninth review's fifth finding).
//!  So each row is run twice, once by real Git and once by the wrapper,
//!  and the question is whether the person's edit inside the submodule is still there.
//!  A row where Git writes nothing is the row that says the guard must not refuse a clean submodule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const form of recursiveForms) expect(readFileSync('sub/f.txt')).toEqual(EDIT);
//! ```

use super::support::{Fixture, Observed, direct, fixture, git, observe, remove, repository, stderr_of, wrapped};
use std::path::{Path, PathBuf};

/// The text the person's edit inside the submodule holds.
const SUBMODULE_EDIT: &[u8] = b"PRECIOUS SUBMODULE EDIT\n";

/// Whether the wrapper itself rejected the command.
fn refused(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// The two commits of the submodule, and the superproject whose `main` and `other` record them.
///  A `struct` is a record with named fields.
struct Trees {
    /// The repository the submodule is read from.
    inner: PathBuf,
    /// The submodule's first commit.
    one: String,
    /// The submodule's second commit.
    two: String,
}

/// Make the two submodule commits once per fixture.
fn trees(fixture: &Fixture) -> Trees {
    let inner: PathBuf = fixture.root.join("inner-source");
    std::fs::create_dir(inner.as_path()).expect("the submodule's source");
    git(fixture, inner.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(inner.join("f.txt"), b"inner one\n").expect("the first inner file");
    git(fixture, inner.as_path(), &["add", "--", "f.txt"]);
    git(fixture, inner.as_path(), &["commit", "--quiet", "--message=one"]);
    let one: String = git(fixture, inner.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice().iter().map(|byte| return char::from(*byte)).collect::<String>().trim().to_string();
    std::fs::write(inner.join("f.txt"), b"inner two\n").expect("the second inner file");
    git(fixture, inner.as_path(), &["add", "--", "f.txt"]);
    git(fixture, inner.as_path(), &["commit", "--quiet", "--message=two"]);
    let two: String = git(fixture, inner.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice().iter().map(|byte| return char::from(*byte)).collect::<String>().trim().to_string();
    return Trees { inner, one, two };
}

/// What a row's fixture makes of the submodule.
///  An `enum` is a closed set of named alternatives.
#[derive(Clone, Copy)]
enum Inside {
    /// The person edited a tracked file and the mark is set to keep that out of the outer status.
    Edited,
    /// The submodule's worktree is as its commit records it.
    Clean,
}

/// One superproject with a linked worktree on `main`, and a submodule the rows reach.
fn prepared(fixture: &Fixture, name: &str, inside: Inside) -> PathBuf {
    let trees: Trees = trees(fixture);
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new(name));
    let linked: PathBuf = fixture.root.join(format!("{name}-linked"));
    std::fs::write(repo.join("a.txt"), b"outer base\n").expect("an outer file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=outer-base"]);
    let source: &str = trees.inner.to_str().expect("a fixture path is text");
    git(fixture, repo.as_path(), &["-c", "protocol.file.allow=always", "submodule", "add", "--quiet", source, "sub"]);
    let sub: PathBuf = repo.join("sub");
    let one: &str = trees.one.as_str();
    git(fixture, sub.as_path(), &["checkout", "--quiet", one]);
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=with-one"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    let two: &str = trees.two.as_str();
    git(fixture, sub.as_path(), &["checkout", "--quiet", two]);
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=with-two"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked_text: &str = linked.to_str().expect("a fixture path is text");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked_text]);
    git(fixture, linked.as_path(), &["-c", "protocol.file.allow=always", "submodule", "update", "--quiet", "--init"]);
    // The repository's own setting keeps the submodule's changes out of the outer status.
    git(fixture, repo.as_path(), &["config", "submodule.sub.ignore", "all"]);
    if matches!(inside, Inside::Edited) {
        std::fs::write(linked.join("sub/f.txt"), SUBMODULE_EDIT).expect("the person's edit");
    }
    return linked;
}

/// The person's edit if it is still there.
fn the_edit(directory: &Path) -> bool {
    return std::fs::read(directory.join("sub/f.txt")).is_ok_and(|bytes| return bytes == SUBMODULE_EDIT);
}

/// One arm of one row: what the command did, and whether the edit inside the submodule survived.
fn run(label: &str, inside: Inside, words: &[&str], configure: Option<(&str, &str)>, wrapped_arm: bool) -> (Option<i32>, bool, bool) {
    let fixture: Fixture = fixture(label);
    let directory: PathBuf = prepared(&fixture, "repo", inside);
    if let Some((key, value)) = configure {
        git(&fixture, directory.as_path(), &["config", key, value]);
    }
    let observed: Observed = if wrapped_arm {
        let mut command = wrapped(&fixture);
        command.current_dir(directory.as_path()).args(words);
        observe(&mut command, b"")
    } else {
        let mut command = direct(&fixture);
        command.current_dir(directory.as_path()).args(words);
        observe(&mut command, b"")
    };
    let survived: bool = the_edit(directory.as_path());
    let answer: (Option<i32>, bool, bool) = (observed.code, refused(&observed), survived);
    remove(&fixture);
    return answer;
}

/// What:
///  Every recursive form over a submodule holding an edit, over a clean one, and over the edit with the
///  recursion turned off: the person's edit inside the submodule is never gone,
///  and a clean submodule is never a refusal.
/// Why:
///  The edit is the whole observable:
///  a refused command leaves it, and a command Git runs that replaces it is the loss this reading exists
///  to prevent.
///  The arm with the recursion off is the control that says the wrapper still runs what Git runs,
///  so a guard that refused every form carrying the option would fail it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const form of recursiveForms) expect(run('edited', form).survived).toBe(true);
/// ```
#[test]
fn a_recursive_form_never_replaces_a_submodules_uncommitted_work() {
    let rows: &[(&str, &[&str], Option<(&str, &str)>)] = &[
        ("checkout -f --recurse-submodules other", &["checkout", "-f", "--recurse-submodules", "other"], None),
        ("switch -f --recurse-submodules other", &["switch", "-f", "--recurse-submodules", "other"], None),
        ("reset --hard --recurse-submodules other", &["reset", "--hard", "--recurse-submodules", "other"], None),
        ("checkout -f other with submodule.recurse", &["checkout", "-f", "other"], Some(("submodule.recurse", "true"))),
        ("checkout -f --recurse-submodules=no other", &["checkout", "-f", "--recurse-submodules=no", "other"], None),
        ("checkout -f --no-recurse-submodules other", &["checkout", "-f", "--no-recurse-submodules", "other"], None),
    ];
    let mut failures: Vec<String> = Vec::<String>::new();
    let mut refused_rows: usize = 0;
    for (name, words, configure) in rows {
        let edited: (Option<i32>, bool, bool) = run("submodule-axis-edited", Inside::Edited, words, *configure, true);
        let native_edited: (Option<i32>, bool, bool) = run("submodule-axis-native-edited", Inside::Edited, words, *configure, false);
        println!(
            "{name}: Git alone exited {:?} and left the edit {}; the wrapper {} and left it {}",
            native_edited.0,
            if native_edited.2 { "in place" } else { "gone" },
            if edited.1 { "refused" } else { "ran it" },
            if edited.2 { "in place" } else { "gone" }
        );
        if !edited.2 {
            failures.push(format!("{name}: the edit inside the submodule is gone"));
        }
        if edited.1 {
            refused_rows += 1;
        }
        // A clean submodule loses nothing, so a form Git runs there must not be refused.
        let clean: (Option<i32>, bool, bool) = run("submodule-axis-clean", Inside::Clean, words, *configure, true);
        let native_clean: (Option<i32>, bool, bool) = run("submodule-axis-native-clean", Inside::Clean, words, *configure, false);
        if clean.1 && native_clean.0 == Some(0) {
            failures.push(format!("{name}: a clean submodule was refused, and Git runs the form there"));
        }
    }
    println!("rows: {}; rows the wrapper refused: {refused_rows}; rows that lost the edit: {}", rows.len(), failures.len());
    assert!(failures.is_empty(), "{} rows:\n{}", rows.len(), failures.join("\n"));
}
