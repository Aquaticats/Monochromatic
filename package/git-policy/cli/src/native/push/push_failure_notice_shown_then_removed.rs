//! What:
//!  One record of a failed backup push as a command holds it before it shows it:
//! the words for the person,
//! the
//!  record's own name,
//! the name this command claimed it under,
//! and the step that removes the claim once those words are out.
//! Why:
//!  Reading a record and removing it are two moments,
//! and the claim is what stands between two commands that both read it before
//!  either showed it:
//! only the command whose rename won holds the claim,
//! and the record's own name stays free for the service to
//!  replace while the words are being made.
//! A record removed before its words reach the person is gone with nothing shown,
//! so the
//!  claim goes only after the words are out,
//! and goes back to the record's name when they could not be written.
//! The reading
//!  stays in `push_failure_notice.rs`;
//! this file holds what the caller does with one record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Notice = { record: string; claim: string; text: string };
//! async function show(notice, out) {
//!   try { out.write(notice.text + '\n'); out.flush(); unlink(notice.claim) }
//!   catch (e) { rename(notice.claim, notice.record); throw e }
//! }
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  One record read from the disk,
/// held under this command's name:
/// the words the person is to be shown,
/// the record's
///  own name,
/// and the claimed name that only this command can remove.
/// Why:
///   The claim is what makes the removal safe:
/// no record the service writes,
/// and no record another command holds,
/// is at
///  that name,
/// so removing it cannot delete words nobody has read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Notice = { record: string; claim: string; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice {
    /// The record's own name,
    /// where a notice whose words could not be written goes back.
    pub(crate) record: PathBuf,
    /// The name this command claimed the record under.
    pub(crate) claim: PathBuf,
    /// The words for the person.
    pub(crate) text: String,
}

impl Notice {
    /// The words the person is shown.
    pub fn text(&self) -> &str {
        return self.text.as_str();
    }

    /// The record's own name.
    pub fn record(&self) -> &Path {
        return self.record.as_path();
    }

    /// What:
    ///  Shows the words on `out`,
    /// and removes the claim once they are out.
    /// Why:
    ///   A record whose words could not be written goes back to its own name,
    /// where the next command finds it,
    /// and one
    ///  whose words are out takes its claim with it,
    /// touching no record of the service's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { out.write(text + '\n'); out.flush() } catch (e) { rename(claim, record); throw e }
    /// unlink(claim)
    /// ```
    pub fn show(&self, out: &mut impl Write) -> std::io::Result<()> {
        if let Err(error) = writeln!(out, "{}", self.text).and_then(|()| { return out.flush(); }) {
            crate::push_failure_notice_claimed_by_one_command::give_back(self.claim.as_path(), self.record.as_path());
            return Err(error);
        }
        self.remove_what_was_read();
        return Ok(());
    }

    /// What:
    ///  Removes this command's claim,
    /// once its words are out.
    /// Why:
    ///   Nothing else was ever at that name:
    /// the service writes records under their own names,
    /// and another command's claim carries
    ///  another name,
    /// so this cannot delete a statement nobody has read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// unlink(claim)
    /// ```
    pub fn remove_what_was_read(&self) {
        #[cfg(test)]
        play_before_removal();
        let _removed = std::fs::remove_file(self.claim.as_path());
    }
}

// The step a control plays in the instant before a claim is removed, as the service writing a newer record there;
// nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The step of the control running on this thread.
    static BEFORE_REMOVAL: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// Plays `step` in the instant before a claim is removed, until the guard drops.
#[cfg(test)]
pub(crate) fn before_removal_in_a_control(step: impl Fn() + 'static) -> BeforeRemovalGuard {
    BEFORE_REMOVAL.with(|slot| { *slot.borrow_mut() = Some(Box::new(step)); });
    return BeforeRemovalGuard;
}

/// Ends the control's step when dropped.
#[cfg(test)]
pub(crate) struct BeforeRemovalGuard;

#[cfg(test)]
impl Drop for BeforeRemovalGuard {
    fn drop(&mut self) {
        BEFORE_REMOVAL.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// Plays the control's step, if one is set.
#[cfg(test)]
fn play_before_removal() {
    // Taken out of the slot while it runs, so a step that itself shows a notice does not reach this one again.
    let step: Option<Box<dyn Fn()>> = BEFORE_REMOVAL.with(|slot| { return slot.borrow_mut().take(); });
    if let Some(step) = step {
        step();
        BEFORE_REMOVAL.with(|slot| { *slot.borrow_mut() = Some(step); });
    }
}
