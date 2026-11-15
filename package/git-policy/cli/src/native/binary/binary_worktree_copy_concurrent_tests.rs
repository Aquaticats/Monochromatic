//! What:
//!  Worktree commands beside a `git worktree add` whose real Git is still running:
//! another add,
//! a move,
//! a removal and a
//!  conversion each finish without waiting for it,
//! and each new worktree receives exactly its own ignored-state copy.
//! A command of one worktree also says nothing about a lock a live program holds in another worktree's
//!  administrative directory,
//!  which is that worktree's own write.
//! Why:
//!  The owner decided on 2026-11-03 that locks are used as little as possible,
//! that the wrapper almost never waits,
//! and that
//!  embarrassingly parallel work is not serialized.
//! One add used to wait 228 s,
//! and another over 20 minutes,
//! for the
//!  per-repository settlement lock another session's add held across its real Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = spawn(wrapped, ['worktree', 'add', '../held']); // its smudge filter waits for a release file
//! expect(run(wrapped, ['worktree', 'add', '../free']).status).toBe(0); // finishes while `held` still runs
//! ```

use super::support::{
    Fixture, Observed, bounded_for, executable, fixture, git, remove, run_wrapped, stderr_of, wrapper_first_path,
};
use super::worktree_copy::repository_with_rules;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime};

/// Where the generated rules file lives,
/// as the repository's configuration names it.
const RULES_FILE: &str = ".cache/forbidden-strings.rules.txt";

/// The wall-clock bound,
/// in seconds,
/// of a command a control keeps running on purpose while others finish.
const HELD_BOUND_SECONDS: &str = "90";

/// How long a control waits for a condition another process creates before it names the stall.
const CONDITION_PATIENCE: Duration = Duration::from_secs(30);

/// How often a wait for a condition looks again.
const CONDITION_POLL: Duration = Duration::from_millis(50);

/// The destination whose smudge filter stalls until the release file appears.
const HELD_NAME: &str = "held";

/// How many tenths of a second the stalled filter looks for the release file before it gives up on its own,
/// so a control
/// that failed or was killed leaves no Git behind.
const FILTER_POLLS: u32 = 600;

/// A fixture repository whose smudge filter stalls the checkout of the worktree named `held` until `release` exists.
/// The untracked file leaves the checkout to Git on every host,
/// so the filter really runs.
struct Held {
    fixture: Fixture,
    repo: PathBuf,
    child: Option<Child>,
}

impl Held {
    /// The repository,
    /// its filter,
    /// and nothing running yet.
    fn new(name: &str) -> Held {
        let fixture: Fixture = fixture(name);
        let repo: PathBuf = repository_with_rules(&fixture, true);
        let filter: PathBuf = fixture.root.join("smudge-filter");
        executable(
            filter.as_path(),
            format!(
                "#!/bin/sh\ncat\ncase \"$PWD\" in\n  */{HELD_NAME})\n    printf started > ../held-started\n    waited=0\n    while [ ! -e ../release ] && [ -e ../held-started ] && [ $waited -lt {FILTER_POLLS} ]; do sleep 0.1; waited=$((waited + 1)); done\n    ;;\nesac\n"
            )
            .as_bytes(),
        );
        std::fs::write(repo.join(".gitattributes"), b"*.probe filter=probe\n").expect("attributes");
        std::fs::write(repo.join("x.probe"), b"probe\n").expect("probe file");
        git(&fixture, repo.as_path(), &["add", "--", ".gitattributes", "x.probe"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=probe"]);
        git(&fixture, repo.as_path(), &["config", "filter.probe.smudge", filter.to_str().expect("UTF-8")]);
        std::fs::write(repo.join("untracked-so-git-checks-out.txt"), b"u\n").expect("untracked file");
        return Held { fixture, repo, child: None };
    }

    /// A wrapped command that may outlive the shared bound while the others run.
    fn command(&self, directory: &Path, arguments: &[&str]) -> Command {
        let mut command: Command = bounded_for(
            &self.fixture,
            self.fixture.root.join("bin/git").as_path(),
            wrapper_first_path(&self.fixture).as_os_str(),
            HELD_BOUND_SECONDS,
        );
        command.current_dir(directory).args(arguments);
        return command;
    }

    /// Run a command to its end under the held bound and return what the caller saw.
    fn run(&self, directory: &Path, arguments: &[&str]) -> Observed {
        let output: Output = self
            .command(directory, arguments)
            .stdin(Stdio::null())
            .output()
            .expect("the command starts");
        return Observed { code: output.status.code(), stdout: output.stdout, stderr: output.stderr };
    }

    /// Start the add of `held` and wait until Git's checkout reached the stalled filter.
    fn start_held_add(&mut self) {
        let child: Child = self
            .command(self.repo.as_path(), &["worktree", "add", "--quiet", "-b", HELD_NAME, "../held"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the held add starts");
        self.child = Some(child);
        self.wait_for(&self.fixture.root.join("held-started"));
    }

    /// Wait until `path` exists,
    /// naming the stall when it never does.
    fn wait_for(&self, path: &Path) {
        let started: Instant = Instant::now();
        while !path.exists() {
            assert!(started.elapsed() < CONDITION_PATIENCE, "{} never appeared", path.display());
            std::thread::sleep(CONDITION_POLL);
        }
    }

    /// Whether the held add is still running.
    fn held_is_running(&mut self) -> bool {
        let child: &mut Child = self.child.as_mut().expect("the held add was started");
        return child.try_wait().expect("the held add can be polled").is_none();
    }

    /// Let the filter go and collect the held add's result.
    fn release_and_finish(&mut self) -> Observed {
        std::fs::write(self.fixture.root.join("release"), b"go").expect("release file");
        let child: Child = self.child.take().expect("the held add was started");
        let output: Output = child.wait_with_output().expect("the held add finishes");
        return Observed { code: output.status.code(), stdout: output.stdout, stderr: output.stderr };
    }
}

impl Drop for Held {
    /// A failed control must not leave the filter waiting:
    /// the release lets the held Git end.
    fn drop(&mut self) {
        let _released: std::io::Result<()> = std::fs::write(self.fixture.root.join("release"), b"go");
        if let Some(mut child) = self.child.take() {
            let _killed: std::io::Result<()> = child.kill();
            let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
        }
        remove(&self.fixture);
    }
}

/// The text one summary line ends with for a single new worktree.
const ONE_WORKTREE_SUMMARY_END: &str = " into 1 new worktree.\n";

/// The text a command that copied ignored state starts its standard error with.
const COPY_SUMMARY_START: &str = "monochromatic-git-policy-cli: copied ";

/// The PID file Git writes beside one lock under `core.lockfilePid`,
/// which the wrapper turns on for every Git it starts.
fn pid_file_of(lock: &Path) -> PathBuf {
    let name: String = lock.file_name().expect("a lock has a name").to_string_lossy().into_owned();
    return lock.with_file_name(format!("{}~pid.lock", name.strip_suffix(".lock").expect("a lock ends in .lock")));
}

/// What:
///  Write `content` to the PID file beside `lock`,
/// again and again until the file system proves that file was born after the lock.
/// Why:
///  The pass that reads a PID file as this lock's own compares the two births,
///  and this file system gives two files Git writes a moment apart one birth time nearly always:
///  a control that needs the proven order writes the file again until the times differ,
///  which holds the step itself and runs no command a second time.
///  A file system that records no birth time fails the control naming both times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePidFileAfterTheLock('/repo/.git/worktrees/held/index.lock', `pid ${process.pid}\n`);
/// ```
fn write_pid_file_after_the_lock(lock: &Path, content: &str) {
    let pid_file: PathBuf = pid_file_of(lock);
    let lock_born: Option<SystemTime> = std::fs::symlink_metadata(lock).ok().and_then(|metadata| { return metadata.created().ok(); });
    let began: Instant = Instant::now();
    loop {
        let _ = std::fs::remove_file(pid_file.as_path());
        std::fs::write(pid_file.as_path(), content.as_bytes()).expect("the PID file");
        let pid_born: Option<SystemTime> = std::fs::symlink_metadata(pid_file.as_path()).ok().and_then(|metadata| { return metadata.created().ok(); });
        if let (Some(lock_at), Some(pid_at)) = (lock_born, pid_born)
            && pid_at > lock_at
        {
            return;
        }
        assert!(
            began.elapsed() < CONDITION_PATIENCE,
            "this file system cannot show that {} was born after {}: the lock is {lock_born:?} and the PID file {pid_born:?}",
            pid_file.display(),
            lock.display()
        );
    }
}

/// Two adds in one repository overlap:
/// the second finishes while the first's real Git is still running,
/// and neither
/// receives the other's copy.
/// The first add registers its worktree before the second starts,
/// and the second registers and finishes inside the first's
/// real Git,
/// so a comparison of the whole set of registrations around the first would find both worktrees.
/// Two gate runs printed a standing-lock line about the first add's `index.lock` before the second's copy summary,
/// because the file system proved that lock's PID file was born after it:
/// Git's two opens are 150 to 208 µs apart in the gate's container,
/// and that container's birth times advance in ticks of about one millisecond.
/// The lock belongs to the first add's worktree,
/// so no command of this one may name it;
/// `an_add_says_nothing_about_a_lock_a_live_program_holds_in_another_worktree` holds that state on purpose.
#[test]
fn an_add_finishes_while_another_adds_real_git_is_running_and_each_gets_its_own_copy() {
    let mut held: Held = Held::new("copy-concurrent-adds");
    held.start_held_add();
    let free: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "free", "../free"]);
    assert_eq!(free.code, Some(0), "the second add waited or failed: {}", stderr_of(&free));
    assert!(held.held_is_running(), "the first add must still be inside its real Git when the second ends");
    let waiting: String = stderr_of(&free);
    assert!(!waiting.contains("waiting"), "{waiting}");
    assert!(waiting.starts_with(COPY_SUMMARY_START), "{waiting}");
    assert!(waiting.ends_with(ONE_WORKTREE_SUMMARY_END), "the second add copied into more than its own worktree: {waiting}");
    let root: PathBuf = held.fixture.root.clone();
    assert_eq!(std::fs::read(root.join("free").join(RULES_FILE)).expect("free's rules"), format!("{}\n", super::worktree_copy::needle()).into_bytes());
    assert_eq!(std::fs::read(root.join("free/heavy/data")).expect("free's heavy tree"), b"payload\n");
    let first: Observed = held.release_and_finish();
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let summary: String = stderr_of(&first);
    assert!(summary.ends_with(ONE_WORKTREE_SUMMARY_END), "the first add copied into more than its own worktree: {summary}");
    assert!(!summary.contains("recovered"), "{summary}");
    assert_eq!(std::fs::read(root.join("held").join(RULES_FILE)).expect("held's rules"), format!("{}\n", super::worktree_copy::needle()).into_bytes());
    assert_eq!(std::fs::read(root.join("held/heavy/data")).expect("held's heavy tree"), b"payload\n");
    let common: PathBuf = std::fs::canonicalize(held.repo.join(".git")).expect("common directory");
    let leftovers: Vec<String> = std::fs::read_dir(common.join("monochromatic-git-policy-cli-worktree-copy/v1"))
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    assert!(leftovers.is_empty(), "nothing of either copy may remain: {leftovers:?}");
}

/// A move of another worktree does not wait for an add whose real Git is still running.
#[test]
fn a_move_finishes_while_an_add_is_running() {
    let mut held: Held = Held::new("copy-move-beside-add");
    let made: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "old", "../old"]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    held.start_held_add();
    let moved: Observed = held.run(held.repo.as_path(), &["worktree", "move", "../old", "../moved"]);
    assert_eq!(moved.code, Some(0), "the move waited or failed: {}", stderr_of(&moved));
    assert!(!stderr_of(&moved).contains("waiting"), "{}", stderr_of(&moved));
    assert!(held.fixture.root.join("moved/.git").exists());
    assert!(held.held_is_running());
    let first: Observed = held.release_and_finish();
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
}

/// A removal of another worktree does not wait for an add whose real Git is still running.
#[test]
fn a_removal_finishes_while_an_add_is_running() {
    let mut held: Held = Held::new("copy-remove-beside-add");
    let made: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "old", "../old"]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    held.start_held_add();
    let removed: Observed = held.run(held.repo.as_path(), &["worktree", "remove", "--force", "../old"]);
    assert_eq!(removed.code, Some(0), "the removal waited or failed: {}", stderr_of(&removed));
    assert!(!stderr_of(&removed).contains("waiting"), "{}", stderr_of(&removed));
    assert!(!held.fixture.root.join("old").exists());
    assert!(held.held_is_running());
    let first: Observed = held.release_and_finish();
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
}

/// A conversion's plan for another worktree runs while an add is running and does not wait for it.
#[test]
fn a_conversion_check_finishes_while_an_add_is_running() {
    let mut held: Held = Held::new("copy-convert-beside-add");
    let made: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "old", "../old"]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    held.start_held_add();
    let planned: Observed =
        held.run(held.repo.as_path(), &["monochromatic-git-policy-cli", "convert-worktree-to-subvolume", "--dry-run", "../old"]);
    assert!(!stderr_of(&planned).contains("waiting"), "{}", stderr_of(&planned));
    assert!(!stderr_of(&planned).contains("settlement"), "{}", stderr_of(&planned));
    assert!(held.held_is_running());
    let first: Observed = held.release_and_finish();
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
}

/// An add whose source is a worktree a conversion is running in (its kernel lock held,
/// its `index.lock` in place) neither
/// waits for the conversion nor touches either of them:
/// the ladder declines that source and Git checks the worktree out.
#[test]
fn an_add_beside_a_running_conversion_neither_waits_nor_touches_it() {
    let held: Held = Held::new("copy-add-beside-conversion");
    let made: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "old", "../old"]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    let old: PathBuf = std::fs::canonicalize(held.fixture.root.join("old")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(held.repo.join(".git")).expect("common directory");
    let admin: PathBuf = common.join("worktrees/old");
    let lock_file: std::fs::File = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(admin.join("cli-git-convert.lock"))
        .expect("conversion lock file");
    // SAFETY: `flock` takes the descriptor `lock_file` keeps open for the call.
    assert_eq!(unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) }, 0, "the control takes the conversion lock");
    let token: &[u8] = b"monochromatic-git-policy-cli migrate-to-subvolume test\n";
    std::fs::write(admin.join("index.lock"), token).expect("conversion index lock");
    let added: Observed = held.run(old.as_path(), &["worktree", "add", "--quiet", "-b", "beside", "../beside"]);
    assert_eq!(added.code, Some(0), "the add waited or failed: {}", stderr_of(&added));
    assert!(!stderr_of(&added).contains("waiting"), "{}", stderr_of(&added));
    assert!(held.fixture.root.join("beside/.git").exists());
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("the conversion's index lock stays"), token);
    // SAFETY: as above; a second non-blocking lock from another open file shows the kernel lock is still held.
    let other: std::fs::File = std::fs::File::open(admin.join("cli-git-convert.lock")).expect("second handle");
    assert_ne!(unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) }, 0, "the conversion lock is still held");
    drop(lock_file);
}

/// A wrapped add that is killed while its real Git is running leaves nothing that stops,
/// or is taken for,
/// the next add:
/// the next add finishes at once,
/// gets its own copy,
/// and reports no recovery that is not its business.
#[test]
fn an_add_killed_inside_its_real_git_does_not_hold_up_the_next_add() {
    let mut held: Held = Held::new("copy-killed-add");
    held.start_held_add();
    let timeout_id: u32 = held.child.as_ref().expect("started").id();
    let children: String = std::fs::read_to_string(format!("/proc/{timeout_id}/task/{timeout_id}/children")).expect("children of the bound");
    for wrapper in children.split_whitespace() {
        let pid: libc::pid_t = wrapper.parse().expect("a process id");
        // SAFETY: `kill` takes any process id and signal number.
        assert_eq!(unsafe { libc::kill(pid, libc::SIGKILL) }, 0, "the wrapper of the held add is killed");
    }
    let killed: Observed = held.release_and_finish();
    assert_ne!(killed.code, Some(0), "the killed add cannot have succeeded");
    let free: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "free", "../free"]);
    assert_eq!(free.code, Some(0), "the next add waited or failed: {}", stderr_of(&free));
    assert!(!stderr_of(&free).contains("waiting"), "{}", stderr_of(&free));
    assert!(stderr_of(&free).ends_with(ONE_WORKTREE_SUMMARY_END), "{}", stderr_of(&free));
    assert!(held.fixture.root.join("free").join(RULES_FILE).exists());
    let again: Observed = run_wrapped(&held.fixture, held.fixture.root.join("free").as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(again.code, Some(0), "{}", stderr_of(&again));
}

/// What:
///  An add says only what it copied when another worktree's administrative directory holds an `index.lock`
///  whose PID file this file system proves was born after the lock and names a live program.
/// Why:
///  The gate on `0.6.1-alpha.34` (`c6a7e45273`) and again on `main` at `92385d3581` (both 2026-11-14) failed
///  `an_add_finishes_while_another_adds_real_git_is_running_and_each_gets_its_own_copy`
///  because those runs' file system gave the first add's two files a proven order:
///  the second add then printed a standing-lock line about the first add's `index.lock` before its own copy summary,
///  and the runs whose file system gave them one birth time printed nothing.
///  Git creates the lock and then opens the PID file beside it (`lockfile.c`),
///  and eight runs of real Git 2.56.1 in that gate's container put the two opens 150 to 208 µs apart (`strace`),
///  while this volume's and that container's birth times advance in ticks of about one millisecond:
///  so about one add in five gets the proven order,
///  and no run may rely on where a tick falls.
///  A lock a live program holds in another worktree is that worktree's own write,
///  its own command ends by itself,
///  and no command of this worktree may report it as standing in its way.
///  The state is planted because a control may not wait for a tick:
///  writing the PID file until the order is proven holds that step without waiting and without a second run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantLiveLockInAnotherWorktree(repo);
/// expect(stderr(run(wrapped, ['worktree', 'add', '--quiet', '-b', 'free', '../free']))).toStartWith('monochromatic-git-policy-cli: copied ');
/// ```
#[test]
fn an_add_says_nothing_about_a_lock_a_live_program_holds_in_another_worktree() {
    let held: Held = Held::new("copy-live-lock-in-another-worktree");
    let other: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "other", "../other"]);
    assert_eq!(other.code, Some(0), "{}", stderr_of(&other));
    let admin: PathBuf = held.repo.join(".git/worktrees/other");
    let lock: PathBuf = admin.join("index.lock");
    std::fs::write(lock.as_path(), b"a half-written index the live program never renamed\n").expect("the lock");
    // This control's own process has the number, so nothing proves the lock abandoned and the lock's own command ends by itself.
    write_pid_file_after_the_lock(lock.as_path(), format!("pid {}\n", std::process::id()).as_str());
    let free: Observed = held.run(held.repo.as_path(), &["worktree", "add", "--quiet", "-b", "free", "../free"]);
    assert_eq!(free.code, Some(0), "the add waited or failed: {}", stderr_of(&free));
    let said: String = stderr_of(&free);
    assert!(said.starts_with(COPY_SUMMARY_START), "a command of this worktree says nothing about another worktree's lock: {said}");
    assert!(said.ends_with(ONE_WORKTREE_SUMMARY_END), "the add copied into more than its own worktree: {said}");
    assert!(lock.exists(), "a live program's lock is left where it is: {said}");
}
