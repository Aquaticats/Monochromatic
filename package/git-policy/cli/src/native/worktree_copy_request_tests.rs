//! Tests of the durable copy request: its text, its order, what a damaged file does, and what a killed writer leaves.

use super::*;
use crate::test_support::{fixture, remove};

fn sample(id: &str) -> CopyRequest {
    return CopyRequest {
        id: id.to_string(),
        admin_id: String::from("topic"),
        registration: String::from("0f0e-token"),
        destination_root: PathBuf::from("/worktrees/topic"),
        source_root: PathBuf::from("/repository"),
        selection: CopySelection::Ignored,
        tries: 0,
    };
}

/// A request survives its own text, with either selection, and the text has one line with sorted keys.
#[test]
fn a_request_round_trips_through_its_text() {
    let ignored: CopyRequest = sample("0001-a");
    let named: CopyRequest = CopyRequest {
        selection: CopySelection::Paths(vec![String::from(".cache/rules.txt"), String::from("a \"quoted\" name")]),
        tries: 2,
        ..sample("0002-b")
    };
    for request in [ignored, named] {
        let text: String = request.encode();
        assert_eq!(text.lines().count(), 1);
        assert!(text.starts_with("{\"adminId\":"), "{text}");
        assert_eq!(CopyRequest::decode(text.as_str()).expect("round trip"), request);
    }
}

/// Every field is checked on read: a name or path that could reach outside its place is refused.
#[test]
fn a_request_with_an_unsafe_field_is_refused() {
    let good: String = sample("0001-a").encode();
    for (from, to) in [
        ("\"id\":\"0001-a\"", "\"id\":\"../escape\""),
        ("\"adminId\":\"topic\"", "\"adminId\":\"a/b\""),
        ("\"adminId\":\"topic\"", "\"adminId\":\"..\""),
        ("\"registration\":\"0f0e-token\"", "\"registration\":\"\""),
        ("\"/worktrees/topic\"", "\"worktrees/topic\""),
        ("\"/repository\"", "\"relative\""),
        ("\"version\":1", "\"version\":2"),
        ("{\"kind\":\"ignored\"}", "{\"kind\":\"paths\",\"paths\":[\"../outside\"]}"),
        ("{\"kind\":\"ignored\"}", "{\"kind\":\"everything\"}"),
    ] {
        assert!(good.contains(from), "{from}");
        assert!(CopyRequest::decode(good.replace(from, to).as_str()).is_err(), "{to} was accepted");
    }
    assert!(CopyRequest::decode("not json").is_err());
}

/// Requests are read in the order they were written, and the stage container is named after the request.
#[test]
fn requests_are_read_in_the_order_they_were_written() {
    let root: PathBuf = fixture("request-order");
    let first: String = new_request_id().expect("identity");
    std::thread::sleep(std::time::Duration::from_millis(3));
    let second: String = new_request_id().expect("identity");
    assert!(first < second, "{first} {second}");
    write_request(root.as_path(), &sample(second.as_str())).expect("written");
    write_request(root.as_path(), &sample(first.as_str())).expect("written");
    let (requests, refused) = read_requests(root.as_path()).expect("read");
    assert!(refused.is_empty());
    assert_eq!(requests.iter().map(|request| return request.id.clone()).collect::<Vec<String>>(), vec![first.clone(), second]);
    assert_eq!(
        requests[0].stage_container(),
        PathBuf::from(format!("/worktrees/{STAGE_PREFIX}{first}"))
    );
    remove(root.as_path());
}

/// A request is rewritten whole when it is tried again, and removed durably when it is finished.
#[test]
fn a_request_is_replaced_and_removed() {
    let root: PathBuf = fixture("request-replace");
    let request: CopyRequest = sample("0001-a");
    write_request(root.as_path(), &request).expect("written");
    assert!(requests_exist(root.as_path()).expect("exist"));
    assert!(copy_pending(root.as_path()).expect("pending"));
    write_request(root.as_path(), &CopyRequest { tries: 1, ..request.clone() }).expect("replaced");
    assert_eq!(read_requests(root.as_path()).expect("read").0[0].tries, 1);
    remove_request(root.as_path(), &request).expect("removed");
    assert!(!requests_exist(root.as_path()).expect("exist"));
    assert!(!copy_pending(root.as_path()).expect("pending"));
    remove_request(root.as_path(), &request).expect("removing twice is fine");
    remove(root.as_path());
}

/// A repository with nothing pending, or no copy directory at all, is not pending; a leftover journal is.
#[test]
fn only_requests_and_journals_make_a_copy_pending() {
    let root: PathBuf = fixture("request-pending");
    assert!(!copy_pending(root.as_path()).expect("absent"));
    ensure_journal_root(root.as_path()).expect("root");
    assert!(!copy_pending(root.as_path()).expect("empty"));
    std::fs::write(journal_root(root.as_path()).join("notes.txt"), b"x").expect("note");
    assert!(!copy_pending(root.as_path()).expect("others"));
    std::fs::write(journal_root(root.as_path()).join("abc.json"), b"{}").expect("journal-shaped file");
    assert!(copy_pending(root.as_path()).expect("journal"));
    remove(root.as_path());
}

/// A file that does not parse, or whose name is not its identity, is moved aside with a notice; the rest are read.
#[test]
fn damaged_requests_are_moved_aside() {
    let root: PathBuf = fixture("request-damaged");
    write_request(root.as_path(), &sample("0001-a")).expect("written");
    let directory: PathBuf = requests_directory(root.as_path());
    std::fs::write(directory.join("0000-broken.json"), b"{").expect("broken");
    std::fs::write(directory.join("0002-misnamed.json"), sample("0003-other").encode()).expect("misnamed");
    let (requests, refused) = read_requests(root.as_path()).expect("read");
    assert_eq!(requests.len(), 1);
    assert_eq!(refused.len(), 2, "{refused:?}");
    assert!(refused.iter().all(|line| return line.starts_with("cli-git: set aside the copy request")));
    assert!(given_up_directory(root.as_path()).join("0000-broken.json").exists());
    assert!(given_up_directory(root.as_path()).join("0002-misnamed.json").exists());
    remove(root.as_path());
}

/// Giving up on a request moves it out of the pending directory.
#[test]
fn giving_up_moves_the_request_aside() {
    let root: PathBuf = fixture("request-give-up");
    let request: CopyRequest = sample("0001-a");
    write_request(root.as_path(), &request).expect("written");
    give_up(root.as_path(), &request).expect("given up");
    assert!(!requests_exist(root.as_path()).expect("exist"));
    assert!(given_up_directory(root.as_path()).join("0001-a.json").exists());
    remove(root.as_path());
}

/// A temporary a killed writer left is removed once it is old; a fresh one, which a live writer is about to rename, stays.
#[test]
fn only_a_stale_temporary_is_removed() {
    let root: PathBuf = fixture("request-temporaries");
    write_request(root.as_path(), &sample("0001-a")).expect("written");
    let directory: PathBuf = requests_directory(root.as_path());
    let fresh: PathBuf = directory.join("0001-a.json.fresh.tmp");
    let stale: PathBuf = directory.join("0001-a.json.stale.tmp");
    std::fs::write(fresh.as_path(), b"being written").expect("fresh");
    std::fs::write(stale.as_path(), b"cut off").expect("stale");
    let now: SystemTime = SystemTime::now();
    std::fs::File::options()
        .write(true)
        .open(stale.as_path())
        .expect("open")
        .set_modified(now - STALE_TEMPORARY - Duration::from_secs(60))
        .expect("aged");
    assert_eq!(remove_stale_temporaries(root.as_path(), now).expect("cleaned"), 1);
    assert!(fresh.exists());
    assert!(!stale.exists());
    assert!(directory.join("0001-a.json").exists(), "the request itself is not a temporary");
    assert_eq!(remove_stale_temporaries(root.join("absent").as_path(), now).expect("absent"), 0);
    remove(root.as_path());
}
