//! Controls for the look at the processes under its ceiling:
//! the scan runs on the blocking pool and the wait for it is awaited,
//! so a task beside the look keeps running,
//! and a scan that outlasts the ceiling counts as "may be running";
//! a command line longer than any program can carry is read whole,
//! and a command line that cannot be read names its process.

use super::*;
use crate::scan_awaited_under_a_ceiling::scan_under_ceiling;
use crate::test_support::{fixture, remove};
use crate::ticker_beside_a_wait::with_a_ticker_beside;
use crate::wait_ceilings::PROCESS_LOOK_CEILING;

/// How long the scan of a control takes.
const SLOW_SCAN: Duration = Duration::from_millis(150);

/// A look with the given ceiling.
fn look_with(ceiling: Duration) -> ServiceLook {
    return ServiceLook { instance: String::from("abc"), journals: Vec::new(), own_pid: 1, own_user: effective_user(), ceiling };
}

/// A look of the instance `abc` whose earlier journals are `journals`,
/// under the ceiling a control gives it.
fn look_of_journals(journals: &[PathBuf]) -> ServiceLook {
    return ServiceLook { instance: String::from("abc"), journals: journals.to_vec(), own_pid: 1, own_user: effective_user(), ceiling: PROCESS_LOOK_CEILING };
}

/// Why a long command line is as long as it is:
/// a word is at most 32 pages (`MAX_ARG_STRLEN`,
/// Linux `include/uapi/linux/binfmts.h`),
/// and the kernel refuses a command line and environment together above 6 MiB (`fs/exec.c`,
/// `bprm_stack_limits`).
/// This control's line is 200 words of 1 KiB,
/// 204,800 bytes:
/// longer than the 65,536 bytes the look used to read whole,
/// and far below what a program may carry.
const LONG_LINE_WORD_BYTES: usize = 1024;
const LONG_LINE_WORDS: usize = 200;

/// The process number the fake process table lists.
const FAKE_PID: u32 = 77;

/// What a process of a fake table is called,
/// as the kernel keeps its program's name.
const FAKE_PROGRAM_NAME: &str = "unrelated-tool";

/// What:
///  A process table of a control:
/// an empty table of sockets,
/// and one process whose command line is `words`.
/// Why:
///   A look reads a table of the process numbers it is given,
/// so a control states every process it is up against and never depends on what else the machine runs
///  (`earlier_service_process_hold`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const procRoot = fakeProcRoot({ words });
/// ```
fn proc_root_with(words: &[String]) -> PathBuf {
    let root: PathBuf = fixture("look-table");
    std::fs::create_dir_all(root.join("net")).expect("a socket table folder");
    std::fs::write(root.join("net/unix"), "Num       RefCount Protocol Flags    Type St Inode Path\n").expect("an empty socket table");
    let process: PathBuf = root.join(FAKE_PID.to_string());
    std::fs::create_dir_all(&process).expect("a process folder");
    std::fs::write(process.join("comm"), format!("{FAKE_PROGRAM_NAME}\n")).expect("the program's kernel name");
    let mut bytes: Vec<u8> = Vec::new();
    for word in words {
        bytes.extend(word.as_bytes());
        bytes.push(0);
    }
    std::fs::write(process.join("cmdline"), bytes).expect("a command line");
    return root;
}

/// What:
///  Gives the one process of a fake table the working folder it was started in:
/// `<proc root>/<pid>/cwd`, the link the kernel keeps for it and follows for a relative word.
/// Why:
///   A relative `--journal` word names a folder in the working folder of the program that carries it,
/// and a control states that folder instead of depending on one the machine happens to have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlinkSync(folder, `${procRoot}/${pid}/cwd`)
/// ```
fn with_working_folder(root: &Path, folder: &Path) {
    std::os::unix::fs::symlink(folder, root.join(FAKE_PID.to_string()).join("cwd")).expect("the process's working folder");
}

/// The words of a long command line that is no service of any worktree.
fn long_command_line() -> Vec<String> {
    let mut words: Vec<String> = vec![String::from("/usr/bin/git"), String::from("commit"), String::from("-m")];
    for at in 0..LONG_LINE_WORDS {
        words.push(format!("{at:07}").chars().cycle().take(LONG_LINE_WORD_BYTES).collect());
    }
    return words;
}

/// What:
///  A command line longer than the limit the look used to read is proved whole,
/// so the look finds no service.
/// Why:
///   The limit exists to keep one process from filling memory,
/// and a limit below what a program may carry counts an ordinary program as one that
///  may be the earlier helper:
/// a commit whose message travels in its arguments,
/// a script passed with `-e`,
/// and a `podman run` line with its mounts all carry tens of kilobytes,
/// which measured 44,174 bytes of one program's command line on this host on 2026-11-10
/// (`package/git-policy/cli/src/native/invocation/management_service_settle_earlier_batches_tests.rs`
/// holds the same fact where the person meets it).
/// A process counted that way refuses every settle command on the machine for as long as it runs,
///  and its reason says a look ran out of time,
/// which nothing can clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(earlierServiceProcessHold(look, procRoot)).toBeUndefined();
/// ```
#[test]
#[cfg(unix)]
fn a_command_line_longer_than_a_program_can_carry_is_proved_whole() {
    let root: PathBuf = proc_root_with(long_command_line().as_slice());
    let held: Option<Holding> = earlier_service_process_hold(look_with(PROCESS_LOOK_CEILING), root.as_path());
    assert!(held.is_none(), "a command line that is no service holds nothing back: {held:?}");
    remove(root.as_path());
}

/// A scan that takes `SLOW_SCAN` and finds no service.
fn slow_scan_that_finds_nothing(_look: &ServiceLook, _proc_root: &Path) -> Option<Holding> {
    std::thread::sleep(SLOW_SCAN);
    return None;
}

/// The look of a control,
/// awaited under `ceiling` as the look functions await it.
async fn look_awaited(ceiling: Duration) -> Option<Holding> {
    return scan_under_ceiling(look_with(ceiling), ceiling, PathBuf::from("/proc"), slow_scan_that_finds_nothing, cannot_look).await;
}

/// A look whose scan takes a while is awaited:
/// the task beside it runs meanwhile,
/// and the scan's answer is the look's.
#[test]
fn a_look_that_takes_a_while_is_awaited_and_leaves_the_other_tasks_their_turns() {
    let (held, ticks) = with_a_ticker_beside(look_awaited(SLOW_SCAN * 100));
    assert!(held.is_none(), "the scan found no service, so nothing holds");
    assert!(ticks > 0, "the wait for the scan was awaited on the runtime");
}

/// How long a held scan waits for its control to let it go,
/// after which it ends by itself:
/// the safety valve that makes a wait which blocks the worker fail the control instead of hanging it.
/// Why longer than `CEILING_OF_A_HELD_LOOK`:
///   The look must reach its ceiling while the scan is still held,
/// and a machine that takes seconds to schedule either
///  thread still leaves the two apart.
const RELEASE_BOUND: Duration = Duration::from_secs(10);

/// The ceiling of the held look;
///  far below `RELEASE_BOUND`,
/// and long enough that the scan has started before it,
/// whatever the machine's speed.
const CEILING_OF_A_HELD_LOOK: Duration = Duration::from_secs(2);

/// What:
///  The rendezvous a held scan and the control that reads a look share:
/// the scan says it started,
/// waits to be let go,
/// and ends once it is.
/// Why:
///   Whether the wait for a scan is awaited or blocks the worker is a fact about two threads,
/// not about how
///  long anything took:
/// an awaited wait reaches its ceiling while the scan is still inside the blocking pool,
/// and a wait
///  that blocked the worker reaches it only after the scan ended.
///  The scan's own bound keeps a blocking wait from hanging
///  the control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const halt = new Halt(); // started, then released by the control, then held until let go
/// ```
struct Halt {
    /// Signs that the scan entered the pool.
    started: std::sync::mpsc::Sender<()>,
    /// Whether the control let the scan go.
    released: std::sync::Mutex<bool>,
    /// Wakes the scan when it is let go.
    let_go: std::sync::Condvar,
    /// Whether the scan has ended,
    /// by the control's release or by its own bound.
    finished: std::sync::atomic::AtomicBool,
}

impl Halt {
    /// A rendezvous no scan has touched.
    fn new() -> Halt {
        let (started, _receiver) = std::sync::mpsc::channel();
        return Halt {
            started,
            released: std::sync::Mutex::new(false),
            let_go: std::sync::Condvar::new(),
            finished: std::sync::atomic::AtomicBool::new(false),
        };
    }

    /// Wait until the scan is let go,
    /// or until `RELEASE_BOUND` has passed.
    fn wait_to_be_let_go(&self) {
        let Ok(released) = self.released.lock() else { return };
        let _waited = self
            .let_go
            .wait_timeout_while(released, RELEASE_BOUND, |let_go| { return !*let_go; });
    }

    /// Let the held scan go.
    fn let_go(&self) {
        if let Ok(mut released) = self.released.lock() {
            *released = true;
        }
        self.let_go.notify_all();
    }
}

/// The rendezvous the one control that holds a scan installs;
///  `None` in every other control of this file.
static HALT: std::sync::OnceLock<std::sync::Arc<Halt>> = std::sync::OnceLock::new();

/// A scan that says it started,
/// waits to be let go,
/// and finds no service.
fn held_scan_that_finds_nothing(_look: &ServiceLook, _proc_root: &Path) -> Option<Holding> {
    let halt: &std::sync::Arc<Halt> = HALT.get()?;
    let _sent = halt.started.send(());
    halt.wait_to_be_let_go();
    halt.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    return None;
}

/// The look of a control under the held scan.
async fn held_look_awaited(ceiling: Duration) -> Option<Holding> {
    return scan_under_ceiling(look_with(ceiling), ceiling, PathBuf::from("/proc"), held_scan_that_finds_nothing, cannot_look).await;
}

/// A scan that outlasts the ceiling is "cannot look",
/// which counts as a service that may be running,
/// and the wait ends at the ceiling while the scan goes on in the pool.
/// Why a held scan says what the wait did,
/// and a clock does not:
///   The proof that the wait was awaited is that the ceiling's own timer fired:
/// the runtime can drive that timer only
///  while the task that waits is not occupying its worker,
/// so a wait that blocked the worker cannot return at the ceiling,
///  whatever the machine's speed.
/// The held scan makes the scan's own end a fact this control chooses,
/// not a duration,
/// and it is
///  still inside the blocking pool when the look returns.
/// A ticker or an elapsed bound read how the operating system scheduled this
///  runtime's single worker instead (this control read 336 ms of a 150 ms scan in a folder run of 745
///  controls),
/// which is the same reading a blocking wait gives.
/// The scan is let go only after the look returned,
/// so a wait that
///  blocked the worker cannot reach the release:
/// the scan ends at `RELEASE_BOUND` by itself,
/// and the control
///  fails with what that means instead of hanging.
/// The look runs on a runtime of its own here,
/// because the ticker helper's runtime does not drive a ceiling's timer until
///  the blocking job it covers ends (measured),
/// which hides the fact this control reads.
#[test]
fn a_look_that_outlasts_its_ceiling_counts_as_may_be_running_at_the_ceiling() {
    let halt: std::sync::Arc<Halt> = std::sync::Arc::new(Halt::new());
    let _installed: Result<(), std::sync::Arc<Halt>> = HALT.set(std::sync::Arc::clone(&halt));
    let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    let began: std::time::Instant = std::time::Instant::now();
    let held: Option<Holding> = runtime.block_on(held_look_awaited(CEILING_OF_A_HELD_LOOK));
    assert!(
        !halt.finished.load(std::sync::atomic::Ordering::SeqCst),
        "the scan was still held when the look returned, so the wait was awaited and not blocked on it"
    );
    assert!(held.is_some(), "a look that could not finish counts as a hold");
    halt.let_go();
    assert!(began.elapsed() >= CEILING_OF_A_HELD_LOOK, "the wait ran to its ceiling: {:?}", began.elapsed());
    assert!(began.elapsed() < RELEASE_BOUND, "the wait ended at the ceiling and not when the scan did: {:?}", began.elapsed());
}

/// The tail every earlier journal of the instance `abc` carries below the folder a control puts it in:
/// `cli-git/service/abc`,
/// which is what an earlier release's unit passed.
const EARLIER_JOURNAL_TAIL: &str = "state/cli-git/service/abc";

/// What:
///  The earlier journal of the instance `abc` below `root`,
///  made,
///  at the tail a control spells a relative word with.
/// Why:
///   A relative word is only meaningful against the working folder of the program that carries it,
/// so a control
///  states that folder and the journal below it,
/// and never depends on a folder the machine happens to hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journal = mkdirp(join(root, 'state/cli-git/service/abc'))
/// ```
fn earlier_journal_below(root: &Path) -> PathBuf {
    let journal: PathBuf = root.join(EARLIER_JOURNAL_TAIL);
    std::fs::create_dir_all(journal.as_path()).expect("the earlier journal");
    return journal;
}

/// What:
///  A program whose `--journal` word is relative is judged by the folder that word names in the working folder of
///  that program,
///  not in the folder this command runs in,
///  and the two ends are stated apart:
///  a word that names another folder holds nothing back,
///  and the same word that names this worktree's earlier journal holds.
/// Why:
///   The look reads the word in the program's own folder,
/// which is what makes another program's word harmless;
/// a look that
///  read it in the settling command's folder would refuse on every program with a relative word,
/// and one that
///  took every relative word for this worktree's would settle a batch beside an earlier helper that writes it.
/// A review of another worktree ran the wrapper as a service with
///  `--journal home/.local/state/...` from its own folder (2026-11-14),
/// and the settle controls of that machine held on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = procRootWith(['/p/monochromatic-git-policy-cli', 'service', 'run', '--journal', 'state/cli-git/service/abc'])
/// ```
#[test]
#[cfg(unix)]
fn a_relative_journal_word_is_read_in_the_working_folder_of_the_program() {
    let fixture: PathBuf = fixture("look-relative-word");
    let journal: PathBuf = earlier_journal_below(fixture.as_path());
    let look: ServiceLook = look_of_journals(&[journal]);
    // The program's own folder holds a journal of its own worktree at the same relative path:
    // it writes there, so it is not this worktree's helper whatever the word spells.
    let another_working_folder: PathBuf = fixture.join("another-worktree");
    std::fs::create_dir_all(another_working_folder.join(EARLIER_JOURNAL_TAIL)).expect("another worktree's journal");
    let elsewhere: PathBuf = proc_root_with(&[String::from("/p/monochromatic-git-policy-cli"), String::from("service"), String::from("run"), String::from("--journal"), String::from(EARLIER_JOURNAL_TAIL)]);
    with_working_folder(elsewhere.as_path(), another_working_folder.as_path());
    let elsewhere_held: Option<Holding> = earlier_service_process_hold(look.clone(), elsewhere.as_path());
    assert!(elsewhere_held.is_none(), "the word names another worktree's journal, so nothing of this one holds: {elsewhere_held:?}");
    // The same word, the same command line, and a working folder this worktree's journal is in: it may be the helper.
    let here: PathBuf = proc_root_with(&[String::from("/p/monochromatic-git-policy-cli"), String::from("service"), String::from("run"), String::from("--journal"), String::from(EARLIER_JOURNAL_TAIL)]);
    with_working_folder(here.as_path(), fixture.as_path());
    let here_held: Option<Holding> = earlier_service_process_hold(look, here.as_path());
    assert_eq!(here_held.map(|holding| { return holding.because; }), Some(Hold::InstanceHelperRuns), "the word names this worktree's earlier journal in the program's own folder");
    // A working folder the kernel reports as deleted, with a word that names a folder inside it:
    // everything that word can name went with the folder, so it holds no batch of this worktree.
    let gone: PathBuf = proc_root_with(&[String::from("/p/monochromatic-git-policy-cli"), String::from("service"), String::from("run"), String::from("--journal"), String::from(EARLIER_JOURNAL_TAIL)]);
    with_working_folder(gone.as_path(), fixture.join("a-working-folder (deleted)").as_path());
    let gone_held: Option<Holding> = earlier_service_process_hold(look_of_journals(&[fixture.join(EARLIER_JOURNAL_TAIL)]), gone.as_path());
    assert!(gone_held.is_none(), "a folder that went with the working folder holds no batch: {gone_held:?}");
    remove(fixture.as_path());
}

/// What:
///  A relative word that names this worktree's earlier journal before the folder is there holds the settling back:
///  a
///  starting earlier helper of the worktree is in exactly that state.
/// Why:
///   A folder that is not there holds no batch by itself,
/// so the word alone would settle;
/// the path it
///  names, resolved as far as the file system knows it,
/// is this worktree's earlier journal's,
/// and an
///  earlier helper of the worktree shows its journal word before it makes the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = earlierServiceProcessHold(look([join(root, 'not-yet/cli-git/service/abc')]), procRootWith(['--journal', 'not-yet/cli-git/service/abc'], root))
/// ```
#[test]
#[cfg(unix)]
fn a_relative_journal_word_that_names_this_worktrees_journal_before_it_is_made_holds() {
    let fixture: PathBuf = fixture("look-relative-absent");
    let journal: PathBuf = fixture.join("not-yet/cli-git/service/abc");
    assert!(!journal.exists(), "the journal is not made yet, as it is not for a starting helper");
    let root: PathBuf = proc_root_with(&[String::from("/p/monochromatic-git-policy-cli"), String::from("--journal"), String::from("not-yet/cli-git/service/abc")]);
    with_working_folder(root.as_path(), fixture.as_path());
    let held: Option<Holding> = earlier_service_process_hold(look_of_journals(&[journal]), root.as_path());
    assert_eq!(held.map(|holding| { return holding.because; }), Some(Hold::InstanceHelperRuns), "the word names this worktree's journal, whose folder is not made yet");
    remove(fixture.as_path());
}

/// What:
///  A relative word whose program's working folder cannot be read leaves the question open,
///  and the program and what stopped the look-up are named.
/// Why:
///   Nothing then says which folder the word names,
/// so the program may be the earlier helper;
/// a hold that
///  names no program and no cause is one a person cannot act on.
/// The table has no `cwd` entry,
/// which is what
///  the kernel shows of a process that hid its working folder or ended while it was looked at.
#[test]
#[cfg(unix)]
fn a_relative_journal_word_whose_working_folder_cannot_be_read_holds_and_names_what_stopped_it() {
    let fixture: PathBuf = fixture("look-relative-unreadable");
    let journal: PathBuf = earlier_journal_below(fixture.as_path());
    let root: PathBuf = proc_root_with(&[String::from("/usr/bin/unrelated-tool"), String::from("--journal"), String::from("home/.local/state/cli-git/service/abc")]);
    let held: Option<Holding> = earlier_service_process_hold(look_of_journals(&[journal]), root.as_path());
    let Some(holding) = held else { panic!("a working folder that cannot be read leaves the question open") };
    assert_eq!(holding.because, Hold::HelperUnsure);
    let Some(program) = holding.program else { panic!("the program that cannot be ruled out is named: {holding:?}") };
    assert_eq!((program.pid, program.journal_word_is_relative), (FAKE_PID, true));
    assert!(program.reason.contains("working folder could not be read"), "the cause is said in plain words: {}", program.reason);
    assert!(program.reason.contains("No such file or directory"), "and with what the kernel answered: {}", program.reason);
    remove(fixture.as_path());
}

/// What:
///  A relative word that climbs out of a working folder the kernel reports as deleted is not settled:
///  it comes back as a program that may be the earlier helper, with the process named.
/// Why:
///   A program started with `--journal ../cli-git/service` from a folder that was removed afterwards can still reach the
///  shared earlier journal, and `/proc/<pid>/cwd` then reads `<folder> (deleted)`.
/// The file system cannot resolve a
///  path below a folder that is gone,
/// so the path it names cannot be compared with this worktree's journals,
/// and an open
///  question must not settle a batch a running helper may write.
/// The kernel's ` (deleted)` marker is stated here as the link's text,
/// which is what
///  `read_link` answers for a deleted working folder.
#[test]
#[cfg(unix)]
fn a_relative_journal_word_that_climbs_out_of_a_deleted_working_folder_holds() {
    let fixture: PathBuf = fixture("look-relative-deleted-folder");
    let journal: PathBuf = earlier_journal_below(fixture.as_path());
    let deleted: PathBuf = fixture.join("a-working-folder (deleted)");
    assert!(!deleted.exists(), "the working folder the kernel reports is gone");
    let root: PathBuf = proc_root_with(&[String::from("/p/cli-git-native"), String::from("--journal"), format!("../{EARLIER_JOURNAL_TAIL}")]);
    with_working_folder(root.as_path(), deleted.as_path());
    let held: Option<Holding> = earlier_service_process_hold(look_of_journals(&[journal]), root.as_path());
    let Some(holding) = held else { panic!("a word below a deleted working folder cannot be shown to name no journal of this worktree") };
    assert_eq!(holding.because, Hold::HelperUnsure, "the question stays open: {holding:?}");
    let Some(program) = holding.program else { panic!("the program that cannot be ruled out is named: {holding:?}") };
    assert_eq!(program.pid, FAKE_PID);
    assert!(program.journal_word_is_relative, "the word is relative, and the words say which folder it is read in");
    remove(fixture.as_path());
}
