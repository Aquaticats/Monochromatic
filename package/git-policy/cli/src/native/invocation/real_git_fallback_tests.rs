//! What:
//!  Controls for the recorded fallback of `resolve_real_git`:
//!  when `PATH` offers no real Git,
//! the Git the commit service's unit records is used after the `PATH` candidate checks,
//!  with a notice,
//! and otherwise the stop stays and names the record.
//! Why:
//!  The coordinator picked this fallback under the owner's delegation,
//! open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`).
//! It must never become a way back to the wrapper,
//!  to a launcher of it,
//! or to a file that is not there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writeUnit(root, { git: join(root, 'real/git') });
//! // expect(resolveRealGit({ path: 'wrapper', serviceUnit })).toMatchObject({ path: join(root, 'real/git') });
//! ```
#![cfg(unix)]

/// The resolver under test and the pieces its results are made of.
use super::{
    Platform, RealGitNotFound, RecordedGit, ResolutionInputs, ResolvedGit, SearchedCandidate, process_resolution_inputs,
    resolve_real_git,
};
use crate::control_terminal::Terminal;
use crate::real_git_candidate::CandidateKind;
use crate::service_provision::{ProvisionHost, SERVICE_UNIT_NAME, unit_texts};
use crate::test_support::{executable, fixture, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  Inputs for a `PATH` that holds only this wrapper,
///  and the unit file `root/unit` as the record.
/// Why:
///  The fixture file `root/wrapper/git` stands in for the running wrapper,
/// so every control can name it as a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inputs = (root) => ({ platform: 'unix', path: `${root}/wrapper`, ownExecutable: `${root}/wrapper/git`, serviceUnit: `${root}/unit` });
/// ```
fn only_the_wrapper(root: &Path) -> ResolutionInputs {
    std::fs::create_dir_all(root.join("wrapper")).expect("wrapper directory");
    executable(root.join("wrapper/git").as_path(), b"\x7fELF wrapper");
    return ResolutionInputs {
        platform: Platform::Unix,
        path: root.join("wrapper").into_os_string(),
        path_extensions: OsString::new(),
        current_directory: root.to_path_buf(),
        common_paths: Vec::<PathBuf>::new(),
        own_executable: root.join("wrapper/git"),
        service_unit: Some(root.join("unit")),
        // The fallback to the unit's record does not depend on a terminal.
        terminal: Terminal::Present,
    };
}

/// Write the unit file `root/unit` the way the wrapper renders it,
/// naming `git`.
fn write_unit(root: &Path, git: &Path) {
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: root.to_path_buf(),
        user_unit_dir: None,
        bus: PathBuf::from("/nonexistent/bus"),
        service_binary: PathBuf::from("/opt/tool/monochromatic-git-policy-cli"),
        git: git.to_path_buf(),
        instance: String::from("x"),
        git_dir: std::path::PathBuf::new(),
        retirement_wait: std::time::Duration::ZERO,
    };
    for (name, text) in unit_texts(&host) {
        if name == SERVICE_UNIT_NAME {
            std::fs::write(root.join("unit"), text).expect("unit file");
        }
    }
}

/// A Git that `PATH` does not show,
///  recorded by the unit,
///  is used with a one-line notice that names the `PATH`,
///  the file,
///  the unit and the way to silence it.
#[test]
fn a_recorded_real_git_is_used_with_a_one_line_notice_when_path_has_only_the_wrapper() {
    let root: PathBuf = fixture("fallback-used");
    let resolution: ResolutionInputs = only_the_wrapper(root.as_path());
    std::fs::create_dir(root.join("recorded")).expect("directory");
    executable(root.join("recorded/git").as_path(), b"\x7fELF real");
    write_unit(root.as_path(), root.join("recorded/git").as_path());
    let resolved: ResolvedGit = resolve_real_git(&resolution).expect("the record is usable");
    assert_eq!(resolved.path, root.join("recorded/git"));
    let notice: String = resolved.notice.expect("a fallback is announced");
    assert_eq!(notice.matches('\n').count(), 1, "{notice}");
    assert!(notice.ends_with('\n'), "{notice}");
    assert!(notice.starts_with("monochromatic-git-policy-cli: no real Git follows this wrapper on PATH ("), "{notice}");
    for named in [
        root.join("wrapper").display().to_string(),
        root.join("recorded/git").display().to_string(),
        root.join("unit").display().to_string(),
        format!("add {} to the PATH", root.join("recorded").display()),
    ] {
        assert!(notice.contains(named.as_str()), "{named}: {notice}");
    }
    remove(root.as_path());
}

/// An unset `PATH` is said in words in the notice,
///  as the failure says it.
#[test]
fn the_notice_names_an_empty_path_in_words() {
    let root: PathBuf = fixture("fallback-empty-path");
    let mut resolution: ResolutionInputs = only_the_wrapper(root.as_path());
    resolution.path = OsString::new();
    std::fs::create_dir(root.join("recorded")).expect("directory");
    executable(root.join("recorded/git").as_path(), b"\x7fELF real");
    write_unit(root.as_path(), root.join("recorded/git").as_path());
    let notice: String = resolve_real_git(&resolution).expect("usable").notice.expect("announced");
    assert!(notice.contains("PATH (empty or unset);"), "{notice}");
    remove(root.as_path());
}

/// A real Git on `PATH` is used as before:
///  the record is not consulted and no notice is made.
#[test]
fn a_real_git_on_path_wins_and_gives_no_notice() {
    let root: PathBuf = fixture("fallback-not-needed");
    let mut resolution: ResolutionInputs = only_the_wrapper(root.as_path());
    std::fs::create_dir(root.join("real")).expect("directory");
    executable(root.join("real/git").as_path(), b"\x7fELF real");
    std::fs::create_dir(root.join("recorded")).expect("directory");
    executable(root.join("recorded/git").as_path(), b"\x7fELF other");
    write_unit(root.as_path(), root.join("recorded/git").as_path());
    resolution.path = OsString::from(format!("{}:{}", root.join("wrapper").display(), root.join("real").display()));
    let resolved: ResolvedGit = resolve_real_git(&resolution).expect("PATH has a real Git");
    assert_eq!(resolved, ResolvedGit { path: root.join("real/git"), notice: None });
    remove(root.as_path());
}

/// No unit file named by the inputs,
///  and a unit file that is not there,
///  and a unit that names no Git,
///  each leave the stop with nothing recorded.
#[test]
fn no_record_keeps_the_stop_and_names_no_record() {
    let root: PathBuf = fixture("fallback-no-record");
    let mut resolution: ResolutionInputs = only_the_wrapper(root.as_path());
    resolution.service_unit = None;
    let without_unit: RealGitNotFound = resolve_real_git(&resolution).expect_err("no unit named");
    assert_eq!(without_unit.recorded, None);
    resolution.service_unit = Some(root.join("unit"));
    let missing_file: RealGitNotFound = resolve_real_git(&resolution).expect_err("no unit file");
    assert_eq!(missing_file.recorded, None);
    std::fs::write(root.join("unit"), b"[Service]\nExecStart=/bin/service --instance x\n").expect("unit without Git");
    let names_nothing: RealGitNotFound = resolve_real_git(&resolution).expect_err("no Git in the unit");
    assert_eq!(names_nothing.recorded, None);
    assert!(!names_nothing.to_string().contains("recorded by the commit service unit"), "{names_nothing}");
    remove(root.as_path());
}

/// A record that names this wrapper,
///  a link to it,
///  or a launcher with a launch record beside it is refused as a `PATH` candidate is,
///  and the failure names the record and the verdict.
#[test]
fn a_record_naming_the_wrapper_a_link_to_it_or_its_launcher_is_refused() {
    let root: PathBuf = fixture("fallback-wrapper-forms");
    let resolution: ResolutionInputs = only_the_wrapper(root.as_path());
    std::fs::create_dir(root.join("linked")).expect("directory");
    std::os::unix::fs::symlink(root.join("wrapper/git"), root.join("linked/git")).expect("link");
    std::fs::create_dir(root.join("launchers")).expect("directory");
    executable(root.join("launchers/monochromatic-git-policy-cli").as_path(), b"\x7fELF launcher");
    std::fs::write(
        root.join("launchers/monochromatic-git-policy-cli.launch"),
        b"{\"toolId\":\"cargo:monochromatic-git-policy-cli\",\"lockFile\":\"/l\",\"real\":\"/r\"}",
    )
    .expect("launch record");
    for named in [root.join("wrapper/git"), root.join("linked/git"), root.join("launchers/monochromatic-git-policy-cli")] {
        write_unit(root.as_path(), named.as_path());
        let failure: RealGitNotFound = resolve_real_git(&resolution).expect_err("the record is refused");
        assert_eq!(
            failure.recorded,
            Some(RecordedGit {
                unit_file: root.join("unit"),
                candidate: SearchedCandidate { path: named.clone(), kind: CandidateKind::Wrapper },
            }),
            "{}",
            named.display()
        );
        let said: String = failure.to_string();
        assert!(said.contains(format!("{}: recorded by the commit service unit {}", named.display(), root.join("unit").display()).as_str()), "{said}");
    }
    remove(root.as_path());
}

/// A record that names a missing file,
///  a file that is not executable,
///  or a directory is refused as unusable,
///  and a relative path in the unit is no record at all.
#[test]
fn a_record_naming_a_missing_or_non_executable_file_is_refused_and_a_relative_one_is_ignored() {
    let root: PathBuf = fixture("fallback-unusable");
    let resolution: ResolutionInputs = only_the_wrapper(root.as_path());
    std::fs::create_dir(root.join("plain")).expect("directory");
    std::fs::write(root.join("plain/git"), b"\x7fELF not executable").expect("plain file");
    for named in [root.join("gone/git"), root.join("plain/git"), root.join("plain")] {
        write_unit(root.as_path(), named.as_path());
        let failure: RealGitNotFound = resolve_real_git(&resolution).expect_err("the record is refused");
        let recorded: RecordedGit = failure.recorded.expect("the record is named");
        assert_eq!(recorded.candidate.path, named);
        assert_eq!(recorded.candidate.kind, CandidateKind::Unusable, "{}", named.display());
    }
    std::fs::write(root.join("unit"), b"ExecStart=/bin/service --git git --instance x\n").expect("relative record");
    assert_eq!(resolve_real_git(&resolution).expect_err("relative").recorded, None);
    remove(root.as_path());
}

/// The inputs of a process look for the unit below an absolute runtime directory only.
#[test]
fn process_inputs_name_the_unit_below_an_absolute_runtime_directory_only() {
    let named = |value: Option<&str>| -> Option<PathBuf> {
        let mut environment: Vec<(OsString, OsString)> = Vec::<(OsString, OsString)>::new();
        if let Some(text) = value {
            environment.push((OsString::from("XDG_RUNTIME_DIR"), OsString::from(text)));
        }
        return process_resolution_inputs(environment.as_slice()).expect("process inputs").service_unit;
    };
    assert_eq!(named(Some("/run/user/7")), Some(PathBuf::from("/run/user/7/systemd/user/monochromatic-git-policy-cli@.service")));
    assert_eq!(named(None), None);
    assert_eq!(named(Some("")), None);
    assert_eq!(named(Some("run/user/7")), None);
}
