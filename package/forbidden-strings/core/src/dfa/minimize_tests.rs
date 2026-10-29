// What:  unit tests for DFA minimization helpers.
// Why:     This file groups the minimize test cases so behavior changes fail near the code path
//          they protect.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("minimize", () => {
//   // test cases below
// });
// ```

use super::distinct_count;

#[test]
fn distinct_count_counts_unique_colors() {
    assert_eq!(distinct_count(&[]), 0);
    assert_eq!(distinct_count(&[5, 5, 5]), 1);
    assert_eq!(distinct_count(&[0, 1, 2, 0, 1]), 3);
    assert_eq!(distinct_count(&[3, 1, 4, 1, 5, 9, 2, 6]), 7);
}
