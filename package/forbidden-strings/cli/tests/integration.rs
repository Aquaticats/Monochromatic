// What:     Integration tests for the `forbidden-strings` binary. Each
//           test spawns the release binary as a subprocess and asserts
//           on its stdout/stderr/exit-code. Cargo's `tests/*.rs` layout
//           compiles each file as a separate test executable that links
//           the binary as a sibling artifact -- the `CARGO_BIN_EXE_<name>`
//           env var (compile-time `env!`) points at the binary.
// Why:      Several BUGs in the audit live in `main.rs` -- read-error
//           handling, path skip logic, the --all walker integration --
//           and can only be verified at the binary boundary. Unit tests
//           on internal helpers would miss the wiring.

use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

// What:     `#[path = "..."] mod blocked_root;` compiles the named file as
//           a private module of this test crate.
// Why:      The embedding-warning suite includes the same file, so both
//           assert one platform-specific reason for a cache root blocked
//           by a regular file.
//
// In TS you'd write (pseudocode):
// ```ts
// import * as blockedRoot from './support/blocked_root.ts';
// ```
#[path = "support/blocked_root.rs"]
mod blocked_root;

// What:     `const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");`
//           uses the compile-time env var Cargo sets for integration
//           tests of a binary crate. The value is the absolute path to
//           the freshly built binary; `env!` panics at compile time if
//           the env var is absent (it never is for this crate's tests).
// Why:      Avoid hardcoding `target/release/forbidden-strings` -- that
//           breaks under `cargo test` (debug profile) and on platforms
//           where the path layout differs.
//
// In TS you'd write (pseudocode):
// ```ts
// const BIN = process.env.CARGO_BIN_EXE_forbidden_strings!;
// ```
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Test command factory injecting one disposable process-scoped cache root.
struct Command;

/// Builds subprocess commands without touching developer's real user cache.
impl Command {
    /// Creates process command with hermetic cache-root override.
    fn configured(program: impl AsRef<std::ffi::OsStr>) -> ProcessCommand {
        static CACHE_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        let cache_root = CACHE_ROOT.get_or_init(|| {
            let path = std::env::temp_dir().join(format!(
                "forbidden-strings-integration-cache-{}",
                std::process::id(),
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create integration cache root");
            return path
        });
        let mut command = ProcessCommand::new(program);
        command.env("FORBIDDEN_STRINGS_CACHE_DIR", cache_root);
        return command
    }
}

// What:     `fn unique_tmp(label) -> PathBuf` returns a fresh empty
//           directory under `std::env::temp_dir()`. Uses PID + label so
//           parallel test runs do not collide.
// Why:      Tests create rules files and target files; sharing a dir
//           across tests is a flake source.
//
// In TS you'd write (pseudocode):
// ```ts
// function uniqueTmp(label: string): string {
//   return path.join(os.tmpdir(), `${label}-${process.pid}`);
// }
// ```
fn unique_tmp(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "fs-int-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create tmp dir");
    return dir
}

// What:     `#[test] fn read_error_surfaces_as_hit_and_nonzero_exit()`.
//           BUG 4 regression test. Creates an unreadable file (chmod
//           000) containing a deny-listed literal. Pre-fix, the scanner
//           silently substituted empty content and exited 0. Post-fix
//           the scanner emits a `<path>: read error: ...` line on
//           stderr and exits 1.
// Why:      Secret-scanning CI must NOT pass on files it could not
//           inspect; a permission error is a signal, not a silent
//           success.
//
// In TS you'd write (pseudocode):
// ```ts
// test("read error surfaces as hit and non-zero exit", () => { ... });
// ```
//
// What:     `#[cfg(unix)]` compiles this test only for Unix targets.
// Why:      Mode bits are a Unix permission model; Windows has no
//           `PermissionsExt`, and its equivalent unreadable file is the
//           sharing-violation case in `windows_locked_file_read_error_surfaces_as_hit`.
#[test]
#[cfg(unix)]
fn read_error_surfaces_as_hit_and_nonzero_exit() {
    let dir = unique_tmp("bug4");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("unreadable.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");
    // What:     `fs::set_permissions(&target, fs::Permissions::from_mode(0))`
    //           chmod 000 the target. Use the Unix-specific extension
    //           trait to set bare-bits permissions; the Windows
    //           equivalent is a separate test because Windows has no
    //           mode bits to clear.
    // Why:      Force an io::ErrorKind::PermissionDenied at read time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.chmodSync(target, 0);
    // ```
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&target, fs::Permissions::from_mode(0o000))
        .expect("chmod 000");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    // What:     Restore permissions so cleanup can remove the file.
    // Why:      Tempdir cleanup on a chmod-000 file would itself fail
    //           with EACCES, leaving the fixture behind for the next
    //           run.
    let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o644));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "BUG 4: unreadable file must produce non-zero exit; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    assert!(
        stderr.contains("read error"),
        "BUG 4: stderr must contain `read error`; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] #[cfg(windows)] fn windows_locked_file_read_error_surfaces_as_hit()`.
//           The Windows equivalent of the BUG 4 regression test. This
//           test process opens the target with share mode 0, which
//           tells Windows that no other open of the file may proceed
//           while this handle lives. The spawned scanner's open then
//           fails with a sharing violation, a real Windows read error.
// Why:      Secret-scanning CI must not pass on a file it could not
//           open on Windows either; mode bits cannot express that there.
//
// In TS you'd write (pseudocode):
// ```ts
// test("locked file read error surfaces as hit", () => { ... });
// ```
#[test]
#[cfg(windows)]
fn windows_locked_file_read_error_surfaces_as_hit() {
    // What:     `use std::os::windows::fs::OpenOptionsExt;` brings the
    //           Windows-only `share_mode` builder method into scope.
    //           Rust adds platform methods to a shared type through such
    //           extension traits, which must be imported to be callable.
    // Why:      `share_mode(0)` is how a std file open denies every other
    //           open of the same file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // import { openExclusive } from 'windows-fs';
    // ```
    use std::os::windows::fs::OpenOptionsExt;

    let dir = unique_tmp("bug4-windows-lock");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("locked.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");
    // What:     `let lock: fs::File = fs::OpenOptions::new().read(true).share_mode(0).open(&target)`
    //           opens the target for reading and denies shared read,
    //           write and delete access to every other opener. `&target`
    //           lends the path without giving it away. `.expect(...)`
    //           unwraps the `Result`, stopping the test if the open fails.
    // Why:      The handle must stay alive while the scanner runs, so it
    //           is bound to a name that lives until the explicit `drop`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lock = openExclusive(target);
    // ```
    let lock: fs::File = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&target)
        .expect("open target exclusively");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    // What:     `drop(lock)` closes the exclusive handle now.
    // Why:      Cleanup cannot delete a file that is still open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lock.close();
    // ```
    drop(lock);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "locked file must produce non-zero exit; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    assert!(
        stderr.contains("locked.txt: read error"),
        "stderr must report the locked file's read error; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] fn nul_byte_in_file_does_not_skip_scan()`. BUG 5
//           regression test. Creates a file whose content begins with
//           the deny-listed literal then a NUL byte. Pre-fix, the
//           presence of NUL in the first 8 KiB caused `is_likely_binary`
//           to return true and the entire scan to short-circuit, so
//           the literal was never detected even though it appeared
//           BEFORE the NUL. Post-fix the binary-skip is removed and AC
//           scans raw bytes content-agnostic.
// Why:      A redacted secret in a binary blob (lockfile sidecar,
//           bundled artifact, accidentally-committed image) is exactly
//           the kind of thing a deny-list scanner should catch. The
//           skip-on-NUL heuristic was an unsound shortcut.
//
// In TS you'd write (pseudocode):
// ```ts
// test("NUL byte does not skip scan", () => { ... });
// ```
// What:     `#[test] fn explicit_arg_with_skip_basename_is_still_scanned()`.
//           BUG 6 regression test. The pre-fix `is_skipped_file` matched
//           by basename anywhere in the tree, so an explicit argument
//           like `sub/forbidden-strings.local.txt` was silently skipped
//           even though it lives in a different directory than the
//           scanner's actual rule file. Post-fix the skip logic is
//           path-anchored AND only applied to walker output (--all
//           mode), not to explicit positional args.
// Why:      "The user asked" -- positional args are an explicit request
//           that overrides every basename-based heuristic. The pre-fix
//           skip was an over-aggressive guard that hid real positive
//           findings under CI invocations like
//           `forbidden-strings <path>` for paths whose basenames
//           happened to collide.
//
// In TS you'd write (pseudocode):
// ```ts
// test("explicit arg with skip-basename is still scanned", () => { ... });
// ```
#[test]
fn explicit_arg_with_skip_basename_is_still_scanned() {
    let dir = unique_tmp("bug6");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    // Create a subdir whose target file's basename collides with the
    // hardcoded skip list. Pre-fix the scanner skipped it silently.
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).expect("create sub");
    let target = sub.join("forbidden-strings.local.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "BUG 6: explicit positional arg must be scanned regardless of basename; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("forbidden-strings.local.txt"),
        "BUG 6: stderr must reference the target file; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     A user regex rule in the engine dialect
//           (`/AKIA[A-Z2-7]{4}/`, a bounded class + bounded
//           repetition) compiles at startup via `compile_from_text`
//           and matches a line containing the shape, emitting a
//           `PATH:LINE rule=N` finding.
// Why:      Replaces the old BUG-8 test, whose rule `/(?i)adafruit[\s]+=/`
//           the new dialect rejects (inline `(?i)` and unbounded `+`
//           are both unsupported). This pins the migration's actual
//           contract: a supported regex rule compiles and fires
//           through the engine line scan.
#[test]
fn user_regex_rule_matches_bounded_pattern() {
    let dir = unique_tmp("user-regex");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "/AKIA[A-Z2-7]{4}/\n").expect("write rules");
    let target = dir.join("key.txt");
    fs::write(&target, b"prefix AKIA2345 suffix\n").expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "user regex rule must match the AKIA shape; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    assert!(
        stderr.contains("key.txt:1 rule=0"),
        "stderr must carry the columnless finding for line 1 rule 0; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     A finding on the third line of a file emits exactly
//           `PATH:LINE rule=N` with no column segment, and reports the
//           correct 1-based line number.
// Why:      The migration drops columns (`PATH:LINE:COL_START..COL_END`
//           becomes `PATH:LINE rule=N`); this pins the new columnless
//           shape and the line-number mapping at the binary boundary.
#[test]
fn output_is_columnless_with_correct_line_number() {
    let dir = unique_tmp("columnless");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "NEEDLE_LITERAL_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("multi.txt");
    fs::write(&target, "clean one\nclean two\nNEEDLE_LITERAL_LONG_ENOUGH\n")
        .expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "must exit non-zero on a hit; stderr: {}", stderr);
    let hit_line = stderr
        .lines()
        .find(|line| return line.contains("rule="))
        .expect("a rule= finding line");
    // Exact columnless shape: the path, a colon, the line number, then ` rule=`.
    assert!(
        hit_line.ends_with("multi.txt:3 rule=0"),
        "finding must be columnless `PATH:3 rule=0`; got: {}",
        hit_line,
    );
    // No column-range segment survives the migration.
    assert!(
        !hit_line.contains(".."),
        "finding must not carry a column range; got: {}",
        hit_line,
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] fn windows_style_path_does_not_basename_skip()`.
//           BUG 11 regression test. Pre-fix the skip-set check did
//           `path.rsplit('/').next()` to extract a basename, which on
//           a Windows-style path like `sub\forbidden-strings.local.txt`
//           returns the full string -- there's no `/` to split on. So
//           the basename comparison failed in a way that depended on
//           the host's path separator: the same logical bug as BUG 6
//           (basename collision skips legitimate files), but triggered
//           by a different surface (the missing separator type). Post-
//           fix the skip check uses `std::fs::canonicalize` + `PathBuf`
//           equality, which is separator-agnostic and OS-aware.
//
//           Construct a target file under a subdirectory using a name
//           that has no path separator INSIDE it (the directory
//           separator is the host's, but the leaf carries no `/`).
//           This exercises the path-anchored matching on the exact
//           shape that broke the old rsplit-based code.
// Why:      Document BUG 11 as a distinct test even though the fix
//           overlaps BUG 6. Future readers grepping for "BUG 11" land
//           on a concrete check rather than a comment-only entry.
#[test]
fn windows_style_path_does_not_basename_skip() {
    let dir = unique_tmp("bug11");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    // Leaf filename contains no separator at all -- mirrors the failure
    // mode where rsplit('/') yielded the entire path on Windows-style
    // input. The BUG 6 integration test exercises a directory hierarchy
    // built with forward slashes; this one uses a flat single file at
    // tempdir root so the leaf itself is the rsplit fallthrough.
    let target = dir.join("forbidden-strings.local.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "BUG 11: explicit positional arg matching the skip basename must \
         still be scanned (path-anchored not basename-anchored); got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] fn config_file_at_cwd_is_skipped_even_as_explicit_arg()`.
//           Regression test for the always-on `forbidden-strings.*.txt`-at-cwd
//           skip. A `forbidden-strings.append.txt` sitting in the scanner's
//           cwd holds the same literal the rules file forbids; passing it as
//           an explicit positional arg must NOT produce a hit, because the
//           scanner's own ruleset files self-match. The contrast invocation
//           proves the skip is name-anchored: a non-config file at cwd with
//           the identical literal IS scanned and DOES hit.
// Why:      CI scans changed files positionally
//           (`forbidden-strings --rules ... <changed>...`); when a PR edits
//           the checked-in `forbidden-strings.append.txt`, it lands in that
//           changed set, and scanning it would re-derive every rule body as a
//           false positive. The skip closes that without reintroducing BUG 6
//           (a same-named file in a subdirectory still scans; see the
//           BUG 6 / BUG 11 tests above).
#[test]
fn config_file_at_cwd_is_skipped_even_as_explicit_arg() {
    let dir = unique_tmp("cwd-config-skip");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    // A forbidden-strings.*.txt config file AT cwd whose body carries the
    // forbidden literal. As the scanner's own ruleset file it must be skipped.
    let cfg = dir.join("forbidden-strings.append.txt");
    fs::write(&cfg, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write cfg");
    // A plain content file at cwd with the SAME literal: must be scanned.
    let content = dir.join("content.txt");
    fs::write(&content, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write content");

    // Run with cwd == dir so the config file sits directly at the cwd root.
    let skipped = Command::configured(BIN)
        .current_dir(&dir)
        .args(["--rules", "rules.txt", "forbidden-strings.append.txt"])
        .output()
        .expect("spawn binary");
    assert!(
        skipped.status.success(),
        "config file forbidden-strings.append.txt at cwd must be skipped even as explicit arg; got non-zero.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&skipped.stdout),
        String::from_utf8_lossy(&skipped.stderr),
    );

    let scanned = Command::configured(BIN)
        .current_dir(&dir)
        .args(["--rules", "rules.txt", "content.txt"])
        .output()
        .expect("spawn binary");
    assert!(
        !scanned.status.success(),
        "a non-config file at cwd with the forbidden literal must still be scanned; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&scanned.stdout),
        String::from_utf8_lossy(&scanned.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn nul_byte_in_file_does_not_skip_scan() {
    let dir = unique_tmp("bug5");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("with_nul.txt");
    // Literal + NUL + tail. The literal is entirely before the NUL.
    fs::write(&target, b"SECRET_NEEDLE_XYZ_LONG_ENOUGH\0and then more")
        .expect("write target");

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

// What:     `#[test] fn help_long_flag_exits_zero_and_lists_usage()`.
//           `--help` must exit 0 (success) and print the usage block --
//           `USAGE:`, the binary name, and at least one flag header --
//           to stdout. Convention matches `cargo`, `rustc`, every
//           POSIX-shaped CLI.
// Why:      A regression that, say, accidentally redirected the help
//           text to stderr or returned exit 2 would break shells and
//           wrappers that pipe `--help` through a pager and assert on
//           the exit code. Pins both the channel and the exit shape.
#[test]
fn help_long_flag_exits_zero_and_lists_usage() {
    let output = Command::configured(BIN)
        .arg("--help")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "--help must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(stdout.contains("forbidden-strings"), "help missing program name; stdout: {}", stdout);
    assert!(stdout.contains("USAGE:"), "help missing USAGE block; stdout: {}", stdout);
    assert!(stdout.contains("--rules"), "help missing --rules flag; stdout: {}", stdout);
    assert!(stdout.contains("--all"), "help missing --all flag; stdout: {}", stdout);
}

// What:     `-h` short alias must behave identically to `--help`.
// Why:      Argv parsing in main.rs is a manual while-loop. A typo or
//           dropped branch on `-h` would silently fall through to the
//           "unknown flag" arm and exit 2; this test pins the alias.
#[test]
fn help_short_flag_exits_zero_and_lists_usage() {
    let output = Command::configured(BIN)
        .arg("-h")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "-h must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(stdout.contains("USAGE:"), "short -h missing USAGE; stdout: {}", stdout);
}

// What:     `--version` exits 0 and prints `forbidden-strings <semver>`
//           on stdout. The semver baked in via `env!("CARGO_PKG_VERSION")`
//           is the version Cargo.toml declares for this crate.
// Why:      Tools (mise, asdf, package managers) parse `--version`
//           output to verify which build is installed. A regression
//           that swapped the channel or dropped the prefix would break
//           those tools silently.
#[test]
fn version_long_flag_exits_zero_and_prints_version_line() {
    let output = Command::configured(BIN)
        .arg("--version")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "--version must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.starts_with("forbidden-strings "),
        "version output must begin with program name; got: {}",
        stdout,
    );
    // Crude semver shape check: must contain at least one digit (the
    // major version). Avoids hardcoding the current version here, which
    // would force this test to update on every release.
    assert!(
        stdout.chars().any(|c| return c.is_ascii_digit()),
        "version output must contain at least one digit; got: {}",
        stdout,
    );
}

#[test]
fn version_short_flag_exits_zero_and_prints_version_line() {
    let output = Command::configured(BIN)
        .arg("-V")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "-V must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.starts_with("forbidden-strings "),
        "short -V must print version line; got: {}",
        stdout,
    );
}

// What:     `--rules <missing-file>` must exit 2 (config error) and
//           emit a `forbidden-strings: ...` error on stderr.
// Why:      A "rules path I cannot read" is a deployment / wiring
//           failure, not a code-level violation. Exit 2 is the agreed
//           channel for "the scanner could not run", distinct from 1
//           ("ran fine, found violations") and 0 ("ran fine, clean").
//           CI wrappers branch on this distinction.
#[test]
fn missing_rules_file_exits_with_config_error() {
    let dir = unique_tmp("missing-rules");
    let rules = dir.join("does-not-exist.txt");
    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "missing rules file must exit 2; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("forbidden-strings:"),
        "stderr must carry the program-prefixed error; got: {}",
        stderr,
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     `--rules` argument must win over `FORBIDDEN_STRINGS_RULES`
//           env var when both are set. Set the env to a literal that
//           DOES match the target; set --rules to a different file whose
//           literal does NOT match. Only --rules should be consulted, so
//           the run exits 0.
// Why:      Documented precedence in README: "--rules flag, then env
//           var, then default". A regression that swapped them would
//           silently use the wrong ruleset in CI. The user catches it
//           when a known-bad string slips through; the test catches it
//           before that.
#[test]
fn rules_flag_wins_over_env_var() {
    let dir = unique_tmp("rules-precedence");
    let env_rules = dir.join("env-rules.txt");
    fs::write(&env_rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write env rules");
    let flag_rules = dir.join("flag-rules.txt");
    // Literal that does not appear in the target file.
    fs::write(&flag_rules, "UNRELATED_LITERAL_NEVER_PRESENT\n").expect("write flag rules");
    let target = dir.join("target.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");

    let output = Command::configured(BIN)
        .env("FORBIDDEN_STRINGS_RULES", &env_rules)
        .args(["--rules"])
        .arg(&flag_rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "--rules must win: when flag rules don't match target, exit must be 0.\n\
         stdout: {}\nstderr: {}",
        stdout,
        stderr,
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     Repeating `--rules` should keep the last value, matching the old
//           handwritten parser's assignment behaviour.
// Why:      Clap rejects duplicate `ArgAction::Set` options unless configured to
//           let later occurrences override earlier ones. CI wrappers can layer a
//           default `--rules` before a caller-supplied override; the migration
//           must not turn that shape into a usage error.
#[test]
fn repeated_rules_flag_uses_last_value() {
    let dir = unique_tmp("rules-repeat");
    let first_rules = dir.join("first-rules.txt");
    fs::write(&first_rules, "UNRELATED_LITERAL_NEVER_PRESENT\n").expect("write first rules");
    let second_rules = dir.join("second-rules.txt");
    fs::write(&second_rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write second rules");
    let target = dir.join("target.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&first_rules)
        .args(["--rules"])
        .arg(&second_rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "last repeated --rules value must drive matching; expected non-zero exit.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     `--rules` must accept a value whose path token begins with `-`.
// Why:      The old manual parser treated the token immediately after `--rules`
//           as a value, not as another flag. Clap needs `allow_hyphen_values` on
//           that option to preserve the same path surface.
#[test]
fn rules_flag_accepts_hyphen_prefixed_path_value() {
    let dir = unique_tmp("rules-hyphen-path");
    let rules = dir.join("-rules.txt");
    fs::write(&rules, "UNRELATED_LITERAL_NEVER_PRESENT\n").expect("write rules");
    let target = dir.join("target.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");

    let output = Command::configured(BIN)
        .current_dir(&dir)
        .args(["--rules", "-rules.txt", "target.txt"])
        .output()
        .expect("spawn binary");

    assert!(
        output.status.success(),
        "hyphen-prefixed rules path should parse as a value and produce clean exit.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     With `FORBIDDEN_STRINGS_RULES` set and no `--rules` flag,
//           the scanner must use the env-pointed rules file. Set it to
//           a rules file whose literal MATCHES the target; expect a
//           non-zero exit and a hit on stderr.
// Why:      Pins the "env var is the second-priority source" half of
//           the precedence rule. Without this test, swapping precedence
//           (env wins over --rules) would still pass `rules_flag_wins_
//           over_env_var` if the env var were silently ignored.
#[test]
fn env_var_supplies_rules_when_no_flag() {
    let dir = unique_tmp("env-supplies-rules");
    let env_rules = dir.join("env-rules.txt");
    fs::write(&env_rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write env rules");
    let target = dir.join("target.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");

    let output = Command::configured(BIN)
        .env("FORBIDDEN_STRINGS_RULES", &env_rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    assert!(
        !output.status.success(),
        "env-supplied rules must drive matching; expected non-zero exit.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     A short bare literal rule matches only at word boundaries.
//           Rule "ACR" must hit a standalone "see ACR here" but must NOT
//           hit a glued "ACRYLIC" occurrence.
// Why:      Bare literals under eight bytes are gated behind word
//           boundaries (so "ACR" compiles to \bACR\b), confining a short
//           run to whole-token matches instead of firing inside a longer
//           token or a base64 blob. The over-matching preference still
//           holds for longer literals and regex rules; this narrowing is
//           specific to short bare literals.
#[test]
fn short_literal_matches_whole_word_not_substring() {
    let dir = unique_tmp("short-word-boundary");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "ACR\n").expect("write rules");

    // (1) Standalone occurrence: must match (word boundaries on both sides).
    let hit_file = dir.join("hit.txt");
    fs::write(&hit_file, "see ACR here\n").expect("write hit file");
    let hit_output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&hit_file)
        .output()
        .expect("spawn binary");
    assert!(
        !hit_output.status.success(),
        "literal `ACR` must match standalone occurrence; exit was success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&hit_output.stdout),
        String::from_utf8_lossy(&hit_output.stderr),
    );

    // (2) Glued occurrence: must NOT match (no trailing word boundary).
    let glued_file = dir.join("glued.txt");
    fs::write(&glued_file, "see ACRYLIC here\n").expect("write glued file");
    let glued_output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&glued_file)
        .output()
        .expect("spawn binary");
    assert!(
        glued_output.status.success(),
        "literal `ACR` must NOT match inside `ACRYLIC` under word-boundary gating.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&glued_output.stdout),
        String::from_utf8_lossy(&glued_output.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `--all` mode must skip the configured rules file. The skip
//           set in main.rs canonicalizes the resolved rules path; the
//           walker's output is filtered against that set. Setup: git-
//           init a fresh repo, write rules containing a deny-listed
//           literal that ALSO appears literally inside the rules file
//           (the rule body itself). Run `--all` from that repo. The
//           rules file is the only tracked content; since it is the
//           configured rules file, the walker output filters it out.
//           Exit must be 0.
// Why:      Without the skip the rules file self-matches every rule it
//           declares -- every `--all` run on a repo that includes a
//           rules file would fail with hits on the rules file itself.
//           The skip-set canonicalization is the fix; this test pins
//           it through the binary.
#[test]
fn all_mode_skips_configured_rules_file() {
    let dir = unique_tmp("all-skip-rules");

    // What:     Resolve real git binary path. The dev environment in
    //           this repo wraps `git` with a CLI policy enforcer that
    //           rejects bulk-add `.` shapes; tests need direct access
    //           to the real binary so the setup steps run unmodified.
    //           Mirrors the helper in src/walk.rs tests.
    let git_bin = if std::path::Path::new("/usr/bin/git").exists() {
        "/usr/bin/git"
    } else {
        "git"
    };
    let init_status = Command::configured(git_bin)
        .args(["init", "-q"])
        .current_dir(&dir)
        .status()
        .expect("git init");
    assert!(init_status.success(), "git init must succeed in {:?}", dir);

    // The rule body must be at least the substring-threshold byte
    // length so a literal copy of it in the rules file would self-match
    // were the file scanned. 30 bytes here -- well past the 7-byte
    // threshold.
    let rules_path = dir.join("myrules.txt");
    fs::write(&rules_path, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let add_status = Command::configured(git_bin)
        .args(["add", "myrules.txt"])
        .current_dir(&dir)
        .status()
        .expect("git add");
    assert!(add_status.success(), "git add must succeed");

    let output = Command::configured(BIN)
        .current_dir(&dir)
        .args(["--rules"])
        .arg(&rules_path)
        .arg("--all")
        .output()
        .expect("spawn binary");

    assert!(
        output.status.success(),
        "--all must skip configured rules file; got non-zero exit.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("myrules.txt"),
        "rules file must not appear as a hit; stderr: {}",
        stderr,
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     An unknown flag (`--no-such-flag`) must exit 2 (usage
//           error) and surface the offending token on stderr.
// Why:      Clap rejects unknown options before the scanner runs. A regression
//           that silently treated unknowns as positional file args would cause
//           `forbidden-strings --typo-flag` to scan a file named `--typo-flag`,
//           a confusing silent failure because the downstream read would surface
//           as a file error. Exit 2 plus clap's "unexpected argument" message is
//           the right shape.
#[test]
fn unknown_flag_exits_with_usage_error() {
    let output = Command::configured(BIN)
        .arg("--no-such-flag")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "unknown flag must exit 2; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("unexpected argument"),
        "stderr must carry clap's unknown-argument wording; got: {}",
        stderr,
    );
    assert!(
        stderr.contains("--no-such-flag"),
        "stderr must name the offending token; got: {}",
        stderr,
    );
}

// What:     `--rules` with no following argument must exit 2 (usage
//           error) and emit a clear "needs an argument" message on
//           stderr.
// Why:      The argv loop advances by 2 when it sees `--rules`; if the
//           value is missing it must short-circuit before indexing
//           past the end. A regression that panicked here would crash
//           the process with a backtrace instead of a clean usage
//           error.
#[test]
fn rules_flag_without_value_exits_with_usage_error() {
    let output = Command::configured(BIN)
        .arg("--rules")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "--rules without value must exit 2; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("--rules"),
        "stderr must mention the flag; got: {}",
        stderr,
    );
}

// What:     `fn fake_github_oauth_token() -> String` builds a string
//           matching the baseline's github-oauth rule
//           (`/gho\_[0-9a-zA-Z]{36}/`) at RUN time.
// Why:      The token must never appear as a literal in this source
//           file: the repository scans its own tree with the same
//           baseline, and a literal match here would flag the test as
//           a leaked credential.
//
// In TS you'd write (pseudocode):
// ```ts
// function fakeGithubOauthToken(): string { return `gho_${'a'.repeat(36)}`; }
// ```
fn fake_github_oauth_token() -> String {
    // What:     `format!("gho_{}", "a".repeat(36))` builds an owned
    //           `String` (sibling `&str` cannot be assembled at run
    //           time) from the prefix plus thirty-six repeated `a`s.
    //           No trailing `;` -- tail expression, so it is the
    //           function's return value.
    // Why:      36 alphanumerics after `gho_` is exactly the shape the
    //           baseline rule matches.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `gho_${'a'.repeat(36)}`;
    // ```
    return format!("gho_{}", "a".repeat(36))
}

// What:     The builtin `curl-auth-user` rule accepts short and long user
//           options at line start or after horizontal whitespace. The
//           fixtures assemble option and credential fragments at runtime
//           so the repository scanner does not flag its own test source.
// Why:      Same-line curl commands, indented or column-zero continuation
//           lines, and deliberately broad non-curl option lines must retain
//           credential detection.
//
// In TS you'd write (pseudocode):
// ```ts
// test('curl auth user accepts separate option tokens', () => { ... });
// ```
#[test]
fn curl_auth_user_accepts_separate_option_tokens() {
    let dir = unique_tmp("curl-auth-user-separate-options");
    let target = dir.join("options.txt");
    let credential = ["alice", "secret"].join(":");
    let content = [
        format!("-u {credential}"),
        format!("curl -u {credential}"),
        format!("curl\t--user={credential}"),
        format!("  -u {credential} \\"),
        format!("curl \\\n-u {credential}"),
        format!("other -u {credential}"),
    ]
    .join("\n");
    fs::write(&target, format!("{content}\n")).expect("write option fixtures");

    let output = Command::configured(BIN)
        .args(["--builtin-rules", "options.txt"])
        .current_dir(&dir)
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "separate credential options must produce findings; stderr: {stderr}",
    );
    let findings: Vec<&str> = stderr
        .lines()
        .filter(|line| return line.contains("rule=curl-auth-user"))
        .collect();
    assert_eq!(
        findings.len(),
        6,
        "expected one curl-auth-user finding per option line: {stderr}",
    );
    for line in [1, 2, 3, 4, 6, 7] {
        assert!(
            findings
                .iter()
                .any(|finding| return finding.contains(&format!("options.txt:{line} "))),
            "missing curl-auth-user finding for line {line}: {stderr}",
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

// What:     The builtin `curl-auth-user` rule refuses `-u` text embedded
//           inside a larger word or package-path task selector.
// Why:      Option-like substrings must not become username-password pairs
//           merely because a later colon-delimited task name has a valid
//           credential shape.
//
// In TS you'd write (pseudocode):
// ```ts
// test('curl auth user refuses embedded option substrings', () => { ... });
// ```
#[test]
fn curl_auth_user_refuses_embedded_option_substrings() {
    let dir = unique_tmp("curl-auth-user-embedded-substrings");
    let target = dir.join("task-paths.txt");
    let content = [
        "mise run //package/pi-plugin/ask-user-question:build",
        "mise run //package/pi-plugin/ask-user-question:lint",
        "prefix-user-question:build",
    ]
    .join("\n");
    fs::write(&target, format!("{content}\n")).expect("write embedded substring fixtures");

    let output = Command::configured(BIN)
        .args(["--builtin-rules", "task-paths.txt"])
        .current_dir(&dir)
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("spawn binary");
    assert!(
        output.status.success(),
        "embedded option substrings must scan cleanly; stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     `--builtin-rules` with NO rules file anywhere (empty cwd, no
//           env var) must scan with the embedded baseline alone: a file
//           containing a github-oauth-shaped token exits 1 with a
//           redacted hit line.
// Why:      Opting into the baseline IS configuration; requiring a rules
//           file on top would make the zero-file quick-start impossible.
#[test]
fn builtin_rules_flag_scans_with_baseline_alone_when_default_absent() {
    let dir = unique_tmp("builtin-alone");
    let target = dir.join("leaky.txt");
    fs::write(&target, format!("{}\n", fake_github_oauth_token())).expect("write target");
    // What:     `.current_dir(&dir)` runs the child in the tmp dir (so
    //           the implicit `./forbidden-strings.local.txt` default
    //           cannot resolve) and `.env_remove(...)` strips the env
    //           fallback the developer's shell may carry.
    // Why:      The test must exercise the "no rules file resolves at
    //           all" branch regardless of the machine it runs on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // spawnSync(BIN, ['--builtin-rules', 'leaky.txt'], { cwd: dir, env: cleaned });
    // ```
    let output = Command::configured(BIN)
        .args(["--builtin-rules", "leaky.txt"])
        .current_dir(&dir)
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 1,
        "baseline-only scan must exit 1 on a hit; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("rule="),
        "stderr must carry a redacted hit line; got: {}",
        stderr,
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     `--builtin-rules` combined with a user rules file must fire
//           BOTH rule sources, and the user's first rule keeps id 0
//           (`rule=0`) because the runtime set takes ids `0..user_len`
//           and the baseline is offset past it.
// Why:      Stable `rule=N` output is part of the scanner's contract;
//           the migration moves from source line numbers to 0-based
//           engine rule ids, but user rules stay independent of the
//           baseline so suppression workflows keyed on the user's own
//           numbering do not shift when the flag is on.
#[test]
fn builtin_rules_flag_appends_after_user_rules() {
    let dir = unique_tmp("builtin-append");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("leaky.txt");
    fs::write(
        &target,
        format!("SECRET_NEEDLE_XYZ_LONG_ENOUGH\n{}\n", fake_github_oauth_token()),
    )
    .expect("write target");
    let output = Command::configured(BIN)
        .args(["--builtin-rules", "--rules"])
        .arg(&rules)
        .arg(&target)
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 1,
        "combined scan must exit 1 on hits; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("rule=0"),
        "user rule must keep engine id 0; got: {}",
        stderr,
    );
    // What:     `stderr.lines().filter(|line| line.contains("rule=")).count()`
    //           counts hit lines. `.lines()` iterates over `\n`-separated
    //           slices; `.filter(...)` keeps hit lines; `.count()`
    //           consumes the iterator into a `usize` (the pointer-width
    //           unsigned integer every std length API uses; siblings
    //           `u32`/`u64` would force casts).
    // Why:      One hit from the user rule plus one from the baseline
    //           proves both sources were active in a single scan.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hits = stderr.split('\n').filter((l) => l.includes('rule=')).length;
    // ```
    let hits = stderr.lines().filter(|line| return line.contains("rule=")).count();
    assert!(
        hits >= 2,
        "expected hits from both the user rule and the baseline; got {} in: {}",
        hits, stderr,
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     `--builtin-rules` must NOT rescue an explicitly named missing
//           rules file: `--rules <missing>` still exits 2.
// Why:      An explicit path that cannot be read is a wiring failure;
//           silently scanning with only the baseline would hide the
//           user's own rules from the scan (a false-clean result).
#[test]
fn builtin_rules_flag_with_explicit_missing_rules_still_errors() {
    let dir = unique_tmp("builtin-explicit-missing");
    let rules = dir.join("does-not-exist.txt");
    let output = Command::configured(BIN)
        .args(["--builtin-rules", "--rules"])
        .arg(&rules)
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "explicit missing rules file must exit 2 even with --builtin-rules; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("forbidden-strings:"),
        "stderr must carry the program-prefixed error; got: {}",
        stderr,
    );
    let _ = fs::remove_dir_all(&dir);
}

// What:     Without `--builtin-rules`, the zero-config invocation (empty
//           cwd, no env var, no flag) must keep erroring with exit 2
//           exactly as before the flag existed.
// Why:      The embedded baseline must be pure opt-in: existing users of
//           the published CLI see byte-identical behavior unless they
//           pass the new flag.
#[test]
fn no_builtin_flag_and_no_rules_file_errors_unchanged() {
    let dir = unique_tmp("no-builtin-no-rules");
    let target = dir.join("leaky.txt");
    fs::write(&target, format!("{}\n", fake_github_oauth_token())).expect("write target");
    let output = Command::configured(BIN)
        .arg("leaky.txt")
        .current_dir(&dir)
        .env_remove("FORBIDDEN_STRINGS_RULES")
        .output()
        .expect("spawn binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "flagless zero-config run must exit 2; got {}.\nstderr: {}",
        code, stderr,
    );
    assert!(
        stderr.contains("read rules"),
        "stderr must carry the read-rules error; got: {}",
        stderr,
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Lists files with requested name under bounded cache hierarchy.
fn files_named(root: &std::path::Path, wanted: &str) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut matches = Vec::new();
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().and_then(|name| return name.to_str()) == Some(wanted) {
                matches.push(path);
            }
        }
    }
    return matches
}

/// Explicit compile operation creates private artifact and emits no success output.
#[test]
fn compile_rules_subcommand_creates_silent_artifact() {
    let dir = unique_tmp("compile-rules-command");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");

    let output = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert!(
        output.status.success(),
        "compile-rules must succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(output.stdout.is_empty(), "compile-rules success stdout must stay empty");
    assert!(output.stderr.is_empty(), "compile-rules success stderr must stay empty");
    assert_eq!(files_named(&cache, "rules.bin").len(), 1);

    let repeated = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert!(repeated.status.success(), "repeated compile-rules must reuse artifact");
    assert!(repeated.stdout.is_empty());
    assert!(repeated.stderr.is_empty());
    let _ = fs::remove_dir_all(dir);
}

/// First scan warns and repairs;
///  repeated scan reuses cache with identical finding.
#[test]
fn scan_repairs_missing_cache_then_reuses_artifact() {
    let dir = unique_tmp("scan-cache-repair");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let run = || {
        return ProcessCommand::new(BIN)
            .args(["--rules"])
            .arg(&rules)
            .arg(&target)
            .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
            .output()
            .expect("spawn binary")
    };
    let first = run();
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    assert_eq!(first.status.code(), Some(1));
    assert!(first_stderr.contains("\"reason\":\"missing\""));
    assert!(first_stderr.contains("target.txt:1 rule=0"));
    assert_eq!(files_named(&cache, "rules.bin").len(), 1);

    let second = run();
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert_eq!(second.status.code(), Some(1));
    assert!(!second_stderr.contains("cache-warning"));
    assert!(second_stderr.contains("target.txt:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}

/// Clean first scan still emits missing-cache warning and exits successfully.
#[test]
fn clean_first_scan_reports_cache_warning_on_stderr() {
    let dir = unique_tmp("clean-cache-warning");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "clean\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"reason\":\"missing\""));
    let _ = fs::remove_dir_all(dir);
}

/// Relative explicit cache override is configuration error rather than cwd path.
#[test]
fn relative_cache_override_exits_with_config_error() {
    let dir = unique_tmp("relative-cache-root");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "clean\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", "relative/cache")
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("FORBIDDEN_STRINGS_CACHE_DIR must be an absolute path"),
    );
    let _ = fs::remove_dir_all(dir);
}

/// Double-dash preserves scan access to positional file named like subcommand.
#[test]
fn double_dash_scans_file_named_compile_rules() {
    let dir = unique_tmp("compile-rules-positional");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(dir.join("compile-rules"), "ALPHA_LITERAL_LONG\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .current_dir(&dir)
        .args(["--rules", "rules.txt", "--", "compile-rules"])
        .env("FORBIDDEN_STRINGS_CACHE_DIR", dir.join("cache"))
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("compile-rules:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}

/// Compile operation rejects missing required rules argument as usage error.
#[test]
fn compile_rules_subcommand_requires_rules_path() {
    let output = ProcessCommand::new(BIN)
        .arg("compile-rules")
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--rules"));
}

/// Compile operation fails closed on invalid text and publishes no artifact.
#[test]
fn compile_rules_subcommand_rejects_invalid_rules() {
    let dir = unique_tmp("compile-rules-invalid");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "/ALPHA*/\n").expect("write invalid rules");
    let output = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("rule 0"));
    assert!(files_named(&cache, "rules.bin").is_empty());
    let _ = fs::remove_dir_all(dir);
}

/// Corrupt artifact warns,
///  recompiles,
///  replaces cache,
///  and preserves finding.
#[test]
fn corrupt_artifact_is_repaired_without_false_clean() {
    let dir = unique_tmp("corrupt-cache-repair");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let compiled = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn compiler");
    assert!(compiled.status.success());
    let artifacts = files_named(&cache, "rules.bin");
    assert_eq!(artifacts.len(), 1);
    fs::write(&artifacts[0], b"corrupt").expect("corrupt artifact");

    let repaired = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&repaired.stderr);
    assert_eq!(repaired.status.code(), Some(1));
    assert!(stderr.contains("\"reason\":\"invalid\""));
    assert!(stderr.contains("target.txt:1 rule=0"));

    let hit = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn scanner");
    assert!(!String::from_utf8_lossy(&hit.stderr).contains("cache-warning"));
    let _ = fs::remove_dir_all(dir);
}

/// Changed source content selects a second immutable content-addressed slot.
#[test]
fn changed_rules_content_selects_new_cache_slot() {
    let dir = unique_tmp("cache-content-change");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write first rules");
    fs::write(&target, "BETA_LITERAL_LONG\n").expect("write target");
    assert!(
        ProcessCommand::new(BIN)
            .args(["compile-rules", "--rules"])
            .arg(&rules)
            .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
            .status()
            .expect("spawn compiler")
            .success(),
    );

    fs::write(&rules, "BETA_LITERAL_LONG\n").expect("write changed rules");
    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("\"reason\":\"missing\""));
    assert!(stderr.contains("target.txt:1 rule=0"));
    assert_eq!(files_named(&cache, "rules.bin").len(), 2);
    let _ = fs::remove_dir_all(dir);
}

/// Failed publication warns but retains correct compiled in-memory finding.
#[test]
fn cache_write_failure_keeps_scan_correct() {
    let dir = unique_tmp("cache-write-failure");
    let cache_root_file = dir.join("cache-root-file");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&cache_root_file, "not a directory").expect("write root blocker");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache_root_file)
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    // The read-side reason for a root blocked by a regular file is the platform's:
    // `unreadable` on Unix and `missing` on Windows. `format!` builds the owned JSON
    // fragment once; `.as_str()` lends it to `contains` without copying.
    let blocked_reason: String = format!("\"reason\":\"{}\"", blocked_root::blocked_root_reason());
    assert!(stderr.contains(blocked_reason.as_str()), "blocked-root reason missing; stderr: {stderr}");
    assert!(stderr.contains("\"reason\":\"write-failed\""));
    assert!(stderr.contains("target.txt:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}

/// Missing native cache environment warns and scans from authoritative text.
#[test]
fn unavailable_native_cache_root_keeps_scan_correct() {
    let dir = unique_tmp("cache-root-unavailable");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env_remove("FORBIDDEN_STRINGS_CACHE_DIR")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("HOME")
        .env_remove("LOCALAPPDATA")
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("\"reason\":\"cache-root-unavailable\""));
    assert!(stderr.contains("target.txt:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}
