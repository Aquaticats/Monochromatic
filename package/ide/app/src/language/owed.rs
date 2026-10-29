//! Answers a server still owes:
//!  requests the worker made on its own (inlay hints,
//!  pull
//! diagnostics) that stayed unanswered through their retries,
//!  and the bounded rule for asking
//! again once the server sends anything.

/// What:
///  An upper limit,
///  as an unsigned 32-bit count (`u32`;
///  siblings:
///  `u64`,
///  `usize`).
/// Why:
///  One server is asked again at most this many times for one displayed text because it
///      sent a message while it owed an answer.
///  Each such ask has its own bounded retries,
///  so
///      the total number of requests per text and server has a fixed ceiling,
///  which `u32` holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_CATCH_UPS = 3;
/// ```
pub(super) const MAX_CATCH_UPS: u32 = 3;

/// What:
///  Which of the worker's own requests one server left unanswered for the displayed text,
///       and how often it was already asked again.
///  `bool` is a plain flag;
///  `u32` is an unsigned
///       32-bit count.
///  `Copy` lets a value be passed like a number;
///  `Default` builds the value
///       with every flag off and the count at zero.
/// Why:
///  A request that timed out,
///  or was superseded through all its retries,
///  must not be asked
///      in a loop,
///  and must not be forgotten either.
///  It is recorded here and asked again when
///      the server next shows that it processes messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Owed = { hints: boolean; pull: boolean; askedAgain: number };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Owed {
    /// Inlay hints for the reported window stayed unanswered.
    pub(super) hints: bool,
    /// Pull diagnostics for the document stayed unanswered.
    pub(super) pull: bool,
    /// Catch-up asks already made for the displayed text.
    asked_again: u32,
}

/// The catch-up rule.
impl Owed {
    /// What:
    ///  The server sent something:
    ///  return what to ask again now,
    ///  or nothing.
    ///  `&mut self`
    ///       lets the count rise;
    ///  `Option<Self>` is "a copy of the flags,
    ///  or nothing".
    /// Why:
    ///  Nothing owed needs no request,
    ///  and a server that used up its catch-up asks for this
    ///      text gets no more until the text changes,
    ///  so a server that never answers cannot
    ///      keep the worker asking.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// catchUp(): Owed | undefined {
    ///   if (!this.hints && !this.pull) return undefined;
    ///   if (this.askedAgain >= MAX_CATCH_UPS) return undefined;
    ///   this.askedAgain += 1;
    ///   return { ...this };
    /// }
    /// ```
    pub(super) fn catch_up(&mut self) -> Option<Self> {
        if !self.hints && !self.pull {
            // `None` is the "nothing" variant of `Option`.
            return None;
        }
        if self.asked_again >= MAX_CATCH_UPS {
            return None;
        }
        self.asked_again += 1;
        // `Some(...)` is the "value present" variant; `*self` copies the record behind the borrow.
        return Some(*self);
    }
}

/// The catch-up rule is exercised without a server.
#[cfg(test)]
#[path = "owed_tests.rs"]
mod tests;
