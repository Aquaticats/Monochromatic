//! What:
//!  Controls for a reference name kept as bytes.
//! Why:
//!  A name that is not text must reach Git byte for byte,
//!  and a record must hold the whole name,
//!  or a recovery moves another branch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(refNameFromGit(Buffer.from('refs/heads/topic-\xff\n')).os).toEqual(Buffer.from('refs/heads/topic-\xff'));
//! ```

/// The name under test.
use super::RefName;

/// What:
///  The name Git prints is everything before the line feed it ends the answer with.
/// Why:
///   `git symbolic-ref --quiet HEAD` prints one ref and a line feed,
/// and the name is the bytes before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refNameFromGit(Buffer.from('refs/heads/main\n')).toString()).toEqual('refs/heads/main')
/// ```
#[test]
fn the_name_is_what_git_printed_before_its_line_feed() {
    assert_eq!(RefName::from_git_output(b"refs/heads/main\n"), RefName::from_text("refs/heads/main"));
    // A name that is not text keeps every byte, which a lossy reading would replace.
    let raw: RefName = RefName::from_git_output(b"refs/heads/topic-\xff\n");
    assert_eq!(raw.bytes(), b"refs/heads/topic-\xff");
    assert_eq!(raw.to_string(), "refs/heads/topic-\u{fffd}");
    assert_eq!(raw.spoken(), "$'refs/heads/topic-\\xff'");
    assert_eq!(raw.short().spoken(), "$'topic-\\xff'");
}

/// What:
///  A record holds a name that is not text as its bytes,
/// and the name comes back whole.
/// Why:
///   A record is text,
/// so a hexadecimal field is the only form that loses no byte,
///  and a name a recovery reads from it must be the ref the person has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refNameOfHex(refName.hex())).toEqual(refName)
/// ```
#[test]
fn a_record_holds_the_name_as_its_bytes() {
    let raw: RefName = RefName::from_git_output(b"refs/heads/topic-\xff\n");
    assert_eq!(raw.hex(), "726566732f68656164732f746f7069632dff");
    assert_eq!(RefName::from_hex(raw.hex().as_str()), Some(raw.clone()));
    assert_eq!(RefName::from_hex("zz"), None);
    assert_eq!(RefName::from_hex("abc"), None);
}

/// What:
///  A name is text only when its bytes read as one,
///  and `HEAD` is the one name the landing moves as the head.
/// Why:
///  `is_text` decides which plain record field an earlier release reads,
/// and `is_head` decides between a branch and a detached head.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refNameOf('refs/heads/main').isText).toBe(true); expect(refNameOf('HEAD').isHead).toBe(true)
/// ```
#[test]
fn a_name_tells_whether_it_is_text_and_whether_it_is_the_head() {
    assert!(RefName::from_text("refs/heads/main").is_text());
    assert!(!RefName::from_git_output(b"refs/heads/topic-\xff\n").is_text());
    assert!(RefName::from_text("HEAD").is_head());
    assert!(!RefName::from_text("refs/heads/HEAD").is_head());
    assert_eq!(RefName::from_text("HEAD").short(), RefName::from_text("HEAD"));
}
