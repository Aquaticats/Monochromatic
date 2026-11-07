//! What:
//!  Differential controls:
//!  the same inputs go to the built `monochromatic-lint` executable in its standard-input fix mode
//!  and to the in-process `fix_source`,
//!  and the two answers must be the same bytes.
//! Why:
//!  `fix_source` is documented as the executable's `--config --stdin --stdin-filename --fix` run
//!  without the program start.
//!  A caller (the Git policy wrapper) relies on that equality for the LFS image rule,
//!  byte-order marks and astral characters,
//!  and nothing but a run of both on the same input shows it holds.
//!  A success compares the fixed source with standard output and the findings,
//!  rendered as the executable renders them (offsets and positions included),
//!  with standard error;
//!  a failure compares the status and the explanation with the executable's message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const input of inputs) expect(fixSource(input)).toEqual(parse(spawnSync(binary, args, { cwd, input })));
//! ```

/// Import child-process control and native paths.
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

/// The in-process entry and the finding renderer the executable uses.
use monochromatic_lint::{
    diagnostic::render,
    run_embedded::{EmbeddedFailure, EmbeddedRequest, EmbeddedRun, fix_source},
    run_finish::PROGRAM,
};

/// The path of the executable Cargo built for this test run.
const BINARY: &str = env!("CARGO_BIN_EXE_monochromatic-lint");

/// The one rule the Git policy wrapper selects.
const RULE: &str = "markdown/lfs-image-url";

/// The LFS endpoint the fixture repository declares.
const ENDPOINT: &str = "https://lfs.example.com/org/repo";

/// The bytes of the one tracked image.
const IMAGE: &[u8] = b"image bytes";

/// The image's path below the top level.
const IMAGE_PATH: &str = "pkg/asset/shot.png";

/// A relative link to the tracked image,
///  which the rule rewrites to an object URL.
const RELATIVE_LINK: &str = "![shot](asset/shot.png)";

/// A process-wide sequence for unique fixture directory names.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A directory this test created and removes when it goes out of scope.
struct Fixture {
    /// Root created exclusively for one test.
    root: PathBuf,
}

/// Create the repository tree with an LFS endpoint,
///  the PNG rule and one tracked image.
impl Fixture {
    /// Exclusively create the tree.
    fn new() -> Fixture {
        let sequence: u64 = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let created: PathBuf = crate::test_scratch::scratch_root().join(format!(
            "monochromatic-lint-differential-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&created).expect("exclusively create fixture directory");
        let root: PathBuf = std::fs::canonicalize(&created).expect("resolve fixture directory");
        let top: PathBuf = root.join("top");
        std::fs::create_dir_all(top.join("pkg/asset")).expect("tree");
        std::fs::write(
            top.join(".lfsconfig"),
            format!("[lfs]\n\turl = {ENDPOINT}\n"),
        )
        .expect("lfsconfig");
        std::fs::write(
            top.join(".gitattributes"),
            b"*.png filter=lfs diff=lfs merge=lfs -text\n",
        )
        .expect("attributes");
        std::fs::write(top.join(IMAGE_PATH), IMAGE).expect("image");
        return Fixture { root };
    }

    /// The directory both runs start from.
    fn top(&self) -> PathBuf {
        return self.root.join("top");
    }
}

/// Remove the directory this fixture created,
///  including after a failed assertion.
impl Drop for Fixture {
    /// Delete only the path this instance created.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).expect("remove fixture directory");
    }
}

/// The configuration the wrapper writes for the LFS rule with these exclude patterns.
fn configuration(severity: &str, exclude: &[&str]) -> String {
    let patterns: String = serde_json::to_string(exclude).expect("patterns as JSON");
    return format!(
        "[{{\"files\":[\"**/*.md\",\"**/*.mdx\"],\"rules\":{{\"{RULE}\":{{\"severity\":\"{severity}\",\"exclude\":{patterns}}}}}}}]"
    );
}

/// The severity the wrapper configures, under which a finding the fixes leave does not fail the run.
const WARN: &str = "warn";

/// A severity under which a finding the fixes leave fails the run with status 1.
const ERROR: &str = "error";

/// One input to both runs.
struct Case {
    /// What the case shows,
    ///  for the failure message.
    label: &'static str,
    /// The name the source is linted as,
    ///  relative to the top level.
    name: OsString,
    /// The source bytes.
    source: Vec<u8>,
    /// The rule's exclude patterns.
    exclude: Vec<&'static str>,
    /// The rule's severity.
    severity: &'static str,
}

/// Run the executable the way the wrapper used to start it.
fn run_executable(fixture: &Fixture, case: &Case) -> Output {
    let configuration_file: PathBuf = fixture.root.join("configuration.jsonc");
    std::fs::write(
        &configuration_file,
        configuration(case.severity, case.exclude.as_slice()).as_bytes(),
    )
    .expect("configuration file");
    let mut config: OsString = OsString::from("--config=");
    config.push(&configuration_file);
    let mut filename: OsString = OsString::from("--stdin-filename=");
    filename.push(&case.name);
    let mut child: Child = Command::new(BINARY)
        .arg(config)
        .arg("--stdin")
        .arg(filename)
        .arg("--fix")
        .current_dir(fixture.top())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the built executable");
    let mut stdin = child.stdin.take().expect("piped standard input");
    stdin
        .write_all(case.source.as_slice())
        .expect("write standard input");
    drop(stdin);
    return child.wait_with_output().expect("wait for the executable");
}

/// Run the in-process entry on the same input.
fn run_in_process(fixture: &Fixture, case: &Case) -> Result<EmbeddedRun, EmbeddedFailure> {
    let document: String = configuration(case.severity, case.exclude.as_slice());
    let top: PathBuf = fixture.top();
    return fix_source(&EmbeddedRequest {
        top_level: top.as_path(),
        configuration: document.as_str(),
        name: Path::new(case.name.as_os_str()),
        source: case.source.as_slice(),
    });
}

/// Assert the two runs agree on one case,
///  and return whether the executable reported the run as usable.
fn assert_same(fixture: &Fixture, case: &Case) -> bool {
    let executable: Output = run_executable(fixture, case);
    let embedded: Result<EmbeddedRun, EmbeddedFailure> = run_in_process(fixture, case);
    let status: i32 = executable
        .status
        .code()
        .expect("the executable ends with a status");
    match embedded {
        Ok(run) => {
            assert_eq!(
                status, 0,
                "{}: the executable disagrees on success",
                case.label
            );
            assert_eq!(
                executable.stdout,
                run.fixed.as_bytes(),
                "{}: fixed source",
                case.label
            );
            assert_eq!(
                String::from_utf8(executable.stderr).expect("records are text"),
                render(run.findings.as_slice()).expect("render the findings"),
                "{}: findings",
                case.label
            );
            return true;
        }
        Err(failure) => {
            assert_eq!(
                i32::from(failure.exit_status),
                status,
                "{}: exit status",
                case.label
            );
            assert_eq!(
                String::from_utf8(executable.stderr).expect("the message is text"),
                format!("{PROGRAM}: {}\n", failure.explanation),
                "{}: explanation",
                case.label
            );
            assert!(
                executable.stdout.is_empty(),
                "{}: standard output",
                case.label
            );
            return false;
        }
    }
}

/// A case with default options and a UTF-8 name.
fn case(label: &'static str, name: &str, source: &[u8]) -> Case {
    return Case {
        label,
        name: OsString::from(name),
        source: source.to_vec(),
        exclude: Vec::new(),
        severity: WARN,
    };
}

/// The object URL text for the given digest and path.
fn object_url(digest: &str, path: &str) -> String {
    return format!("{ENDPOINT}/{digest}/{path}");
}

/// The cases that run to a usable result in both.
fn usable_cases() -> Vec<Case> {
    let stale: String = format!(
        "![shot]({})\n",
        object_url(&"b".repeat(64), "pkg/asset/gone.png")
    );
    let relative: String = format!("# T\n\n{RELATIVE_LINK}\n");
    // A byte-order mark is one character in the source and shifts every column of the first line.
    let mut marked: Vec<u8> = "\u{feff}".as_bytes().to_vec();
    marked.extend_from_slice(stale.as_bytes());
    // An astral character is two UTF-16 units and four UTF-8 bytes before the stale link.
    let astral: String = format!("\u{1F600} {stale}");
    let mut excluded: Case = case("excluded", "pkg/README.md", relative.as_bytes());
    excluded.exclude = vec!["pkg/"];
    let mut long_name: Case = case("long name", "pkg/README.md", relative.as_bytes());
    long_name.exclude = vec!["other/"];
    let mut cases: Vec<Case> = vec![
        case("relative link", "pkg/README.md", relative.as_bytes()),
        case("relative link in mdx", "pkg/page.mdx", relative.as_bytes()),
        case(
            "name starting with a dash",
            "pkg/-x.mdx",
            relative.as_bytes(),
        ),
        case("stale object url", "pkg/README.md", stale.as_bytes()),
        case(
            "byte-order mark before a stale link",
            "pkg/README.md",
            marked.as_slice(),
        ),
        case(
            "byte-order mark before a relative link",
            "pkg/README.md",
            format!("\u{feff}{relative}").as_bytes(),
        ),
        case(
            "astral character before a stale link",
            "pkg/README.md",
            astral.as_bytes(),
        ),
        case(
            "astral character before a relative link",
            "pkg/README.md",
            format!("\u{1F600} {relative}").as_bytes(),
        ),
        case(
            "crlf line endings",
            "pkg/README.md",
            relative.replace('\n', "\r\n").as_bytes(),
        ),
        case("empty source", "pkg/README.md", b""),
        case(
            "no final newline",
            "pkg/README.md",
            RELATIVE_LINK.as_bytes(),
        ),
        case(
            "unfinished link",
            "pkg/README.md",
            b"![shot](asset/shot.png",
        ),
        case("a name no block selects", "pkg/lib.rs", b"fn main() {}\n"),
        excluded,
        long_name,
    ];
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        cases.push(Case {
            label: "name that is not UTF-8",
            name: OsString::from(std::ffi::OsStr::from_bytes(b"pkg/caf\xe9.md")),
            source: relative.as_bytes().to_vec(),
            exclude: Vec::new(),
            severity: WARN,
        });
    }
    return cases;
}

/// Every usable case gives the same fixed source and the same findings,
///  and the cases include a fix, a kept finding and no change at all.
#[test]
fn the_executable_and_the_in_process_entry_agree_on_every_usable_input() {
    let fixture: Fixture = Fixture::new();
    for input in usable_cases() {
        assert!(
            assert_same(&fixture, &input),
            "{}: usable in both",
            input.label
        );
    }
}

/// A fix, a finding and an unchanged source are all among the cases,
///  so the equality above is not between empty results.
#[test]
fn the_usable_cases_cover_a_fix_a_kept_finding_and_no_change() {
    let fixture: Fixture = Fixture::new();
    let mut fixed: bool = false;
    let mut kept: bool = false;
    let mut unchanged: bool = false;
    for input in usable_cases() {
        let run: EmbeddedRun = run_in_process(&fixture, &input).expect("usable");
        fixed |= run.fixed.as_bytes() != input.source.as_slice();
        kept |= !run.findings.is_empty();
        unchanged |= run.fixed.as_bytes() == input.source.as_slice();
    }
    assert!(
        fixed && kept && unchanged,
        "fixed {fixed}, kept {kept}, unchanged {unchanged}"
    );
}

/// A source that is not UTF-8, a name the linter cannot classify and a configuration it cannot parse
///  fail the same way in both.
#[test]
fn the_executable_and_the_in_process_entry_agree_on_every_refusal() {
    let fixture: Fixture = Fixture::new();
    let refused: Vec<Case> = vec![
        case(
            "source that is not UTF-8",
            "pkg/README.md",
            b"# T\n\xff\xfe\n",
        ),
        case("name with no language", "pkg/notes.txt", b"text\n"),
    ];
    for input in refused {
        assert!(
            !assert_same(&fixture, &input),
            "{}: refused in both",
            input.label
        );
    }
}

/// The messages of the records a failed run printed that are errors, joined as the in-process entry joins them.
fn error_messages(stderr: &[u8]) -> String {
    let text: &str = std::str::from_utf8(stderr).expect("records are text");
    let mut messages: Vec<String> = Vec::new();
    for line in text.lines() {
        let record: serde_json::Value = serde_json::from_str(line).expect("a record");
        if record["severity"] == "error" {
            messages.push(String::from(record["message"].as_str().expect("a message")));
        }
    }
    return messages.join("; ");
}

/// Assert both runs failed the same way after the source was linted:
///  the same non-zero status and the explanation made of the records the executable printed.
fn assert_same_failed_run(fixture: &Fixture, case: &Case, status: u8) {
    let executable: Output = run_executable(fixture, case);
    let embedded: EmbeddedFailure = run_in_process(fixture, case).expect_err("the run fails");
    assert_eq!(
        executable.status.code(),
        Some(i32::from(status)),
        "{}: executable status",
        case.label
    );
    assert_eq!(
        embedded.exit_status, status,
        "{}: in-process status",
        case.label
    );
    assert_eq!(
        embedded.explanation,
        error_messages(executable.stderr.as_slice()),
        "{}: explanation",
        case.label
    );
    assert!(
        !embedded.explanation.is_empty(),
        "{}: the failure explains itself",
        case.label
    );
}

/// A finding at error severity fails the run with status 1 in both, and the explanation is the finding's message.
#[test]
fn the_executable_and_the_in_process_entry_agree_on_a_finding_at_error_severity() {
    let fixture: Fixture = Fixture::new();
    let stale: String = format!(
        "![shot]({})\n",
        object_url(&"b".repeat(64), "pkg/asset/gone.png")
    );
    let mut failing: Case = case(
        "stale object url at error severity",
        "pkg/README.md",
        stale.as_bytes(),
    );
    failing.severity = ERROR;
    assert_same_failed_run(&fixture, &failing, 1);
}

/// A processing failure (an endpoint the rule cannot use) fails the run with status 2 in both,
///  through the records the executable prints and not through its setup message.
#[test]
fn the_executable_and_the_in_process_entry_agree_on_a_processing_failure() {
    let fixture: Fixture = Fixture::new();
    std::fs::write(
        fixture.top().join(".lfsconfig"),
        b"[lfs]\n\turl = not a url\n",
    )
    .expect("unusable endpoint");
    let source: String = format!("# T\n\n{RELATIVE_LINK}\n");
    assert_same_failed_run(
        &fixture,
        &case("unusable endpoint", "pkg/README.md", source.as_bytes()),
        2,
    );
}

/// The scratch root of this test run, placed in the build directory instead of `/tmp`.
#[path = "test_scratch.rs"]
mod test_scratch;
