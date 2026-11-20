//! What:
//!  Guard for the owner's rule that nothing of the wrapper starts at system startup or at login.
//! The source this release ships has
//!  no `[Install]` section,
//! no `WantedBy=`,
//! `RequiredBy=`,
//! `UpheldBy=`,
//! `Also=`,
//! `Alias=` or `DefaultInstance=` line in a unit it
//!  writes,
//! never asks the user manager to enable,
//! link,
//! preset or default anything,
//! and speaks to the manager only with the
//!  members on the list below.
//! Why:
//!  Owner,
//! 2026-11-07:
//! "Were you implying our wrapper or something in it auto starts itself on system startup or user login?
//! It
//!  should never do that!
//! Follow Bazel."
//! Then:
//! "Systemd units are fine."
//! The commit service is started by the first command that
//!  needs it (socket activation of a runtime unit),
//! and exits after three idle hours,
//! like Bazel's server.
//! A unit with an
//!  `[Install]` section is what `systemctl --user enable` links into a `*.wants` folder that the login session starts,
//! so the
//!  guard keeps that out of the texts,
//! and the call that would enable one out of the code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseSources()) expect(codeLines(file).filter(forbidden)).toEqual([]); expect(membersCalled()).toSubsetOf(ALLOWED);
//! ```

use std::path::{Path, PathBuf};

/// The members of the user manager (and of the bus daemon) the wrapper may call:
/// say hello,
/// reset a failed unit,
/// start,
/// restart a unit the manager already holds,
///  stop,
/// and read a unit's state.
/// A reload is deliberately absent:
/// it makes the manager read every unit it holds,
/// at a cost that grows with everything
///  else it runs,
/// and nothing of this wrapper may ask for it
/// (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
/// Nothing that enables,
/// links,
/// presets,
/// masks or changes the default target is on it.
/// A restart neither writes a `[Install]` line nor links a unit into a `*.wants` folder:
/// it is the manager's own stop and start of a unit this wrapper asked for,
/// so a command that needs the answer of a service whose program stopped inside its own process gets one
/// (`systemd_dbus.rs`,
/// `restart_unit`;
/// `SPEC.md`,
/// the wait for the commit service),
/// and it starts nothing at startup or at login.
const ALLOWED_MEMBERS: &[&str] = &["Hello", "StartUnit", "RestartUnit", "StopUnit", "ResetFailedUnit", "GetUnit", "Get"];

/// Texts that no release code line may carry:
/// the manager's calls that make a unit start at startup or login,
/// and the program
///  that makes them from a shell.
const FORBIDDEN_IN_CODE: &[&str] = &[
    "EnableUnitFiles",
    "LinkUnitFiles",
    "AddDependencyUnitFiles",
    "PresetUnitFiles",
    "SetDefaultTarget",
    "MaskUnitFiles",
    "Command::new(\"systemctl",
    "WantedBy=",
    "RequiredBy=",
    "[Install]",
];

/// The justified exceptions:
/// a file,
/// and the text it may carry.
/// The removal of the link an earlier release made.
const ALLOWED_CODE: &[(&str, &str)] = &[("service/service_provision_earlier_service_retirement.rs", "sockets.target.wants")];

/// The lines of a unit's text that would make it start at startup or at login (`systemd.unit(5)`,
/// "[Install] Section Options"):
///  the section header and the options that only an install reads.
/// Comment lines are skipped.
pub(crate) fn lines_that_start_at_login(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        let lower: String = trimmed.to_ascii_lowercase();
        let install: bool = lower == "[install]"
            || ["wantedby=", "requiredby=", "upheldby=", "also=", "alias=", "defaultinstance="].iter().any(|key| { return lower.starts_with(key); });
        if install {
            found.push(trimmed.to_string());
        }
    }
    return found;
}

/// Every file below `root` as a path relative to it,
/// found with a work stack,
/// keeping those whose extension is in `extensions`,
/// and never reading through a link:
/// a `uses` folder holds one link per source folder its code uses
/// (`AGENTS.md`,
/// rule `LYD`),
/// so a link that names a folder below `root` is skipped there,
/// and any other link refuses the walk instead of being skipped silently,
/// because a link this walk passed over would put its target outside the guard.
fn files_below(root: &Path, extensions: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let canonical_root: PathBuf = std::fs::canonicalize(root).expect("the walk's own root resolves");
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable source directory").flatten() {
            let path: PathBuf = entry.path();
            // The entry's own type, so a link is seen as a link.
            let kind: std::fs::FileType = entry.file_type().expect("an entry's own type");
            if kind.is_symlink() {
                let target: PathBuf = std::fs::canonicalize(&path).unwrap_or_default();
                let uses_record: bool = directory.file_name().is_some_and(|name| return name == "uses") && target.starts_with(canonical_root.as_path()) && target.is_dir();
                assert!(
                    uses_record,
                    "{} is a symbolic link that is not a `uses` link to a folder below {}; this guard cannot prove what it reads, so it refuses rather than skipping it",
                    path.display(),
                    root.display()
                );
                continue;
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            let extension: String = path.extension().map(|text| { return text.to_string_lossy().into_owned(); }).unwrap_or_default();
            if extensions.contains(&extension.as_str()) {
                let relative: &Path = path.strip_prefix(root).expect("a path below the root");
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    return found;
}

/// Whether a file is test code by its name,
/// which is how the release sources are told apart here.
fn is_test_file(relative: &str) -> bool {
    let name: &str = relative.rsplit('/').next().unwrap_or(relative);
    return name.ends_with("_tests.rs") || name.contains("test_support") || relative.starts_with("testing/") || relative.starts_with("binary/");
}

/// The source root of this package,
/// `src/native`.
fn native_root() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
}

/// The code lines of a text:
/// not blank,
/// not a comment.
fn code_lines(text: &str) -> Vec<(usize, &str)> {
    return text
        .lines()
        .enumerate()
        .filter(|(_, line)| { let trimmed: &str = line.trim_start(); return !trimmed.is_empty() && !trimmed.starts_with("//"); })
        .map(|(index, line)| { return (index + 1, line); })
        .collect();
}

/// The unit files the wrapper writes carry no line that makes a unit start at startup or at login:
/// the source of the templates.
#[test]
fn the_unit_templates_have_no_install_section() {
    let root: PathBuf = native_root().join("service").join("service_units");
    let templates: Vec<String> = files_below(root.as_path(), &["socket", "service"]);
    assert!(templates.len() >= 2, "the socket and the service template are found: {templates:?}");
    for name in templates {
        let text: String = std::fs::read_to_string(root.join(name.as_str())).expect("a template");
        assert_eq!(lines_that_start_at_login(text.as_str()), Vec::<String>::new(), "{name}");
    }
}

/// The detector finds each forbidden line in a unit text,
/// so the guards above and in the provisioning controls cannot pass for
///  want of looking.
#[test]
fn the_detector_finds_every_install_line_and_skips_comments() {
    for bad in ["[Install]", "WantedBy=default.target", "RequiredBy=sockets.target", "UpheldBy=x.target", "Also=y.socket", "Alias=z.service", "DefaultInstance=a", "  wantedby=sockets.target"] {
        let text: String = format!("[Unit]\nDescription=x\n{bad}\n");
        assert_eq!(lines_that_start_at_login(text.as_str()).len(), 1, "{bad}");
    }
    assert!(lines_that_start_at_login("[Unit]\n# WantedBy=default.target is not written\n; [Install]\n[Socket]\nListenStream=%t/x.sock\n").is_empty());
}

/// What:
///  The walk skips a `uses` link that names a folder below its root,
///  and refuses every other link,
///  naming the link it refused.
/// Why:
///  The walk skipped every link,
///  so a symlinked unit template or source was silently outside this guard.
#[test]
fn a_link_that_is_not_a_uses_folder_link_is_refused() {
    let root: PathBuf = crate::test_support::fixture("no-start-at-login-links");
    let sources: PathBuf = root.join("sources");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(&sources).expect("a source folder");
    std::fs::create_dir(&elsewhere).expect("the folder the uses link names");
    std::fs::write(sources.join("inside.service"), "[Unit]\n").expect("a template");
    std::fs::create_dir(sources.join("uses")).expect("the uses folder");
    std::os::unix::fs::symlink(&elsewhere, sources.join("uses").join("elsewhere")).expect("a uses link to a folder below the root");
    assert_eq!(files_below(root.as_path(), &["service"]), vec![String::from("sources/inside.service")], "a uses link to a folder below the root is skipped");
    std::os::unix::fs::symlink(sources.join("inside.service"), sources.join("linked.service")).expect("a link to a file");
    let refused: Result<Vec<String>, Box<dyn std::any::Any + Send>> = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return files_below(root.as_path(), &["service"]); }));
    let message: &str = match &refused {
        Err(payload) => payload.downcast_ref::<String>().map_or("", |text| { return text.as_str(); }),
        Ok(files) => panic!("a link to a file is refused rather than skipped silently: {files:?}"),
    };
    assert!(message.contains("linked.service") && message.contains("symbolic link"), "the link is named: {message}");
    crate::test_support::remove(root.as_path());
}

/// No release code line enables,
/// links,
/// presets,
/// masks or defaults anything with the manager,
/// starts `systemctl`,
/// or writes an
///  install line;
/// the one text from the list that stays is the removal of the link an earlier release made.
#[test]
fn no_release_code_makes_a_unit_start_at_startup_or_login() {
    let root: PathBuf = native_root();
    let mut found: Vec<String> = Vec::new();
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (number, line) in code_lines(text.as_str()) {
            for forbidden in FORBIDDEN_IN_CODE {
                if line.contains(forbidden) {
                    found.push(format!("{relative}:{number}: {forbidden}"));
                }
            }
            let wants: bool = line.contains(".wants") || line.contains(".requires");
            let excused: bool = ALLOWED_CODE.iter().any(|(file, allowed_text)| { return *file == relative && line.contains(allowed_text); });
            if wants && !excused {
                found.push(format!("{relative}:{number}: a wants or requires folder"));
            }
        }
    }
    assert_eq!(found, Vec::<String>::new());
    let provision: String = std::fs::read_to_string(root.join("service").join("service_provision_earlier_service_retirement.rs")).expect("retirement of the earlier service");
    assert!(provision.contains("sockets.target.wants"), "the exception is still needed; remove it from the list when the code goes");
}

/// The wrapper's client of the manager calls only the members on the list:
/// every `member: "..."` of a target in the release
///  sources is on it.
/// A reload is what that list is there to keep out,
/// so a test beside it names the absence directly too.
#[test]
fn the_manager_is_called_only_with_the_members_on_the_list() {
    let root: PathBuf = native_root();
    let mut called: Vec<String> = Vec::new();
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (_, line) in code_lines(text.as_str()) {
            let Some(at) = line.find("member: \"") else { continue };
            let rest: &str = &line[at + "member: \"".len()..];
            let name: &str = rest.split('"').next().unwrap_or("");
            called.push(name.to_string());
        }
    }
    assert!(called.len() >= ALLOWED_MEMBERS.len() - 1, "the client's targets are found: {called:?}");
    let outside: Vec<&String> = called.iter().filter(|name| { return !ALLOWED_MEMBERS.contains(&name.as_str()); }).collect();
    assert!(outside.is_empty(), "members outside the list: {outside:?}");
}

/// What:
///  No release code line asks the manager to read every unit it holds,
/// on any path:
/// no `Reload` member,
/// no `DaemonReload` and no
///  `reload` call.
/// Why:
///   A reload holds the manager's own event loop for a time that grows with every unit it holds,
/// and on 2026-11-12 one
///  of those,
/// 139,770 ms long,
/// expired the desktop's reply to its window manager and ended the session
/// (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
/// The list above refuses the member;
/// this names the absence wherever it would come back,
/// including a manager of a control's own and a helper that
///  spells the call some other way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(releaseCodeLines().filter((line) => /Reload|"reload"/.test(line))).toEqual([]);
/// ```
#[test]
fn no_release_code_line_makes_the_manager_read_every_unit_it_holds() {
    let root: PathBuf = native_root();
    let mut found: Vec<String> = Vec::new();
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (number, line) in code_lines(text.as_str()) {
            if line.contains("Reload") || line.contains("\"reload\"") {
                found.push(format!("{relative}:{number}: {line}"));
            }
        }
    }
    assert_eq!(found, Vec::<String>::new(), "nothing of this wrapper may make the manager read every unit it holds");
    // What: The list the members are checked against is the reason a `Reload` member cannot be added, so it is checked
    //       while the absence is.
    assert!(!ALLOWED_MEMBERS.iter().any(|member| { return member.contains("Reload"); }), "a reload is no member this wrapper may call: {ALLOWED_MEMBERS:?}");
}
