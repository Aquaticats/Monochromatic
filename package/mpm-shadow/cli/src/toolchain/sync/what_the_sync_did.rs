//! What:
//! What one `install` run did:
//! the tools already at their version,
//! the ones it installed,
//! the ones that failed with why,
//! the declared bins still unresolved,
//! and the warnings it printed,
//! together with the warning for an install that would replace another version.
//! Why:
//! `install` prints the report at the end and the caller decides its exit status from it,
//! and the replacement warning is asked for a tool before it is installed,
//! so both travel through one type and one sentence.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type SyncReport = { unchanged: string[]; installed: string[]; reconciled: string[]; failed: [string, string][]; ... };
//! export function replacementWarning(id: string, observed: string | undefined, locked: string): string | undefined;
//! ```

/// Imports what a later install did with the leftovers of a killed one.
use super::super::archive_staging::Recovered;
/// Imports the version comparison the warning uses.
use super::super::parse_output::version_matches;
/// What `install` did:
/// tools already at their version,
/// tools installed,
/// tools that failed (with why),
/// declared bins still
/// unresolved afterwards,
/// and warnings (a shared install being replaced).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SyncReport {
    /// Tool ids already at the locked version.
    pub unchanged: Vec<String>,
    /// Tool ids installed by this run.
    pub installed: Vec<String>,
    /// One sentence per declared Rust component or target this run added to a toolchain it did not install
    /// (one another program put at the locked version,
    /// which `install_tool` never sees).
    pub reconciled: Vec<String>,
    /// One sentence per version this run finished the publish of,
    /// or whose copy a killed install had moved aside and this run put back,
    /// with the hidden directories it removed.
    pub recovered: Vec<String>,
    /// Tool id and failure text.
    pub failed: Vec<(String, String)>,
    /// Declared bin names without a resolving link.
    pub missing_bins: Vec<String>,
    /// Warnings printed before the install that caused them.
    pub warnings: Vec<String>,
    /// The tools `install` resolved and locked itself before installing,
    /// as tool id and locked version:
    /// the plan entries that were new or whose resolution-affecting fields changed.
    pub resolved: Vec<(String, String)>,
    /// What `install` wrote into the lock from the plan without a lookup:
    /// one sentence per offline-fixable difference,
    /// such as a changed `bins` list.
    pub rewritten: Vec<String>,
    /// The cleanup of old copies of this repository's tools:
    /// one sentence per link re-pointed and per old copy removed.
    pub old_copies: Vec<String>,
    /// One sentence per postinstall link an earlier run made and this run removed
    /// because the lock no longer declares it.
    pub retired_links: Vec<String>,
    /// The environment variable names the generated mise fragment holds after this run
    /// (empty when the lock declares none,
    /// or for a partial sync that leaves the fragment alone).
    pub exported_env: Vec<String>,
}

/// What:
/// The sentence the install report prints for a version a killed install left unfinished,
/// or `None` when the recovery found and did nothing.
/// Why:
/// An install that finished the publish of a version without downloading anything did something a person should see,
/// and one that only removed hidden directories of an installer that no longer runs says what went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverySentence(id: string, version: string, recovered: Recovered): string | undefined
/// ```
pub(crate) fn recovery_sentence(
    id: &str,
    version: &str,
    recovered: &Recovered,
) -> Option<String> {
    let removed: usize = recovered.removed.len();
    if recovered.published {
        let mut sentence: String =
            format!("finished the publish of {id} {version} that a killed install left");
        if removed > 0 {
            sentence.push_str(&format!(
                ", and removed the {removed} hidden directories it left behind"
            ));
        }
        return Some(sentence);
    }
    if recovered.restored {
        let mut sentence: String =
            format!("put back the copy of {id} {version} that a killed install had moved aside");
        if removed > 0 {
            sentence.push_str(&format!(
                ", and removed the {removed} hidden directories it left behind"
            ));
        }
        return Some(sentence);
    }
    if removed > 0 {
        return Some(format!(
            "removed the {removed} hidden directories that a killed install of {id} {version} left behind"
        ));
    }
    return None;
}

/// What:
/// The warning for an install that would replace a different version,
/// or `None` when the tool is absent or
///           already matches.
/// Why:
/// Several worktrees share one install directory per tool version,
/// and a manager-level install (`cargo
///           binstall`,
/// `uv tool install`) replaces the one version everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replacementWarning(id: string, observed: string | undefined, locked: string): string | undefined
/// ```
pub fn replacement_warning(
    id: &str,
    observed: &Option<String>,
    locked: &str,
) -> Option<String> {
    let found: &str = observed.as_deref()?;
    if version_matches(found, locked) {
        return None;
    }
    return Some(format!(
        "{id} is installed at {found} but this tree's lock pins {locked}; installing {locked} replaces it for every tree \
         that shares this install, and a tree whose lock pins the other version replaces it back on its next sync"
    ));
}
