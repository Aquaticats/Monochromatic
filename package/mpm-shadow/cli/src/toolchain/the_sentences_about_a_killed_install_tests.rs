//! Unit tests of the sentences an install prints about the copies an earlier install left:
//! the publish it finished,
//! the copy it left beside a hidden name whose installer may still run,
//! the copy it dropped for another install's publish,
//! the copy it put back,
//! the finished copy it left hidden because an older repository tool is never kept,
//! the hidden directory it removed with a finished copy inside,
//! whether that copy was a second one or the one the version had,
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
/// Imports the names an install that ended leaves,
/// the purposes those names carry,
/// the owner facts a name records,
/// the write of a finished copy's marker,
/// the start time a control reads for a process it started,
/// and the plant a control arms so another install publishes the version in the instant this run does.
use super::archive_staging::{
    Owner,
    Plant,
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    STAGING_PURPOSE,
    clear_the_plant,
    hidden_sibling,
    plant_these_steps,
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
/// Imports the request a lock's row makes,
/// which the test of a tool no program name finishes rewrites.
use super::model::Request;
/// Imports the question that tells a finished install from an unfinished one.
use super::published_archive::is_finished;
/// Imports the constants and the helpers the archive tests share:
/// the tool id,
/// the program its release holds,
/// a digest no lock vouches for,
/// the lock at one version,
/// the context of a run,
/// the digest the lock records for a release,
/// the finish of an install made by hand,
/// and the fake network serving releases an install downloads.
use super::published_archive_tests::{
    A_TEST_SHA256,
    BIN,
    ID,
    context_of,
    finish_archive,
    lock_at,
    release_digest,
    release_url,
    serve_releases,
    unmarked_archive,
};
/// Imports the install itself,
/// which is the run whose report these tests read,
/// and the report it answers with.
use super::sync::{
    SyncReport,
    sync_tools,
};
/// Imports the text of a directory's own name,
/// which the planted publish reads its version from.
use super::util::file_name_text;

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

/// What:
/// A finished copy of the locked version at the version's name,
/// written as another install publishes it while this run stands before its own rename.
/// Why:
/// The sentence for a copy this run dropped is printed only when another install gave the version its name first,
/// and no disposable fixture can time that;
/// the plant makes the instant exact instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anotherInstallPublishesTheVersion(staging: string, versionDir: string): void {
///   writeExecutable(join(versionDir, 'release-folder', BIN));
///   writeCompleteMarker(versionDir, ID, basename(versionDir), A_TEST_SHA256);
/// }
/// ```
fn another_install_publishes_the_version(
    _staging: &std::path::Path,
    version_dir: &std::path::Path,
) {
    // What: the program and then the marker, the order an install writes them in.
    // Why: another install's copy is only a finished one when the run finds both.
    write_the_program(version_dir);
    write_complete_marker(version_dir, ID, &file_name_text(version_dir), A_TEST_SHA256)
        .expect("write the completion marker");
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
fn the_removal_says_the_copy_went_because_no_finished_copy_is_at_the_version_name() {
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
            "removed 1 hidden directory left behind by an install of {ID} that has ended, \
             1 of which held a finished copy of 0.2.1 that this run did not publish, \
             because that copy's completion marker does not record the archive this lock names; \
             no finished copy of 0.2.1 is at its name, so that copy is gone"
        )],
        "the sentence says the copy went and no finished copy is at the version's name: {report:?}"
    );
    assert!(!unvouched.exists(), "the hidden directory is gone");
    assert!(
        !version_dir.exists(),
        "the sentence says it, and this run did not put a copy there: {report:?}"
    );
}

#[test]
fn the_removal_says_a_second_copy_went_when_the_version_is_finished_at_its_name() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The version itself, installed by an earlier run of this package from its own lock, so its marker records the
    // archive the lock names.
    serve_releases(&rig, &["0.2.1"]);
    let installing: SyncReport =
        sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(
        installing.failed.is_empty() && is_finished(&version_dir, BIN),
        "the version is installed before the run this test reads: {installing:?}"
    );
    // And a second finished copy of that same version, the one a recovery claimed and left behind, whose marker
    // records the lock's row as a finished copy does.
    let second: std::path::PathBuf = hidden_sibling(
        &version_dir,
        RECOVERED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&second);
    write_complete_marker(&second, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "removed 1 hidden directory left behind by an install of {ID} that has ended, \
             1 of which held a second finished copy of 0.2.1; \
             0.2.1 holds its completion marker and its program at its name, so that copy was a second one"
        )],
        "the sentence says what went and what it read at the version's name: {report:?}"
    );
    assert!(
        is_finished(&version_dir, BIN),
        "the version keeps the finished copy it had: {report:?}"
    );
    assert!(!second.exists(), "the second finished copy is gone");
}

#[test]
fn the_copy_put_back_names_an_earlier_install_and_not_a_kill() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The copy an install moved aside and left there when it did not finish: unfinished, as the copy that was there
    // is before a replace step moves it.
    let aside: std::path::PathBuf = hidden_sibling(
        &version_dir,
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&aside);

    // Nothing is served, so the install after the recovery cannot download: the sentence is the report's one line.
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "put back the copy of {ID} 0.2.1 that an earlier install had moved aside"
        )],
        "the sentence names an earlier install, which is what the name it read carries: {report:?}"
    );
    assert!(
        version_dir.is_dir(),
        "the copy that was there is back at the version's name: {report:?}"
    );
}

#[test]
fn the_copy_dropped_for_another_publish_names_an_earlier_install_and_not_a_kill() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The finished copy a kill after the marker leaves, whose marker records the lock's own row.
    let finished: std::path::PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&finished);
    write_complete_marker(&finished, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");
    // What: another install gives the version its name while this run stands before its own rename.
    // Why: only that makes this run drop the copy it holds, and no fixture can time it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plantTheseSteps({ anotherInstallActs: anotherInstallPublishesTheVersion });
    // ```
    plant_these_steps(Plant {
        another_install_acts: Some(another_install_publishes_the_version),
        ..Plant::default()
    });

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    clear_the_plant();

    assert_eq!(
        report.recovered,
        vec![format!(
            "dropped the copy of {ID} 0.2.1 that an earlier install left, \
             because another install published that version meanwhile"
        )],
        "the sentence names an earlier install and the publish that took the version's name: {report:?}"
    );
    assert!(
        is_finished(&version_dir, BIN),
        "another install's publish is what the version's name holds: {report:?}"
    );
}

#[test]
fn the_finished_copy_left_hidden_names_an_earlier_install_and_not_a_kill() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // A newer version of this repository's tool is installed, so an older one is never kept.
    finish_archive(&root, "0.2.2");
    let finished: std::path::PathBuf = hidden_sibling(
        &version_dir,
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&finished);
    write_complete_marker(&finished, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "left the finished copy of {ID} 0.2.1 under the hidden name an earlier install left it at, \
             because 0.2.2 is installed and an older repository tool is never kept"
        )],
        "the sentence names an earlier install, which is what the name it read carries: {report:?}"
    );
    assert!(
        is_finished(&finished, BIN),
        "the copy stays hidden where a later install publishes it: {report:?}"
    );
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
            "or by copying that copy's contents into {} by hand",
            version_dir.display()
        )),
        "the failure names the way an offline machine gets the version back: {message}"
    );
}

/// What:
/// The one failure a run printed for the tool these tests install.
/// Why:
/// A failed download's sentence is part of that message,
/// so a test reads it from the report the run answered with.
fn the_tools_failure(report: &SyncReport) -> &str {
    let found = report
        .failed
        .iter()
        .find(|(id, _message)| return id == ID);
    return found
        .map(|(_id, message)| return message.as_str())
        .expect("the run reports the tool's failure");
}

/// What:
/// The finished copy an earlier build's install left under a name of its own:
/// the process id and its start time, no boot and no namespace, and a marker that records no archive digest.
/// Why:
/// No run of this build judges such a name or publishes its copy,
/// which is the state a failed download's sentence and the `prune` line are printed in.
fn write_the_earlier_builds_copy(
    tool_dir: &std::path::Path,
    version: &str,
) -> std::path::PathBuf {
    let earlier: std::path::PathBuf = tool_dir.join(format!(".{version}.{STAGING_PURPOSE}.42.100"));
    write_the_program(&earlier);
    fs::write(earlier.join(COMPLETE_MARKER), format!("{ID} {version}\n"))
        .expect("write the marker an earlier build wrote");
    return earlier;
}

/// What:
/// Runs the step a failed download's sentence names for a person,
/// as the sentence prints it: `cp --archive <copy>/. <version dir>/`.
/// Why:
/// The sentence is advice, so a test follows it instead of rewriting it,
/// and the step has to work whether the version's name holds nothing or an unfinished copy.
fn copy_the_contents_where_the_sentence_says(
    copy: &std::path::Path,
    version_dir: &std::path::Path,
) {
    let copied = Command::new("cp")
        .arg("--archive")
        .arg(copy.join("."))
        .arg(version_dir)
        .status()
        .expect("run cp");
    assert!(copied.success(), "the step the sentence names copies the copy's contents");
}

#[test]
fn a_failed_listing_names_the_hidden_names_an_earlier_install_left() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    fs::create_dir_all(&root).expect("make the toolchain root");
    // A listing that fails, stood in here by a path that is a regular file: `read_dir` answers `Not a directory`,
    // and every failed listing reaches the one sentence this test reads.
    fs::write(&tool_dir, "not a directory").expect("make the tool's path a file");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains(&format!(
            "cannot list {} for the hidden names an earlier install left",
            tool_dir.display()
        )),
        "the sentence says what the run read: {message}"
    );
    assert!(
        !message.contains("killed"),
        "the sentence names no kill the run did not read: {message}"
    );
    assert!(
        message.contains("the install that followed it also failed"),
        "the failure still says what stopped the install: {message}"
    );
}

#[test]
fn the_removal_says_only_what_it_read_at_the_version_name() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The version installed the way an install makes it, so a finished copy stands at its name.
    serve_releases(&rig, &["0.2.1"]);
    let installing: SyncReport =
        sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(
        installing.failed.is_empty() && is_finished(&version_dir, BIN),
        "the version is installed before the run this test reads: {installing:?}"
    );
    // The program at that name cut to twelve bytes under its marker: `is_finished` reads the marker and a program
    // found under the directory, and no size and no digest, so this run reads the version as finished.
    write_the_program(&version_dir);
    let program: std::path::PathBuf = version_dir.join("release-folder").join(BIN);
    fs::write(&program, "#!/bin/sh\nec").expect("cut the program short");
    // And a second finished copy of that same version, under an installer the system reports ended.
    let second: std::path::PathBuf = hidden_sibling(
        &version_dir,
        RECOVERED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&second);
    write_complete_marker(&second, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "removed 1 hidden directory left behind by an install of {ID} that has ended, \
             1 of which held a second finished copy of 0.2.1; \
             0.2.1 holds its completion marker and its program at its name, so that copy was a second one"
        )],
        "the sentence says what went and only what the run read at the version's name: {report:?}"
    );
    assert!(
        is_finished(&version_dir, BIN),
        "what the run read at the version's name is what it read: {report:?}"
    );
    assert!(!second.exists(), "the second finished copy is gone");
}

#[test]
fn the_removal_says_a_second_copy_went_when_no_program_name_is_known() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // A tool this repository does not publish names no program, so a copy finishes on its marker alone.
    let mut lock = lock_at(&rig, "0.2.1");
    lock.tools[0].declaration.request =
        Request::parse("github:example/other@latest").expect("parses");
    lock.tools[0].declaration.options.tag_prefix = None;
    finish_archive(&root, "0.2.1");
    let second: std::path::PathBuf = hidden_sibling(
        &version_dir,
        RECOVERED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_the_program(&second);
    write_complete_marker(&second, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock), None);

    assert_eq!(
        report.recovered,
        vec![format!(
            "removed 1 hidden directory left behind by an install of {ID} that has ended, \
             1 of which held a second finished copy of 0.2.1; \
             0.2.1 holds its completion marker at its name, so that copy was a second one"
        )],
        "the sentence names the marker alone when no program name is known: {report:?}"
    );
    assert!(!second.exists(), "the second finished copy is gone");
}

#[test]
fn the_removal_says_the_copy_went_when_an_unfinished_copy_stands_at_the_version_name() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // A directory at the version's name that is not finished: the program and no marker, which is what a kill
    // during the unpacking leaves.
    unmarked_archive(&root, "0.2.1");
    // A dead install's finished copy whose marker records a digest this lock does not, so nothing publishes it.
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
            "removed 1 hidden directory left behind by an install of {ID} that has ended, \
             1 of which held a finished copy of 0.2.1 that this run did not publish, \
             because that copy's completion marker does not record the archive this lock names; \
             no finished copy of 0.2.1 is at its name, so that copy is gone"
        )],
        "the sentence says the copy went and that no finished copy is at the version's name: {report:?}"
    );
    assert!(
        version_dir.is_dir() && !is_finished(&version_dir, BIN),
        "the copy at the version's name is the unfinished one: {report:?}"
    );
}

#[test]
fn the_move_a_failed_download_names_puts_the_copy_at_a_version_name_that_holds_nothing() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    let earlier: std::path::PathBuf = write_the_earlier_builds_copy(&tool_dir, "0.2.1");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains(&format!(
            "or by copying that copy's contents into {} by hand",
            version_dir.display()
        )),
        "the failure says what step brings the version back: {message}"
    );
    // The step the sentence prints, run as a person runs it.
    copy_the_contents_where_the_sentence_says(&earlier, &version_dir);
    assert!(
        is_finished(&version_dir, BIN),
        "the step the sentence names leaves a finished copy at the version's name"
    );

    let after: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(after.failed.is_empty(), "{after:?}");
    assert_eq!(
        after.unchanged,
        vec![ID.to_string()],
        "the version the step put there is the one the lock asks for: {after:?}"
    );
}

#[test]
fn the_move_a_failed_download_names_reaches_a_version_name_that_holds_an_unfinished_copy() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // The version's name already holds a directory: a kill after the unpacking and before the marker.
    unmarked_archive(&root, "0.2.1");
    let earlier: std::path::PathBuf = write_the_earlier_builds_copy(&tool_dir, "0.2.1");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains(&format!(
            "or by copying that copy's contents into {} by hand",
            version_dir.display()
        )),
        "the failure says what step brings the version back: {message}"
    );
    copy_the_contents_where_the_sentence_says(&earlier, &version_dir);

    let after: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);
    assert!(after.failed.is_empty(), "{after:?}");
    assert_eq!(
        after.unchanged,
        vec![ID.to_string()],
        "the version the step put there is the one the lock asks for: {after:?}"
    );
}

#[test]
fn a_download_that_does_not_match_the_lock_says_what_brings_the_version_back() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    let earlier: std::path::PathBuf = write_the_earlier_builds_copy(&tool_dir, "0.2.1");
    // The network is there and serves bytes the lock does not name, so the download cannot bring the version back
    // until the bytes and the lock agree.
    rig.http
        .serve(&release_url("0.2.1"), b"the bytes the lock does not name");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains("SHA-256 mismatch"),
        "the failure is the digest's: {message}"
    );
    assert!(
        message.contains(
            "the version comes back from a download that matches this lock's SHA-256 and that the install can keep"
        ),
        "the sentence says what a download must meet: {message}"
    );
    assert!(
        !message.contains("with the network there"),
        "the network was there and did not bring the version back: {message}"
    );
    assert!(
        message.contains(&format!(
            "sits under the hidden name {}",
            earlier.display()
        )),
        "the sentence still names the copy a person can move: {message}"
    );
}

#[test]
fn a_finished_copy_of_another_version_is_not_named_for_a_person_to_move() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    // The only copy this build cannot judge is another version's, which that version's own install publishes.
    write_the_earlier_builds_copy(&tool_dir, "0.2.2");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains("download failed"),
        "the failure is the download's: {message}"
    );
    assert!(
        !message.contains("sits under the hidden name"),
        "another version's copy is not named for this version: {message}"
    );
}

#[test]
fn a_running_installs_finished_copy_is_not_named_for_a_person_to_move() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    let version_dir = tool_dir.join("0.2.1");
    // A finished copy whose installer the system reports running: its own publish is still to come.
    let running = RunningProgram::start();
    let held: std::path::PathBuf = hidden_sibling(&version_dir, STAGING_PURPOSE, running.owner());
    write_the_program(&held);
    write_complete_marker(&held, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains("download failed"),
        "the failure is the download's: {message}"
    );
    assert!(
        !message.contains("sits under the hidden name"),
        "a running install's copy is not named for a person to move: {message}"
    );
}

#[test]
fn a_name_that_holds_no_marker_is_not_named_for_a_person_to_move() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir = tool_directory(&root, ID).expect("a plain name");
    // A name nothing can judge, holding an unpacking that never finished: no copy to move.
    let unfinished: std::path::PathBuf =
        tool_dir.join(format!(".0.2.1.{STAGING_PURPOSE}.42.100"));
    write_the_program(&unfinished);

    let report = sync_tools(&context_of(&rig, &rig.scope, &lock_at(&rig, "0.2.1")), None);

    let message: &str = the_tools_failure(&report);
    assert!(
        message.contains("download failed"),
        "the failure is the download's: {message}"
    );
    assert!(
        !message.contains("sits under the hidden name"),
        "a name that holds no marker is not named as a finished copy: {message}"
    );
}
