//! What:
//!  The threads that read this run,
//! as the kernel numbers them,
//! and how many waits each is inside.
//! Why:
//!   A look reads the counters of every task,
//! and those reads are the reader's own work,
//! not the run's,
//! so every look must leave out every reader of this process and not only its own thread.
//! The registry is one per process and the per-thread figure of readings is one per thread,
//! so a look a control takes without a registration of its own still counts what its reads moved and takes them off its next look.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // readers.ts: export const begin: () => Reading | undefined; export const threads: () => number[];
//! ```

/// The kernel's number for the calling thread,
/// which the look leaves out so its own reading never counts as the run's work.
use crate::thread_quiet::current_thread_id;

/// What:
///  The threads that read this run,
/// as the kernel numbers them:
/// every watcher,
/// and every thread inside a
///  control's wait.
/// Why:
///   A reader reads the counters of every task,
/// and those reads are its own work,
/// not the run's;
/// a second reader
///  (a control arms a watcher beside the fixture's,
/// or waits beside another control's wait)
/// would otherwise count the other's
///  reads at every look,
/// so two of them would keep each other,
/// and the run they read,
/// alive for ever.
static READING_THREADS: std::sync::Mutex<Vec<Reader>> = std::sync::Mutex::new(Vec::new());

/// What:
///  A thread that reads this run,
///  and how many waits it is inside.
/// Why:
///   One thread may be inside more than one wait at a time (a wait whose event another wait reports),
/// and a second entry for it would be
///  read as a second watcher whose readings were never counted,
/// so its whole counter would look like the run's work at every look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reader = { thread: number; waits: number };
/// ```
#[derive(Clone, Copy)]
pub(super) struct Reader {
    /// The thread the kernel numbers for the reader.
    pub(super) thread: i32,
    /// How many registrations of this thread are still live.
    waits: usize,
}

thread_local! {
    /// What:
    ///  Every byte the reads this thread's own looks have made of `/proc` have moved.
    /// Why:
    ///   The counter a look reads is cumulative,
    /// so what a look reads is taken off the next one,
    /// exactly,
    /// including the one reading each look makes to learn the counter itself.
    /// The figure belongs to the thread and not to a registration of it:
    ///  a look a control takes without keeping a registration of its own (`look_now`)
    /// would otherwise forget its readings when the registration that covered it ended,
    /// and every look would count them as the run's work,
    /// so a run that had stopped working would never be judged quiet.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // let ownReadings = 0; // one per thread
    /// ```
    pub(super) static OWN_READINGS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What:
///  A thread registered as a reader of this run,
/// for as long as the value lives.
/// Why:
///   The reads of a waiter happen between its looks as well as inside them,
/// and a look taken meanwhile must
///  leave them out too,
/// so the registration lasts the whole wait rather than one reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using reading = Reading.begin(); // unregistered when the block ends
/// ```
pub(crate) struct Reading {
    /// The thread the kernel numbers for the reader.
    thread: i32,
}

impl Reading {
    /// What:
    ///  Register the calling thread as a reader of this run,
    ///  or nothing where this system numbers no thread,
    /// in which case no look can read the run
    ///  either.
    /// Why:
    ///   Every reader registers,
    /// so no look taken by any of them counts another's readings as the run's work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const reading = Reading.begin();
    /// ```
    pub(crate) fn begin() -> Option<Reading> {
        let thread: i32 = current_thread_id()?;
        let Ok(mut reading) = READING_THREADS.lock() else {
            return None;
        };
        for each in reading.iter_mut() {
            if each.thread == thread {
                each.waits = each.waits.saturating_add(1);
                return Some(Reading {
                    thread,
                });
            }
        }
        reading.push(Reader {
            thread,
            waits: 1,
        });
        return Some(Reading {
            thread,
        });
    }
}

/// A reader that ends,
/// however its thread ends,
/// is no longer left out.
impl Drop for Reading {
    fn drop(&mut self) {
        let Ok(mut reading) = READING_THREADS.lock() else {
            return;
        };
        let Some(at) = reading.iter().position(|each: &Reader| {
            return each.thread == self.thread;
        }) else {
            return;
        };
        let ending: &mut Reader = &mut reading[at];
        ending.waits = ending.waits.saturating_sub(1);
        if ending.waits == 0 {
            let _gone: Reader = reading.remove(at);
        }
    }
}

/// The threads a look leaves out,
/// which is every reader of this process,
/// each once.
pub(crate) fn reading_threads() -> Vec<i32> {
    let Ok(reading) = READING_THREADS.lock() else {
        return Vec::new();
    };
    return reading
        .iter()
        .map(|each: &Reader| {
            return each.thread;
        })
        .collect();
}

/// The readers of this process,
/// one entry per thread.
pub(super) fn the_readers() -> Vec<Reader> {
    let Ok(reading) = READING_THREADS.lock() else {
        return Vec::new();
    };
    return reading.clone();
}
