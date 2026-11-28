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
//! except the root boot gate's own text and the nested-dataset units' text,
//!  which the one-time ZFS setup installs at boot
//!  by the owner's cited exception of 2026-11-25
//!  (`doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`;
//!  step 19's accepted design,
//!  `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`),
//!  and every other unit text this release writes still carries none;
//! never asks the user manager to enable,
//! link,
//! preset or default anything,
//! and speaks to the manager only with the
//!  members on the list below.
//! What this reading does not see,
//! and what a reader therefore checks by hand:
//!  a start whose program is a value rather than a literal
//!  (`Command::new(EXECUTABLE_SYSTEMCTL)`,
//!  whose path `checkout_boot_units/boot_unit_names.rs` holds),
//!  a `member:` whose name is a constant or whose literal a line break separates from its key,
//!  a unit text or an install line that a macro of another file expands to,
//!  and a release file whose name this guard reads as test code
//!  (`testing/`,
//!  `binary/`,
//!  a name ending `_tests.rs` or holding `test_support`),
//!  which `lib.rs` gates with `#[cfg(test)]` today.
//!  The manager's program is read where the call's `(` ends its line and its literal stands on the next
//!  one, and the import that renames the type (`use std::process::Command as Cmd;`) is a finding of its own,
//!  so a start whose type another name spells is refused where that name is brought in.
//!  A literal further than the next code line from the call's `(`, and one a comment line separates from
//!  it, are not read.
//!  A comment is a line whose first characters are `//`,
//!  so a banned text written after code on a line is a finding.
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
//!  guard keeps that out of every other text,
//! and the call that would enable one out of the code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseSources()) expect(codeLines(file).filter(forbidden)).toEqual([]); expect(membersCalled()).toSubsetOf(ALLOWED);
//! ```

use std::path::{
    Path,
    PathBuf,
};

/// The shared scan's text reading,
/// so the import that renames the type of a start is judged by the same rule every other text ban uses.
use super::lines_holding_one_of;

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
const ALLOWED_MEMBERS: &[&str] = &[
    "Hello",
    "StartUnit",
    "RestartUnit",
    "StopUnit",
    "ResetFailedUnit",
    "GetUnit",
    "Get",
];

/// Texts that no release code line may carry:
/// the manager's calls that make a unit start at startup or login,
/// the program
///  that makes them from a shell,
///  and the import that brings the type of a start in under another name.
/// The manager's own program is read by `starts_the_manager_s_program` instead of by a text,
/// because a start may spell its path (`Command::new("/usr/bin/systemctl")`).
/// Why `Command as ` is here:
///  `Cmd::new("systemctl")` after `use std::process::Command as Cmd;` starts the same program,
///  and the line that says the name is the import (the alpha.52 read's L2).
const FORBIDDEN_IN_CODE: &[&str] = &[
    "EnableUnitFiles",
    "LinkUnitFiles",
    "AddDependencyUnitFiles",
    "PresetUnitFiles",
    "SetDefaultTarget",
    "MaskUnitFiles",
    "WantedBy=",
    "RequiredBy=",
    "[Install]",
    "Command as ",
];

/// What:
///  The justified exceptions:
///  a file,
///  the text it may carry,
///  and the line that use is written on.
/// Why:
///  The removal of the link an earlier release made is the one use this guard allows,
///  and an entry that named only the file and the text excused a second `.wants` line added beside it.
///  The line is what an entry names:
///  a use swapped for another leaves the line this entry names holding nothing,
///  so the list is reported stale rather than excusing the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALLOWED_CODE = [['service/service_provision_earlier_service_retirement.rs', 'sockets.target.wants', '.join("sockets.target.wants")']];
/// ```
const ALLOWED_CODE: &[(&str, &str, &str)] = &[(
    "service/service_provision_earlier_service_retirement.rs",
    "sockets.target.wants",
    ".join(\"sockets.target.wants\")",
)];

/// What:
///  `the_entry_of_a_wants_line(relative, line, taken)` is the index of the entry that excuses that line,
///  where no earlier line took it,
///  or nothing where no entry names it.
/// Why:
///  The one rule an excuse is read by,
///  so a control can plant a swapped use and ask the question the guard asks.
fn the_entry_of_a_wants_line(
    relative: &str,
    line: &str,
    taken: &[bool],
) -> Option<usize> {
    return ALLOWED_CODE.iter().enumerate().position(|(at, entry)| {
        let (file, allowed_text, written) = entry;
        return !taken[at]
            && *file == relative
            && line.contains(allowed_text)
            && line.trim() == *written;
    });
}

/// Whether this release code line starts the system manager's own program,
/// however the literal is spelled:
/// the bare name,
/// a path that ends in it,
/// or a raw literal.
/// Why:
///  the ban read one literal (`Command::new("systemctl`),
///  so `Command::new("/usr/bin/systemctl").arg("enable")` passed it,
///  and `checkout_boot_units/boot_unit_names.rs` holds the path as `EXECUTABLE_SYSTEMCTL`.
///  A start whose program is a value names no literal here,
///  which this guard's top doc says.
fn starts_the_manager_s_program(line: &str) -> bool {
    let Some(at) = line.find("Command::new") else {
        return false;
    };
    let named: &str = line[at + "Command::new".len()..].trim_start();
    let Some(inside_the_call) = named.strip_prefix('(') else {
        return false;
    };
    let arguments: &str = inside_the_call.trim_start();
    let literal: &str = if let Some(after) = arguments.strip_prefix("r\"") {
        after
    } else if let Some(after) = arguments.strip_prefix('"') {
        after
    } else {
        return false;
    };
    let spelled: &str = literal.split('"').next().unwrap_or("");
    return spelled == "systemctl" || spelled.ends_with("/systemctl");
}

/// What:
///  `fn lines_starting_the_manager_s_program(text: &str) -> Vec<(usize, String)>` lists the code lines of one source
///  that start the manager's own program,
///  each with the number of the line the start is written on,
///  joining a call whose `(` ends its line with the code line after it.
/// Why:
///  A start may put its literal on the line after the call's `(`,
///  and that literal is the only thing that says which program it starts
///  (the alpha.52 read's L2).
///  Only the next code line is joined, which is the line a literal there stands on;
///  a literal further down, or one a comment line separates from the call, is no reading here,
///  and this guard's top doc says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linesStartingTheManagerSProgram(text: string): [number, string][]
/// ```
fn lines_starting_the_manager_s_program(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<(usize, &str)> = code_lines(text);
    let mut found: Vec<(usize, String)> = Vec::new();
    for (at, (number, line)) in lines.iter().enumerate() {
        if starts_the_manager_s_program(line) {
            found.push((*number, String::from(*line)));
            continue;
        }
        let Some((_, after)) = lines.get(at + 1) else {
            continue;
        };
        let joined: String = format!("{line} {after}");
        if starts_the_manager_s_program(joined.as_str()) {
            found.push((*number, joined));
        }
    }
    return found;
}

/// The unit texts the one-time ZFS setup installs under `multi-user.target`,
/// and the only lines they may carry that start a unit at boot:
///  the install section,
///  and the one target line below.
/// The root boot gate reconciles every clone mount after a reboot,
/// and one unit per nested dataset mounts a dataset the boot left unmounted;
/// the owner chose this cited exception on 2026-11-25
///  over installing the gate without an install section,
///  which would leave every clone worktree an empty folder until a command ran
///  (`doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`;
///  step 19's accepted design,
///  `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`).
/// It is by exact file,
/// never by folder or pattern.
const UNIT_TEXTS_INSTALLED_AT_BOOT: &[&str] = &[
    "checkout_boot_units/boot_gate_unit_text.rs",
    "checkout_boot_units/nested_dataset_unit_text.rs",
];

/// The one target line those unit texts may name.
/// Any other target,
/// and every other install line,
/// stays refused in them too.
const BOOT_TARGET_LINE: &str = "WantedBy=multi-user.target";

/// Whether this release code line is the boot-start exception,
/// given the exact file it sits in:
///  the install section,
///  and the one target line that names `multi-user.target`.
/// Every other install line stays refused in those files too,
/// and both lines stay refused in every other file.
fn is_the_boot_start_exception(
    relative: &str,
    line: &str,
    forbidden: &str,
) -> bool {
    if !UNIT_TEXTS_INSTALLED_AT_BOOT.contains(&relative) {
        return false;
    }
    if forbidden == "[Install]" {
        return true;
    }
    return forbidden == "WantedBy=" && line.contains(BOOT_TARGET_LINE);
}

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
            || [
                "wantedby=",
                "requiredby=",
                "upheldby=",
                "also=",
                "alias=",
                "defaultinstance=",
            ]
            .iter()
            .any(|key| {
                return lower.starts_with(key);
            });
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
fn files_below(
    root: &Path,
    extensions: &[&str],
) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let canonical_root: PathBuf =
        std::fs::canonicalize(root).expect("the walk's own root resolves");
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .expect("a readable source directory")
            .flatten()
        {
            let path: PathBuf = entry.path();
            // The entry's own type, so a link is seen as a link.
            let kind: std::fs::FileType = entry.file_type().expect("an entry's own type");
            if kind.is_symlink() {
                let target: PathBuf = std::fs::canonicalize(&path).unwrap_or_default();
                let uses_record: bool = directory
                    .file_name()
                    .is_some_and(|name| return name == "uses")
                    && target.starts_with(canonical_root.as_path())
                    && target.is_dir();
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
            let extension: String = path
                .extension()
                .map(|text| {
                    return text.to_string_lossy().into_owned();
                })
                .unwrap_or_default();
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
    return name.ends_with("_tests.rs")
        || name.contains("test_support")
        || relative.starts_with("testing/")
        || relative.starts_with("binary/");
}

/// The source root of this package,
/// `src/native`.
fn native_root() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("native");
}

/// The code lines of a text:
/// not blank,
/// not a comment.
fn code_lines(text: &str) -> Vec<(usize, &str)> {
    return text
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed: &str = line.trim_start();
            return !trimmed.is_empty() && !trimmed.starts_with("//");
        })
        .map(|(index, line)| {
            return (index + 1, line);
        })
        .collect();
}

/// The unit files the wrapper writes carry no line that makes a unit start at startup or at login:
/// the source of the templates.
#[test]
fn the_unit_templates_have_no_install_section() {
    let root: PathBuf = native_root().join("service").join("service_units");
    let templates: Vec<String> = files_below(root.as_path(), &["socket", "service"]);
    assert!(
        templates.len() >= 2,
        "the socket and the service template are found: {templates:?}"
    );
    for name in templates {
        let text: String = std::fs::read_to_string(root.join(name.as_str())).expect("a template");
        assert_eq!(
            lines_that_start_at_login(text.as_str()),
            Vec::<String>::new(),
            "{name}"
        );
    }
}

/// The detector finds each forbidden line in a unit text,
/// so the guards above and in the provisioning controls cannot pass for
///  want of looking.
#[test]
fn the_detector_finds_every_install_line_and_skips_comments() {
    for bad in [
        "[Install]",
        "WantedBy=default.target",
        "RequiredBy=sockets.target",
        "UpheldBy=x.target",
        "Also=y.socket",
        "Alias=z.service",
        "DefaultInstance=a",
        "  wantedby=sockets.target",
    ] {
        let text: String = format!("[Unit]\nDescription=x\n{bad}\n");
        assert_eq!(lines_that_start_at_login(text.as_str()).len(), 1, "{bad}");
    }
    assert!(lines_that_start_at_login("[Unit]\n# WantedBy=default.target is not written\n; [Install]\n[Socket]\nListenStream=%t/x.sock\n").is_empty());
}

/// The manager's own program is read however its literal is spelled,
/// and a start of another program,
/// a sentence that mentions the program,
/// or the constant that holds its path is none.
#[test]
fn the_manager_s_program_is_read_however_the_literal_spells_it() {
    for planted in [
        "let _ = Command::new(\"systemctl\").arg(\"enable\");",
        "let _ = Command::new(\"/usr/bin/systemctl\").arg(\"enable\");",
        "let _ = Command::new(r\"systemctl\").arg(\"enable\");",
    ] {
        assert!(starts_the_manager_s_program(planted), "{planted}");
    }
    for fine in [
        "let _ = Command::new(git);",
        "let _ = Command::new(\"git\");",
        "let _ = Command::new(EXECUTABLE_SYSTEMCTL).arg(\"enable\");",
        "pub const EXECUTABLE_SYSTEMCTL: &str = \"/usr/bin/systemctl\";",
        "let advice = format!(\"stop it with `systemctl --user stop {unit}` and run again\");",
    ] {
        assert!(!starts_the_manager_s_program(fine), "{fine}");
    }
}

/// A start whose literal stands on the line after its `(` is read,
/// and a start of another program written the same way is not.
/// Why:
///  the literal was read from the call's own line alone,
///  so `Command::new(` with `"systemctl"` on the next line started the manager with no finding
///  (the alpha.52 read's L2).
#[test]
fn the_manager_s_program_is_read_when_its_literal_is_on_the_next_line() {
    for planted in [
        "fn start() {\n    let _ = std::process::Command::new(\n        \"systemctl\",\n    );\n}\n",
        "fn start() {\n    let _ = Command::new(\n        \"/usr/bin/systemctl\",\n    );\n}\n",
        "fn start() {\n    let _ = Command::new(\n        r\"systemctl\",\n    );\n}\n",
    ] {
        assert_eq!(
            lines_starting_the_manager_s_program(planted).len(),
            1,
            "{planted}"
        );
    }
    for fine in [
        "fn start() {\n    let _ = Command::new(\n        git,\n    );\n}\n",
        "fn start() {\n    let _ = Command::new(git);\n    let _ = Command::new(\n        \"git\",\n    );\n}\n",
        "fn ask() {\n    let _ = Command::new(EXECUTABLE_SYSTEMCTL)\n        .arg(\"enable\");\n}\n",
    ] {
        assert!(
            lines_starting_the_manager_s_program(fine).is_empty(),
            "{fine}"
        );
    }
}

/// A start under another name of the type is a finding where the name is brought in,
/// so `Cmd::new("systemctl")` after `use std::process::Command as Cmd;` is refused,
/// and the type imported under its own name is still no finding.
/// Why:
///  the call spells no `Command::new`,
///  and the import is the one line that says which type the name starts
///  (the alpha.52 read's L2).
#[test]
fn a_start_under_another_name_is_read_where_the_name_is_brought_in() {
    for import in [
        "use std::process::Command as Cmd;\n",
        "use std::process::{Command as Cmd};\n",
    ] {
        let planted: Vec<(String, String)> = vec![(
            String::from("service/planted.rs"),
            format!("fn start() {{\n    {import}    let _ = Cmd::new(\"systemctl\");\n}}\n"),
        )];
        assert_eq!(
            lines_holding_one_of(planted.as_slice(), FORBIDDEN_IN_CODE).len(),
            1,
            "{import}"
        );
    }
    let imported_under_its_own_name: Vec<(String, String)> = vec![(
        String::from("service/planted.rs"),
        String::from("use std::process::Command;\n"),
    )];
    assert!(
        lines_holding_one_of(imported_under_its_own_name.as_slice(), FORBIDDEN_IN_CODE).is_empty(),
        "the type imported under its own name names no other start"
    );
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
    std::os::unix::fs::symlink(&elsewhere, sources.join("uses").join("elsewhere"))
        .expect("a uses link to a folder below the root");
    assert_eq!(
        files_below(root.as_path(), &["service"]),
        vec![String::from("sources/inside.service")],
        "a uses link to a folder below the root is skipped"
    );
    std::os::unix::fs::symlink(
        sources.join("inside.service"),
        sources.join("linked.service"),
    )
    .expect("a link to a file");
    let refused: Result<Vec<String>, Box<dyn std::any::Any + Send>> =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            return files_below(root.as_path(), &["service"]);
        }));
    let message: &str = match &refused {
        Err(payload) => payload.downcast_ref::<String>().map_or("", |text| {
            return text.as_str();
        }),
        Ok(files) => panic!("a link to a file is refused rather than skipped silently: {files:?}"),
    };
    assert!(
        message.contains("linked.service") && message.contains("symbolic link"),
        "the link is named: {message}"
    );
    crate::test_support::remove(root.as_path());
}

/// No release code line enables,
/// links,
/// presets,
/// masks or defaults anything with the manager,
/// starts `systemctl`,
/// or writes an
///  install line;
/// the one text from the list that stays is the removal of the link an earlier release made,
/// and the two unit texts the owner's record names may carry their own install section
/// and `WantedBy=multi-user.target`.
#[test]
fn no_release_code_makes_a_unit_start_at_startup_or_login() {
    let root: PathBuf = native_root();
    let mut found: Vec<String> = Vec::new();
    let mut excused: Vec<bool> = vec![false; ALLOWED_CODE.len()];
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String =
            std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (number, line) in code_lines(text.as_str()) {
            for forbidden in FORBIDDEN_IN_CODE {
                let boot_start: bool =
                    is_the_boot_start_exception(relative.as_str(), line, forbidden);
                if line.contains(forbidden) && !boot_start {
                    found.push(format!("{relative}:{number}: {forbidden}"));
                }
            }
            if starts_the_manager_s_program(line) {
                found.push(format!(
                    "{relative}:{number}: a start of the manager's own program"
                ));
            }
            let wants: bool = line.contains(".wants") || line.contains(".requires");
            if !wants {
                continue;
            }
            let entry: Option<usize> =
                the_entry_of_a_wants_line(relative.as_str(), line, excused.as_slice());
            match entry {
                Some(at) => excused[at] = true,
                None => found.push(format!("{relative}:{number}: a wants or requires folder")),
            }
        }
        for (number, line) in lines_starting_the_manager_s_program(text.as_str()) {
            found.push(format!(
                "{relative}:{number}: a start of the manager's own program: {line}"
            ));
        }
    }
    assert_eq!(found, Vec::<String>::new());
    let mut stale: Vec<String> = Vec::new();
    for (at, (file, allowed_text, written)) in ALLOWED_CODE.iter().enumerate() {
        if excused[at] {
            continue;
        }
        stale.push(format!("{file} ({allowed_text}) {written}"));
    }
    assert!(
        stale.is_empty(),
        "an entry excuses a line that no longer holds it, so the list can only shrink:\n{}",
        stale.join("\n")
    );
    let provision: String = std::fs::read_to_string(
        root.join("service")
            .join("service_provision_earlier_service_retirement.rs"),
    )
    .expect("retirement of the earlier service");
    assert!(
        provision.contains("sockets.target.wants"),
        "the exception is still needed; remove it from the list when the code goes"
    );
}

/// What:
///  The entry names the line it excuses,
///  in the file it names,
///  and one entry excuses one line.
/// Why:
///  The review of release 0.6.1-alpha.50 found this list excused a file and a text by existence alone
///  (`source_guard/no_start_at_login_tests.rs:108`).
#[test]
fn the_excuse_names_the_line_it_covers() {
    let taken: Vec<bool> = vec![false; ALLOWED_CODE.len()];
    assert_eq!(
        the_entry_of_a_wants_line(
            "service/service_provision_earlier_service_retirement.rs",
            "        .join(\"sockets.target.wants\")",
            taken.as_slice(),
        ),
        Some(0),
        "the line the entry names is the use it excuses"
    );
    assert_eq!(
        the_entry_of_a_wants_line(
            "service/service_provision_earlier_service_retirement.rs",
            "        .join(\"graphical.target.wants\")",
            taken.as_slice(),
        ),
        None,
        "a use swapped for another is excused by no entry"
    );
    assert_eq!(
        the_entry_of_a_wants_line(
            "service/another_file.rs",
            "        .join(\"sockets.target.wants\")",
            taken.as_slice(),
        ),
        None,
        "the same line in another file is excused by no entry"
    );
    assert_eq!(
        the_entry_of_a_wants_line(
            "service/service_provision_earlier_service_retirement.rs",
            "        .join(\"sockets.target.wants\")",
            &[true],
        ),
        None,
        "one entry excuses one line"
    );
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
        let text: String =
            std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (_, line) in code_lines(text.as_str()) {
            let Some(at) = line.find("member: \"") else {
                continue;
            };
            let rest: &str = &line[at + "member: \"".len()..];
            let name: &str = rest.split('"').next().unwrap_or("");
            called.push(name.to_string());
        }
    }
    assert!(
        called.len() >= ALLOWED_MEMBERS.len() - 1,
        "the client's targets are found: {called:?}"
    );
    let outside: Vec<&String> = called
        .iter()
        .filter(|name| {
            return !ALLOWED_MEMBERS.contains(&name.as_str());
        })
        .collect();
    assert!(outside.is_empty(), "members outside the list: {outside:?}");
}

/// Whether this release code line asks the manager to read every unit it holds:
///  the `Reload` and `DaemonReload` members,
///  a `reload` call spelled some other way,
///  and the shell's own `daemon-reload`.
/// A unit text's own `ExecReload=` line is not one of them:
///  it names the line a reload of that one unit runs,
///  and the manager re-reads only that unit
///  (`doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`).
fn asks_the_manager_for_a_reload(line: &str) -> bool {
    // What: the line with every `ExecReload` taken out, so a unit text's own key is not read as a request.
    let outside_a_unit_text: String = line.replace("ExecReload", "");
    // What: the three spellings left are the member, the quoted call and the shell's own reply to a new unit file.
    return outside_a_unit_text.contains("Reload")
        || outside_a_unit_text.contains("\"reload\"")
        || outside_a_unit_text.contains("daemon-reload");
}

/// What:
///  No release code line asks the manager to read every unit it holds,
/// on any path:
/// no `Reload` member,
/// no `DaemonReload`,
/// no
///  `reload` call and no `daemon-reload`.
/// A unit text's own `ExecReload=` line is not such a request:
///  it names the line that reloads that one unit,
///  and the record accepts it beside the gate
///  (`doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`).
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
/// expect(releaseCodeLines().filter(asksTheManagerForAReload)).toEqual([]);
/// ```
#[test]
fn no_release_code_line_makes_the_manager_read_every_unit_it_holds() {
    let root: PathBuf = native_root();
    let mut found: Vec<String> = Vec::new();
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String =
            std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (number, line) in code_lines(text.as_str()) {
            if asks_the_manager_for_a_reload(line) {
                found.push(format!("{relative}:{number}: {line}"));
            }
        }
    }
    assert_eq!(
        found,
        Vec::<String>::new(),
        "nothing of this wrapper may make the manager read every unit it holds"
    );
    // What: The list the members are checked against is the reason a `Reload` member cannot be added, so it is checked
    //       while the absence is.
    assert!(
        !ALLOWED_MEMBERS.iter().any(|member| {
            return member.contains("Reload");
        }),
        "a reload is no member this wrapper may call: {ALLOWED_MEMBERS:?}"
    );
}

/// What:
///  The boot-start exception is one exact file at a time,
///  and one target line at a time.
/// Why:
///  the owner's exception names the root boot gate and the nested-dataset units,
/// so the same two lines in another release file,
/// and another target in the same file,
/// stay refused,
/// and the list cannot outlive the two texts that need it
///  (`doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`).
#[test]
fn the_boot_start_exception_is_one_file_and_one_target() {
    let elsewhere: &str = "service/service_instance.rs";
    for file in UNIT_TEXTS_INSTALLED_AT_BOOT {
        let text: String =
            std::fs::read_to_string(native_root().join(file)).expect("a unit text file");
        assert!(
            text.contains("[Install]") && text.contains(BOOT_TARGET_LINE),
            "the exception is still needed in {file}; remove it from the list when the file goes"
        );
        assert!(
            is_the_boot_start_exception(file, "String::from(\"[Install]\")", "[Install]"),
            "{file}"
        );
        assert!(
            is_the_boot_start_exception(
                file,
                "String::from(\"WantedBy=multi-user.target\")",
                "WantedBy="
            ),
            "{file}"
        );
        assert!(
            !is_the_boot_start_exception(
                file,
                "String::from(\"WantedBy=graphical.target\")",
                "WantedBy="
            ),
            "{file}"
        );
        assert!(
            !is_the_boot_start_exception(
                file,
                "String::from(\"RequiredBy=x.target\")",
                "RequiredBy="
            ),
            "{file}"
        );
    }
    assert!(!is_the_boot_start_exception(
        elsewhere,
        "String::from(\"[Install]\")",
        "[Install]"
    ));
    assert!(!is_the_boot_start_exception(
        elsewhere,
        "String::from(\"WantedBy=multi-user.target\")",
        "WantedBy="
    ));
}

/// What:
///  The narrowed reload detector still refuses a manager reload in every spelling,
///  and passes a unit text's own `ExecReload=` line.
/// Why:
///  the record narrows the guard by that one key,
/// so the request the guard exists for has to be shown still refused,
/// and the key that reloads one unit accepted
///  (`doc/decision/cli-git-zfs-boot-units-start-at-boot-and-the-reload-guard.md`).
#[test]
fn the_reload_detector_refuses_every_spelling_of_a_manager_reload() {
    for planted in [
        "member: \"Reload\"",
        "member: \"DaemonReload\"",
        "call(\"reload\")",
        "Command::new(\"systemctl\").arg(\"daemon-reload\")",
        "args([\"daemon-reload\"])",
    ] {
        assert!(asks_the_manager_for_a_reload(planted), "{planted}");
    }
    for a_unit_text_line in [
        "format!(\"ExecReload={reconcile}\")",
        "line.starts_with(\"ExecReload=\")",
        "UnitTextRefusal::MissingLine { line: \"ExecReload=\", unit: BOOT_GATE_UNIT_NAME }",
    ] {
        assert!(
            !asks_the_manager_for_a_reload(a_unit_text_line),
            "{a_unit_text_line}"
        );
    }
}
