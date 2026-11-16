//! What:
//!  What a form that recurses into submodules would replace inside them:
//!  the uncommitted tracked changes of a submodule's own worktree,
//!  each named as this worktree spells it.
//! Why:
//!  A submodule's worktree holds a person's files as much as the outer one,
//!  and a recursive checkout, switch or reset moves each submodule to the commit the target tree
//!  records, writing that submodule's own worktree.
//!  The outer status never names those changes when the repository sets
//!  `submodule.<name>.ignore`, so the dirty question cannot see them
//!  (`submodule.sub.ignore=all` hid an edit and `git checkout -f --recurse-submodules other`
//!  replaced it, the ninth review's fifth finding).
//!  The question is asked of the submodule itself, in its own repository,
//!  where that setting is not in force.
//!  Measured on this host with native Git 2.56.0 (2026-11-16):
//!  the recursive forms replace a submodule's tracked edits,
//!  and they leave its untracked and ignored files where they are,
//!  so this reading asks about tracked changes alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // uncommittedInsideSubmodules(written) === ['sub/f.txt']
//! ```

/// The paths one form writes, as the places they are.
use super::repository_facts_written_paths::{WrittenPlace, paths_the_tree_writes, worktree_root_of};
/// The provider whose queries this reading makes.
use super::repository_facts::GitFacts;
/// What one query answered: its exit status and both streams.
use crate::git_metadata::MetadataOutput;
/// Git's own global options, without the directory changes this process already made.
use crate::global_arguments::without_directory_changes;
/// How many submodules one form is asked about before the reading refuses instead of starting more
///  processes.
/// Why:
///  A repository can register hundreds of submodules,
///  and every one of them is a Git start on the command's own path;
///  a recursive form in this repository meets a handful, so the bound is a guard against a pathological
///  registration rather than a limit a person meets,
///  and a form past it is refused with the opt-out rather than read.
const SUBMODULES_ASKED_AT_MOST: usize = 32;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  Whether this worktree registers any submodule at all,
///  which is a metadata read and no Git start.
/// Why:
///  A repository without `.gitmodules` has nothing a recursive form could reach inside,
///  so the two questions below cost a process start only where they can mean something.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // holdsSubmodules() === existsSync(join(root, '.gitmodules'))
/// ```
pub(super) fn holds_submodules(facts: &mut GitFacts) -> Result<bool, String> {
    let root: PathBuf = worktree_root_of(facts)?;
    let marker: PathBuf = root.join(".gitmodules");
    let Some(answer) = crate::metadata_of_a_path_within_a_ceiling::symlink_metadata_within(
        crate::wait_ceilings::WORKTREE_LOOKUP_CEILING,
        marker.as_path(),
    ) else {
        return Err(format!(
            "the guard could not read `{}` within {} seconds, so it cannot tell whether this worktree registers submodules",
            marker.display(),
            crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
        ));
    };
    return match answer {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("Git could not read `{}`: {error}", marker.display())),
    };
}

/// What:
///  Whether this repository's configuration turns submodule recursion on for the forms that honour it.
/// Why:
///  `submodule.recurse=true` makes `checkout`, `switch`, `reset` and `restore` recurse without the option,
///  so a reading that only looked for `--recurse-submodules` would miss the form Git recurses with
///  (`git -c submodule.recurse=true checkout -f other` replaced a submodule edit in the same measurement).
///  Asked only when the form writes paths, the worktree registers submodules and the form carries no
///  recursion option of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const configured = (await git(['config', '--get', 'submodule.recurse'])).stdout.trim() === 'true';
/// ```
pub(super) fn submodule_recurse_configured(facts: &mut GitFacts) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "config", "--type=bool", "--get", "submodule.recurse"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "whether this repository recurses into submodules")?;
    // A setting that is not there answers nothing through the exit status, which is Git's own "no".
    if !output.success {
        return Ok(false);
    }
    let answer: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    return Ok(answer == "true" || answer == "1" || answer == "yes" || answer == "on");
}

/// What:
///  The uncommitted tracked changes inside the submodules a form would write,
///  each named as this worktree spells it (`<submodule>/<path>`),
///  or why the reading could not be made.
///  `&WrittenPaths` borrows what the form writes.
/// Why:
///  Only a submodule the form reaches can be replaced:
///  a whole-tree write reaches every path of the tree, a gitlink included,
///  and a path form reaches one when it names the submodule or a path below it.
///  A gitlink directory holding no repository is one Git has not initialized,
///  so nothing of a person's stands there and it is passed over;
///  a submodule whose status cannot be read is a failure, never "nothing to lose".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // uncommittedInsideSubmodules({ kind: 'tree', revision: 'other' }) === ['sub/f.txt']
/// ```
pub(super) fn uncommitted_changes_inside_submodules(
    facts: &mut GitFacts,
    written: &crate::command_written_paths::WrittenPaths,
) -> Result<Vec<String>, String> {
    let root: PathBuf = worktree_root_of(facts)?;
    let run_in: PathBuf = crate::command_run_directory::command_run_directory(facts.global_prefix.as_slice())?;
    let places: Vec<WrittenPlace> = paths_the_tree_writes(facts, written, run_in.as_path())?;
    let mut asked: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for place in places {
        if let WrittenPlace::Inside(path) = place {
            asked.push(path);
        }
    }
    if asked.is_empty() {
        return Ok(Vec::<String>::new());
    }
    let mut listed: Vec<OsString> = without_directory_changes(facts.global_prefix.as_slice());
    for token in ["--no-optional-locks", "ls-files", "--stage", "-z", "--full-name"] {
        listed.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(listed, "the submodules this worktree holds")?;
    if !output.success {
        let detail: String = String::from_utf8_lossy(output.stderr.as_slice()).lines().next().unwrap_or_default().trim().to_string();
        return Err(format!("Git could not read the submodules this worktree holds: {detail}"));
    }
    let mut reached: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for entry in output.stdout.split(|byte| return *byte == 0) {
        // Each entry is `<mode> <object> <stage>\t<path>`, and a submodule is mode `160000`.
        let Some(tab) = entry.iter().position(|byte| return *byte == b'\t') else {
            continue;
        };
        if !entry.starts_with(b"160000 ") {
            continue;
        }
        let path: &[u8] = &entry[tab + 1..];
        let under_it = |place: &Vec<u8>| {
            return place.as_slice() == path
                || (place.len() > path.len() && place.starts_with(path) && place.get(path.len()) == Some(&b'/'));
        };
        if asked.iter().any(under_it) && !reached.iter().any(|found| return found.as_slice() == path) {
            reached.push(path.to_vec());
        }
    }
    if reached.len() > SUBMODULES_ASKED_AT_MOST {
        return Err(format!(
            "the guard cannot tell what this command would replace: it recurses into {} submodules, and this reading asks at most {SUBMODULES_ASKED_AT_MOST}",
            reached.len()
        ));
    }
    let mut changes: Vec<String> = Vec::<String>::new();
    for path in reached {
        let directory: PathBuf = root.join(crate::repository_facts_written_paths::path_from_bytes(path.as_slice()));
        let marker: PathBuf = directory.join(".git");
        let Some(answer) = crate::metadata_of_a_path_within_a_ceiling::symlink_metadata_within(
            crate::wait_ceilings::WORKTREE_LOOKUP_CEILING,
            marker.as_path(),
        ) else {
            return Err(format!(
                "the guard could not read `{}` within {} seconds, so it cannot tell what the submodule holds",
                marker.display(),
                crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
            ));
        };
        match answer {
            // A gitlink whose directory holds no repository is one Git has not initialized: no person's file stands there.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("Git could not read `{}`: {error}", marker.display())),
            Ok(_) => {}
        }
        let mut arguments: Vec<OsString> = without_directory_changes(facts.global_prefix.as_slice());
        for token in ["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=no", "--ignore-submodules=none", "--"] {
            arguments.push(OsString::from(token));
        }
        let status: MetadataOutput = facts.query_in(directory.as_path(), arguments, "the changes inside that submodule")?;
        if !status.success {
            let detail: String = String::from_utf8_lossy(status.stderr.as_slice()).lines().next().unwrap_or_default().trim().to_string();
            return Err(format!(
                "Git could not read the changes inside the submodule `{}`, so the guard cannot tell what this command would replace there: {detail}",
                String::from_utf8_lossy(path.as_slice())
            ));
        }
        for entry in status.stdout.split(|byte| return *byte == 0) {
            // Each entry is `<two status letters><space><path>`, so the name starts at the fourth byte:
            //  the first status letter is a space for an unstaged change (` M f.txt`).
            if entry.len() < 4 || entry.get(2) != Some(&b' ') {
                continue;
            }
            let change: &[u8] = &entry[3..];
            let mut named: Vec<u8> = path.clone();
            named.push(b'/');
            named.extend_from_slice(change);
            changes.push(String::from_utf8_lossy(named.as_slice()).into_owned());
        }
    }
    changes.sort();
    changes.dedup();
    return Ok(changes);
}
