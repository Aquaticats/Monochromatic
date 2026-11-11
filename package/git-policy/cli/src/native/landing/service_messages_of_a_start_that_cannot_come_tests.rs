//! What:
//!  Controls for the two sentences of a start that cannot come:
//! what each names,
//! and that neither promises a second way to commit.
//! Why:
//!  These are the messages that replace two minutes of waiting,
//! so the fact a person acts on has to be in them,
//! and the sentence that says nothing changed has to be there too,
//! because the service is the only writer of a commit.

use super::*;
use crate::service_messages::DENYLIST;

/// Every term of the denylist,
/// matched as whole words and ignoring case,
/// as the message controls match it.
fn leaks(text: &str, term: &str) -> bool {
    let haystack: String = text.to_lowercase();
    let needle: String = term.to_lowercase();
    let mut from: usize = 0;
    while let Some(at) = haystack[from..].find(needle.as_str()) {
        let start: usize = from + at;
        let end: usize = start + needle.len();
        let before: bool = haystack[..start].chars().next_back().is_none_or(|character| { return !character.is_alphanumeric(); });
        let after: bool = haystack[end..].chars().next().is_none_or(|character| { return !character.is_alphanumeric(); });
        if before && after {
            return true;
        }
        from = start + 1;
    }
    return false;
}

/// What:
///  The sentence for an address that no socket address can hold names the address,
/// both lengths,
/// the setting to change and the room it may take,
/// and says nothing changed.
/// Why:
///  The length is the whole cause,
/// and the runtime directory is the only part of the address the person sets.
#[test]
fn an_address_that_cannot_exist_names_the_setting_its_room_and_the_remedy() {
    let said: String = socket_path_too_long("/home/a/temp/agent/x/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock", 113, 107, 40);
    for part in [
        "/home/a/temp/agent/x/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock",
        "113 bytes",
        "107 bytes",
        "XDG_RUNTIME_DIR",
        "40 bytes",
        "run the commit again",
        NOTHING_CHANGED,
    ] {
        assert!(said.contains(part), "the sentence names {part:?}: {said}");
    }
    for term in DENYLIST {
        assert!(!leaks(said.as_str(), term), "the sentence leaks {term:?}: {said}");
    }
}

/// What:
///  The sentence for a service the manager made elsewhere names both addresses,
/// both runtime directories and the setting,
/// and says nothing changed.
/// Why:
///  A person has two runtime directories in play here,
/// and the wrong one in the remedy would send them back to the same wait.
#[test]
fn a_service_made_elsewhere_names_both_addresses_and_both_directories() {
    let said: String = service_elsewhere(
        "/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock",
        "/home/a/temp/agent/x/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock",
        "/run/user/1000",
        "/home/a/temp/agent/x",
    );
    for part in [
        "/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock",
        "/home/a/temp/agent/x/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock",
        "runtime directory /run/user/1000",
        "/home/a/temp/agent/x",
        "XDG_RUNTIME_DIR",
        "Run the commit from your login session",
        NOTHING_CHANGED,
    ] {
        assert!(said.contains(part), "the sentence names {part:?}: {said}");
    }
    for term in DENYLIST {
        assert!(!leaks(said.as_str(), term), "the sentence leaks {term:?}: {said}");
    }
}
