//! Controls of the claim on one address's takeover:
//! one process holds it and another is told so,
//! a claim file whose holder is gone blocks nobody and is taken over,
//! the file exists only while a takeover does,
//! and the claim is never taken by waiting.

use super::*;

/// A folder of this test's own, short enough for a socket address.
fn folder(name: &str) -> PathBuf {
    return crate::test_support::short_directory_for_sockets(format!("address-claim-{name}").as_str());
}

/// The address whose takeover is claimed here, in a folder of this test's own.
fn an_address(root: &Path) -> PathBuf {
    return root.join("3f9a.sock");
}

/// What:
///  One process holds the claim on an address and a second is told that another process has it,
///  and the claim is free again once the holder is done,
///  with no claim file left behind.
/// Why:
///  Two servers that both probe the same stale socket both unlink and bind unless the takeover is one process's at a
///  time, and the second unlink removes the first's live socket (the review of `76a08c9fac`, finding 3).
///  A claim file that stayed was a file the wrapper left in a runtime directory it served for good (the review of
///  `55f236a0a8`, finding 3): the claim's own holder removes it as its takeover ends.
#[test]
fn one_process_holds_the_claim_and_a_second_is_told_another_process_has_it() {
    let root: PathBuf = folder("held");
    let address: PathBuf = an_address(root.as_path());
    let first: AddressTakeoverClaim = match claim(address.as_path()).expect("the claim is made") {
        Claim::Held(held) => held,
        Claim::AnotherProcessTakesOver => panic!("nothing held this claim, so the first process takes it"),
    };
    assert_eq!(path(address.as_path()), root.join("3f9a.sock.taking-over"), "the file is named for the address it claims");
    assert!(path(address.as_path()).exists(), "the claim is on a file beside the address while it is held");
    assert!(matches!(claim(address.as_path()).expect("the claim is asked for"), Claim::AnotherProcessTakesOver), "the second process is told the claim is another's");
    drop(first);
    assert!(!path(address.as_path()).exists(), "the claim file goes with the takeover that made it: {}", path(address.as_path()).display());
    assert!(matches!(claim(address.as_path()).expect("the claim is asked for again"), Claim::Held(_)), "and the claim is free once its holder is done");
}

/// What:
///  A claim file whose holder is gone blocks nothing and is taken over,
///  and the takeover that took it over removes it.
/// Why:
///  A service killed inside its takeover leaves the file; a claim that had to be reasoned about (a text, an owner, a
///  birth identity) would be one more thing to get wrong, and the kernel's own claim has no stale state at all.
#[test]
fn a_claim_file_whose_holder_is_gone_blocks_nothing_and_is_taken_over() {
    let root: PathBuf = folder("stale");
    let address: PathBuf = an_address(root.as_path());
    std::fs::write(path(address.as_path()), b"").expect("a claim file a killed holder left");
    let Claim::Held(taken) = claim(address.as_path()).expect("the claim is made") else {
        panic!("a file nobody holds is no claim");
    };
    drop(taken);
    assert!(!path(address.as_path()).exists(), "the takeover that took the file over removes it");
}

/// What:
///  A claim file that is another claim's is never removed by this one.
/// Why:
///  Two claims on one address is the defect the file exists for; a path that names another file is another claim's, and
///  this one removes nothing.
#[test]
fn a_claim_file_that_became_another_claims_is_not_removed() {
    let root: PathBuf = folder("another");
    let address: PathBuf = an_address(root.as_path());
    let Claim::Held(taken) = claim(address.as_path()).expect("the claim is made") else { panic!("nothing held this claim") };
    // What a second command's file at the same path looks like once this one's was unlinked by something else.
    let theirs: PathBuf = path(address.as_path());
    std::fs::remove_file(theirs.as_path()).expect("this claim's file goes");
    std::fs::write(theirs.as_path(), b"").expect("another command's claim file");
    drop(taken);
    assert!(theirs.exists(), "the other claim's file is left where it is");
}

/// What:
///  A claim is asked for and never waited for:
///  a process that finds it held answers at once.
/// Why:
///  A command may not wait on a lock. The holder is taking the address over in three system calls, and this process's
///  caller has its own wait for the service's greeting.
#[test]
fn a_claim_that_is_held_is_answered_at_once() {
    let root: PathBuf = folder("no-wait");
    let address: PathBuf = an_address(root.as_path());
    let _held: AddressTakeoverClaim = match claim(address.as_path()).expect("the claim is made") {
        Claim::Held(held) => held,
        Claim::AnotherProcessTakesOver => panic!("nothing held this claim"),
    };
    let began: std::time::Instant = std::time::Instant::now();
    let second: Claim = claim(address.as_path()).expect("the claim is asked for");
    let took: std::time::Duration = began.elapsed();
    assert!(matches!(second, Claim::AnotherProcessTakesOver), "another process's claim is an answer");
    assert!(took < std::time::Duration::from_millis(50), "it is answered without waiting: {took:?}");
}

/// What:
///  A claim of an address whose folder is not there is an error, and never a claim taken on something else.
/// Why:
///  The caller then leaves the address as it is: a takeover that could not be claimed is no takeover.
#[test]
fn a_claim_in_a_folder_that_is_not_there_is_an_error() {
    let root: PathBuf = folder("missing");
    let address: PathBuf = root.join("not-there").join("3f9a.sock");
    assert!(claim(address.as_path()).is_err(), "no claim file can be made beside an address whose folder is not there");
}
