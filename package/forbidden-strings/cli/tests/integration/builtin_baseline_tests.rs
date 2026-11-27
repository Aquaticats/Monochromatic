//! What:
//!  The embedded betterleaks baseline that `--builtin-rules` turns on,
//!  and the `curl-auth-user` rule it carries:
//!  the baseline alone can be the whole ruleset,
//!  appended rules keep the user's own ids,
//!  and an explicitly named missing rules file still fails closed.
//! Why:
//!  Existing invocations must stay byte-identical unless the flag is passed,
//!  and a user rule keyed on `rule=0` must not shift when the baseline joins.
//!  The fixtures build their credential shapes at run time,
//!  so the repository's own scan of this source finds no violation in it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // builtin-baseline.test.ts: opt-in only, ids stable, explicit miss exits 2
//! ```

use std::fs;

use crate::{
    BIN,
    Command,
    unique_tmp,
};

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
    return format!("gho_{}", "a".repeat(36));
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
        format!(
            "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n{}\n",
            fake_github_oauth_token()
        ),
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
    let hits = stderr
        .lines()
        .filter(|line| return line.contains("rule="))
        .count();
    assert!(
        hits >= 2,
        "expected hits from both the user rule and the baseline; got {} in: {}",
        hits,
        stderr,
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
