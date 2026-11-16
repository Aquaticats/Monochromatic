//! What:
//!  The beat one running commit service keeps,
//!  and the reading of it that tells a working service from one that stopped inside itself.
//! Why:
//!   A service that is alive but silent is one a command must not wait the whole recovering ceiling for
//! (`SPEC.md`, "A service that greeted and then went silent for `HEARTBEAT_SILENCE` is the service that stopped"):
//!  a frozen service, a debugger holding it, or a service whose threads are all in a call that never returns hands the
//! person two minutes of nothing.
//!  The service's own protocol beat cannot answer that question before its runtime exists, and the runtime is exactly what
//!  a starved service spends its time building, so the beat here is written by a thread of its own that needs no runtime:
//! one file beside the address and the note, its modification time moved every `BEAT_EVERY`.
//!  A service whose beat is older than `BEAT_PATIENCE` is not working, whatever its process state says, because a thread
//!  that only touches a file runs whenever the process runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = beatPath(dir, instance); thread(() => { while (true) touch(file); sleep(500) });
//! const working = fresh(file) || fresh(logBesideIt);
//! ```

use std::path::{Path, PathBuf};

/// The ending of the file that holds one instance's beat.
pub const BEAT_ENDING: &str = ".beat";

/// How often a running service moves its beat.
pub const BEAT_EVERY: std::time::Duration = std::time::Duration::from_millis(500);

/// What:
///  How old a beat may be before the service that keeps it is taken for one that stopped.
/// Why:
///   Four beats of margin, so a service whose beat thread is descheduled under load is still working at the second look;
/// a frozen process moves no beat at all, and the whole point is to find it in seconds, where the recovering ceiling is
/// two minutes.
pub const BEAT_PATIENCE: std::time::Duration = std::time::Duration::from_secs(2);

/// The beat of one instance, beside its address and its note.
pub fn path(socket_directory: &Path, instance: &str) -> PathBuf {
    return socket_directory.join(format!("{instance}{BEAT_ENDING}"));
}

/// What:
///  The beat one service keeps:
///  the file it moves, the object it made, and the way to give it up again.
/// Why:
///   The service makes the file before it binds and moves it while it works,
/// and at its end it removes only the file it made:
/// a second service that took the name over has a beat of its own,
/// and the kernel never hands one inode to two files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const beat = take(path); beat.start(); ... beat.removeIfStillMine()
/// ```
pub struct Beat {
    /// The file that holds the beat.
    pub path: PathBuf,
    /// The device and inode of the file this service made.
    identity: (u64, u64),
    /// The handle the moving thread uses, until the thread has it.
    file: Option<std::fs::File>,
}

impl Beat {
    /// What:
    ///  Makes the beat file and moves it once.
    /// Why:
    ///   The file is made before the socket is bound, so a reader that finds the address answering has a beat to read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const beat = take(beatPath(dir, instance))
    /// ```
    pub fn take(path: PathBuf) -> std::io::Result<Self> {
        use std::os::unix::fs::MetadataExt;
        // What: The file is this service's own, so a beat another service made is replaced rather than shared.
        // Why:  Opening one that is there hands both services the same inode, and then the end of either removes the beat the
        //       other is still keeping, which reads as a service that stopped (found by
        //       `the_thread_moves_the_beat_and_an_earlier_services_end_leaves_a_newer_beat`, 2026-11-16).
        let _gone = std::fs::remove_file(path.as_path());
        let file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_path())?;
        touch(&file);
        let metadata: std::fs::Metadata = file.metadata()?;
        return Ok(Self { path, identity: (metadata.dev(), metadata.ino()), file: Some(file) });
    }

    /// Starts the thread that moves the beat until this process ends.
    pub fn start(&mut self) {
        let Some(file) = self.file.take() else { return };
        let _started: std::io::Result<()> = std::thread::Builder::new().name(String::from("service-beat")).spawn(move || {
            loop {
                std::thread::sleep(BEAT_EVERY);
                touch(&file);
            }
        }).map(|_handle| { return (); });
    }

    /// Removes the file, when it is still the one this service made.
    pub fn remove(self) {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = std::fs::symlink_metadata(self.path.as_path())
            && (metadata.dev(), metadata.ino()) == self.identity
        {
            let _removed = std::fs::remove_file(self.path.as_path());
        }
    }
}

/// What:
///  Whether the service of `instance` is working:
///  its beat is fresh,
///  or, where no beat was ever made, its log was written lately.
/// Why:
///   The reader is a command that has a connection which greeted nothing,
/// and the two answers have two next steps: a working service is waited for
/// and one that is not is replaced
/// (`service_client_reach_service.rs`).
///  A beat file that is not there (a service of an earlier release, or one killed before it made it) falls back to the log,
/// whose lines a service writes as it starts,
///  and a log that is not there either is no sign of work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const working = fresh(beat, PATIENCE) ?? fresh(log, PATIENCE)
/// ```
pub fn is_working(socket_directory: &Path, instance: &str) -> bool {
    let beat: PathBuf = path(socket_directory, instance);
    if let Some(age) = age_of(beat.as_path()) {
        return age <= BEAT_PATIENCE;
    }
    let log: PathBuf = socket_directory.join(format!("{instance}{LOG_ENDING}"));
    return age_of(log.as_path()).is_some_and(|age| { return age <= BEAT_PATIENCE; });
}

/// The ending of one instance's log, which sits beside its beat.
const LOG_ENDING: &str = ".log";

/// Moves the beat of `path` to now, for a control that needs a service that is working.
pub fn touch_for_a_control(path: &Path) -> std::io::Result<()> {
    let file: std::fs::File = std::fs::OpenOptions::new().write(true).open(path)?;
    touch(&file);
    return Ok(());
}

/// Moves the beat's modification time to now.
fn touch(file: &std::fs::File) {
    let _moved = file.set_modified(std::time::SystemTime::now());
}

/// How long ago the file at `path` was written, or nothing when it is not there.
fn age_of(path: &Path) -> Option<std::time::Duration> {
    let modified: std::time::SystemTime = std::fs::metadata(path).ok()?.modified().ok()?;
    return modified.elapsed().ok();
}

/// Controls of the beat stay out of the release executable.
#[cfg(test)]
#[path = "service_beat_of_an_instance_tests.rs"]
mod tests;
