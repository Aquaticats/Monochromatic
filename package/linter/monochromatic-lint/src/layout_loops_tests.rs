//! What:
//!  Controls of loop finding.
//! Why:
//!  Both layout rules report exactly these loops.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a ring is one loop', () => expect(findLoops([[1], [2], [0]])).toEqual([[0, 1, 2]]));
//! ```

use super::find_loops;

/// A chain longer than any recursion would survive,
/// so the search provably keeps its own stack.
const DEEP_CHAIN: usize = 300_000;

/// No edges,
/// a chain and a self-edge have no loop.
#[test]
fn chains_and_self_edges_are_not_loops() {
    assert!(find_loops(&[]).is_empty());
    assert!(find_loops(&[vec![1], vec![2], vec![]]).is_empty());
    assert!(find_loops(&[vec![0]]).is_empty());
}

/// A ring is one loop with every member,
/// whatever order the edges are listed in.
#[test]
fn a_ring_is_one_loop() {
    assert_eq!(find_loops(&[vec![1], vec![2], vec![0]]), [[0, 1, 2]]);
    assert_eq!(find_loops(&[vec![2], vec![0], vec![1]]), [[0, 1, 2]]);
}

/// A tail leading into a loop and a branch leading out of it are not members.
#[test]
fn tails_and_branches_are_not_members() {
    // 0 -> 1 -> 2 -> 1, 2 -> 3
    assert_eq!(
        find_loops(&[vec![1], vec![2], vec![1, 3], vec![]]),
        [[1, 2]]
    );
}

/// Separate loops are reported separately,
/// ordered by their first member,
/// and two rings joined by one edge stay two loops.
#[test]
fn separate_loops_stay_separate() {
    let graph: Vec<Vec<usize>> = vec![vec![1], vec![0, 2], vec![3], vec![2]];
    assert_eq!(find_loops(graph.as_slice()), [vec![0, 1], vec![2, 3]]);
}

/// A very long chain and a very long ring finish without exhausting the call stack.
#[test]
fn very_deep_graphs_finish() {
    let mut chain: Vec<Vec<usize>> = Vec::<Vec<usize>>::new();
    for index in 0..DEEP_CHAIN {
        chain.push(vec![index + 1]);
    }
    chain.push(vec![0]);
    let loops: Vec<Vec<usize>> = find_loops(chain.as_slice());
    assert_eq!(loops.len(), 1);
    assert_eq!(loops[0].len(), DEEP_CHAIN + 1);
}
