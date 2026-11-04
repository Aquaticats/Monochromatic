//! Controls for the shell quoting: the form chosen for each kind of value, and a round trip through a real shell for
//!  hostile values (a separator, a substitution, quotes, a newline, bytes that are not UTF-8).

use super::*;
use std::os::unix::ffi::OsStrExt;
use std::process::Command;

/// A value of safe characters is printed as it is, the empty value as two quotes, and anything else in single quotes
///  with each single quote spelled `'\''`.
#[test]
fn each_kind_of_value_gets_its_form() {
    assert_eq!(shell_quote(b""), "''");
    assert_eq!(shell_quote(b"/mnt/work-1_a.b,c=d:e@f%g+h"), "/mnt/work-1_a.b,c=d:e@f%g+h");
    assert_eq!(shell_quote(b"/mnt/work; sudo reboot"), "'/mnt/work; sudo reboot'");
    assert_eq!(shell_quote(b"it's"), "'it'\\''s'");
    assert_eq!(shell_quote("naïve dir".as_bytes()), "'naïve dir'");
    assert_eq!(shell_quote(b"$(id)`id`\"x\" \\y"), "'$(id)`id`\"x\" \\y'");
}

/// Control characters and bytes that are not UTF-8 are spelled as escapes in the ANSI-C form, never printed.
#[test]
fn controls_and_invalid_bytes_are_escaped_in_the_ansi_c_form() {
    assert_eq!(shell_quote(b"a\nb\tc"), "$'a\\nb\\tc'");
    assert_eq!(shell_quote(b"a\x1b[0m"), "$'a\\x1b[0m'");
    assert_eq!(shell_quote(b"work-\xff"), "$'work-\\xff'");
    assert_eq!(shell_quote("a\u{85}b".as_bytes()), "$'a\\xc2\\x85b'");
    assert_eq!(shell_quote(b"it's \\ \xff"), "$'it\\'s \\\\ \\xff'");
}

/// Hostile values come back from a real shell as exactly one word with exactly the original bytes.
#[test]
fn a_real_shell_reads_each_hostile_value_back_unchanged() {
    let values: [&[u8]; 8] = [
        b"/mnt/work; echo injected",
        b"/mnt/$(echo injected)",
        b"`echo injected`",
        b"it's \"quoted\"",
        b"two  spaces\tand a tab",
        b"line one\nline two",
        b"work-\xff-\xfe",
        b"--delete=rootflags=\"subvol=@my root,compress=zstd\"",
    ];
    for value in values {
        let quoted: String = shell_quote(value);
        let script: String = format!("printf '%s\\0' {quoted}");
        let output = Command::new("bash").arg("-c").arg(script).output().expect("bash runs");
        assert!(output.status.success(), "{quoted}");
        let words: Vec<&[u8]> = output.stdout.split(|byte: &u8| { return *byte == 0; }).collect();
        assert_eq!(words.len(), 2, "one word and the empty tail after the last NUL: {quoted}");
        assert_eq!(words[0], value, "{quoted}");
        assert_eq!(std::ffi::OsStr::from_bytes(words[0]).as_bytes(), value);
    }
}
