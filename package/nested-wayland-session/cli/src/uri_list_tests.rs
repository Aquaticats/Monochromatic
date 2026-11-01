//! Unit tests for the file URI encoder.

use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};

use super::{file_uri, uri_list};

/// Plain paths pass through unchanged after the scheme.
#[test]
fn plain_paths_are_not_encoded() {
    assert_eq!(file_uri(Path::new("/home/user/music/a-b_c.d~e.mp3")), "file:///home/user/music/a-b_c.d~e.mp3");
}

/// A space becomes `%20` and a literal percent sign becomes `%25`.
#[test]
fn spaces_and_percent_signs_are_encoded() {
    assert_eq!(file_uri(Path::new("/music/Jean Michel Jarre/a b.mp3")), "file:///music/Jean%20Michel%20Jarre/a%20b.mp3");
    assert_eq!(file_uri(Path::new("/music/100%.mp3")), "file:///music/100%25.mp3");
    assert_eq!(file_uri(Path::new("/music/%20.mp3")), "file:///music/%2520.mp3");
}

/// Characters with meaning in a URI are encoded.
#[test]
fn uri_delimiters_are_encoded() {
    assert_eq!(file_uri(Path::new("/m/a#b?c&d=e;f+g")), "file:///m/a%23b%3Fc%26d%3De%3Bf%2Bg");
    assert_eq!(file_uri(Path::new("/m/a\"b'c<d>e")), "file:///m/a%22b%27c%3Cd%3Ee");
    assert_eq!(file_uri(Path::new("/m/a\nb\r\nc\td")), "file:///m/a%0Ab%0D%0Ac%09d");
}

/// Non-ASCII names are encoded as their UTF-8 bytes,
///  and non-UTF-8 bytes are kept as they are on disk.
#[test]
fn non_ascii_bytes_are_encoded() {
    assert_eq!(file_uri(Path::new("/m/Bj\u{f6}rk/\u{304b}\u{3081}.mp3")), "file:///m/Bj%C3%B6rk/%E3%81%8B%E3%82%81.mp3");
    assert_eq!(file_uri(Path::new(OsStr::from_bytes(b"/m/\xff\xfe"))), "file:///m/%FF%FE");
}

/// Several files make one CRLF-terminated line each.
#[test]
fn a_list_has_one_crlf_line_per_file() {
    let list = uri_list(&[Path::new("/a b"), Path::new("/c")]);
    assert_eq!(list, b"file:///a%20b\r\nfile:///c\r\n".to_vec());
    assert_eq!(uri_list(&[]), Vec::<u8>::new());
}
