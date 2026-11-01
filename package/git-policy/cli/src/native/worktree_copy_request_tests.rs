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
        identity: String::from("1:1:-"),
        allow_locked: false,
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
        ("\"version\":2", "\"version\":3"),
        ("\"identity\":\"1:1:-\"", "\"identity\":\"\""),
        ("\"allowLocked\":false", "\"allowLocked\":\"no\""),
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

/// Giving up writes the record with the tries it took before the pending request goes, so a client that asks later is told.
#[test]
fn giving_up_leaves_a_durable_record_with_the_tries() {
    let root: PathBuf = fixture("request-give-up");
    let request: CopyRequest = CopyRequest { tries: MAX_TRIES, ..sample("0001-a") };
    write_request(root.as_path(), &CopyRequest { tries: MAX_TRIES - 1, ..request.clone() }).expect("written");
    give_up(root.as_path(), &request).expect("given up");
    assert!(!requests_exist(root.as_path()).expect("exist"));
    let kept: String = std::fs::read_to_string(given_up_directory(root.as_path()).join("0001-a.json")).expect("record");
    assert_eq!(CopyRequest::decode(kept.as_str()).expect("a request").tries, MAX_TRIES, "the record keeps the last try");
    assert_eq!(terminal_of(root.as_path(), "0001-a").expect("terminal"), Terminal::GivenUp(request));
    remove(root.as_path());
}

/// A request that ended well leaves a record of how, before the pending request goes; absence is not an answer.
#[test]
fn a_finished_request_leaves_its_outcome() {
    let root: PathBuf = fixture("request-finish");
    let request: CopyRequest = sample("0001-a");
    assert_eq!(terminal_of(root.as_path(), "0001-a").expect("unknown"), Terminal::Unknown);
    write_request(root.as_path(), &request).expect("written");
    assert_eq!(terminal_of(root.as_path(), "0001-a").expect("pending"), Terminal::Pending);
    finish_request(root.as_path(), &request, DoneKind::Copied(7)).expect("finished");
    assert!(!requests_exist(root.as_path()).expect("exist"));
    assert_eq!(terminal_of(root.as_path(), "0001-a").expect("done"), Terminal::Done(DoneKind::Copied(7)));
    for (kind, id) in [(DoneKind::Already, "0002-a"), (DoneKind::Removed, "0003-a")] {
        let other: CopyRequest = sample(id);
        write_request(root.as_path(), &other).expect("written");
        finish_request(root.as_path(), &other, kind).expect("finished");
        assert_eq!(terminal_of(root.as_path(), id).expect("done"), Terminal::Done(kind));
    }
    remove(root.as_path());
}

/// Only the newest finished-request records are kept.
#[test]
fn old_finished_records_are_pruned() {
    let root: PathBuf = fixture("request-prune");
    let keep: usize = 3;
    for number in 0..=keep {
        let request: CopyRequest = sample(format!("{number:016x}-a").as_str());
        write_request(root.as_path(), &request).expect("written");
        finish_request(root.as_path(), &request, DoneKind::Already).expect("finished");
    }
    prune_done(done_directory(root.as_path()).as_path(), keep).expect("pruned");
    let kept: usize = std::fs::read_dir(done_directory(root.as_path())).expect("listing").count();
    assert_eq!(kept, keep);
    assert_eq!(terminal_of(root.as_path(), format!("{:016x}-a", 0).as_str()).expect("oldest"), Terminal::Unknown);
    assert_eq!(
        terminal_of(root.as_path(), format!("{keep:016x}-a").as_str()).expect("newest"),
        Terminal::Done(DoneKind::Already)
    );
    assert!(KEPT_DONE_RECORDS > keep, "the real bound is larger than the one this control exercises");
    remove(root.as_path());
}

/// A complete temporary of a request write is published, never deleted, whatever its age; one that does not parse is left
/// alone, because its writer is mid-write; one whose request stands, or ended, is not published over it.
#[test]
fn a_complete_temporary_is_adopted_and_nothing_is_deleted_by_age() {
    let root: PathBuf = fixture("request-temporaries");
    let directory: PathBuf = requests_directory(root.as_path());
    write_request(root.as_path(), &sample("0009-z")).expect("written");
    let complete: PathBuf = directory.join("0001-a.json.complete.tmp");
    let partial: PathBuf = directory.join("0002-b.json.partial.tmp");
    let standing: PathBuf = directory.join("0009-z.json.newer.tmp");
    std::fs::write(complete.as_path(), sample("0001-a").encode()).expect("complete");
    std::fs::write(partial.as_path(), b"{\"adminId\":").expect("partial");
    std::fs::write(standing.as_path(), sample("0009-z").encode()).expect("standing");
    // A temporary a day old stands exactly as a fresh one does.
    for path in [&complete, &partial, &standing] {
        std::fs::File::options()
            .write(true)
            .open(path.as_path())
            .expect("open")
            .set_modified(SystemTime::now() - std::time::Duration::from_secs(24 * 3600))
            .expect("aged");
    }
    assert_eq!(adopt_temporaries(root.as_path()).expect("adopted"), 1);
    assert!(directory.join("0001-a.json").exists(), "the complete temporary was published");
    assert!(!complete.exists());
    assert!(partial.exists(), "a partial temporary is not deleted");
    assert!(standing.exists(), "a temporary is not published over a request that stands");
    assert_eq!(adopt_temporaries(root.join("absent").as_path()).expect("absent"), 0);
    remove(root.as_path());
}

/// A writer whose temporary was adopted finds its request in place with its own text, and counts that as written.
#[test]
fn a_writer_whose_temporary_was_adopted_succeeds() {
    let root: PathBuf = fixture("request-adopted-writer");
    let request: CopyRequest = sample("0001-a");
    // The adopted state: the request stands, the writer's temporary is gone, and the writer's rename would fail.
    write_request(root.as_path(), &request).expect("written");
    write_request(root.as_path(), &request).expect("writing again is the same text");
    assert!(request_for_registration(root.as_path(), "topic", "0f0e-token").expect("lookup") == Some(String::from("0001-a")));
    assert_eq!(request_for_registration(root.as_path(), "topic", "another").expect("lookup"), None);
    remove(root.as_path());
}
