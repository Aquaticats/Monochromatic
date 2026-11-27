//! The polling probe around the worker handle:
//!  it steps the worker until an expected state arrives,
//!  and it stops at the patience the tests share.

use super::child_process::{
    PATIENCE,
    install_log,
};
use helix_core::Rope;
use ide_app::{
    document::Document,
    language::{
        LanguageWorker,
        config::LanguageSetup,
        diagnostics::DiagnosticsSnapshot,
        hints::HintsSnapshot,
        identity::DocumentStamp,
        reply::{
            LanguageReply,
            PositionRequest,
            RequestKind,
        },
        status::LanguageStatus,
        sync::{
            DocumentOpen,
            DocumentReload,
        },
    },
};
use std::{
    fs,
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

/// The worker handle plus everything polled from it so far.
pub struct Probe {
    /// The handle under test.
    pub worker: LanguageWorker,
    /// Latest status seen.
    pub status: Arc<LanguageStatus>,
    /// Every reply the fence let through,
    ///  in arrival order.
    pub replies: Vec<LanguageReply>,
    /// Latest diagnostics seen.
    pub diagnostics: Option<Arc<DiagnosticsSnapshot>>,
    /// Latest hints seen.
    pub hints: Option<Arc<HintsSnapshot>>,
    /// The document as the application would hold it.
    pub document: Document,
    /// File-open generation of the displayed file.
    pub file: u64,
    /// Path of the displayed file.
    pub path: PathBuf,
    /// What the current wait is for,
    ///  named in the failure when the wait times out.
    waiting_for: String,
    /// When the current wait began.
    wait_started: Instant,
}

impl Probe {
    /// Start a worker for `root` with application-supplied definitions.
    pub fn new(
        root: &Path,
        definitions: String,
    ) -> Self {
        let setup = LanguageSetup {
            extra_languages: Some(definitions),
            ..LanguageSetup::unconfined()
        };
        return Self::with_setup(root, setup);
    }

    /// Start a worker for `root` with an explicit setup.
    pub fn with_setup(
        root: &Path,
        setup: LanguageSetup,
    ) -> Self {
        install_log();
        let worker = LanguageWorker::with_setup(root, setup).expect("language worker");
        return Self {
            worker,
            status: Arc::new(LanguageStatus::closed()),
            replies: Vec::new(),
            diagnostics: None,
            hints: None,
            document: Document::new(""),
            file: 0,
            path: PathBuf::new(),
            waiting_for: String::new(),
            wait_started: Instant::now(),
        };
    }

    /// The stamp of the displayed text.
    pub fn stamp(&self) -> DocumentStamp {
        return DocumentStamp {
            file: self.file,
            revision: self.document.revision(),
        };
    }

    /// Write `text` to `path` and display it,
    ///  as the application does after reading a file.
    pub fn open(
        &mut self,
        path: &Path,
        text: &str,
    ) {
        fs::write(path, text).expect("source file");
        self.display(path, text);
    }

    /// Display `text` for an existing `path` without writing it.
    pub fn display(
        &mut self,
        path: &Path,
        text: &str,
    ) {
        self.file += 1;
        self.document = Document::new(text);
        self.path = path.to_path_buf();
        let sent = self
            .worker
            .open(DocumentOpen {
                path: path.to_path_buf(),
                text: Rope::from_str(text),
                stamp: self.stamp(),
            })
            .expect("worker accepts open");
        assert!(sent, "command queue full");
    }

    /// Replace the file on disk and apply the reload exactly as the application's reload path does.
    pub fn reload(
        &mut self,
        text: &str,
    ) {
        fs::write(&self.path, text).expect("changed source file");
        let reload = self.document.prepare_reload(text);
        let command = DocumentReload::from_reload(self.file, &reload);
        assert!(self.document.apply_reload(reload), "reload accepted");
        assert!(self.worker.reload(command).expect("worker accepts reload"));
    }

    /// Send a position request and return its number.
    pub fn request(
        &mut self,
        kind: RequestKind,
        position: usize,
    ) -> u64 {
        let request = PositionRequest {
            stamp: self.stamp(),
            kind,
            position,
        };
        return self
            .worker
            .request(request)
            .expect("worker accepts request")
            .expect("command queue has room");
    }

    /// Drain everything the worker published since the last poll.
    pub fn poll(&mut self) {
        if let Some(status) = self.worker.try_take_status().expect("status poll") {
            self.status = status;
        }
        while let Some(reply) = self.worker.try_take_reply().expect("reply poll") {
            self.replies.push(reply);
        }
        if let Some(diagnostics) = self
            .worker
            .try_take_diagnostics()
            .expect("diagnostics poll")
        {
            self.diagnostics = Some(diagnostics);
        }
        if let Some(hints) = self.worker.try_take_hints().expect("hints poll") {
            self.hints = Some(hints);
        }
    }

    /// Begin a wait for `what` and poll once,
    ///  so the first check of the caller's loop reads fresh state.
    /// The caller loops `while !condition { probe.step(); }`.
    pub fn wait_for(
        &mut self,
        what: &str,
    ) {
        self.waiting_for = what.to_string();
        self.wait_started = Instant::now();
        self.poll();
    }

    /// One more round of the wait begun by `wait_for`:
    ///  fail with the last observed state when the patience ran out,
    ///  else pause and poll again.
    pub fn step(&mut self) {
        assert!(
            self.wait_started.elapsed() < PATIENCE,
            "timed out waiting for {}; status {:?}; replies {:?}; diagnostics {:?}; hints {:?}",
            self.waiting_for,
            self.status,
            self.replies,
            self.diagnostics,
            self.hints
        );
        std::thread::sleep(Duration::from_millis(5));
        self.poll();
    }
}
