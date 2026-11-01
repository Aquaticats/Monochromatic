//! What:
//!  Whole-invocation controls for fixing,
//!  standard input and the LFS rule's end-to-end path.
//! Why:
//!  `--fix` rewrites user files and `--stdin --fix` feeds a commit pipeline;
//!  both are verified
//! as complete invocations,
//!  including the incumbent's standard-input cases.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('fix and stdin', () => { /* rewrite, mode, processors, stdin routing, lfs */ });
//! ```

/// Import the shared invocation fixtures.
use crate::run_output::RunOutput;
use crate::run_test_support::{ALL_RULES, CONFIG, codes, read, records, run, write};
use crate::test_fs::Fixture;
use std::path::Path;

/// Measured with coreutils `sha256sum` for the bytes `image bytes`.
const IMAGE_OID: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";

/// A configuration that selects only the LFS rule,
///  as the commit-time adapter writes it.
const LFS_ONLY: &str = r#"[{ "files": ["**/*.md", "**/*.mdx"], "rules": { "markdown/lfs-image-url": { "severity": "error" } } }]"#;

/// The incumbent's repository fixture:
///  endpoint with a credential,
///  one tracked image,
///  one plain file.
fn lfs_repository(root: &Path) {
    write(
        root,
        ".lfsconfig",
        "[lfs]\n\turl = https://lfs:token@lfs.example\n",
    );
    write(
        root,
        ".gitattributes",
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    );
    write(root, "pkg/asset/shot.png", "image bytes");
    write(root, "pkg/asset/plain.svg", "<svg/>");
}

/// The object URL the rule produces for the fixture image.
fn fixture_url() -> String {
    return format!("https://lfs.example/{IMAGE_OID}/pkg/asset/shot.png");
}

/// `--fix` rewrites fixable findings in every language and reports only what remains.
#[test]
fn fix_rewrites_files_and_reports_the_remainder() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(
        &fixture.path,
        "a.md",
        "# Title.\n\n```\ncode\n```\n\n# Second\n",
    );
    write(
        &fixture.path,
        "b.mdx",
        "# Title:\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n",
    );
    write(&fixture.path, "c.rs", "fn main() {}\n");
    let output: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(output.exit_code, 1);
    assert_eq!(output.stderr, "");
    assert_eq!(
        read(&fixture.path, "a.md"),
        "# Title\n\n```text\ncode\n```\n\n# Second\n"
    );
    let mdx: String = read(&fixture.path, "b.mdx");
    assert!(mdx.starts_with("# Title\n\n<table>"), "{mdx}");
    assert!(!mdx.contains("| A | B |"), "{mdx}");
    // Rust rules have no fixes; the file is untouched and still reported.
    assert_eq!(read(&fixture.path, "c.rs"), "fn main() {}\n");
    assert_eq!(
        codes(output.stdout.as_str()),
        [
            "markdown/single-h1",
            "rust/require-rustdoc",
            "rust/require-rustdoc"
        ]
    );
    // A second fixing run changes nothing and reports the same remainder.
    let again: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(again, output);
}

/// A rewritten file keeps its permission bits;
///  an untouched file keeps its bytes and its modification state.
#[cfg(unix)]
#[test]
fn fixed_files_keep_their_mode() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let fixable = write(&fixture.path, "a.md", "# Title.\n");
    let clean = write(&fixture.path, "b.md", "# Title\n");
    std::fs::set_permissions(&fixable, std::fs::Permissions::from_mode(0o640)).expect("chmod");
    std::fs::set_permissions(&clean, std::fs::Permissions::from_mode(0o444)).expect("chmod");
    let output: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!((output.exit_code, output.stdout.as_str()), (0, ""));
    assert_eq!(read(&fixture.path, "a.md"), "# Title\n");
    assert_eq!(
        std::fs::metadata(&fixable)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o7777,
        0o640
    );
    assert_eq!(
        std::fs::metadata(&clean)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o7777,
        0o444
    );
    let mut names: Vec<String> = Vec::<String>::new();
    for entry in std::fs::read_dir(&fixture.path).expect("directory") {
        names.push(
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    assert_eq!(names, ["a.md", "b.md", CONFIG]);
}

/// Processor findings carry host positions,
///  and a rustdoc fix lands inside the Rust host's comment.
#[test]
fn processor_findings_and_fixes_use_the_host_file() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[
          { "files": ["**/*.rs"], "ignores": ["**/*.md/**"], "rules": { "rust/require-rustdoc": { "severity": "error" } } },
          { "files": ["**/*.rs/*.md"], "rules": { "markdown/fenced-code-language": { "severity": "error" } } },
          { "files": ["**/*.md/*.rs", "**/*.rs/*.md/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "warn" } } }
        ]"#,
    );
    let rust: &str = "//! File.\n\n/// Item.\n///\n/// ```\n/// //! Example.\n/// let value = 1; fn helper() {}\n/// ```\nfn item() {}\n";
    write(&fixture.path, "src/a.rs", rust);
    write(
        &fixture.path,
        "doc/b.md",
        "Text.\n\n```rust\n//! Example.\nlet value = 2; fn helper() {}\n```\n",
    );
    let lint: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(lint.exit_code, 1);
    let lines: Vec<serde_json::Value> = records(lint.stdout.as_str());
    let mut summary: Vec<(String, String, u64)> = Vec::<(String, String, u64)>::new();
    for line in &lines {
        summary.push((
            String::from(line["filename"].as_str().expect("filename")),
            String::from(line["code"].as_str().expect("code")),
            line["labels"][0]["span"]["line"].as_u64().expect("line"),
        ));
    }
    assert_eq!(
        summary,
        [
            (
                String::from("doc/b.md"),
                String::from("rust/require-rustdoc"),
                5
            ),
            (
                String::from("src/a.rs"),
                String::from("markdown/fenced-code-language"),
                5
            ),
            (
                String::from("src/a.rs"),
                String::from("rust/require-rustdoc"),
                7
            ),
        ]
    );
    // The undocumented function's host byte range is its own text in each host file.
    let markdown_offset: usize = lines[0]["labels"][0]["span"]["offset"]
        .as_u64()
        .expect("offset") as usize;
    assert_eq!(
        &"Text.\n\n```rust\n//! Example.\nlet value = 2; fn helper() {}\n```\n"
            [markdown_offset..markdown_offset + 14],
        "fn helper() {}"
    );
    let rust_offset: usize = lines[2]["labels"][0]["span"]["offset"]
        .as_u64()
        .expect("offset") as usize;
    assert_eq!(&rust[rust_offset..rust_offset + 14], "fn helper() {}");
    let fixed: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(fixed.exit_code, 0);
    assert_eq!(
        read(&fixture.path, "src/a.rs"),
        "//! File.\n\n/// Item.\n///\n/// ```rust\n/// //! Example.\n/// let value = 1; fn helper() {}\n/// ```\nfn item() {}\n"
    );
    assert_eq!(
        codes(fixed.stdout.as_str()),
        ["rust/require-rustdoc", "rust/require-rustdoc"]
    );
}

/// Standard input is linted under the configuration of its logical filename;
///  no file is read or written.
#[test]
fn standard_input_routes_source_and_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let source: &str = "# Title.\n\n# Second\n";
    let lint: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "doc/never-written.md"],
        source,
    );
    assert_eq!(lint.exit_code, 1);
    assert_eq!(lint.stderr, "");
    assert_eq!(
        codes(lint.stdout.as_str()),
        ["markdown/no-trailing-punctuation", "markdown/single-h1"]
    );
    assert_eq!(
        records(lint.stdout.as_str())[0]["filename"],
        "doc/never-written.md"
    );
    let fixed: RunOutput = run(
        &fixture.path,
        &[
            "--stdin",
            "--stdin-filename",
            "doc/never-written.md",
            "--fix",
        ],
        source,
    );
    assert_eq!(fixed.exit_code, 1);
    assert_eq!(fixed.stdout, "# Title\n\n# Second\n");
    assert_eq!(codes(fixed.stderr.as_str()), ["markdown/single-h1"]);
    assert!(!fixture.path.join("doc").exists());
    // A clean source is echoed byte for byte, including a byte order mark and CRLF line endings.
    let clean: &str = "\u{feff}# Title\r\n\r\nOne sentence.\r\n";
    let echoed: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix"],
        clean,
    );
    assert_eq!(
        (
            echoed.exit_code,
            echoed.stdout.as_str(),
            echoed.stderr.as_str()
        ),
        (0, clean, "")
    );
    // The extension chooses the language: the same text is not Markdown under a Rust filename.
    let rust: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "src/a.rs"],
        "//! File.\n\n/// Item.\nfn item() {}\n",
    );
    assert_eq!((rust.exit_code, rust.stdout.as_str()), (0, ""));
    // Silent fixing still prints the fixed source and keeps the status.
    let silent: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix", "--silent"],
        source,
    );
    assert_eq!(
        (
            silent.exit_code,
            silent.stdout.as_str(),
            silent.stderr.as_str()
        ),
        (1, "# Title\n\n# Second\n", "")
    );
}

/// Standard-input setup failures exit 2 and print no source.
#[test]
fn standard_input_setup_failures_exit_two() {
    let fixture: Fixture = Fixture::new();
    let unconfigured: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix"],
        "# Title.\n",
    );
    assert_eq!(
        (unconfigured.exit_code, unconfigured.stdout.as_str()),
        (2, "")
    );
    assert!(
        unconfigured.stderr.contains("standard input filename a.md"),
        "{}",
        unconfigured.stderr
    );
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "ignores": ["skip/"] }, { "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } }]"#,
    );
    let unsupported: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "notes.txt"],
        "x",
    );
    assert_eq!(
        (unsupported.exit_code, unsupported.stdout.as_str()),
        (2, "")
    );
    assert!(
        unsupported.stderr.contains("must end in .rs, .md or .mdx"),
        "{}",
        unsupported.stderr
    );
    let parsed =
        crate::run_test_support::options(&["--stdin", "--stdin-filename", "a.md", "--fix"]);
    let invalid: RunOutput =
        crate::run_command::run_command(&parsed, &fixture.path, &mut [b'#', 0xff].as_slice());
    assert_eq!((invalid.exit_code, invalid.stdout.as_str()), (2, ""));
    assert!(
        invalid
            .stderr
            .contains("not valid UTF-8 (first invalid byte at offset 1)"),
        "{}",
        invalid.stderr
    );
    // An ignored logical filename is not linted; fixing echoes its source unchanged.
    let ignored: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "skip/a.md", "--fix"],
        "# Title.\n",
    );
    assert_eq!(
        (
            ignored.exit_code,
            ignored.stdout.as_str(),
            ignored.stderr.as_str()
        ),
        (0, "# Title.\n", "")
    );
    // A refused fix prints the original source and reports the refusal on standard error.
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/link-image-reference-definitions": { "severity": "warn" } } }]"#,
    );
    let refused: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "a.md", "--fix"],
        "[unused]: https://example.com\n",
    );
    assert_eq!(refused.exit_code, 2);
    assert_eq!(refused.stdout, "[unused]: https://example.com\n");
    assert_eq!(
        codes(refused.stderr.as_str()),
        [
            "core/fix-refused",
            "markdown/link-image-reference-definitions"
        ]
    );
}

/// `--config` outside the repository resolves its patterns against the working directory,
///  for files and standard input.
#[test]
fn an_explicit_configuration_outside_the_tree_matches_tree_paths() {
    let fixture: Fixture = Fixture::new();
    let work = fixture.path.join("work");
    write(&work, "doc/a.md", "# Title.\n");
    write(&work, "other/a.md", "# Title.\n");
    // A nearer discovered configuration would turn the rule off; --config must skip it.
    write(&work, &format!("doc/{CONFIG}"), "[]");
    let config = write(
        &fixture.path,
        "temporary/rules.jsonc",
        r#"[{ "files": ["doc/**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } }]"#,
    );
    let config_text: String = config.to_string_lossy().into_owned();
    let files: RunOutput = run(&work, &["--config", config_text.as_str()], "");
    assert_eq!(files.exit_code, 1);
    assert_eq!(records(files.stdout.as_str()).len(), 1);
    assert_eq!(records(files.stdout.as_str())[0]["filename"], "doc/a.md");
    let stdin: RunOutput = run(
        &work,
        &[
            "--config",
            config_text.as_str(),
            "--stdin",
            "--stdin-filename",
            "doc/new.md",
            "--fix",
        ],
        "# Title.\n",
    );
    assert_eq!(
        (
            stdin.exit_code,
            stdin.stdout.as_str(),
            stdin.stderr.as_str()
        ),
        (0, "# Title\n", "")
    );
    let unmatched: RunOutput = run(
        &work,
        &[
            "--config",
            config_text.as_str(),
            "--stdin",
            "--stdin-filename",
            "other/new.md",
            "--fix",
        ],
        "# Title.\n",
    );
    assert_eq!(
        (unmatched.exit_code, unmatched.stdout.as_str()),
        (0, "# Title.\n")
    );
}

/// The incumbent's standard-input LFS cases:
///  fix as if at the path,
///  report without rewriting,
///  exclude,
///  and MDX.
#[test]
fn standard_input_lfs_rewrites_follow_the_incumbent_cases() {
    let fixture: Fixture = Fixture::new();
    lfs_repository(&fixture.path);
    write(&fixture.path, CONFIG, LFS_ONLY);
    let url: String = fixture_url();
    let fixed: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "pkg/README.md", "--fix"],
        "![shot](asset/shot.png)\n",
    );
    assert_eq!(fixed.stdout, format!("![shot]({url})\n"));
    assert_eq!((fixed.exit_code, fixed.stderr.as_str()), (0, ""));
    let reported: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "pkg/README.md"],
        "![shot](asset/shot.png)\n",
    );
    assert_eq!(reported.exit_code, 1);
    assert_eq!(codes(reported.stdout.as_str()), ["markdown/lfs-image-url"]);
    assert_eq!(
        records(reported.stdout.as_str())[0]["filename"],
        "pkg/README.md"
    );
    // Only the selected rule runs: the pipe table stays.
    let selected: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "pkg/README.md", "--fix"],
        "![shot](asset/shot.png)\n\n| A | B |\n| - | - |\n| 1 | 2 |\n",
    );
    assert!(
        selected.stdout.contains(url.as_str()),
        "{}",
        selected.stdout
    );
    assert!(selected.stdout.contains("| A | B |"), "{}", selected.stdout);
    assert_eq!(selected.exit_code, 0);
    let mdx: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "pkg/page.mdx", "--fix"],
        "import X from \"./x\";\n\n![shot](asset/shot.png)\n",
    );
    assert!(mdx.stdout.contains(url.as_str()), "{}", mdx.stdout);
    // The exclude option leaves the file alone.
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/lfs-image-url": { "severity": "error", "exclude": ["pkg/"] } } }]"#,
    );
    let excluded: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "pkg/README.md", "--fix"],
        "![shot](asset/shot.png)\n",
    );
    assert_eq!(
        (excluded.exit_code, excluded.stdout.as_str()),
        (0, "![shot](asset/shot.png)\n")
    );
}

/// On disk,
///  the LFS rule rewrites tracked images,
///  leaves plain ones,
///  and fails loudly on an unusable endpoint.
#[test]
fn lfs_rewrites_files_and_unusable_endpoints_are_processing_failures() {
    let fixture: Fixture = Fixture::new();
    lfs_repository(&fixture.path);
    write(&fixture.path, CONFIG, LFS_ONLY);
    write(
        &fixture.path,
        "pkg/README.md",
        "![shot](asset/shot.png)\n\n![plain](asset/plain.svg)\n",
    );
    let fixed: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!((fixed.exit_code, fixed.stdout.as_str()), (0, ""));
    assert_eq!(
        read(&fixture.path, "pkg/README.md"),
        format!("![shot]({})\n\n![plain](asset/plain.svg)\n", fixture_url())
    );
    // An endpoint outside the supported form is not "no LFS here": every checked file fails with the reason.
    write(
        &fixture.path,
        ".lfsconfig",
        "[lfs]\n\turl = ssh://git:secret@lfs.example/x\n",
    );
    let unusable: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(unusable.exit_code, 2);
    assert_eq!(codes(unusable.stdout.as_str()), ["core/processing-failure"]);
    let message: String = String::from(
        records(unusable.stdout.as_str())[0]["message"]
            .as_str()
            .expect("message"),
    );
    assert!(
        message.contains("markdown/lfs-image-url could not check this file"),
        "{message}"
    );
    assert!(message.contains("https://host/path"), "{message}");
    assert!(!message.contains("secret"), "{message}");
}
