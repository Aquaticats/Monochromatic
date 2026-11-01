//! The private temporary directory of each process:
//!  its name,
//!  the second process that must leave it alone,
//!  and the removal of what an ended process left.

use super::{lock_path, scratch_name};
use crate::language::launch::{ServerLaunch, prepare};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command, Stdio},
};

/// Where the helper process finds the state root and the project of the test that started it.
const HELPER_STATE: &str = "IDE_SCRATCH_HELPER_STATE";
const HELPER_PROJECT: &str = "IDE_SCRATCH_HELPER_PROJECT";

fn canonical(directory: &tempfile::TempDir) -> PathBuf {
    return directory
        .path()
        .canonicalize()
        .expect("canonical directory");
}

/// Prepare the launch of a server named `server` the way the confining policy does,
/// from this process.
fn prepare_here(state: &Path, project: &Path) -> PathBuf {
    let server = state.join("server");
    let scratch = server.join(scratch_name());
    let wanted = ServerLaunch {
        command: "/bin/true".to_string(),
        args: Vec::new(),
        environment: HashMap::new(),
        settings: None,
        directories: vec![server],
        scratch: vec![scratch.clone()],
    };
    prepare(&wanted, project, Some(state)).expect("preparation");
    return scratch;
}

/// The second process of the tests below:
///  it prepares the scratch directory of the same server and project,
///  writes a file in it,
///  reports its path,
///  and stays alive until its standard input closes.
#[test]
#[ignore = "runs as the second process of the tests that start it"]
fn helper_process_prepares_its_scratch_directory_and_waits() {
    let state = std::env::var(HELPER_STATE).expect("the starting test names the state root");
    let project = std::env::var(HELPER_PROJECT).expect("the starting test names the project");
    let scratch = prepare_here(Path::new(&state), Path::new(&project));
    fs::write(scratch.join("child-file"), "child").expect("file of the helper");
    println!("SCRATCH_DIRECTORY={}", scratch.display());
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("standard input of the helper");
}

/// A running helper process,
///  stopped when the value is dropped.
struct Helper {
    child: Child,
    scratch: PathBuf,
    // Kept open so the helper's later output does not meet a closed pipe.
    _output: BufReader<ChildStdout>,
}

impl Drop for Helper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Helper {
    /// Close the helper's standard input and wait for it to end,
    ///  as the system does when a process ends.
    fn finish(&mut self) {
        drop(self.child.stdin.take());
        let status = self.child.wait().expect("helper exit");
        assert!(status.success(), "the helper process failed: {status}");
    }
}

fn start_helper(state: &Path, project: &Path) -> Helper {
    let module = module_path!()
        .split_once("::")
        .expect("module path below the crate")
        .1;
    let mut child = Command::new(std::env::current_exe().expect("test binary"))
        .arg(format!(
            "{module}::helper_process_prepares_its_scratch_directory_and_waits"
        ))
        .args(["--exact", "--ignored", "--nocapture"])
        .env(HELPER_STATE, state)
        .env(HELPER_PROJECT, project)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start the helper process");
    let mut output = BufReader::new(child.stdout.take().expect("helper output"));
    let mut scratch = None;
    let mut line = String::new();
    while scratch.is_none() {
        line.clear();
        let read = output.read_line(&mut line).expect("helper output line");
        assert!(read > 0, "the helper ended before it reported a directory");
        if let Some(path) = line.trim_end().strip_prefix("SCRATCH_DIRECTORY=") {
            scratch = Some(PathBuf::from(path));
        }
    }
    return Helper {
        child,
        scratch: scratch.expect("reported directory"),
        _output: output,
    };
}

#[test]
fn a_scratch_directory_is_named_for_the_process() {
    let name = scratch_name();
    assert!(
        name.starts_with(&format!("tmp-{}-", std::process::id())),
        "the name does not carry the process id: {name}"
    );
    assert_eq!(name, scratch_name(), "the name changed within one process");
}

/// Two IDE processes on one project must not empty each other's temporary files:
/// the second one's launch used to empty the directory the first one's running server has as `/tmp`.
#[test]
fn a_second_process_does_not_empty_the_scratch_directory_of_a_running_first_one() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    let first = prepare_here(&state, &root);
    fs::write(first.join("made-by-first"), "first").expect("file of the first process");
    let second = start_helper(&state, &root);
    assert_ne!(
        second.scratch, first,
        "two processes were given one scratch directory"
    );
    assert!(
        first.join("made-by-first").exists(),
        "the second process emptied the scratch directory of the running first one"
    );
    assert!(second.scratch.join("child-file").exists());
    // The first process starts its server again: only its own directory is emptied.
    let again = prepare_here(&state, &root);
    assert_eq!(again, first);
    assert!(
        !first.join("made-by-first").exists(),
        "a restart kept a file of the earlier launch"
    );
    assert!(
        second.scratch.join("child-file").exists(),
        "the first process emptied the scratch directory of the running second one"
    );
}

/// A process that ended cannot say so,
/// so the next preparation finds the lock it held released.
#[test]
fn the_scratch_directory_of_an_ended_process_is_removed_by_the_next_preparation() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    let mut second = start_helper(&state, &root);
    let mine = prepare_here(&state, &root);
    assert!(
        second.scratch.join("child-file").exists(),
        "a running process's directory was removed as if its owner had ended"
    );
    second.finish();
    prepare_here(&state, &root);
    assert!(
        !second.scratch.exists(),
        "the directory of an ended process stayed"
    );
    assert!(
        !lock_path(&second.scratch).exists(),
        "the lock file of an ended process stayed"
    );
    assert!(mine.is_dir(), "the directory of this process was removed");
}

/// A held lock is what marks a live owner.
/// The test holds one itself:
/// a lock taken through another open of the same file conflicts even inside one process.
#[test]
fn a_scratch_directory_whose_lock_is_held_is_kept_and_one_without_a_lock_is_removed() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    let server = state.join("server");
    let live = server.join("tmp-4-aa");
    let unlocked = server.join("tmp-5-bb");
    let no_lock_file = server.join("tmp-6-cc");
    for directory in [&live, &unlocked, &no_lock_file] {
        fs::create_dir_all(directory).expect("sibling directory");
        fs::write(directory.join("file"), "x").expect("sibling file");
    }
    fs::write(lock_path(&unlocked), "").expect("lock file of an ended owner");
    let held = fs::File::create(lock_path(&live)).expect("lock file of the live owner");
    held.try_lock().expect("the live owner takes its lock");
    prepare_here(&state, &root);
    assert!(
        live.join("file").exists(),
        "a directory whose owner holds its lock was removed"
    );
    assert!(lock_path(&live).exists());
    assert!(
        !unlocked.exists() && !lock_path(&unlocked).exists(),
        "the directory of an owner without a held lock stayed"
    );
    assert!(
        !no_lock_file.exists(),
        "a directory without a lock file stayed"
    );
    drop(held);
    prepare_here(&state, &root);
    assert!(
        !live.exists(),
        "the directory stayed after its owner released the lock"
    );
}

/// Only names this module makes are removed:
/// the shared directory of earlier versions may still be the `/tmp` of a running older process.
#[test]
fn entries_that_are_not_scratch_directories_of_a_process_are_left() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    let server = state.join("server");
    for name in [
        "tmp",
        "cache",
        "tmp-notes",
        "tmp-12-xyz",
        "tmp-1-2-3",
        "tmp--aa",
    ] {
        fs::create_dir_all(server.join(name)).expect("unrelated directory");
        fs::write(server.join(name).join("keep"), "keep").expect("unrelated file");
    }
    fs::write(server.join("tmp-9-aa"), "a file, not a directory").expect("unrelated file");
    prepare_here(&state, &root);
    for name in [
        "tmp",
        "cache",
        "tmp-notes",
        "tmp-12-xyz",
        "tmp-1-2-3",
        "tmp--aa",
    ] {
        assert!(
            server.join(name).join("keep").exists(),
            "{name} was removed as a scratch directory"
        );
    }
    assert!(server.join("tmp-9-aa").exists());
}

/// A server that starts again in the same process starts clean in the same directory,
/// and the process never takes its own directory for one of an ended process.
#[test]
fn a_process_empties_its_own_scratch_directory_in_place_and_keeps_it() {
    let parent = crate::test_scratch::scratch().expect("parent");
    let root = canonical(&parent).join("project");
    let state = canonical(&parent).join("state");
    fs::create_dir(&root).expect("project");
    let scratch = prepare_here(&state, &root);
    fs::write(scratch.join("old"), "old").expect("file");
    assert_eq!(prepare_here(&state, &root), scratch);
    assert!(scratch.is_dir());
    assert!(!scratch.join("old").exists());
    assert!(lock_path(&scratch).exists());
}
