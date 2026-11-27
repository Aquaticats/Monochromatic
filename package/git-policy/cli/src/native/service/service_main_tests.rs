//! Controls for the words that select the service mode and for the places that may use them.

use super::*;
use crate::service_provision::{
    SERVICE_UNIT,
    SERVICE_UNIT_NAME,
};
// What:
//  `release_sources_in` lists the files of this crate the module tree reaches as release code,
//  and the other three name a source file, read one of them, and give the crate's own source root.
// Why:
//  All four are the shared source scan,
//  so this check walks the tree exactly as every other check of the crate's sources does.
//
// In TS you'd write (pseudocode):
// ```ts
// import { releaseSourcesIn, sourceText, sourceRoot, realRoots } from './source_guard'
// ```
use crate::source_guard::release_sources_in;
use crate::source_guard::source_scan_tests::{
    real_roots,
    source_root,
    source_text,
};
use crate::wrapper_names_old_and_new::COMMAND_WORD;
use std::path::Path;

fn words(texts: &[&str]) -> Vec<OsString> {
    return texts.iter().map(OsString::from).collect();
}

/// Exactly the three words first select the service mode,
/// with any options after them.
#[test]
fn the_three_leading_words_select_the_service_mode() {
    assert!(is_service_start(words(SERVICE_WORDS).as_slice()));
    let mut with_options: Vec<OsString> = words(SERVICE_WORDS);
    with_options.extend(words(&["--journal", "/j", "--path", "/s"]));
    assert!(is_service_start(with_options.as_slice()));
    assert_eq!(
        service_options(with_options.as_slice()),
        Ok(vec![
            String::from("--journal"),
            String::from("/j"),
            String::from("--path"),
            String::from("/s")
        ])
    );
}

/// A unit file an earlier release wrote starts the service with the earlier command word,
///  and the service mode answers it the same,
///  so a helper started by an older unit and a unit that still names the older word both keep working.
#[test]
fn the_earlier_command_word_selects_the_service_mode_the_same() {
    assert_eq!(
        EARLIER_SERVICE_WORDS,
        [COMMAND_WORD.earlier, "service", "run"]
    );
    assert!(is_service_start(words(EARLIER_SERVICE_WORDS).as_slice()));
    let mut with_options: Vec<OsString> = words(EARLIER_SERVICE_WORDS);
    with_options.extend(words(&["--journal", "/j", "--instance", "i"]));
    assert!(is_service_start(with_options.as_slice()));
    assert_eq!(
        service_options(with_options.as_slice()),
        Ok(vec![
            String::from("--journal"),
            String::from("/j"),
            String::from("--instance"),
            String::from("i")
        ])
    );
}

/// The current words start with the program's command word,
///  which is what the unit this release writes passes.
#[test]
fn the_current_service_words_start_with_the_new_command_word() {
    assert_eq!(SERVICE_WORDS, [COMMAND_WORD.current, "service", "run"]);
}

/// Anything else is the wrapper:
/// another word,
/// a word out of place,
/// the words behind a global option of Git (which the
///  management grammar refuses without starting anything),
/// a shorter list,
/// or different case.
#[test]
fn nothing_else_selects_the_service_mode() {
    for other in [
        &[][..],
        &["monochromatic-git-policy-cli"][..],
        &["monochromatic-git-policy-cli", "service"][..],
        &["monochromatic-git-policy-cli", "service", "status"][..],
        &["monochromatic-git-policy-cli", "service", "Run"][..],
        &["Cli-git", "service", "run"][..],
        &["service", "run"][..],
        &["service", "monochromatic-git-policy-cli", "run"][..],
        &[
            "-C",
            "/repo",
            "monochromatic-git-policy-cli",
            "service",
            "run",
        ][..],
        &[
            "--git-dir=/x",
            "monochromatic-git-policy-cli",
            "service",
            "run",
        ][..],
        &["commit", "monochromatic-git-policy-cli", "service", "run"][..],
        &["monochromatic-git-policy-cli", "check", "--all"][..],
        &["status"][..],
    ] {
        assert!(!is_service_start(words(other).as_slice()), "{other:?}");
    }
}

/// A word that is not text cannot be an option of the service,
/// and the refusal names it.
#[test]
fn an_option_that_is_not_text_is_refused() {
    use std::os::unix::ffi::OsStringExt;
    let mut arguments: Vec<OsString> = words(SERVICE_WORDS);
    arguments.push(OsString::from_vec(vec![b'/', 0xff, b'x']));
    assert_eq!(
        service_options(arguments.as_slice()),
        Err(OsString::from_vec(vec![b'/', 0xff, b'x']))
    );
}

/// The unit's `ExecStart` starts the program with the very words that select the mode,
/// so the unit and the dispatch cannot drift.
#[test]
fn the_unit_starts_the_program_with_the_service_mode_words() {
    let exec: &str = SERVICE_UNIT
        .lines()
        .find(|line| {
            return line.starts_with("ExecStart=");
        })
        .expect("an ExecStart line");
    let after_program: Vec<&str> = exec
        .trim_start_matches("ExecStart=")
        .split_whitespace()
        .skip(1)
        .collect();
    assert_eq!(
        after_program
            .iter()
            .take(SERVICE_WORDS.len())
            .copied()
            .collect::<Vec<&str>>(),
        SERVICE_WORDS.to_vec(),
        "{exec}"
    );
    assert!(
        exec.trim_start_matches("ExecStart=")
            .starts_with("@SERVICE_BINARY@ "),
        "{exec}"
    );
    assert!(SERVICE_UNIT_NAME.ends_with(".service"));
    assert!(
        SERVICE_UNIT.lines().any(|line| {
            return line == "SyslogIdentifier=monochromatic-git-policy-cli-service";
        }),
        "the journal identifier stays"
    );
}

/// What:
///  Each word the service mode owns,
///  with the `/`-separated files of this crate that may say it in code.
///  `&str` is borrowed text and `&[&str]` a slice of it,
///  which never carries a hand-counted length.
/// Why:
///  One list read by one pass over the sources keeps the words and the files that may say them beside
///  each other;
///  a slice rather than an array means adding a word edits no count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SERVICE_MODE_WORDS: [string, string[]][] = [['service_signal::install()', ['service/service_main.rs']]]
/// ```
const SERVICE_MODE_WORDS: &[(&str, &[&str])] = &[
    ("service_signal::install()", &["service/service_main.rs"]),
    (
        "push_deadline_advice::mark_service_process()",
        &["service/service_main.rs"],
    ),
    ("SERVICE_WORDS", &["service/service_main.rs", "main.rs"]),
    ("service_main::", &["main.rs"]),
];

/// The process setup of the service is taken in its mode and nowhere else:
/// the wrapper's own start never installs the
///  service's signal lines,
/// never marks itself the service,
/// and the words that select the mode are used by the dispatch alone.
#[test]
fn the_service_setup_and_its_words_are_used_only_by_the_service_mode() {
    // What:
    //  `release_sources_in` lists the files the module tree reaches as release code,
    //  and `source_text` reads one of them;
    //  both come from the shared source scan.
    // Why:
    //  The walk this replaces listed the tree itself and followed the `uses` links,
    //  so it read 104,219 files and 888,676,493 bytes where the tree holds 3,905 Rust sources and
    //  46,090,082 bytes (issue 6598).
    let sources: Vec<String> =
        release_sources_in(&source_root(), &real_roots()).expect("the crate's release sources");
    assert!(sources.len() > 100, "the scan reads the crate's sources");
    // What:
    //  One list of files per word, filled in one pass over the sources: `SERVICE_MODE_WORDS.iter()` walks the
    //  words and `.map(|_| Vec::new())` makes an empty list for each of them.
    // Why:
    //  Each file is read once and its text dropped before the next one,
    //  so the words are answered without the tree's whole text in memory.
    let mut users: Vec<Vec<String>> = SERVICE_MODE_WORDS
        .iter()
        .map(|_| return Vec::new())
        .collect();
    for path in &sources {
        let text: String = source_text(path.as_str());
        // What:
        //  The file's code lines: `lines()` splits the text at newlines, `filter` keeps those that do not start
        //  with `//` once trimmed, and `collect` gathers the borrowed lines into one list.
        // Why:
        //  A word named in a comment is prose about the code and not a use of it.
        let code: Vec<&str> = text
            .lines()
            .filter(|line| return !line.trim_start().starts_with("//"))
            .collect();
        for (index, (needle, _allowed)) in SERVICE_MODE_WORDS.iter().enumerate() {
            if code.iter().any(|line| return line.contains(needle)) {
                users[index].push(path.clone());
            }
        }
    }
    for ((needle, allowed), found) in SERVICE_MODE_WORDS.iter().zip(users.iter()) {
        assert!(
            found
                .iter()
                .all(|path| return allowed.contains(&path.as_str())),
            "{needle} is used by {found:?}, allowed {allowed:?}"
        );
        assert!(!found.is_empty(), "{needle} is used by the dispatch");
    }
}

/// The three process-wide steps come first in the service's start,
/// in this order,
/// before it reads an option or creates a file.
#[test]
fn the_service_start_takes_its_process_setup_first() {
    let text: String = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/service/service_main.rs"),
    )
    .expect("service_main.rs");
    let body: &str = text.split("pub fn run(").nth(1).expect("the run function");
    let position = |needle: &str| -> usize {
        return body
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} is in run"));
    };
    let hooks: usize = position("hooks_off::activate()");
    let signal: usize = position("service_signal::install()");
    let mark: usize = position("mark_service_process()");
    let first_option: usize = position("option(arguments,");
    let refusal: usize = position("socket_passed_in()");
    let first_directory: usize = position("create_dir_all");
    assert!(
        hooks < signal && signal < mark && mark < first_option && first_option < first_directory,
        "{hooks} {signal} {mark} {first_option} {first_directory}"
    );
    assert!(
        refusal < first_directory,
        "the refusal of a start with no socket comes before anything is created"
    );
}

/// A start that serves nothing because the worktree is gone ends without a failure status:
/// the unit is meant to be gone,
/// and `Restart=on-failure` would start it again for a failure status.
#[test]
fn a_start_that_serves_nothing_is_not_a_failure() {
    assert_eq!(EXIT_NOTHING_TO_SERVE, 0);
    for refusal in [EXIT_REFUSED, EXIT_NOT_LISTENING] {
        assert_ne!(EXIT_NOTHING_TO_SERVE, refusal);
    }
}

/// The fewest workers a multi-thread runtime has that the owner's "Full on tokio.
/// Multi-thread."
/// leaves room for.
const FEWEST_WORKERS_OF_A_MULTI_THREAD_RUNTIME: usize = 2;

/// The service's cap keeps it multi-thread:
/// never below two workers.
#[test]
fn the_workers_cap_of_the_service_keeps_it_a_multi_thread_runtime() {
    const { assert!(SERVICE_WORKER_CAP >= FEWEST_WORKERS_OF_A_MULTI_THREAD_RUNTIME) };
}
