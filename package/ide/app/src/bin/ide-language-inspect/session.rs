//! One inspection session:
//!  the worker handle,
//!  the document as the application holds it,
//!  and
//! the polling loop every step waits in.

/// Rendering of observations.
use crate::render;
/// Failures name the operation that failed.
use anyhow::{
    Result,
    bail,
};
/// The conditions steps wait for.
use condition::{
    Answered,
    Condition,
};
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// The handle and the types it exchanges.
use ide_app::language::{
    LanguageWorker,
    config::LanguageSetup,
    diagnostics::DiagnosticsSnapshot,
    hints::HintsSnapshot,
    identity::DocumentStamp,
    reply::{
        LanguageReply,
        PositionRequest,
        RequestKind,
        RequestOutcome,
    },
    status::LanguageStatus,
    sync::DocumentReload,
};
/// The application's own document and reload path,
///  and its change watcher and project boundary.
use ide_app::{
    change_watch::ChangeWatcher,
    document::Document,
    workspace::Workspace,
};
/// JSON values.
use serde_json::Value;
/// What:
///  `Path`/`PathBuf` are borrowed and owned filesystem paths;
///  `Arc` is a thread-safe shared
///       pointer;
///  `Duration` and `Instant` measure time.
/// Why:
///  Every wait is bounded,
///  so a missing answer ends the step instead of hanging it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = Date.now() + seconds * 1000;
/// ```
use std::{
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
    time::{
        Duration,
        Instant,
    },
};

/// The conditions a step waits for.
mod condition;
/// The steps about folders watched for the servers,
///  and the relay the application's tick performs.
mod watching;

/// Pause between polls,
///  as the application's timer polls its workers.
const POLL: Duration = Duration::from_millis(20);

/// Pause before a position request is repeated while a server warms up.
const REPEAT: Duration = Duration::from_millis(500);

/// One plan step,
/// dispatched to the session,
///  and the counters printed when it ends.
mod plan_steps;

/// Everything one session holds.
pub struct Session {
    /// The handle under inspection.
    worker: LanguageWorker,
    /// What:
    ///  `Option<ChangeWatcher>` is the project's change watcher,
    ///  or `None` in the positive control.
    /// Why:
    ///  The application watches folders for the servers;
    ///  the control shows what happens without it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watcher?: ChangeWatcher;
    /// ```
    watcher: Option<ChangeWatcher>,
    /// Resolved project root.
    project: PathBuf,
    /// Path of the displayed file.
    path: PathBuf,
    /// The displayed document.
    document: Document,
    /// File-open generation.
    ///  `u64` is an unsigned 64-bit counter.
    file: u64,
    /// Latest status seen.
    status: Arc<LanguageStatus>,
    /// Replies not yet consumed by a step.
    replies: Vec<LanguageReply>,
    /// Latest diagnostics seen.
    diagnostics: Option<Arc<DiagnosticsSnapshot>>,
    /// Latest hints seen.
    hints: Option<Arc<HintsSnapshot>>,
    /// When the session started,
    ///  for timestamps.
    started: Instant,
}

/// Print one observation line with the milliseconds since the session started.
fn emit(
    started: Instant,
    mut record: Value,
) {
    record["ms"] = json_value!(started.elapsed().as_millis());
    println!("{record}");
}

/// Session steps.
impl Session {
    /// Start the worker for a project;
    ///  no server starts until a file is opened.
    pub fn new(
        project: &Path,
        setup: LanguageSetup,
        forward: bool,
    ) -> Result<Self> {
        // The trailing `?` returns the start error to the caller.
        let worker = LanguageWorker::with_setup(project, setup)?;
        let watcher = if forward {
            Some(ChangeWatcher::new(Workspace::new(project)?)?)
        } else {
            None
        };
        // `Ok(...)` is the success variant of `Result`.
        return Ok(Self {
            worker,
            watcher,
            project: project.to_path_buf(),
            path: PathBuf::new(),
            document: Document::new(""),
            file: 0,
            status: Arc::new(LanguageStatus::closed()),
            replies: Vec::new(),
            diagnostics: None,
            hints: None,
            started: Instant::now(),
        });
    }

    /// The stamp of the displayed text.
    fn stamp(&self) -> DocumentStamp {
        return DocumentStamp {
            file: self.file,
            revision: self.document.revision(),
        };
    }

    /// What:
    ///  Take everything the worker published,
    ///  printing each change.
    ///  `&mut self` allows
    ///       storing what was taken.
    /// Why:
    ///  This is the application's polling contract:
    ///  nothing blocks,
    ///  stale results never appear.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// poll() { for (const change of worker.takeAll()) { record(change); } }
    /// ```
    fn poll(&mut self) -> Result<()> {
        self.relay_file_changes();
        // `if let Some(x) = ...?` runs the block only when something new was published.
        if let Some(status) = self.worker.try_take_status()? {
            emit(
                self.started,
                json_value!({ "status": render::status(&status) }),
            );
            self.status = status;
        }
        // `while let Some(x) = ...?` repeats until the reply queue is empty.
        while let Some(reply) = self.worker.try_take_reply()? {
            emit(
                self.started,
                json_value!({ "reply": render::reply(&reply) }),
            );
            self.replies.push(reply);
        }
        if let Some(diagnostics) = self.worker.try_take_diagnostics()? {
            emit(
                self.started,
                json_value!({ "diagnostics": render::diagnostics(&diagnostics, self.document.text()) }),
            );
            // `Some(...)` is the "value present" variant of `Option`.
            self.diagnostics = Some(diagnostics);
        }
        if let Some(hints) = self.worker.try_take_hints()? {
            emit(
                self.started,
                json_value!({ "hints": render::hints(&hints, self.document.text()) }),
            );
            self.hints = Some(hints);
        }
        return Ok(());
    }

    /// What:
    ///  Poll until `done` accepts the session or the time is up;
    ///  returns whether it did.
    ///       `&dyn Condition` borrows any value that implements the `Condition` trait.
    /// Why:
    ///  Real servers answer after an unknown warm-up;
    ///  every step states its own limit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// until(seconds: number, done: (session: Session) => boolean): boolean
    /// ```
    fn until(
        &mut self,
        limit: Duration,
        done: &dyn Condition,
    ) -> Result<bool> {
        let start = Instant::now();
        loop {
            self.poll()?;
            if done.holds(self) {
                return Ok(true);
            }
            if start.elapsed() >= limit {
                return Ok(false);
            }
            std::thread::sleep(POLL);
        }
    }

    /// Send one position request;
    ///  nothing is returned when the command queue is full.
    fn ask(
        &mut self,
        kind: RequestKind,
        at: usize,
    ) -> Result<Option<u64>> {
        let request = PositionRequest {
            stamp: self.stamp(),
            kind,
            position: at,
        };
        return self.worker.request(request);
    }

    /// Write new content to the displayed file and apply the reload as the application does.
    fn reload(
        &mut self,
        text: &str,
    ) -> Result<()> {
        if let Err(error) = std::fs::write(&self.path, text) {
            return Err(
                anyhow::Error::new(error).context(format!("Cannot write {}", self.path.display()))
            );
        }
        let reload = self.document.prepare_reload(text);
        let command = DocumentReload::from_reload(self.file, &reload);
        if !self.document.apply_reload(reload) {
            bail!("the document refused its own reload");
        }
        if !self.worker.reload(command)? {
            bail!("the command queue was full");
        }
        return Ok(());
    }

    /// What:
    ///  Repeat a position request until its outcome is the wanted kind or time is up.
    /// Why:
    ///  A server that is still loading answers `null` or "content modified";
    ///  the record
    ///      keeps every attempt so the warm-up is visible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// request(kind: string, at: number, until: string, seconds: number): object
    /// ```
    fn request(
        &mut self,
        kind: &str,
        at: usize,
        until: &str,
        seconds: u64,
    ) -> Result<Value> {
        let wanted = match kind {
            "definition" => RequestKind::Definition,
            "references" => RequestKind::References,
            "hover" => RequestKind::Hover,
            other => bail!("unknown request kind {other}"),
        };
        let start = Instant::now();
        let limit = Duration::from_secs(seconds);
        let mut attempts = 0;
        loop {
            attempts += 1;
            self.replies.clear();
            let Some(number) = self.ask(wanted, at)? else {
                bail!("the command queue was full");
            };
            // What: `Answered` holds the request number by copy.
            // Why: The wait ends when the last server answered this request.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const answered = (session: Session) => session.replies.some(r => r.request === number && r.remaining === 0);
            // ```
            let answered = Answered {
                request: number,
            };
            let complete = self.until(limit.saturating_sub(start.elapsed()), &answered)?;
            let mut matched = false;
            for reply in self.replies.iter() {
                // `matches!` is true when the pair fits one of the listed patterns.
                if matches!(
                    (&reply.outcome, until),
                    (RequestOutcome::Hover(_), "hover")
                        | (RequestOutcome::Locations(_), "locations")
                        | (RequestOutcome::Empty, "empty")
                        | (_, "")
                ) {
                    matched = true;
                    break;
                }
            }
            if matched || start.elapsed() >= limit {
                return Ok(
                    json_value!({ "attempts": attempts, "complete": complete, "matched": matched }),
                );
            }
            std::thread::sleep(REPEAT);
        }
    }
}
