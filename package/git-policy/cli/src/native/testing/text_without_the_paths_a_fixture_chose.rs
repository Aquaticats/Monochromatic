//! What:
//!  The text a control judges,
//! with the paths a fixture chose taken out of it.
//! Why:
//!  A control judges the wrapper's words,
//! and a wrapper's sentence names the repository,
//! the lock and the worktree it acted on.
//!  Those paths begin with a folder the run names and the control does not:
//! `CARGO_TARGET_TMPDIR`,
//! `CLI_GIT_TEST_ROOT`,
//! the worktree's own directory name.
//!  A word inside that folder used to decide a control:
//! on 2026-11-14 a control that forbids `refus` in the words for a held commit failed only because its run's
//!  scratch folder was named `…-removal-refuses-a-worktree-a-process-works-in-…`,
//! and the same control passed with a neutral folder.
//!  Nothing is written in a path's place:
//! a marker would reach the judgement as words the wrapper never wrote.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = paths.reduce((text, path) => text.replaceAll(path, ''), said);
//! ```

/// The path type the caller names.
use std::path::Path;

/// What:
///  `text` with every occurrence of each path in `paths` removed.
/// Why:
///   The caller names the paths its text can hold and only those,
/// so a word the wrapper wrote beside a path is kept for the judgement,
/// and a word that nothing but a folder's name supplied cannot decide it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutThePaths('at /tmp/x/repo: held', ['/tmp/x']) // 'at /repo: held'
/// ```
pub fn without_the_paths<ChosenPath: AsRef<Path>>(text: &str, paths: &[ChosenPath]) -> String {
    let mut words: String = String::from(text);
    for path in paths {
        words = words.replace(path.as_ref().display().to_string().as_str(), "");
    }
    return words;
}

/// Controls for the cut above.
#[path = "text_without_the_paths_a_fixture_chose_tests.rs"]
mod tests;
