//! What:
//!  The words a waiting client reads about the write the landing thread is inside:
//! how long the write has run,
//! what it is doing,
//! and the notice a request of the group being landed gets while it waits.
//! Why:
//!  What the board keeps is one file of the journal (`service_busy.rs`);
//! this part turns that file into sentences a person reads,
//! and it has its own file so the decisions the board makes stay in one place.
//!  The words name no moment of the write,
//! because the client says them once the person's wait has mattered,
//! so a moment taken here would be false by the time it is read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! board.describe(); // the notice about what the service is stuck in
//! board.insideNotice(id); // { key, text } for a client whose request is in the group being landed
//! ```

use super::service_busy::{
    LandingBoard,
    STEP_FILE,
    now_millis,
    record_text,
};
use crate::service_messages::RESTART_COMMAND;

impl LandingBoard {
    /// The write the landing thread is inside,
    /// as `(millis when it began, command, detail, millis it has run)`,
    /// when there is one.
    fn running(&self) -> Option<(u128, String, Option<String>, u128)> {
        let text: String = record_text(self.directory.join(STEP_FILE).as_path())?;
        let mut lines = text.lines();
        let began: u128 = lines.next()?.parse().ok()?;
        let command: String = String::from(lines.next()?);
        let detail: Option<String> = match lines.next() {
            Some(line) if !line.is_empty() => Some(String::from(line)),
            _ => None,
        };
        return Some((began, command, detail, now_millis().saturating_sub(began)));
    }

    /// The words that tell a client what the service is stuck in,
    /// when the landing thread has been inside a write that is never
    ///  ended for longer than the board's `slow_after`;
    /// nothing otherwise.
    pub fn describe(&self) -> Option<String> {
        let (_, command, found, running) = self.running()?;
        if running <= self.slow_after.as_millis() {
            return None;
        }
        let minutes: u128 = running / 60_000;
        let detail: String = match found {
            Some(text) => format!(" ({text})"),
            None => String::new(),
        };
        return Some(format!(
            "monochromatic-git-policy-cli: the commit service has been inside a write for {minutes} minutes and has not heard back: `{command}`{detail}. The service never ends a write in \
             the middle, because that could lose data, so commits queue behind it. If it never returns, look at what that process \
             waits for with `ps` and at the disk, and when no program uses the repository, restart the service with `{RESTART_COMMAND}`.\n"
        ));
    }

    /// What:
    ///  The notice for the client whose own request is in the group being landed while the landing thread is inside a write that is
    ///  never ended:
    /// a key that changes when the notice should be sent again,
    /// and the words.
    /// Why:
    ///   The client must learn that its commit may be on the branch when the write starts,
    /// not a minute later (owner,
    /// 2026-11-04,
    ///  "Short extra wait");
    /// the words that follow a minute's stuck write are the longer ones.
    /// The words name no moment of the write:
    /// the client says them once the person's wait
    ///  has mattered (`waiting_notice_after`),
    /// so a moment taken here would be false by the time it is read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// inFlight(id) && running ? { key: [began, minutes], text } : undefined
    /// ```
    pub fn inside_notice(
        &self,
        id: &str,
    ) -> Option<(String, String)> {
        let (began, command, found, running) = self.running()?;
        if !self.is_in_flight(id) {
            return None;
        }
        if let Some(stuck) = self.describe() {
            return Some((format!("{began}:{}", running / 60_000), stuck));
        }
        let detail: String = match found {
            Some(text) => format!(" ({text})"),
            None => String::new(),
        };
        return Some((
            format!("{began}:start"),
            format!(
                "monochromatic-git-policy-cli: this commit is inside a write of the commit service, `{command}`{detail}. The service never ends a write \
             in the middle, so if it does not return this command reports that the result is not known after its wait.\n"
            ),
        ));
    }
}
