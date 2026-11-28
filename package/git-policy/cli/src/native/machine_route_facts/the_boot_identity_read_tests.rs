//! What:
//!  The controls of this boot's identifier:
//!  a kernel file that holds one line,
//!  a file that holds nothing,
//!  and a file that is not there.
//! Why:
//!  Every readiness record and every receipt is compared with this one string,
//!  so a terminator left in it would report this boot's own records as another
//!  boot's,
//!  and a boot nobody could name must answer the two checks an unknown rather
//!  than compare them with an empty string.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theLiveBoot({ places, unread })).toBe('4c4c…');
//! expect(unread.map((r) => r.check)).toEqual(['readiness', 'unresolved-requests']);
//! ```

/// The places a fixture names.
use super::super::the_route_places::TheRoutePlaces;
/// The read a control judges.
use super::the_live_boot;
/// The check a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The owned paths these controls use.
use std::path::PathBuf;

/// The identifier a control writes.
const AN_IDENTIFIER: &str = "4c4c4544-0042-3510-8051-b4c04f4e4d32";

/// One fixture with the boot's file at the text the control chose.
fn with_the_boot_file(
    name: &str,
    text: &str,
) -> TheRoutePlaces {
    let root: PathBuf = fixture(name);
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    if let Some(folder) = places.boot_identity.parent() {
        std::fs::create_dir_all(folder).expect("the boot's folder");
    }
    std::fs::write(places.boot_identity.as_path(), text.as_bytes()).expect("the boot's file");
    return places;
}

/// What:
///  The kernel's line answers its identifier without the terminator,
///  and no check answers an unknown.
/// Why:
///  The comparison is with the record's own text,
///  so one trailing byte would make every record of this boot another boot's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theLiveBoot({ places, unread })).toBe(AN_IDENTIFIER);
/// ```
#[test]
fn the_kernels_line_answers_its_identifier_without_the_terminator() {
    let places: TheRoutePlaces =
        with_the_boot_file("boot-identity", format!("{AN_IDENTIFIER}\n").as_str());
    let mut unread: Vec<CheckReport> = Vec::new();
    assert_eq!(the_live_boot(&places, &mut unread), AN_IDENTIFIER);
    assert!(unread.is_empty(), "{unread:?}");
}

/// What:
///  A file that holds no text and a file that is not there each answer an empty
///  identifier and the two checks an unknown naming the file.
/// Why:
///  An empty identifier is a value every record would be compared against,
///  so the checks must answer unknown instead of reporting another boot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theLiveBoot({ places, unread })).toBe('');
/// expect(unread.map((r) => r.check)).toEqual(['readiness', 'unresolved-requests']);
/// ```
#[test]
fn no_identifier_answers_the_two_checks_an_unknown() {
    let empty: TheRoutePlaces = with_the_boot_file("boot-identity-empty", "");
    let mut unread: Vec<CheckReport> = Vec::new();
    assert_eq!(the_live_boot(&empty, &mut unread), "");
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![Check::Readiness, Check::UnresolvedRequests]
    );
    assert!(
        unread[0]
            .line()
            .contains(empty.boot_identity.display().to_string().as_str()),
        "{}",
        unread[0].line()
    );
    let root: PathBuf = fixture("boot-identity-gone");
    let absent: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let mut missing: Vec<CheckReport> = Vec::new();
    assert_eq!(the_live_boot(&absent, &mut missing), "");
    assert_eq!(missing.len(), 2);
}
