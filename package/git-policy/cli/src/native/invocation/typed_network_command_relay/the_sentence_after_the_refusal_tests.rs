//! Controls for the sentence a refusal prints:
//! which refusal a transcript reads as,
//! the three texts byte for byte,
//! and the record that one was printed.

use super::{
    Refusal,
    printed_for_this_command,
    refusal_in,
};
use crate::typed_network_command_relay::a_target_the_sentence_may_name::Target;

/// What:
///  The host the proposal's fixture names,
///  so the sentence can be compared with the kept file byte for byte.
/// Why:
///  The three texts were measured with `target` standing for the host,
///  and a comparison against them is only exact with the
///  same word in the slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const target = { host: 'target', connect: 'target' };
/// ```
fn the_fixture_s_host() -> Target {
    return Target {
        host: String::from("target"),
        connect: String::from("target"),
    };
}

/// A transcript that is OpenSSH's own reads as the refusal it printed.
#[test]
fn each_refusal_is_read_from_the_words_it_prints() {
    let unknown: &str = "Cloning into 'probe'...\nHost key verification failed.\nfatal: Could not read from remote repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.\n";
    assert_eq!(refusal_in(unknown), Some(Refusal::HostKey));
    let strict: &str = "No ED25519 host key is known for [localhost]:2222 and you have requested strict checking.\nHost key verification failed.\n";
    assert_eq!(refusal_in(strict), Some(Refusal::StrictChecking));
    let locked: &str = "root@localhost: Permission denied (publickey).\nfatal: Could not read from remote repository.\n";
    assert_eq!(refusal_in(locked), Some(Refusal::KeyNotUsable));
    let missing: &str = "Host key verification failed.\nfatal: repository 'x' does not exist\n";
    // The last line alone is an unknown key's own words, which is the default `ask` refusal.
    assert_eq!(refusal_in(missing), Some(Refusal::HostKey));
    let changed: &str = "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!\nHost key for [localhost]:2222 has changed and you have requested strict checking.\nHost key verification failed.\n";
    assert_eq!(
        refusal_in(changed),
        None,
        "a changed key keeps OpenSSH's own words"
    );
    let unreachable: &str = "ssh: connect to host 127.0.0.1 port 2299: Connection refused\nfatal: Could not read from remote repository.\n";
    assert_eq!(refusal_in(unreachable), None);
    assert_eq!(refusal_in(""), None);
}

/// A line Git relayed from the far end is not a refusal,
/// with or without the escape a terminal puts in front of it.
#[test]
fn a_remote_s_own_line_is_dropped_before_the_reading() {
    let relayed: &str = "remote: Host key verification failed.\n";
    assert_eq!(refusal_in(relayed), None);
    let relayed_at_a_terminal: &str = "\u{1b}[Kremote: Host verification failed.\n";
    assert_eq!(refusal_in(relayed_at_a_terminal), None);
    // The same words on the program's own stream are still a refusal.
    let own: &str = "remote: Host key verification failed.\nHost key verification failed.\n";
    assert_eq!(refusal_in(own), Some(Refusal::HostKey));
}

/// The record starts unset and is set by the printing,
/// so a second face of one refusal stays silent.
#[test]
fn one_refusal_is_recorded_once() {
    assert!(!printed_for_this_command());
    let target: Target = Target {
        host: String::from("target"),
        connect: String::from("target"),
    };
    // A command that ended well prints nothing, however its transcript reads.
    super::print_after_a_refusal(b"Host key verification failed.\n", 0, &target);
    assert!(!printed_for_this_command());
    // A transcript that reads as no refusal prints nothing either.
    super::print_after_a_refusal(b"fatal: not a git repository\n", 128, &target);
    assert!(!printed_for_this_command());
    super::print_after_a_refusal(b"Host key verification failed.\n", 128, &target);
    assert!(printed_for_this_command());
}

/// The host-key sentence is the seventh revision's kept file byte for byte,
/// with the fixture host in both of its slots:
/// the one the owner's decision names,
/// whose fingerprint clause is in the design's "The words".
#[test]
fn the_host_key_sentence_is_the_kept_file_byte_for_byte() {
    // What: `r#"..."#` is a raw string: backslashes and quotes are literal, so the kept text is compared byte for byte.
    // Why:  A comparison that re-escaped the text could hide a difference in a quote or a backslash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const expected = String.raw`...`;
    // ```
    let expected: &str = r#"monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, or holds an outdated one for it, and no command the wrapper runs asks whether to trust a new one, so no connection got past the key. Connect with plain `ssh target`, which the wrapper does not touch, and answer the question it asks; if it names a key that changed instead, compare the fingerprint it shows with the one you know to be right before you accept it, then remove the line it names and connect once more, which then asks about the new key, and run this command again.
"#;
    let sentence: String = super::sentence_for(Refusal::HostKey, &the_fixture_s_host());
    assert_eq!(sentence, expected);
    assert_eq!(sentence.len(), 565);
}

/// The strict-checking sentence is the kept file byte for byte,
/// with the fixture host in both of its slots.
#[test]
fn the_strict_sentence_is_the_kept_file_byte_for_byte() {
    // What: `r#"..."#` is a raw string: backslashes and quotes are literal, so the kept text is compared byte for byte.
    // Why:  A comparison that re-escaped the text could hide a difference in a quote or a backslash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const expected = String.raw`...`;
    // ```
    let expected: &str = r#"monochromatic-git-policy-cli: this machine has no SSH host key recorded for `target`, and your settings refuse to ask about a new one (`StrictHostKeyChecking=yes`), so the connection stopped before any login. Connect once with plain `ssh -o StrictHostKeyChecking=ask target`, which the wrapper does not touch and which asks once, then run this command again; or add that host's key to `known_hosts` yourself.
"#;
    let sentence: String = super::sentence_for(Refusal::StrictChecking, &the_fixture_s_host());
    assert_eq!(sentence, expected);
    assert_eq!(sentence.len(), 409);
}

/// The key sentence is the kept file byte for byte;
/// it names no host,
/// so no slot is filled.
#[test]
fn the_passphrase_sentence_is_the_kept_file_byte_for_byte() {
    // What: `r#"..."#` is a raw string: backslashes and quotes are literal, so the kept text is compared byte for byte.
    // Why:  A comparison that re-escaped the text could hide a difference in a quote or a backslash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const expected = String.raw`...`;
    // ```
    let expected: &str = r#"monochromatic-git-policy-cli: the key this connection offered was refused or could not be unlocked, and no command the wrapper runs asks a person for a passphrase, so a key no agent holds cannot be used and a key the server does not accept answers the same way. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add` (`ssh-add -l` lists what the agent holds), or use a key without a passphrase, then run this command again; a key the server still refuses needs the server's account to accept it.
"#;
    let sentence: String = super::sentence_for(Refusal::KeyNotUsable, &the_fixture_s_host());
    assert_eq!(sentence, expected);
    assert_eq!(sentence.len(), 547);
}
