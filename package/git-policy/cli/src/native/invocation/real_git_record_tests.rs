//! What:
//!  Controls for reading the real Git that the commit service's unit file records.
//! Why:
//!  The fallback of `real_git.rs` is only as sound as this reading:
//!  a path taken from the wrong word,
//! a relative path,
//! or a file past the bound would start the wrong program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(recordedGitPath('ExecStart=/bin/service --git /usr/bin/git --instance x')).toBe('/usr/bin/git');
//! ```
#![cfg(unix)]

/// The reading under test,
///  the product's own unit renderer so the controls follow the text the wrapper writes,
///  and the fixture helpers.
use super::{read_recorded_git, recorded_git_path, service_unit_file};
use crate::real_git_candidate::MAX_SCRIPT_INSPECTION_BYTES;
use crate::service_provision::{ProvisionHost, SERVICE_UNIT_NAME, unit_texts};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// The unit text the wrapper writes for `git`,
///  rendered by the product.
fn rendered_service_unit(git: &Path) -> String {
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: PathBuf::from("/run/user/1"),
        user_unit_dir: None,
        bus: PathBuf::from("/nonexistent/bus"),
        service_binary: PathBuf::from("/opt/tool/monochromatic-git-policy-cli"),
        git: git.to_path_buf(),
        instance: String::from("x"),
        retirement_wait: std::time::Duration::ZERO,
    };
    for (name, text) in unit_texts(&host) {
        if name == SERVICE_UNIT_NAME {
            return text;
        }
    }
    panic!("the service unit is one of the two units the wrapper writes");
}

/// The text the product writes holds the path after `--git`,
///  and the comment lines above it do not confuse the reading.
#[test]
fn the_path_after_git_on_the_exec_start_line_of_the_product_unit_is_read() {
    let text: String = rendered_service_unit(Path::new("/opt/git/bin/git"));
    assert_eq!(recorded_git_path(text.as_str()), Some(PathBuf::from("/opt/git/bin/git")));
}

/// A line that is not `ExecStart`,
///  a missing option,
///  an option with no value after it and a relative value each name nothing.
#[test]
fn text_without_an_absolute_git_option_on_an_exec_start_line_names_nothing() {
    assert_eq!(recorded_git_path(""), None);
    assert_eq!(recorded_git_path("Description=--git /usr/bin/git\n"), None);
    assert_eq!(recorded_git_path("ExecStart=/bin/service --journal /j --instance x\n"), None);
    assert_eq!(recorded_git_path("ExecStart=/bin/service --git"), None);
    assert_eq!(recorded_git_path("ExecStart=/bin/service --git git --instance x\n"), None);
    assert_eq!(recorded_git_path("ExecStart=/bin/service --git ./git --instance x\n"), None);
}

/// The first `ExecStart` line that names an absolute Git wins,
///  after an earlier line that names none.
#[test]
fn the_first_exec_start_line_that_names_a_git_is_the_record() {
    let text: &str = "ExecStart=/bin/a --instance x\n  ExecStart=/bin/b --git /one/git\nExecStart=/bin/c --git /two/git\n";
    assert_eq!(recorded_git_path(text), Some(PathBuf::from("/one/git")));
}

/// The unit file is looked for below the runtime directory in the place the wrapper writes it.
#[test]
fn the_unit_file_is_the_service_template_below_the_runtime_directory() {
    assert_eq!(
        service_unit_file(Path::new("/run/user/7")),
        PathBuf::from("/run/user/7/systemd/user/monochromatic-git-policy-cli@.service")
    );
}

/// A readable unit gives its path;
///  a missing file,
/// a directory,
/// a file that is not text and a file past the bound give nothing.
#[test]
fn a_unit_file_is_read_with_a_bound_and_an_unreadable_one_names_nothing() {
    let root: PathBuf = fixture("record-read");
    let unit: PathBuf = root.join("unit");
    std::fs::write(&unit, rendered_service_unit(Path::new("/opt/git/bin/git"))).expect("unit file");
    assert_eq!(read_recorded_git(unit.as_path()), Some(PathBuf::from("/opt/git/bin/git")));
    assert_eq!(read_recorded_git(root.join("missing").as_path()), None);
    assert_eq!(read_recorded_git(root.as_path()), None);
    let binary: PathBuf = root.join("binary");
    std::fs::write(&binary, b"ExecStart=/bin/s --git /opt/git\xff/git\n").expect("binary file");
    assert_eq!(read_recorded_git(binary.as_path()), None);
    // The record is in the first bytes, and the file is one byte over the bound: it is not the wrapper's unit.
    let mut long: String = rendered_service_unit(Path::new("/opt/git/bin/git"));
    long.push_str("#".repeat(MAX_SCRIPT_INSPECTION_BYTES).as_str());
    let over: PathBuf = root.join("over");
    std::fs::write(&over, long).expect("long file");
    assert_eq!(read_recorded_git(over.as_path()), None);
    remove(&root);
}
