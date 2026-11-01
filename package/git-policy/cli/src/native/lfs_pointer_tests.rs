//! Controls for reading a blob as a Git LFS pointer the way `git-lfs` 3.8.1 does.

use super::*;

/// A valid object name.
const OID: &str = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

/// The canonical pointer text for an object of `size` bytes.
fn pointer(size: &str) -> String {
    return format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\nsize {size}\n");
}

/// The canonical pointer, the older versions, and tolerated spellings (no final line feed, carriage returns, blank lines,
///  a leading `+` on the size) are pointers.
#[test]
fn pointers_in_every_accepted_spelling_name_their_object() {
    let expected: Verdict = Verdict::Pointer(OID.to_owned());
    assert_eq!(decode_pointer(pointer("12345").as_bytes()), expected);
    assert_eq!(decode_pointer(pointer("12345").trim_end().as_bytes()), expected);
    assert_eq!(decode_pointer(pointer("+7").as_bytes()), expected);
    assert_eq!(decode_pointer(pointer("12345").replace('\n', "\r\n").as_bytes()), expected);
    assert_eq!(decode_pointer(format!("\n\n{}\n\n", pointer("1")).as_bytes()), expected);
    for version in ["http://git-media.io/v/2", "https://hawser.github.com/spec/v1"] {
        let text: String = pointer("5").replace("https://git-lfs.github.com/spec/v1", version);
        assert_eq!(decode_pointer(text.as_bytes()), expected, "{version}");
    }
}

/// Everything that is not a pointer for `git-lfs` uploads nothing: plain text, a wrong version, a bad object name, a bad
///  size, keys out of order, a missing key, an extra line, an empty blob and a pointer of size zero.
#[test]
fn text_that_git_lfs_would_not_take_for_a_pointer_uploads_nothing() {
    let canonical: String = pointer("9");
    let cases: Vec<(&str, String)> = vec![
        ("plain", String::from("hello\nworld\n")),
        ("empty", String::new()),
        ("zero size", pointer("0")),
        ("negative size", pointer("-1")),
        ("size with letters", pointer("12a")),
        ("size beyond i64", pointer("99999999999999999999")),
        ("wrong version", canonical.replace("spec/v1", "spec/v2")),
        ("upper-case oid", canonical.replace(OID, OID.to_uppercase().as_str())),
        ("short oid", canonical.replace(OID, &OID[1..])),
        ("other hash", canonical.replace("sha256:", "sha1:")),
        ("keys out of order", format!("version https://git-lfs.github.com/spec/v1\nsize 9\noid sha256:{OID}\n")),
        ("missing size", format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\n")),
        ("missing version", format!("oid sha256:{OID}\nsize 9\n")),
        ("extra line", format!("{canonical}other value\n")),
        ("a line without a space", format!("version https://git-lfs.github.com/spec/v1\nlonely\nsize 9\n")),
        ("no header word", canonical.replace("git-lfs.github.com", "example.invalid").replace("spec/v1", "v1")),
    ];
    for (name, text) in cases {
        assert_eq!(decode_pointer(text.as_bytes()), Verdict::NotUploadable, "{name}");
    }
}

/// A blob of the cutoff size or more is never read as a pointer, one byte under it still is.
#[test]
fn the_size_cutoff_is_exclusive() {
    let canonical: String = pointer("9");
    let padding_for = |total: usize| -> String {
        return format!("{}{}", canonical.trim_end(), " ".repeat(total - canonical.trim_end().len()));
    };
    assert_eq!(decode_pointer(padding_for(BLOB_SIZE_CUTOFF - 1).as_bytes()), Verdict::Pointer(OID.to_owned()));
    assert_eq!(decode_pointer(padding_for(BLOB_SIZE_CUTOFF).as_bytes()), Verdict::NotUploadable);
    assert_eq!(decode_pointer(&vec![b'a'; BLOB_SIZE_CUTOFF * 4]), Verdict::NotUploadable);
}

/// Extension lines and non-ASCII edges around a text the port cannot judge are declined, never guessed.
#[test]
fn spellings_the_port_cannot_judge_are_declined() {
    let with_extension: String = format!(
        "version https://git-lfs.github.com/spec/v1\next-0-foo sha256:{OID}\noid sha256:{OID}\nsize 9\n"
    );
    assert_eq!(decode_pointer(with_extension.as_bytes()), Verdict::Decline);
    let mut edged: Vec<u8> = pointer("9").into_bytes();
    edged.extend_from_slice("\u{a0}".as_bytes());
    edged.extend_from_slice(b"extra line here\n\xc2\xa0");
    assert_eq!(decode_pointer(edged.as_slice()), Verdict::Decline);
}
