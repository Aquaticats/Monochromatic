//! What:
//!  The status query of each scope and the reading of its answers.
//! Why:
//!  A scope read as the other one would reject a harmless command or allow a deleting one;
//! a failed status read
//!  as clean would let every discarding command through.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(dirtyFromOutput({ success: false, ... })).toThrow();
//! ```

/// The query builder,
/// the reader and the scopes.
use super::{
    DirtyScope, dirty_from_output, dirty_query_arguments, ignored_paths_from_answer,
    ignored_paths_queries, index_file_is_absent, replaced_answers, untracked_paths_queries, with_ancestors,
};
use crate::command_test_support::{byte_argument, fixture, git, os_arguments, remove, repository};
use crate::git_metadata::MetadataOutput;
use std::ffi::{OsStr, OsString};

/// Build the output of one status run.
fn output(success: bool, stdout: &[u8], stderr: &[u8]) -> MetadataOutput {
    return MetadataOutput {
        success,
        code: Some(if success { 0 } else { 128 }),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    };
}

/// The global options come first,
/// then the read-only status with the scope's untracked setting.
#[test]
fn the_query_names_the_scope_after_the_global_options() {
    let prefix: Vec<OsString> = os_arguments(&["-C", "dir"]);
    assert_eq!(
        dirty_query_arguments(prefix.as_slice(), DirtyScope::Tracked),
        os_arguments(&[
            "-C",
            "dir",
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=no"
        ])
    );
    // A reading that counts untracked files asks for them; the tracked-only reading asks for none.
    assert_eq!(
        dirty_query_arguments(&[], DirtyScope::TrackedAndUntracked),
        os_arguments(&[
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=normal"
        ])
    );
}

/// Output means dirty,
/// silence means clean,
/// and a failure is an error carrying Git's first line.
#[test]
fn output_means_dirty_and_failure_is_an_error() {
    assert_eq!(dirty_from_output(&output(true, b" M a.txt\n", b"")), Ok(true));
    assert_eq!(dirty_from_output(&output(true, b"", b"")), Ok(false));
    assert_eq!(
        dirty_from_output(&output(false, b"", b"fatal: not a git repository\nmore\n")),
        Err(String::from(
            "Git could not read the worktree status: fatal: not a git repository"
        ))
    );
    assert_eq!(
        dirty_from_output(&output(false, b" M a.txt\n", b"")),
        Err(String::from("Git could not read the worktree status: "))
    );
}

/// Real Git:
/// a tracked edit counts in both scopes,
/// an untracked file only in the wider one,
/// and an ignored file in neither,
/// because a clean in a linked worktree may delete it.
#[test]
fn each_scope_counts_the_state_it_can_destroy() {
    let root = fixture("dirty-scope");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b".env\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    for scope in [DirtyScope::Tracked, DirtyScope::TrackedAndUntracked] {
        assert!(!ask(repo.as_path(), scope), "{scope:?} clean");
    }
    std::fs::write(repo.join("new.txt"), "x\n").expect("untracked file");
    assert!(!ask(repo.as_path(), DirtyScope::Tracked), "an untracked file is not tracked work");
    assert!(ask(repo.as_path(), DirtyScope::TrackedAndUntracked));
    git(repo.as_path(), &["add", "--", "new.txt"]);
    assert!(ask(repo.as_path(), DirtyScope::Tracked));
    git(repo.as_path(), &["commit", "--quiet", "--message=untracked"]);
    std::fs::write(repo.join(".env"), b"SECRET=1\n").expect("ignored file");
    assert!(!ask(repo.as_path(), DirtyScope::Tracked), "an ignored file is not tracked work");
    assert!(
        !ask(repo.as_path(), DirtyScope::TrackedAndUntracked),
        "an ignored file is not untracked, and a clean may delete it in a linked worktree"
    );
    remove(root.as_path());
}

/// Run the query with real Git inside a repository and read it.
fn ask(repo: &std::path::Path, scope: DirtyScope) -> bool {
    let arguments: Vec<OsString> = dirty_query_arguments(&[], scope);
    let raw = crate::command_test_support::git_output(repo, arguments.as_slice());
    let output = output(raw.status.success(), &raw.stdout, &raw.stderr);
    return dirty_from_output(&output).expect("status answers");
}

/// A Git directory without an `index` file is absent;
/// one with an `index` file,
/// empty included,
/// is not.
#[test]
fn the_index_file_is_absent_only_where_the_git_directory_has_none() {
    let root = fixture("index-file-absent");
    let git_directory = root.join("git-directory");
    std::fs::create_dir(&git_directory).expect("git directory");
    assert!(index_file_is_absent(Some(git_directory.as_path()), None));
    std::fs::write(git_directory.join("index"), b"").expect("empty index file");
    assert!(!index_file_is_absent(Some(git_directory.as_path()), None));
    remove(root.as_path());
}

/// A Git directory that does not exist,
/// no Git directory,
/// and a `GIT_INDEX_FILE` that names another index leave the question to the status.
#[test]
fn the_index_file_is_not_absent_when_the_question_is_not_the_default_index() {
    let root = fixture("index-file-unknown");
    let missing = root.join("no-such-git-directory");
    assert!(!index_file_is_absent(Some(missing.as_path()), None));
    assert!(!index_file_is_absent(None, None));
    let git_directory = root.join("git-directory");
    std::fs::create_dir(&git_directory).expect("git directory");
    assert!(!index_file_is_absent(Some(git_directory.as_path()), Some(OsStr::new("other-index"))));
    remove(root.as_path());
}

/// The ignored-paths query asks Git for the named paths alone,
/// exactly as they are,
/// in as many bounded
///  starts as the paths need.
#[test]
fn the_ignored_paths_query_names_each_path_exactly() {
    let prefix: Vec<OsString> = os_arguments(&["-C", "dir"]);
    let paths: Vec<Vec<u8>> = vec![b".env".to_vec(), b"a*b".to_vec(), vec![0xff, 0xfe]];
    let mut expected: Vec<OsString> = vec![
        OsString::from("-C"),
        OsString::from("dir"),
        OsString::from("--no-optional-locks"),
        OsString::from("--no-literal-pathspecs"),
        OsString::from("ls-files"),
        OsString::from("--others"),
        OsString::from("--ignored"),
        OsString::from("--exclude-standard"),
        OsString::from("-z"),
        OsString::from("--full-name"),
        OsString::from("--"),
        OsString::from(":(top,literal).env"),
        OsString::from(":(top,literal)a*b"),
    ];
    let mut not_utf8: Vec<u8> = b":(top,literal)".to_vec();
    not_utf8.extend_from_slice(&[0xff, 0xfe]);
    expected.push(byte_argument(not_utf8.as_slice()));
    assert_eq!(ignored_paths_queries(prefix.as_slice(), paths.as_slice()), vec![expected]);
    // No path, no start at all.
    assert!(ignored_paths_queries(&[], &[]).is_empty());
    // A long list is asked in bounded starts, and every path lands in exactly one of them.
    // 4000 paths of about 50 bytes each are more than one start may carry.
    let many: Vec<Vec<u8>> = (0..4000).map(|index| return format!("dir with a fairly long name/{index}.bin").into_bytes()).collect();
    let queries: Vec<Vec<OsString>> = ignored_paths_queries(&[], many.as_slice());
    assert!(queries.len() > 1, "one start carried every path");
    let mut asked: Vec<String> = Vec::<String>::new();
    for query in &queries {
        let specs: Vec<&OsString> = query
            .iter()
            .filter(|argument| return argument.to_string_lossy().starts_with(":(top,literal)"))
            .collect();
        assert!(!specs.is_empty(), "a start carried no path");
        for argument in specs {
            asked.push(argument.to_string_lossy().into_owned());
        }
    }
    assert_eq!(asked.len(), many.len());
    assert_eq!(asked.first(), Some(&String::from(":(top,literal)dir with a fairly long name/0.bin")));
    assert_eq!(asked.last(), Some(&String::from(":(top,literal)dir with a fairly long name/3999.bin")));
}

/// The untracked query is the ignored one without the flag that would narrow it,
///  so Git names every untracked file at the asked paths.
#[test]
fn the_untracked_paths_query_names_each_path_exactly_without_the_ignored_flag() {
    let prefix: Vec<OsString> = os_arguments(&["-C", "dir"]);
    let paths: Vec<Vec<u8>> = vec![b"victim.txt".to_vec()];
    let expected: Vec<OsString> = vec![
        OsString::from("-C"),
        OsString::from("dir"),
        OsString::from("--no-optional-locks"),
        OsString::from("--no-literal-pathspecs"),
        OsString::from("ls-files"),
        OsString::from("--others"),
        OsString::from("--exclude-standard"),
        OsString::from("-z"),
        OsString::from("--full-name"),
        OsString::from("--"),
        OsString::from(":(top,literal)victim.txt"),
    ];
    assert_eq!(untracked_paths_queries(prefix.as_slice(), paths.as_slice()), vec![expected]);
    assert!(untracked_paths_queries(&[], &[]).is_empty());
}

/// The ignored answer lists exactly the paths Git printed,
/// and a failed query is an error rather than "nothing is ignored".
#[test]
fn the_ignored_answer_lists_the_paths_git_printed() {
    assert_eq!(
        ignored_paths_from_answer(&output(true, b".env\0ignored/data.bin\0", b"")),
        Ok(vec![b".env".to_vec(), b"ignored/data.bin".to_vec()])
    );
    assert_eq!(ignored_paths_from_answer(&output(true, b"", b"")), Ok(Vec::<Vec<u8>>::new()));
    // A path that holds a newline or bytes that are not UTF-8 keeps its bytes exactly.
    assert_eq!(
        ignored_paths_from_answer(&output(true, b"od\nd name\0\xff\xfe\0", b"")),
        Ok(vec![b"od\nd name".to_vec(), vec![0xff, 0xfe]])
    );
    assert_eq!(
        ignored_paths_from_answer(&output(false, b"", b"fatal: not a git repository\n")),
        Err(String::from("Git could not read the ignored files: fatal: not a git repository"))
    );
}

/// Real Git:
///  the ignored-paths query names the ignored files the tree tracks,
/// a path that is only untracked is not one of them,
/// and a path that is ignored but absent is left to the worktree's disk.
#[test]
fn real_git_lists_the_ignored_file_the_tree_tracks() {
    let root = fixture("dirty-ignored-shared");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b".env\nignored/\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"tracked version\n").expect("tracked environment file");
    std::fs::create_dir_all(repo.join("ignored")).expect("ignored directory");
    std::fs::write(repo.join("ignored/data.bin"), b"kept elsewhere\n").expect("ignored file");
    git(repo.as_path(), &["add", "--force", "--", ".env", "ignored/data.bin"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    // The worktree holds both ignored entries again; only the local environment file differs from the tree.
    std::fs::write(repo.join(".env"), b"the person value\n").expect("ignored file");
    std::fs::create_dir_all(repo.join("ignored")).expect("ignored directory");
    std::fs::write(repo.join("ignored/data.bin"), b"the person version\n").expect("ignored file");
    std::fs::write(repo.join("ignored/local.bin"), b"the person other file\n").expect("ignored work");
    let tree = crate::command_test_support::git_output(
        repo.as_path(),
        os_arguments(&["ls-tree", "-r", "-z", "--name-only", "tracks"]).as_slice(),
    );
    let mut tracked: Vec<Vec<u8>> = tree
        .stdout
        .split(|byte| return *byte == 0)
        .filter(|path| return !path.is_empty())
        .map(|path| return path.to_vec())
        .collect();
    tracked.sort();
    assert_eq!(
        tracked,
        vec![b".env".to_vec(), b".gitignore".to_vec(), b"ignored/data.bin".to_vec(), b"tracked.txt".to_vec()]
    );
    let queries: Vec<Vec<OsString>> = ignored_paths_queries(&[], tracked.as_slice());
    assert_eq!(queries.len(), 1);
    let listed = crate::command_test_support::git_output(repo.as_path(), queries[0].as_slice());
    let answer = output(listed.status.success(), &listed.stdout, &listed.stderr);
    assert_eq!(
        ignored_paths_from_answer(&answer),
        Ok(vec![b".env".to_vec(), b"ignored/data.bin".to_vec()])
    );
    // A tree path that is ignored but absent from the worktree is not a file the command would replace.
    std::fs::remove_file(repo.join("ignored/data.bin")).expect("drop the ignored file");
    let remaining = crate::command_test_support::git_output(repo.as_path(), queries[0].as_slice());
    let after = output(remaining.status.success(), &remaining.stdout, &remaining.stderr);
    assert_eq!(ignored_paths_from_answer(&after), Ok(vec![b".env".to_vec()]));
    remove(root.as_path());
}

/// Every directory on the way to a tracked path is a path the worktree may hold as a file,
/// and an answer
///  inside one of those directories is not such a path.
#[test]
fn the_question_names_every_directory_the_tree_needs() {
    let tracked: Vec<Vec<u8>> = vec![
        b"a.txt".to_vec(),
        b"one/two/three.txt".to_vec(),
        b"one/four.txt".to_vec(),
    ];
    assert_eq!(
        with_ancestors(tracked.as_slice()),
        vec![
            b"a.txt".to_vec(),
            b"one".to_vec(),
            b"one/four.txt".to_vec(),
            b"one/two".to_vec(),
            b"one/two/three.txt".to_vec()
        ]
    );
    assert!(with_ancestors(&[]).is_empty());
    // A directory a worktree holds as a file is asked about as the directory itself;
    // an entry Git lists inside
    //  a directory the tree also has is one the form leaves alone.
    assert_eq!(
        replaced_answers(
            &with_ancestors(tracked.as_slice()),
            tracked.as_slice(),
            &[b"one/two/three.txt".to_vec(), b"one/two/other.txt".to_vec()]
        ),
        vec![b"one/two/three.txt".to_vec()]
    );
    assert_eq!(
        replaced_answers(&[b"one".to_vec()], tracked.as_slice(), &[b"one".to_vec()]),
        vec![b"one".to_vec()]
    );
    assert!(
        replaced_answers(&[b"one".to_vec()], tracked.as_slice(), &[b"one/inside.txt".to_vec()]).is_empty(),
        "a directory the tree also has keeps the files inside it"
    );
    // A tree path the worktree holds as a directory loses everything inside it when the tree writes a file
    //  there, so those answers are the files the form deletes.
    let replaced: Vec<Vec<u8>> = vec![b"private".to_vec(), b"nested/file.txt".to_vec()];
    assert_eq!(
        replaced_answers(
            &with_ancestors(replaced.as_slice()),
            replaced.as_slice(),
            &[
                b"private/secret.txt".to_vec(),
                b"private/deeper/other.txt".to_vec(),
                b"nested/file.txt/inside.txt".to_vec(),
                b"private-notes.txt".to_vec()
            ]
        ),
        vec![
            b"private/secret.txt".to_vec(),
            b"private/deeper/other.txt".to_vec(),
            b"nested/file.txt/inside.txt".to_vec()
        ]
    );
    // A name that merely starts the same letters is below nothing.
    assert!(
        replaced_answers(&[], &[b"private".to_vec()], &[b"private-notes.txt".to_vec()]).is_empty()
    );
}
