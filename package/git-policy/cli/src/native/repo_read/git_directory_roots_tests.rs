//! What:
//!  Controls of the one answer to "which repository does this command work on":
//!  for every spelling of a repository,
//!  the Git directory and the common Git directory this crate resolves are the ones real Git names for the same
//!  command line and environment.
//! Why:
//!  The lock pass, the record of the processes a command starts and the reference-lock clearing before a forwarded
//!  `fetch` or `push` all act on this answer.
//!  It was once resolved twice,
//!  so a relative `-C` beside a relative `--git-dir` applied the chain twice and looked in a directory Git never uses
//!  (a review of `2af2ecb879`, finding 1).
//!  Real Git is the oracle:
//!  `git rev-parse --path-format=absolute --absolute-git-dir --git-common-dir` answers both directories for exactly the
//!  command line and environment the resolver is given.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const row of spellings) expect(canonical(resolve(row))).toEqual(canonical(gitRevParse(row)));
//! ```

use crate::git_directory_roots::{CommandRepository, repository_a_command_works_on};
use crate::test_support::{REAL_GIT, fixture, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One way of spelling a repository:
///  the directory the command runs in,
///  the global options before its subcommand,
///  and the variables it is started with.
struct Spelling {
    /// What the row names, for a failure to read.
    name: &'static str,
    /// The directory the command runs in.
    cwd: PathBuf,
    /// The global options before the subcommand.
    prefix: Vec<OsString>,
    /// The environment the invocation carries.
    environment: Vec<(OsString, OsString)>,
}

/// The environment every row's real Git runs under:
///  no system or global configuration,
///  and an identity for the commits that make the fixtures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const baseEnvironment = { GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/nonexistent-config', GIT_AUTHOR_NAME: 'Fixture', ... };
/// ```
fn base_environment() -> Vec<(OsString, OsString)> {
    let pairs: [(&str, &str); 6] = [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ];
    return pairs.iter().map(|(name, value)| { return (OsString::from(name), OsString::from(value)); }).collect();
}

/// One invocation of real Git, returning what it wrote of both streams and its status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const git = (cwd, args, env) => spawnSync('/usr/bin/git', args, { cwd, env });
/// ```
fn real_git(cwd: &Path, arguments: &[&str], environment: &[(OsString, OsString)]) -> std::process::Output {
    let mut command: Command = Command::new(REAL_GIT);
    command.current_dir(cwd).args(arguments);
    // The selectors come only from the row, never from the process this test runs in.
    for selector in ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR"] {
        command.env_remove(selector);
    }
    for (name, value) in environment {
        command.env(name, value);
    }
    return command.output().expect("real Git runs");
}

/// What:
///  The two directories real Git names for one row:
///  its Git directory and its common Git directory.
/// Why:
///   Both are asked in one run,
///  with the path format that makes the common directory absolute,
///  so neither answer depends on the directory this test runs in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [gitDir, commonDir] = lines(run(row, ['rev-parse', '--path-format=absolute', '--absolute-git-dir', '--git-common-dir']));
/// ```
fn git_answers(row: &Spelling) -> (PathBuf, PathBuf) {
    let mut arguments: Vec<&str> = row.prefix.iter().map(|word| { return word.to_str().expect("a row's option is text"); }).collect();
    arguments.extend(["rev-parse", "--path-format=absolute", "--absolute-git-dir", "--git-common-dir"]);
    let output: std::process::Output = real_git(row.cwd.as_path(), arguments.as_slice(), row.environment.as_slice());
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let stderr: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(output.status.success(), "{}: real Git answered {text:?} and refused with {stderr:?}", row.name);
    let mut lines = text.lines();
    let git_dir: PathBuf = canonical(lines.next().expect("the Git directory"), row.name, "the Git directory");
    let common: PathBuf = canonical(lines.next().expect("the common Git directory"), row.name, "the common Git directory");
    return (git_dir, common);
}

/// Resolve one path as far as it exists,
///  so two spellings of one directory compare equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const canonical = (path) => realpathSync(path);
/// ```
fn canonical(path: &str, row: &str, which: &str) -> PathBuf {
    return std::fs::canonicalize(path).unwrap_or_else(|error| { panic!("{row}: {which} {path} does not resolve: {error}") });
}

/// What:
///  The repositories the table's rows name, made once:
///  two plain repositories,
///  a subdirectory,
///  a linked worktree of one of them,
///  and a bare repository.
///  `(PathBuf, ...)` is the directory the rows are built under.
/// Why:
///  Every spelling needs a real repository to point at,
///  and one fixture serves every row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = makeRepositories(fixtureRoot);
/// ```
fn repositories(control: &str) -> PathBuf {
    let root: PathBuf = fixture(control);
    let parent: PathBuf = root.join("parent");
    std::fs::create_dir_all(parent.join("running-in").as_path()).expect("the directory of a repository");
    std::fs::create_dir_all(parent.join("named/sub").as_path()).expect("a subdirectory of a repository");
    let setup: Vec<Vec<&str>> = vec![
        vec!["init", "--quiet", "--initial-branch=main", "running-in"],
        vec!["init", "--quiet", "--initial-branch=main", "named"],
        vec!["-C", "running-in", "commit", "--quiet", "--allow-empty", "--message=base"],
        vec!["-C", "named", "commit", "--quiet", "--allow-empty", "--message=base"],
        vec!["init", "--quiet", "--bare", "bare.git"],
        vec!["-C", "named", "worktree", "add", "--quiet", "../linked"],
    ];
    for arguments in setup {
        let output: std::process::Output = real_git(parent.as_path(), arguments.as_slice(), base_environment().as_slice());
        assert!(output.status.success(), "the fixture's Git {arguments:?} ran: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    }
    return parent;
}

/// What:
///  Every row of the table:
///  a `-C` once and twice, relative and absolute,
///  `--git-dir` with and without `--work-tree`,
///  a relative one under a relative `-C`,
///  `GIT_DIR`, `GIT_WORK_TREE` and `GIT_COMMON_DIR`,
///  a linked worktree and a bare repository.
/// Why:
///  These are the spellings a command uses to say which repository it means,
///  and the resolver must read each as Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = (parent) => [{ name: 'plain', cwd: join(parent, 'named'), prefix: [], environment: base() }, ...];
/// ```
fn spellings(parent: &Path) -> Vec<Spelling> {
    let base: Vec<(OsString, OsString)> = base_environment();
    let with = |extra: Vec<(OsString, OsString)>| -> Vec<(OsString, OsString)> {
        let mut all: Vec<(OsString, OsString)> = base.clone();
        all.extend(extra);
        return all;
    };
    let pair = |name: &str, value: &Path| -> (OsString, OsString) { return (OsString::from(name), value.as_os_str().to_os_string()); };
    let named: PathBuf = parent.join("named");
    let named_git: PathBuf = named.join(".git");
    let running_in: PathBuf = parent.join("running-in");
    let running_in_git: PathBuf = running_in.join(".git");
    return vec![
        Spelling { name: "plain in the repository root", cwd: named.clone(), prefix: vec![], environment: with(vec![]) },
        Spelling { name: "plain in a subdirectory", cwd: named.join("sub"), prefix: vec![], environment: with(vec![]) },
        Spelling { name: "one relative -C", cwd: parent.to_path_buf(), prefix: vec![OsString::from("-C"), OsString::from("named/sub")], environment: with(vec![]) },
        Spelling { name: "an absolute -C", cwd: parent.to_path_buf(), prefix: vec![OsString::from("-C"), named.as_os_str().to_os_string()], environment: with(vec![]) },
        Spelling {
            name: "two -C links, each from the one before",
            cwd: parent.to_path_buf(),
            prefix: vec![OsString::from("-C"), OsString::from("named"), OsString::from("-C"), OsString::from("sub")],
            environment: with(vec![]),
        },
        Spelling {
            name: "an absolute --git-dir with its --work-tree",
            cwd: running_in.clone(),
            prefix: vec![OsString::from(format!("--git-dir={}", named_git.display())), OsString::from(format!("--work-tree={}", named.display()))],
            environment: with(vec![]),
        },
        Spelling {
            name: "a --git-dir split from its value, with a --work-tree",
            cwd: running_in.clone(),
            prefix: vec![
                OsString::from("--git-dir"),
                named_git.as_os_str().to_os_string(),
                OsString::from("--work-tree"),
                named.as_os_str().to_os_string(),
            ],
            environment: with(vec![]),
        },
        Spelling {
            name: "a relative --git-dir under one relative -C",
            cwd: parent.to_path_buf(),
            prefix: vec![
                OsString::from("-C"),
                OsString::from("running-in"),
                OsString::from("--git-dir=../named/.git"),
                OsString::from("--work-tree=../named"),
            ],
            environment: with(vec![]),
        },
        Spelling {
            name: "a bare repository as the directory the command runs in",
            cwd: parent.join("bare.git"),
            prefix: vec![],
            environment: with(vec![]),
        },
        Spelling { name: "GIT_DIR naming a Git directory", cwd: running_in, prefix: vec![], environment: with(vec![pair("GIT_DIR", named_git.as_path())]) },
        Spelling {
            name: "a relative GIT_DIR under one relative -C",
            cwd: parent.to_path_buf(),
            prefix: vec![OsString::from("-C"), OsString::from("running-in")],
            environment: with(vec![(OsString::from("GIT_DIR"), OsString::from("../named/.git"))]),
        },
        Spelling {
            name: "GIT_DIR with GIT_COMMON_DIR naming another repository's common directory",
            cwd: parent.to_path_buf(),
            prefix: vec![],
            environment: with(vec![pair("GIT_DIR", running_in_git.as_path()), pair("GIT_COMMON_DIR", named_git.as_path())]),
        },
        Spelling { name: "a linked worktree", cwd: parent.join("linked"), prefix: vec![], environment: with(vec![]) },
        Spelling {
            name: "a linked worktree whose common directory is named",
            cwd: parent.join("linked"),
            prefix: vec![],
            environment: with(vec![pair("GIT_COMMON_DIR", named_git.as_path())]),
        },
        Spelling {
            name: "GIT_WORK_TREE alone, which names no Git directory",
            cwd: named,
            prefix: vec![],
            environment: with(vec![(OsString::from("GIT_WORK_TREE"), OsString::from("/nonexistent-work-tree"))]),
        },
    ];
}

/// What:
///  Every row's Git directory and common Git directory are the ones real Git names,
///  and the row's environment says which of them the row sets.
/// Why:
///  This is the table the answer to "which repository does this command work on" is checked against,
///  and one wrong row is one repository a lock pass would search in vain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const row of rows) { const expected = gitAnswers(row); expect(canonical(resolve(row))).toEqual(expected); }
/// ```
#[test]
fn the_git_directories_the_resolver_answers_are_the_ones_real_git_names_for_every_spelling() {
    let parent: PathBuf = repositories("command-repository-spellings");
    let rows: Vec<Spelling> = spellings(parent.as_path());
    assert!(rows.len() >= 15, "the table holds every spelling this control knows: {} rows", rows.len());
    for row in &rows {
        let (expected_git, expected_common): (PathBuf, PathBuf) = git_answers(row);
        let resolved: CommandRepository = repository_a_command_works_on(row.cwd.as_path(), row.prefix.as_slice(), row.environment.as_slice());
        let git_dir: PathBuf = std::fs::canonicalize(resolved.git_dir.as_ref().unwrap_or_else(|| { panic!("{}: no Git directory was named", row.name) }))
            .unwrap_or_else(|error| { panic!("{}: the resolver's Git directory does not resolve: {error}", row.name) });
        let common: PathBuf = std::fs::canonicalize(resolved.common_dir.as_ref().unwrap_or_else(|| { panic!("{}: no common directory was named", row.name) }))
            .unwrap_or_else(|error| { panic!("{}: the resolver's common directory does not resolve: {error}", row.name) });
        assert_eq!(git_dir, expected_git, "{}: the Git directory is the one Git names", row.name);
        assert_eq!(common, expected_common, "{}: the common Git directory is the one Git names", row.name);
    }
    remove(parent.parent().expect("the fixture root"));
}
