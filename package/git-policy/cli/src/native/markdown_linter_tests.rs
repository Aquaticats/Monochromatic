//! What:
//!  Controls for the executable's linter:
//!  lookup,
//!  arguments,
//!  configuration file,
//!       bounds and every failure,
//!  with stand-in programs,
//!  and the real linter's contract.
//! Why:
//!  The stand-ins reach each branch quickly;
//!  the real linter,
//!  built from this
//!      repository and named by `GIT_POLICY_NATIVE_TEST_LINTER`,
//!  proves the contract the
//!      policy relies on.
//!  A test that needs it fails when the variable is unset,
//!  so an
//!      absent linter can never read as a pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = processLinter(env, limits, scratch).lint({ topLevel, options, path, source });
//! ```

/// The linter under test and its private parts.
use super::{
    LINTER_LIMITS, ProcessLinter, child_failure_reason, executable_linter, process_linter,
};
use crate::bounded_child::{ChildFailure, ChildLimits};
use crate::config_schema::{MarkdownAutofixOptions, PolicyConfig};
use crate::markdown_linter_output::{LintRun, RemainingFinding};
use crate::policy_markdown::{LintRequest, MarkdownLinter};
use crate::test_support::{SharedSink, executable, fixture, remove};
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The variable naming the linter built from this repository,
///  set in the gate image.
const LINTER_VARIABLE: &str = "GIT_POLICY_NATIVE_TEST_LINTER";

/// The default options:
///  the LFS rule and no exclusion.
fn default_options() -> MarkdownAutofixOptions {
    return PolicyConfig::defaults().markdown_autofix;
}

/// An environment whose PATH is `bin` and system programs,
///  with `extra` pairs.
fn environment(bin: &Path, extra: &[(&str, &Path)]) -> Vec<(OsString, OsString)> {
    let mut path: OsString = bin.as_os_str().to_os_string();
    path.push(":/usr/bin:/bin");
    let mut pairs: Vec<(OsString, OsString)> = vec![(OsString::from("PATH"), path)];
    for (name, value) in extra {
        pairs.push((OsString::from(name), value.as_os_str().to_os_string()));
    }
    return pairs;
}

/// Write a stand-in named `monochromatic-lint` into `<root>/bin` and return that directory.
fn stand_in(root: &Path, script: &str) -> PathBuf {
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(&bin).expect("bin");
    executable(
        bin.join("monochromatic-lint").as_path(),
        format!("#!/bin/sh\n{script}\n").as_bytes(),
    );
    return bin;
}

/// Lint `source` as `path` from `top_level` with `linter` and the default options.
fn lint(
    linter: &mut ProcessLinter,
    top_level: &Path,
    path: &[u8],
    source: &[u8],
) -> Result<LintRun, String> {
    let options: MarkdownAutofixOptions = default_options();
    return linter.lint(&LintRequest {
        top_level,
        options: &options,
        path,
        source,
    });
}

/// The bounds of the executable's linter and its scratch directory.
#[test]
fn the_executable_linter_uses_the_measured_bounds() {
    assert_eq!(
        LINTER_LIMITS,
        ChildLimits {
            timeout: Duration::from_secs(60),
            max_stdout: 67_108_864,
            max_stderr: 4_194_304,
        }
    );
    let linter: ProcessLinter = executable_linter(&[]);
    assert_eq!(linter.limits, LINTER_LIMITS);
    assert!(linter.scratch.is_absolute(), "{:?}", linter.scratch);
    assert_eq!(linter.environment, Vec::<(OsString, OsString)>::new());
}

/// Without the linter on PATH every candidate fails with the same reason.
#[test]
fn a_missing_linter_is_reported() {
    let root: PathBuf = fixture("linter-missing");
    let reason: &str = "cli-git could not find monochromatic-lint in any absolute directory on PATH, and the policy markdown/autofix runs it for every Markdown candidate; install it so that it is on PATH";
    let mut unset: ProcessLinter = process_linter(&[], LINTER_LIMITS, root.clone());
    let mut empty: ProcessLinter = process_linter(
        environment(root.as_path(), &[]).as_slice(),
        LINTER_LIMITS,
        root.clone(),
    );
    for linter in [&mut unset, &mut empty] {
        for _ in 0..2 {
            assert_eq!(
                lint(linter, root.as_path(), b"a.md", b"a\n"),
                Err(String::from(reason))
            );
        }
    }
    remove(root.as_path());
}

/// Whether one byte ends a line,
///  for splitting the recorded arguments.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// The stand-in records its arguments,
///  directory and configuration,
///  then echoes its input.
const RECORDING: &str = "printf '%s\\n' \"$@\" > \"$RECORD/arguments\"\npwd > \"$RECORD/directory\"\ncat \"${1#--config=}\" > \"$RECORD/configuration\"\ncat";

/// The linter is started from the top level with one `--name=value` argument per option,
/// the candidate's exact name and the one-rule configuration,
///  written once for the
/// invocation and removed with it;
///  its input comes back as the fixed source.
#[test]
fn the_linter_gets_the_contract_arguments() {
    let root: PathBuf = fixture("linter-arguments");
    let bin: PathBuf = stand_in(root.as_path(), RECORDING);
    let record: PathBuf = root.join("record");
    let top: PathBuf = root.join("top");
    std::fs::create_dir(&record).expect("record");
    std::fs::create_dir(&top).expect("top");
    let mut linter: ProcessLinter = process_linter(
        environment(bin.as_path(), &[("RECORD", record.as_path())]).as_slice(),
        LINTER_LIMITS,
        root.clone(),
    );
    let mut configurations: Vec<Vec<u8>> = Vec::new();
    for name in [&b"-x.md"[..], b"dir/caf\xe9 =.md"] {
        assert_eq!(
            lint(&mut linter, top.as_path(), name, b"source\n"),
            Ok(LintRun {
                fixed: b"source\n".to_vec(),
                remaining: Vec::new(),
            })
        );
        let arguments: Vec<u8> = std::fs::read(record.join("arguments")).expect("arguments");
        let lines: Vec<&[u8]> = arguments.split(is_line_feed).collect();
        assert_eq!(lines.len(), 5, "{lines:?}");
        configurations.push(lines[0].to_vec());
        assert_eq!(lines[1], b"--stdin");
        assert_eq!(
            lines[2],
            [&b"--stdin-filename="[..], name].concat().as_slice()
        );
        assert_eq!((lines[3], lines[4]), (&b"--fix"[..], &b""[..]));
        assert_eq!(
            std::fs::read_to_string(record.join("directory")).expect("directory"),
            format!("{}\n", top.display())
        );
    }
    assert_eq!(
        configurations[0], configurations[1],
        "one configuration per invocation"
    );
    let configuration: PathBuf = PathBuf::from(OsString::from_vec(
        configurations[0]["--config=".len()..].to_vec(),
    ));
    assert_eq!(configuration.parent(), Some(root.as_path()));
    assert_eq!(
        std::fs::read_to_string(record.join("configuration")).expect("copy"),
        crate::markdown_linter_config::one_rule_configuration(
            default_options().rules.as_slice(),
            &[]
        )
    );
    assert!(configuration.exists());
    drop(linter);
    assert!(
        !configuration.exists(),
        "the configuration leaves with the invocation"
    );
    remove(root.as_path());
}

/// The program found first is kept for the invocation:
///  once it is gone,
///  the next start
/// fails instead of searching again.
#[test]
fn the_program_is_looked_up_once() {
    let root: PathBuf = fixture("linter-once");
    let bin: PathBuf = stand_in(root.as_path(), "cat");
    let mut linter: ProcessLinter = process_linter(
        environment(bin.as_path(), &[]).as_slice(),
        LINTER_LIMITS,
        root.clone(),
    );
    assert!(lint(&mut linter, root.as_path(), b"a.md", b"a\n").is_ok());
    std::fs::remove_file(bin.join("monochromatic-lint")).expect("remove the program");
    let started: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("gone");
    assert!(
        started.starts_with(
            format!(
                "cli-git could not start monochromatic-lint at {}/monochromatic-lint: ",
                bin.display()
            )
            .as_str()
        ),
        "{started}"
    );
    remove(root.as_path());
}

/// A run past its bound or its cap,
///  a run that could not finish,
///  a configuration that
/// cannot be written and a name that cannot be spelled each come back with their reason.
#[test]
fn each_failure_comes_back_with_its_reason() {
    let root: PathBuf = fixture("linter-failures");
    let small: ChildLimits = ChildLimits {
        timeout: Duration::from_millis(300),
        max_stdout: 10,
        max_stderr: 1 << 20,
    };
    let sleeper: PathBuf = stand_in(&root.join("sleeper"), "exec sleep 30");
    let mut slow: ProcessLinter = process_linter(
        environment(sleeper.as_path(), &[]).as_slice(),
        small,
        root.clone(),
    );
    assert_eq!(
        lint(&mut slow, root.as_path(), b"a.md", b"a\n"),
        Err(String::from(
            "monochromatic-lint did not finish within 300 milliseconds and was stopped; run the command again, \
             run monochromatic-lint by hand on the file to see why it stalls, or set the policy markdown/autofix \
             to off to skip the check"
        ))
    );
    let flood: PathBuf = stand_in(&root.join("flood"), "exec head -c 1000 /dev/zero");
    let mut loud: ProcessLinter = process_linter(
        environment(flood.as_path(), &[]).as_slice(),
        small,
        root.clone(),
    );
    assert_eq!(
        lint(&mut loud, root.as_path(), b"a.md", b"a\n"),
        Err(String::from(
            "monochromatic-lint wrote more than 10 bytes to its standard output and was stopped"
        ))
    );
    let broken: PathBuf = stand_in(
        &root.join("broken"),
        "echo 'monochromatic-lint: Cannot read standard input.' >&2\nexit 2",
    );
    let mut failing: ProcessLinter = process_linter(
        environment(broken.as_path(), &[]).as_slice(),
        LINTER_LIMITS,
        root.clone(),
    );
    assert_eq!(
        lint(&mut failing, root.as_path(), b"a.md", b"a\n"),
        Err(String::from(
            "monochromatic-lint could not finish (exit status 2): Cannot read standard input."
        ))
    );
    assert_eq!(
        lint(&mut failing, root.as_path(), b"", b"a\n"),
        Err(String::from(
            "cli-git cannot spell this candidate's pathname for monochromatic-lint on this platform"
        ))
    );
    let nowhere: PathBuf = root.join("no-scratch");
    let mut homeless: ProcessLinter = process_linter(
        environment(broken.as_path(), &[]).as_slice(),
        LINTER_LIMITS,
        nowhere.clone(),
    );
    for _ in 0..2 {
        let reason: String =
            lint(&mut homeless, root.as_path(), b"a.md", b"a\n").expect_err("no file");
        assert!(
            reason.starts_with(
                format!(
                    "cli-git could not create the temporary linter configuration in {}: ",
                    nowhere.display()
                )
                .as_str()
            ),
            "{reason}"
        );
    }
    remove(root.as_path());
}

/// Every child failure has its own sentence.
#[test]
fn child_failures_read_as_sentences() {
    let program: &Path = Path::new("/opt/lint");
    for (failure, sentence) in [
        (
            ChildFailure::Files(String::from("disk full")),
            "cli-git could not create the temporary files for monochromatic-lint's input and output: disk full",
        ),
        (
            ChildFailure::Start(String::from("denied")),
            "cli-git could not start monochromatic-lint at /opt/lint: denied",
        ),
        (
            ChildFailure::Wait(String::from("interrupted")),
            "cli-git could not wait for monochromatic-lint: interrupted",
        ),
        (
            ChildFailure::TimedOut(Duration::from_secs(60)),
            "monochromatic-lint did not finish within 60000 milliseconds and was stopped; run the command again, \
             run monochromatic-lint by hand on the file to see why it stalls, or set the policy markdown/autofix \
             to off to skip the check",
        ),
        (
            ChildFailure::TooLarge {
                stream: "standard error",
                limit: 7,
            },
            "monochromatic-lint wrote more than 7 bytes to its standard error and was stopped",
        ),
    ] {
        assert_eq!(child_failure_reason(&failure, program), sentence);
    }
}

/// The linter built from this repository,
///  by the variable the gate image sets.
fn real_linter() -> PathBuf {
    let Some(path) = std::env::var_os(LINTER_VARIABLE) else {
        panic!(
            "{LINTER_VARIABLE} is unset: build package/linter/monochromatic-lint and name its executable in \
             {LINTER_VARIABLE}; the gate image sets it to the linter it builds"
        );
    };
    return PathBuf::from(path);
}

/// A repository directory with an LFS endpoint,
///  the PNG rule and one tracked image.
fn lfs_tree(root: &Path) -> PathBuf {
    let top: PathBuf = root.join("top");
    std::fs::create_dir_all(top.join("pkg/asset")).expect("tree");
    std::fs::write(
        top.join(".lfsconfig"),
        b"[lfs]\n\turl = https://lfs.example.com/org/repo\n",
    )
    .expect("lfsconfig");
    std::fs::write(
        top.join(".gitattributes"),
        b"*.png filter=lfs diff=lfs merge=lfs -text\n",
    )
    .expect("attributes");
    std::fs::write(top.join("pkg/asset/shot.png"), b"image bytes").expect("image");
    return top;
}

/// The object URL of `pkg/asset/shot.png`;
///  the digest is `sha256sum` of `image bytes`.
const SHOT_URL: &str = "https://lfs.example.com/org/repo/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png";

/// The real linter rewrites a relative image link,
///  keeps an exact one,
///  reports a stale
/// object URL as a remaining finding,
///  honors the exclude option,
///  and lints a name that is
/// not UTF-8 and one that starts with `-`.
#[test]
fn the_real_linter_meets_the_contract() {
    let root: PathBuf = fixture("linter-real");
    let top: PathBuf = lfs_tree(root.as_path());
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir(&bin).expect("bin");
    std::os::unix::fs::symlink(real_linter(), bin.join("monochromatic-lint")).expect("link");
    let mut linter: ProcessLinter = process_linter(
        environment(bin.as_path(), &[]).as_slice(),
        LINTER_LIMITS,
        root.clone(),
    );
    let rewritten: String = format!("# T\n\n![shot]({SHOT_URL})\n");
    for name in [&b"pkg/README.md"[..], b"pkg/caf\xe9.md", b"pkg/-x.mdx"] {
        assert_eq!(
            lint(
                &mut linter,
                top.as_path(),
                name,
                b"# T\n\n![shot](asset/shot.png)\n"
            ),
            Ok(LintRun {
                fixed: rewritten.clone().into_bytes(),
                remaining: Vec::new(),
            }),
            "{:?}",
            std::ffi::OsStr::from_bytes(name)
        );
    }
    assert_eq!(
        lint(
            &mut linter,
            top.as_path(),
            b"pkg/README.md",
            rewritten.as_bytes()
        ),
        Ok(LintRun {
            fixed: rewritten.clone().into_bytes(),
            remaining: Vec::new(),
        })
    );
    let stale: String = format!(
        "![shot](https://lfs.example.com/org/repo/{}/pkg/asset/gone.png)\n",
        "b".repeat(64)
    );
    assert_eq!(
        lint(
            &mut linter,
            top.as_path(),
            b"pkg/README.md",
            stale.as_bytes()
        ),
        Ok(LintRun {
            fixed: stale.clone().into_bytes(),
            remaining: vec![RemainingFinding {
                rule: crate::config_schema::MarkdownRule::LfsImageUrl,
                line: 1,
                column: 1,
                message: String::from(
                    "Object URL names pkg/asset/gone.png, which no longer exists in the repository."
                ),
            }],
        })
    );
    let mut excluding: ProcessLinter = process_linter(
        environment(bin.as_path(), &[]).as_slice(),
        LINTER_LIMITS,
        root.clone(),
    );
    let mut options: MarkdownAutofixOptions = default_options();
    options.exclude = vec![String::from("pkg/")];
    assert_eq!(
        excluding.lint(&LintRequest {
            top_level: top.as_path(),
            options: &options,
            path: b"pkg/README.md",
            source: b"![shot](asset/shot.png)\n",
        }),
        Ok(LintRun {
            fixed: b"![shot](asset/shot.png)\n".to_vec(),
            remaining: Vec::new(),
        })
    );
    remove(root.as_path());
}

/// A linter run that takes long prints notices that name the linter and the file it checks, and still returns its result.
#[test]
fn a_linter_run_that_takes_long_names_the_file_it_checks() {
    let root: PathBuf = fixture("linter-notices");
    let slow: PathBuf = stand_in(&root.join("slow"), "sleep 0.5\ncat");
    let sink: SharedSink = SharedSink::default();
    let mut linter: ProcessLinter = process_linter(environment(slow.as_path(), &[]).as_slice(), LINTER_LIMITS, root.clone())
        .with_notices(
            crate::wait_progress::Cadence { grace: Duration::from_millis(100), interval: Duration::from_millis(100) },
            Box::new(sink.clone()),
        );
    let run: Result<LintRun, String> = lint(&mut linter, root.as_path(), b"docs/a.md", b"a\n");
    assert!(run.is_ok(), "{run:?}");
    let text: String = sink.text();
    assert!(text.lines().count() >= 3, "{text}");
    for line in text.lines() {
        assert!(
            line.starts_with("cli-git: still waiting for monochromatic-lint to check docs/a.md (process "),
            "{text}"
        );
        assert!(line.contains("stopping it after 60 seconds"), "{text}");
    }
    remove(root.as_path());
}
