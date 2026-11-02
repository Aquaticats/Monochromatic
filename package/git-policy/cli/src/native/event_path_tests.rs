//! What:
//!  Controls for the event form of a pathname and its base64 encoding.
//! Why:
//!  A UTF-8 name must stay exactly as today,
//!  with no extra field;
//!  every other name
//!      must keep bytes a reader can decode back to the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(eventPath(Buffer.from('caf\xe9.txt', 'latin1')).exact).toEqual(...);
//! ```

/// The carrier and the encoder under test.
use super::{EventPath, base64_standard};

/// What:
///  Decode standard padded base64,
///  for the round-trip controls only.
/// Why:
///   An encoder checked only against itself could be wrong in a way its own output
///       hides;
///  this decoder is written from the alphabet,
///  not from the encoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decode = (text: string) => Buffer.from(text, 'base64');
/// ```
fn decode(text: &str) -> Vec<u8> {
    let alphabet: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    assert_eq!(text.len() % 4, 0, "padded base64 comes in groups of four");
    let mut bytes: Vec<u8> = Vec::new();
    for group in text.as_bytes().chunks(4) {
        let mut values: Vec<u32> = Vec::new();
        for character in group {
            if *character == b'=' {
                continue;
            }
            let position: usize = alphabet
                .find(char::from(*character))
                .expect("alphabet character");
            values.push(position as u32);
        }
        let mut joined: u32 = 0;
        for (index, value) in values.iter().enumerate() {
            joined |= value << (18 - 6 * index as u32);
        }
        let produced: usize = values.len() - 1;
        for index in 0..produced {
            bytes.push((joined >> (16 - 8 * index as u32)) as u8);
        }
    }
    return bytes;
}

/// The RFC 4648 section 10 test vectors,
///  every padding length included.
#[test]
fn base64_matches_the_rfc_vectors() {
    for (input, expected) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64_standard(input.as_bytes()), expected, "{input:?}");
    }
}

/// The decision brief's own example,
///  `caf` 0xE9 `.txt`,
///  encodes as the brief shows it.
#[test]
fn base64_encodes_the_brief_example() {
    assert_eq!(
        base64_standard(b"caf\xe9.txt"),
        "Y2Fm6S50eHQ=",
        "doc/planning/cli-git-rust-open-decisions.md, Option C"
    );
}

/// Every byte value,
///  including the two that use `+` and `/`,
///  survives a round trip.
#[test]
fn base64_round_trips_every_byte_value() {
    let every: Vec<u8> = (0..=255).collect();
    let encoded: String = base64_standard(every.as_slice());
    assert!(encoded.contains('+') && encoded.contains('/'), "{encoded}");
    assert_eq!(encoded.len(), 344);
    assert_eq!(decode(encoded.as_str()), every);
    for length in 0..=7 {
        let prefix: &[u8] = &every[250 - length..250];
        assert_eq!(decode(base64_standard(prefix).as_str()), prefix);
    }
}

/// A UTF-8 name,
///  multi-byte characters and control characters included,
///  keeps no bytes.
#[test]
fn a_utf8_name_has_no_exact_bytes() {
    for name in ["a.txt", "doc/café.md", "\u{1}tab\there.md", "😀/x", ""] {
        let path: EventPath = EventPath::from_git_bytes(name.as_bytes());
        assert_eq!((path.text(), path.exact()), (name, None), "{name:?}");
        assert_eq!(path.bytes(), name.as_bytes());
    }
}

/// Every kind of invalid sequence keeps the exact bytes,
///  and the text replaces them.
#[test]
fn a_name_that_is_not_utf8_keeps_its_bytes() {
    let cases: [(&[u8], &str); 7] = [
        // A Latin-1 byte inside ASCII.
        (b"caf\xe9.txt", "caf\u{fffd}.txt"),
        // A name made only of invalid bytes.
        (b"\xff\xfe\xfd", "\u{fffd}\u{fffd}\u{fffd}"),
        // A valid multi-byte name, the control that keeps nothing.
        ("é".as_bytes(), "é"),
        // Valid multi-byte UTF-8 next to an invalid byte.
        (b"\xc3\xa9\xff/x.md", "é\u{fffd}/x.md"),
        // A truncated three-byte sequence.
        (b"a\xe2\x82", "a\u{fffd}"),
        // An overlong encoding of `/`.
        (b"\xc0\xaf", "\u{fffd}\u{fffd}"),
        // An encoded surrogate, which UTF-8 forbids.
        (b"\xed\xa0\x80.md", "\u{fffd}\u{fffd}\u{fffd}.md"),
    ];
    for (bytes, text) in cases {
        let path: EventPath = EventPath::from_git_bytes(bytes);
        assert_eq!(path.text(), text, "{bytes:?}");
        let valid: bool = std::str::from_utf8(bytes).is_ok();
        if valid {
            assert_eq!(path.exact(), None, "{bytes:?}");
        } else {
            assert_eq!(path.exact(), Some(bytes), "{bytes:?}");
        }
        assert_eq!(path.bytes(), bytes);
    }
}

/// Every byte Git allows in a name (all but NUL) survives,
///  and two names that read alike
/// stay apart by their bytes.
#[test]
fn names_that_read_alike_stay_apart() {
    let every: Vec<u8> = (1..=255).collect();
    let path: EventPath = EventPath::from_git_bytes(every.as_slice());
    assert_eq!(path.exact(), Some(every.as_slice()));
    assert_eq!(decode(base64_standard(path.bytes()).as_str()), every);
    let first: EventPath = EventPath::from_git_bytes(b"caf\xe9.txt");
    let second: EventPath = EventPath::from_git_bytes(b"caf\xe8.txt");
    assert_eq!(first.text(), second.text());
    assert_ne!(first, second);
}

/// A display path is text only,
///  even when it spells escapes.
#[test]
fn a_display_path_never_has_bytes() {
    let path: EventPath = EventPath::display("folder/\\xffname");
    assert_eq!((path.text(), path.exact()), ("folder/\\xffname", None));
    assert_eq!(path.bytes(), b"folder/\\xffname");
}
