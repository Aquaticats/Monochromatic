//! Unit tests of the sentences an install prints about the copies a killed install left:
//! the publish it finished,
//! the copy it left beside a hidden name whose installer may still run,
//! the hidden directory it removed with a finished copy inside,
//! and the copy a failed download leaves a person to deal with.
//! Each sentence is read from a run of `sync_tools`,
//! never from the function that words it,
//! so a sentence that stops being reached fails here.
//! Every state is built from real names and real markers,
//! and the copies are finished the way an install finishes them,
//! so nothing here stands in for another program's output.

/// Imports `fs` from `std`.
use std::fs;
/// Imports the child process a test keeps running while it reads the sentence for an installer that runs.
use std::process::{
    Child,
    Command,
    Stdio,
};

/// Imports the marker file whose text an earlier build wrote.
use super::archive_staging::COMPLETE_MARKER;
/// Imports the names a killed install leaves,
/// the purposes those names carry,
/// the owner facts a name records,
/// the write of a finished copy's marker,
/// and the start time a control reads for a process it started.
use super::archive_staging::{
    Owner,
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    STAGING_PURPOSE,
    hidden_sibling,
    start_time_of_a_process,
    write_complete_marker,
};
/// Imports the rig of a disposable home,
/// the fake network and the fake runner,
/// and the writer of an executable stand-in.
use super::fakes::{
    Rig,
    write_executable,
};
/// Imports the tool directory of a tool id.
use super::layout::tool_directory;
/// Imports the constants and the helpers the archive tests share:
/// the tool id,
/// the program its release holds,
/// a digest no lock vouches for,
/// the lock at one version,
/// the context of a run,
/// and the digest the lock records for a release.
use super::published_archive_tests::{
    A_TEST_SHA256,
    BIN,
    ID,
    context_of,
    lock_at,
    release_digest,
};
/// Imports the install itself,
/// which is the run whose report these tests read.
use super::sync::sync_tools;

/// What:
/// The owner facts of an installer that no process answers for:
/// the first process's number with a moment no process carrying it ever started at.
/// Why:
/// A hidden name whose installer has ended is what a later install judges and either publishes or removes,
/// and this pair is dead without waiting for a process to end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anInstallerThatNoLongerRuns(): Owner { return { pid: 1, started: startTimeOf(1)! + 1, boot: here.boot, namespace: here.namespace }; }
/// ```
fn an_installer_that_no_longer_runs() -> Owner {
    let here: Owner = Owner::here();
    let started: u64 =
        start_time_of_a_process(1).expect("the system reports the first process's start");
    // What: `started + 1` is a moment no process carrying that number started at, so the pair answers "ended".
    // Why: the pair must not name a live process, or nothing would be removed or published.
    return Owner {
        pid: 1,
        started: started + 1,
        boot: here.boot,
        namespace: here.namespace,
    };
}

/// What:
/// The owner facts of an installer in another process namespace of this boot.
/// Why:
/// A process id means another process in another namespace,
/// so this reader cannot judge such a name at all:
/// that is the state whose sentence these tests read,
/// and the name is built by hand because no test may make a namespace here.
/// This is a stand-in for a real install in a container sharing the toolchain root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anInstallerInAnotherNamespace(): Owner { return { ...here, namespace: here.namespace + 1 }; }
/// ```
fn an_installer_in_another_namespace() -> Owner {
    let here: Owner = Owner::here();
    let elsewhere: u64 = here
        .namespace
        .map(|space| return space + 1)
        .expect("the kernel reports this process namespace");
    return Owner {
        pid: here.pid,
        started: here.started,
        boot: here.boot,
        namespace: Some(elsewhere),
    };
}

/// What:
/// A program a test keeps running while it reads the sentence for an install that is still running,
/// ended and reaped when the test's body leaves.
/// Why:
/// A sentence about an installer that runs has to be read from a name whose process really runs,
/// and a child left behind by a failure would hold the machine's process count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RunningProgram { child = spawn('sleep', ['120']); owner(); [Symbol.dispose]() { this.child.kill(); } }
/// ```
struct RunningProgram {
    /// The program itself.
    child: Child,
}

/// The methods of `RunningProgram`.
impl RunningProgram {
    /// Starts a program that stays until the guard drops.
    fn start() -> RunningProgram {
        let child: Child = Command::new("sleep")
            .arg("120")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start a program that stays running");
        return RunningProgram {
            child,
        };
    }

    /// The owner facts of that program,
    /// in this reader's boot and namespace.
    fn owner(&self) -> Owner {
        let here: Owner = Owner::here();
        let pid: u32 = self.child.id();
        return Owner {
            pid,
            started: start_time_of_a_process(pid).expect("the system reports the program's start"),
            boot: here.boot,
            namespace: here.namespace,
        };
    }
}

/// Ends and reaps the program when the test's body leaves,
/// however it left.
impl Drop for RunningProgram {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

/// What:
/// Writes the copy an archive install publishes into `directory`:
/// the program inside the folder a release archive unpacks into.
/// Why:
/// A finished copy has to hold its program as an executable file,
/// or `is_finished` reads it as unfinished and no sentence about a finished copy is printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeTheProgram(directory: string): void { writeExecutable(join(directory, 'release-folder', BIN)); }
/// ```
fn write_the_program(directory: &std::path::Path) {
    // What: `Path::join` appends a path segment, and `write_executable` writes a program with its execute bits set.
    // Why: the program has to be found the way an unpacked release is found.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeExecutable(join(directory, 'release-folder', BIN));
    // ```
    write_executable(&directory.join("release-folder").join(BIN));
}

#[test]
fn the_publish_this_run_finished_names_the_copy_an_earlier_install_left() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The state a kill after the marker leaves: a finished copy under a dead installer's staging name.
    let finished: std::path::PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&finished);
    write_complete_marker(&finished, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    // Nothing is served: the version has to come from the copy the earlier install left.
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        rig.http.requests().is_empty(),
        "no download happens: {:?}",
        rig.http.requests()
    );
    assert_eq!(
        report.recovered,
        vec![format!(
            "finished the publish of {ID} 0.2.1 from the finished copy an earlier install left at {}",
            finished.display()
        )],
        "the sentence names the copy it read and claims no kill: {report:?}"
    );
}

#[test]
fn the_copy_left_for_a_name_nothing_can_judge_names_the_path_and_the_finished_copy_it_holds() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The copy a kill moved aside: an unfinished copy under a dead installer's name.
    let aside: std::path::PathBuf = hidden_sibling(
        &version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&aside);
    // A finished copy under a name of this boot whose installer this reader cannot see: a killed install
    // in another process namespace, built by hand because no test may make one here.
    let claim: std::path::PathBuf = hidden_sibling(
        &version_dir,
        RECOVERED_PURPOSE,
        an_installer_in_another_namespace(),
    );
    write_the_program(&claim);
    write_complete_marker(&claim, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "left the copy of {ID} 0.2.1 that an earlier install had moved aside where it is, \
             because the hidden name {} may belong to an install that is still running, \
             and nothing here can judge that install; \
             that name holds a finished copy of 0.2.1, \
             which nothing here publishes while that install cannot be judged",
            claim.display()
        )],
        "the sentence names the path that holds the wait and the finished copy it holds: {report:?}"
    );
    assert!(
        !version_dir.exists(),
        "nothing is put at the version's name: {:?}",
        fs::read_dir(&tool_dir)
            .expect("list the tool directory")
            .flatten()
            .map(|entry| return entry.file_name())
            .collect::<Vec<_>>()
    );
    assert!(aside.is_dir(), "the copy that was there waits");
    assert!(claim.is_dir(), "nothing of the unjudged name is touched");
    assert!(
        report
            .failed
            .iter()
            .any(|(_id, message)| return message.contains("download failed")),
        "the run says why the version is not there: {report:?}"
    );
}

#[test]
fn the_copy_left_for_an_install_that_runs_names_that_install_and_not_a_claim() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    let aside: std::path::PathBuf = hidden_sibling(
        &version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&aside);
    // An install of this version that is running and only unpacking: its staging name holds no finished copy.
    let running = RunningProgram::start();
    let unpacking: std::path::PathBuf =
        hidden_sibling(&version_dir, STAGING_PURPOSE, running.owner());
    write_the_program(&unpacking);

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "left the copy of {ID} 0.2.1 that an earlier install had moved aside where it is, \
             because an install that is still running holds the hidden name {}",
            unpacking.display()
        )],
        "the sentence says what was read: an install that runs, holding that name: {report:?}"
    );
    assert!(aside.is_dir(), "the copy that was there waits");
    assert!(
        unpacking.is_dir(),
        "nothing of the running install's name is touched"
    );
}

#[test]
fn the_removal_says_when_a_finished_copy_went_with_the_hidden_directory() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // A dead install's finished copy whose marker records a digest this lock does not: no install here publishes it.
    let unvouched: std::path::PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&unvouched);
    write_complete_marker(&unvouched, ID, "0.2.1", A_TEST_SHA256)
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "removed 1 hidden directory that a killed install of {ID} left behind, \
             1 of which held a finished copy that this run could not publish, \
             so that copy is gone"
        )],
        "the sentence says a finished copy went: {report:?}"
    );
    assert!(!unvouched.exists(), "the hidden directory is gone");
}

#[test]
fn an_install_whose_download_failed_says_where_a_finished_copy_it_cannot_judge_sits() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The name the release before this one wrote: the process id and its start time, and no boot or namespace.
    // Its marker records no archive digest either, so no install of this build publishes it.
    let earlier: std::path::PathBuf = tool_dir.join(format!(".0.2.1.{STAGING_PURPOSE}.42.100"));
    write_the_program(&earlier);
    fs::write(earlier.join(COMPLETE_MARKER), format!("{ID} 0.2.1\n"))
        .expect("write the marker an earlier build wrote");

    // Nothing is served, so the download fails with the finished copy still on disk.
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(report.failed.len(), 1, "{report:?}");
    let message: &str = &report.failed[0].1;
    assert!(
        message.contains("download failed"),
        "the failure still says what failed: {message}"
    );
    assert!(
        message.contains(&format!(
            "a finished copy of {ID} 0.2.1 sits under the hidden name {}",
            earlier.display()
        )),
        "the failure names the copy and its path: {message}"
    );
    assert!(
        message.contains("whose installer this build cannot judge"),
        "the failure says why nothing publishes it: {message}"
    );
    assert!(
        message.contains(&format!(
            "by moving that copy to {} by hand",
            version_dir.display()
        )),
        "the failure names the way an offline machine gets the version back: {message}"
    );
}
