//! Controls for the by-hand steps every retained-transaction message ends with.

use super::*;
use crate::transaction_journal_encode::tests::sample_preparing;

/// With the capture record the steps name the real index lock, the shadow repository and the directory;
///  without it they name the directory and say where an index lock may be.
#[test]
fn the_steps_name_what_to_remove() {
    let directory: &Path = Path::new("/repo/.git/cli-git-transactions/0b6c2c1e");
    let preparing: PreparingRecord = sample_preparing();
    let with_record: String = finish_by_hand_steps(directory, Some(&preparing));
    assert!(with_record.contains(&format!("{}.lock (if it exists)", preparing.real_index_path)), "{with_record}");
    assert!(with_record.contains(&format!("{} (if it exists)", preparing.shadow_path)), "{with_record}");
    assert!(with_record.contains("and /repo/.git/cli-git-transactions/0b6c2c1e, run `git status`"), "{with_record}");
    assert!(with_record.contains("run `git reset`"), "{with_record}");
    assert!(with_record.ends_with("Every guarded command stops until /repo/.git/cli-git-transactions/0b6c2c1e is removed."), "{with_record}");
    let without_record: String = finish_by_hand_steps(directory, None);
    assert!(without_record.contains("any index.lock file in the repository's Git directory"), "{without_record}");
    assert!(!without_record.contains("(if it exists)"), "{without_record}");
}
