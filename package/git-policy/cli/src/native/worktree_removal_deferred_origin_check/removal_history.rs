//! What:
//!  The loss a removal's own record already holds,
//!  and the one sentence every refusal of this folder prints from it.
//! Why:
//!  A refusal at the check after the snapshot batch finds part of the clone's history already
//!  gone,
//!  and no later refusal brings it back,
//!  so the message names it
//!  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
//!  "What a refusal at the second check leaves";
//!  `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals").
//!  The sentence comes from what the plan records rather than from which call site refused,
//!  so it is true on a resumed removal and at a later chunk's check,
//!  and no sentence claims that this run did or did not delete something.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const history = RemovalHistory.ofDestroyed(plan.destroyed);
//! message(refusal, history); // 'This clone has already lost `a` and `b`, ...'
//! ```

//region The history a refusal prints

/// What:
///  The snapshots this clone has already lost,
///  as the removal's plan records them.
/// Why:
///  A plan that records no destroyed tuple says nothing was destroyed,
///  and a plan that records destroyed tuples names every one:
///  one sentence serves both check points,
///  and the look before the unmount always holds the empty one,
///  because it runs before the removal has recorded anything it destroyed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalHistory = { kind: 'nothing-destroyed' } | { kind: 'lost'; snapshots: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemovalHistory {
    /// The plan records no destroyed snapshot.
    NothingDestroyed,
    /// The plan records these destroyed snapshots, in its own order.
    Lost {
        /// The full names of the snapshots the plan records as destroyed.
        snapshots: Vec<String>,
    },
}

/// Each name in backticks,
/// separated by commas and joined by `and` before the last one.
fn quote_and_join(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| return format!("`{name}`")).collect();
    let Some((last, rest)) = quoted.split_last() else {
        return String::new();
    };
    if rest.is_empty() {
        return last.clone();
    }
    return format!("{} and {last}", rest.join(", "));
}

impl RemovalHistory {
    /// What:
    ///  The history a set of recorded destroyed snapshots gives.
    /// Why:
    ///  Nothing destroyed is the empty plan,
    ///  so a caller hands the plan's own list and this type decides which sentence is true.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const history = RemovalHistory.ofDestroyed(['fix-flaky-test@manual']);
    /// ```
    pub fn of_destroyed(snapshots: Vec<String>) -> RemovalHistory {
        if snapshots.is_empty() {
            return RemovalHistory::NothingDestroyed;
        }
        return RemovalHistory::Lost { snapshots };
    }

    /// The snapshot names the history names,
    /// and none for a history that destroyed nothing.
    pub fn lost_snapshots(&self) -> &[String] {
        return match self {
            RemovalHistory::NothingDestroyed => &[],
            RemovalHistory::Lost { snapshots } => snapshots.as_slice(),
        };
    }

    /// What:
    ///  The sentence a refusal prints for this history.
    /// Why:
    ///  It says "in this removal or an earlier one" rather than "this run",
    ///  because a resumed removal and a retry both present a plan an earlier run wrote.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// history.sentence(); // 'This clone has already lost `a`, in this removal or an earlier one, and it does not come back.'
    /// ```
    pub fn sentence(&self) -> String {
        let snapshots: &[String] = self.lost_snapshots();
        if snapshots.is_empty() {
            return String::from("This removal has not destroyed anything yet.");
        }
        let named: String = quote_and_join(snapshots);
        let comeback: &str = if snapshots.len() == 1 { "it does not come back" } else { "they do not come back" };
        return format!("This clone has already lost {named}, in this removal or an earlier one, and {comeback}.");
    }
}

//endregion


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_history_tests.rs"]
mod tests;
