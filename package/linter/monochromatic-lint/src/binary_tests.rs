//! What:
//!  Black-box controls that run the built `monochromatic-lint` executable.
//! Why:
//!  Unit tests passing is not verification of a command-line program.
//!  These tests start the
//! real binary as a child process in disposable directories and assert on its actual exit status,
//! standard output and standard error bytes,
//!  and on what it left on disk.
//! They run inside the bounded,
//!  mount-free,
//!  network-disabled test container with the other tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { status, stdout, stderr } = spawnSync(binary, args, { cwd, input });
//! ```

/// Import child-process control,
///  stream plumbing and native paths.
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

/// The path of the executable Cargo built for this test run.
const BINARY: &str = env!("CARGO_BIN_EXE_monochromatic-lint");

/// The configuration file name the executable looks up.
const CONFIG: &str = "monochromatic-lint.config.jsonc";

/// A configuration selecting the syntax-only Rust rules and the Markdown rules that need no repository.
const RULES: &str = r#"[
  { "files": ["**/*.rs"], "ignores": ["**/*.md/**", "**/*.mdx/**"], "rules": {
    "rust/max-lines": { "severity": "error", "max": 300 },
    "rust/require-rustdoc": { "severity": "error" },
    "rust/tests-in-sidecars": { "severity": "error" }
  } },
  { "files": ["**/*.md", "**/*.mdx"], "ignores": ["**/*.rs/**"], "rules": {
    "markdown/heading-increment": { "severity": "error" },
    "markdown/single-h1": { "severity": "error" },
    "markdown/no-trailing-punctuation": { "severity": "error" },
    "markdown/fenced-code-language": { "severity": "error" },
    "markdown/no-pipe-tables": { "severity": "error" }
  } }
]"#;

/// A process-wide sequence for unique fixture directory names.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A directory this test created and removes when it goes out of scope.
struct Fixture {
    /// Root created exclusively for one test.
    path: PathBuf,
}

/// Create and own one disposable directory.
impl Fixture {
    /// Exclusively create a fresh directory under the system temporary directory.
    fn new() -> Fixture {
        let sequence: u64 = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path: PathBuf = std::env::temp_dir().join(format!(
            "monochromatic-lint-binary-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("exclusively create fixture directory");
        // Resolve symbolic links in the temporary root so reported absolute paths compare exactly.
        return Fixture {
            path: std::fs::canonicalize(&path).expect("resolve fixture directory"),
        };
    }

    /// Write one file below the root,
    ///  creating its parent directories.
    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path: PathBuf = self.path.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture file has a parent"))
            .expect("fixture directories");
        std::fs::write(&path, contents).expect("fixture file");
        return path;
    }

    /// Read one file below the root as text.
    fn read(&self, relative: &str) -> String {
        return std::fs::read_to_string(self.path.join(relative)).expect("fixture text");
    }
}

/// Remove the directory this fixture created,
///  including after a failed assertion.
impl Drop for Fixture {
    /// Delete only the path this instance created.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("remove fixture directory");
    }
}

/// What one invocation produced.
struct Run {
    /// Exit status;
    ///  a signal death has none and fails the test.
    status: i32,
    /// Standard output as text.
    stdout: String,
    /// Standard error as text.
    stderr: String,
}

/// Run the executable in a directory with arguments and standard input,
///  and wait for it.
fn run(cwd: &Path, arguments: &[&str], input: &[u8]) -> Run {
    let mut child: Child = Command::new(BINARY)
        .args(arguments)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable");
    // Dropping the handle after writing closes the pipe, so the child sees end of input.
    let mut stdin = child.stdin.take().expect("piped standard input");
    stdin.write_all(input).expect("write standard input");
    drop(stdin);
    let output: Output = child.wait_with_output().expect("wait for the executable");
    return Run {
        status: output
            .status
            .code()
            .expect("the executable exits; it is not killed by a signal"),
        stdout: String::from_utf8(output.stdout).expect("standard output is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("standard error is UTF-8"),
    };
}

/// Decode JSONL:
///  every non-empty line must be one JSON object.
fn records(text: &str) -> Vec<serde_json::Value> {
    let mut values: Vec<serde_json::Value> = Vec::<serde_json::Value>::new();
    for line in text.lines() {
        let value: serde_json::Value =
            serde_json::from_str::<serde_json::Value>(line).expect("one JSON object per line");
        assert!(value.is_object(), "{line}");
        values.push(value);
    }
    return values;
}

/// `filename code line column` for every record,
///  in output order.
fn located(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    for record in records(text) {
        found.push(format!(
            "{} {} {} {}",
            record["filename"].as_str().expect("filename"),
            record["code"].as_str().expect("code"),
            record["labels"][0]["span"]["line"],
            record["labels"][0]["span"]["column"]
        ));
    }
    return found;
}

/// The sorted names of a directory's entries.
fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::<String>::new();
    for entry in std::fs::read_dir(directory).expect("readable directory") {
        names.push(
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    return names;
}

/// Help and version are parser outcomes with status 0;
///  usage errors are status 2 with nothing on standard output.
#[test]
fn help_version_and_usage_errors_use_the_documented_statuses() {
    let fixture: Fixture = Fixture::new();
    let help: Run = run(&fixture.path, &["--help"], b"");
    assert_eq!(help.status, 0);
    assert!(
        help.stdout.starts_with(
            "Lint Rust, Markdown and MDX with repository-owned policies.\n\nUsage: monochromatic-lint [OPTIONS] [PATH]..."
        ),
        "{}",
        help.stdout
    );
    for flag in [
        "--config",
        "--fix",
        "--stdin",
        "--stdin-filename",
        "--max-warnings",
        "--quiet",
        "--silent",
        "--print-config",
        "--init",
        "--rules",
        "--concurrency",
        "--ignore-pattern",
        "--ignore-path",
        "--no-ignore",
        "--no-error-on-unmatched-pattern",
        "--debug",
    ] {
        assert!(help.stdout.contains(flag), "{flag}");
    }
    assert_eq!(help.stderr, "");
    // Read the version from the manifest so a release bump cannot leave this assertion stale.
    let expected_version: String = format!("monochromatic-lint {}\n", env!("CARGO_PKG_VERSION"));
    let version: Run = run(&fixture.path, &["--version"], b"");
    assert_eq!(
        (
            version.status,
            version.stdout.as_str(),
            version.stderr.as_str()
        ),
        (0, expected_version.as_str(), "")
    );
    for arguments in [
        vec!["--bogus"],
        vec!["--stdin"],
        vec!["--stdin-filename", "a.md"],
        vec!["--stdin", "--stdin-filename", "a.md", "extra.md"],
        vec!["--concurrency", "0"],
        vec!["--max-warnings", "-1"],
        vec!["--rule", "markdown/single-h1"],
        vec!["--fix", "--rules"],
        vec!["--init", "--rules"],
    ] {
        let usage: Run = run(&fixture.path, arguments.as_slice(), b"");
        assert_eq!(usage.status, 2, "{arguments:?}");
        assert_eq!(usage.stdout, "", "{arguments:?}");
        assert!(
            usage.stderr.starts_with("error: "),
            "{arguments:?}: {}",
            usage.stderr
        );
    }
    assert!(entries(&fixture.path).is_empty());
}

/// Status 0 prints nothing,
///  status 1 prints JSONL on standard output only,
///  status 2 explains itself on standard error.
#[test]
fn exit_statuses_and_streams_follow_the_contract() {
    let fixture: Fixture = Fixture::new();
    fixture.write("a.md", "# Title.\n");
    let unconfigured: Run = run(&fixture.path, &[], b"");
    assert_eq!((unconfigured.status, unconfigured.stdout.as_str()), (2, ""));
    assert!(
        unconfigured
            .stderr
            .starts_with("monochromatic-lint: No monochromatic-lint.config.jsonc was found"),
        "{}",
        unconfigured.stderr
    );
    assert_eq!(unconfigured.stderr.lines().count(), 1);
    fixture.write(CONFIG, RULES);
    let findings: Run = run(&fixture.path, &[], b"");
    assert_eq!((findings.status, findings.stderr.as_str()), (1, ""));
    assert_eq!(
        findings.stdout,
        "{\"message\":\"Heading ends with punctuation; remove the trailing punctuation.\",\"code\":\"markdown/no-trailing-punctuation\",\"severity\":\"error\",\"causes\":[],\"filename\":\"a.md\",\"labels\":[{\"span\":{\"offset\":0,\"length\":8,\"line\":1,\"column\":1}}],\"related\":[]}\n"
    );
    fixture.write("a.md", "# Title\n");
    let clean: Run = run(&fixture.path, &[], b"");
    assert_eq!(
        (clean.status, clean.stdout.as_str(), clean.stderr.as_str()),
        (0, "", "")
    );
    fixture.write("broken.mdx", "<A>\ntext\n</B>\n");
    let incomplete: Run = run(&fixture.path, &[], b"");
    assert_eq!((incomplete.status, incomplete.stderr.as_str()), (2, ""));
    assert_eq!(located(incomplete.stdout.as_str()).len(), 1);
    assert_eq!(
        records(incomplete.stdout.as_str())[0]["code"],
        "core/processing-failure"
    );
    assert_eq!(
        records(incomplete.stdout.as_str())[0]["filename"],
        "broken.mdx"
    );
    fixture.write(CONFIG, "[{ \"files\": 3 }]");
    let malformed: Run = run(&fixture.path, &["--fix"], b"");
    assert_eq!((malformed.status, malformed.stdout.as_str()), (2, ""));
    assert!(
        malformed
            .stderr
            .starts_with("monochromatic-lint: Configuration "),
        "{}",
        malformed.stderr
    );
    assert!(malformed.stderr.contains(CONFIG), "{}", malformed.stderr);
    let missing: Run = run(&fixture.path, &["absent.md"], b"");
    assert_eq!(missing.status, 2);
    let warned: Fixture = Fixture::new();
    warned.write(CONFIG, r#"[{ "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "warn" } } }]"#);
    warned.write("a.md", "# Title.\n");
    assert_eq!(run(&warned.path, &[], b"").status, 0);
    assert_eq!(run(&warned.path, &["--max-warnings", "0"], b"").status, 1);
    let quiet: Run = run(&warned.path, &["--max-warnings", "0", "--quiet"], b"");
    assert_eq!(
        (quiet.status, quiet.stdout.as_str(), quiet.stderr.as_str()),
        (1, "", "")
    );
}

/// The walker reads ignore files,
///  skips Git metadata and `node_modules`,
///  enters hidden directories and picks only supported extensions.
#[test]
fn walking_selects_supported_files_and_honours_ignores() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    std::fs::create_dir(fixture.path.join(".git")).expect("repository marker");
    fixture.write(".git/info/exclude", "excluded-by-info/\n");
    fixture.write(".git/a.md", "# Title.\n");
    fixture.write(".gitignore", "ignored/\n*.generated.md\n");
    fixture.write("nested/.gitignore", "local-only.md\n");
    fixture.write(".ignore", "dot-ignored/\n");
    for file in [
        "a.md",
        "b.mdx",
        "c.rs",
        "d.txt",
        "e.MD",
        ".hidden/a.md",
        "ignored/a.md",
        "dot-ignored/a.md",
        "excluded-by-info/a.md",
        "node_modules/pkg/a.md",
        "nested/a.md",
        "nested/local-only.md",
        "nested/deep/x.generated.md",
        "extra/a.md",
    ] {
        fixture.write(file, "# Title.\n");
    }
    fixture.write(
        "c.rs",
        "//! File.\n\n/// Item.\nfn item() {}\n\nfn undocumented() {}\n",
    );
    let default: Run = run(&fixture.path, &[], b"");
    assert_eq!((default.status, default.stderr.as_str()), (1, ""));
    assert_eq!(
        located(default.stdout.as_str()),
        [
            ".hidden/a.md markdown/no-trailing-punctuation 1 1",
            "a.md markdown/no-trailing-punctuation 1 1",
            "b.mdx markdown/no-trailing-punctuation 1 1",
            "c.rs rust/require-rustdoc 6 1",
            "extra/a.md markdown/no-trailing-punctuation 1 1",
            "nested/a.md markdown/no-trailing-punctuation 1 1",
        ]
    );
    fixture.write("more.ignore", "extra/\n");
    let flagged: Run = run(
        &fixture.path,
        &[
            "--ignore-path",
            "more.ignore",
            "--ignore-pattern",
            "nested/**",
            "--ignore-pattern",
            "*.mdx",
        ],
        b"",
    );
    assert_eq!(
        located(flagged.stdout.as_str()),
        [
            ".hidden/a.md markdown/no-trailing-punctuation 1 1",
            "a.md markdown/no-trailing-punctuation 1 1",
            "c.rs rust/require-rustdoc 6 1",
        ]
    );
    let unignored: Run = run(&fixture.path, &["--no-ignore", "--quiet"], b"");
    assert_eq!(records(unignored.stdout.as_str()).len(), 12);
    // Path arguments: a directory is walked, a named file bypasses traversal ignores, a glob is expanded.
    let named: Run = run(&fixture.path, &["nested", "ignored/a.md", "ex*/*.md"], b"");
    assert_eq!(
        located(named.stdout.as_str()),
        [
            "extra/a.md markdown/no-trailing-punctuation 1 1",
            "ignored/a.md markdown/no-trailing-punctuation 1 1",
            "nested/a.md markdown/no-trailing-punctuation 1 1",
        ]
    );
    let unmatched: Run = run(&fixture.path, &["no-such-*/*.md"], b"");
    assert_eq!((unmatched.status, unmatched.stdout.as_str()), (2, ""));
    let allowed: Run = run(
        &fixture.path,
        &["--no-error-on-unmatched-pattern", "no-such-*/*.md"],
        b"",
    );
    assert_eq!(
        (
            allowed.status,
            allowed.stdout.as_str(),
            allowed.stderr.as_str()
        ),
        (0, "", "")
    );
}

/// The nearest configuration is used alone,
///  `--config` resolves against the working directory,
///  and blocks merge in order.
#[test]
fn configuration_lookup_and_merging_follow_the_design() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        CONFIG,
        r#"// Comments and trailing commas are accepted.
[
  { "ignores": ["vendor/"], },
  { "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 2 }, "rust/require-rustdoc": { "severity": "error" } } },
  { "files": ["src/**/*.rs"], "rules": { "rust/max-lines": { "severity": "warn" } } },
  { "files": ["**/*.md"], "rules": { "markdown/lfs-image-url": { "severity": "off", "exclude": ["a/"] } } },
  { "files": ["doc/**"], "rules": { "markdown/lfs-image-url": { "severity": "off", "exclude": ["b/"] } } },
]"#,
    );
    fixture.write(
        &format!("nested/{CONFIG}"),
        r#"[{ "files": ["*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "warn" } } }]"#,
    );
    // Each file holds one documented stray unit test on its eleventh line, which only the sidecar rule reports.
    let three_lines: &str = "//! File.\n\n/// One.\nfn one() {}\n/// Two.\nfn two() {}\n/// Three.\nfn three() {}\n/// Stray.\n#[test]\nfn stray() {}\n";
    for file in [
        "src/a.rs",
        "top.rs",
        "vendor/a.rs",
        "nested/a.rs",
        "nested/deep/a.rs",
    ] {
        fixture.write(file, three_lines);
    }
    let output: Run = run(&fixture.path, &[], b"");
    assert_eq!((output.status, output.stderr.as_str()), (1, ""));
    let lines: Vec<serde_json::Value> = records(output.stdout.as_str());
    let mut summary: Vec<String> = Vec::<String>::new();
    for line in &lines {
        summary.push(format!(
            "{} {} {}",
            line["filename"].as_str().expect("filename"),
            line["code"].as_str().expect("code"),
            line["severity"].as_str().expect("severity")
        ));
    }
    // The nested configuration governs its subtree alone: no rustdoc or line budget there, and its
    // single-star pattern does not reach `nested/deep`. The later `src` block changes only the severity.
    // The outer configuration has no sidecar rule, so the stray test is reported only below `nested`.
    assert_eq!(
        summary,
        [
            "nested/a.rs rust/tests-in-sidecars warn",
            "src/a.rs rust/max-lines warn",
            "top.rs rust/max-lines error",
        ]
    );
    assert!(
        lines[1]["message"]
            .as_str()
            .expect("message")
            .contains("limit is 2"),
        "{}",
        lines[1]["message"]
    );
    // Arrays from every applying block concatenate; scalars take the last value.
    let printed: Run = run(&fixture.path, &["--print-config", "doc/a.md"], b"");
    assert_eq!((printed.status, printed.stderr.as_str()), (0, ""));
    let compact: String = printed
        .stdout
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("");
    assert!(
        compact.contains(r#""state":"configured""#),
        "{}",
        printed.stdout
    );
    assert!(
        compact.contains(r#""exclude":["a/","b/"]"#),
        "{}",
        printed.stdout
    );
    let merged: Run = run(&fixture.path, &["--print-config", "src/a.rs"], b"");
    let merged_compact: String = merged
        .stdout
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("");
    assert!(
        merged_compact.contains(r#""rust/max-lines":{"severity":"warn","max":2}"#),
        "{}",
        merged.stdout
    );
    for (file, state) in [
        ("vendor/a.rs", "ignored"),
        ("nested/deep/a.rs", "unconfigured"),
        ("a.txt", "unconfigured"),
    ] {
        let state_output: Run = run(&fixture.path, &["--print-config", file], b"");
        assert!(
            state_output
                .stdout
                .contains(format!("\"{state}\"").as_str()),
            "{file}: {}",
            state_output.stdout
        );
    }
    // An explicit configuration outside the tree replaces lookup and matches working-directory paths.
    let outside: Fixture = Fixture::new();
    let explicit: PathBuf = outside.write("only.jsonc", r#"[{ "files": ["nested/**/*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "error" } } }]"#);
    let explicit_text: String = explicit.to_string_lossy().into_owned();
    let overridden: Run = run(&fixture.path, &["--config", explicit_text.as_str()], b"");
    assert_eq!(
        located(overridden.stdout.as_str()),
        [
            "nested/a.rs rust/tests-in-sidecars 11 4",
            "nested/deep/a.rs rust/tests-in-sidecars 11 4",
        ]
    );
    // Running from a subdirectory still finds the ancestor configuration and names files relative to that directory.
    let below: Run = run(&fixture.path.join("src"), &[], b"");
    // The finding sits on the first code line beyond the budget of two.
    assert_eq!(located(below.stdout.as_str()), ["a.rs rust/max-lines 8 1"]);
}

/// A configuration that still selects the removed anonymous-function rule is a setup error on status 2,
///  naming the rule,
///  and lints nothing;
///  the same file without that rule runs normally.
#[test]
fn a_configuration_naming_the_removed_anonymous_function_rule_is_a_setup_error() {
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() { call(|| 1); }\n");
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let stale: Run = run(&fixture.path, &[], b"");
    assert_eq!((stale.status, stale.stdout.as_str()), (2, ""));
    assert!(
        stale
            .stderr
            .starts_with("monochromatic-lint: Configuration "),
        "{}",
        stale.stderr
    );
    assert!(
        stale
            .stderr
            .contains("Unknown built-in rule rust/no-anonymous-functions"),
        "{}",
        stale.stderr
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 300 } } }]"#,
    );
    let current: Run = run(&fixture.path, &[], b"");
    assert_eq!(
        (
            current.status,
            current.stdout.as_str(),
            current.stderr.as_str()
        ),
        (0, "", "")
    );
}

/// Every configuration rejection diagnostic names the running binary,
///  so which binary refused the configuration is readable from the failure itself.
/// A lockfile pinning exactly the running version measures no staleness,
///  so no hint appears.
#[test]
fn configuration_rejection_diagnostics_identify_the_running_binary() {
    // Read the version from the manifest so a release bump cannot leave this assertion stale.
    let identity: String = format!("(monochromatic-lint {})", env!("CARGO_PKG_VERSION"));
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() {}\n");
    fixture.write(
        "mise.lock",
        format!(
            "[[tools.\"cargo:monochromatic-lint\"]]\nversion = \"{}\"\n",
            env!("CARGO_PKG_VERSION")
        )
        .as_str(),
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let unknown: Run = run(&fixture.path, &[], b"");
    assert_eq!((unknown.status, unknown.stdout.as_str()), (2, ""));
    assert!(
        unknown
            .stderr
            .contains("Unknown built-in rule rust/no-anonymous-functions"),
        "{}",
        unknown.stderr
    );
    assert!(
        unknown.stderr.contains(identity.as_str()),
        "{}",
        unknown.stderr
    );
    assert!(
        !unknown.stderr.contains("may be stale"),
        "{}",
        unknown.stderr
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "warning" } } }]"#,
    );
    let severity: Run = run(&fixture.path, &[], b"");
    assert_eq!((severity.status, severity.stdout.as_str()), (2, ""));
    assert!(
        severity.stderr.contains("severity must be"),
        "{}",
        severity.stderr
    );
    assert!(
        severity.stderr.contains(identity.as_str()),
        "{}",
        severity.stderr
    );
    fixture.write(CONFIG, "[{");
    let malformed: Run = run(&fixture.path, &[], b"");
    assert_eq!((malformed.status, malformed.stdout.as_str()), (2, ""));
    assert!(
        malformed.stderr.contains("JSONC syntax error"),
        "{}",
        malformed.stderr
    );
    assert!(
        malformed.stderr.contains(identity.as_str()),
        "{}",
        malformed.stderr
    );
}

/// The unknown-rule error names a stale resolved binary only when the repository's lockfile
///  measures a pin newer than the running binary,
///  and the hint quotes that measurement.
#[test]
fn a_newer_repository_pin_adds_the_stale_binary_hint() {
    let major: u64 = env!("CARGO_PKG_VERSION")
        .split('.')
        .next()
        .expect("manifest version has a major component")
        .parse()
        .expect("major component is numeric");
    // One major version ahead of this build, so a release bump cannot make the control stale.
    let pin: String = format!("{}.0.0", major + 1);
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() {}\n");
    fixture.write(
        "mise.lock",
        format!("[[tools.\"cargo:monochromatic-lint\"]]\nversion = \"{pin}\"\n").as_str(),
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let stale: Run = run(&fixture.path, &[], b"");
    assert_eq!((stale.status, stale.stdout.as_str()), (2, ""));
    assert_eq!(
        stale.stderr,
        format!(
            "monochromatic-lint: Configuration {0}/{1}: Unknown built-in rule rust/no-anonymous-functions. (monochromatic-lint {2}) The resolved binary may be stale: {0}/mise.lock pins cargo:monochromatic-lint {pin}, newer than this binary.\n",
            fixture.path.display(),
            CONFIG,
            env!("CARGO_PKG_VERSION")
        )
    );
}

/// A repository pin older than the running binary measures no staleness,
///  so the hint stays out
/// even though the rule is still unknown.
#[test]
fn an_older_repository_pin_adds_no_stale_binary_hint() {
    assert_ne!(
        env!("CARGO_PKG_VERSION"),
        "0.0.0",
        "the control needs a strictly older pin"
    );
    let fixture: Fixture = Fixture::new();
    fixture.write("a.rs", "fn main() {}\n");
    fixture.write(
        "mise.lock",
        "[[tools.\"cargo:monochromatic-lint\"]]\nversion = \"0.0.0\"\n",
    );
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/no-anonymous-functions": { "severity": "off" } } }]"#,
    );
    let resolved: Run = run(&fixture.path, &[], b"");
    assert_eq!((resolved.status, resolved.stdout.as_str()), (2, ""));
    assert!(
        resolved
            .stderr
            .contains("Unknown built-in rule rust/no-anonymous-functions"),
        "{}",
        resolved.stderr
    );
    assert!(
        !resolved.stderr.contains("may be stale"),
        "{}",
        resolved.stderr
    );
}

/// `--fix` replaces files atomically,
///  keeps their permission bits,
///  leaves no temporary files,
///  and reports the remainder.
#[cfg(unix)]
#[test]
fn fixing_writes_atomically_and_keeps_the_file_mode() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    let fixable: PathBuf = fixture.write("doc/a.md", "# Title.\n\n```\ncode\n```\n\n# Second\n");
    let clean: PathBuf = fixture.write("doc/clean.md", "# Title\n");
    let unfixable: PathBuf = fixture.write("doc/b.md", "# One\n\n### Three\n");
    std::fs::set_permissions(&fixable, std::fs::Permissions::from_mode(0o640)).expect("chmod");
    std::fs::set_permissions(&clean, std::fs::Permissions::from_mode(0o444)).expect("chmod");
    std::fs::set_permissions(&unfixable, std::fs::Permissions::from_mode(0o600)).expect("chmod");
    let before_fixable: std::fs::Metadata = std::fs::metadata(&fixable).expect("metadata");
    let before_clean: std::fs::Metadata = std::fs::metadata(&clean).expect("metadata");
    let before_unfixable: std::fs::Metadata = std::fs::metadata(&unfixable).expect("metadata");
    let output: Run = run(&fixture.path, &["--fix"], b"");
    assert_eq!((output.status, output.stderr.as_str()), (1, ""));
    assert_eq!(
        located(output.stdout.as_str()),
        [
            "doc/a.md markdown/single-h1 7 1",
            "doc/b.md markdown/heading-increment 3 1",
        ]
    );
    assert_eq!(
        fixture.read("doc/a.md"),
        "# Title\n\n```text\ncode\n```\n\n# Second\n"
    );
    let after_fixable: std::fs::Metadata = std::fs::metadata(&fixable).expect("metadata");
    assert_eq!(after_fixable.permissions().mode() & 0o7777, 0o640);
    // A rename replaced the file: it is a different inode, never a truncated original.
    assert_ne!(after_fixable.ino(), before_fixable.ino());
    // Files with nothing to fix are not rewritten at all: same inode, same modification time, same mode.
    for (path, before) in [(&clean, &before_clean), (&unfixable, &before_unfixable)] {
        let after: std::fs::Metadata = std::fs::metadata(path).expect("metadata");
        assert_eq!(after.ino(), before.ino());
        assert_eq!(after.mtime_nsec(), before.mtime_nsec());
        assert_eq!(after.mtime(), before.mtime());
        assert_eq!(after.permissions().mode(), before.permissions().mode());
    }
    assert_eq!(fixture.read("doc/clean.md"), "# Title\n");
    assert_eq!(
        entries(&fixture.path.join("doc")),
        ["a.md", "b.md", "clean.md"]
    );
    // A second run is a fixed point: nothing is rewritten and the same findings remain.
    let settled: std::fs::Metadata = std::fs::metadata(&fixable).expect("metadata");
    let again: Run = run(&fixture.path, &["--fix"], b"");
    assert_eq!(again.stdout, output.stdout);
    assert_eq!(
        std::fs::metadata(&fixable).expect("metadata").ino(),
        settled.ino()
    );
    // A symbolic link is followed: the link stays a link and its target is fixed.
    let target: PathBuf = fixture.write("real/target.md", "# Linked.\n");
    std::os::unix::fs::symlink(&target, fixture.path.join("link.md")).expect("symlink");
    let linked: Run = run(&fixture.path, &["--fix", "link.md"], b"");
    assert_eq!((linked.status, linked.stdout.as_str()), (0, ""));
    assert!(
        std::fs::symlink_metadata(fixture.path.join("link.md"))
            .expect("link")
            .file_type()
            .is_symlink()
    );
    assert_eq!(fixture.read("real/target.md"), "# Linked\n");
    // A refused fix leaves the file's bytes and inode alone and exits 2.
    let refusal: Fixture = Fixture::new();
    refusal.write(CONFIG, r#"[{ "files": ["**/*.md"], "rules": { "markdown/link-image-reference-definitions": { "severity": "error" } } }]"#);
    let only_definition: PathBuf = refusal.write("a.md", "[unused]: https://example.com\n");
    let before_refusal: u64 = std::fs::metadata(&only_definition).expect("metadata").ino();
    let refused: Run = run(&refusal.path, &["--fix"], b"");
    assert_eq!((refused.status, refused.stderr.as_str()), (2, ""));
    let mut refused_codes: Vec<String> = Vec::<String>::new();
    for record in records(refused.stdout.as_str()) {
        refused_codes.push(String::from(record["code"].as_str().expect("code")));
    }
    assert_eq!(
        refused_codes,
        [
            "core/fix-refused",
            "markdown/link-image-reference-definitions"
        ]
    );
    assert_eq!(refusal.read("a.md"), "[unused]: https://example.com\n");
    assert_eq!(
        std::fs::metadata(&only_definition).expect("metadata").ino(),
        before_refusal
    );
    assert_eq!(entries(&refusal.path), ["a.md", CONFIG]);
}

/// Processor findings report the host file and host positions,
///  and a rustdoc fix is written into the host comment.
#[test]
fn processors_report_and_fix_at_host_positions() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        CONFIG,
        r#"[
          { "files": ["**/*.rs"], "ignores": ["**/*.md/**", "**/*.mdx/**"], "rules": { "rust/require-rustdoc": { "severity": "error" } } },
          { "files": ["**/*.rs/*.md"], "rules": { "markdown/fenced-code-language": { "severity": "error" }, "markdown/no-trailing-punctuation": { "severity": "error" } } },
          { "files": ["**/*.md/*.rs", "**/*.mdx/*.rs", "**/*.rs/*.md/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "warn" } } }
        ]"#,
    );
    let markdown: &str = "Intro 🚀.\r\n\r\n> ```rust,no_run\r\n> # let hidden = 1;\r\n> let value = \"🚀\"; fn helper() {}\r\n> ```\r\n";
    let rust: &str = "//! File.\n\n    /// # Heading.\n    ///\n    /// ```\n    /// //! Example.\n    /// let value = 1; fn helper() {}\n    /// ```\n    fn item() {}\n";
    fixture.write("doc/a.md", markdown);
    fixture.write(
        "doc/b.mdx",
        "<section>\n\n```rs\n//! Embedded.\nlet value = 2; fn helper() {}\n```\n\n</section>\n",
    );
    fixture.write("src/c.rs", rust);
    let output: Run = run(&fixture.path, &[], b"");
    assert_eq!((output.status, output.stderr.as_str()), (1, ""));
    assert_eq!(
        located(output.stdout.as_str()),
        [
            "doc/a.md rust/require-rustdoc 4 5",
            "doc/a.md rust/require-rustdoc 5 21",
            "doc/b.mdx rust/require-rustdoc 5 16",
            "src/c.rs markdown/no-trailing-punctuation 3 9",
            "src/c.rs markdown/fenced-code-language 5 9",
            "src/c.rs rust/require-rustdoc 7 24",
        ]
    );
    // Byte ranges index the host file exactly, across CRLF, a quote prefix and astral characters.
    let lines: Vec<serde_json::Value> = records(output.stdout.as_str());
    let markdown_item: &serde_json::Value = &lines[1]["labels"][0]["span"];
    let offset: usize = markdown_item["offset"].as_u64().expect("offset") as usize;
    let length: usize = markdown_item["length"].as_u64().expect("length") as usize;
    assert_eq!(&markdown[offset..offset + length], "fn helper() {}");
    let rust_item: &serde_json::Value = &lines[5]["labels"][0]["span"];
    let rust_offset: usize = rust_item["offset"].as_u64().expect("offset") as usize;
    let rust_length: usize = rust_item["length"].as_u64().expect("length") as usize;
    assert_eq!(
        &rust[rust_offset..rust_offset + rust_length],
        "fn helper() {}"
    );
    let heading: usize = lines[3]["labels"][0]["span"]["offset"]
        .as_u64()
        .expect("offset") as usize;
    assert!(
        rust[heading..].starts_with("# Heading."),
        "{}",
        &rust[heading..]
    );
    // Fixing rewrites only the host comment lines, keeping their indentation and prefix.
    let fixed: Run = run(&fixture.path, &["--fix"], b"");
    assert_eq!(fixed.status, 0);
    assert_eq!(
        fixture.read("src/c.rs"),
        "//! File.\n\n    /// # Heading\n    ///\n    /// ```rust\n    /// //! Example.\n    /// let value = 1; fn helper() {}\n    /// ```\n    fn item() {}\n"
    );
    assert_eq!(fixture.read("doc/a.md"), markdown);
    assert_eq!(
        located(fixed.stdout.as_str()),
        [
            "doc/a.md rust/require-rustdoc 4 5",
            "doc/a.md rust/require-rustdoc 5 21",
            "doc/b.mdx rust/require-rustdoc 5 16",
            "src/c.rs rust/require-rustdoc 7 24",
        ]
    );
}

/// Standard-input fixing prints exactly the fixed source on standard output and only JSONL on standard error.
#[test]
fn standard_input_fixing_separates_source_from_findings() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    let source: &str = "\u{feff}# Title.\r\n\r\n```\r\ncode 🚀\r\n```\r\n\r\n# Second\r\n";
    let fixed: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "doc/new.md", "--fix"],
        source.as_bytes(),
    );
    assert_eq!(fixed.status, 1);
    assert_eq!(
        fixed.stdout,
        "\u{feff}# Title\r\n\r\n```text\r\ncode 🚀\r\n```\r\n\r\n# Second\r\n"
    );
    assert_eq!(
        located(fixed.stderr.as_str()),
        ["doc/new.md markdown/single-h1 7 1"]
    );
    assert!(!fixture.path.join("doc").exists());
    let lint: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "doc/new.md"],
        source.as_bytes(),
    );
    assert_eq!((lint.status, lint.stderr.as_str()), (1, ""));
    assert_eq!(
        located(lint.stdout.as_str()),
        [
            "doc/new.md markdown/no-trailing-punctuation 1 1",
            "doc/new.md markdown/fenced-code-language 3 1",
            "doc/new.md markdown/single-h1 7 1",
        ]
    );
    let clean: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.rs", "--fix"],
        b"//! File.\n",
    );
    assert_eq!(
        (clean.status, clean.stdout.as_str(), clean.stderr.as_str()),
        (0, "//! File.\n", "")
    );
    let silent: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix", "--silent"],
        b"# Title.\n\n# Second\n",
    );
    assert_eq!(
        (
            silent.status,
            silent.stdout.as_str(),
            silent.stderr.as_str()
        ),
        (1, "# Title\n\n# Second\n", "")
    );
    // Setup failures print no source: a consumer must not write standard output on a non-zero status.
    let invalid: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix"],
        &[b'#', b' ', 0xff, b'\n'],
    );
    assert_eq!((invalid.status, invalid.stdout.as_str()), (2, ""));
    assert!(
        invalid
            .stderr
            .starts_with("monochromatic-lint: Standard input is not valid UTF-8"),
        "{}",
        invalid.stderr
    );
    let unsupported: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.txt", "--fix"],
        b"x",
    );
    assert_eq!((unsupported.status, unsupported.stdout.as_str()), (2, ""));
    // An MDX error under --fix returns the original source with the failure on standard error.
    let broken: Run = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.mdx", "--fix"],
        b"# Title.\n\n<A>\ntext\n</B>\n",
    );
    assert_eq!(broken.status, 2);
    assert_eq!(broken.stdout, "# Title.\n\n<A>\ntext\n</B>\n");
    let mut broken_codes: Vec<String> = Vec::<String>::new();
    for record in records(broken.stderr.as_str()) {
        broken_codes.push(String::from(record["code"].as_str().expect("code")));
    }
    assert_eq!(
        broken_codes,
        ["core/fix-refused", "core/processing-failure"]
    );
}

/// The commit-time shape:
///  a temporary one-rule configuration outside the repository,
///  standard input,
///  and fixing.
#[test]
fn the_commit_adapter_invocation_rewrites_lfs_images() {
    let repository: Fixture = Fixture::new();
    repository.write(
        ".lfsconfig",
        "[lfs]\n\turl = https://lfs:token@lfs.example\n",
    );
    repository.write(
        ".gitattributes",
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    );
    repository.write("pkg/asset/shot.png", "image bytes");
    repository.write("pkg/asset/plain.svg", "<svg/>");
    // A discovered configuration that would turn the rule off must be skipped by --config.
    repository.write(CONFIG, "[]");
    let temporary: Fixture = Fixture::new();
    let config: PathBuf = temporary.write(
        "one-rule.jsonc",
        r#"[{ "files": ["**/*.md", "**/*.mdx"], "rules": { "markdown/lfs-image-url": { "severity": "error", "exclude": ["package/ssg/"] } } }]"#,
    );
    let config_text: String = config.to_string_lossy().into_owned();
    // Measured with coreutils `sha256sum` for the bytes `image bytes`.
    let url: &str = "https://lfs.example/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png";
    let arguments: [&str; 6] = [
        "--config",
        config_text.as_str(),
        "--stdin",
        "--stdin-filename",
        "pkg/README.md",
        "--fix",
    ];
    let fixed: Run = run(
        &repository.path,
        &arguments,
        b"# Title.\n\n![shot](asset/shot.png)\n\n![plain](asset/plain.svg)\n",
    );
    assert_eq!((fixed.status, fixed.stderr.as_str()), (0, ""));
    // Only the selected rule ran: the heading punctuation is untouched.
    assert_eq!(
        fixed.stdout,
        format!("# Title.\n\n![shot]({url})\n\n![plain](asset/plain.svg)\n")
    );
    let stale: String = format!(
        "![shot](https://lfs.example/{}/pkg/asset/gone.png)\n",
        "b".repeat(64)
    );
    let reported: Run = run(&repository.path, &arguments, stale.as_bytes());
    assert_eq!(reported.status, 1);
    assert_eq!(reported.stdout, stale);
    assert_eq!(
        located(reported.stderr.as_str()),
        ["pkg/README.md markdown/lfs-image-url 1 1"]
    );
    let excluded: Run = run(
        &repository.path,
        &[
            "--config",
            config_text.as_str(),
            "--stdin",
            "--stdin-filename",
            "package/ssg/a.md",
            "--fix",
        ],
        b"![shot](../../pkg/asset/shot.png)\n",
    );
    assert_eq!(
        (
            excluded.status,
            excluded.stdout.as_str(),
            excluded.stderr.as_str()
        ),
        (0, "![shot](../../pkg/asset/shot.png)\n", "")
    );
    // On-disk linting of the same repository reports the host file and never contacts the server.
    repository.write("pkg/README.md", "![shot](asset/shot.png)\n");
    let files: Run = run(
        &repository.path,
        &["--config", config_text.as_str(), "--fix"],
        b"",
    );
    assert_eq!(
        (files.status, files.stdout.as_str(), files.stderr.as_str()),
        (0, "", "")
    );
    assert_eq!(
        repository.read("pkg/README.md"),
        format!("![shot]({url})\n")
    );
}

/// The non-linting modes work on the real binary and never modify source files.
#[test]
fn rules_init_print_config_and_debug_modes() {
    let fixture: Fixture = Fixture::new();
    fixture.write("a.md", "# Title.\n");
    let rules: Run = run(&fixture.path, &["--rules"], b"");
    assert_eq!((rules.status, rules.stderr.as_str()), (0, ""));
    let listed: Vec<serde_json::Value> = records(rules.stdout.as_str());
    assert_eq!(listed.len(), 27);
    assert_eq!(listed[0]["id"], "rust/max-lines");
    assert_eq!(listed[17]["id"], "markdown/lfs-image-url");
    assert_eq!(listed[26]["id"], "shell/no-ampersand");
    let init: Run = run(&fixture.path, &["--init"], b"");
    assert_eq!((init.status, init.stderr.as_str()), (0, ""));
    assert_eq!(
        init.stdout,
        format!("{}\n", fixture.path.join(CONFIG).display())
    );
    let starter: String = fixture.read(CONFIG);
    let again: Run = run(&fixture.path, &["--init"], b"");
    assert_eq!((again.status, again.stdout.as_str()), (2, ""));
    assert!(
        again
            .stderr
            .starts_with("monochromatic-lint: Cannot create "),
        "{}",
        again.stderr
    );
    assert_eq!(fixture.read(CONFIG), starter);
    // The starter configuration is immediately usable.
    let linted: Run = run(&fixture.path, &[], b"");
    assert_eq!(
        located(linted.stdout.as_str()),
        ["a.md markdown/no-trailing-punctuation 1 1"]
    );
    let printed: Run = run(&fixture.path, &["--print-config", "a.md"], b"");
    assert_eq!((printed.status, printed.stderr.as_str()), (0, ""));
    assert!(
        printed.stdout.contains("\"markdown/semantic-line-breaks\""),
        "{}",
        printed.stdout
    );
    let debug: Run = run(&fixture.path, &["--debug", "--concurrency", "1"], b"");
    assert_eq!(debug.stdout, linted.stdout);
    assert_eq!(debug.status, 1);
    assert!(!debug.stderr.is_empty());
    for line in debug.stderr.lines() {
        assert!(line.starts_with("monochromatic-lint: debug: "), "{line}");
    }
    assert_eq!(fixture.read("a.md"), "# Title.\n");
}

/// `--debug` streams semantic-workspace progress to standard error while a workspace loads,
///  and a run without
/// it prints nothing there.
///  An unparsable `Cargo.toml` makes the load fail after its first progress message,
/// so both runs end quickly with the same single processing finding and status 2.
#[test]
fn debug_streams_workspace_progress_and_plain_runs_stay_silent() {
    let fixture: Fixture = Fixture::new();
    fixture.write(
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }]"#,
    );
    fixture.write("Cargo.toml", "[package\n");
    fixture.write("src/lib.rs", "//! Library.\n");
    let debug: Run = run(&fixture.path, &["--debug", "src/lib.rs"], b"");
    assert_eq!(debug.status, 2, "{}", debug.stderr);
    assert!(
        debug
            .stderr
            .contains("monochromatic-lint: debug: discovering sysroot\n"),
        "{}",
        debug.stderr
    );
    let plain: Run = run(&fixture.path, &["src/lib.rs"], b"");
    assert_eq!((plain.status, plain.stderr.as_str()), (2, ""));
    // The messages quote Cargo's command line, which names a fresh temporary lockfile per run,
    // so the two runs are compared by finding location and code rather than byte for byte.
    for output in [&debug, &plain] {
        assert_eq!(
            located(output.stdout.as_str()),
            ["src/lib.rs core/processing-failure 1 1"]
        );
    }
}

/// A write failure other than a closed pipe exits 2 whichever stream failed,
///  instead of the findings' status.
/// `/dev/full` refuses every write with "no space left on device".
#[test]
fn a_failing_output_stream_exits_two() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    fixture.write("a.md", "# Title.\n");
    let baseline: Run = run(&fixture.path, &[], b"");
    assert_eq!((baseline.status, baseline.stderr.as_str()), (1, ""));
    // Standard output fails; standard error stays empty because nothing else had to be said.
    let full_stdout: std::fs::File = std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let stdout_failed: Output = Command::new(BINARY)
        .current_dir(&fixture.path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(full_stdout))
        .stderr(Stdio::piped())
        .output()
        .expect("run the built executable");
    assert_eq!(stdout_failed.status.code(), Some(2));
    assert!(stdout_failed.stderr.is_empty());
    // Standard error fails while `--debug` writes its notes there; the findings still reach standard output.
    let full_stderr: std::fs::File = std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let stderr_failed: Output = Command::new(BINARY)
        .arg("--debug")
        .current_dir(&fixture.path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(full_stderr))
        .output()
        .expect("run the built executable");
    assert_eq!(stderr_failed.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(stderr_failed.stdout).expect("UTF-8"),
        baseline.stdout
    );
}

/// Run the executable as `run` does,
///  but with the main thread's stack limited to `kibibytes` by the shell.
/// `sh -c` receives the limit as `$0` and the executable with its arguments as `$@`,
///  so nothing is quoted by hand,
/// and `exec` replaces the shell,
///  so the status is the executable's own.
fn run_with_main_stack(cwd: &Path, kibibytes: &str, arguments: &[&str], input: &[u8]) -> Run {
    let mut child: Child = Command::new("sh")
        .args(["-c", "ulimit -s \"$0\" && exec \"$@\"", kibibytes, BINARY])
        .args(arguments)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable through sh");
    let mut stdin = child.stdin.take().expect("piped standard input");
    stdin.write_all(input).expect("write standard input");
    drop(stdin);
    let output: Output = child.wait_with_output().expect("wait for the executable");
    // A stack overflow aborts the process, which then has a signal and no exit code.
    let Some(status): Option<i32> = output.status.code() else {
        panic!(
            "the executable was killed by a signal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    return Run {
        status,
        stdout: String::from_utf8(output.stdout).expect("standard output is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("standard error is UTF-8"),
    };
}

/// One file,
///  `--concurrency 1` and standard input are linted on a thread with the explicit lint stack,
///  not on the
/// main thread,
///  whose stack the platform sets (the MSVC linker reserves 1 MB by default).
///  With the main thread
/// limited to 1 MiB,
///  Rust nested 1,200 parentheses deep,
///  which needed between 3 and 4 MiB in the bounded container,
/// still parses in every mode.
///  Before the invocation thread existed,
///  all three runs aborted with a stack overflow.
#[test]
fn a_small_main_thread_stack_does_not_limit_nesting() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    // Documented items and one nested expression, so a completed check has no finding at all.
    let nested: String = format!(
        "//! Nested expression.\n\n/// Entry point.\nfn main() {{\n    let x: u32 = {}1{};\n}}\n",
        "(".repeat(1200),
        ")".repeat(1200)
    );
    fixture.write("a.rs", nested.as_str());
    fixture.write("b.rs", nested.as_str());
    let modes: [(&[&str], &[u8]); 3] = [
        (&["a.rs"], b""),
        (&["--concurrency", "1", "a.rs", "b.rs"], b""),
        (&["--stdin", "--stdin-filename", "a.rs"], nested.as_bytes()),
    ];
    for (arguments, input) in modes {
        let limited: Run = run_with_main_stack(&fixture.path, "1024", arguments, input);
        assert_eq!(
            (
                limited.status,
                limited.stdout.as_str(),
                limited.stderr.as_str()
            ),
            (0, "", ""),
            "{arguments:?}"
        );
    }
}

/// Output is the same at every concurrency limit,
///  and a reader that closes the pipe early causes no error output.
#[test]
fn concurrency_and_closed_pipes_do_not_change_results() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    for index in 0..300 {
        fixture.write(
            format!("d{}/f{index:03}.md", index % 7).as_str(),
            format!(
                "# Title {index}.\n\n# Second {index}\n\n### Third {index}\n\n```\ncode\n```\n"
            )
            .as_str(),
        );
    }
    let sequential: Run = run(&fixture.path, &["--concurrency", "1"], b"");
    assert_eq!((sequential.status, sequential.stderr.as_str()), (1, ""));
    assert_eq!(records(sequential.stdout.as_str()).len(), 1200);
    // More output than a pipe buffer holds, so an early close is observed by the writer.
    assert!(sequential.stdout.len() > 200_000);
    for limit in ["2", "4", "64"] {
        let parallel: Run = run(&fixture.path, &["--concurrency", limit], b"");
        assert_eq!(parallel.status, 1, "limit {limit}");
        assert_eq!(parallel.stderr, "", "limit {limit}");
        assert!(
            parallel.stdout == sequential.stdout,
            "limit {limit} changed the output"
        );
    }
    let default: Run = run(&fixture.path, &[], b"");
    assert!(default.stdout == sequential.stdout);
    // Start the executable, close the read end of its standard output at once, and collect what it says.
    let mut child: Child = Command::new(BINARY)
        .current_dir(&fixture.path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable");
    drop(child.stdout.take());
    let closed: Output = child.wait_with_output().expect("wait for the executable");
    assert_eq!(closed.status.code(), Some(1));
    assert_eq!(String::from_utf8(closed.stderr).expect("UTF-8"), "");
}

/// Inline directives through the real executable:
///  suppression,
///  the unused-directive options and their usage errors,
///  standard-input linting and the rule listing's suppressibility.
#[test]
fn directives_through_the_executable() {
    let fixture: Fixture = Fixture::new();
    fixture.write(CONFIG, RULES);
    fixture.write(
        "a.md",
        "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- legacy title -->\n# Second\n",
    );
    fixture.write(
        "lib.rs",
        "//! Docs.\n// monochromatic-lint-disable-next-line rust/max-lines -- budget\n/// Item.\nfn item() {}\n",
    );
    // The Markdown directive suppresses its finding; the Rust directive names an unsuppressible rule.
    let plain: Run = run(&fixture.path, &[], b"");
    assert_eq!((plain.status, plain.stderr.as_str()), (1, ""));
    assert_eq!(
        located(plain.stdout.as_str()),
        ["lib.rs directive/non-suppressible 2 41"]
    );
    // An unused directive is a warning on request, an error with the severity option.
    fixture.write("lib.rs", "//! Docs.\n");
    let warned: Run = run(&fixture.path, &["--report-unused-disable-directives"], b"");
    assert_eq!((warned.status, warned.stderr.as_str()), (0, ""));
    assert!(warned.stdout.is_empty(), "{}", warned.stdout);
    fixture.write("a.md", "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- stale -->\nplain\n");
    let warning: Run = run(&fixture.path, &["--report-unused-disable-directives"], b"");
    assert_eq!(warning.status, 0);
    assert_eq!(records(warning.stdout.as_str())[0]["severity"], "warn");
    let error: Run = run(
        &fixture.path,
        &["--report-unused-disable-directives-severity", "error"],
        b"",
    );
    assert_eq!(error.status, 1);
    assert_eq!(
        records(error.stdout.as_str())[0]["code"],
        "directive/unused"
    );
    // Giving both spellings, or an unknown severity, is a usage error.
    for arguments in [
        vec![
            "--report-unused-disable-directives",
            "--report-unused-disable-directives-severity=warn",
        ],
        vec!["--report-unused-disable-directives-severity=loud"],
    ] {
        let refused: Run = run(&fixture.path, &arguments, b"");
        assert_eq!(
            (refused.status, refused.stdout.as_str()),
            (2, ""),
            "{arguments:?}"
        );
        assert!(
            refused
                .stderr
                .contains("--report-unused-disable-directives"),
            "{}",
            refused.stderr
        );
    }
    // Standard input is linted with the same directives.
    let piped: Run = run(
        &fixture.path,
        &[
            "--stdin",
            "--stdin-filename",
            "n.md",
            "--report-unused-disable-directives",
        ],
        b"# One\n\n<!-- monochromatic-lint-disable-next-line markdown/single-h1 -- r -->\nplain\n",
    );
    assert_eq!(
        located(piped.stdout.as_str()),
        ["n.md directive/unused 3 43"]
    );
    // The listing declares which rules refuse directives.
    let rules: Run = run(&fixture.path, &["--rules"], b"");
    let listed: Vec<serde_json::Value> = records(rules.stdout.as_str());
    assert_eq!(listed[0]["id"], "rust/max-lines");
    assert_eq!(listed[0]["suppressible"], false);
    assert_eq!(listed[1]["suppressible"], false);
    assert_eq!(listed[2]["suppressible"], true);
}
