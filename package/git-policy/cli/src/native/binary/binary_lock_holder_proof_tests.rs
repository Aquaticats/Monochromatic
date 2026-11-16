//! What:
//!  Controls of the repository proof through the built wrapper, with real Git beside it, at the two ends the seventh
//!  review named:
//!  a name in the object store that is a pipe,
//!  which must never hold a command,
//!  and the repository shapes real Git works in,
//!  in which the proof must not switch the clearing of a dead Git's lock off.
//! Why:
//!  The proof is taken before every command,
//!  so an open it makes on a path that is not a regular file holds the person's command for good,
//!  and its answer decides whether a lock a killed Git left is ever cleared:
//!  refusing a repository Git works in leaves the person meeting `index.lock` in every later command
//!  (a review of `faf314ba2f`,
//! findings 1 and 2).
//!  `main` had neither end, because it had no proof at all;
//!  these controls hold the proof to what `main` did there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mkfifo(join(repo, '.git/objects/pack/pack-x.idx')); expect(runWrapped(repo, ['status']).code).toBe(0);
//! plantDeadHoldersLock(reftableRepo); expect(runWrapped(repo, ['status']).stderr).toContain('cleared')
//! ```

/// The fixture, real Git and the wrapped command of a binary-level control.
use super::support::{Fixture, direct, fixture, git, remove, repository, run_direct, run_wrapped};
/// The record a dead writer leaves: the folder, the one spelling of a line, and the birth this boot reads for a process.
use git_policy_cli::lock_holder_records::{folder_of, identity_of_process, record_text};
/// `OsStr` names a repository folder;
/// `Path`/`PathBuf` are the borrowed and owned paths of the fixtures.
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
/// The stand-in writer this control starts, ends, and reads the birth of.
use std::process::{Child, Command, Stdio};

/// How long the control waits between a lock and its PID file, so the file system records two different birth times.
/// The same wait the author's unit controls and the review's scripts use: some file systems keep a coarse clock, and two
/// files made in one tick cannot be told apart, which keeps the lock for a reason this control is not about.
const BETWEEN_THE_LOCK_AND_ITS_PID_FILE: std::time::Duration = std::time::Duration::from_millis(50);

/// What:
///  A process of this control's that has ended,
///  with the birth identity this boot read for it before it ended.
/// Why:
///  A record only proves a holder gone when the identity it names is one of this boot's own and the process is gone,
///  which is what a killed Git leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeadWriter = { pid: number; identity: string };
/// ```
struct DeadWriter {
    /// The process, still running while its identity is read.
    child: Child,
    /// Its number, which its record and its PID file name.
    pid: i64,
    /// The birth identity read for it while it ran.
    identity: String,
}

impl DeadWriter {
    /// What:
    ///  Start a stand-in writer and read its birth identity.
    /// Why:
    ///   The identity is readable only while the process runs,
    ///  and the record must name the same identity the pass will judge.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const child = spawn('sleep', ['60'], { stdio: 'ignore' }); const identity = identityOfProcess(child.pid);
    /// ```
    fn start() -> DeadWriter {
        let child: Child = Command::new("sleep").arg("60").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("the stand-in writer starts");
        let pid: i64 = i64::from(child.id());
        let identity: String = identity_of_process(pid).expect("this boot reads the stand-in writer's birth");
        return DeadWriter { child, pid, identity };
    }

    /// End the stand-in, so its number is one no process has.
    fn end(&mut self) {
        let _killed = self.child.kill();
        let _waited = self.child.wait();
    }
}

/// What:
///  Plant, in `repo`, the state a killed Git leaves:
///  the record of a writer that has ended,
///  its lock,
///  and a PID file made a moment after the lock that names it.
/// Why:
///   The lock is clearable only on the record,
///  and the record's number must be gone,
///  so the writer is ended after both are written.
///  The PID file is made after the lock because a file system with a coarse clock
///  cannot prove the order of two files made in one tick.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeRecord(pid, identity); writeFile(lock); await sleep(50); writeFile(`${stem}~pid.lock`, `pid ${pid}\n`); writer.end();
/// ```
fn plant_a_dead_holders_lock(repo: &Path) {
    let mut writer: DeadWriter = DeadWriter::start();
    let folder: PathBuf = folder_of(repo.join(".git").as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    std::fs::write(
        folder.join(format!("{}.record", writer.pid)).as_path(),
        // The record's own spelling of a line about its writer, as the file itself carries it.
        record_text(&[("self", writer.pid, writer.identity.clone())]).as_bytes(),
    )
    .expect("the record a killed command leaves");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"").expect("the lock a killed Git left");
    std::thread::sleep(BETWEEN_THE_LOCK_AND_ITS_PID_FILE);
    std::fs::write(repo.join(".git/index~pid.lock").as_path(), format!("pid {}\n", writer.pid).as_bytes()).expect("the PID file Git made for that lock");
    writer.end();
}

/// What:
///  A wrapper command that meets the planted lock ends with Git's own status,
///  says it cleared that lock,
///  and leaves it gone.
/// Why:
///  This is the whole of finding 2:
///  a repository the proof refuses gets no pass,
///  so the killed Git's lock stands,
///  and every later command of the person meets it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runWrapped(repo, ['status']).code).toBe(0); expect(stderr).toContain('cleared'); expect(existsSync(lock)).toBe(false);
/// ```
fn assert_the_lock_is_cleared(fixture: &Fixture, repo: &Path, shape: &str) {
    let native: super::support::Observed = run_direct(fixture, repo, &["status", "--short"]);
    assert_eq!(native.code, Some(0), "real Git works in the {shape}, so the wrapper must too: {}", String::from_utf8_lossy(native.stderr.as_slice()));
    plant_a_dead_holders_lock(repo);
    let wrapped: super::support::Observed = run_wrapped(fixture, repo, &["status", "--short"]);
    let said: String = String::from_utf8_lossy(wrapped.stderr.as_slice()).into_owned();
    assert_eq!(wrapped.code, Some(0), "the wrapped command ends with Git's own status in the {shape}: {said}");
    assert!(said.contains("cleared"), "the proof must pass in the {shape}, so the dead Git's lock is cleared and the words say so: {said}");
    assert!(!repo.join(".git/index.lock").exists(), "the lock of the killed Git is gone in the {shape}: {said}");
}

/// What:
///  A name in `objects/pack` that is a pipe never holds the wrapper's command,
///  whatever Git later does with it.
/// Why:
///  The proof asks the object store whether a detached `HEAD`'s object is there,
///  and the reader it used opened every `*.idx` and its `.pack` with a blocking open:
///  a pipe at either name held every command before Git started
///  (a review of `faf314ba2f`,
/// finding 1).
///  Real Git returns at once,
///  so the wrapper may not do worse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(repo, '.git/objects/pack/pack-x.idx')); expect(runWrapped(repo, ['status']).code).toBe(0)
/// ```
#[test]
fn a_pipe_named_like_a_pack_index_never_holds_a_command() {
    let fixture: Fixture = fixture("pipe-named-like-a-pack-index");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let pipe: PathBuf = repo.join(".git/objects/pack/pack-x.idx");
    let made: std::io::Result<std::process::ExitStatus> = Command::new("mkfifo").arg(pipe.as_path()).status();
    assert!(made.map(|status| { return status.success(); }).unwrap_or(false), "the control made a pipe at the pack index's name");
    let native: super::support::Observed = run_direct(&fixture, repo.as_path(), &["status", "--short"]);
    let wrapped: super::support::Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--short"]);
    assert_eq!(native.code, Some(0), "real Git returns at once with a pipe at a pack index's name");
    assert_eq!(wrapped.code, native.code, "a pipe at a name the proof reads must not hold the command: the wrapper ended with {:?}", wrapped.code);
    remove(&fixture);
}

/// What:
///  A pack index whose `.pack` is a pipe never holds the wrapper's command either.
/// Why:
///  The same blocking open covered a real `.idx` whose `.pack` is a pipe,
///  and the object it names is reachable from the index alone
///  (a review of `faf314ba2f`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gc(repo); cp(pack-*.idx, pack-pipe.idx); mkfifo(pack-pipe.pack); expect(runWrapped(repo, ['status']).code).toBe(0)
/// ```
#[test]
fn an_index_whose_pack_is_a_pipe_never_holds_a_command() {
    let fixture: Fixture = fixture("index-whose-pack-is-a-pipe");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("a tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "--detach"]);
    git(&fixture, repo.as_path(), &["gc", "--quiet"]);
    let pack: PathBuf = repo.join(".git/objects/pack");
    let index: PathBuf = std::fs::read_dir(pack.as_path())
        .expect("the pack folder")
        .flatten()
        .map(|entry| { return entry.path(); })
        .find(|path| { return path.extension() == Some(OsStr::new("idx")); })
        .expect("the packed repository holds an index");
    let copied: PathBuf = pack.join("pack-pipe.idx");
    std::fs::copy(index.as_path(), copied.as_path()).expect("the index copied under the pipe's name");
    let pipe: PathBuf = pack.join("pack-pipe.pack");
    let made: std::io::Result<std::process::ExitStatus> = Command::new("mkfifo").arg(pipe.as_path()).status();
    assert!(made.map(|status| { return status.success(); }).unwrap_or(false), "the control made a pipe at the pack's name");
    let native: super::support::Observed = run_direct(&fixture, repo.as_path(), &["status", "--short"]);
    let wrapped: super::support::Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--short"]);
    assert_eq!(native.code, Some(0), "real Git returns at once with a pipe at a pack's name");
    assert_eq!(wrapped.code, native.code, "a pipe at a name the proof reads must not hold the command: the wrapper ended with {:?}", wrapped.code);
    remove(&fixture);
}

/// What:
///  Every repository shape below is one real Git works in,
///  and the proof must not be the reason a killed Git's lock is never cleared there.
/// Why:
///  The proof refuses a `HEAD` it cannot parse whole and a detached `HEAD` whose object it cannot find:
///  the reftable backend's `ref: refs/heads/.invalid`,
///  a `HEAD` that is a symbolic link,
///  an object store whose alternates hold a stale or a quoted entry,
///  and a store holding an index whose pack is missing all made it answer no,
///  so no record was written and no lock cleared,
///  where `main` cleared in every one
///  (a review of `faf314ba2f`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const shape of [reftable, symlinkedHead, staleAlternates, quotedAlternates, indexWithoutPack, shared]) { assertTheLockIsCleared(shape) }
/// ```
#[test]
fn repositories_real_git_works_in_keep_their_lock_clearing() {
    let fixture: Fixture = fixture("repositories-real-git-works-in");
    // 1. The reftable backend's HEAD names `refs/heads/.invalid`, which Git accepts and this proof used to refuse.
    let reftable: PathBuf = fixture.root.join("reftable");
    std::fs::create_dir(reftable.as_path()).expect("the reftable repository folder");
    git(&fixture, reftable.as_path(), &["init", "--quiet", "--initial-branch=main", "--ref-format=reftable"]);
    git(&fixture, reftable.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    assert!(reftable.join(".git/HEAD").exists(), "the reftable repository has a HEAD");
    assert_the_lock_is_cleared(&fixture, reftable.as_path(), "reftable repository");
    // 2. A HEAD that is a symbolic link, as `core.preferSymlinkRefs` writes it.
    let symlinked: PathBuf = repository(&fixture, OsStr::new("symlinked-head"));
    std::fs::remove_file(symlinked.join(".git/HEAD")).expect("the real HEAD gives way to the link");
    std::os::unix::fs::symlink("refs/heads/main", symlinked.join(".git/HEAD")).expect("the link at HEAD");
    assert_the_lock_is_cleared(&fixture, symlinked.as_path(), "repository whose HEAD is a symbolic link");
    // 3. A detached HEAD and an alternates entry naming a clone that is gone: Git warns and works.
    let stale: PathBuf = repository(&fixture, OsStr::new("stale-alternates"));
    git(&fixture, stale.as_path(), &["checkout", "--quiet", "--detach"]);
    std::fs::write(stale.join(".git/objects/info/alternates"), b"/nonexistent/deleted-clone/objects\n").expect("the stale alternates entry");
    assert_the_lock_is_cleared(&fixture, stale.as_path(), "repository with a stale alternates entry");
    // 4. The same with a quoted entry, which Git reports and skips.
    let quoted: PathBuf = repository(&fixture, OsStr::new("quoted-alternates"));
    git(&fixture, quoted.as_path(), &["checkout", "--quiet", "--detach"]);
    std::fs::write(quoted.join(".git/objects/info/alternates"), b"\"/quoted/path\"\n").expect("the quoted alternates line");
    assert_the_lock_is_cleared(&fixture, quoted.as_path(), "repository with a quoted alternates line");
    // 5. A detached HEAD whose store holds an index with no pack beside it.
    let orphan: PathBuf = repository(&fixture, OsStr::new("index-without-pack"));
    std::fs::write(orphan.join("tracked.txt"), b"one\n").expect("a tracked file");
    git(&fixture, orphan.as_path(), &["add", "--", "tracked.txt"]);
    git(&fixture, orphan.as_path(), &["commit", "--quiet", "--message=one"]);
    git(&fixture, orphan.as_path(), &["checkout", "--quiet", "--detach"]);
    git(&fixture, orphan.as_path(), &["gc", "--quiet"]);
    let pack: PathBuf = orphan.join(".git/objects/pack");
    let index: PathBuf = std::fs::read_dir(pack.as_path())
        .expect("the pack folder")
        .flatten()
        .map(|entry| { return entry.path(); })
        .find(|path| { return path.extension() == Some(OsStr::new("idx")); })
        .expect("the packed repository holds an index");
    std::fs::copy(index.as_path(), pack.join("orphan.idx").as_path()).expect("an index with no pack beside it");
    assert_the_lock_is_cleared(&fixture, orphan.as_path(), "repository holding an index with no pack");
    // 6. A shared clone, whose object lives only in the alternate store, which cleared before and must keep clearing.
    let origin: PathBuf = repository(&fixture, OsStr::new("shared-origin"));
    let shared: PathBuf = fixture.root.join("shared");
    let cloned: std::io::Result<std::process::ExitStatus> = direct(&fixture).args(["clone", "--quiet", "--shared", origin.to_string_lossy().as_ref(), shared.to_string_lossy().as_ref()]).status();
    assert!(cloned.map(|status| { return status.success(); }).unwrap_or(false), "the control made a shared clone");
    git(&fixture, shared.as_path(), &["checkout", "--quiet", "--detach"]);
    assert_the_lock_is_cleared(&fixture, shared.as_path(), "shared clone whose object is in the alternate store");
    remove(&fixture);
}

/// What:
///  The wrapper reaches real Git in a repository where an object the proof looks for is a pipe,
///  rather than answering for Git with a bookkeeping failure.
/// Why:
///   A command that the proof could not finish is a command the person's Git never runs,
///  and the notice of a proof that gave up must not be mistaken for Git's own refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(repo, '.git/objects/pack/pack-x.idx')); expect(runWrapped(repo, ['rev-parse', 'HEAD']).stdout).toBe(sha)
/// ```
#[test]
fn a_repository_whose_pack_name_is_a_pipe_still_answers_with_git() {
    let fixture: Fixture = fixture("pack-name-is-a-pipe-still-answers");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let pipe: PathBuf = repo.join(".git/objects/pack/pack-x.idx");
    let made: std::io::Result<std::process::ExitStatus> = Command::new("mkfifo").arg(pipe.as_path()).status();
    assert!(made.map(|status| { return status.success(); }).unwrap_or(false), "the control made a pipe at the pack index's name");
    let native: super::support::Observed = run_direct(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let wrapped: super::support::Observed = run_wrapped(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(native.code, Some(0), "real Git answers the object name with a pipe at a pack index's name");
    assert_eq!(wrapped.stdout, native.stdout, "the wrapper answers with Git's own bytes, so Git really ran");
    remove(&fixture);
}
