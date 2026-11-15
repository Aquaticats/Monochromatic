//! What:
//!  The walk over every spelling of a path a guarded form writes:
//!  each row a command line run against real Git 2.56.0 in a disposable repository,
//!  held against the places the guard says that command writes.
//! Why:
//!  The guard compares the files a form writes with the entries Git did not put there,
//!  and it learns those files from the form's own words.
//!  A word it places differently from Git is a file nobody looked at:
//!  `git mv -f /abs/worktree/a.txt /abs/worktree/b.txt` writes `b.txt`, while an absolute path was
//!  answered "nothing",
//!  and `git checkout-index -f --prefix=/abs/outside/ -a` writes a person's file outside the
//!  worktree, which the guard dropped from its answer.
//!  So every spelling Git accepts is run, and the question is the one a loss turns on:
//!  did Git change or delete a file holding a person's data, and did the guard name that file's
//!  place?
//!  A file the index holds is no part of this question: its content is in the repository, and an
//!  edit to it is the dirty question's business.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for each row: guardPlaces(row) covers every USER DATA file real Git changed or deleted
//! ```

/// The reading under test, the forms it answers for, and the fixture helpers.
use crate::command_test_support::{fixture, git, git_status, remove};
use crate::command_written_paths::written_paths;
use crate::repo_read::repository_facts::{GitFacts, git_facts};
/// The trait whose question this walk asks the provider.
use crate::repository_facts::RepositoryFacts;
use crate::rule_linked_worktree::GuardedCommand;
use crate::test_support::{REAL_GIT, isolated_overlay};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The text every file a person holds begins with, so a control finds a loss by its content and not
/// by a name it would have to keep in step with the fixture.
const USER_DATA: &str = "USER DATA";

/// The absolute paths one row's fixture gives the row.
struct Places {
    /// The repository's worktree root.
    repo: PathBuf,
    /// A directory outside the worktree, holding files a person's data stands in.
    outside: PathBuf,
}

/// One row: the subcommand word, the words after it, and the directory Git runs it in.
struct Row {
    /// What the row shows, spelled without any fixture's own paths.
    label: String,
    /// The subcommand word, as Git receives it.
    word: &'static str,
    /// The guarded form the written-path reading answers for.
    command: GuardedCommand,
    /// The words after the subcommand, exactly as Git receives them.
    words: Vec<String>,
    /// The directory the command runs in.
    directory: PathBuf,
}

/// Every entry the guard would refuse one row over, as it names them: the reading the refusal's
/// sentence lists.
/// Every file below `root`, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if entry.file_name() == ".git" {
                continue;
            }
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(kind) if kind.is_file() => {
                    let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
                    found.insert(path, bytes);
                }
                _ => continue,
            }
        }
    }
    return found;
}

/// What one row wrote: every file a person's data stood in before and whose bytes changed, or which
/// went away.
fn files_a_person_lost(before: &BTreeMap<PathBuf, Vec<u8>>, after: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<PathBuf> {
    let mut lost: Vec<PathBuf> = Vec::<PathBuf>::new();
    for (path, bytes) in before {
        if !String::from_utf8_lossy(bytes.as_slice()).starts_with(USER_DATA) {
            continue;
        }
        if after.get(path) != Some(bytes) {
            lost.push(path.clone());
        }
    }
    return lost;
}

/// Whether one lost path is one of the entries the guard named.
/// `&Path` is the worktree root an inside name is relative to.
fn named(lost: &Path, names: &[String], root: &Path) -> bool {
    let shown: String = match lost.strip_prefix(root) {
        Ok(relative) => String::from_utf8_lossy(relative.as_os_str().as_encoded_bytes()).into_owned(),
        Err(_) => lost.display().to_string(),
    };
    return names.iter().any(|name| return name == &shown);
}

/// What the guard would refuse one row over: the entries its question names, or why it cannot tell.
fn guard_names(directory: &Path, command: GuardedCommand, words: &[String]) -> Result<Vec<String>, String> {
    let region: Vec<OsString> = words.iter().map(OsString::from).collect();
    let Some(written) = written_paths(command, region.as_slice()) else {
        return Ok(Vec::<String>::new());
    };
    let prefix: Vec<OsString> = vec![OsString::from("-C"), directory.as_os_str().to_os_string()];
    let mut facts: GitFacts = git_facts(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice());
    return facts.paths_in_the_way(&written);
}

/// One repository, one branch whose tree differs, and the files a person holds inside and outside it.
fn fixture_for(index: usize) -> Places {
    let directory: PathBuf = fixture(format!("written-place-{index:02}").as_str());
    let repo: PathBuf = directory.join("repo");
    let outside: PathBuf = directory.join("outside");
    std::fs::create_dir_all(repo.as_path()).expect("the repository");
    std::fs::create_dir_all(outside.as_path()).expect("a directory outside the worktree");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), b"committed a\n").expect("a tracked file");
    std::fs::create_dir_all(repo.join("sub")).expect("a tracked directory");
    std::fs::write(repo.join("sub/deep.txt"), b"committed deep\n").expect("a tracked file below the root");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    git(repo.as_path(), &["add", "--", "a.txt", "sub/deep.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=main"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("a.txt"), b"other a\n").expect("the other tree's a");
    std::fs::write(repo.join("b.txt"), b"other b\n").expect("the other tree's b");
    std::fs::write(repo.join("sub/deep.txt"), b"other deep\n").expect("the other tree's deep file");
    std::fs::write(repo.join("sub/other.txt"), b"other only\n").expect("a file only the other tree holds");
    git(repo.as_path(), &["add", "--", "a.txt", "b.txt", "sub/deep.txt", "sub/other.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    // The files a person holds: untracked, ignored, below a directory a prefix names, and outside.
    for relative in [
        "b.txt",
        "sub/precious.txt",
        "ignored/precious.txt",
        "out/a.txt",
        "outa.txt",
        "outsub/deep.txt",
        "out-in/a.txt",
    ] {
        let path: PathBuf = repo.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("a directory on the way");
        }
        std::fs::write(path.as_path(), format!("{USER_DATA} {relative}\n")).expect("a file a person holds");
    }
    for relative in ["a.txt", "b.txt", "sub/deep.txt"] {
        let path: PathBuf = outside.join(relative);
        std::fs::create_dir_all(path.parent().expect("a directory")).expect("a directory outside");
        std::fs::write(path.as_path(), format!("{USER_DATA} outside {relative}\n")).expect("a file outside the worktree");
    }
    // One link leaves the worktree and one stays inside it.
    std::os::unix::fs::symlink(outside.as_path(), repo.join("link-out")).expect("a link outside");
    std::os::unix::fs::symlink("sub", repo.join("link-in")).expect("a link inside");
    return Places { repo, outside };
}

/// One row, named so that no row's label carries a fixture's own paths.
fn row(label: String, word: &'static str, command: GuardedCommand, words: Vec<String>, directory: &Path) -> Row {
    return Row { label, word, command, words, directory: directory.to_path_buf() };
}

/// Every row this walk runs, built for one fixture's absolute paths.
fn row_specs(places: &Places) -> Vec<Row> {
    let repo: &Path = places.repo.as_path();
    let outside: String = places.outside.display().to_string();
    let repo_text: String = repo.display().to_string();
    let mut rows: Vec<Row> = Vec::<Row>::new();
    // One word's spellings: relative, through a dot and a parent component, and absolute inside.
    let inside_spellings: Vec<(&str, String)> = vec![
        ("relative", String::from("b.txt")),
        ("dot-relative", String::from("./b.txt")),
        ("parent-relative", format!("sub/../b.txt")),
        ("absolute-inside", format!("{repo_text}/b.txt")),
    ];
    for (kind, spelling) in &inside_spellings {
        rows.push(row(
            format!("checkout -f other -- <{kind}> b.txt"),
            "checkout",
            GuardedCommand::CheckoutPaths,
            vec![String::from("-f"), String::from("other"), String::from("--"), spelling.clone()],
            repo,
        ));
        rows.push(row(
            format!("restore --source=other -- <{kind}> b.txt"),
            "restore",
            GuardedCommand::RestoreWorktree,
            vec![String::from("--source=other"), String::from("--"), spelling.clone()],
            repo,
        ));
    }
    rows.push(row(
        String::from("reset --hard other"),
        "reset",
        GuardedCommand::ResetHard,
        vec![String::from("--hard"), String::from("other")],
        repo,
    ));
    rows.push(row(
        String::from("reset --hard other -- b.txt"),
        "reset",
        GuardedCommand::ResetHard,
        vec![String::from("--hard"), String::from("other"), String::from("--"), String::from("b.txt")],
        repo,
    ));
    rows.push(row(String::from("switch -f other"), "switch", GuardedCommand::SwitchForce, vec![String::from("-f"), String::from("other")], repo));
    rows.push(row(
        String::from("read-tree --reset -u other"),
        "read-tree",
        GuardedCommand::ReadTree,
        vec![String::from("--reset"), String::from("-u"), String::from("other")],
        repo,
    ));
    let prefixes: Vec<(&str, String)> = vec![
        ("relative with a separator", String::from("out/")),
        ("relative without one", String::from("out")),
        ("dot-relative", String::from("./out/")),
        ("absolute inside", format!("{repo_text}/out-in/")),
        ("absolute outside", format!("{outside}/")),
        ("absolute outside below a directory", format!("{outside}/sub/")),
        ("parent-relative outside", String::from("../outside/")),
        ("through a link that leaves", String::from("link-out/")),
        ("through a link that stays", String::from("link-in/")),
    ];
    for (kind, prefix) in &prefixes {
        rows.push(row(
            format!("read-tree --reset -u --prefix=<{kind}> other"),
            "read-tree",
            GuardedCommand::ReadTree,
            vec![String::from("--reset"), String::from("-u"), format!("--prefix={prefix}"), String::from("other")],
            repo,
        ));
        rows.push(row(
            format!("checkout-index -f --prefix=<{kind}> -a"),
            "checkout-index",
            GuardedCommand::CheckoutIndex,
            vec![String::from("-f"), format!("--prefix={prefix}"), String::from("-a")],
            repo,
        ));
    }
    // A forced move: the destination is the file Git writes, in every spelling it accepts.
    let destinations: Vec<(&str, String)> = vec![
        ("relative", String::from("b.txt")),
        ("dot-relative", String::from("./b.txt")),
        ("parent-relative", String::from("sub/../b.txt")),
        ("absolute inside", format!("{repo_text}/b.txt")),
        ("absolute outside", format!("{outside}/b.txt")),
        ("absolute outside where nothing is", format!("{outside}/new.txt")),
        ("parent-relative outside", String::from("../outside/b.txt")),
        ("parent-relative outside where nothing is", String::from("../outside/new.txt")),
        ("through a link that leaves", String::from("link-out/b.txt")),
        ("through a link that leaves where nothing is", String::from("link-out/new.txt")),
        ("through a link that stays", String::from("link-in/precious.txt")),
    ];
    for (kind, destination) in &destinations {
        rows.push(row(
            format!("mv -f <relative a.txt> <{kind}> destination"),
            "mv",
            GuardedCommand::Move,
            vec![String::from("-f"), String::from("a.txt"), destination.clone()],
            repo,
        ));
        rows.push(row(
            format!("mv -f <absolute-inside a.txt> <{kind}> destination"),
            "mv",
            GuardedCommand::Move,
            vec![String::from("-f"), format!("{repo_text}/a.txt"), destination.clone()],
            repo,
        ));
    }
    for spelling in ["a.txt", "./a.txt", "sub/../a.txt", "sub/deep.txt"] {
        rows.push(row(
            format!("checkout-index -f -- <{spelling}>"),
            "checkout-index",
            GuardedCommand::CheckoutIndex,
            vec![String::from("-f"), String::from("--"), String::from(spelling)],
            repo,
        ));
    }
    rows.push(row(
        String::from("checkout-index -f -a"),
        "checkout-index",
        GuardedCommand::CheckoutIndex,
        vec![String::from("-f"), String::from("-a")],
        repo,
    ));
    // The same words from a directory below the root, where a relative word means another place.
    for (kind, spelling) in [("relative below", String::from("b.txt")), ("parent below", String::from("../b.txt")), ("dot below", String::from("./b.txt"))] {
        rows.push(row(
            format!("checkout -f other -- <{kind}>"),
            "checkout",
            GuardedCommand::CheckoutPaths,
            vec![String::from("-f"), String::from("other"), String::from("--"), spelling],
            repo.join("sub").as_path(),
        ));
    }
    return rows;
}

/// What:
///  Every row of the walk: the files a person held that real Git changed or deleted are files the
///  guard named the place of.
/// Why:
///  This is the loss the guard exists to prevent, stated as one question per row and answered by
///  running Git rather than by reading its source.
///  A row where the guard cannot place a word at all stops the command in production, which is safe;
///  such a row is counted rather than failed, and named in the summary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const row of rows) {
///   const places = guardPlaces(row);
///   runGit(row);
///   expect(filesAPersonLost(row)).every((path) => named(path, places));
/// }
/// ```
#[test]
fn every_row_names_the_place_of_a_file_git_writes() {
    // One fixture answers how many rows there are and what each is called; every row then gets its
    // own, because a row that replaced a file would answer the next one.
    let probe: Places = fixture_for(0);
    let labels: Vec<String> = row_specs(&probe).into_iter().map(|row| return row.label).collect();
    remove(probe.repo.parent().expect("the fixture's directory"));
    let mut refused: Vec<String> = Vec::<String>::new();
    let mut failures: Vec<String> = Vec::<String>::new();
    for (number, label) in labels.iter().enumerate() {
        let places: Places = fixture_for(number + 1);
        let row: Row = row_specs(&places).into_iter().nth(number).unwrap_or_else(|| panic!("row {number} is missing"));
        assert_eq!(&row.label, label, "the rows shift between fixtures");
        let guarded: Result<Vec<String>, String> = guard_names(row.directory.as_path(), row.command, row.words.as_slice());
        let mut before: BTreeMap<PathBuf, Vec<u8>> = snapshot(places.repo.as_path());
        before.extend(snapshot(places.outside.as_path()));
        let mut arguments: Vec<&str> = vec![row.word];
        arguments.extend(row.words.iter().map(String::as_str));
        let observed: std::process::Output = git_status(row.directory.as_path(), arguments.as_slice());
        let mut after: BTreeMap<PathBuf, Vec<u8>> = snapshot(places.repo.as_path());
        after.extend(snapshot(places.outside.as_path()));
        let lost: Vec<PathBuf> = files_a_person_lost(&before, &after);
        match guarded {
            // A word the guard cannot place stops the command in production, so the file Git alone would have
            //  written is not lost; the row is counted and named with what Git would have written.
            Err(cause) => {
                refused.push(format!("{}: {cause}; Git alone would have written {lost:?}", row.label));
            }
            Ok(names) => {
                for path in &lost {
                    if !named(path.as_path(), names.as_slice(), places.repo.as_path()) {
                        failures.push(format!(
                            "{}: Git wrote {} and the guard named {names:?} (git exited {:?})",
                            row.label,
                            path.display(),
                            observed.status.code()
                        ));
                    }
                }
            }
        }
        remove(places.repo.parent().expect("the fixture's directory"));
    }
    println!(
        "rows: {}; refused because a word could not be placed: {}; rows that lost a file the guard did not name: {}",
        labels.len(),
        refused.len(),
        failures.len()
    );
    for line in &refused {
        println!("refused: {line}");
    }
    assert!(failures.is_empty(), "{} rows:\n{}", labels.len(), failures.join("\n"));
}
