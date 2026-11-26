//! What:
//!  The monitor thread of the hang guard:
//!  one thread for the whole test process that finds the shared-bound commands among the test's own children,
//!  looks at every watched command,
//!  ends the ones the judge gives a reason for,
//!  aborts on a kill nobody claimed,
//!  and the call that puts one command under it.
//! Why:
//!  One monitor per test process watches every way a test runs a command,
//!  whether through `observe_raw` or by starting it itself,
//!  so no control needs to remember to arm a bound.
//!  A child between `fork` and `exec` still shows the test's own command line and `exec` keeps its process ID and start
//!  time,
//!  so nothing is remembered about a child that is not a shared-bound command:
//!  the verdict about it is read again at every look.
//!  The look itself is `binary_supervisor_proc.rs`,
//!  the judgement is `binary_supervisor_judge.rs`,
//!  and the record of an ending is `binary_supervisor_kills.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! setInterval(() => { discover(ownPid); for (const pid of watchedIds) judgeOne(pid); abortOnUnclaimed(now); }, POLL);
//! ```
use super::supervisor_judge::Judge;
use super::supervisor_kills::{
    abort_on_unclaimed,
    end_command,
    fail_unclaimed_at_exit,
};
use super::supervisor_limits::{
    Limits,
    POLL,
    Reason,
    SHARED,
};
use super::supervisor_pin::{
    Pin,
    end_tree,
};
use super::supervisor_proc::{
    Look,
    Sample,
    children,
    command_line_of,
    describe,
    directory_of,
    look_at,
    stat_of,
    tick,
};
use super::supervisor_registry::{
    Watched,
    with_registry,
};
use super::supervisor_shared_bound_commands::is_shared_command_line;
use std::sync::Once;
use std::time::{
    Duration,
    Instant,
};

/// Start the monitor thread of this process,
/// once,
/// and register the exit handler.
pub fn ensure_monitor() {
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        std::thread::Builder::new()
            .name(String::from("binary-supervisor"))
            .spawn(monitor)
            .expect("the supervisor thread starts");
        // SAFETY:
        // the handler is an `extern "C"` function with no arguments,
        // as `atexit` requires.
        unsafe { libc::atexit(fail_unclaimed_at_exit) };
    });
}

/// Ask the monitor to watch the child `pid` with `limits`,
/// however the test runs it (the controls of the monitor itself).
///  The child is pinned now,
/// while the test still holds it unreaped,
/// and entered into the registry directly,
/// so no look of the
///  monitor that came before can have written it off.
pub fn watch(
    pid: u32,
    limits: Limits,
) {
    let watched: i32 = i32::try_from(pid).expect("a process ID fits");
    ensure_monitor();
    let started: u64 = stat_of(watched)
        .expect("the child to watch is running")
        .started;
    let pin: Pin = Pin::open(watched, started).expect("the child to watch can be pinned");
    with_registry(|registry| {
        registry.watched.insert(
            watched,
            Watched {
                judge: Judge::new(limits, tick()),
                began: Instant::now(),
                pin,
            },
        );
    });
}

/// The body of the monitor thread:
/// forever,
/// look at the test's own children.
fn monitor() {
    let own: i32 = i32::try_from(std::process::id()).expect("a process ID fits");
    let tick: Duration = tick();
    loop {
        std::thread::sleep(POLL);
        discover(own, tick);
        let now: Instant = Instant::now();
        let ids: Vec<i32> = with_registry(|registry| {
            return registry.watched.keys().copied().collect();
        });
        for pid in ids {
            judge_one(pid, now);
        }
        abort_on_unclaimed(now);
    }
}

/// Start watching the children that are shared-bound commands.
/// Nothing is remembered about a child that is not one:
/// a child
///  between `fork` and `exec` still has the test's own command line,
/// and `exec` keeps its process ID and start time,
/// so a
///  verdict about it is read again at every look.
fn discover(
    own: i32,
    tick: Duration,
) {
    for pid in children(own) {
        let Some(stat) = stat_of(pid) else { continue };
        if stat.parent != own
            || with_registry(|registry| {
                return registry.watched.get(&pid).is_some_and(|watched| {
                    return watched.pin.started() == stat.started;
                });
            })
        {
            continue;
        }
        let line: Vec<String> = command_line_of(pid);
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        if !is_shared_command_line(words.as_slice()) {
            continue;
        }
        let Some(pin) = Pin::open(pid, stat.started) else {
            continue;
        };
        with_registry(|registry| {
            registry.watched.insert(
                pid,
                Watched {
                    judge: Judge::new(SHARED, tick),
                    began: Instant::now(),
                    pin,
                },
            );
        });
    }
}

/// Look at one watched command and end it if the judge says so.
fn judge_one(
    pid: i32,
    now: Instant,
) {
    let Some(started) = with_registry(|registry| {
        return registry.watched.get(&pid).map(|watched| {
            return watched.pin.started();
        });
    }) else {
        return;
    };
    let sample: Sample = match look_at(pid, started) {
        // The command is over;
        // the entry goes,
        // and a kill that was recorded for it stays for its test.
        Look::Gone | Look::Exited => {
            with_registry(|registry| {
                if registry.watched.get(&pid).is_some_and(|watched| {
                    return watched.pin.started() == started;
                }) {
                    registry.watched.remove(&pid);
                }
            });
            return;
        }
        Look::Seen(seen) => seen,
    };
    let verdict: Option<Reason> = with_registry(|registry| {
        *registry.looks.entry(pid).or_default() += 1;
        if sample.disk_wait {
            *registry.disk_wait_looks.entry(pid).or_default() += 1;
        }
        let held: bool = registry.holds.contains_key(&pid);
        let watched: &mut Watched = registry.watched.get_mut(&pid)?;
        let began: Instant = watched.began;
        return watched
            .judge
            .observe(now.saturating_duration_since(began), &sample, held);
    });
    let Some(reason) = verdict else { return };
    let line: String = command_line_of(pid).join(" ");
    let message: String = format!(
        "the harness supervisor ended the command `{line}` in {}: {reason}\nthe tasks of its tree when it was ended:\n{}",
        directory_of(pid),
        describe(pid)
    );
    let Some(watched) = with_registry(|registry| {
        return registry.watched.remove(&pid);
    }) else {
        return;
    };
    end_command(&watched.pin, message.as_str(), &end_tree);
    eprintln!("{message}");
}
