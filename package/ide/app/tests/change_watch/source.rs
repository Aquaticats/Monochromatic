//! Displayed-file notifications:
//!  finished and unfinished writes,
//!  and silence for the IDE's own reads.

/// Waits and fixture helpers shared by this crate.
use super::support::{Arrival, quiet, set, settle, start};
/// Settled changes are read at once;
///  unsettled ones wait for the writer to go quiet.
use ide_app::change_watch::{ChangeWatcher, SourceChange};
/// What:
///  `OpenOptions` opens a file with chosen flags;
///  `Write` provides `write_all` on the open handle.
/// Why:
///  Holding a written file open shows what an unfinished write looks like before its close.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = openSync(path, 'w'); writeSync(handle, 'partial');
/// ```
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

/// Return the latest classification reported for the displayed file,
///  after the caller drained and made one change.
fn classification(watcher: &mut ChangeWatcher, what: &str) -> Option<SourceChange> {
    let first = {
        let mut arrival = Arrival::begin(watcher, what);
        while !(arrival.record.source.is_some()) {
            arrival.again(watcher);
        }
        arrival.into_record()
    };
    // Later events of the same operation may still arrive; let them land before reading the latest.
    let later = quiet(watcher);
    if later.source.is_some() {
        return later.source;
    }
    return first.source;
}

/// Writes still open are unsettled;
///  closing,
///  replacing by rename,
///  and removing are settled.
#[test]
fn displayed_file_changes_are_settled_only_when_finished() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let file = root.join("view.txt");
    fs::write(&file, "start").expect("displayed file");
    watcher.watch_only(&set(&[&root]), Some(&file));
    settle(&mut watcher, &[&root]);
    // What: `Option<std::fs::File>` holds the open handle so the test decides when it closes.
    // Why: Truncation and writes before the close must not count as a finished write.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let handle: number | undefined;
    // ```
    let mut handle;
    quiet(&mut watcher);
    {
        let mut opened = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file)
            .expect("open for writing");
        opened.write_all(b"partial").expect("partial write");
        handle = Some(opened);
    }
    let open_write = classification(&mut watcher, "a truncating write");
    assert_eq!(
        open_write,
        Some(SourceChange::Unsettled),
        "a truncating write still open looked finished"
    );
    quiet(&mut watcher);
    {
        drop(handle.take());
    }
    let closed = classification(&mut watcher, "the close");
    assert_eq!(
        closed,
        Some(SourceChange::Settled),
        "closing the written file did not settle it"
    );
    quiet(&mut watcher);
    {
        let staged = root.join(".view.txt.tmp");
        fs::write(&staged, "replacement").expect("staged replacement");
        fs::rename(&staged, &file).expect("atomic replace");
    }
    let replaced = classification(&mut watcher, "an atomic replace");
    assert_eq!(
        replaced,
        Some(SourceChange::Settled),
        "an atomic replace did not settle"
    );
    quiet(&mut watcher);
    {
        fs::remove_file(&file).expect("remove displayed file");
    }
    let removed = classification(&mut watcher, "a removal");
    assert_eq!(
        removed,
        Some(SourceChange::Settled),
        "removing the displayed file did not settle"
    );
    quiet(&mut watcher);
    {
        fs::write(&file, "recreated").expect("recreate displayed file");
    }
    let recreated = classification(&mut watcher, "a recreation");
    assert_eq!(
        recreated,
        Some(SourceChange::Settled),
        "a recreated and closed file did not settle"
    );
}

/// Opening,
///  reading,
///  and listing what is watched reports nothing;
///  a real write still does.
#[test]
fn the_ides_own_reads_report_nothing() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    let file = folder.join("view.txt");
    fs::write(&file, "view").expect("displayed file");
    watcher.watch_only(&set(&[&root, &folder]), Some(&file));
    settle(&mut watcher, &[&root, &folder]);
    for _ in 0..20 {
        fs::read_to_string(&file).expect("source read");
        workspace.list(&folder).expect("folder listing");
        workspace.list(&root).expect("root listing");
    }
    let silent = quiet(&mut watcher);
    assert!(
        silent.directories.is_empty() && silent.source.is_none() && !silent.everything,
        "reads reported changes, which would reread forever: {silent:?}"
    );
    // Positive control: the same probe sees a real write to the same file.
    fs::write(&file, "changed").expect("external write");
    {
        let mut arrival = Arrival::begin(&mut watcher, "the control write");
        while !(arrival.record.source.is_some()) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}
