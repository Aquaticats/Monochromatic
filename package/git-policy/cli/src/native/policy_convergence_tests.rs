//! What:
//!  Every ending of the bounded correction loop,
//!  driven by a scripted implementer.
//! Why:
//!  The loop decides whether corrected content is accepted.
//!  A wrong bound accepts
//!      content that never settled or refuses content that did;
//!  a missed cycle spins.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(converge(scripted({ passes: ['proposed', 'stable'], states: ['a', 'b'] }))).toEqual({ kind: 'settled', changedPasses: 1 });
//! ```

/// The loop under test and its interface.
use super::{
    Convergence, FIX_CYCLE_MESSAGE, FIX_PASS_LIMIT_MESSAGE, FixPasses, MAXIMUM_CHANGED_PASSES,
    PassResult, converge,
};

/// An implementer that answers from a script and logs every call.
struct Scripted {
    /// What the pass with each candidate version returns.
    passes: Vec<PassResult>,
    /// The exact state stored under each snapshot number;
    ///  number 0 is the initial state.
    states: Vec<&'static str>,
    /// The snapshot number whose `apply` fails,
    ///  when one does.
    failing_apply: Option<usize>,
    /// Every `run_pass` and `apply` in call order.
    log: Vec<String>,
}

impl FixPasses for Scripted {
    fn run_pass(&mut self, candidate_version: usize) -> PassResult {
        self.log.push(format!("pass {candidate_version}"));
        return self.passes[candidate_version];
    }

    fn apply(&mut self, snapshot: usize) -> bool {
        self.log.push(format!("apply {snapshot}"));
        return self.failing_apply != Some(snapshot);
    }

    fn snapshots_equal(&mut self, left: usize, right: usize) -> bool {
        return self.states[left] == self.states[right];
    }
}

/// Run the loop over a script and return its ending with the call log.
fn run(
    passes: &[PassResult],
    states: &[&'static str],
    failing_apply: Option<usize>,
) -> (Convergence, Vec<String>) {
    let mut scripted: Scripted = Scripted {
        passes: passes.to_vec(),
        states: states.to_vec(),
        failing_apply,
        log: Vec::<String>::new(),
    };
    let ending: Convergence = converge(&mut scripted);
    return (ending, scripted.log);
}

/// Nine distinct states:
///  the initial one and one per allowed change,
///  plus one the loop must never store.
const DISTINCT: [&str; 10] = ["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9"];

/// `count` proposing passes followed by `last`.
fn proposing(count: usize, last: PassResult) -> Vec<PassResult> {
    let mut passes: Vec<PassResult> = vec![PassResult::Proposed; count];
    passes.push(last);
    return passes;
}

/// A first pass that proposes nothing settles without applying anything.
#[test]
fn stable_first_pass_settles_at_once() {
    assert_eq!(
        run(&[PassResult::Stable], &["a"], None),
        (
            Convergence::Settled { changed_passes: 0 },
            vec![String::from("pass 0")]
        )
    );
}

/// Each changed state restarts the pass with the next candidate version until a pass is stable.
#[test]
fn changes_restart_the_pass_until_stable() {
    assert_eq!(
        run(
            &[
                PassResult::Proposed,
                PassResult::Proposed,
                PassResult::Stable
            ],
            &["a", "b", "c"],
            None
        ),
        (
            Convergence::Settled { changed_passes: 2 },
            vec![
                String::from("pass 0"),
                String::from("apply 1"),
                String::from("pass 1"),
                String::from("apply 2"),
                String::from("pass 2"),
            ]
        )
    );
}

/// A failed pass or a failed application blocks,
///  at the start and after a change.
#[test]
fn failures_block() {
    assert_eq!(
        run(&[PassResult::Failed], &["a"], None).0,
        Convergence::Blocked
    );
    assert_eq!(
        run(
            &[PassResult::Proposed, PassResult::Failed],
            &["a", "b"],
            None
        )
        .0,
        Convergence::Blocked
    );
    let (first, first_log) = run(&[PassResult::Proposed], &["a", "b"], Some(1));
    assert_eq!(first, Convergence::Blocked);
    assert_eq!(first_log, [String::from("pass 0"), String::from("apply 1")]);
    assert_eq!(
        run(
            &[PassResult::Proposed, PassResult::Proposed],
            &["a", "b", "c"],
            Some(2)
        )
        .0,
        Convergence::Blocked
    );
}

/// Corrections that leave the state exactly as it was settle without counting as a change.
#[test]
fn unchanged_state_after_corrections_is_stable() {
    assert_eq!(
        run(&[PassResult::Proposed], &["a", "a"], None),
        (
            Convergence::Settled { changed_passes: 0 },
            vec![String::from("pass 0"), String::from("apply 1")]
        )
    );
    assert_eq!(
        run(
            &[PassResult::Proposed, PassResult::Proposed],
            &["a", "b", "b"],
            None
        )
        .0,
        Convergence::Settled { changed_passes: 1 }
    );
}

/// Returning to any state older than the preceding one is a cycle.
#[test]
fn returning_to_an_older_state_is_a_cycle() {
    for states in [
        vec!["a", "b", "a"],
        vec!["a", "b", "c", "a"],
        vec!["a", "b", "c", "b"],
        vec!["a", "b", "c", "d", "b"],
    ] {
        let passes: Vec<PassResult> = vec![PassResult::Proposed; states.len() - 1];
        let (ending, log) = run(passes.as_slice(), states.as_slice(), None);
        assert_eq!(ending, Convergence::FixCycle, "{states:?}");
        // The loop ends on the application that reproduced the state, without another pass.
        assert_eq!(
            log.last(),
            Some(&format!("apply {}", states.len() - 1)),
            "{states:?}"
        );
    }
}

/// Eight changes are allowed;
///  the ninth pass may only confirm stability.
#[test]
fn eight_changes_are_the_limit() {
    assert_eq!(MAXIMUM_CHANGED_PASSES, 8);
    let (settled, settled_log) = run(
        proposing(8, PassResult::Stable).as_slice(),
        &DISTINCT[..9],
        None,
    );
    assert_eq!(settled, Convergence::Settled { changed_passes: 8 });
    assert_eq!(settled_log.len(), 17);
    assert_eq!(settled_log[15], "apply 8");
    assert_eq!(settled_log[16], "pass 8");
    let (limited, limited_log) = run(
        proposing(8, PassResult::Proposed).as_slice(),
        &DISTINCT,
        None,
    );
    assert_eq!(limited, Convergence::FixPassLimit);
    // The ninth proposal is refused before anything is applied or stored.
    assert_eq!(limited_log, settled_log);
    assert_eq!(
        run(
            proposing(8, PassResult::Failed).as_slice(),
            &DISTINCT[..9],
            None
        )
        .0,
        Convergence::Blocked
    );
    // Seven changes and a stable eighth pass are well inside the limit.
    assert_eq!(
        run(
            proposing(7, PassResult::Stable).as_slice(),
            &DISTINCT[..8],
            None
        )
        .0,
        Convergence::Settled { changed_passes: 7 }
    );
}

/// The failure messages are the incumbent's.
#[test]
fn failure_messages_are_the_incumbent_text() {
    assert_eq!(
        FIX_PASS_LIMIT_MESSAGE,
        "Policy patches did not converge within eight changed passes."
    );
    assert_eq!(
        FIX_CYCLE_MESSAGE,
        "Policy patches repeated an exact prior candidate state."
    );
}
