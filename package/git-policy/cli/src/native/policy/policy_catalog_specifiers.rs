//! What:
//!  The optional `mono/catalog-specifiers` policy:
//!  every dependency in a tracked `package.json` is `catalog:`,
//! `catalog:<name>` or exactly `workspace:*`,
//!  and no `package.json5`,
//! `package.yaml` or `package.yml` is tracked.
//! Why:
//!  `pnpm install` accepts a literal version without a word,
//! so the repository checks the rule itself
//!      (`doc/handover/catalog-specifier-policy.md`).
//!  The rule,
//! its findings and their wording belong to the `monochromatic-catalog-specifier` crate,
//!      which the pnpm wrapper shares;
//!  this module only hands it each candidate manifest's path and its candidate bytes,
//! never the worktree's,
//!      and turns its findings into the engine's.
//!  Only the last path segment decides what a file is,
//! and it is decided on the path's raw bytes,
//!      so a manifest below a directory whose name is not UTF-8 is still checked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const c of candidates) findings.push(...(basename(c) === 'package.json' ? checkManifest(c.path, bytes(c)) : checkManifestPath(c.path) ?? []));
//! ```

/// Import the candidate's mode.
use crate::candidate_object::CandidateMode;
/// Import the candidate change kinds.
use crate::candidate_record::CandidateChange;
/// Import the candidate and version types.
use crate::candidate_version::{Candidate, CandidateVersion};
/// Import the event form of a pathname.
use crate::event_path::EventPath;
/// Import the lossy text of a pathname for the library's messages.
use crate::path_bytes::path_message_text;
/// Import the lifecycle's candidates.
use crate::policy_content::{ContentState, LifecycleContent};
/// Import the finding and outcome types of a check.
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
/// Import the facts interface that prepares candidates.
use crate::repository_facts::RepositoryFacts;
/// The shared rule:
///  its entry for a manifest's bytes,
///  its entry for a file name alone,
///  and its finding.
use monochromatic_catalog_specifier::{Finding, check_manifest, check_manifest_path};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The one file name whose bytes the rule reads.
pub const MANIFEST_FILE_NAME: &[u8] = b"package.json";

/// What:
///  Whether one byte is `/`,
///  the separator of Git pathnames.
/// Why:
///   A named predicate for `rsplit`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSlash = (byte: number) => byte === 0x2f;
/// ```
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  The last segment of a Git pathname.
///  `.rsplit(..).next()` yields the piece after the final separator,
///  or the whole path when it has none.
/// Why:
///   The file name decides the rule's reading,
///  and a directory named like a manifest (`package.yaml/readme`) must not count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const basename = (path: Uint8Array) => path.slice(path.lastIndexOf(0x2f) + 1);
/// ```
fn file_name(path: &[u8]) -> &[u8] {
    return path.rsplit(is_slash).next().unwrap_or(path);
}

/// What:
///  Whether the rule looks at a candidate.
/// Why:
///   Deletions have no content and cannot add a literal version.
///  A submodule entry has no blob here and is no manifest.
///  A symbolic link is looked at:
/// its blob is the link target,
///  which the rule reports as an unreadable manifest,
/// so a link cannot hide a manifest from the policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const looked = candidate.change !== 'deleted' && candidate.mode !== 'gitlink';
/// ```
fn is_looked_at(candidate: &Candidate) -> bool {
    if candidate.change == CandidateChange::Deleted {
        return false;
    }
    return candidate.mode != CandidateMode::Gitlink;
}

/// What:
///  The engine's finding for one of the rule's findings about a candidate.
///  `finding.kind.as_str()` is the rule's kebab-case kind,
///  which becomes the policy-local code.
/// Why:
///   The message is the rule's own sentence naming the file,
///  the section and the dependency with the remedy;
///  the pathname field keeps the candidate's exact bytes,
///  which the message's text cannot when the name is not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const policyFinding = (f, candidate) => ({ code: f.kind, message: String(f), path: candidate.path });
/// ```
fn policy_finding(finding: &Finding, candidate: &Candidate) -> PolicyFinding {
    return PolicyFinding {
        code: finding.kind.as_str(),
        message: finding.to_string(),
        path: Some(EventPath::from_git_bytes(candidate.path.as_slice())),
        location: None,
        fix_available: false,
    };
}

/// What:
///  Check every candidate of the lifecycle.
///  `<F: RepositoryFacts>` accepts any facts provider;
///  `&mut ContentState` lends the shared candidates for reading.
/// Why:
///   A `package.json` is read through the candidate bytes and checked as a whole.
///  Any other file is given to the rule by name only,
///  with no read,
/// so a `package.yaml` is refused without its bytes being fetched
///  and an ordinary file costs nothing.
///  No candidates means no findings;
///  unreadable content means the policy failed,
/// never that it passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkCatalogSpecifiers(content, lifecycle, facts): Promise<PolicyOutcome>;
/// ```
pub fn check_catalog_specifiers<F: RepositoryFacts>(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut F,
) -> PolicyOutcome {
    let version: Rc<CandidateVersion> = match content.version(lifecycle, facts) {
        Ok(Some(found)) => found,
        // `Vec::new()` is the empty list: a lifecycle without candidates has nothing to report.
        Ok(None) => return PolicyOutcome::Findings(Vec::new()),
        Err(outcome) => return outcome,
    };
    let mut findings: Vec<PolicyFinding> = Vec::new();
    for candidate in version.candidates() {
        if !is_looked_at(candidate) {
            continue;
        }
        // The rule takes text; the lossy form keeps every ASCII byte, so the file name it tests is the real one.
        let shown: String = path_message_text(candidate.path.as_slice());
        let found: Vec<Finding> = if file_name(candidate.path.as_slice()) == MANIFEST_FILE_NAME {
            let bytes: Rc<[u8]> = match content.bytes(candidate) {
                Ok(read) => read,
                Err(outcome) => return outcome,
            };
            check_manifest(shown.as_str(), &bytes)
        } else {
            // `.into_iter().collect()` turns the rule's "a finding or nothing" into a list of at most one.
            check_manifest_path(shown.as_str()).into_iter().collect()
        };
        for finding in &found {
            findings.push(policy_finding(finding, candidate));
        }
    }
    return PolicyOutcome::Findings(findings);
}

/// Check controls over real candidates stay out of the release executable.
#[cfg(test)]
#[path = "policy_catalog_specifiers_tests.rs"]
mod tests;
