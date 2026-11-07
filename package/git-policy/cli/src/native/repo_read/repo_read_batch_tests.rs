//! Differential controls for the in-process `cat-file --batch`:
//!  a request carrying object names on standard input gets Git's bytes for blobs,
//! trees and commits,
//! loose and packed,
//!  in SHA-1 and SHA-256,
//! and a name the store lacks is left to Git.

use crate::repo_read_hook::switch_off;
use crate::test_support::{REAL_GIT, executable, fixture, git, git_output, remove};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// The variables that make the host's own configuration irrelevant.
fn hermetic(root: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
        (OsString::from("HOME"), root.join("no-home").into_os_string()),
    ];
}

/// A stand-in Git that records each start and runs the real one.
fn counting_git(root: &Path) -> (PathBuf, PathBuf) {
    let directory = root.join("counting-bin");
    std::fs::create_dir(&directory).expect("directory");
    let program = directory.join("git");
    let count = root.join("count");
    executable(program.as_path(), format!("#!/bin/sh\nprintf 'start\\n' >> '{}'\nexec {REAL_GIT} \"$@\"\n", count.display()).as_bytes());
    return (program, count);
}

/// The number of recorded starts.
fn starts(count: &Path) -> usize {
    return std::fs::read_to_string(count).map_or(0, |text| { return text.lines().count(); });
}

/// What real Git prints for `cat-file --batch` fed `input` in `repo`.
fn real_batch(repo: &Path, input: &[u8]) -> Vec<u8> {
    let mut child = std::process::Command::new(REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cat-file");
    child.stdin.take().expect("stdin").write_all(input).expect("write");
    return child.wait_with_output().expect("output").stdout;
}

/// Objects of both formats,
/// loose and packed,
/// are printed as Git prints them;
/// a missing name starts Git.
#[test]
fn batch_requests_equal_git() {
    let root = fixture("repo-read-batch");
    for format in ["sha1", "sha256"] {
        let repo = root.join(format);
        std::fs::create_dir(&repo).expect("dir");
        git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--object-format={format}")]);
        std::fs::write(repo.join("a.txt"), "alpha\n").expect("a");
        std::fs::write(repo.join("empty"), "").expect("empty");
        std::fs::write(repo.join("binary"), [0_u8, 255, 10, 13, 0]).expect("binary");
        git(repo.as_path(), &["add", "--all"]);
        git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
        let names: Vec<String> = String::from_utf8(git_output(repo.as_path(), &["rev-list", "--objects", "--all"]).stdout).expect("UTF-8").lines().map(|line| { return line.split(' ').next().expect("name").to_string(); }).collect();
        let input: Vec<u8> = format!("{}\n", names.join("\n")).into_bytes();
        let holder = root.join(format!("holder-{format}"));
        std::fs::create_dir(&holder).expect("holder");
        let (program, count) = counting_git(holder.as_path());
        let context = GitContext { real_git: program, overlay: hermetic(root.as_path()), global_prefix: Vec::new(), common_dir: Some(repo.join(".git")) };
        for packed in [false, true] {
            if packed {
                git(repo.as_path(), &["repack", "-a", "-d", "--quiet"]);
                git(repo.as_path(), &["prune-packed"]);
            }
            let mut request = GitRequest::new(repo.as_path(), &["cat-file", "--batch"]);
            request.input = Some(input.clone());
            let before = starts(count.as_path());
            let found = run_git(&context, &request).expect("answer");
            assert_eq!(found.stdout, real_batch(repo.as_path(), &input), "{format} packed: {packed}");
            assert_eq!(starts(count.as_path()), before, "{format} packed: {packed}: no Git start");
        }
        let mut missing_input = input.clone();
        missing_input.extend_from_slice("1".repeat(names[0].len()).as_bytes());
        missing_input.push(b'\n');
        let mut request = GitRequest::new(repo.as_path(), &["cat-file", "--batch"]);
        request.input = Some(missing_input);
        let before = starts(count.as_path());
        let found = run_git(&context, &request).expect("Git answers");
        assert!(found.stdout.ends_with(b" missing\n"));
        assert_eq!(starts(count.as_path()), before + 1, "a missing name starts Git");
        let mut off_context = GitContext { real_git: context.real_git.clone(), overlay: context.overlay.clone(), global_prefix: Vec::new(), common_dir: Some(repo.join(".git")) };
        off_context.overlay.extend(switch_off());
        let mut plain = GitRequest::new(repo.as_path(), &["cat-file", "--batch"]);
        plain.input = Some(input.clone());
        let before_switch = starts(count.as_path());
        run_git(&off_context, &plain).expect("Git answers");
        assert_eq!(starts(count.as_path()), before_switch + 1, "the switch starts Git");
    }
    remove(root.as_path());
}
