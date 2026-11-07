//! Controls for reading a blob as a Git LFS pointer the way `git-lfs` 3.8.1 does.

use super::*;

/// A valid object name.
const OID: &str = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

/// The canonical pointer text for an object of `size` bytes.
fn pointer(size: &str) -> String {
    return format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\nsize {size}\n");
}

/// The canonical pointer,
/// the older versions,
/// and tolerated spellings (no final line feed,
/// carriage returns,
/// blank lines,
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

/// White space around the text,
/// trailing spaces after a value and a missing final line feed do not change the verdict.
#[test]
fn white_space_around_the_text_is_ignored() {
    let expected: Verdict = Verdict::Pointer(OID.to_owned());
    assert_eq!(decode_pointer(format!("{}  \n\t\n ", pointer("5")).as_bytes()), expected);
    assert_eq!(decode_pointer(format!("  \n{}", pointer("5")).as_bytes()), expected);
    assert_eq!(decode_pointer(pointer("5").trim_end().as_bytes()), expected);
}

/// Everything that is not a pointer for `git-lfs` uploads nothing:
/// plain text,
/// a wrong version,
/// a bad object name,
/// a bad
///  size,
/// keys out of order,
/// a missing key,
/// an extra line,
/// an empty blob and a pointer of size zero.
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

/// A blob of the cutoff size or more is never read as a pointer,
/// one byte under it still is.
#[test]
fn the_size_cutoff_is_exclusive() {
    let canonical: String = pointer("9");
    // Leading line feeds pad the blob to the size wanted and read as blank lines.
    let padding_for = |total: usize| -> String {
        return format!("{}{}", "\n".repeat(total - canonical.trim_end().len()), canonical.trim_end());
    };
    assert_eq!(decode_pointer(padding_for(BLOB_SIZE_CUTOFF - 1).as_bytes()), Verdict::Pointer(OID.to_owned()));
    assert_eq!(decode_pointer(padding_for(BLOB_SIZE_CUTOFF).as_bytes()), Verdict::NotUploadable);
    assert_eq!(decode_pointer(&vec![b'a'; BLOB_SIZE_CUTOFF * 4]), Verdict::NotUploadable);
}

/// Extension lines and non-ASCII edges around a text the port cannot judge are declined,
/// never guessed.
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

/// The real `git-lfs` on the test process's PATH,
/// if any.
fn real_git_lfs() -> Option<std::path::PathBuf> {
    let path: std::ffi::OsString = std::env::var_os("PATH")?;
    return std::env::split_paths(&path)
        .map(|directory| { return directory.join("git-lfs"); })
        .find(|candidate| { return candidate.is_file(); });
}

/// Whether `git-lfs pointer --check` takes the text for a pointer (`DecodePointerFromFile`,
/// the decoder `git-lfs` uses).
fn oracle_accepts(program: &std::path::Path, directory: &std::path::Path, text: &[u8]) -> bool {
    let file: std::path::PathBuf = directory.join("blob.txt");
    std::fs::write(&file, text).expect("blob");
    let output = std::process::Command::new(program)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", directory)
        .args(["pointer", "--check", "--file"])
        .arg(&file)
        .output()
        .expect("git-lfs starts");
    return match output.status.code() {
        Some(0) => true,
        Some(1) => false,
        other => panic!("git-lfs pointer --check ended with {other:?}: {}", String::from_utf8_lossy(output.stderr.as_slice())),
    };
}

/// The port agrees with the real decoder:
/// whatever it names as a pointer `git-lfs` accepts,
/// and whatever `git-lfs` rejects
///  it does not name;
/// the only accepted texts it does not name are the ones that upload nothing (empty,
/// size zero) and the
///  ones it declines (extension lines).
/// Skipped,
/// with a note,
/// on hosts without `git-lfs`.
#[test]
fn the_port_agrees_with_the_real_decoder() {
    let Some(program) = real_git_lfs() else {
        eprintln!("skipped: git-lfs is not on PATH, so the real decoder was not consulted");
        return;
    };
    let root = crate::test_support::fixture("lfs-pointer-oracle");
    let canonical: String = pointer("9");
    let big_padding: String = format!("{}{}", "\n".repeat(BLOB_SIZE_CUTOFF), canonical);
    let cases: Vec<Vec<u8>> = vec![
        canonical.clone().into_bytes(),
        pointer("0").into_bytes(),
        pointer("+7").into_bytes(),
        pointer("-1").into_bytes(),
        pointer("12a").into_bytes(),
        pointer("99999999999999999999").into_bytes(),
        canonical.trim_end().as_bytes().to_vec(),
        canonical.replace('\n', "\r\n").into_bytes(),
        format!("\n\n{canonical}\n\n").into_bytes(),
        format!("{canonical}  \n\t").into_bytes(),
        canonical.replace("https://git-lfs.github.com/spec/v1", "http://git-media.io/v/2").into_bytes(),
        canonical.replace("https://git-lfs.github.com/spec/v1", "https://hawser.github.com/spec/v1").into_bytes(),
        canonical.replace("spec/v1", "spec/v2").into_bytes(),
        canonical.replace(OID, OID.to_uppercase().as_str()).into_bytes(),
        canonical.replace(OID, &OID[1..]).into_bytes(),
        canonical.replace("sha256:", "sha1:").into_bytes(),
        format!("version https://git-lfs.github.com/spec/v1\nsize 9\noid sha256:{OID}\n").into_bytes(),
        format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\n").into_bytes(),
        format!("oid sha256:{OID}\nsize 9\n").into_bytes(),
        format!("{canonical}other value\n").into_bytes(),
        format!("version https://git-lfs.github.com/spec/v1\next-0-foo sha256:{OID}\noid sha256:{OID}\nsize 9\n").into_bytes(),
        format!("version https://git-lfs.github.com/spec/v1\noid sha256:{OID}\nsize 9\next-0-foo sha256:{OID}\n").into_bytes(),
        b"hello\nworld\n".to_vec(),
        b"see git-lfs docs\n".to_vec(),
        Vec::new(),
        big_padding.into_bytes(),
        [canonical.as_bytes(), "\u{a0}".as_bytes()].concat(),
        vec![b'a'; BLOB_SIZE_CUTOFF * 2],
    ];
    for text in cases {
        let verdict: Verdict = decode_pointer(text.as_slice());
        let accepted: bool = oracle_accepts(program.as_path(), root.as_path(), text.as_slice());
        let shown: String = String::from_utf8_lossy(text.as_slice()).into_owned();
        match verdict {
            Verdict::Pointer(_) => assert!(accepted, "named as a pointer but git-lfs rejects it: {shown:?}"),
            Verdict::NotUploadable => {
                let uploads_nothing: bool = text.is_empty() || shown.lines().any(|line| { return line == "size 0"; });
                assert!(!accepted || uploads_nothing, "git-lfs accepts it but the port names nothing: {shown:?}");
            }
            Verdict::Decline => {}
        }
    }
    crate::test_support::remove(root.as_path());
}
