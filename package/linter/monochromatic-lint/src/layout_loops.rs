//! What:
//!  Find the loops in a directed graph:
//!  the groups of two or more nodes that can each reach every other.
//! Why:
//!  "Two folders never use each other, directly or through others" and "files never use each other in a loop"
//!  are both the same question on different graphs.
//!  The search is Tarjan's strongly connected components with an explicit work stack,
//!  because the graph comes from the repository and its depth is not bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findLoops(adjacency: number[][]): number[][];
//! ```

/// What:
///  One step of the explicit work stack:
///  a node and how many of its neighbours are already visited.
/// Why:
///  It replaces the call stack of the textbook recursion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Frame = { node: number; next: number };
/// ```
struct Frame {
    /// The node being expanded.
    node: usize,
    /// Index of the next neighbour to visit.
    next: usize,
}

/// What:
///  The loops of a graph,
///  each as its sorted node list,
///  ordered by their first node.
/// Why:
///  A node on its own is not a loop,
///  and an edge from a node to itself is not a loop either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findLoops(adjacency: number[][]): number[][];
/// ```
pub fn find_loops(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let count: usize = adjacency.len();
    let unvisited: usize = usize::MAX;
    let mut order: Vec<usize> = vec![unvisited; count];
    let mut low: Vec<usize> = vec![0; count];
    let mut on_stack: Vec<bool> = vec![false; count];
    let mut stack: Vec<usize> = Vec::<usize>::new();
    let mut counter: usize = 0;
    let mut loops: Vec<Vec<usize>> = Vec::<Vec<usize>>::new();
    for root in 0..count {
        if order[root] != unvisited {
            continue;
        }
        let mut work: Vec<Frame> = vec![Frame { node: root, next: 0 }];
        order[root] = counter;
        low[root] = counter;
        counter += 1;
        stack.push(root);
        on_stack[root] = true;
        while let Some(frame) = work.last_mut() {
            let node: usize = frame.node;
            if frame.next < adjacency[node].len() {
                let neighbour: usize = adjacency[node][frame.next];
                frame.next += 1;
                if order[neighbour] == unvisited {
                    order[neighbour] = counter;
                    low[neighbour] = counter;
                    counter += 1;
                    stack.push(neighbour);
                    on_stack[neighbour] = true;
                    work.push(Frame {
                        node: neighbour,
                        next: 0,
                    });
                } else if on_stack[neighbour] {
                    low[node] = low[node].min(order[neighbour]);
                }
                continue;
            }
            // All neighbours are done: pop the frame and pass the low-link to the parent.
            work.pop();
            if let Some(parent) = work.last() {
                low[parent.node] = low[parent.node].min(low[node]);
            }
            if low[node] != order[node] {
                continue;
            }
            let mut component: Vec<usize> = Vec::<usize>::new();
            while let Some(member) = stack.pop() {
                on_stack[member] = false;
                component.push(member);
                if member == node {
                    break;
                }
            }
            if component.len() > 1 {
                component.sort_unstable();
                loops.push(component);
            }
        }
    }
    loops.sort();
    return loops;
}

/// Controls of loop finding stay outside release artifacts.
#[cfg(test)]
#[path = "layout_loops_tests.rs"]
mod tests;
