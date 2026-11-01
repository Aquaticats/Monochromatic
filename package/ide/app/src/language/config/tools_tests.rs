//! The registry's use of the toolchain on disposable projects:
//!  where servers are found,
//!  what environment they get,
//!  when the lookup is asked again,
//!  and that a failure falls back to the plain lookup.

use super::server_environment;
use crate::language::{
    config::{LanguageSetup, Languages, Unavailable},
    launch::{LaunchRequest, ServerLaunch, launch_directly},
    toolchain::{Failed, Provided, RETRY_AFTER, ToolchainRequest, ToolchainState},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    time::Instant,
};

/// A language whose servers are `/bin/sh` (on `PATH` everywhere) and a program only the toolchain provides.
const SCRIPTED: &str = r#"
[language-server.sh-ls]
command = "sh"
args = ["-c", "exit 0"]

[language-server.toolchain-only-ls]
command = "toolchain-only-ls"

[[language]]
name = "scripted"
scope = "source.scripted"
file-types = ["scripted"]
roots = []
language-servers = ["sh-ls", "toolchain-only-ls"]
"#;

thread_local! {
    /// The answers of the test policy,
    ///  the first until it is used,
    ///  then the last one again.
    static ANSWERS: RefCell<Vec<ToolchainState>> = const { RefCell::new(Vec::new()) };
    /// How many times the test policy was asked.
    static ASKED: Cell<usize> = const { Cell::new(0) };
    /// The launch requests the recording policy saw.
    static LAUNCHES: RefCell<Vec<LaunchRequest>> = const { RefCell::new(Vec::new()) };
}

fn policy(_request: &ToolchainRequest) -> ToolchainState {
    ASKED.set(ASKED.get() + 1);
    let asked = ASKED.get();
    let answers = ANSWERS.take();
    let chosen = answers[(asked - 1).min(answers.len() - 1)].clone();
    ANSWERS.set(answers);
    return chosen;
}

fn recording(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    let mut launches = LAUNCHES.take();
    launches.push(request.clone());
    LAUNCHES.set(launches);
    return launch_directly(request);
}

fn answers(states: Vec<ToolchainState>) {
    ASKED.set(0);
    LAUNCHES.set(Vec::new());
    ANSWERS.set(states);
}

fn setup() -> LanguageSetup {
    return LanguageSetup {
        toolchain: policy,
        launch: recording,
        extra_languages: Some(SCRIPTED.to_string()),
        ..LanguageSetup::unconfined()
    };
}

fn project() -> (tempfile::TempDir, PathBuf) {
    let directory = crate::test_scratch::scratch().expect("project");
    let root = directory.path().canonicalize().expect("canonical project");
    return (directory, root);
}

fn tool(directory: &Path, name: &str) {
    fs::create_dir_all(directory).expect("directory");
    fs::write(directory.join(name), "#!/bin/sh\n").expect("program");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).expect("mode");
}

fn provided(directories: &[&Path], warnings: &str) -> ToolchainState {
    let mut owned: Vec<PathBuf> = Vec::new();
    for directory in directories {
        owned.push(directory.to_path_buf());
    }
    return ToolchainState::Provided(Provided {
        directories: owned,
        warnings: warnings.to_string(),
    });
}

fn failed(message: &str) -> ToolchainState {
    return ToolchainState::Failed(Failed {
        message: message.to_string(),
    });
}

fn missing_reason(languages: &Languages, server: &str) -> String {
    let Some(Unavailable::Missing(reason)) = languages.unavailable(server) else {
        panic!(
            "{server} was not reported missing: {:?}",
            languages.unavailable(server)
        );
    };
    return reason.clone();
}

#[test]
fn a_server_in_a_toolchain_directory_is_found_and_the_directory_leads_its_path_read_only() {
    let (_directory, root) = project();
    let tools = root.join("tools/node 26/bin");
    tool(&tools, "toolchain-only-ls");
    answers(vec![provided(&[&tools], "")]);
    let languages = Languages::new(&root, setup()).expect("registry");
    assert_eq!(languages.unavailable("toolchain-only-ls"), None);
    let launch = languages.launch("toolchain-only-ls").expect("launch");
    assert_eq!(
        launch.command,
        tools.join("toolchain-only-ls").to_str().expect("Unicode")
    );
    let path = launch.environment.get("PATH").expect("PATH for the server");
    assert!(path.starts_with(&format!("{}:", tools.display())), "{path}");
    let launches = LAUNCHES.take();
    let mut found_request = None;
    for seen in launches.iter() {
        if seen.server == "toolchain-only-ls" {
            found_request = Some(seen.clone());
            break;
        }
    }
    LAUNCHES.set(launches);
    let request = found_request.expect("launch request");
    assert_eq!(request.read_only_binds, [tools]);
    assert!(languages.toolchain_advisory().is_empty());
}

#[test]
fn without_the_toolchain_the_lookup_and_its_message_are_what_they_were() {
    let (_directory, root) = project();
    for state in [
        ToolchainState::Disabled,
        ToolchainState::Unused("no node program was found".to_string()),
    ] {
        answers(vec![state]);
        let languages = Languages::new(&root, setup()).expect("registry");
        assert_eq!(
            missing_reason(&languages, "toolchain-only-ls"),
            "the program 'toolchain-only-ls' for toolchain-only-ls was not found on PATH"
        );
        assert!(
            languages
                .launch("sh-ls")
                .expect("launch")
                .environment
                .get("PATH")
                .is_none()
        );
        assert!(languages.toolchain_advisory().is_empty());
    }
}

#[test]
fn a_failure_falls_back_to_the_plain_lookup_and_is_told_to_the_reader() {
    let (_directory, root) = project();
    answers(vec![failed("Error: toolchain.lock.json is unreadable")]);
    let languages = Languages::new(&root, setup()).expect("registry");
    // `sh` is on the plain PATH, so its server still starts.
    assert_eq!(languages.unavailable("sh-ls"), None);
    assert_eq!(
        missing_reason(&languages, "toolchain-only-ls"),
        "the program 'toolchain-only-ls' for toolchain-only-ls was not found on PATH"
    );
    let notice = languages.toolchain_advisory().join("\n");
    assert!(
        notice.contains("the toolchain lookup said: Error: toolchain.lock.json is unreadable"),
        "{notice}"
    );
    assert!(notice.contains(&root.display().to_string()), "{notice}");
}

#[test]
fn a_tool_the_lookup_lists_but_does_not_provide_is_explained_with_its_warning() {
    let (_directory, root) = project();
    let empty = root.join("tools/empty");
    fs::create_dir_all(&empty).expect("directory");
    answers(vec![provided(&[&empty], "a warning about the machine")]);
    let languages = Languages::new(&root, setup()).expect("registry");
    let reason = missing_reason(&languages, "toolchain-only-ls");
    assert!(
        reason
            .contains("in the tool directories the toolchain provides for this project or on PATH"),
        "{reason}"
    );
    assert!(
        reason.contains("the toolchain lookup reported:\na warning about the machine"),
        "{reason}"
    );
}

#[test]
fn a_leftover_mise_shim_is_never_started_for_a_project_the_toolchain_serves() {
    let (_directory, root) = project();
    let mise = root.join("bin/mise");
    tool(mise.parent().expect("parent"), "mise");
    let shims = root.join("shims");
    fs::create_dir_all(&shims).expect("directory");
    symlink(&mise, shims.join("toolchain-only-ls")).expect("shim");
    answers(vec![provided(&[&shims], "")]);
    let languages = Languages::new(&root, setup()).expect("registry");
    let reason = missing_reason(&languages, "toolchain-only-ls");
    assert!(
        reason.contains("the only match is the mise shim"),
        "{reason}"
    );
    assert!(reason.contains("cannot run inside the sandbox"), "{reason}");
}

#[test]
fn the_toolchain_is_asked_again_for_a_failure_only_after_the_retry_time() {
    let (_directory, root) = project();
    let tools = root.join("tools/bin");
    tool(&tools, "toolchain-only-ls");
    answers(vec![failed("the lookup failed"), provided(&[&tools], "")]);
    let mut languages = Languages::new(&root, setup()).expect("registry");
    assert_eq!(ASKED.with(Cell::get), 1);
    assert!(!languages.refresh("scripted", None).expect("refresh"));
    assert_eq!(ASKED.with(Cell::get), 1, "asked again at once");
    languages.toolchain.1 = Instant::now()
        .checked_sub(RETRY_AFTER * 2)
        .expect("a clock reading in the past");
    assert!(languages.refresh("scripted", None).expect("refresh"));
    assert_eq!(ASKED.with(Cell::get), 2);
    assert_eq!(languages.unavailable("toolchain-only-ls"), None);
    assert!(languages.toolchain_advisory().is_empty());
}

#[test]
fn a_server_installed_later_is_found_when_the_toolchain_is_asked_again_for_a_missing_one() {
    let (_directory, root) = project();
    let tools = root.join("tools/bin");
    fs::create_dir_all(&tools).expect("directory");
    answers(vec![provided(&[&tools], ""), provided(&[&tools], "")]);
    let mut languages = Languages::new(&root, setup()).expect("registry");
    languages.toolchain.1 = Instant::now().checked_sub(RETRY_AFTER * 2).expect("past");
    // Nothing changed: asked, same answer, still missing, no rebuild.
    assert!(!languages.refresh("scripted", None).expect("refresh"));
    assert_eq!(ASKED.with(Cell::get), 2);
    tool(&tools, "toolchain-only-ls");
    assert!(
        languages.refresh("scripted", None).expect("refresh"),
        "a program that appeared must rebuild"
    );
    assert_eq!(languages.unavailable("toolchain-only-ls"), None);
}

#[test]
fn a_project_without_missing_servers_does_not_ask_the_toolchain_again() {
    let (_directory, root) = project();
    let tools = root.join("tools/bin");
    tool(&tools, "toolchain-only-ls");
    answers(vec![provided(&[&tools], "")]);
    let mut languages = Languages::new(&root, setup()).expect("registry");
    languages.toolchain.1 = Instant::now().checked_sub(RETRY_AFTER * 20).expect("past");
    assert!(!languages.refresh("scripted", None).expect("refresh"));
    assert_eq!(ASKED.with(Cell::get), 1);
}

#[test]
fn server_environment_puts_the_toolchain_first_and_keeps_a_definitions_own_path_behind_it() {
    let own: HashMap<String, String> = HashMap::from([
        ("KEY".to_string(), "value".to_string()),
        ("PATH".to_string(), "/definition/bin".to_string()),
    ]);
    let tools = PathBuf::from("/tools/it's \"quoted\"/-dash/bin");
    let environment = server_environment(
        &own,
        &provided(&[&tools, Path::new("/tools/with:colon")], ""),
    );
    assert_eq!(environment["KEY"], "value");
    assert_eq!(
        environment["PATH"],
        "/tools/it's \"quoted\"/-dash/bin:/definition/bin"
    );
    // Without directories the environment is the definition's, untouched.
    assert_eq!(server_environment(&own, &ToolchainState::Disabled), own);
    assert_eq!(server_environment(&own, &failed("x")), own);
    assert_eq!(server_environment(&own, &provided(&[], "")), own);
}
