//! The bound one command holds when it finishes a copy it did not start:
//! an unbounded copy never stops,
//! a bounded one stops once its time is up,
//! and the words a stopped copy is reported with name the bound.

use super::*;

/// A copy a creating command makes is held by no ceiling,
/// whatever time passes.
#[test]
fn a_copy_no_ceiling_holds_never_reaches_one() {
    let unbounded: CopyCeiling = CopyCeiling::unbounded();
    assert!(!unbounded.reached(), "a copy the add makes is its own work and is waited for");
}

/// A bounded copy stops once the span it was given is over,
/// and not before it starts.
#[test]
fn a_bounded_copy_stops_when_its_span_is_over() {
    let at_once: CopyCeiling = CopyCeiling::lasting(Duration::ZERO);
    assert!(at_once.reached(), "a copy with no time left stops before its first entry");
    let ample: CopyCeiling = CopyCeiling::lasting(Duration::from_secs(60));
    assert!(!ample.reached(), "a copy with a minute ahead of it goes on");
}

/// The control's shortened bound reaches the copies the thread that set it starts,
/// and leaves the next thread's alone.
#[test]
fn a_shorter_ceiling_bounds_the_copies_this_thread_starts() {
    {
        let _shorter: ShorterCeilingForTheControl = a_shorter_ceiling_for_this_thread(Duration::ZERO);
        assert!(CopyCeiling::lasting(OWED_COPY_CEILING).reached(), "the control's bound stands in for the shipped one");
        let other: std::thread::JoinHandle<bool> = std::thread::spawn(|| { return CopyCeiling::lasting(OWED_COPY_CEILING).reached(); });
        assert!(!other.join().expect("the other thread"), "another thread keeps the shipped bound");
    }
    assert!(!CopyCeiling::lasting(OWED_COPY_CEILING).reached(), "the shipped bound is back once the control is dropped");
}

/// What a stopped copy says names the span it was given,
/// and that the worktree it was filling still misses its ignored files.
#[test]
fn a_stopped_copy_names_the_ceiling_and_the_missing_files() {
    let words: String = CopyCeiling::lasting(Duration::from_secs(360)).stopped_sentence();
    assert!(words.contains("the 360-second ceiling one command holds for a copy it did not start"), "{words}");
    assert!(words.contains("ignored files are still missing"), "{words}");
    assert_eq!(CopyCeiling::lasting(Duration::from_secs(360)).stopped_error().0, words, "the failure is the same sentence");
}
