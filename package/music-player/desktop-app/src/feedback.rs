//! Transient feedback queue,
//!  ported from `FeedbackQueue.kt`.
//!
//! `FeedbackQueue` is an immutable set of visible messages.
//!  Every transition
//! returns a new queue and leaves the old one unchanged.
//!  The caller supplies the
//! clock,
//!  so the queue never reads time and expiry stays deterministic.
//!
//! Kotlin throws `IllegalArgumentException` for invalid input.
//!  Here the same
//! checks return a `FeedbackError` whose `Display` keeps the Kotlin message text.

/// What:
///      `pub const FEEDBACK_ERROR_DURATION_MS` is the default visible time of an error,
///  in milliseconds.
/// Why:
///      The accepted study used the Short snackbar duration,
///  which is 4000 ms before any platform adjustment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FEEDBACK_ERROR_DURATION_MS = 4000;
/// ```
pub const FEEDBACK_ERROR_DURATION_MS: i64 = 4_000;

/// What:
///      `pub const FEEDBACK_UNDO_DURATION_MS` is the default visible time of Undo,
///  in milliseconds.
/// Why:
///      The study showed the Undo message with the same Short duration as the error message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FEEDBACK_UNDO_DURATION_MS = 4000;
/// ```
pub const FEEDBACK_UNDO_DURATION_MS: i64 = 4_000;

/// What:
///      `pub const FEEDBACK_MAX_DURATION_MS` bounds any caller-supplied visible time,
///  in milliseconds.
/// Why:
///      Feedback is transient,
///  and the bound keeps the expiry sum far from integer overflow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FEEDBACK_MAX_DURATION_MS = 60_000;
/// ```
pub const FEEDBACK_MAX_DURATION_MS: i64 = 60_000;

/// What:
///      `pub enum FeedbackKind` declares the two closed kinds of transient feedback.
/// Why:
///      Replacement,
///  ordering,
///  and default duration depend only on which kind a message is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FeedbackKind = "ERROR" | "UNDO";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackKind {
    /// A failed operation whose text states the outcome.
    ///  Never offers Undo.
    Error,
    /// A successful reversible operation that offers an Undo action.
    Undo,
}

/// Methods of `FeedbackKind`.
impl FeedbackKind {
    /// What:
    ///      `default_duration_ms` returns the visible time used when a show call names none.
    /// Why:
    ///      Each kind keeps its own accepted duration beside the kind itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// defaultDurationMs: number;
    /// ```
    pub fn default_duration_ms(self) -> i64 {
        return match self {
            Self::Error => FEEDBACK_ERROR_DURATION_MS,
            Self::Undo => FEEDBACK_UNDO_DURATION_MS,
        };
    }
}

/// What:
///      `pub enum FeedbackError` is the failure of the message constructor and of `show_for`.
/// Why:
///      Invalid text is rejected at construction,
///  and an out-of-range duration is rejected at show time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FeedbackError extends Error { /* the message text names the failed check */ }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackError {
    /// The message id is empty or whitespace only.
    BlankId,
    /// The message text is empty or whitespace only.
    BlankText,
    /// The action label is present but empty or whitespace only.
    BlankActionLabel,
    /// The visible time is outside 1 through `FEEDBACK_MAX_DURATION_MS`.
    DurationOutOfRange {
        /// The rejected visible time in milliseconds.
        duration_ms: i64,
    },
}

/// Display text for `FeedbackError`.
impl std::fmt::Display for FeedbackError {
    /// What:
    ///      `fmt` writes the message for each rejected input,
    ///  using the Kotlin message text.
    /// Why:
    ///      Callers and the tests compare these exact strings.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string;
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::BlankId => formatter.write_str("Feedback message id must not be blank."),
            Self::BlankText => formatter.write_str("Feedback message text must not be blank."),
            Self::BlankActionLabel => {
                formatter.write_str("Feedback action label must not be blank when present.")
            }
            Self::DurationOutOfRange { duration_ms } => write!(
                formatter,
                "Feedback duration must be between 1 and {FEEDBACK_MAX_DURATION_MS} ms, was {duration_ms}."
            ),
        };
    }
}

/// Marks `FeedbackError` as a standard error.
impl std::error::Error for FeedbackError {}

/// What:
///      `pub struct FeedbackMessage` holds one message's stable id,
///  text,
///  kind,
///  and optional action label.
/// Why:
///      The fields are private so that every message passes the checks in `new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FeedbackMessage = { id: string; text: string; kind: FeedbackKind; actionLabel: string | null };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackMessage {
    /// Stable key that identifies the message.
    id: String,
    /// The text the message shows.
    text: String,
    /// Kind that sets the message ranking and styling.
    kind: FeedbackKind,
    /// Label of the optional action,
    ///  or `None` when the message has no action.
    action_label: Option<String>,
}

/// Methods of `FeedbackMessage`.
impl FeedbackMessage {
    /// What:
    ///      `new` builds a message after checking that id,
    ///  text,
    ///  and any action label are not blank.
    /// Why:
    ///      A blank id cannot be dismissed reliably,
    ///  and a blank text would show an empty message.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (id.trim() === "") throw new Error("Feedback message id must not be blank.");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `FeedbackError::BlankId`,
    ///  `BlankText`,
    ///  or `BlankActionLabel` for the first blank field.
    pub fn new(
        id: &str,
        text: &str,
        kind: FeedbackKind,
        action_label: Option<&str>,
    ) -> Result<Self, FeedbackError> {
        if id.trim().is_empty() {
            return Err(FeedbackError::BlankId);
        }
        if text.trim().is_empty() {
            return Err(FeedbackError::BlankText);
        }
        if action_label.is_some_and(|label| return label.trim().is_empty()) {
            return Err(FeedbackError::BlankActionLabel);
        }
        return Ok(Self {
            id: id.to_owned(),
            text: text.to_owned(),
            kind,
            action_label: action_label.map(str::to_owned),
        });
    }

    /// What:
    ///      `id` returns the caller's stable key for this message.
    /// Why:
    ///      Dismissal and action callbacks name the message by this key,
    ///  never by position.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// id: string;
    /// ```
    pub fn id(&self) -> &str {
        return &self.id;
    }

    /// What:
    ///      `text` returns the literal message shown to the user.
    /// Why:
    ///      The text is never shortened by the model,
    ///  so the overlay decides how much to draw.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// text: string;
    /// ```
    pub fn text(&self) -> &str {
        return &self.text;
    }

    /// What:
    ///      `kind` returns the kind that selects replacement,
    ///  display order,
    ///  and default visible time.
    /// Why:
    ///      The kind is the only property that decides which messages can be visible together.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind: FeedbackKind;
    /// ```
    pub fn kind(&self) -> FeedbackKind {
        return self.kind;
    }

    /// What:
    ///      `action_label` returns the action button text,
    ///  or `None` when the message offers no action.
    /// Why:
    ///      Undo carries its action here,
    ///  and any kind may carry one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// actionLabel: string | null;
    /// ```
    pub fn action_label(&self) -> Option<&str> {
        return self.action_label.as_deref();
    }
}

/// What:
///      `FeedbackEntry` pairs one message with the clock value and duration it was shown with.
/// Why:
///      Expiry needs the show time and the duration,
///  which the public message does not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FeedbackEntry = { message: FeedbackMessage; shownAtMs: number; durationMs: number };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
struct FeedbackEntry {
    /// The message this entry holds.
    message: FeedbackMessage,
    /// Time the entry was first shown,
    ///  in milliseconds.
    shown_at_ms: i64,
    /// How long the entry stays visible,
    ///  in milliseconds.
    duration_ms: i64,
}

/// Methods of `FeedbackEntry`.
impl FeedbackEntry {
    /// What:
    ///      `is_expired_at` reports whether the full visible time has elapsed at `now_ms`.
    /// Why:
    ///      Expiry is the single boundary rule the queue applies,
    ///  so tests can pin it exactly.
    ///      A clock value before the show time gives a negative elapsed time,
    ///  which never expires.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isExpiredAt(nowMs: number): boolean { return nowMs - this.shownAtMs >= this.durationMs; }
    /// ```
    fn is_expired_at(&self, now_ms: i64) -> bool {
        let elapsed_ms = now_ms.saturating_sub(self.shown_at_ms);
        return elapsed_ms >= self.duration_ms;
    }

    /// What:
    ///      `expires_at_ms` returns the clock value at which this entry's visible time is fully elapsed.
    /// Why:
    ///      The wiring step waits for the earliest of these instead of polling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const expiresAtMs = this.shownAtMs + this.durationMs;
    /// ```
    fn expires_at_ms(&self) -> i64 {
        return self.shown_at_ms.saturating_add(self.duration_ms);
    }
}

/// What:
///      `pub struct FeedbackQueue` is an immutable set of visible transient messages.
/// Why:
///      The player keeps one value and replaces it on each event,
///  so every state is reproducible.
///      At most one message per kind and at most one message per id is kept at any time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FeedbackQueue { private constructor(private readonly entries: FeedbackEntry[]) {} }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackQueue {
    /// Queued entries,
    ///  in the order they were added.
    entries: Vec<FeedbackEntry>,
}

/// Methods of `FeedbackQueue`.
impl FeedbackQueue {
    /// What:
    ///      `EMPTY` is a queue with no visible messages.
    /// Why:
    ///      Starting every screen from the same value keeps the initial state identical.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static readonly EMPTY: FeedbackQueue = new FeedbackQueue([]);
    /// ```
    pub const EMPTY: FeedbackQueue = FeedbackQueue { entries: Vec::new() };

    /// What:
    ///      `show` returns a queue in which `message` is visible from `now_ms` for its kind's default time.
    /// Why:
    ///      A new message replaces the visible one of the same kind,
    ///  and a repeated id restarts its timer.
    ///      The default time is always in range,
    ///  so this method cannot fail.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// show(message, nowMs, durationMs = defaults[message.kind]): FeedbackQueue
    /// ```
    pub fn show(&self, message: FeedbackMessage, now_ms: i64) -> FeedbackQueue {
        let duration_ms = message.kind().default_duration_ms();
        return self.show_entry(message, now_ms, duration_ms);
    }

    /// What:
    ///      `show_for` is `show` with a caller-chosen visible time.
    /// Why:
    ///      Each entry keeps its own timer,
    ///  so a short-lived message can expire before the default one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// showFor(message, nowMs, durationMs): FeedbackQueue  // throws outside 1 to FEEDBACK_MAX_DURATION_MS
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `FeedbackError::DurationOutOfRange` when `duration_ms` is outside 1 through `FEEDBACK_MAX_DURATION_MS`.
    pub fn show_for(
        &self,
        message: FeedbackMessage,
        now_ms: i64,
        duration_ms: i64,
    ) -> Result<FeedbackQueue, FeedbackError> {
        if !(1..=FEEDBACK_MAX_DURATION_MS).contains(&duration_ms) {
            return Err(FeedbackError::DurationOutOfRange { duration_ms });
        }
        return Ok(self.show_entry(message, now_ms, duration_ms));
    }

    /// What:
    ///      `show_entry` appends `message`,
    ///  after dropping every entry it supersedes.
    /// Why:
    ///      Same-kind replacement and id reuse both remove the older entry,
    ///  so the invariant holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = entries.filter(e => e.message.id !== message.id && e.message.kind !== message.kind);
    /// ```
    fn show_entry(&self, message: FeedbackMessage, now_ms: i64, duration_ms: i64) -> FeedbackQueue {
        let mut entries: Vec<FeedbackEntry> = self
            .entries
            .iter()
            .filter(|entry| return entry.message.id() != message.id() && entry.message.kind() != message.kind())
            .cloned()
            .collect();
        entries.push(FeedbackEntry {
            message,
            shown_at_ms: now_ms,
            duration_ms,
        });
        return FeedbackQueue { entries };
    }

    /// What:
    ///      `dismiss` returns a queue without the message with `id`.
    /// Why:
    ///      Dismissal by id cannot remove a newer message that replaced the one the user saw,
    ///      and an unknown id leaves the visible messages unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dismiss(id: string): FeedbackQueue { return new FeedbackQueue(this.entries.filter(e => e.message.id !== id)); }
    /// ```
    pub fn dismiss(&self, id: &str) -> FeedbackQueue {
        return FeedbackQueue {
            entries: self
                .entries
                .iter()
                .filter(|entry| return entry.message.id() != id)
                .cloned()
                .collect(),
        };
    }

    /// What:
    ///      `expire` returns a queue without the entries whose visible time has elapsed at `now_ms`.
    /// Why:
    ///      The caller supplies the clock,
    ///  so the queue never reads time and expiry stays deterministic.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// expire(nowMs: number): FeedbackQueue {
    ///   return new FeedbackQueue(this.entries.filter(e => !e.isExpiredAt(nowMs)));
    /// }
    /// ```
    pub fn expire(&self, now_ms: i64) -> FeedbackQueue {
        return FeedbackQueue {
            entries: self
                .entries
                .iter()
                .filter(|entry| return !entry.is_expired_at(now_ms))
                .cloned()
                .collect(),
        };
    }

    /// What:
    ///      `visible` returns the messages in top-to-bottom display order:
    ///  Undo above error.
    /// Why:
    ///      The overlay draws the list as given,
    ///  so the display order is decided once here,
    ///      independent of the order the messages were shown in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visible(): FeedbackMessage[] { return sortBy(this.entries, kindRank).map(e => e.message); }
    /// ```
    pub fn visible(&self) -> Vec<FeedbackMessage> {
        let mut ordered: Vec<&FeedbackEntry> = self.entries.iter().collect();
        ordered.sort_by_key(|entry| return display_rank(entry.message.kind()));
        return ordered
            .into_iter()
            .map(|entry| return entry.message.clone())
            .collect();
    }

    /// What:
    ///      `next_expiry_ms` returns the earliest clock value at which a visible entry expires.
    /// Why:
    ///      A single delay wakes the caller in time for every message,
    ///  and `None` means no timer is needed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextExpiryMs(): number | null { ... }
    /// ```
    pub fn next_expiry_ms(&self) -> Option<i64> {
        return self
            .entries
            .iter()
            .map(FeedbackEntry::expires_at_ms)
            .min();
    }
}

/// What:
///      `display_rank` gives the top-to-bottom position of a kind,
///  where a lower rank is drawn higher.
/// Why:
///      Decision D29 places the Undo message above the error message,
///  whatever order they were shown in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FEEDBACK_DISPLAY_ORDER: FeedbackKind[] = ["UNDO", "ERROR"];
/// ```
fn display_rank(kind: FeedbackKind) -> u8 {
    return match kind {
        FeedbackKind::Undo => 0,
        FeedbackKind::Error => 1,
    };
}

/// Unit tests in `feedback_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "feedback_tests.rs"]
mod tests;
