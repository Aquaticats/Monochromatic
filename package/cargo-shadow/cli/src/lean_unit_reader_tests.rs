//! The tests of the reader of a profile directory's units:
//! a name in a build tree that cannot be read is reported and holds nothing,
//! and a pipe at one of those names never holds the pass that reads it.

use std::{fs, path::PathBuf};

use crate::lean_test_support::{make_pipe, make_profile, within};
use crate::lean_unit_reader::{ProfileRead, ProfileUnits, read_profile};
use crate::test_scratch::scratch;

/// The label of the one unit of these fixtures.
const LABEL: &str = "alpha-00000000000000aa";

/// What:
/// `fn units_of(read: ProfileRead) -> ProfileUnits` is the units of a read this test knows a layout of.
/// Why:
/// The three tests differ in which name is a pipe and assert on what the read found,
/// so the unwrapping is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unitsOf = (read: ProfileRead): ProfileUnits => { if (read.kind !== 'known') throw new Error(read.reason); return read.units; };
/// ```
fn units_of(read: ProfileRead) -> ProfileUnits {
    return match read {
        ProfileRead::Known(units) => units,
        ProfileRead::Unknown(reason) => panic!("the layout is one this reader knows: {reason}"),
    };
}

/// What:
/// `fn fingerprint_with_a_pipe(root: &Path) -> PathBuf` makes an old-layout unit whose fingerprint JSON is a pipe with no writer.
/// Why:
/// A build tree is a directory other commands write,
/// so a pipe at the name Cargo writes is the shape that holds a blocking open forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fingerprintWithAPipe(root: string): string
/// ```
fn fingerprint_with_a_pipe(root: &std::path::Path) -> PathBuf {
    let profile: PathBuf = make_profile(root, "debug");
    let fingerprint: PathBuf = profile.join(".fingerprint").join(LABEL);
    fs::create_dir_all(&fingerprint).expect("fingerprint directory");
    make_pipe(&fingerprint.join("lib-alpha.json"));
    return profile;
}

/// A pipe where Cargo writes a fingerprint's `rustc` field is no unit and holds nothing:
/// the read answers at once and reports the fingerprint it could not read,
/// so the sweep never plans a removal from a partial read of a unit.
#[test]
fn a_pipe_at_a_fingerprint_json_is_an_unreadable_unit_and_does_not_hold_the_read() {
    let root = scratch();
    let profile: PathBuf = fingerprint_with_a_pipe(root.path());
    let read: ProfileRead = within(move || { return read_profile(&profile); });
    let units: ProfileUnits = units_of(read);
    assert!(units.units.is_empty(), "a unit whose compiler cannot be read is never a unit: {units:?}");
    assert!(units.unreadable.contains(&String::from(LABEL)), "the fingerprint is named as unreadable: {units:?}");
}

/// A pipe at the name of a unit's last invocation is not the time of an invocation and holds nothing:
/// the time falls back to the fingerprint file beside it,
/// which is what the reader does for a fingerprint Cargo never wrote one for.
#[test]
fn a_pipe_at_the_invoked_timestamp_is_no_time_and_does_not_hold_the_read() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let fingerprint: PathBuf = profile.join(".fingerprint").join(LABEL);
    fs::create_dir_all(&fingerprint).expect("fingerprint directory");
    fs::write(fingerprint.join("lib-alpha.json"), "{\"rustc\":111}").expect("fingerprint json");
    make_pipe(&fingerprint.join("invoked.timestamp"));
    let read: ProfileRead = within(move || { return read_profile(&profile); });
    let units: ProfileUnits = units_of(read);
    assert_eq!(units.units.len(), 1, "{units:?}");
    assert_eq!(units.units[0].compiler, 111, "{units:?}");
}

/// A pipe in the new layout is the same read:
/// `build/<package>/<hash>/fingerprint/lib-<package>.json` with no writer answers at once.
#[test]
fn a_pipe_at_a_new_layout_fingerprint_json_does_not_hold_the_read() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let fingerprint: PathBuf = profile.join("build").join("alpha").join("00000000000000aa").join("fingerprint");
    fs::create_dir_all(&fingerprint).expect("new-layout fingerprint directory");
    make_pipe(&fingerprint.join("lib-alpha.json"));
    let read: ProfileRead = within(move || { return read_profile(&profile); });
    let units: ProfileUnits = units_of(read);
    assert!(units.units.is_empty(), "{units:?}");
    assert_eq!(units.unreadable, vec![String::from(LABEL)], "{units:?}");
}
