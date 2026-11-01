// What:     Tests of `change_set.rs`, pulled in by `change_set.rs`.
// Why:      The change set is what stands between a storm of watcher events and the refresh: it must ignore reads, treat
//           lost events as "everything", keep what an event named, and stay bounded however many events arrive.

use super::*;
use notify_debouncer_mini::notify::event::{AccessKind, AccessMode, CreateKind, DataChange, Flag, MetadataKind, RemoveKind, RenameMode};
use std::time::Instant;

/// What: `ok` wraps an event the way the watcher callback receives it.
/// Why: `Change::of_event` takes the callback's result.
fn ok(event: Event) -> notify_debouncer_mini::notify::Result<Event> {
    return Ok(event);
}

/// What: `paths_of` unwraps a `Paths` change into its sorted entries.
/// Why: assertions compare the entries.
fn paths_of(change: Option<Change>) -> Vec<(PathBuf, bool)> {
    match change {
        Some(Change::Paths(set)) => return set.sorted(),
        other => panic!("expected paths, got {other:?}"),
    }
}

#[test]
fn a_read_is_not_a_change_but_a_close_after_writing_is_also_not_one() {
    let read = Event::new(EventKind::Access(AccessKind::Read)).add_path(PathBuf::from("/lib/a.flac"));
    let open = Event::new(EventKind::Access(AccessKind::Open(AccessMode::Any))).add_path(PathBuf::from("/lib"));
    let close = Event::new(EventKind::Access(AccessKind::Close(AccessMode::Write))).add_path(PathBuf::from("/lib/a.flac"));
    assert_eq!(Change::of_event(&ok(read)), None);
    assert_eq!(Change::of_event(&ok(open)), None);
    assert_eq!(Change::of_event(&ok(close)), None, "the data events came before the close");
}

#[test]
fn lost_events_and_errors_mean_everything() {
    let overflow = Event::new(EventKind::Other).set_flag(Flag::Rescan);
    assert_eq!(Change::of_event(&ok(overflow)), Some(Change::Everything));
    let failure: notify_debouncer_mini::notify::Result<Event> = Err(notify_debouncer_mini::notify::Error::generic("the kernel queue overflowed"));
    assert_eq!(Change::of_event(&failure), Some(Change::Everything));
    let nameless = Event::new(EventKind::Other);
    assert_eq!(Change::of_event(&ok(nameless)), Some(Change::Everything), "an event that names nothing cannot be refreshed by path");
}

#[test]
fn an_event_names_its_paths_and_says_whether_an_entry_appeared_or_went() {
    let written = Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any))).add_path(PathBuf::from("/lib/a.flac"));
    assert_eq!(paths_of(Change::of_event(&ok(written))), vec![(PathBuf::from("/lib/a.flac"), false)]);
    let touched = Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any))).add_path(PathBuf::from("/lib/a.flac"));
    assert_eq!(paths_of(Change::of_event(&ok(touched))), vec![(PathBuf::from("/lib/a.flac"), false)]);
    let created = Event::new(EventKind::Create(CreateKind::Folder)).add_path(PathBuf::from("/lib/new"));
    assert_eq!(paths_of(Change::of_event(&ok(created))), vec![(PathBuf::from("/lib/new"), true)]);
    let removed = Event::new(EventKind::Remove(RemoveKind::File)).add_path(PathBuf::from("/lib/gone.flac"));
    assert_eq!(paths_of(Change::of_event(&ok(removed))), vec![(PathBuf::from("/lib/gone.flac"), true)]);
    let renamed = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both))).add_path(PathBuf::from("/lib/old")).add_path(PathBuf::from("/lib/new"));
    assert_eq!(paths_of(Change::of_event(&ok(renamed))), vec![(PathBuf::from("/lib/new"), true), (PathBuf::from("/lib/old"), true)]);
}

// What:     An unspecific "modified" event is a plain write on Windows and a structural change elsewhere.
// Why:      The notify Windows backend reports every FILE_ACTION_MODIFIED as `Modify(Any)`; read as structural, one rewritten
//           file made the planner read its whole folder. Elsewhere that kind says nothing about what changed, so it stays
//           structural.
#[test]
fn an_unspecific_modify_is_a_plain_write_only_where_the_platform_reports_writes_that_way() {
    let modified = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("/lib/a.flac"));
    assert_eq!(paths_of(Change::of_event(&ok(modified))), vec![(PathBuf::from("/lib/a.flac"), !cfg!(windows))]);
    let other = Event::new(EventKind::Any).add_path(PathBuf::from("/lib/a.flac"));
    assert_eq!(paths_of(Change::of_event(&ok(other))), vec![(PathBuf::from("/lib/a.flac"), true)], "an event of unknown kind stays structural");
}

// What:     A folder reported as modified is no change on Windows, because reading a folder makes Windows report it.
// Why:      Each refresh reads every folder; without this rule each one would request the next without end (measured on a
//           GitHub Windows runner: `read_dir` of a watched folder alone produced a `Modify(Any)` event for that folder).
#[test]
fn a_folder_reported_as_modified_is_no_change_only_on_windows() {
    let folder = crate::test_scratch::scratch_root();
    let modified = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(folder.clone());
    if cfg!(windows) {
        assert_eq!(Change::of_event(&ok(modified)), None, "a folder only touched names nothing");
    } else {
        assert_eq!(paths_of(Change::of_event(&ok(modified))), vec![(folder.clone(), true)]);
    }
    let removed = Event::new(EventKind::Remove(RemoveKind::Folder)).add_path(folder.clone());
    assert_eq!(paths_of(Change::of_event(&ok(removed))), vec![(folder, true)], "a folder that went is still named");
}

#[test]
fn a_path_named_twice_is_one_path_and_keeps_the_stronger_meaning() {
    let mut set = DirtyPaths::default();
    set.insert(PathBuf::from("/lib/a.flac"), false);
    set.insert(PathBuf::from("/lib/a.flac"), true);
    set.insert(PathBuf::from("/lib/a.flac"), false);
    assert_eq!(set.len(), 1);
    assert_eq!(set.sorted(), vec![(PathBuf::from("/lib/a.flac"), true)]);
}

#[test]
fn a_storm_of_a_hundred_thousand_events_on_few_paths_stays_a_small_set() {
    let started = Instant::now();
    let mut gathered = Change::Paths(DirtyPaths::default());
    for number in 0..100_000_usize {
        let event = Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any))).add_path(PathBuf::from(format!("/lib/album/{}.flac", number % 100)));
        gathered = gathered.merge(Change::of_event(&ok(event)).expect("a write is a change"));
    }
    match gathered {
        Change::Paths(set) => assert_eq!(set.len(), 100, "duplicates collapse"),
        Change::Everything => panic!("a hundred paths are not everything"),
    }
    println!("REFRESH storm: 100,000 events merged in {:.0} ms", started.elapsed().as_secs_f64() * 1000.0);
}

#[test]
fn a_storm_over_the_cap_becomes_everything_and_stays_everything() {
    let mut gathered = Change::Paths(DirtyPaths::default());
    for number in 0..=MAX_DIRTY_PATHS {
        let event = Event::new(EventKind::Create(CreateKind::File)).add_path(PathBuf::from(format!("/lib/{number}.flac")));
        gathered = gathered.merge(Change::of_event(&ok(event)).expect("a create is a change"));
    }
    assert_eq!(gathered, Change::Everything);
    let after = gathered.merge(Change::Paths(DirtyPaths::single(PathBuf::from("/lib/x.flac"), false)));
    assert_eq!(after, Change::Everything, "everything absorbs paths");
}

#[test]
fn exactly_the_cap_is_still_a_set() {
    let mut gathered = Change::Paths(DirtyPaths::default());
    for number in 0..MAX_DIRTY_PATHS {
        gathered = gathered.merge(Change::Paths(DirtyPaths::single(PathBuf::from(format!("/lib/{number}.flac")), true)));
    }
    assert!(matches!(gathered, Change::Paths(ref set) if set.len() == MAX_DIRTY_PATHS));
}

#[test]
fn an_overflow_flag_means_everything_even_when_the_event_also_names_a_path() {
    // Some backends attach the watched root to the overflow notice; the paths must not be taken for the whole change.
    let overflow = Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any))).add_path(PathBuf::from("/lib")).set_flag(Flag::Rescan);
    assert_eq!(Change::of_event(&ok(overflow)), Some(Change::Everything));
}
