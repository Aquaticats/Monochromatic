//! What:
//!  The record a thread keeps of the write it is inside when that write is never ended: which landing board the thread
//!  serves, the detail that says what the write does, and the step file that tells waiting clients what the service is stuck in.
//! Why:
//!  `service_busy.rs` holds the landing board that clients ask; this part is the thread-side writing of the step into it,
//!  and depends on the board and not the other way round.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! using step = enterStep(commandText); // writes the step file until the guard is dropped
//! ```

use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::service_busy::{LandingBoard, STEP_FILE, now_millis};

/// What:
///  Whether the calling thread has a landing board.
/// Why:
///  `on_landing_thread` asks the thread-local board slot, and `LocalKey::with` hands the slot to a function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const boardIsSet = (slot: { value?: LandingBoard }) => slot.value !== undefined;
/// ```
pub(crate) fn board_is_set(slot: &RefCell<Option<Arc<LandingBoard>>>) -> bool {
    return slot.borrow().is_some();
}

/// What:
///  Text on one line: the words of `text` joined by single spaces.
/// Why:
///  The step record is a file of lines, so a command or detail with line breaks must not add lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneLine = (text: string) => text.split(/\s+/).filter(Boolean).join(' ');
/// ```
pub(crate) fn one_line(text: &str) -> String {
    return text.split_whitespace().collect::<Vec<&str>>().join(" ");
}

thread_local! {
    /// The board of the service whose landing runs on this thread; no other thread has one.
    static BOARD: RefCell<Option<Arc<LandingBoard>>> = const { RefCell::new(None) };
    /// What the landing knows about the next write that is never ended, for the words that name it.
    static DETAIL: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Whether the calling thread is the commit service's landing thread, the one thread that waits for a write it cannot end.
pub fn on_landing_thread() -> bool {
    return BOARD.with(board_is_set);
}

/// Make `board` the record of the landing that runs on the calling thread.
pub fn use_on_this_thread(board: Arc<LandingBoard>) {
    BOARD.set(Some(board));
}

/// Clears the detail when the work it describes is over.
pub struct DetailGuard;

impl Drop for DetailGuard {
    fn drop(&mut self) {
        DETAIL.set(None);
    }
}

/// Say what the write that follows does, for as long as the guard lives; the words of a stuck write carry it.
pub fn detail(text: String) -> DetailGuard {
    DETAIL.set(Some(text));
    return DetailGuard;
}

/// Clears the running step when the write ends, however it ends.
pub struct StepGuard {
    /// The file that names the step.
    file: PathBuf,
}

impl Drop for StepGuard {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(self.file.as_path());
    }
}

/// Write `text` to `path` as a complete file or not at all: a rename of a file written beside it.
pub(crate) fn publish(path: &Path, text: &str) -> bool {
    let beside: PathBuf = path.with_extension("tmp");
    let written: bool = match std::fs::File::create(beside.as_path()) {
        Ok(mut file) => file.write_all(text.as_bytes()).is_ok(),
        Err(_refused) => false,
    };
    return written && std::fs::rename(beside.as_path(), path).is_ok();
}

/// What:
///  Record that this thread is inside a write that is never ended, until the guard is dropped; nothing on a thread that has no
///  board (the wrapper's own commands, the other threads of the service).
/// Why:
///   Only the landing thread holds the service up, so only its steps are recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using step = enterStep(commandText);
/// ```
pub fn enter_step(command: String) -> Option<StepGuard> {
    let board: Arc<LandingBoard> = BOARD.with_borrow(Clone::clone)?;
    let detail: Option<String> = DETAIL.with_borrow(Clone::clone);
    let file: PathBuf = board.directory.join(STEP_FILE);
    let text: String = format!("{}\n{}\n{}\n", now_millis(), one_line(command.as_str()), detail.as_deref().map_or_else(String::new, one_line));
    // A step that cannot be recorded is still run; the clients just are not told about it.
    let _recorded = publish(file.as_path(), text.as_str());
    return Some(StepGuard { file });
}
