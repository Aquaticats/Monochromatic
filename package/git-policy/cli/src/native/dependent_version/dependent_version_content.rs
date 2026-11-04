//! What: The repository content the dependent-version planner reads, behind one small
//!       interface, and the two ways planning can fail to finish.
//! Why: The planner is pure with respect to the repository: it never starts Git and never
//!      reads the filesystem. The policy's wiring answers from the wrapper's candidate
//!      layer, the release path from the worktree, and tests from memory. Failures are
//!      reported by cause: content that could not be read is `content-unavailable`;
//!      content that was read but that the planner cannot use is `policy-incomplete`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // interface WorkspaceContent { trackedPaths(): TrackedPath[]; candidateBytes(path): Uint8Array; baseBytes(path): Uint8Array | undefined }
//! ```

/// What: The Git file mode of a tracked path. An `enum` is a closed set of named
///       alternatives; `#[derive(...)]` generates copying, debug printing and `==`.
/// Why:  Only ordinary files can receive a full-content patch, so the policy drops a
///       bump whose manifest is a symbolic link or a submodule, as the incumbent does
///       (`dependent-version-bump-policy.ts:209`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackedMode = 'regular' | 'executable' | 'symlink' | 'submodule';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackedMode {
    /// Mode `100644`.
    Regular,
    /// Mode `100755`.
    Executable,
    /// Mode `120000`; its bytes are the link target.
    Symlink,
    /// Mode `160000`, a gitlink.
    Submodule,
}

/// What: One tracked path in the candidate state. `Vec<u8>` is an owned list of bytes;
///       Git paths are bytes, not necessarily UTF-8.
/// Why:  The planner selects manifests, the registry configuration and source files from
///       this list itself, so the provider needs no knowledge of what the planner reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackedPath = { path: Uint8Array; mode: TrackedMode };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackedPath {
    /// Repository-relative path, `/`-separated.
    pub path: Vec<u8>,
    /// Its mode in the candidate state.
    pub mode: TrackedMode,
}

/// What: Something the provider could not read. `Option<Vec<u8>>` is "a path, or
///       nothing" (nothing when the tracked-path listing itself failed); `String` is
///       owned UTF-8 text.
/// Why:  The engine reports this as `content-unavailable`: the policy is intact, its
///       input is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ContentUnavailable = { path?: Uint8Array; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentUnavailable {
    /// The path whose bytes could not be read, or nothing for the listing.
    pub path: Option<Vec<u8>>,
    /// The provider's explanation.
    pub reason: String,
}

/// What: The repository facts the planner reads. A `trait` is a named set of methods a
///       type promises to provide, like a TS `interface`; `&mut self` lends the provider
///       for writing, because it may remember what it read; `&[u8]` borrows a path.
/// Why:  The planner asks only for what it needs: every manifest at both states, the
///       registry configuration once a version was raised, and source files of the
///       dependents whose bundled imports decide an edge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WorkspaceContent { trackedPaths(): TrackedPath[]; candidateBytes(path: Uint8Array): Uint8Array; baseBytes(path: Uint8Array): Uint8Array | undefined }
/// ```
pub trait WorkspaceContent {
    /// Every tracked path in the candidate state, in the provider's order (Git's index
    /// order in the wrapper).
    fn tracked_paths(&mut self) -> Result<Vec<TrackedPath>, ContentUnavailable>;
    /// The bytes of a tracked path in the candidate state.
    fn candidate_bytes(&mut self, path: &[u8]) -> Result<Vec<u8>, ContentUnavailable>;
    /// The bytes of a path at the comparison base (`HEAD` for the policy), or nothing when
    /// the base lacks it.
    fn base_bytes(&mut self, path: &[u8]) -> Result<Option<Vec<u8>>, ContentUnavailable>;
}

/// What: Why the planner's own machinery could not finish although its input was read.
/// Why:  The engine reports each of these as `policy-incomplete`. `ManifestShape` carries
///       the incumbent's `ManifestShapeError` text exactly, so both implementations can
///       be compared message for message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyIncomplete = { kind: 'not-utf8'; path } | { kind: 'manifest-syntax'; path; detail } | { kind: 'manifest-shape'; path; problem } | { kind: 'duplicate-name'; name; first; second };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyIncomplete {
    /// A manifest or the registry configuration is not valid UTF-8.
    NotUtf8 {
        /// The file that failed to decode.
        path: Vec<u8>,
    },
    /// A manifest is not JSON.
    ManifestSyntax {
        /// The manifest.
        path: Vec<u8>,
        /// The parser's explanation.
        detail: String,
    },
    /// A manifest is JSON but lacks a fact the plan needs, or its version cannot be
    /// rewritten in place.
    ManifestShape {
        /// The manifest.
        path: Vec<u8>,
        /// The incumbent's message, which names the manifest.
        problem: String,
    },
    /// Two workspace manifests declare the same package name, so the dependency graph is
    /// ambiguous.
    DuplicateName {
        /// The shared name.
        name: String,
        /// The earlier manifest in tracked order.
        first: Vec<u8>,
        /// The later manifest.
        second: Vec<u8>,
    },
}

/// What: Any failure to finish planning. `From` conversions below let `?` lift either
///       cause into this type.
/// Why:  The wiring maps the two variants to the two engine failure codes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanError = { kind: 'content-unavailable'; error: ContentUnavailable } | { kind: 'policy-incomplete'; error: PolicyIncomplete };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanError {
    /// Reported as `content-unavailable`.
    ContentUnavailable(
        /// What could not be read.
        ContentUnavailable,
    ),
    /// Reported as `policy-incomplete`.
    PolicyIncomplete(
        /// What the planner could not use.
        PolicyIncomplete,
    ),
}

/// What: Lift an unreadable-content failure into a planning failure. `impl Trait for Type`
///       gives a type the methods a trait requires.
/// Why:  Lets the planner write `content.candidate_bytes(path)?`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure: PlanError = { kind: 'content-unavailable', error };
/// ```
impl From<ContentUnavailable> for PlanError {
    /// Wrap the unreadable-content failure.
    fn from(error: ContentUnavailable) -> PlanError {
        return PlanError::ContentUnavailable(error);
    }
}

/// What: Lift a machinery failure into a planning failure.
/// Why:  Lets the planner write `read_manifest_facts(...)?`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure: PlanError = { kind: 'policy-incomplete', error };
/// ```
impl From<PolicyIncomplete> for PlanError {
    /// Wrap the machinery failure.
    fn from(error: PolicyIncomplete) -> PlanError {
        return PlanError::PolicyIncomplete(error);
    }
}

/// What: Show a repository path as text. `String::from_utf8_lossy` replaces bytes that are
///       not UTF-8 with U+FFFD; `.into_owned()` makes the result an owned `String`.
/// Why:  Messages and finding paths are text; Git paths are bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = new TextDecoder().decode(path);
/// ```
pub fn display_path(path: &[u8]) -> String {
    return String::from_utf8_lossy(path).into_owned();
}

/// The escape hatch named in every remedy.
const HATCH: &str = "--no-enforce-mono/dependent-version-bump";

/// What: The complete diagnostic for a planning failure, naming the input and every way
///       forward.
/// Why:  The person who ran the command must be able to act without reading the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planErrorMessage(error: PlanError): string;
/// ```
pub fn plan_error_message(error: &PlanError) -> String {
    match error {
        PlanError::ContentUnavailable(unavailable) => {
            // `.as_deref()` borrows the optional path; `map_or` picks a default for none.
            let subject: String = unavailable.path.as_deref().map_or_else(
                || return String::from("the list of tracked files"),
                |path: &[u8]| return display_path(path),
            );
            return format!(
                "mono/dependent-version-bump could not read {subject}: {}. \
                 Nothing was decided; retry once the content is readable, \
                 or pass {HATCH} to skip this policy for one command.",
                unavailable.reason
            );
        }
        PlanError::PolicyIncomplete(incomplete) => {
            return format!(
                "mono/dependent-version-bump could not finish: {}. \
                 Correct the file and retry, or pass {HATCH} to skip this policy for one command.",
                incomplete_reason(incomplete)
            );
        }
    }
}

/// What: The cause part of a `policy-incomplete` diagnostic.
/// Why:  Each cause names the file and what is wrong with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function incompleteReason(error: PolicyIncomplete): string;
/// ```
fn incomplete_reason(incomplete: &PolicyIncomplete) -> String {
    match incomplete {
        PolicyIncomplete::NotUtf8 { path } => {
            return format!("{} is not valid UTF-8", display_path(path));
        }
        PolicyIncomplete::ManifestSyntax { path, detail } => {
            return format!("{} is not JSON ({detail})", display_path(path));
        }
        PolicyIncomplete::ManifestShape { problem, .. } => return problem.clone(),
        PolicyIncomplete::DuplicateName {
            name,
            first,
            second,
        } => {
            return format!(
                "{} and {} both declare the package name {name:?}, so which one a dependency \
                 names is ambiguous",
                display_path(first),
                display_path(second)
            );
        }
    }
}

/// Error conversions and diagnostics.
#[cfg(test)]
#[path = "dependent_version_content_tests.rs"]
mod tests;
