//! What:
//!  A withdrawal answered on the connection thread while the landing thread is stuck inside a write that is never ended:
//! the token that keeps two withdrawals of one request apart,
//! the decision the answer wins or loses,
//! and the ceiling that keeps the record from growing without a bound.
//! Why:
//!  The landing thread answers withdrawals between two landings (`service_busy.rs`),
//! so behind a stuck write none would be answered,
//! and a client queued behind it would
//!  reach its own answer ceiling and end with an error that names nothing.
//! This part answers a request that has not begun,
//! and it has its own file so
//!  the decisions the landing thread makes stay in one place.
//! A request that has not begun cannot land while the thread is stuck,
//! and the decision
//!  keeps it from beginning later.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = board.withdrawEarly(id); // 'queued', 'withdrawn' with words, or 'landed'
//! ```

#[cfg(test)]
use super::service_busy::AFTER_THE_DECISION_IS_WON;
use super::service_busy::{
    LandingBoard,
    WITHDRAWALS_RETAINED,
    now_millis,
};
use crate::service_busy_answers_and_name_checks::{
    Decision,
    EarlyAnswer,
    Naming,
    WITHDRAWN,
    is_confirmation,
    is_plain,
    withdrawal_token,
};
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// The most confirmed withdrawals the record holds at any moment,
/// twice what it retains:
/// only the landing thread forgets (between two
///  groups,
/// so there is one pruner),
/// and while it is stuck in a write that is never ended nothing is forgotten,
/// so past this number
///  the connection threads leave further withdrawals to the landing thread instead of growing the record without a bound.
/// Two connection
///  threads that count together can pass it by the number of connection threads at that moment.
pub(crate) const WITHDRAWALS_CEILING: usize = 2 * WITHDRAWALS_RETAINED;

/// Distinguishes the tokens of withdrawals.
static TEMPORARY_NUMBER: AtomicU64 = AtomicU64::new(0);

/// A token no other withdrawal in any service has,
/// made of letters,
/// digits and hyphens so it can stand in a file name.
fn new_token() -> String {
    return format!(
        "{}-{}-{}",
        std::process::id(),
        TEMPORARY_NUMBER.fetch_add(1, Ordering::Relaxed),
        now_millis()
    );
}

impl LandingBoard {
    /// What:
    ///  Answer a withdrawal on the connection thread when the landing thread is stuck in a write that is never ended and the
    ///  request is not the landing thread's.
    /// Why:
    ///   The landing thread answers withdrawals between two landings,
    /// so behind a stuck write none would be answered.
    /// A request
    ///  that has not begun cannot land while the thread is stuck,
    /// and the decision keeps it from beginning later.
    /// The journal is
    ///  asked before the decision is tried and again after it is won,
    /// because a request that landed and was let go of between the
    ///  two looks like a request nobody took.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stuck && journal(id) === 'no' && won(decide(id, 'withdrawn')) && journal(id) === 'no' ? 'withdrawn' : (landed ?? 'queued')
    /// ```
    pub fn withdraw_early(
        &self,
        id: &str,
    ) -> EarlyAnswer {
        if !is_plain(id) {
            return EarlyAnswer::Queued;
        }
        let Some(words) = self.describe() else {
            return EarlyAnswer::Queued;
        };
        match self.naming(id) {
            Naming::Landed(commit) => return EarlyAnswer::Landed(commit),
            Naming::Open | Naming::Unknown => return EarlyAnswer::Queued,
            Naming::No => {}
        }
        // The landing thread forgets old withdrawals between two groups, and a stuck write is one group: the record is bounded here.
        if self.confirmed_withdrawals() >= WITHDRAWALS_CEILING {
            return EarlyAnswer::Queued;
        }
        let token: String = new_token();
        match self.decide(id, format!("{WITHDRAWN}:{token}").as_str()) {
            Err(_) => return EarlyAnswer::Queued,
            // Another withdrawer won. Its decision is acknowledged only when it has confirmed it, with the confirmation that carries
            // the token of that decision; before that it is provisional, and may still be taken back (the journal may name the
            // request when the winner asks again). A confirmation of an earlier decision of this request has another token.
            Ok(Decision::Lost(content)) => {
                let confirmed: bool = match withdrawal_token(content.as_str()) {
                    Some(theirs) => self.cancelled_path(id, theirs).exists(),
                    None => false,
                };
                return if confirmed {
                    EarlyAnswer::Withdrawn(words)
                } else {
                    EarlyAnswer::Queued
                };
            }
            Ok(Decision::Won) => {}
        }
        #[cfg(test)]
        AFTER_THE_DECISION_IS_WON.with(|hook| {
            if let Some(pause) = hook.borrow().as_ref() {
                pause();
            }
        });
        // The decision is ours, so the request cannot be admitted any more; what could have happened is that it was admitted,
        // landed and let go of before the decision was tried.
        match self.naming(id) {
            Naming::No => {
                // Confirmed: from now on any withdrawer may acknowledge it, and nothing removes it but the retention.
                if std::fs::File::create_new(self.cancelled_path(id, token.as_str())).is_err() {
                    // Not confirmed, and nobody may have acknowledged a decision that was not: it is taken back.
                    let _undone = std::fs::remove_file(self.decision_path(id));
                    return EarlyAnswer::Queued;
                }
                return EarlyAnswer::Withdrawn(words);
            }
            // Never confirmed, so never acknowledged by anyone else: taken back.
            Naming::Landed(commit) => {
                let _undone = std::fs::remove_file(self.decision_path(id));
                return EarlyAnswer::Landed(commit);
            }
            Naming::Open | Naming::Unknown => {
                let _undone = std::fs::remove_file(self.decision_path(id));
                return EarlyAnswer::Queued;
            }
        }
    }

    /// How many confirmed withdrawals the record holds;
    /// the ceiling is reached when the directory cannot be listed,
    /// so that a record that
    ///  cannot be counted is not grown.
    fn confirmed_withdrawals(&self) -> usize {
        let Ok(entries) = std::fs::read_dir(self.directory.as_path()) else {
            return usize::MAX;
        };
        return entries.flatten().filter(is_confirmation).count();
    }
}
