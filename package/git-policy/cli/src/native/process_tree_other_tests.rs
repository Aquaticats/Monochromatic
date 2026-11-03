//! Controls for the process-tree branches of platforms without `/proc`, which the Linux build does not compile.

use super::*;
use std::process::{Child, Command, Stdio};

/// A live child of this process is not reported: only Linux lists processes, so the tree is empty elsewhere.
#[test]
fn no_descendant_is_found_without_proc() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30").stdin(Stdio::null())).expect("sleep starts");
    let own: i32 = i32::try_from(std::process::id()).expect("a process number");
    assert_eq!(descendants_of(own), Vec::<TreeMember>::new());
    sleeper.kill().expect("sleep is killed");
    sleeper.wait().expect("sleep is collected");
}

/// A process that exists is still not "still running": nothing was found, so nothing counts as found.
#[test]
fn a_live_process_is_not_still_running_without_proc() {
    let own: TreeMember = TreeMember { pid: i32::try_from(std::process::id()).expect("a process number"), started: 1 };
    assert!(!still_running(&own));
}
