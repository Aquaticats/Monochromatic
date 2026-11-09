//! Pins the one hexadecimal spelling that pin keys and watcher state names share.

/// Brings the encoder under test into scope.
use super::*;
/// The runtime directory watcher state names live under.
use crate::application_watch_state::watch_state_directory;
/// The bpffs directory pin keys live under.
use crate::pin::{pin_dir_for, PIN_ROOT};
/// Expected spellings are compared as whole paths.
use std::path::{Path, PathBuf};

/// Watcher key and cgroup path whose bytes both call sites encode.
///
/// A space,
///  a lowercase hex letter pair inside a name,
///  and a two-byte UTF-8 character cover the cases a wrong alphabet would change.
const SAMPLE_BYTES: &[u8] = b"/sys/fs/cgroup/app.slice/a b \xc3\xbc";
/// The one spelling `SAMPLE_BYTES` must produce,
///  written out by hand from the bytes.
const SAMPLE_SPELLING: &str = "2f7379732f66732f6367726f75702f6170702e736c6963652f61206220c3bc";

/// No bytes stay no characters.
#[test]
fn empty_bytes_stay_empty() {
    assert_eq!(bytes_as_lowercase_hex(b""), "");
}

/// Each byte becomes its high nibble then its low nibble,
///  lowercase,
///  with no separator.
#[test]
fn each_byte_is_two_lowercase_digits_with_the_high_nibble_first() {
    let spelled: String = bytes_as_lowercase_hex(b"\x00\x0f\xa5\xff\x10");
    assert_eq!(spelled, "000fa5ff10");
    assert!(
        spelled.bytes().all(|byte| return matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "lowercase hexadecimal digits only: {spelled}",
    );
}

/// The pin key and the watcher state name spell one input the same way,
///  which is what lets either one be read back by the other.
#[test]
fn pin_key_and_watcher_state_name_spell_one_sample_the_same_way() {
    let sample_key: &str = std::str::from_utf8(SAMPLE_BYTES).expect("the sample is UTF-8");
    let watcher_directory: PathBuf = watch_state_directory(sample_key).expect("watcher state directory");
    assert_eq!(
        watcher_directory,
        Path::new("/run/wg-quicker-exempt/watchers").join(SAMPLE_SPELLING),
    );
    let pin_directory: PathBuf = pin_dir_for(Path::new(sample_key)).expect("pin directory");
    assert_eq!(pin_directory, Path::new(PIN_ROOT).join(SAMPLE_SPELLING));
}
