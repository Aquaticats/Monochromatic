//! Controls for the line that names a kept copy a program holds open.
//! Why:
//!  The name in that line is the one a program chose for itself (`/proc/<pid>/comm`),
//!  and the line is the only pointer a person has to which program holds the copy,
//!  so the name must reach the terminal as text that can act on nothing (`AGENTS.md`, rule `SYB`).
//!  The body runs in a helper process of its own (`helper_process_test_support`),
//!  because the name this control judges is that whole process's,
//!  and a threaded run shares one process between its controls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! setProcessName('\u001b[2K'); expect(sayTheCopyStaysHeldOpen(copy, scanOf(theFile))).not.toContain('\u001b');
//! ```

use super::*;
use crate::helper_process_test_support::{
    helper_directory,
    in_a_helper,
    verdict_in_a_helper,
};
use crate::index_lock_holders::{
    Holder,
    scan_proc_fd_holders_until,
};
use crate::merge_isolation_kept_copies::KeptCopy;
use crate::plain_file::open_plain_file;
use std::path::PathBuf;
use std::time::{
    Duration,
    Instant,
};

/// What:
///  How long this control looks for a scan that names this process holding the copy,
///  and how often it looks.
/// Why:
///  One walk of the process table stops at its own bound,
///  so a look may end before it reaches the process that started last;
///  the wait is stated here, so a host that is slow fails the control with what it waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WINDOW = 30_000; const LOOK_EVERY = 50;
/// ```
const WINDOW: Duration = Duration::from_secs(30);
const LOOK_EVERY: Duration = Duration::from_millis(50);

/// What:
///  The variable that makes a run of this control the helper it starts.
/// Why:
///  The helper runs this same test function,
///  so one variable is what tells the parent and the body apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HELPER = 'CLI_GIT_TEST_ONLY_KEPT_COPY_NAME_HELPER';
/// ```
const HELPER: &str = "CLI_GIT_TEST_ONLY_KEPT_COPY_NAME_HELPER";

/// What:
///  The name this control gives the helper's process:
///  a terminal escape that erases the line,
///  a line break that would start a line this program never wrote,
///  a closing parenthesis that would end the parentheses around the name,
///  and a quote.
/// Why:
///  Those are the shapes the repository's security rules list for text that crosses into a line,
///  and they fit the fifteen bytes a process name may hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAME_A_PROGRAM_CHOSE = '\u001b[2K\n)"';
/// ```
const NAME_A_PROGRAM_CHOSE: &str = "\u{1b}[2K\n)\"";

/// What:
///  The sentences that name this process holding the copy, when it holds it.
/// Why:
///  The parent's whole assertion is that the one test the helper ran passed,
///  and a control that finds no holder must fail rather than judge an empty line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theHolder(): Holder[]; // the scan's holders, which must name this process
/// ```
fn the_holder_of_this_process(
    device: u64,
    inode: u64,
) -> Option<Vec<Holder>> {
    // What: how long one walk of the process table may take here.
    // Why:  the release's own scan is bounded the same way, and the bound is what keeps one look from holding the control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const deadline = Date.now() + 2_000;
    // ```
    let deadline: Instant = Instant::now() + crate::index_lock_holders::SERVICE_SCAN_BUDGET;
    // What: how the walk is told which process to leave out, which is none here.
    // Why:  the holder this control proves is this very process, so a walk that skipped it would name nobody holding the copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const LEAVE_OUT_NO_PROCESS = 0; // no process has this number
    // ```
    const LEAVE_OUT_NO_PROCESS: u32 = 0;
    // What: the scan's answer for the copy's own identity.
    // Why:  the release line is written from this scan, so the control reads the holder the way the release does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const evidence = scanProcFdHoldersUntil('/proc', device, inode, LEAVE_OUT_NO_PROCESS, deadline);
    // ```
    let evidence = scan_proc_fd_holders_until(
        std::path::Path::new("/proc"),
        device,
        inode,
        LEAVE_OUT_NO_PROCESS,
        Some(deadline),
    );
    // What: this process's own number, which the scan must name, because it is the holder.
    // Why:  a scan that named no process would let the sentence prove nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const own = process.pid;
    // ```
    let own: i64 = i64::from(std::process::id());
    if !evidence
        .holders
        .iter()
        .any(|holder| return holder.pid == own)
    {
        return None;
    }
    return Some(evidence.holders);
}

/// What:
///  The sentences this control's helper failed with:
///  the line a kept copy stays with,
///  written for the holder this process is,
///  with the name the kernel holds for it.
/// Why:
///  The name is read where a program really is,
///  through the scan the release code uses,
///  so no hand-made holder can hide the bytes a kernel would hand over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theNameNeverReachesTheLine(): string[];
/// ```
fn the_name_never_reaches_the_line(directory: &std::path::Path) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    // What: the copy the sentence names, written where only this fixture reads it.
    // Why:  the scan answers for one open file, and the file has to be this control's own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const copyPath = join(directory, 'the-copy'); writeFileSync(copyPath, 'bytes\n');
    // ```
    let copy_path: PathBuf = directory.join("the-copy");
    std::fs::write(copy_path.as_path(), b"bytes\n").expect("the copy");
    // What: the copy held open by this helper, so the scan finds a holder.
    // Why:  a descriptor has to live until the scan reads the process table, and the file is opened as a plain one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = openPlainFile(copyPath);
    // ```
    let held: std::fs::File = open_plain_file(copy_path.as_path()).expect("the copy held open");
    // What: the file's identity as the kernel states it: the device and the inode a scan matches.
    // Why:  a path is not an identity, and the scan answers for one open file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const meta = fstatSync(held);
    // ```
    use std::os::unix::fs::MetadataExt as _;
    let meta: std::fs::Metadata = held.metadata().expect("the identity of the held copy");
    // What: whether the kernel took the name this control gives this process.
    // Why:  a process's name is what the line prints, and a refusal would leave the control judging a name nobody set.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!renameThisProcess(NAME_A_PROGRAM_CHOSE)) { failures.push('the kernel refused the name'); return failures; }
    // ```
    if !rename_this_process(NAME_A_PROGRAM_CHOSE) {
        failures.push(String::from("the kernel refused this process's name"));
        return failures;
    }
    // What: the holders of the copy, once a look of the process table names this process.
    // Why:  the scan is bounded and walks newest first, so one look may end before it reads this process,
    //  and the wait is bounded and fails saying what it waited for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let holders = undefined; const began = Date.now(); while (holders === undefined && Date.now() - began < WINDOW) { holders = theHolderOfThisProcess(copyPath, meta.dev, meta.ino); if (holders === undefined) sleep(LOOK_EVERY); }
    // ```
    let began: Instant = Instant::now();
    let mut holders: Option<Vec<Holder>> = None;
    while began.elapsed() < WINDOW {
        holders = the_holder_of_this_process(meta.dev(), meta.ino());
        if holders.is_some() {
            break;
        }
        std::thread::sleep(LOOK_EVERY);
    }
    let Some(holders) = holders else {
        failures.push(format!(
            "no look of the process table named this process holding the copy within {} s",
            WINDOW.as_secs()
        ));
        return failures;
    };
    // What: the copy the sentence is about, with paths of the fixture's own.
    // Why:  the sentence names the copy and the file, and neither is what this control judges.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const copy = { location: join(directory, 'a.txt'), aside: copyPath };
    // ```
    let copy: KeptCopy = KeptCopy {
        location: directory.join("a.txt"),
        aside: copy_path.clone(),
    };
    // What: the line a person reads, written by the release code itself.
    // Why:  the control judges the bytes a terminal would take, not a sentence of its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const out = []; sayTheCopyStaysHeldOpen(out, copy, holders); const line = out.join('');
    // ```
    let mut out: Vec<u8> = Vec::new();
    say_the_copy_stays_held_open(&mut out, &copy, holders.as_slice());
    let line: String = String::from_utf8(out).expect("the line is text");
    // What: the line as a terminal takes it: the sentence's own final newline ends it, and nothing else ends a line.
    // Why:  the sentence ends with one newline of its own, so a control character anywhere before it is the name's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const body = line.replace(/\n$/, '');
    // ```
    let body: &str = line.strip_suffix('\n').unwrap_or(line.as_str());
    if body.chars().any(char::is_control) {
        failures.push(format!(
            "a control character of the name reached the line: {line:?}"
        ));
    }
    if line.matches('\n').count() != 1 || !line.ends_with('\n') {
        failures.push(format!(
            "the line is the one line a terminal reads: {line:?}"
        ));
    }
    if !line.contains("\\u{1b}[2K\\n\\)") {
        failures.push(format!(
            "the name is shown with its escape, its line break and its parenthesis spelled out: {line:?}"
        ));
    }
    if !line.contains(format!("PID {}", std::process::id()).as_str()) {
        failures.push(format!(
            "the line names which process holds the copy: {line:?}"
        ));
    }
    return failures;
}

/// A name a program chose cannot end, erase or add to the line that says a kept copy stays,
/// and the person still reads which program holds it.
#[test]
fn a_name_a_program_chose_cannot_rewrite_the_copy_s_line() {
    // What: the fixture directory, when this process is the helper the parent started.
    // Why:  the body below is the same test function as the parent's, and only the helper holds this variable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (directory) { process.exit(verdictInAHelper(directory, theNameNeverReachesTheLine(directory))); }
    // ```
    if let Some(directory) = helper_directory(HELPER) {
        std::process::exit(verdict_in_a_helper(
            directory.as_path(),
            the_name_never_reaches_the_line(directory.as_path()),
        ));
    }
    // What: the sentences the helper failed with, which are this control's whole assertion.
    // Why:  the body runs where the process's name is its own, and its words come back to this run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const failures = inAHelper(HELPER, 'kept-copy-a-name-a-program-chose', 'merge::merge_isolation_kept_copies_words::tests::a_name_a_program_chose_cannot_rewrite_the_copy_s_line');
    // ```
    let failures: Vec<String> = in_a_helper(
        HELPER,
        "kept-copy-a-name-a-program-chose",
        "merge::merge_isolation_kept_copies_words::tests::a_name_a_program_chose_cannot_rewrite_the_copy_s_line",
    );
    assert!(failures.is_empty(), "{failures:?}");
}

/// What:
///  Whether the kernel took `name` as this process's name.
/// Why:
///  A process's name is what the line prints,
///  and only the process itself may set it (`/proc/<pid>/comm`),
///  which is why this control's body runs in a helper process of its own.
/// Gotcha:
///  The kernel keeps what one write sets,
///  so the name goes in one write:
///  a shell's `printf` writes a line at a time and would leave only the tail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync('/proc/self/comm', name); // one write, or the kernel keeps only the last part
/// ```
fn rename_this_process(name: &str) -> bool {
    return std::fs::write("/proc/self/comm", name.as_bytes()).is_ok();
}
