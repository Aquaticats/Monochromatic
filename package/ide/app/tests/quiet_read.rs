//! A read that no change notification asked for accepts a file only after it has been quiet for the
//! write-quiet period,
//!  so a safety-sweep read that meets a save in progress shows nothing of it.

/// What:
///  the quiet read,
///  its outcome,
///  the reader worker,
///  and the shipped quiet period.
/// Why:
///  The checks run against the same reader the native timer uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readReloadIfQuiet, QuietRead, ReloadWorker, WRITE_QUIET } from 'ide';
/// ```
use ide_app::{
    change_watch::SourceChange,
    document::Document,
    file_reload::{QuietRead, read_reload_if_quiet},
    refresh_policy::{REREAD_GAP, SourceRefresh, WRITE_QUIET},
    reload_worker::{ReloadReply, ReloadRequest, ReloadWorker},
};
/// What:
///  `File` opens a file to change its modification time;
///  `Path` borrows a path;
///  `Duration`,
///       `Instant` and `SystemTime` are a time span,
///  a monotonic time,
///  and a wall-clock time.
/// Why:
///  The fixtures set the modification time directly,
///  so the outcome does not depend on how fast
///      the test machine is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { utimesSync } from 'node:fs';
/// ```
use std::{
    fs::File,
    path::Path,
    time::{Duration, Instant, SystemTime},
};

/// Quiet period the reader tests require:
///  long enough that a worker which takes a while to start
/// (it loads the highlighting engine first) still reads a fixture written now within it.
const QUIET: Duration = Duration::from_secs(10);

/// Text the displayed document holds before the read.
const DISPLAYED: &str = "I am a big cat\n";

/// Text on disk,
///  which differs from the displayed text.
const ON_DISK: &str = "I was a big cat, but now I am a human!\n";

/// Write `ON_DISK` to `path` and give it the modification time `modified`.
fn write_with_time(path: &Path, modified: SystemTime) {
    std::fs::write(path, ON_DISK).expect("fixture source");
    // What: `File::options().write(true).open` opens for writing without truncating; `set_modified`
    //       changes the modification time only.
    // Why: The fixture's age decides the outcome, not the moment the test happened to write it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // utimesSync(path, modified, modified);
    // ```
    File::options()
        .write(true)
        .open(path)
        .expect("open fixture for its time")
        .set_modified(modified)
        .expect("set the fixture's modification time");
}

/// Name an outcome,
///  so a failed comparison prints it.
fn outcome(read: &QuietRead) -> &'static str {
    // What: `match` on the enum: `Read(Some(_))` is a changed file, `Read(None)` an unchanged one.
    // Why: The outcome holds a prepared replacement that cannot be printed, so tests compare its name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return read.kind === 'recentlyWritten' ? 'recent' : read.reload === undefined ? 'unchanged' : 'changed';
    // ```
    match read {
        QuietRead::RecentlyWritten => {
            return "recent";
        }
        QuietRead::Read(None) => {
            return "unchanged";
        }
        QuietRead::Read(Some(_)) => {
            return "changed";
        }
    }
}

/// Wait for one worker reply,
///  failing after three seconds.
fn reply(worker: &mut ReloadWorker) -> ReloadReply {
    let start = Instant::now();
    loop {
        // What: `expect` fails the test on a stopped worker; `if let Some(response)` takes a finished reply.
        // Why: A reply is polled the way the UI polls it, without blocking.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const response = worker.tryTake(); if (response !== undefined) return response;
        // ```
        if let Some(response) = worker.try_take().expect("poll source worker") {
            return response;
        }
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "source worker did not reply"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// A file modified just now,
///  or with a time ahead of this clock,
///  is dropped;
///  an old one is read.
#[test]
fn a_quiet_read_drops_a_file_written_within_the_quiet_period() {
    let fixture = tempfile::tempdir().expect("disposable source directory");
    let path = fixture.path().join("source.txt");
    let document = Document::new(DISPLAYED);
    let now = SystemTime::now();
    write_with_time(&path, now);
    let recent = read_reload_if_quiet(&document, &path, QUIET).expect("read a recent file");
    assert_eq!(
        outcome(&recent),
        "recent",
        "a file written just now was accepted"
    );
    write_with_time(&path, now + Duration::from_secs(60));
    let ahead = read_reload_if_quiet(&document, &path, QUIET).expect("read a file from ahead");
    assert_eq!(
        outcome(&ahead),
        "recent",
        "a file with a modification time ahead of this clock was accepted"
    );
    write_with_time(&path, now - QUIET - Duration::from_secs(1));
    let quiet = read_reload_if_quiet(&document, &path, QUIET).expect("read a quiet file");
    assert_eq!(
        outcome(&quiet),
        "changed",
        "a file quiet for longer than the quiet period was not read"
    );
}

/// The worker reports a recent write without a result to apply,
///  and reads the same file once quiet.
#[test]
fn the_worker_reports_a_recent_write_instead_of_its_bytes() {
    let fixture = tempfile::tempdir().expect("disposable source directory");
    let path = fixture.path().join("source.txt");
    let document = Document::new(DISPLAYED);
    let mut worker = ReloadWorker::new().expect("start source worker");
    let now = SystemTime::now();
    write_with_time(&path, now);

    assert!(
        worker
            .request(request(&path, &document))
            .expect("first timer read")
    );
    let recent = reply(&mut worker);
    assert!(recent.recent_write, "a recent write was not reported");
    // What: `if let Ok(Some(_))` matches a reply that carries a prepared replacement.
    // Why: A recent write must carry nothing the UI could install.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (recent.result.ok && recent.result.value !== undefined) fail();
    // ```
    if let Ok(Some(_)) = recent.result {
        panic!("a recent write carried a result to apply");
    }
    assert!(
        recent.result.is_ok(),
        "a recent write was reported as a read failure"
    );
    write_with_time(&path, now - QUIET - Duration::from_secs(1));
    assert!(
        worker
            .request(request(&path, &document))
            .expect("second timer read")
    );
    let quiet = reply(&mut worker);
    assert!(!quiet.recent_write, "a quiet file was reported as written");
    // Only a prepared replacement shows that the quiet file's change was read.
    let Ok(Some(_)) = quiet.result else {
        panic!("a quiet file's change was not prepared");
    };
}

/// The native timer asks for quiet on every read except the one an unread notification asked for:
/// the first read,
///  the sweep,
///  an unwatched timer,
///  and a highlighting retry all require it.
#[test]
fn only_the_read_a_notification_asked_for_skips_the_quiet_check() {
    let start = Instant::now();
    let mut refresh = SourceRefresh::default();
    assert!(
        !refresh.has_unread_change(),
        "the first read counted as asked for by a notification"
    );
    refresh.requested(start);
    refresh.changed(SourceChange::Unsettled, start + REREAD_GAP);
    assert!(
        refresh.has_unread_change(),
        "an unread notification was not reported"
    );
    refresh.requested(start + REREAD_GAP + WRITE_QUIET);
    assert!(
        !refresh.has_unread_change(),
        "a read that admitted the notification left it unread"
    );
}

/// A reread with no write behind it (a new watch,
///  a newly displayed file,
///  a full reread) is read
/// promptly but with the quiet requirement,
///  and it never ends the wait of an unfinished write.
#[test]
fn a_reread_with_no_write_behind_it_requires_quiet() {
    let start = Instant::now();
    let later = start + REREAD_GAP;
    let mut reread = SourceRefresh::default();
    reread.requested(start);
    reread.changed(SourceChange::Reread, later);
    assert!(reread.due(later, false), "a reread was not read promptly");
    assert!(
        !reread.has_unread_change(),
        "a reread with no write behind it skipped the quiet check"
    );
    let mut writing = SourceRefresh::default();
    writing.requested(start);
    writing.changed(SourceChange::Unsettled, later);
    writing.changed(SourceChange::Reread, later);
    assert!(
        !writing.due(later, false),
        "a reread ended the wait for an unfinished write"
    );
    assert!(
        writing.has_unread_change(),
        "a reread hid the pending write notification"
    );
}

/// What:
///  The request a timer read sends;
///  `clone` copies the path and snapshot.
/// Why:
///  Both reads send the same request;
///  only the file's age differs between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = (path, snapshot) => ({ path, snapshot, generation: 7, requireQuiet: QUIET });
/// ```
fn request(path: &Path, snapshot: &Document) -> ReloadRequest {
    return ReloadRequest {
        path: path.to_path_buf(),
        snapshot: snapshot.clone(),
        generation: 7,
        highlight_unchanged: false,
        require_quiet: Some(QUIET),
    };
}
