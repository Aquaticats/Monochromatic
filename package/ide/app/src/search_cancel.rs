//! Cloneable search cancellation reaches both child processes,
//!  including while neither pipe produces output.

/// Tokio watch channels provide a retained cancellation value plus an awaitable change notification.
use tokio::sync::watch;

/// One-way cancellation signal;
///  a newer search gets a new independent signal rather than resetting this one.
#[derive(Clone, Debug)]
pub struct SearchCancellation {
    /// Retained state lets cancellation before process startup work without a timing race.
    sender: watch::Sender<bool>,
}

/// Search callers can cancel without owning or killing unrelated process IDs.
impl SearchCancellation {
    /// Construct an initially active search signal.
    pub fn new() -> Self {
        // What: watch retains the latest bool; unlike a queue, repeated cancellation cannot accumulate messages.
        // Why: The UI can signal both readers without waiting for them to reach a stdout record boundary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new AbortController();
        // ```
        let (sender, _receiver) = watch::channel(false);
        return Self { sender };
    }

    /// Cancellation is idempotent and survives the absence of a subscribed worker.
    pub fn cancel(&self) {
        self.sender.send_replace(true);
        tracing::debug!("cancelled project search");
    }

    /// Read retained cancellation without waiting or changing channel state.
    pub fn is_cancelled(&self) -> bool {
        // Dereference the temporary watch borrow into an owned bool, not a guard retained across await.
        return *self.sender.borrow();
    }

    /// Each child subscribes independently to the same one-way signal.
    pub(crate) fn subscribe(&self) -> watch::Receiver<bool> {
        return self.sender.subscribe();
    }
}

/// Default construction has the same active state as new.
impl Default for SearchCancellation {
    /// Construct an active signal using the same initialization as new.
    fn default() -> Self {
        return Self::new();
    }
}
