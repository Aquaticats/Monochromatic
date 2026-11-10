//! What:
//!  The names a wrapped command left behind in a repository's two scratch places,
//!  waited for within a bound.
//! Why:
//!  A commit's scratch tree is removed by the resident service when the client's connection closes,
//!  which is after the client printed its summary,
//!  so a control that reads the directory once can see a tree that is on its way out and call it a leak.
//!  The wait is bounded because a tree that stays is exactly the finding these controls report
//!  (`SPEC.md`,
//! "The commit's scratch directory").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await leftBehind(repo); // waits until the places are empty, or the bound passes
//! ```

/// The time a control gives the service to remove what it holds.
///
/// Measured on 2026-11-09 in the shaped fixtures:
/// the removal of a 9,081,951-byte tree is 34 `unlinkat` calls and
/// 1.5 ms of system time,
/// and the service starts it as soon as the client's connection closes.
/// Thirty seconds is far past that and still short enough to fail a control that would otherwise hang.
const SETTLE: std::time::Duration = std::time::Duration::from_secs(30);

/// How often a control looks again while it waits.
const POLL: std::time::Duration = std::time::Duration::from_millis(20);

/// The names of a commit's scratch trees,
/// which are the entries a wrapped command makes and removes.
///  A saved message is not one of them:
///  the wrapper writes that when it keeps the text a person typed,
///  nothing removes it,
///  and the command's own message names the file it is in.
const REMOVED_BY_THE_WRAPPER: [&str; 2] = [
    "monochromatic-git-policy-cli-service-",
    "cli-git-service-",
];

/// What:
///  Everything the wrapper would have removed and did not,
///  in the worktree's Git directory and in the repository scratch directory beside it.
/// Why:
///   No scratch tree of a command that has ended belongs in a repository,
///  and one that stays goes to the next sweep instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = leftBehind(repo); // bounded wait
/// ```
pub fn left_behind(repo: &std::path::Path) -> Vec<String> {
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        let left: Vec<String> = names_left_now(repo);
        if left.is_empty() || began.elapsed() >= SETTLE {
            return left;
        }
        std::thread::sleep(POLL);
    }
}

/// The names present at this moment,
/// in both places,
/// with the place named when it is the repository scratch directory.
fn names_left_now(repo: &std::path::Path) -> Vec<String> {
    let mut left: Vec<String> = Vec::new();
    let places: [std::path::PathBuf; 2] = [
        repo.join(".git"),
        repo.join(".git").join("monochromatic-git-policy-cli-scratch"),
    ];
    for place in places {
        let Ok(entries) = std::fs::read_dir(&place) else { continue };
        for entry in entries.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            if REMOVED_BY_THE_WRAPPER.iter().any(|prefix| { return name.starts_with(prefix); }) {
                left.push(format!("{}/{}", place.display(), name));
            }
        }
    }
    return left;
}
