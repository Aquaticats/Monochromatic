//! Controls for the commit command's pure helpers.

use super::*;

/// The directory a `-C` chain reaches.
#[test]
fn the_effective_directory_follows_every_chdir_option() {
    let start: &Path = Path::new("/work");
    assert_eq!(effective_directory(start, &[]), PathBuf::from("/work"));
    assert_eq!(
        effective_directory(start, &[OsString::from("-C"), OsString::from("sub")]),
        PathBuf::from("/work/sub")
    );
    assert_eq!(
        effective_directory(
            start,
            &[
                OsString::from("-c"),
                OsString::from("user.name=x"),
                OsString::from("-C"),
                OsString::from("a"),
                OsString::from("-C"),
                OsString::from("b"),
            ]
        ),
        PathBuf::from("/work/a/b")
    );
    // An absolute path replaces what came before it.
    assert_eq!(
        effective_directory(start, &[OsString::from("-C"), OsString::from("/elsewhere")]),
        PathBuf::from("/elsewhere")
    );
    // A `-C` with no value changes nothing.
    assert_eq!(effective_directory(start, &[OsString::from("-C")]), PathBuf::from("/work"));
}

/// What:
///  A command that ended without landing states its reason only when it failed.
/// Why:
///  A command that made no commit because nothing needed doing returned `0` and nothing stopped it,
///  so a reason there would be one nobody can check,
///  and the record's "unidentified" is kept for a route that really reports nothing.
#[test]
fn an_ending_that_did_not_fail_names_no_reason() {
    let quiet: TransactionResult = TransactionResult { events: Vec::new(), exit_code: 0, landed_oid: None, uses_lfs: false };
    assert_eq!(stop_of_an_ending(&quiet), None);
    let blocked: TransactionResult = TransactionResult {
        events: vec![PolicyEvent::CoreFinding { core_id: "commit-only", code: "no-change", message: String::from("nothing left to commit") }],
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    };
    assert_eq!(stop_of_an_ending(&blocked), Some(CommitStopReason::core_finding("commit-only", "no-change")));
}
