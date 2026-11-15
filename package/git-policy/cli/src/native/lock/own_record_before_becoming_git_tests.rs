//! What:
//!  A source control of the step between `fork` and `exec`:
//!  the functions a child runs before it becomes Git hold no allocating construct.
//! Why:
//!  A child forked from a process with threads inherits the allocator's lock,
//!  and another thread may have held it at the fork:
//!  nothing in the child can release it,
//!  so an allocating error branch there waits for good and holds the parent inside `spawn`
//!  (a review of `58cd4d9f7c`,
//! finding 5).
//!  The rules of `source_guard::blocking_code_off_the_runtime` match a line inside a place they can name
//!  (everywhere,
//! a function that builds a `Command`,
//! a task),
//!  and the child's functions live in this file,
//!  in no such place:
//!  so the control of that shape is this one,
//!  reading the file and asking it of every line a child runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const childSafe = sourceBetween('pub fn write(&self)', 'pub fn own_line_for');
//! expect(childSafe.filter((line) => ALLOCATING.some((needle) => line.includes(needle)))).toEqual([]);
//! ```

/// The path of the file these lines live in, from the crate's own folder.
const SOURCE: &str = "src/native/lock/own_record_before_becoming_git.rs";

/// The first line of the region a child runs, in the file's own spelling.
const CHILD_REGION_STARTS: &str = "pub fn write(&self)";

/// The parent's own preparation, which runs in the parent and may allocate:
///  it lies before the child's region and is not part of it.
const PARENT_PREPARATION: &str = "pub fn prepare_own_line(";

/// What a child may not do between `fork` and `exec`:
///  put a byte on the heap.
const ALLOCATING: &[&str] = &[
    "format!(",
    "println!(",
    "eprintln!(",
    "String::from(",
    "String::new(",
    ".to_owned()",
    ".to_string()",
    "Vec::new(",
    "vec![",
    "Box::new(",
    "Error::other(",
    "Error::new(",
    ".collect::",
];

/// What:
///  Every line a child runs between `fork` and `exec` holds no allocating construct.
/// Why:
///   The allocator's lock can be held by a parent thread at the fork,
///  and a child that allocates then waits for a thread that does not exist in it.
///  The region is read from the file itself,
///  so a line added there later is judged by this control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(allocatingLinesIn(childRegion(SOURCE))).toEqual([])
/// ```
#[test]
fn the_childs_side_of_a_start_allocates_nothing() {
    let path: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE);
    let text: String = std::fs::read_to_string(path.as_path()).expect("the child-side source is readable");
    let started: usize = text.find(CHILD_REGION_STARTS).expect("the child's write function is in that file");
    let parent: usize = text.find(PARENT_PREPARATION).expect("the parent's own preparation is in that file");
    assert!(parent < started, "the parent's preparation comes before the child's region, so none of it is judged here");
    let region: &str = text.get(started..).expect("the region is text");
    let found: Vec<&str> = region
        .lines()
        .filter(|line| { return ALLOCATING.iter().any(|needle| { return line.contains(needle); }); })
        .collect();
    assert!(found.is_empty(), "a child between fork and exec allocates nothing, and these lines do: {found:?}");
}

/// What:
///  The control above reads a region that holds code at all,
///  so an emptied or moved child region cannot make it pass by holding nothing.
/// Why:
///   A scan of no lines finds no allocating line,
///   which would be a pass that proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(childRegion(SOURCE).filter((line) => line.includes('fn '))).not.toEqual([])
/// ```
#[test]
fn the_childs_region_holds_the_functions_a_child_runs() {
    let path: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE);
    let text: String = std::fs::read_to_string(path.as_path()).expect("the child-side source is readable");
    let started: usize = text.find(CHILD_REGION_STARTS).expect("the child's write function is in that file");
    let region: &str = text.get(started..).expect("the region is text");
    for named in ["fn write(", "fn read_bounded(", "fn write_all(", "fn close(", "fn errno(", "fn is_a_plain_file("] {
        assert!(region.contains(named), "the child's region names {named}");
    }
}
