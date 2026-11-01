//! What:
//!  Disposable-PATH controls for real-Git resolution.
//! Why:
//!  Candidate order,
//!  platform names,
//!  conventional-location priority and wrapper
//!      skipping decide which program every forwarded command runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(resolveRealGit({ pathEnv: `${wrapperDir}:${gitDir}`, ... })).toBe(`${gitDir}/git`);
//! ```
#![cfg(unix)]

/// Import the resolver under test and shared fixtures.
use super::{
    DEFAULT_WINDOWS_PATH_EXTENSIONS, Platform, RealGitNotFound, ResolutionInputs,
    candidate_sequence, common_git_paths, executable_names, host_platform,
    process_resolution_inputs, resolve_real_git,
};
use crate::test_support::{executable, fixture, remove};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Build inputs for one disposable PATH with no conventional locations.
fn inputs(path: &str, root: &Path) -> ResolutionInputs {
    return ResolutionInputs {
        platform: Platform::Unix,
        path: OsString::from(path),
        path_extensions: OsString::new(),
        current_directory: root.to_path_buf(),
        common_paths: Vec::<PathBuf>::new(),
        own_executable: root.join("wrapper/git"),
    };
}

/// Build an environment list from text pairs.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    let mut result: Vec<(OsString, OsString)> = Vec::<(OsString, OsString)>::new();
    for (name, value) in pairs {
        result.push((OsString::from(name), OsString::from(value)));
    }
    return result;
}

/// Unix tries `git` alone;
///  Windows tries each non-empty extension with a dot.
#[test]
fn executable_names_follow_platform_rules() {
    for platform in [Platform::Unix, Platform::MacOs] {
        assert_eq!(
            executable_names(platform, OsStr::new(".COM;.EXE")),
            vec![OsString::from("git")]
        );
    }
    assert_eq!(
        executable_names(
            Platform::Windows,
            OsStr::new(DEFAULT_WINDOWS_PATH_EXTENSIONS)
        ),
        vec![
            OsString::from("git.COM"),
            OsString::from("git.EXE"),
            OsString::from("git.BAT"),
            OsString::from("git.CMD"),
        ]
    );
    assert_eq!(
        executable_names(Platform::Windows, OsStr::new(";EXE;;.cmd;")),
        vec![OsString::from("git.EXE"), OsString::from("git.cmd")]
    );
    assert_eq!(
        executable_names(Platform::Windows, OsStr::new("")),
        Vec::<OsString>::new()
    );
    use std::os::unix::ffi::OsStrExt;
    assert_eq!(
        executable_names(Platform::Windows, OsStr::from_bytes(b".\xff;.EXE")),
        vec![OsString::from("git.EXE")]
    );
}

/// Conventional locations are fixed on Unix and macOS and derived from the environment on Windows.
#[test]
fn common_paths_follow_platform_rules() {
    assert_eq!(
        common_git_paths(Platform::Unix, &[]),
        vec![
            PathBuf::from("/usr/bin/git"),
            PathBuf::from("/usr/local/bin/git")
        ]
    );
    assert_eq!(
        common_git_paths(Platform::MacOs, &[]),
        vec![
            PathBuf::from("/usr/bin/git"),
            PathBuf::from("/usr/local/bin/git"),
            PathBuf::from("/opt/homebrew/bin/git"),
            PathBuf::from("/opt/local/bin/git"),
        ]
    );
    assert_eq!(
        common_git_paths(Platform::Windows, &[]),
        vec![
            PathBuf::from("C:\\Program Files\\Git\\cmd\\git.exe"),
            PathBuf::from("C:\\Program Files\\Git\\bin\\git.exe"),
        ]
    );
    assert_eq!(
        common_git_paths(
            Platform::Windows,
            environment(&[
                ("ProgramFiles", "D:\\Apps\\"),
                ("ProgramW6432", "C:\\Program Files"),
                ("ProgramFiles(x86)", "C:\\Program Files (x86)//"),
                ("LOCALAPPDATA", "C:\\Users\\u\\AppData\\Local\\"),
            ])
            .as_slice()
        ),
        vec![
            PathBuf::from("D:\\Apps\\Git\\cmd\\git.exe"),
            PathBuf::from("D:\\Apps\\Git\\bin\\git.exe"),
            PathBuf::from("C:\\Program Files\\Git\\cmd\\git.exe"),
            PathBuf::from("C:\\Program Files\\Git\\bin\\git.exe"),
            PathBuf::from("C:\\Program Files (x86)\\Git\\cmd\\git.exe"),
            PathBuf::from("C:\\Program Files (x86)\\Git\\bin\\git.exe"),
            PathBuf::from("C:\\Users\\u\\AppData\\Local\\Programs\\Git\\cmd\\git.exe"),
            PathBuf::from("C:\\Users\\u\\AppData\\Local\\Programs\\Git\\bin\\git.exe"),
        ]
    );
    // A root that is only separators trims to nothing rather than panicking.
    assert_eq!(
        common_git_paths(
            Platform::Windows,
            environment(&[("ProgramFiles", "\\")]).as_slice()
        )[0],
        PathBuf::from("\\Git\\cmd\\git.exe")
    );
}

/// PATH order is kept,
///  relative and empty entries resolve against the current directory,
///  repeats collapse.
#[test]
fn candidate_sequence_preserves_path_order_and_removes_repeats() {
    let mut resolution: ResolutionInputs =
        inputs("/a:relative::/a/:/b/./c//:/a", Path::new("/cwd"));
    assert_eq!(
        candidate_sequence(&resolution),
        vec![
            PathBuf::from("/a/git"),
            PathBuf::from("/cwd/relative/git"),
            PathBuf::from("/cwd/git"),
            PathBuf::from("/b/c/git"),
        ]
    );
    resolution.path = OsString::new();
    // An unset PATH is one empty entry: the current directory, as for a shell.
    assert_eq!(
        candidate_sequence(&resolution),
        vec![PathBuf::from("/cwd/git")]
    );
    resolution.path = OsString::from("/x/../y");
    // `..` is kept: resolving it without the filesystem would be wrong across links.
    assert_eq!(
        candidate_sequence(&resolution),
        vec![PathBuf::from("/x/../y/git")]
    );
}

/// A conventional location wins only when PATH exposes it,
///  in its PATH spelling and conventional order.
#[test]
fn common_paths_are_promoted_only_when_exposed() {
    let mut resolution: ResolutionInputs =
        inputs("/custom:/usr/local/bin:/usr/./bin", Path::new("/cwd"));
    resolution.common_paths = vec![
        PathBuf::from("/usr/bin/git"),
        PathBuf::from("/usr/local/bin/git"),
        PathBuf::from("/opt/absent/git"),
    ];
    assert_eq!(
        candidate_sequence(&resolution),
        vec![
            PathBuf::from("/usr/bin/git"),
            PathBuf::from("/usr/local/bin/git"),
            PathBuf::from("/custom/git"),
        ]
    );
    resolution.path = OsString::from("/custom");
    assert_eq!(
        candidate_sequence(&resolution),
        vec![PathBuf::from("/custom/git")]
    );
    // Unix identity is case-sensitive.
    resolution.path = OsString::from("/custom:/USR/BIN");
    assert_eq!(
        candidate_sequence(&resolution),
        vec![PathBuf::from("/custom/git"), PathBuf::from("/USR/BIN/git")]
    );
}

/// Windows expands each extension per directory and matches conventional locations case-insensitively.
#[test]
fn windows_candidates_use_extensions_and_case_insensitive_identity() {
    let mut resolution: ResolutionInputs = inputs("/custom:/Git/CMD", Path::new("/cwd"));
    resolution.platform = Platform::Windows;
    resolution.path_extensions = OsString::from(".EXE;.CMD");
    resolution.common_paths = vec![PathBuf::from("/git/cmd/GIT.exe")];
    assert_eq!(
        candidate_sequence(&resolution),
        vec![
            PathBuf::from("/Git/CMD/git.EXE"),
            PathBuf::from("/custom/git.EXE"),
            PathBuf::from("/custom/git.CMD"),
            PathBuf::from("/Git/CMD/git.CMD"),
        ]
    );
}

/// Wrappers are skipped through links,
///  duplicates,
///  copies and launcher scripts until real Git is reached.
#[test]
fn resolution_skips_every_wrapper_form() {
    let root: PathBuf = fixture("resolve-skip");
    for name in ["wrapper", "linked", "copied", "shim", "real", "later"] {
        std::fs::create_dir(root.join(name)).expect("directory");
    }
    executable(root.join("wrapper/git").as_path(), b"\x7fELF wrapper");
    std::os::unix::fs::symlink(root.join("wrapper/git"), root.join("linked/git")).expect("link");
    executable(root.join("copied/git").as_path(), b"\x7fELF wrapper");
    executable(
        root.join("shim/git").as_path(),
        b"#!/bin/sh\nexec node node_modules/@monochromatic-dev/git-policy-cli/x.mjs\n",
    );
    executable(root.join("real/git").as_path(), b"\x7fELF real");
    executable(root.join("later/git").as_path(), b"\x7fELF later");
    let resolution: ResolutionInputs = inputs(
        "wrapper:wrapper:linked:missing:copied:shim:real:later",
        root.as_path(),
    );
    assert_eq!(resolve_real_git(&resolution), Ok(root.join("real/git")));
    // The same wrapper reached first through a relative link directory is still excluded.
    let mut through_link: ResolutionInputs = inputs("linked:real", root.as_path());
    through_link.own_executable = root.join("linked/git");
    assert_eq!(resolve_real_git(&through_link), Ok(root.join("real/git")));
    remove(root.as_path());
}

/// When only wrappers and unusable entries exist,
///  the failure reports both counts.
#[test]
fn resolution_failure_reports_examined_and_skipped_counts() {
    let root: PathBuf = fixture("resolve-fail");
    for name in ["wrapper", "linked", "plain"] {
        std::fs::create_dir(root.join(name)).expect("directory");
    }
    executable(root.join("wrapper/git").as_path(), b"\x7fELF wrapper");
    std::os::unix::fs::symlink(root.join("wrapper/git"), root.join("linked/git")).expect("link");
    std::fs::write(root.join("plain/git"), b"\x7fELF not executable").expect("plain");
    let failure: RealGitNotFound =
        resolve_real_git(&inputs("wrapper:linked:plain:missing", root.as_path()))
            .expect_err("no real Git");
    assert_eq!(
        failure,
        RealGitNotFound {
            candidate_count: 4,
            skipped_wrapper_count: 2
        }
    );
    assert_eq!(
        failure.to_string(),
        "Could not find a real Git executable after examining 4 PATH candidates and skipping 2 \
         cli-git wrappers. Ensure Git is installed and PATH/PATHEXT expose its executable."
    );
    let empty: RealGitNotFound =
        resolve_real_git(&inputs("missing", root.as_path())).expect_err("nothing on PATH");
    assert_eq!(
        empty,
        RealGitNotFound {
            candidate_count: 1,
            skipped_wrapper_count: 0
        }
    );
    remove(root.as_path());
}

/// A conventional location exposed by PATH is selected ahead of an earlier PATH entry.
#[test]
fn resolution_prefers_exposed_common_locations() {
    let root: PathBuf = fixture("resolve-common");
    for name in ["early", "system"] {
        std::fs::create_dir(root.join(name)).expect("directory");
    }
    executable(root.join("early/git").as_path(), b"#!/bin/sh\nexit 3\n");
    executable(root.join("system/git").as_path(), b"\x7fELF system");
    let mut resolution: ResolutionInputs = inputs("early:system", root.as_path());
    assert_eq!(resolve_real_git(&resolution), Ok(root.join("early/git")));
    resolution.common_paths = vec![root.join("system/git")];
    assert_eq!(resolve_real_git(&resolution), Ok(root.join("system/git")));
    remove(root.as_path());
}

/// The gate runs on Linux,
///  whose rules are the Unix ones.
#[cfg(target_os = "linux")]
#[test]
fn host_platform_is_unix_on_linux() {
    assert_eq!(host_platform(), Platform::Unix);
}

/// Process inputs take PATH and PATHEXT from the given environment and identity from the process.
#[test]
fn process_inputs_combine_environment_and_process_identity() {
    let from_environment: ResolutionInputs = process_resolution_inputs(
        environment(&[("PATH", "/first:/second"), ("PATHEXT", ".X")]).as_slice(),
    )
    .expect("process facts");
    assert_eq!(from_environment.platform, host_platform());
    assert_eq!(from_environment.path, OsString::from("/first:/second"));
    assert_eq!(from_environment.path_extensions, OsString::from(".X"));
    assert_eq!(
        from_environment.own_executable,
        std::env::current_exe().expect("current executable")
    );
    assert_eq!(
        from_environment.current_directory,
        std::env::current_dir().expect("current directory")
    );
    assert_eq!(
        from_environment.common_paths,
        common_git_paths(host_platform(), &[])
    );
    let unset: ResolutionInputs = process_resolution_inputs(&[]).expect("process facts");
    assert_eq!(unset.path, OsString::new());
    assert_eq!(
        unset.path_extensions,
        OsString::from(DEFAULT_WINDOWS_PATH_EXTENSIONS)
    );
}
