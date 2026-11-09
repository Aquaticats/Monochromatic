//! What:
//!  Controls for the chain record:
//!  the marker it keeps,
//!  the file beside it,
//!  and the lookup that finds the record a process inherited.
//! Why:
//!  A record is inherited through a descriptor table,
//! which a control cannot arrange for another process,
//! so the lookup is
//!  shown against a table it fabricates,
//! and the record itself against a descriptor it makes;
//! the binary control of the
//!  launcher that execs the wrapper shows the inheritance.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(readParts(record).marker).toBe(markerText); expect(inheritedDescriptorIn(fabricatedTable)).toBe(7);
//! ```
#![cfg(unix)]

/// The record under test.
use super::{MARKER_ROOM, RECORD_NAME, create_descriptor, for_this_start_with, inherited_descriptor_in, read_parts, write_part};
use std::path::{Path, PathBuf};

/// A directory under the build directory for a fixture.
fn fixture(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture(name);
    std::fs::create_dir_all(&root).expect("fixture root");
    return root;
}

/// What:
///  A record holds the marker written into it and the file beside it,
///  and an empty one holds neither.
/// Why:
///   The guard reads the marker exactly as it reads one from an environment,
/// and the message names the file that
///  marker's start chose;
/// a record nothing wrote is what a chain's first start finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fd = createChainRecord(); expect(readParts(fd)).toEqual([undefined, undefined]); writePart(fd, 0, marker); expect(readParts(fd)[0]).toBe(marker);
/// ```
#[test]
fn a_record_keeps_the_marker_and_the_file_written_into_it() {
    let descriptor: i32 = create_descriptor().expect("a record is made");
    assert_eq!(read_parts(descriptor), (None, None), "a record nothing wrote");
    let marker: &str = "4242\u{1f}linux:9\u{1f}child\u{1f}3";
    assert!(write_part(descriptor, 0, MARKER_ROOM, std::ffi::OsStr::new(marker)));
    assert_eq!(read_parts(descriptor).0.as_deref(), Some(marker), "the marker the record keeps");
    assert_eq!(read_parts(descriptor).1, None, "and nothing beside it, yet");
    // The file is written into its own room by a later moment of the same start.
    assert!(write_part(descriptor, i32::try_from(MARKER_ROOM).expect("room"), super::CHOSEN_ROOM, std::ffi::OsStr::new("/opt/launchers/git")));
    assert_eq!(read_parts(descriptor).0.as_deref(), Some(marker), "the marker is undisturbed");
    assert_eq!(read_parts(descriptor).1.as_deref(), Some("/opt/launchers/git"), "the file that start chose");
    // A shorter text ends at its own NUL, so the bytes of the longer one behind it are read as nothing.
    assert!(write_part(descriptor, i32::try_from(MARKER_ROOM).expect("room"), super::CHOSEN_ROOM, std::ffi::OsStr::new("/bin/git")));
    assert_eq!(read_parts(descriptor).1.as_deref(), Some("/bin/git"));
}

/// What:
///  A record this start makes is empty,
/// and one handed in is read as it stands.
/// Why:
///   The start above is what the record says,
/// and a start that makes its own record has none above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(forThisStartWith(() => undefined).marker()).toBeUndefined();
/// ```
#[test]
fn a_made_record_names_no_start_above() {
    let made: super::ChainRecord = for_this_start_with(|| { return None }).expect("a record is made");
    assert_eq!(made.marker(), None);
    assert_eq!(made.chosen(), None);
    let inherited: i32 = create_descriptor().expect("a record is made");
    assert!(write_part(inherited, 0, MARKER_ROOM, std::ffi::OsStr::new("7\u{1f}linux:1\u{1f}query\u{1f}1")));
    let read: super::ChainRecord = for_this_start_with(|| { return Some(inherited) }).expect("the inherited record is used");
    assert_eq!(read.marker(), Some("7\u{1f}linux:1\u{1f}query\u{1f}1"), "what the start above wrote");
}

/// What:
///  The lookup finds the record by the name the memory file carries,
///  whatever number a process gave it,
///  and nothing in a table
///  that holds none.
/// Why:
///   The number belongs to the process that made it,
/// so a table of descriptors is read and the record is found by what
///  it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(`/memfd:${RECORD_NAME}`, `${table}/7`); expect(inheritedDescriptorIn(table)).toBe(7);
/// ```
#[test]
fn the_lookup_finds_the_record_by_the_name_it_carries() {
    let root: PathBuf = fixture("chain-record-lookup");
    let table: PathBuf = root.join("fd");
    std::fs::create_dir_all(&table).expect("table");
    std::os::unix::fs::symlink("/dev/null", table.join("3")).expect("a descriptor of the person's");
    assert_eq!(inherited_descriptor_in(table.as_path()), None, "no record in this table");
    std::os::unix::fs::symlink(format!("/memfd:{RECORD_NAME} (deleted)").as_str(), table.join("7")).expect("the record");
    assert_eq!(inherited_descriptor_in(table.as_path()), Some(7), "the record, at the number the process gave it");
    assert_eq!(inherited_descriptor_in(root.join("absent").as_path()), None, "a table that cannot be read");
    crate::test_support::remove(root.as_path());
}

/// What:
///  The name the record's memory file carries is the one a descriptor table shows,
/// so a descriptor of another program is never
///  taken for a record.
/// Why:
///   A lookup that matched a prefix would take a file the person's own program opened for a chain record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(`/memfd:${RECORD_NAME}`).toBe('/memfd:monochromatic-git-policy-cli-chain-start');
/// ```
#[test]
fn the_record_name_is_the_one_a_descriptor_table_shows() {
    assert_eq!(format!("/memfd:{RECORD_NAME} (deleted)"), "/memfd:monochromatic-git-policy-cli-chain-start (deleted)");
    assert_eq!(inherited_descriptor_in(Path::new("/nonexistent-descriptor-table")), None);
}
