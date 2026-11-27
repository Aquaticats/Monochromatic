//! What:
//!  Where the rules list comes from and how a bad invocation is refused:
//!  `--rules` wins over `FORBIDDEN_STRINGS_RULES`,
//!  a repeated flag keeps its last value,
//!  a hyphen-prefixed path is still a value,
//!  and a missing file or malformed flag exits 2.
//! Why:
//!  CI layers a default flag under a caller's override,
//!  so the precedence and the last-value rule are load-bearing;
//!  exit 2 is the documented channel for "the scanner could not run",
//!  distinct from exit 1 ("ran,
//!  found violations").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // rules-flag.test.ts: flag over env, last flag wins, usage errors exit 2
//! ```

use std::fs;

use crate::{
    BIN,
    Command,
    unique_tmp,
};

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
