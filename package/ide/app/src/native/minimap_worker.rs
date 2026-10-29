//! The minimap's worker thread:
//! it builds the scene of the picture of the whole text off the window thread,
//!  newest request first,
//! and hands the finished scene back to be drawn by the GPU.
//!  Its serving loop and the row cache are in `minimap_worker/serve.rs`.

/// The row key of the vertical mapping a picture is built from.
use super::rows::RowKey;
/// A worker that cannot start or has stopped is a visible diagnostic.
use anyhow::{Context, Result, bail};
/// Hints and diagnostics that decorate the lines.
use ide_app::annotation::Mark;
/// The annotation inks.
use ide_app::annotation_layout::AnnotationColors;
/// The palette whose inks the picture uses.
use ide_app::code_colors::CodeColors;
/// Canonical source,
/// cloned as a rope snapshot.
use ide_app::document::Document;
/// The matches of the in-file find.
use ide_app::find::{FindRange, FindRanges};
/// Which text a picture is for.
use ide_app::language::identity::DocumentStamp;
/// The fill inks.
use ide_app::minimap::Fills;
/// The vertical mapping the picture is built from,
/// copied into each job.
use ide_app::row_map::RowMap;
/// Shaping finds where wrapped lines break.
use ide_app::shaped_text::TextShaper;
/// Syntax classes,
/// compared by shared identity.
use ide_app::source_style::SourceStyles;
/// Shaping choices a worker shaper must match.
use ide_app::source_typography::SourceTypography;
/// The scene and the size of the texture it is drawn into.
use ide_app::tile_scene::TileScene;
/// Blocks of virtual rows.
use ide_app::virtual_row::Block;
/// Owned snapshots and the newest-request flag are shared between the window thread and the worker.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, Sender, TryRecvError, channel},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// The serving loop and the cache of rendered units.
mod serve;
/// The test gate that holds the worker.
#[cfg(test)]
pub(super) use serve::MINIMAP_GATE;

/// What:
///  The name of the worker thread.
/// Why:
///  A debug build counts pictures built on any other thread,
/// so a build that moves onto the window thread is seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const THREAD_NAME = 'ide-minimap';
/// ```
pub(super) const THREAD_NAME: &str = "ide-minimap";

/// What:
///  Pictures built on a thread other than the worker,
/// counted in debug builds.
/// Why:
///  The structural tests assert that the window thread never builds a picture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let offWorkerBuilds = 0;
/// ```
#[cfg(debug_assertions)]
pub(super) static OFF_WORKER_BUILDS: AtomicU64 = AtomicU64::new(0);

/// What:
///  What a picture is valid for:
/// the text,
/// its syntax classes,
/// the vertical mapping,
/// the wrap width,
/// the palette,
///  the display scale,
/// the minimap's size,
/// the bands near the visible part of the picture,
/// the device's texture limit,
/// and what the window draws over the text (the selection,
/// the matches,
/// and the annotations).
/// Why:
///  A request is posted only when this changes,
/// and a finished picture is shown only while it still describes
///  the displayed text;
/// the accept rule compares its stamp with the window's.
///  The scroll offset is none of it but through the bands it shows:
/// a scroll that keeps the same bands moves the picture and the viewport box in the markup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Key = { stamp: Stamp; styles: number; rows: RowKey | undefined; wrap: number; colors: Colors; scale: number;
///   size: [number, number]; window: [number, number]; selection?: [number, number]; find: [number, number];
///   annotations: number };
/// ```
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Key {
    /// File generation and text revision.
    pub(super) stamp: DocumentStamp,
    /// Address of the syntax snapshot.
    pub(super) styles: usize,
    /// The row key of the vertical mapping that was installed.
    pub(super) rows: Option<RowKey>,
    /// Exact bits of the wrap width,
    /// zero while the view has none.
    pub(super) wrap: u32,
    /// The palette and the scheme.
    pub(super) colors: CodeColors,
    /// Exact bits of the display scale.
    pub(super) scale: u32,
    /// Exact bits of the minimap's width and height.
    pub(super) size: (u32, u32),
    /// The numbers of the first band the picture holds and of the one past its last,
    /// counted from the top of the text.
    pub(super) window: (usize, usize),
    /// The largest texture side the device offers as the shaped text records it:
    /// the textures the picture is stacked from are cut for it.
    pub(super) texture_limit: u32,
    /// The selected characters,
    /// nothing while the caret is alone.
    pub(super) selection: Option<(usize, usize)>,
    /// The address and length of the list of matches.
    pub(super) find: (usize, usize),
    /// The match the selection covers.
    pub(super) active: Option<(usize, usize)>,
    /// The change counter of the annotations.
    pub(super) annotations: u64,
    /// The inks of the fills and the annotations.
    pub(super) inks: (Fills, AnnotationColors),
}

/// What:
///  One request:
/// the owned inputs of a picture.
/// Why:
///  The worker must not borrow native state,
/// which lives on the window thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Job = { id: number; key: Key; document: Document; styles: Styles; map: RowMap; /* paint inputs */ };
/// ```
pub(super) struct Job {
    /// Request number;
    /// a finished picture of an older request is never shown over a newer one.
    pub(super) id: u64,
    /// What the picture is valid for,
    /// echoed back.
    pub(super) key: Key,
    /// Rope snapshot at the revision the job was taken from,
    /// with the reading position that is the selection.
    pub(super) document: Document,
    /// Syntax classes,
    /// shared by identity.
    pub(super) styles: SourceStyles,
    /// Shaping choices the worker shaper must match.
    pub(super) typography: SourceTypography,
    /// The vertical mapping the picture is built from.
    pub(super) map: RowMap,
    /// The palette of the source glyphs.
    pub(super) colors: CodeColors,
    /// The inks of hints and diagnostics.
    pub(super) inks: AnnotationColors,
    /// The inks of the selection and match fills.
    pub(super) fills: Fills,
    /// Every block of virtual rows.
    pub(super) blocks: Vec<Arc<Block>>,
    /// Every diagnostic mark.
    pub(super) marks: Vec<Mark>,
    /// The matches of the in-file find.
    pub(super) ranges: FindRanges,
    /// The match the selection covers.
    pub(super) active: Option<FindRange>,
    /// Wrap width in logical pixels,
    /// or nothing while the view has none.
    pub(super) wrap: Option<f32>,
    /// Physical pixels per logical pixel.
    pub(super) scale: f32,
    /// Logical width and height of the minimap.
    pub(super) size: (f32, f32),
}

/// What:
///  A finished picture of a request:
/// the scene to draw,
/// the row of the whole picture it starts at,
/// and what it cost.
/// Why:
///  The picture holds the bands near the visible part of the whole picture,
/// so the window needs to know where it stands to place it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reply = { id: number; key: Key; top: number; chunks: TileScene[]; stats: Stats; elapsed: number };
/// ```
pub(super) struct Reply {
    /// The request this answers.
    pub(super) id: u64,
    /// What the picture is valid for.
    pub(super) key: Key,
    /// The row of the whole picture that the picture's first row is,
    /// in physical pixels.
    pub(super) top: u32,
    /// The scene of each chunk of the picture,
    /// top first,
    /// each with the size of the texture region it fills.
    pub(super) chunks: Vec<TileScene>,
    /// What the build cost and left out.
    pub(super) stats: ide_app::minimap::Stats,
    /// Time from the start of the request to the end of this build.
    pub(super) elapsed: Duration,
}

/// What:
///  The handle the window thread holds:
/// the request channel,
/// the reply channel,
/// and the newest request's number.
/// Why:
///  Requests replace each other;
/// polling returns each finished picture in turn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MinimapWorker { post(job: Job): void; poll(): Reply[] }
/// ```
pub(super) struct MinimapWorker {
    /// Taking this sender closes the request channel,
    /// which ends the worker.
    jobs: Option<Sender<Job>>,
    /// The window thread polls this receiver without blocking.
    replies: Receiver<Reply>,
    /// Number of the newest request;
    /// the build of an older one stops at its next check.
    newest: Arc<AtomicU64>,
    /// Detached on drop,
    /// never joined:
    /// closing the window does not wait for a build.
    thread: Option<JoinHandle<()>>,
}

/// Start,
/// feed,
/// and poll the worker.
impl MinimapWorker {
    /// Start a named worker that builds with `shaper`.
    pub(super) fn new(shaper: TextShaper) -> Result<Self> {
        let (job_sender, job_receiver) = channel::<Job>();
        let (reply_sender, reply_receiver) = channel::<Reply>();
        let newest = Arc::new(AtomicU64::new(0));
        let worker_newest = Arc::clone(&newest);
        let thread = thread::Builder::new()
            .name(THREAD_NAME.to_string())
            .spawn(move || {
                // `catch_unwind` stops an unwinding build at the thread boundary: a detached thread has no join to
                // report a panic, so it is logged here, and the next request starts a new worker.
                let outcome = catch_unwind(AssertUnwindSafe(
                    #[cfg_attr(
                        dylint_lib = "monochromatic_lint_closure",
                        expect(
                            avoidable_closure,
                            reason = "catch_unwind accepts only a callable without parameters, so the shaper and the channel ends the worker serves with must be moved into a closure"
                        )
                    )]
                    || {
                        serve::serve(shaper, &job_receiver, &reply_sender, &worker_newest);
                    },
                ));
                if outcome.is_err() {
                    tracing::error!(
                        "minimap worker panicked; the next picture starts a new worker"
                    );
                }
            })
            .context("Cannot start the minimap worker")?;
        return Ok(Self {
            jobs: Some(job_sender),
            replies: reply_receiver,
            newest,
            thread: Some(thread),
        });
    }

    /// Make `job` the request the worker builds next,
    /// stopping the build of an older one.
    pub(super) fn post(&mut self, job: Job) -> Result<()> {
        self.newest.store(job.id, Ordering::SeqCst);
        let sender = self
            .jobs
            .as_ref()
            .context("The minimap worker is closed; a new one starts with the next picture")?;
        if let Err(error) = sender.send(job) {
            return Err(anyhow::anyhow!("{error}").context("Cannot send a minimap request"));
        }
        return Ok(());
    }

    /// Every finished picture waiting,
    /// oldest first;
    /// an error when the worker has stopped.
    pub(super) fn poll(&mut self) -> Result<Vec<Reply>> {
        let mut found = Vec::new();
        loop {
            match self.replies.try_recv() {
                Ok(reply) => found.push(reply),
                Err(TryRecvError::Empty) => return Ok(found),
                Err(TryRecvError::Disconnected) => {
                    self.jobs.take();
                    bail!(
                        "The minimap worker stopped unexpectedly; a new one starts with the next picture"
                    );
                }
            }
        }
    }
}

/// Close input and detach the thread,
/// so closing the window never waits for a build in flight.
impl Drop for MinimapWorker {
    /// The detached thread ends after its current build,
    /// when the closed channel ends its loop.
    fn drop(&mut self) {
        self.jobs.take();
        if self.thread.take().is_some() {
            tracing::debug!("minimap worker detached");
        }
    }
}

/// Fails to compile when `T` cannot cross to another thread.
fn assert_send<T: Send>() {}

/// Names every type that crosses to the worker thread.
fn assert_crossing_types_are_send() {
    assert_send::<Job>();
    assert_send::<Reply>();
    assert_send::<TextShaper>();
}

/// Compile-time proof that jobs and replies may cross to the worker thread.
const _: fn() = assert_crossing_types_are_send;
