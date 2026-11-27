//! What:
//!  Disposable fixtures and bounded process helpers for binary-level controls.
//! Why:
//!  Every control runs the built executable through a `git`-named link on a
//!      fixture PATH,
//!  with no real home or global Git configuration,
//!  and a hard time
//!      bound so a forwarding loop fails the test instead of hanging it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const fixture = await mkdtemp(...); await symlink(wrapper, join(fixture, 'bin/git'));
//! ```

/// Native string,
///  path and process types used by the helpers.
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Output,
    Stdio,
};

/// The helpers that moved to files of their own,
/// named here so every `support::name` path stays the same.
pub use crate::support_btrfs_controls::*;
pub use crate::support_first_physical_extent::*;
pub use crate::support_fixture_directories::*;
pub use crate::support_fixture_service::*;
pub use crate::support_fixture_service_log::*;
pub use crate::support_fixture_service_readiness::*;
pub use crate::support_native_git_command::*;
pub use crate::support_observation::*;
pub use crate::support_package_filling::*;
pub use crate::support_pipes_and_pseudo_terminals::*;
pub use crate::support_process_cpu_counters::*;
pub use crate::support_process_group_members::*;
pub use crate::support_program_names_and_time_bound::*;
pub use crate::support_reflink_controls::*;
pub use crate::support_resource_limits::*;
pub use crate::support_running_a_command_and_reading_the_answer::*;
pub use crate::support_scratch_left_behind::*;
pub use crate::support_zfs_controls::*;

/// Apply the hermetic environment every wrapped and direct run shares.
pub fn hermetic(
    command: &mut Command,
    fixture: &Fixture,
    path: &OsStr,
) {
    command
        .env_clear()
        .env("PATH", path)
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        // No run reaches the host's journal: the entries the wrapper sends go to an abstract name nobody listens at, unless a
        //  control binds the name it passes itself.
        .env(
            git_policy_cli::journald_native::TEST_SOCKET_VARIABLE,
            "@cli-git-test-journal-nobody-hermetic",
        )
        .current_dir(&fixture.root);
    // A developer who sets the wrapper's verbose switch for a run gets its debug log in every wrapped command's
    //  output; no run sets it otherwise, so the environment stays hermetic.
    if let Some(verbose) = std::env::var_os("MONOCHROMATIC_VERBOSE") {
        command.env("MONOCHROMATIC_VERBOSE", verbose);
    }
}

/// The PATH most controls use:
///  the wrapper's link directory ahead of real Git.
pub fn wrapper_first_path(fixture: &Fixture) -> OsString {
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    return path;
}

/// What:
///  A `PATH` that names one new fixture folder and nothing else,
///  holding a link for each program the caller names.
/// Why:
///   A control that proves what the wrapper says when a program is absent must not name a folder the host or the gate
/// image installs it in:
/// `/usr/bin` holds a real `git-lfs` on a developer's machine and in the gate image,
/// so the search found it there and the control failed wherever the program was installed (L5 of the alpha.45 release
/// pass).
/// Naming a folder of the fixture's own keeps the control the same on every machine,
/// and the programs the push itself needs are linked into it:
/// a link of the real Git,
/// never of the wrapper,
/// because the wrapper passes over every candidate that leads back to itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = onlyThesePrograms(fixture, 'push-needs', [['git', '/usr/bin/git'], ['sh', '/bin/sh']]);
/// ```
pub fn path_of_a_folder_that_links_only(
    fixture: &Fixture,
    folder: &str,
    programs: &[(&str, &Path)],
) -> OsString {
    let directory: PathBuf = fixture.root.join(folder);
    std::fs::create_dir_all(directory.as_path()).expect("the folder of linked programs");
    for (name, program) in programs {
        std::os::unix::fs::symlink(program, directory.join(name)).expect("a linked program");
    }
    return directory.into_os_string();
}

/// A command for `program` (a wrapper path) under the given PATH,
///  killed after `seconds`.
pub fn bounded_for(
    fixture: &Fixture,
    program: &Path,
    path: &OsStr,
    seconds: &str,
) -> Command {
    if seconds == TIME_BOUND_SECONDS {
        // The shared bound is the supervisor's; its thread finds the command among the test's children once it runs.
        super::supervisor::ensure_monitor();
    }
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(seconds).arg(program);
    hermetic(&mut command, fixture, path);
    if let Some(runtime) = service_runtime(fixture) {
        command.env("XDG_RUNTIME_DIR", runtime);
    }
    // The place a waiting commit is told is a line a healthy commit must not print, so it is set past the bound that
    //  ends this command: a control that checks the line itself sets its own.
    command.env(
        git_policy_cli::wait_ceilings::WAITING_NOTICE_VARIABLE,
        seconds,
    );
    return command;
}

/// A time-bounded command for `program` (a wrapper path) under the given PATH.
pub fn bounded(
    fixture: &Fixture,
    program: &Path,
    path: &OsStr,
) -> Command {
    return bounded_for(fixture, program, path, TIME_BOUND_SECONDS);
}

/// What:
///  Put a `man` in the fixture's `bin` directory that prints one marker line naming the page and exits 0.
/// Why:
///  Git renders `--help` by running `man`,
/// which starts `preconv`,
/// `tbl`,
/// `nroff`,
/// `col`,
/// `groff`,
/// `troff` and
///  `grotty` (11 process starts in all,
/// 18 ms warm and 0.7 s cold on the development host,
/// 3.2 s once under load),
///  none of it the wrapper's work.
/// Git looks `man` up on the command's `PATH`,
/// whose first entry is this directory,
///  so the wrapper still receives and forwards the real `--help` and Git still takes its manual-page path.
pub fn stub_manual_viewer(fixture: &Fixture) {
    executable(
        fixture.root.join("bin/man").as_path(),
        b"#!/bin/sh\necho \"manual page requested: $*\"\n",
    );
}

/// Expose real Git at `<root>/real/git`,
///  a location that is not a conventional one.
/// Conventional locations are promoted to the front of the candidate list,
///  which would
/// hide whether earlier PATH entries were examined and skipped.
pub fn link_real_git(fixture: &Fixture) {
    std::fs::create_dir(fixture.root.join("real")).expect("real directory");
    std::os::unix::fs::symlink(REAL_GIT, fixture.root.join("real/git")).expect("real Git link");
}

/// A time-bounded command for the fixture's `bin/git` link with the wrapper first on PATH.
pub fn wrapped(fixture: &Fixture) -> Command {
    return bounded(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
    );
}

/// What:
///  Make the in-process object readers decline the repository,
/// with a replace ref on two blobs nothing uses.
/// Why:
///   The controls with a stand-in `git-lfs` check what the wrapper does around the program it starts (order,
/// status,
///  explanation,
/// deadlines).
/// The wrapper starts the program only when it finds a pointer in the pushed commits or when
///  the readers decline and `git-lfs` has to scan;
/// these repositories hold no pointer,
/// so they take the second path,
///  which is the fallback the readers' declines lead to in production.
pub fn decline_in_process_lfs_scan(
    fixture: &Fixture,
    repo: &Path,
) {
    let mut hashes: Vec<String> = Vec::new();
    for (name, text) in [("replace-from.txt", "from\n"), ("replace-to.txt", "to\n")] {
        let file: PathBuf = fixture.root.join(name);
        std::fs::write(&file, text).expect("blob source");
        let mut hashing: Command = direct(fixture);
        hashing.args(["hash-object", "-w"]).arg(&file);
        let output: Output =
            native_git_output(fixture, &mut hashing, repo, NATIVE_GIT_BOUND_SECONDS);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        hashes.push(
            String::from_utf8_lossy(output.stdout.as_slice())
                .trim()
                .to_owned(),
        );
    }
    let mut replacing: Command = direct(fixture);
    replacing.args(["replace", hashes[0].as_str(), hashes[1].as_str()]);
    let replaced: Output =
        native_git_output(fixture, &mut replacing, repo, NATIVE_GIT_BOUND_SECONDS);
    assert!(
        replaced.status.success(),
        "{}",
        String::from_utf8_lossy(replaced.stderr.as_slice())
    );
}

/// What:
///  Gives the fixture's commit service a variable,
/// as the user manager's environment would.
/// Why:
///   The push runs in the service's environment,
/// not in the terminal's,
/// so a control that sets a variable for the push (a
///  deadline,
/// a marker,
/// a temporary directory) must set it where the push runs.
/// The service reads it at its start,
/// so it is
///  given before the fixture's first wrapped command.
///
/// # Panics
///
/// When the service already started with another value of the variable.
pub fn give_service_environment(
    fixture: &Fixture,
    name: &str,
    value: &str,
) {
    // The service's lock first and the environment's second: `service_runtime` holds the service's lock while it starts a service,
    //  and the start reads the environment, so the other order let two threads wait for each other for ever.
    let service = fixture.service.lock().expect("the fixture's service lock");
    let mut given = fixture
        .service_environment
        .lock()
        .expect("the service environment");
    let present = service.is_some();
    let already: bool = given.iter().any(|(held, text)| {
        return held == name && text == value;
    });
    assert!(
        already || !present,
        "the fixture's service already started without {name}={value}"
    );
    if !already {
        given.push((OsString::from(name), OsString::from(value)));
    }
}

/// The words a notice of a backup push that did not finish carries.
///  A push the network,
/// the LFS server,
/// a stall or a held lock stopped can pass by itself,
/// so the service
///  keeps it in its retry plan and the person is told that the remote may or may not have the commit;
/// only a conflict,
/// a login
///  the service does not have or a hook on the server that declined it stops the retry,
/// and only that wording says the push
///  failed for good.
pub fn push_unfinished_words() -> &'static str {
    return git_policy_cli::push::push_failure_notice_words::PUSH_UNFINISHED_NOTE;
}

/// What a person is told about the push of the commit that `stderr` belongs to:
/// the command's own report,
/// and the notices the
///  service left in the repository at `directory`,
/// worded as the next command shows them (and taken as it takes them).
pub fn push_report(
    directory: &Path,
    stderr: &str,
) -> String {
    let mut report: String = stderr.to_string();
    settle_service_work(directory);
    let git_dir: Option<PathBuf> = directory
        .ancestors()
        .map(|candidate| {
            return candidate.join(".git");
        })
        .find(|candidate| {
            return candidate.is_dir();
        });
    if let Some(found) = git_dir {
        // The notices are worded the way the next command in the worktree shows them, and are shown and removed as that
        //  command does.
        for notice in git_policy_cli::push_failure_notice::push_failures(found.as_path()) {
            let mut shown: Vec<u8> = Vec::new();
            let _written = notice.show(&mut shown);
            report.push_str(String::from_utf8_lossy(shown.as_slice()).as_ref());
        }
    }
    return report;
}

/// An editor that records that it started and then waits for a minute,
/// so a commit that opens it sits inside native
///  `git commit` until a signal arrives.
/// Hooks are unsupported,
/// so controls that need the commit to stand still use this.
///
/// The editor catches the signals the controls send and ends with the signal's status,
/// so no timing decides the
///  verdict.
/// A shell that waits for a foreground child ignores `SIGINT` and `SIGQUIT` unless that child dies of the
///  same signal,
/// and a shell that is about to `exec` can lose one too (measured:
/// the editor with `touch` then
///  `sleep` lost an interrupt,
/// and with `exec sleep` it lost a quit,
/// at the instant after it announced itself).
///  The editor then slept its full minute and ended the commit with status 1 and "Aborting commit due to empty
///  commit message."
/// instead of the signal's status.
/// The handlers are set before the announcement,
/// so every signal
///  that can follow the announcement is caught;
/// `wait` returns at once on a caught signal;
/// the sleep has no
///  terminal and no pipe of the test,
/// so a sleep left behind cannot hold the commit's standard error open.
pub fn stalling_editor(
    fixture: &Fixture,
    started: &Path,
) -> PathBuf {
    let path: PathBuf = fixture.root.join("stalling-editor");
    executable(
        path.as_path(),
        format!(
            "#!/bin/sh\nstall=\ntrap 'kill $stall 2>/dev/null; exit 130' INT\ntrap 'kill $stall 2>/dev/null; exit 131' QUIT\nprintf started > '{}'\nsleep 60 </dev/null >/dev/null 2>&1 &\nstall=$!\nwait $stall\n",
            started.display()
        )
        .as_bytes(),
    );
    return path;
}

/// The transaction ID registry fixtures use.
pub const TRANSACTION_ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// What:
///  Leave a commit transaction directory of an earlier release in a repository's registry.
/// Why:
///   The wrapper no longer reads such a directory;
/// a control that proves it goes on beside one,
/// and only names it,
/// needs one there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(join(registry, transactionId), { recursive: true });
/// ```
pub fn registered_transaction(repo: &Path) -> PathBuf {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    std::fs::create_dir_all(registry.join(TRANSACTION_ID))
        .expect("a transaction directory an earlier release left");
    return registry;
}

/// What:
///  The owner of a process that really started,
/// had its birth identity read while it ran,
/// and was then killed and reaped.
/// Why:
///   An interrupted copy is a journal or a claim whose owner is gone;
/// a control builds one with this owner instead of
///  waiting for a real crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dead = await deadCopyOwner(); // { pid, birthIdentity, pidNamespace } of a killed child
/// ```
pub fn dead_copy_owner() -> git_policy_cli::worktree_copy_owner::CopyOwner {
    let mut child: Child = Command::new("sleep")
        .arg("60")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep starts");
    let pid: i64 = i64::from(child.id());
    let birth: String = git_policy_cli::process_identity::process_birth_identity(pid)
        .expect("identity readable")
        .expect("the child runs");
    child.kill().expect("child is killed");
    child.wait().expect("child is reaped");
    return git_policy_cli::worktree_copy_owner::CopyOwner {
        pid,
        birth_identity: birth,
        pid_namespace: git_policy_cli::pid_namespace::current_pid_namespace(),
    };
}
