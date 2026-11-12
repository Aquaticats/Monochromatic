//! What:
//!  The one budget a command gives the commit service to answer,
//!  counted across every step of its exchange with it.
//! Why:
//!  The steps of an exchange are walked one after another:
//! the greeting of a connection,
//! the write of the request,
//!  the answer,
//! a connection again after one broke,
//! a withdrawal.
//! Bounding each step on its own says nothing about the whole:
//! five steps of five seconds is twenty-five,
//!  and the owner's bound is five seconds for a whole commit
//! (`package/git-policy/cli/mise.toml`,
//  `native:perf`).
//! A complete message from the service moves the end of the wait forward,
//! so a service that keeps saying what it does is waited for as long as it works,
//! and a service that says nothing complete for the whole budget is one that stopped.
//! A line that never ends is therefore not an answer:
//! the budget of the read is what the last complete message started,
//!  and bytes that never become a message do not move it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const patience = Patience.of(2_000); patience.heard(); patience.bound(5_000); // 2_000 or less
//! ```

use std::time::{Duration, Instant};

/// The shortest wait a timeout of the operating system may name.
/// Why:
///  A read or write timeout of no time at all is refused by the system,
///  so a budget that is spent asks for the shortest wait there is instead of zero.
const SHORTEST_WAIT: Duration = Duration::from_millis(1);

/// What:
///  The moment a command stops waiting for a service that has said nothing complete,
///  and how long the service is given to say something complete.
/// Why:
///  Every read and every write of an exchange asks this for what is left,
///  so the sum of the steps is the budget and not their count times their own bounds;
///  and a budget spent is read by the line reader between two chunks,
///  so a service that dribbles bytes and never ends a line cannot outlast it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Patience = { each: number; until: number }
/// ```
#[derive(Debug)]
pub(crate) struct Patience {
    /// How long the service may say nothing complete.
    each: Duration,
    /// The end of the wait now,
    /// moved forward by every complete message.
    until: Instant,
}

impl Patience {
    /// What:
    ///  The budget of one command:
    ///  the service may say nothing complete for `each`,
    ///  counted from now.
    /// Why:
    ///  The budget starts when the command first asks the service,
    ///  so a start that has not greeted yet spends it,
    ///  and nothing before that (a preparation, a policy) does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const patience = Patience.of(2_000)
    /// ```
    pub(crate) fn of(each: Duration) -> Self {
        return Self { each, until: Instant::now() + each };
    }

    /// What:
    ///  Records that the service sent a complete message,
    ///  so the wait for the next one starts now.
    /// Why:
    ///  The budget is a budget of silence:
    ///  a landing that keeps reporting its place and its work is not a service that stopped,
    ///  however long it takes.
    ///  A part of a line is not a message, so it moves nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// patience.until = now() + patience.each
    /// ```
    pub(crate) fn heard(&mut self) {
        self.until = Instant::now() + self.each;
    }

    /// What:
    ///  How much of the budget is left,
    ///  nothing once it is spent.
    /// Why:
    ///  Every step of the exchange asks this to shorten its own bound to what the command has left,
    ///  and a step whose wait is already over waits for the shortest time the system accepts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// patience.left()
    /// ```
    pub(crate) fn left(&self) -> Duration {
        return self.until.saturating_duration_since(Instant::now());
    }

    /// What:
    ///  Whether the budget is spent,
    ///  which is a service that said nothing complete for all of it.
    /// Why:
    ///  A spent budget ends the exchange:
    ///  the next read waits for the shortest time there is and reports silence,
    ///  and no step starts a wait of its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// patience.left().milliseconds === 0
    /// ```
    pub(crate) fn spent(&self) -> bool {
        return self.left().is_zero();
    }

    /// What:
    ///  The longest one read or write may wait now,
    ///  which is what is left of the budget or the shortest wait the system accepts.
    /// Why:
    ///  A step that sets a timeout of the operating system asks this,
    ///  so no wait outlasts the budget and none asks for zero.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// patience.wait()
    /// ```
    pub(crate) fn wait(&self) -> Duration {
        return self.left().max(SHORTEST_WAIT);
    }

    /// What:
    ///  The wait a step may take:
    ///  its own bound,
    ///  or what the command has left when that is shorter.
    /// Why:
    ///  The whole exchange is inside one budget,
    ///  so no step keeps its own bound past the end of it,
    ///  and a step whose budget is spent asks for the shortest wait the system accepts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// patience.bound(5_000) // min(5_000, patience.left()), at least 1ms
    /// ```
    pub(crate) fn bound(&self, own: Duration) -> Duration {
        return self.left().min(own).max(SHORTEST_WAIT);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_client_patience_tests.rs"]
mod tests;
