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
//!  did Git change or delete an entry a person holds, and did the guard name that entry's place?
//!  The entry is the one at the name: a file the person wrote, and a link the person made,
//!  because Git replaces the entry at a written name rather than writing through it
//!  (measured on this host with native Git 2.56.0, 2026-11-16: a link at a tracked `a.txt` is
//!  replaced by `checkout -f`, `restore`, `reset --hard`, `read-tree --reset -u` and
//!  `checkout-index -f`, and a link at a forced move's destination is replaced by the rename).
//!  A file the index holds is no part of this question: its content is in the repository, and an
//!  edit to it is the dirty question's business.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for each row: guardPlaces(row) covers every USER DATA file and every link real Git changed
//! ```

/// The reading under test, the forms it answers for, and the fixture helpers.
use crate::command_test_support::{fixture, git, git_status, remove};
use crate::command_checkout_index_table::CHECKOUT_INDEX_TABLE;
use crate::command_discard::READ_TREE_TABLE;
use crate::command_options::{Arity, OptionSpec};
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
    /// The fixture directory holding the worktree and everything beside it.
    root: PathBuf,
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
/// One entry of a fixture as a person holds it:
///  a file with the bytes they wrote, or the link they made.
/// Why:
///  A fixture's links are entries a person made,
///  and Git replaces a link at a written name instead of writing through it,
///  so a walk that recorded only regular files would call a replaced link no loss.
enum Held {
    /// A file and its bytes.
    File(Vec<u8>),
    /// A symbolic link and what it points at.
    Link(PathBuf),
}

/// Every entry below `root` that a person holds, with what it is.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Held> {
    let mut found: BTreeMap<PathBuf, Held> = BTreeMap::new();
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
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if kind.is_symlink() {
                if let Ok(target) = std::fs::read_link(path.as_path()) {
                    found.insert(path, Held::Link(target));
                }
                continue;
            }
            if kind.is_file() {
                let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
                found.insert(path, Held::File(bytes));
            }
        }
    }
    return found;
}

/// What one row wrote: every entry a person held before and that is no longer the entry they held,
/// whether its bytes changed, the link was replaced, or it went away.
/// Why:
///  The question is the entry at the name, because that is what Git writes over:
///  a link replaced by a regular file is the person's link gone,
///  even though no byte of the file it pointed at moved.
fn files_a_person_lost(before: &BTreeMap<PathBuf, Held>, after: &BTreeMap<PathBuf, Held>) -> Vec<PathBuf> {
    let mut lost: Vec<PathBuf> = Vec::<PathBuf>::new();
    for (path, held) in before {
        let theirs: bool = match held {
            // A fixture's links are all the person's own, and so is any file they wrote.
            Held::Link(_) => true,
            Held::File(bytes) => String::from_utf8_lossy(bytes.as_slice()).starts_with(USER_DATA),
        };
        if !theirs {
            continue;
        }
        let same: bool = match (held, after.get(path)) {
            (Held::File(bytes), Some(Held::File(now))) => bytes == now,
            (Held::Link(target), Some(Held::Link(now))) => target == now,
            _ => false,
        };
        if !same {
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
    std::fs::write(repo.join("target.txt"), b"committed target\n").expect("a tracked file a link reaches");
    std::fs::create_dir_all(repo.join("sub")).expect("a tracked directory");
    std::fs::write(repo.join("sub/deep.txt"), b"committed deep\n").expect("a tracked file below the root");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    git(repo.as_path(), &["add", "--", "a.txt", "target.txt", "sub/deep.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=main"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("a.txt"), b"other a\n").expect("the other tree's a");
    std::fs::write(repo.join("b.txt"), b"other b\n").expect("the other tree's b");
    std::fs::write(repo.join("sub/deep.txt"), b"other deep\n").expect("the other tree's deep file");
    std::fs::write(repo.join("sub/other.txt"), b"other only\n").expect("a file only the other tree holds");
    git(repo.as_path(), &["add", "--", "a.txt", "b.txt", "target.txt", "sub/deep.txt", "sub/other.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    // The files a person holds: untracked, ignored, below a directory a prefix names, and outside.
    for relative in [
        "b.txt",
        "sub/precious.txt",
        "ignored/precious.txt",
        // The files a `read-tree --prefix=<ignored directory>` writes over, which Git writes although its
        //  ignore rules cover them.
        "ignored/a.txt",
        "ignored/sub/deep.txt",
        "out/a.txt",
        "outa.txt",
        "outsub/deep.txt",
        "out-in/a.txt",
        // The names a prefix spelled without a separator glues an index path to: `--prefix=out` and
        //  `a.txt` write `outa.txt`, `--prefix=.` and `a.txt` write `.a.txt`, and `--prefix=..` writes
        //  `..a.txt` (measured on this host with native Git 2.56.0, 2026-11-16).
        ".a.txt",
        "..a.txt",
        "linka.txt",
        "dot/a.txt",
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
    // A link whose name a prefix continues: `--prefix=link` and `a.txt` write `linka.txt`, and the link
    //  itself is not touched.
    std::os::unix::fs::symlink("sub", repo.join("link")).expect("a link a prefix continues");
    // The links a person made at a name a form writes: the destination of a forced move,
    //  each pointing at a file the index holds, a file it does not, nothing at all, and a place
    //  outside the worktree,
    //  and one at a path a prefix glues an index path to.
    std::fs::write(repo.join("untracked-target.txt"), format!("{USER_DATA} untracked target\n")).expect("a file a person holds");
    std::os::unix::fs::symlink("target.txt", repo.join("dest-tracked.txt")).expect("a link to a tracked file");
    std::os::unix::fs::symlink("untracked-target.txt", repo.join("dest-untracked.txt")).expect("a link to an untracked file");
    std::os::unix::fs::symlink("nothing-at-all.txt", repo.join("dest-dangling.txt")).expect("a link to nothing");
    std::os::unix::fs::symlink("../outside/a.txt", repo.join("dest-outside.txt")).expect("a link outside the worktree");
    std::fs::create_dir_all(repo.join("outlink")).expect("a directory a prefix names");
    std::os::unix::fs::symlink("../b.txt", repo.join("outlink/a.txt")).expect("a link at a prefixed path");
    return Places { root: repo.parent().expect("the fixture's directory").to_path_buf(), repo, outside };
}

/// What:
///  Every spelling of a destination as text: with and without a trailing separator,
///  ending inside a file name, a parent, a link, and a place outside the worktree.
///  `&Places` borrows the fixture's own absolute paths.
/// Why:
///  The two shapes Git reads differently are a prefix that names a folder and one that continues a
///  name, and a link stands where both can land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// destinationSpellings(places).map(([, text]) => text) === ['out/', 'out', '.', './', '..', 'link/', 'link', ...]
/// ```
fn destination_spellings(places: &Places) -> Vec<(&'static str, String)> {
    let outside: String = places.outside.display().to_string();
    return vec![
        ("with a separator", String::from("out/")),
        ("without a separator", String::from("out")),
        ("a dot", String::from(".")),
        ("a dot and a separator", String::from("./")),
        ("a parent", String::from("..")),
        ("through a link with a separator", String::from("link/")),
        ("through a link without a separator", String::from("link")),
        ("absolute outside with a separator", format!("{outside}/")),
        ("absolute outside without a separator", outside),
        ("parent-relative outside", String::from("../outside/")),
    ];
}

/// What:
///  Every guarded subcommand whose written paths are read below a prefix, with its table,
///  the words before that option and the words after it.
/// Why:
///  The prefix is the one option in the guarded tables that names a destination Git glues a path to,
///  so both subcommands that carry it are walked with every value-taking row of their own table,
///  generated from the tables rather than listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prefixedForms() === [['checkout-index', CheckoutIndex, CHECKOUT_INDEX_TABLE, ['-f'], ['-a']], ['read-tree', ReadTree, READ_TREE_TABLE, ['-u'], ['other']]];
/// ```
fn prefixed_forms() -> Vec<(&'static str, GuardedCommand, &'static [OptionSpec], &'static [&'static str], &'static [&'static str])> {
    return vec![
        ("checkout-index", GuardedCommand::CheckoutIndex, CHECKOUT_INDEX_TABLE, &["-f"], &["-a"]),
        ("read-tree", GuardedCommand::ReadTree, READ_TREE_TABLE, &["-u"], &["other"]),
    ];
}

/// One table row's long name, for a row's label.
fn spec_long(spec: &OptionSpec) -> String {
    return match spec.long {
        Some(long) => String::from(long),
        None => match spec.short {
            Some(letter) => format!("-{}", char::from(letter)),
            None => String::from("?"),
        },
    };
}

/// What:
///  One table row spelled with a value: the attached form for a long name,
///  and the long name with the value in the token after it for a short one.
///  Nothing for a row whose value Git refuses to read as a destination.
/// Why:
///  A short option's attached spelling takes no `=`, so the value is the next token there;
///  and a row that takes no value is not a destination at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spelledWithAValue({ long: 'prefix', arity: 'required' }, 'out/') === ['--prefix=out/'];
/// ```
fn spelled_with_a_value(spec: &OptionSpec, value: &str) -> Option<Vec<String>> {
    return match spec.long {
        Some(long) => Some(vec![format!("--{long}={value}")]),
        None => spec.short.map(|letter| return vec![format!("-{}", char::from(letter)), String::from(value)]),
    };
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
        ("whose last component is a link", String::from("outlink/")),
        ("an ignored directory without a separator", String::from("ignored")),
        ("an ignored directory with one", String::from("ignored/")),
    ];
    for (kind, prefix) in &prefixes {
        // `--prefix` and `--reset` exclude each other (`fatal: Which one? -m, --reset, or --prefix?`),
        //  so the form that runs is `-u --prefix=<directory>`; the reset form is the whole-tree row.
        rows.push(row(
            format!("read-tree -u --prefix=<{kind}> other"),
            "read-tree",
            GuardedCommand::ReadTree,
            vec![String::from("-u"), format!("--prefix={prefix}"), String::from("other")],
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
        // The destination itself is a link, which Git replaces rather than writing through
        //  (measured on this host with native Git 2.56.0, 2026-11-16).
        ("whose last component is a link to a tracked file", String::from("dest-tracked.txt")),
        ("whose last component is a link to an untracked file", String::from("dest-untracked.txt")),
        ("whose last component is a link to nothing", String::from("dest-dangling.txt")),
        ("whose last component is a link outside the worktree", String::from("dest-outside.txt")),
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
    // What:
    //  Every option of the guarded commands that takes a value Git reads as a destination,
    //  crossed with every spelling of a destination as text.
    // Why:
    //  Git glues the option's text to each path and reads the pair as one name,
    //  so a prefix that does not end with a separator continues the path's own name:
    //  `checkout-index -f --prefix=. -a` writes `.a.txt`, `--prefix=out -a` writes `outa.txt`, and
    //  `--prefix=link` writes `linka.txt` while the link itself is untouched
    //  (measured on this host with native Git 2.56.0, 2026-11-16).
    //  A guard that reads the prefix as a folder checks another file and lets the write through,
    //  which is the tenth review's first finding.
    //  The rows come from the tables the guard parses, so an option a newer Git adds is walked in
    //  each of its spellings, and every spelling of a destination is taken from one list rather than
    //  written out per row.
    for (kind, spelling) in destination_spellings(places) {
        for (subcommand, command, table, base, tail) in prefixed_forms() {
            for spec in table.iter().filter(|spec| return spec.arity != Arity::None) {
                let Some(option) = spelled_with_a_value(spec, spelling.as_str()) else {
                    continue;
                };
                let mut words: Vec<String> = base.iter().map(|word| return String::from(*word)).collect();
                words.extend(option);
                words.extend(tail.iter().map(|word| return String::from(*word)));
                rows.push(row(
                    format!("{subcommand} {base:?} {} <{kind}> {tail:?}", spec_long(spec)),
                    subcommand,
                    command,
                    words,
                    repo,
                ));
            }
        }
        // A forced move names its destination as a word, so the same spellings are walked there too.
        rows.push(row(
            format!("mv -f <relative a.txt> <{kind}> destination"),
            "mv",
            GuardedCommand::Move,
            vec![String::from("-f"), String::from("a.txt"), spelling],
            repo,
        ));
    }
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
        let before: BTreeMap<PathBuf, Held> = snapshot(places.root.as_path());
        let mut arguments: Vec<&str> = vec![row.word];
        arguments.extend(row.words.iter().map(String::as_str));
        let observed: std::process::Output = git_status(row.directory.as_path(), arguments.as_slice());
        let after: BTreeMap<PathBuf, Held> = snapshot(places.root.as_path());
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
