//! What:
//!  The 8 KiB probe contract of `read_with_binary_check` at the binary
//!  boundary:
//!  a file whose first 8 KiB holds a NUL byte is scanned there and
//!  no further,
//!  while a NUL-free file is read whole.
//! Why:
//!  A secret in a binary blob is exactly what a deny-list scanner must catch,
//!  and an unsound NUL shortcut once hid it (BUG 5).
//!  The cap that remains is a stated tradeoff,
//!  so both directions need a test:
//!  the probe always runs,
//!  and the tail past it is an acceptable miss.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // binary-probe.test.ts: one case per row of the probe design table
//! ```

use std::fs;

use crate::{
    BIN,
    Command,
    unique_tmp,
};

#[test]
fn nul_byte_in_file_does_not_skip_scan() {
    let dir = unique_tmp("bug5");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("with_nul.txt");
    // Literal + NUL + tail. The literal is entirely before the NUL.
    fs::write(&target, b"SECRET_NEEDLE_XYZ_LONG_ENOUGH\0and then more").expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "BUG 5: file containing NUL must still be scanned; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("with_nul.txt"),
        "BUG 5: stderr must reference the target file; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] fn large_text_file_secret_after_probe_is_matched()`
//           creates a > 8 KiB file with NO NUL byte and the deny-listed
//           literal placed AFTER the first 8 KiB probe span. Asserts
//           the scanner reads the whole file and finds the literal.
// Why:      Validates the text-file path of `read_with_binary_check`:
//           when the first 8 KiB has no NUL the heuristic must read
//           the entire file. This is the row of the design table where
//           the file is large but text, ending with the secret.
#[test]
fn large_text_file_secret_after_probe_is_matched() {
    let dir = unique_tmp("bin-probe-large-text");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("large_text.txt");
    let mut content = Vec::with_capacity(9000 + 32);
    content.extend(std::iter::repeat_n(b'X', 9000));
    content.extend_from_slice(b"SECRET_NEEDLE_XYZ_LONG_ENOUGH");
    fs::write(&target, &content).expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "large text file (>8 KiB, no NUL): secret at byte 9000 must be \
         matched (we read the whole file when the probe is NUL-free); \
         got success.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] fn large_binary_file_secret_in_probe_before_nul_is_matched()`
//           creates a > 8 KiB file whose first 8 KiB contains BOTH the
//           deny-listed literal AND a NUL byte AFTER the literal, and
//           whose tail past 8 KiB is more bytes. Asserts the scanner
//           emits the hit.
// Why:      Validates that detecting a NUL in the probe does NOT cause
//           the probe itself to be discarded: the literal sits inside
//           the probe window and must still match. This is the soundness
//           guarantee that closes BUG 5 while keeping the binary-tail
//           bound.
#[test]
fn large_binary_file_secret_in_probe_before_nul_is_matched() {
    let dir = unique_tmp("bin-probe-secret-in-probe");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("binary_with_leading_secret.bin");
    let mut content: Vec<u8> = Vec::with_capacity(9000);
    content.extend_from_slice(b"SECRET_NEEDLE_XYZ_LONG_ENOUGH");
    content.push(0);
    content.extend(std::iter::repeat_n(b'X', 9000));
    fs::write(&target, &content).expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "large binary file (>8 KiB, NUL in probe): literal before the \
         NUL must match; got success.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] fn large_binary_file_secret_after_probe_is_acceptably_missed()`
//           creates a > 8 KiB file whose first 8 KiB contains a NUL
//           byte and whose deny-listed literal sits AFTER the probe
//           span. Asserts the scanner exits clean (the secret is
//           NOT reported).
// Why:      Pins the "acceptable miss" half of the binary heuristic:
//           when the probe shows the file is binary AND the file is
//           larger than the probe, we cap per-file work at 8 KiB and
//           skip the rest. Documenting this behavior in a test prevents
//           silent regression in either direction (re-scanning binary
//           tails, or accidentally discarding the probe).
#[test]
fn large_binary_file_secret_after_probe_is_acceptably_missed() {
    let dir = unique_tmp("bin-probe-secret-after-probe");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("binary_secret_in_tail.bin");
    let mut content: Vec<u8> = Vec::with_capacity(9000 + 32);
    content.extend(std::iter::repeat_n(b'X', 100));
    content.push(0);
    content.extend(std::iter::repeat_n(b'X', 8900));
    content.extend_from_slice(b"SECRET_NEEDLE_XYZ_LONG_ENOUGH");
    fs::write(&target, &content).expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        output.status.success(),
        "binary heuristic (file >8 KiB with NUL in probe): the literal \
         sitting in the binary tail past byte 8192 is an acceptable \
         miss; got non-zero exit.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}
