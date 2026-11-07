//! What:
//!  Controls of how a folder use is recorded.
//! Why:
//!  The proof kept for a pair of folders is the smallest one,
//!  so a finding reads the same whatever order the uses were found in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the smallest proof wins in any order', () => expect(recordInEachOrder(proofs)).toEqual(smallest));
//! ```

use super::{FolderUses, Proof, record_use};
use std::path::PathBuf;

/// A proof on a line of one file.
fn proof_on(line: usize) -> Proof {
    return Proof {
        file: PathBuf::from("src/a/mod.rs"),
        line,
        text: String::from("crate::b::Thing"),
    };
}

/// Several proofs of the same use leave the smallest one,
///  in ascending,
///  descending and mixed order.
#[test]
fn layout_the_smallest_proof_of_a_folder_use_is_kept_in_any_order() {
    let from: PathBuf = PathBuf::from("src/a");
    let to: PathBuf = PathBuf::from("src/b");
    let ascending: Vec<usize> = (1..=3).collect::<Vec<usize>>();
    let mut descending: Vec<usize> = ascending.clone();
    descending.reverse();
    let mixed: Vec<usize> = vec![ascending[1], ascending[2], ascending[0]];
    for order in [ascending.clone(), descending, mixed] {
        let mut uses: FolderUses = FolderUses::new();
        for line in &order {
            record_use(&mut uses, from.clone(), to.clone(), proof_on(*line));
        }
        assert_eq!(uses[&from][&to], proof_on(ascending[0]), "{order:?}");
    }
}

/// A use of another pair is recorded beside the first and does not replace it.
#[test]
fn layout_a_second_target_is_recorded_beside_the_first() {
    let from: PathBuf = PathBuf::from("src/a");
    let mut uses: FolderUses = FolderUses::new();
    record_use(&mut uses, from.clone(), PathBuf::from("src/b"), proof_on(2));
    record_use(&mut uses, from.clone(), PathBuf::from("src/c"), proof_on(1));
    assert_eq!(uses[&from].len(), 2);
}
