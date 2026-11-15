//! Controls for the removal of old finished records beside the loop:
//!  it removes the records past the count the journal keeps and says how many,
//!  and it leaves the newest ones where a client may still read them.

use super::*;
use std::path::Path;
use std::time::Duration;

/// A journal holding `count` finished batches,
/// each a record of its own and none of them carrying a reading of the clocks,
/// written two hours ago:
/// a removal takes a record only once no client of it may still read it,
/// which a record written this instant never is.
fn journal_with(name: &str, count: usize) -> PathBuf {
    let journal: PathBuf = crate::test_support::fixture(name).join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now().checked_sub(Duration::from_secs(2 * 60 * 60)).expect("two hours before now");
    for seq in 1..=count {
        let record: PathBuf = journal.join(format!("{seq}.done"));
        std::fs::write(record.as_path(), format!("seq={seq}\nold={}\nnew={}\nids=request-{seq}\n", "0".repeat(40), "1".repeat(40))).expect("a finished record");
        let held: std::fs::File = std::fs::File::options().write(true).open(record.as_path()).expect("the finished record");
        held.set_times(std::fs::FileTimes::new().set_modified(long_ago)).expect("the record's own time");
    }
    return journal;
}

/// How many finished records the journal holds.
fn finished(journal: &Path) -> usize {
    return std::fs::read_dir(journal).expect("the journal").flatten().filter(|entry| { return entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("done"); }).count();
}

/// What:
///  The removal where the loop stands takes the records past the count the journal keeps, and leaves the newest.
/// Why:
///  A client whose answer was lost asks again and is answered from the newest records,
/// so a removal that took them would make it land twice.
#[test]
fn the_removal_leaves_the_newest_records_and_takes_the_rest() {
    let journal: PathBuf = journal_with("prune-old-records", DONE_RETAINED + 3);
    let board: Arc<LandingBoard> = LandingBoard::new(journal.clone(), Duration::from_secs(60));
    prune_old_records_where_the_loop_stands(journal.clone(), board);
    assert_eq!(finished(journal.as_path()), DONE_RETAINED, "the newest records stay and the rest go");
}
