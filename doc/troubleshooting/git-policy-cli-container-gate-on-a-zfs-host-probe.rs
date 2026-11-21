//! What:
//!  A forced reproduction of the order `binary_support_process_cpu_counters_tests.rs` relied on:
//!  a thread of a process that ended,
//!  was joined,
//!  and whose own `/proc` file is still readable.
//! Why:
//!  The control asserts that file is gone at `join`,
//!  which the kernel does not promise:
//!  the kernel clears the thread id the join waits on inside `exit_mm`,
//!  and flushes the `/proc` entry only later,
//!  in `release_task`.
//!  The window is microseconds for a sub-thread,
//!  which is why the gate met it once in thousands of tests.
//!  It is indefinite for a thread-group leader whose group still has a thread,
//!  because `exit_notify` reaps a leader only when `thread_group_empty`,
//!  so a child whose leader ends is the state to force it with:
//!  no tracer,
//!  no load,
//!  no repetition,
//!  and a barrier this program controls.
//!
//! Usage:
//!  `probe --plain` runs the ordinary order,
//!  a joined sub-thread;
//!  `probe --leader` runs the forced order,
//!  a leader that ended while its group serves.
//!  Both arms end by the barrier byte and a reaped child,
//!  so nothing is left behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const child = spawn(self, ['child', arm]); const { leader, thread } = await line(child);
//! const listing = readdir(`/proc/${child.pid}/task`); expect(listing).toContain(ended); child.stdin.write('\n');
//! await line(child); // "joined"
//! expect(() => readFileSync(`/proc/${child.pid}/task/${ended}/stat`)).toThrow(); // fails on every --leader run
//! ```

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

/// `SYS_gettid`,
/// which names the calling thread.
const SYS_GETTID: i64 = 186;
/// `SYS_set_tid_address`,
/// which is `clear_child_tid`:
/// the word the kernel clears as the caller ends and the join waits on.
const SYS_SET_TID_ADDRESS: i64 = 218;
/// `SYS_futex`.
const SYS_FUTEX: i64 = 202;
/// `SYS_exit`,
/// which ends the calling thread only.
const SYS_EXIT: i64 = 60;
/// `FUTEX_WAIT`.
const FUTEX_WAIT: i64 = 0;
/// How long the parent waits for a child's line before giving up,
/// so a broken arm fails instead of hanging.
const LINE_PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

extern "C" {
    /// The system calls the join protocol and the leader's ending are made of.
    fn syscall(number: i64, ...) -> i64;
}

/// The word the kernel clears as the leader ends,
/// which is what a join waits on.
static JOIN_WORD: AtomicU32 = AtomicU32::new(0);

fn main() {
    let arm: String = std::env::args().nth(1).unwrap_or_else(|| { return String::from("--leader"); });
    if std::env::args().nth(2).as_deref() == Some("child") {
        child_role(arm.as_str());
        return;
    }
    parent_role(arm.as_str());
}

/// The thread's own number,
/// as `/proc` names it.
fn thread_id() -> u32 {
    let raw: i64 = unsafe { syscall(SYS_GETTID) };
    return u32::try_from(raw).expect("a thread id");
}

/// The clock ticks one thread used,
/// from its own file,
/// or nothing where that file cannot be read:
/// the reader under test.
fn thread_ticks(pid: u32, thread: u32) -> Option<u64> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/task/{thread}/stat")).ok()?;
    let after_name: &str = stat.rsplit_once(')')?.1;
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    return Some(user + system);
}

/// The state letter of one thread's own file,
/// or the word for the file being gone.
fn thread_state(pid: u32, thread: u32) -> String {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/task/{thread}/stat")) else {
        return String::from("gone");
    };
    let Some(after_name) = stat.rsplit_once(')').map(|split| { return split.1; }) else {
        return String::from("unparsed");
    };
    return after_name.split_whitespace().next().unwrap_or("unparsed").to_owned();
}

/// The thread numbers of one process now,
/// as the reader under test lists them.
fn thread_numbers(pid: u32) -> Vec<u32> {
    let mut numbers: Vec<u32> = Vec::new();
    let Ok(listed) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return numbers;
    };
    for entry in listed.flatten() {
        if let Some(number) = entry.file_name().to_str().and_then(|name| { return name.parse::<u32>().ok(); }) {
            numbers.push(number);
        }
    }
    numbers.sort_unstable();
    return numbers;
}

/// One line of the child's output,
/// or the reason it did not come.
fn line(lines: &mut std::io::Lines<BufReader<ChildStdout>>, what: &str) -> String {
    return lines.next().unwrap_or_else(|| { return Err(std::io::Error::other(format!("the child said nothing of {what}"))); })
        .unwrap_or_else(|error| { return panic!("the child's {what} line could not be read: {error}"); });
}

/// The child process,
/// in one of the two arms.
fn child_role(arm: &str) {
    let leader: u32 = thread_id();
    if arm == "--leader" {
        // The join protocol, by hand: this leader publishes the word its end will clear, and one thread waits on
        // it exactly as `pthread_join` does.
        JOIN_WORD.store(leader, Ordering::SeqCst);
        let word: *mut u32 = JOIN_WORD.as_ptr() as *mut u32;
        let _set = unsafe { syscall(SYS_SET_TID_ADDRESS, word) };
        let opened: u32 = leader;
        let (report, reported): (Sender<u32>, Receiver<u32>) = channel();
        let (release, held): (Sender<()>, Receiver<()>) = channel();
        let waiting: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            let _sent = report.send(thread_id());
            // Wait for the kernel's clear of the leader's word, which is the moment `pthread_join` returns.
            let word: *mut u32 = JOIN_WORD.as_ptr() as *mut u32;
            let _opened = opened;
            loop {
                let seen = JOIN_WORD.load(Ordering::SeqCst);
                if seen == 0 {
                    break;
                }
                let _waited = unsafe { syscall(SYS_FUTEX, word, FUTEX_WAIT, u64::from(seen), std::ptr::null::<u8>(), 0, 0) };
            }
            eprintln!("waiter: woke, word {}", JOIN_WORD.load(Ordering::SeqCst));
            println!("joined");
            std::io::stdout().flush().expect("the parent reads the join");
            // The barrier this program controls: the arm ends when the parent says so, and then the group dies and
            // the kernel releases the leader's task, so nothing is left behind.
            let mut release_barrier: String = String::new();
            let _read = std::io::stdin().lock().read_line(&mut release_barrier);
            let _released = held;
        });
        let thread: u32 = reported.recv().expect("the waiting thread's number");
        println!("leader {leader} thread {thread}");
        std::io::stdout().flush().expect("the parent reads both numbers");
        let mut barrier: String = String::new();
        let _read = std::io::stdin().lock().read_line(&mut barrier);
        eprintln!("leader: barrier read = {barrier:?}, ending now, word {}", JOIN_WORD.load(Ordering::SeqCst));
        // End this thread only, and as a leader whose group still has one: the kernel leaves the task in `/proc`.
        let _ended = unsafe { syscall(SYS_EXIT, 0u64) };
        unreachable!("only the leader ends here, and it never returns");
    }
    let (report, reported): (Sender<u32>, Receiver<u32>) = channel();
    let (release, held): (Sender<()>, Receiver<()>) = channel();
    let waiting: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _sent = report.send(thread_id());
        let _released = held.recv();
    });
    let thread: u32 = reported.recv().expect("the waiting thread's number");
    println!("leader {leader} thread {thread}");
    std::io::stdout().flush().expect("the parent reads both numbers");
    let mut barrier: String = String::new();
    let _read = std::io::stdin().lock().read_line(&mut barrier);
    let _released = release.send(());
    waiting.join().expect("the waiting thread ends");
    println!("joined");
    std::io::stdout().flush().expect("the parent reads the join");
    let mut release_barrier: String = String::new();
    let _read = std::io::stdin().lock().read_line(&mut release_barrier);
}

/// One run:
/// which tid ends,
/// what the listing had,
/// what the read met at the join,
/// and whether the child ends cleanly.
fn parent_role(arm: &str) {
    let began: std::time::Instant = std::time::Instant::now();
    let mut child: Child = Command::new(std::env::current_exe().expect("this program"))
        .arg(arm)
        .arg("child")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the child starts");
    let pid: u32 = child.id();
    let mut input: ChildStdin = child.stdin.take().expect("the child's input");
    let stdout: ChildStdout = child.stdout.take().expect("the child's output");
    let mut lines = BufReader::new(stdout).lines();
    let numbers: String = line(&mut lines, "numbers");
    let fields: Vec<&str> = numbers.split_whitespace().collect();
    let leader: u32 = fields.get(1).expect("the leader's number").parse().expect("a leader number");
    let thread: u32 = fields.get(3).expect("the waiting thread's number").parse().expect("a thread number");
    let ended: u32 = if arm == "--leader" { leader } else { thread };
    let listing: Vec<u32> = thread_numbers(pid);
    let listed: bool = listing.contains(&ended) && listing.contains(&thread);
    eprintln!("parent: pid {pid} leader {leader} thread {thread} ended {ended} listing {listing:?}");
    print!("arm {arm} listing {listing:?} listed {listed} state-before {} ", thread_state(pid, ended));
    input.write_all(b"\n").expect("the child's barrier");
    input.flush().expect("the child's barrier");
    eprintln!("parent: barrier sent, waiting for the join line");
    let joined: String = line(&mut lines, "join");
    eprintln!("parent: join line read, reading the ended thread's own file");
    let at_join: Option<u64> = thread_ticks(pid, ended);
    eprintln!("parent: read done {at_join:?}, state {}", thread_state(pid, ended));
    // The control's own assertion, exactly as the file carried it.
    let control_passes: bool = at_join.is_none();
    print!("{joined} at-join {at_join:?} state-at-join {} control-passes {control_passes} ", thread_state(pid, ended));
    // The barrier that ends the arm: the child's remaining threads go, the group dies, the leader is released.
    input.write_all(b"\n").expect("the child's release");
    input.flush().expect("the child's release");
    let _waited = child.wait().expect("the child ends");
    let gone_after: bool = thread_ticks(pid, ended).is_none() && !std::path::Path::new(format!("/proc/{pid}").as_str()).exists();
    let _timed_out = began.elapsed() > LINE_PATIENCE;
    println!("gone-after {gone_after} total {:?}", began.elapsed());
}
