//! What:
//!  The measurement of the fence's size readers on one repository:
//! every reader's whole read timed over rotating rounds,
//! and every reader's map compared with the shipped read's.
//! Why:
//!  A report of options times them on the same repository and the same rounds,
//!  with the first reader rotated so no option keeps a cold slot,
//! and a map that differs is reported before any number is read.
//! It is `#[ignore]`d because it reads a repository named by a variable and takes minutes,
//! so no control run reaches it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const round of rounds) for (const reader of rotated(round)) out.push({ round, reader, ms: time(reader) });
//! ```

use crate::repo_read_layout::ObjectFormat;
use crate::test_support::isolated_overlay;
use crate::transaction_git::GitContext;
use crate::worktree_clone_size_reader::{SizeRead, SizeReader, read_sizes, wanted_paths};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The repository the measurement reads,
/// from the variable a run sets.
fn repository() -> PathBuf {
    let named: String = std::env::var("FENCE_SIZE_READER_REPO").expect("FENCE_SIZE_READER_REPO names the repository to measure");
    return PathBuf::from(named);
}

/// The rounds to time,
/// from the variable a run sets.
fn rounds() -> usize {
    return std::env::var("FENCE_SIZE_READER_ROUNDS").ok().and_then(|text| return text.parse().ok()).unwrap_or(7);
}

/// Where the lines of the measurement are kept.
fn out() -> Option<PathBuf> {
    return std::env::var("FENCE_SIZE_READER_OUT").ok().map(PathBuf::from);
}

/// The sizes of every path the index holds,
/// as `reader` answers them,
/// timed.
fn timed(reader: SizeReader, context: &GitContext, repo: &Path, wanted: &[(Vec<u8>, Vec<u8>)], format: ObjectFormat) -> (Duration, HashMap<Vec<u8>, u64>) {
    let common: PathBuf = repo.join(".git");
    let read: SizeRead<'_> = SizeRead {
        context,
        root: repo,
        common_dir: common.as_path(),
        format,
        head_commit: "HEAD",
    };
    let began: Instant = Instant::now();
    let sizes: HashMap<Vec<u8>, u64> = read_sizes(reader, &read, wanted).expect("the size read answered");
    return (began.elapsed(), sizes);
}

#[test]
#[ignore = "measurement: times the size readers on a repository a variable names"]
fn the_readers_answer_one_map_at_their_measured_costs() {
    let repo: PathBuf = repository();
    // One reader, for a run that times what the fence pays in a fresh process: the read a single add takes.
    let only: Option<SizeReader> = std::env::var("FENCE_SIZE_READER_ONLY").ok().and_then(|name| return SizeReader::named(name.as_str()));
    let context: GitContext = GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay: isolated_overlay(),
        global_prefix: Vec::new(),
        common_dir: Some(repo.join(".git")),
    };
    let index: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("the repository's index");
    let format: ObjectFormat = ObjectFormat::Sha1;
    let wanted: Vec<(Vec<u8>, Vec<u8>)> = wanted_paths(index.as_slice(), format, &|_path, _mode| return true).expect("the index parsed");
    let distinct: std::collections::HashSet<&[u8]> = wanted.iter().map(|(_path, id)| return id.as_slice()).collect();
    let mut lines: Vec<String> = vec![format!(
        "{{\"what\":\"shape\",\"repository\":{},\"paths\":{},\"distinctIds\":{}}}",
        format!("{:?}", repo.display().to_string()),
        wanted.len(),
        distinct.len()
    )];
    let mut reference: Option<Vec<(String, u64)>> = None;
    let readers: Vec<SizeReader> = match only {
        Some(reader) => vec![reader],
        None => SizeReader::ALL.to_vec(),
    };
    for round in 1..=rounds() {
        for at in 0..readers.len() {
            let reader: SizeReader = readers[(at + round - 1) % readers.len()];
            let (spent, sizes): (Duration, HashMap<Vec<u8>, u64>) = timed(reader, &context, repo.as_path(), wanted.as_slice(), format);
            let mut listed: Vec<(String, u64)> = sizes.iter().map(|(path, size)| return (String::from_utf8_lossy(path).to_string(), *size)).collect();
            listed.sort();
            match reference.as_ref() {
                Some(known) => assert_eq!(&listed, known, "{} answered another map in round {round}", reader.name()),
                None => reference = Some(listed),
            }
            lines.push(format!(
                "{{\"what\":\"round\",\"round\":{round},\"reader\":\"{}\",\"ms\":{:.3}}}",
                reader.name(),
                spent.as_secs_f64() * 1000.0
            ));
        }
    }
    for line in &lines {
        println!("{line}");
    }
    if let Some(path) = out() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the measurement's directory");
        }
        std::fs::write(path, lines.join("\n") + "\n").expect("the measurement's file");
    }
}
