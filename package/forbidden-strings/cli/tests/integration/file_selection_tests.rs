//! What:
//!  Which paths one run scans and which it steps over:
//!  an explicit positional
//!  argument always scans,
//!  the scanner's own ruleset files at the working directory never do,
//!  `--all` skips every listed rules file,
//!  and `--` ends option parsing so a file named like a subcommand still
//!  scans.
//! Why:
//!  A skip keyed on a basename anywhere in the tree hid real findings
//!  (BUGs 6 and 11),
//!  and the opposite mistake makes a rules file match every rule body it
//!  declares.
//!  Both mistakes are silent,
//!  so each selection rule is pinned at the binary boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // file-selection.test.ts: positional args, the cwd skip, and --all
//! ```

use std::fs;
use std::process::Command as ProcessCommand;

use crate::{
    BIN,
    Command,
    unique_tmp,
};

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
