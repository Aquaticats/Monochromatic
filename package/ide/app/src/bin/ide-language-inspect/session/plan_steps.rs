//! The plan steps one session runs:
//!  each step of the plan is dispatched here,
//!  and the session's final counters are printed at its end.

/// The session whose steps these are.
use super::Session;
/// The conditions a step waits for.
use super::condition::{
    DiagnosticsWithin,
    HintsEnough,
    Never,
    ServerReady,
};
/// Printing an observation line with the session's elapsed milliseconds.
use super::emit;
/// Plan steps.
use crate::Step;
/// Failures name the operation that failed.
use anyhow::Result;
/// The application's own document.
use ide_app::document::Document;
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// The handle's requests and the records a step hands it.
use ide_app::language::{
    hints::HintWindow,
    reply::RequestKind,
    sync::DocumentOpen,
};
/// JSON values and the literal-building macro.
use serde_json::json;
/// Deadlines of the waits a step makes.
use std::time::Duration;

/// One plan step at a time,
/// and the counters when the session ends.
impl Session {
    /// What:
    ///  Run one plan step and print its result.
    /// Why:
    ///  A step that does not reach its goal is recorded as such;
    ///  only a broken session is an error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(number: number, step: Step): void
    /// ```
    pub fn run(
        &mut self,
        number: usize,
        step: &Step,
    ) -> Result<()> {
        let result = match step {
            Step::Open {
                file,
            } => {
                let path = self.project.join(file);
                let text = match std::fs::read_to_string(&path) {
                    Ok(found) => found,
                    Err(error) => {
                        return Err(anyhow::Error::new(error)
                            .context(format!("Cannot read {}", path.display())));
                    }
                };
                self.file += 1;
                self.document = Document::new(&text);
                self.path = path.clone();
                self.replies.clear();
                self.diagnostics = None;
                self.hints = None;
                let sent = self.worker.open(DocumentOpen {
                    path,
                    // `clone` on a rope copies a small handle.
                    text: self.document.text().clone(),
                    stamp: self.stamp(),
                })?;
                json_value!({ "sent": sent })
            }
            Step::Ready {
                seconds,
            } => {
                json_value!({ "ready": self.until(Duration::from_secs(*seconds), &ServerReady)? })
            }
            Step::Request {
                kind,
                at,
                until,
                seconds,
            } => self.request(kind, *at, until, *seconds)?,
            Step::Hints {
                first,
                visible,
                minimum,
                seconds,
            } => {
                let window = HintWindow {
                    first_line: *first,
                    visible_lines: *visible,
                };
                let sent = self.worker.request_hints(self.stamp(), window)?;
                // Hints are latest-value state: an unchanged snapshot is not published again, so the
                // one already held counts when it describes the displayed revision.
                let enough = HintsEnough {
                    stamp: self.stamp(),
                    least: *minimum,
                };
                json_value!({ "sent": sent, "enough": self.until(Duration::from_secs(*seconds), &enough)? })
            }
            Step::Diagnostics {
                minimum,
                maximum,
                seconds,
            } => {
                let within = DiagnosticsWithin {
                    stamp: self.stamp(),
                    least: *minimum,
                    most: *maximum,
                };
                json_value!({ "within": self.until(Duration::from_secs(*seconds), &within)? })
            }
            Step::Reload {
                text,
            } => {
                self.reload(text)?;
                json_value!({ "revision": self.document.revision() })
            }
            Step::Stale {
                at,
                text,
                settle,
            } => {
                let before = self.worker.fence_counts();
                self.replies.clear();
                let overtaken = self.ask(RequestKind::Hover, *at)?;
                self.reload(text)?;
                self.until(Duration::from_millis(*settle), &Never)?;
                let after = self.worker.fence_counts();
                let mut delivered = false;
                for reply in self.replies.iter() {
                    if Some(reply.request) == overtaken {
                        delivered = true;
                        break;
                    }
                }
                json_value!({
                    "request": overtaken,
                    "delivered": delivered,
                    "droppedStaleRevision": after.stale_revision - before.stale_revision,
                })
            }
            Step::Write {
                file,
                text,
            } => self.write(file, text)?,
            Step::Folders {
                minimum,
                seconds,
            } => self.folders(*minimum, *seconds)?,
            Step::Sleep {
                milliseconds,
            } => {
                self.until(Duration::from_millis(*milliseconds), &Never)?;
                json!({})
            }
            Step::Close => json_value!({ "sent": self.worker.close()? }),
        };
        emit(
            self.started,
            json_value!({ "step": number, "do": format!("{step:?}").split_whitespace().next(), "result": result }),
        );
        return Ok(());
    }

    /// Print the final counters;
    ///  dropping the session then stops every server.
    pub fn finish(self) {
        let counts = self.worker.fence_counts();
        emit(
            self.started,
            json_value!({ "done": true, "fence": format!("{counts:?}") }),
        );
    }
}
