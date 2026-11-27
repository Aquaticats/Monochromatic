//! What:
//!  The process numbers below one process,
//!  read from the kernel's per-thread child lists.
//! Why:
//!  A notice that looks many times must not pay a pass over `/proc` for each look.
//!  Measured on 2026-11-09 on this host
//!  (1,210 entries in `/proc`;
//!  the load average was 44 on 6 threads when the runs ended and 91 an hour before them):
//!  one pass reading every `/proc/<pid>/stat` took 17.6 ms at the median and 19.7 ms at most over 30 looks,
//!  and one walk of the child lists 19.6 microseconds at the median and 26.3 at most over 30 walks,
//!  about 900 times cheaper.
//!  These numbers prove no start time,
//!  so a caller that ends what it finds still reads the tree itself
//!  (`process_tree.rs`,
//!  `descendants_of`);
//!  this file answers only which processes are below one,
//!  which is what a notice needs on every look.
//!
//! Files beside this one,
//!  and why the tree is not one file:
//!  `process_tree.rs` holds the tree itself,
//!  read from `/proc` with the start time that proves each member,
//!  and re-exports this file's one question so no caller changes.
//!  `rust/max-lines` splits the tree by role,
//!  and the cheap look through the kernel's own child lists is the role a reader can hold alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const numbers = descendantNumbersFromThreadLists(root); // undefined without the kernel's lists
//! ```

/// What:
///  The process numbers below `root`,
/// read from the kernel's per-thread child lists.
/// Why:
///   A notice that looks many times must not pay a pass over `/proc` for each look.
/// Measured on 2026-11-09 on this host (1,210 entries in `/proc`;
/// the load average was 44 on 6 threads when the runs ended and 91 an hour before them):
/// one pass reading every `/proc/<pid>/stat` took 17.6 ms at the median and 19.7 ms at most over 30 looks,
/// and one walk of the child lists 19.6 microseconds at the median and 26.3 at most over 30 walks,
/// about 900 times cheaper.
///  The kill still reads all of `/proc` (`descendants_of`),
/// because that read names the start time that proves a process is still the one that was found.
/// Nothing is returned when the kernel has no child lists (`CONFIG_PROC_CHILDREN` off or a `/proc` that is not there),
/// so the caller can read `/proc` instead.
///  A child of any thread of `root` is below it:
/// the list of the thread that started it is the one that names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numbers = descendantNumbersFromThreadLists(root); // undefined without the kernel's lists
/// ```
#[cfg(target_os = "linux")]
pub fn descendant_numbers_from_thread_lists(root: i32) -> Option<Vec<i32>> {
    let mut frontier: Vec<i32> = child_numbers_of(root)?;
    let mut below: Vec<i32> = Vec::new();
    while let Some(pid) = frontier.pop() {
        below.push(pid);
        frontier.extend(child_numbers_of(pid).unwrap_or_default());
    }
    return Some(below);
}

/// How many bytes of one thread's list of children this reads.
/// Why:
///  The list is one number per child,
///  so its length is the number of children a process started,
///  which is the system's state and not this command's:
///  1 MiB holds about forty thousand children,
///  and a list past it is not used rather than read whole.
const CHILDREN_BYTES_AT_MOST: u64 = 1 << 20;

/// The process numbers of the children `pid` started,
/// from the list of every one of its threads,
/// and nothing when it has no child list at all.
#[cfg(target_os = "linux")]
fn child_numbers_of(pid: i32) -> Option<Vec<i32>> {
    let tasks = std::fs::read_dir(format!("/proc/{pid}/task")).ok()?;
    let mut found: Vec<i32> = Vec::new();
    let mut read_one: bool = false;
    for task in tasks.flatten() {
        let Ok(text) = crate::plain_file::read_plain_text_at(
            task.path().join("children").as_path(),
            CHILDREN_BYTES_AT_MOST,
        ) else {
            continue;
        };
        read_one = true;
        for word in text.split_ascii_whitespace() {
            if let Ok(member) = word.parse::<i32>() {
                found.push(member);
            }
        }
    }
    if !read_one {
        return None;
    }
    return Some(found);
}

/// Only Linux lists child processes;
/// elsewhere nothing is found and the caller reads `/proc`.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn descendant_numbers_from_thread_lists(_root: i32) -> Option<Vec<i32>> {
    return None;
}
