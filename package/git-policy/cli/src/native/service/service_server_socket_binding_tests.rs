//! Controls of the text a wait and a failure give the address of a service:
//! a path a person can look at,
//! an abstract name that is said to be a name,
//! and the descriptor the service manager passed in.

use super::*;
use std::path::PathBuf;

/// What:
///  Every kind of address is named in words a person can act on,
/// and an address that is no path never claims to be one.
/// Why:
///   The notice and the failure that end a wait name the address,
/// which is the one fact that tells two worktrees'
///  services apart;
/// an abstract name has no file to look at,
/// so naming it as a path would send the person to a file that does not
///  exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(addressText({ path: '/run/user/1000/x.sock' })).toBe('/run/user/1000/x.sock');
/// ```
#[test]
fn every_kind_of_address_is_named_in_words_a_person_can_act_on() {
    let socket: PathBuf = PathBuf::from("/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock");
    assert_eq!(address_text(&Bind::Path(socket.clone())), socket.display().to_string(), "a path is the path");
    let name: String = String::from("monochromatic-git-policy-cli-0123");
    assert_eq!(address_text(&Bind::Abstract(name.clone())), format!("the name {name}"), "an abstract name is said to be a name and never a path");
    assert_eq!(
        address_text(&Bind::Inherited),
        String::from("the one this service was started with"),
        "the descriptor the service manager passed in names no address of its own"
    );
}
