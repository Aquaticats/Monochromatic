//! Controls for the words that select the service mode and for the places that may use them.

use super::*;
use crate::wrapper_names_old_and_new::COMMAND_WORD;
use std::path::Path;

fn words(texts: &[&str]) -> Vec<OsString> {
    return texts.iter().map(OsString::from).collect();
}

/// Exactly the three words first select the service mode,
/// with any options after them.
#[test]
fn the_three_leading_words_select_the_service_mode() {
    assert!(is_service_start(words(&SERVICE_WORDS).as_slice()));
    let mut with_options: Vec<OsString> = words(&SERVICE_WORDS);
    with_options.extend(words(&["--journal", "/j", "--path", "/s"]));
    assert!(is_service_start(with_options.as_slice()));
    assert_eq!(service_options(with_options.as_slice()), Ok(vec![String::from("--journal"), String::from("/j"), String::from("--path"), String::from("/s")]));
}

/// A unit file an earlier release wrote starts the service with the earlier command word,
///  and the service mode answers it the same,
///  so a helper started by an older unit and a unit that still names the older word both keep working.
#[test]
fn the_earlier_command_word_selects_the_service_mode_the_same() {
    assert_eq!(EARLIER_SERVICE_WORDS, [COMMAND_WORD.earlier, "service", "run"]);
    assert!(is_service_start(words(&EARLIER_SERVICE_WORDS).as_slice()));
    let mut with_options: Vec<OsString> = words(&EARLIER_SERVICE_WORDS);
    with_options.extend(words(&["--journal", "/j", "--instance", "i"]));
    assert!(is_service_start(with_options.as_slice()));
    assert_eq!(service_options(with_options.as_slice()), Ok(vec![String::from("--journal"), String::from("/j"), String::from("--instance"), String::from("i")]));
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
        &["-C", "/repo", "monochromatic-git-policy-cli", "service", "run"][..],
        &["--git-dir=/x", "monochromatic-git-policy-cli", "service", "run"][..],
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
    let mut arguments: Vec<OsString> = words(&SERVICE_WORDS);
    arguments.push(OsString::from_vec(vec![b'/', 0xff, b'x']));
    assert_eq!(service_options(arguments.as_slice()), Err(OsString::from_vec(vec![b'/', 0xff, b'x'])));
}

/// The words the wrapper starts the service with select the mode and name the worktree's files,
/// so the start and the dispatch cannot drift.
#[test]
fn the_start_names_the_service_mode_words_and_the_worktrees_files() {
    let server = crate::service_server_start_by_the_wrapper::ServerToStart {
        socket: std::path::PathBuf::from("/run/user/1000/monochromatic-git-policy-cli/3f9a.sock"),
        instance: String::from("3f9a"),
        journal: std::path::PathBuf::from("/state/monochromatic-git-policy-cli/service/3f9a"),
        git: std::path::PathBuf::from("/usr/bin/git"),
        program: std::path::PathBuf::from("/bin/git"),
        manager: None,
        named_runtime_dir: None,
        bus_address: None,
    };
    let arguments: Vec<String> = server.arguments().iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
    let first: Vec<String> = arguments.iter().take(SERVICE_WORDS.len()).cloned().collect();
    assert_eq!(first, SERVICE_WORDS.map(String::from).to_vec());
    assert!(arguments.windows(2).any(|pair| { return pair[0] == "--instance" && pair[1] == "3f9a"; }));
    assert!(arguments.windows(2).any(|pair| { return pair[0] == "--path"; }));
    assert!(arguments.windows(2).any(|pair| { return pair[0] == "--journal"; }));
}

/// The source files of this crate that are not controls,
/// as `(path under src/native, text)`.
fn release_sources() -> Vec<(String, String)> {
    fn walk(directory: &Path, root: &Path, found: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(directory).expect("source directory").flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(path.as_path(), root, found);
                continue;
            }
            let relative: String = path.strip_prefix(root).expect("below the root").to_string_lossy().into_owned();
            if relative.ends_with(".rs") && !relative.ends_with("_tests.rs") && !relative.starts_with("binary/") && !relative.starts_with("testing/") {
                found.push((relative, std::fs::read_to_string(&path).expect("source text")));
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native");
    let mut found: Vec<(String, String)> = Vec::new();
    walk(root.as_path(), root.as_path(), &mut found);
    return found;
}

/// The process setup of the service is taken in its mode and nowhere else:
/// the wrapper's own start never installs the
///  service's signal lines,
/// never marks itself the service,
/// and the words that select the mode are used by the dispatch alone.
#[test]
fn the_service_setup_and_its_words_are_used_only_by_the_service_mode() {
    let sources = release_sources();
    assert!(sources.len() > 100, "the scan reads the crate's sources");
    for (needle, allowed) in [
        ("service_signal::install()", vec!["service/service_main.rs"]),
        ("push_deadline_advice::mark_service_process()", vec!["service/service_main.rs"]),
        ("SERVICE_WORDS", vec!["service/service_main.rs", "main.rs"]),
        ("service_main::", vec!["main.rs"]),
    ] {
        let users: Vec<&str> = sources
            .iter()
            .filter(|(_, text)| { return text.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).any(|line| { return line.contains(needle); }); })
            .map(|(path, _)| { return path.as_str(); })
            .collect();
        assert!(users.iter().all(|path| { return allowed.contains(path); }), "{needle} is used by {users:?}, allowed {allowed:?}");
        assert!(!users.is_empty(), "{needle} is used by the dispatch");
    }
}

/// The three process-wide steps come first in the service's start,
/// in this order,
/// before it reads an option or creates a file.
#[test]
fn the_service_start_takes_its_process_setup_first() {
    let text: String = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/service/service_main.rs")).expect("service_main.rs");
    let body: &str = text.split("pub fn run(").nth(1).expect("the run function");
    let position = |needle: &str| -> usize { return body.find(needle).unwrap_or_else(|| { panic!("{needle} is in run") }); };
    let hooks: usize = position("hooks_off::activate()");
    let signal: usize = position("service_signal::install()");
    let mark: usize = position("mark_service_process()");
    let first_option: usize = position("option(arguments,");
    let refusal: usize = position("socket_passed_in()");
    let first_directory: usize = position("create_dir_all");
    assert!(hooks < signal && signal < mark && mark < first_option && first_option < first_directory, "{hooks} {signal} {mark} {first_option} {first_directory}");
    assert!(refusal < first_directory, "the refusal of a start with no socket comes before anything is created");
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
