//! What:
//!  The fixtures and the two comparisons the store-form table of `repo_read_tree_store_forms_tests.rs` is built from:
//!  a repository whose history makes a repack delta its trees,
//!  the whole tree of `HEAD` as real Git lists it,
//!  the same tree read through this package's reader,
//!  and the store of one object as `git verify-pack -v` reports it.
//! Why:
//!   Every row of the table makes one store of a real Git repository and compares the reader's answer with Git's,
//!  so the fixtures and the comparison live together and each row only says which store it makes.

use crate::repo_read_object::{ObjectKind, from_hex};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_tree::FlatEntry;
use crate::test_support::{fixture, git, git_output};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// How many commits the fixture history holds:
/// enough that a repack has similar trees to delta one against another.
pub(crate) const COMMITS: usize = 24;

/// How many files the nested directory of the fixture history holds,
/// so a tree of it is large enough to be worth storing as a delta.
pub(crate) const FILES: usize = 60;

/// How many file versions the deep-chain history holds,
/// which is more than the reader may refuse a chain for.
pub(crate) const DEEP_FILES: usize = 300;

/// How many commits the deep-chain history holds.
pub(crate) const DEEP_COMMITS: usize = 130;

/// The object format of the repository at `repo`,
/// as real Git reports it.
pub(crate) fn format_of(repo: &Path) -> ObjectFormat {
    let named = String::from_utf8(git_output(repo, &["rev-parse", "--show-object-format"]).stdout).expect("UTF-8");
    return if named.trim() == "sha256" { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
}

/// What:
///  A repository with a nested directory of many files and a history whose trees are close to one another.
/// Why:
///   A repack deltifies trees only when there are trees to compare,
/// and the flat list of `HEAD` must cross a nested tree to be the whole tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = placedHistory(await history(root, 'repo', ['--object-format=sha256']));
/// ```
pub(crate) fn history(root: &Path, name: &str, init_flags: &[&str]) -> PathBuf {
    let repo: PathBuf = place(root, name, init_flags);
    std::fs::create_dir_all(repo.join("deep/nest")).expect("directories");
    for number in 0..FILES {
        std::fs::write(repo.join(format!("deep/nest/file-{number}.txt")), format!("file {number}\n")).expect("file");
    }
    std::fs::write(repo.join("top.txt"), "top\n").expect("top");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    for step in 0..COMMITS {
        std::fs::write(repo.join(format!("deep/nest/file-{}.txt", step % FILES)), format!("edited in step {step}\n")).expect("edit");
        std::fs::write(repo.join(format!("deep/added-{step}.txt")), format!("added {step}\n")).expect("added");
        git(repo.as_path(), &["add", "--all"]);
        git(repo.as_path(), &["commit", "--quiet", "--message", &format!("step {step}")]);
    }
    return repo;
}

/// A repository at `root/name` with nothing committed,
/// in the object format `init_flags` ask for.
pub(crate) fn place(root: &Path, name: &str, init_flags: &[&str]) -> PathBuf {
    let repo: PathBuf = root.join(name);
    std::fs::create_dir(&repo).expect("directory");
    let mut arguments: Vec<&str> = vec!["init", "--quiet", "--initial-branch=main"];
    arguments.extend_from_slice(init_flags);
    git(repo.as_path(), arguments.as_slice());
    return repo;
}

/// What:
///  A repository whose history one `git fast-import` writes:
///  a directory of `DEEP_FILES` files and `DEEP_COMMITS` commits,
///  each editing one of them.
/// Why:
///   A repack stores similar trees as a chain of deltas,
/// and a chain longer than the reader's own bound needs more versions of one tree than a fixture can afford to commit one at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = place(root, 'repo', []); await fastImport(repo, deepStream());
/// ```
pub(crate) fn imported_history(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = place(root, name, &[]);
    let mut stream: String = String::new();
    // Every file of the directory is one blob, numbered from two: mark one is the file beside the directory.
    for mark in 2..DEEP_FILES + 2 {
        let body: String = format!("row {mark}\n");
        stream.push_str(format!("blob\nmark :{mark}\ndata {}\n{body}", body.len()).as_str());
    }
    stream.push_str("blob\nmark :1\ndata 2\nx\n");
    stream.push_str("commit refs/heads/main\ncommitter Fixture <fixture@example.invalid> 1700000000 +0000\ndata 4\nbase\n");
    stream.push_str("M 100644 :1 top.txt\n");
    for mark in 2..DEEP_FILES + 2 {
        stream.push_str(format!("M 100644 :{mark} d/r{mark}.txt\n").as_str());
    }
    let mut mark: usize = DEEP_FILES + 2;
    for step in 0..DEEP_COMMITS {
        let body: String = format!("edited in step {step}\n");
        let message: String = format!("step {step}");
        stream.push_str(format!("blob\nmark :{mark}\ndata {}\n{body}", body.len()).as_str());
        stream.push_str(format!("commit refs/heads/main\ncommitter Fixture <fixture@example.invalid> {} +0000\ndata {}\n{message}\n", 1700000001 + step, message.len()).as_str());
        stream.push_str(format!("M 100644 :{mark} d/r{}.txt\n", step % DEEP_FILES).as_str());
        mark += 1;
    }
    let answer = asked(repo.as_path(), &["fast-import", "--quiet"], stream.as_bytes());
    assert!(answer.0, "git fast-import failed: {}", answer.1);
    return repo;
}

/// What:
///  Real Git's answer for one command line given `input` on its standard input:
///  whether it succeeded,
///  and its standard error.
/// Why:
///   `git fast-import` and `git cat-file --batch-check` take their work on standard input,
/// which the shared fixture helpers do not write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [ok, stderr] = runGit(repo, ['cat-file','--batch-check'], input);
/// ```
pub(crate) fn asked(repo: &Path, arguments: &[&str], input: &[u8]) -> (bool, String) {
    let mut child = Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("native Git fixture command");
    let mut standard_input = child.stdin.take().expect("piped standard input");
    let written: std::io::Result<()> = standard_input.write_all(input);
    drop(standard_input);
    let output = child.wait_with_output().expect("the query ends");
    return (written.is_ok() && output.status.success(), String::from_utf8_lossy(output.stderr.as_slice()).into_owned());
}

/// What:
///  Real Git's standard output for one command line given `input` on its standard input.
/// Why:
///   The one question the deep-chain fixture needs of Git answers with a name per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = await gitAsked(repo, ['cat-file','--batch-check'], ask);
/// ```
pub(crate) fn answered(repo: &Path, arguments: &[&str], input: &[u8]) -> String {
    let mut child = Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("native Git fixture command");
    let mut standard_input = child.stdin.take().expect("piped standard input");
    let _written = standard_input.write_all(input);
    drop(standard_input);
    let output = child.wait_with_output().expect("the query ends");
    assert!(output.status.success(), "git {:?} failed: {}", arguments, String::from_utf8_lossy(output.stderr.as_slice()));
    return String::from_utf8(output.stdout).expect("UTF-8");
}

/// What:
///  The whole tree of `HEAD` as real Git lists it:
///  mode,
///  object name and path of every non-directory entry.
/// Why:
///   `git ls-tree -r --full-tree -z` is the authority the reader is compared with,
/// and `-z` keeps a path's bytes exactly as stored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = parseLsTreeZ(git(['ls-tree','-r','--full-tree','-z','HEAD']).stdout);
/// ```
pub(crate) fn listed_by_git(repo: &Path) -> Vec<FlatEntry> {
    let format: ObjectFormat = format_of(repo);
    let output = git_output(repo, &["ls-tree", "-r", "--full-tree", "-z", "HEAD"]);
    assert!(output.status.success(), "git ls-tree failed: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    let mut entries: Vec<FlatEntry> = Vec::new();
    for record in output.stdout.split(|byte| { return *byte == 0; }) {
        if record.is_empty() {
            continue;
        }
        let tab: usize = record.iter().position(|byte| { return *byte == b'\t'; }).expect("a tab before the path");
        let (fields, path) = (&record[..tab], &record[tab + 1..]);
        let mut words = fields.split(|byte| { return *byte == b' '; });
        let mode_text: &str = std::str::from_utf8(words.next().expect("a mode")).expect("UTF-8 mode");
        let mode: u32 = u32::from_str_radix(mode_text, 8).expect("octal mode");
        let _kind = words.next().expect("a kind word");
        let id: Vec<u8> = from_hex(words.next().expect("a name"), format).expect("a hexadecimal object name");
        entries.push(FlatEntry { path: path.to_vec(), mode, id });
    }
    return entries;
}

/// What:
///  The commit `HEAD` names and its whole tree,
/// read in process,
/// or the words the cleanliness proof answers with when one of the reads declines.
/// Why:
///   The words are `read_state`'s own (`worktree/worktree_clean.rs`),
/// so a row that declines states exactly what a person meets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = readHeadTree(repo); // { ok: FlatEntry[] } | { refused: string }
/// ```
pub(crate) fn read_head_tree(repo: &Path) -> Result<Vec<FlatEntry>, String> {
    let format: ObjectFormat = format_of(repo);
    let Some(database) = crate::repo_read_odb::ObjectDatabase::open(repo.join(".git").as_path(), None, format) else {
        return Err("the object database has a layout the reader does not know".to_string());
    };
    let head = String::from_utf8(git_output(repo, &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    let name: Vec<u8> = from_hex(head.trim().as_bytes(), format).expect("a hexadecimal object name");
    let Some(commit) = database.read(name.as_slice()) else {
        return Err("the commit of HEAD is unreadable".to_string());
    };
    if commit.kind != ObjectKind::Commit {
        return Err("the commit of HEAD is unreadable".to_string());
    }
    let Some(fields) = crate::repo_read_tree::parse_commit(commit.bytes.as_slice(), format) else {
        return Err("the commit of HEAD has a layout the reader does not know".to_string());
    };
    return match crate::repo_read_tree::flatten_tree(&database, fields.tree.as_slice(), format) {
        Some(flat) => Ok(flat),
        None => Err("the tree of HEAD is unreadable".to_string()),
    };
}

/// Require the reader's flat list to be the one real Git lists,
/// byte for byte,
/// and answer how many entries were compared.
pub(crate) fn the_reader_lists_what_git_lists(repo: &Path, label: &str) -> usize {
    let expected: Vec<FlatEntry> = listed_by_git(repo);
    assert!(!expected.is_empty(), "{label}: git lists the tree of HEAD");
    let Ok(read) = read_head_tree(repo) else {
        let refused: String = read_head_tree(repo).expect_err("the refusal words");
        panic!("{label}: the reader declined: {refused}");
    };
    assert_eq!(read.len(), expected.len(), "{label}: the two lists hold as many entries");
    for (position, (mine, theirs)) in read.iter().zip(expected.iter()).enumerate() {
        assert_eq!(mine.mode, theirs.mode, "{label}: the mode of entry {position}");
        assert_eq!(mine.id, theirs.id, "{label}: the object name of {}", String::from_utf8_lossy(mine.path.as_slice()));
        assert_eq!(mine.path, theirs.path, "{label}: the path of entry {position}");
    }
    return expected.len();
}

/// The pack files of `repo` under `objects/pack`,
/// by their extension.
pub(crate) fn pack_files(repo: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(repo.join(".git/objects/pack"))
        .expect("pack directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.extension().is_some_and(|found| { return found == extension; }); })
        .collect();
    found.sort();
    return found;
}

/// What real Git's own `verify-pack -v` says about the object named `name`:
/// `None` when no pack of the repository holds it.
pub(crate) fn stored_as(repo: &Path, name: &str) -> Option<String> {
    for pack in pack_files(repo, "idx") {
        let table: String = String::from_utf8(git_output(repo, &["verify-pack", "-v", pack.to_str().expect("UTF-8")]).stdout).expect("UTF-8");
        for line in table.lines() {
            if line.starts_with(name) && line[name.len()..].starts_with(' ') {
                return Some(line.to_string());
            }
        }
    }
    return None;
}

/// The object name of the tree of `HEAD`.
pub(crate) fn head_tree_name(repo: &Path) -> String {
    let tree = git_output(repo, &["rev-parse", "HEAD^{tree}"]);
    assert!(tree.status.success(), "git rev-parse HEAD^{{tree}} failed");
    return String::from_utf8(tree.stdout).expect("UTF-8").trim().to_string();
}

/// What:
///  The delta base of every packed entry,
///  by object name,
///  and the number of entries the pack holds,
///  as `git verify-pack -v` reports them.
/// Why:
///   The chain a tree is stored in is the one real Git names,
/// and the fixture chooses the commit whose tree is deepest by following it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bases, entries } = chainOf(pack);
/// ```
pub(crate) fn delta_bases(repo: &Path) -> HashMap<String, String> {
    let mut bases: HashMap<String, String> = HashMap::new();
    for pack in pack_files(repo, "idx") {
        let table: String = String::from_utf8(git_output(repo, &["verify-pack", "-v", pack.to_str().expect("UTF-8")]).stdout).expect("UTF-8");
        for line in table.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.len() == 7 && words[0].len() == 40 {
                bases.insert(words[0].to_string(), words[6].to_string());
            }
        }
    }
    return bases;
}

/// How many deltas stand between `name` and the entry that stores it whole.
pub(crate) fn chain_depth(bases: &HashMap<String, String>, name: &str) -> usize {
    let mut depth: usize = 0;
    let mut at: String = name.to_string();
    while let Some(base) = bases.get(at.as_str()) {
        at = base.clone();
        depth += 1;
    }
    return depth;
}

/// The commit whose subtree at `path` is stored at the deepest delta chain,
/// and that depth.
pub(crate) fn deepest_subtree_commit(repo: &Path, path: &str, bases: &HashMap<String, String>) -> (String, usize) {
    let commits: String = String::from_utf8(git_output(repo, &["rev-list", "--all"]).stdout).expect("UTF-8");
    let ask: String = commits.lines().map(|commit| { return format!("{commit}:{path}\n"); }).collect();
    let answers: String = answered(repo, &["cat-file", "--batch-check"], ask.as_bytes());
    let mut best: (String, usize) = (String::new(), 0);
    for (commit, answer) in commits.lines().zip(answers.lines()) {
        let name: &str = answer.split_whitespace().next().unwrap_or_default();
        let depth: usize = chain_depth(bases, name);
        if depth > best.1 {
            best = (commit.to_string(), depth);
        }
    }
    return best;
}

/// A fixture root of this table's own.
pub(crate) fn table_fixture(name: &str) -> PathBuf {
    return fixture(name);
}
