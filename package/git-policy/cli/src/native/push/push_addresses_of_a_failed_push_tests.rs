//! Controls of the addresses a failed push is judged by:
//! what a remote's answer names,
//! and what a destination that is no remote name is read as.

use super::*;

/// What:
///  A remote with several push addresses has every one of them,
/// in the order Git prints them.
/// Why:
///  Git stops a push at the first address that fails,
/// so one address that can never be a remote is enough for no try to land,
/// and the order is what says which address a person should look at first.
#[test]
fn every_push_address_the_configuration_names_is_read_in_order() {
    let answer: QueryOutput = QueryOutput { code: Some(0), stdout: b"DISABLED\n/srv/real.git\n".to_vec(), stderr: Vec::new() };
    assert_eq!(push_addresses_from(Some(&answer), "origin"), vec![String::from("DISABLED"), String::from("/srv/real.git")]);
    let one: QueryOutput = QueryOutput { code: Some(0), stdout: b"  /srv/real.git  \n\n".to_vec(), stderr: Vec::new() };
    assert_eq!(push_addresses_from(Some(&one), "origin"), vec![String::from("/srv/real.git")], "surrounding spaces and blank lines are not addresses");
}

/// What:
///  A destination that is no remote name is its own address,
/// and so is one whose answer says nothing.
/// Why:
///  `branch.<name>.pushRemote` and `remote.pushDefault` may be a URL,
/// and Git refuses
/// `remote get-url` for anything that names no remote;
/// the address the push went to is then the destination written in the configuration.
/// An answer that failed or named nothing is nothing known,
/// and the destination is judged as it stands.
#[test]
fn a_destination_that_names_no_remote_is_read_as_itself() {
    assert_eq!(push_addresses_from(None, "DISABLED"), vec![String::from("DISABLED")], "a query that could not run");
    let refused: QueryOutput = QueryOutput { code: Some(2), stdout: Vec::new(), stderr: b"error: No such remote 'DISABLED'\n".to_vec() };
    assert_eq!(push_addresses_from(Some(&refused), "DISABLED"), vec![String::from("DISABLED")], "a query Git refused");
    let empty: QueryOutput = QueryOutput { code: Some(0), stdout: Vec::new(), stderr: Vec::new() };
    assert_eq!(push_addresses_from(Some(&empty), "https://host/repo.git"), vec![String::from("https://host/repo.git")], "an answer that named nothing");
}

/// What:
///  The question both faces ask is the push addresses of the named remote,
/// all of them.
/// Why:
///  `--push` reads the push address rather than the fetch one,
/// and `--all` gives every one,
/// which is what a plan's fate depends on.
#[test]
fn the_query_asks_for_every_push_address_of_the_remote() {
    let asked: Vec<OsString> = query_arguments("origin");
    assert_eq!(asked, vec![OsString::from("remote"), OsString::from("get-url"), OsString::from("--push"), OsString::from("--all"), OsString::from("origin")]);
}
