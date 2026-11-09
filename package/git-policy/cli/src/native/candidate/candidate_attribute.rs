//! What:
//!  Which candidate paths have their `text` Git attribute unset (`-text`,
//! which the `binary` macro includes),
//!  resolved by `git check-attr` against the same state the candidates were listed from.
//! Why:
//!  A path whose `text` attribute is unset declares that its exact bytes matter,
//!  as a fuzz seed's trailing bytes do.
//!  The attribute lives in the package's own `.gitattributes`,
//!  so it moves with the package,
//!  unlike a path list kept elsewhere.
//!  Staged and projected candidates are answered from the private index,
//!  so a `.gitattributes` staged by the same command applies to it
//!  and an unstaged worktree edit does not;
//!  landed and pushed candidates are answered from the tree of the commit they came from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const unset = await attributes.textUnset([Buffer.from('seed/case')]);
//! ```

/// Import the layer's failure type and its causes.
use crate::candidate_error::{CandidateError, CandidateFailure};
/// Import the validated object name a tree source is named by.
use crate::candidate_object::ObjectId;
/// Import the captured-query runner and the byte helpers.
use crate::git_metadata::{MetadataOutput, path_from_git_bytes, read_stopped_text, run_metadata_git_without, strip_git_line};
/// What:
///  `HashMap<K, V>` is a key-to-value table and `HashSet<T>` a collection of distinct values.
/// Why:
///   Paths are grouped by the tree that answers for them and returned as a set to test membership.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byTree = new Map<string, Buffer[]>(); const unset = new Set<string>();
/// ```
use std::collections::{HashMap, HashSet};
/// Names and paths of the Git child.
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// How many paths one `git check-attr` command names,
///  as many as one pathspec command of the candidate layer.
///  Exempt from a citation under `LQM` (class:
/// chunk size):
/// it splits one command into several starts of the same
///  command,
/// so the result is the same and only the number of starts changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub const ATTRIBUTE_CHUNK_SIZE: usize = crate::candidate_pathspec_chunk_size::PATHSPEC_CHUNK_SIZE;

/// What:
///  Where the attributes of the candidates are read from.
///  `Index` is the private index the candidates were prepared on;
///  `Tree` is one commit or tree for every candidate;
///  `PerPath` names the commit or tree each path was first listed from.
/// Why:
///   Staged candidates are judged by what is staged.
///  A landed commit is judged by its own tree,
///  and a push lists several commits and trees,
///  each of which is judged by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeSource = { kind: 'index' } | { kind: 'tree'; tree: ObjectId } | { kind: 'per-path'; trees: Map<string, ObjectId> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeSource {
    /// The `.gitattributes` files of the private index.
    Index,
    /// The `.gitattributes` files of one commit or tree.
    Tree(ObjectId),
    /// The commit or tree each pathname was first listed from.
    PerPath(HashMap<Vec<u8>, ObjectId>),
}

/// What:
///  The resolver of the `text` attribute for one prepared set of candidates.
///  It owns copies of the Git executable,
///  the global options and the child environment,
///  which already name the private index.
/// Why:
///   The policy asks lazily,
///  after the prepared candidates were handed over,
///  so the resolver cannot borrow the provider that prepared them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateAttributes { constructor(git, prefix, overlay, source) {} textUnset(paths: Buffer[]): Promise<Set<string>> {} }
/// ```
#[derive(Clone, Debug)]
pub struct CandidateAttributes {
    /// The selected real Git executable.
    real_git: PathBuf,
    /// The arguments before the subcommand.
    global_prefix: Vec<OsString>,
    /// The child environment additions,
    ///  including `GIT_INDEX_FILE` of the private index.
    overlay: Vec<(OsString, OsString)>,
    /// Where the attributes are read from.
    source: AttributeSource,
}

/// What:
///  The failure of an attribute lookup,
///  worded like the layer's other Git failures and naming no candidate.
/// Why:
///   The message reaches the person as the policy's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributeFailure = (failure, detail) => new CandidateError(failure, `monochromatic-git-policy-cli could not read the text attribute of candidate files: ${detail}`);
/// ```
fn attribute_failure(failure: CandidateFailure, detail: &str) -> CandidateError {
    return CandidateError::new(
        failure,
        format!("monochromatic-git-policy-cli could not read the text attribute of candidate files: {detail}").as_str(),
    );
}

/// What:
///  Whether one byte is the NUL byte that ends each field of `git check-attr -z` output.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Parse the `-z` output of `git check-attr text` for the paths asked about,
///  and return the asked paths whose value is `unset`.
///  `asked` holds the arguments given to Git (the top level and the path),
///  `paths` the candidate pathnames in the same order.
/// Why:
///   Each record is three NUL-terminated fields,
///  path,
/// attribute name and value,
///  in the order asked.
///  Only the exact value `unset` preserves a file:
///  `set`,
/// `auto`,
/// `unspecified` and any other value leave it checked.
///  A record that does not match what was asked is an error,
///  never a default to text or to binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseTextUnset(output: Buffer, asked: Buffer[], paths: Buffer[]): Buffer[]; // throws CandidateError
/// ```
pub fn parse_text_unset(
    output: &[u8],
    asked: &[Vec<u8>],
    paths: &[&[u8]],
) -> Result<Vec<Vec<u8>>, CandidateError> {
    // `.split(..)` yields the NUL-separated fields; the last one is empty after the final terminator.
    let mut fields: Vec<&[u8]> = output.split(is_nul_byte).collect();
    if fields.pop() != Some(b"".as_slice()) || fields.len() != asked.len() * 3 {
        return Err(attribute_failure(
            CandidateFailure::ListingMalformed,
            "git check-attr did not print one record for each path it was asked about.",
        ));
    }
    let mut unset: Vec<Vec<u8>> = Vec::new();
    // `.chunks(3)` yields each record's path, attribute name and value.
    for (position, record) in fields.chunks(3).enumerate() {
        if record[0] != asked[position].as_slice() || record[1] != b"text" {
            return Err(attribute_failure(
                CandidateFailure::ListingMalformed,
                "git check-attr printed a record for a different path or attribute than it was asked about.",
            ));
        }
        if record[2] == b"unset" {
            unset.push(paths[position].to_vec());
        }
    }
    return Ok(unset);
}

/// What:
///  `impl CandidateAttributes { ... }` attaches the constructor and the lookup.
/// Why:
///   One owner builds the Git commands,
///  so a policy cannot ask against another state than the candidates'.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateAttributes { textUnset(paths) {} }
/// ```
impl CandidateAttributes {
    /// What:
    ///  A resolver over `source`,
    ///  running the given Git with the given options and environment.
    /// Why:
    ///   The environment is the one that names the candidates' private index.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(realGit, globalPrefix, overlay, source) {}
    /// ```
    pub fn new(
        real_git: &std::path::Path,
        global_prefix: &[OsString],
        overlay: &[(OsString, OsString)],
        source: AttributeSource,
    ) -> CandidateAttributes {
        return CandidateAttributes {
            real_git: real_git.to_path_buf(),
            global_prefix: global_prefix.to_vec(),
            overlay: overlay.to_vec(),
            source,
        };
    }

    /// What:
    ///  Run one Git command and return its standard output.
    /// Why:
    ///   A Git that cannot start or that exits unsuccessfully fails the lookup.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stdout = await runGit([...prefix, ...command]);
    /// ```
    fn run(&self, command: &[OsString]) -> Result<Vec<u8>, CandidateError> {
        let mut arguments: Vec<OsString> = self.global_prefix.clone();
        arguments.extend_from_slice(command);
        let operation: String = match command.first() {
            Some(word) => word.to_string_lossy().into_owned(),
            None => String::from("(no command)"),
        };
        // The commands here name paths the wrapper reads, so no pathspec mode the caller set reaches them.
        let output: MetadataOutput = match run_metadata_git_without(
            self.real_git.as_path(),
            arguments.as_slice(),
            self.overlay.as_slice(),
            crate::conflict_side_modes::MODE_VARIABLES.as_slice(),
        ) {
            Ok(captured) => captured,
            Err(error) => {
                // A read the wrapper ended at its ceiling says what blocked it and what stayed as it was.
                if let Some(stopped) = read_stopped_text(operation.as_str(), &error) {
                    return Err(attribute_failure(
                        CandidateFailure::GitStopped,
                        stopped.as_str(),
                    ));
                }
                return Err(attribute_failure(
                    CandidateFailure::GitNotStarted,
                    format!("git {operation} could not be started: {error}.").as_str(),
                ));
            }
        };
        if !output.success {
            return Err(attribute_failure(
                CandidateFailure::GitFailed,
                format!(
                    "git {operation} failed: {}",
                    String::from_utf8_lossy(output.stderr.as_slice()).trim()
                )
                .as_str(),
            ));
        }
        return Ok(output.stdout);
    }

    /// What:
    ///  The absolute path of the worktree's top level,
    ///  as Git prints it.
    /// Why:
    ///   Candidate pathnames are relative to the top level,
    ///  and `git check-attr` resolves its arguments from the directory it runs in,
    ///  which may be a subdirectory or,
    ///  for a command run with `GIT_DIR` and `GIT_WORK_TREE`,
    ///  a directory outside the worktree.
    ///  An absolute path inside the worktree means the same file from anywhere.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const top = (await runGit(['rev-parse', '--show-toplevel'])).trimEnd();
    /// ```
    fn top_level(&self) -> Result<Vec<u8>, CandidateError> {
        let output: Vec<u8> = self.run(&[
            OsString::from("rev-parse"),
            OsString::from("--show-toplevel"),
        ])?;
        let top: &[u8] = strip_git_line(output.as_slice());
        if top.is_empty() {
            return Err(attribute_failure(
                CandidateFailure::ListingMalformed,
                "git rev-parse --show-toplevel printed no path.",
            ));
        }
        return Ok(top.to_vec());
    }

    /// What:
    ///  The `git check-attr` command for one chunk of names under one source.
    /// Why:
    ///   The index source adds `--cached`,
    ///  which reads `.gitattributes` from the private index and ignores the worktree.
    ///  A tree source adds `--source`,
    ///  which reads them from that commit or tree.
    ///  The names follow `--`,
    /// so none can be read as an option.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const command = ['check-attr', '-z', ...sourceFlags, 'text', '--', ...names];
    /// ```
    fn command(&self, tree: Option<&ObjectId>, names: &[Vec<u8>]) -> Result<Vec<OsString>, CandidateError> {
        let mut command: Vec<OsString> = vec![
            OsString::from("check-attr"),
            OsString::from("-z"),
        ];
        match tree {
            Some(object) => command.push(OsString::from(format!("--source={}", object.as_str()))),
            None => command.push(OsString::from("--cached")),
        }
        command.push(OsString::from("text"));
        command.push(OsString::from("--"));
        for name in names {
            let Some(native) = path_from_git_bytes(name.as_slice()) else {
                return Err(attribute_failure(
                    CandidateFailure::ListingMalformed,
                    "a candidate name is not a path on this platform.",
                ));
            };
            command.push(native.into_os_string());
        }
        return Ok(command);
    }

    /// What:
    ///  The tree each path is answered by,
    ///  as groups of paths in first-seen order.
    ///  `None` stands for the private index.
    /// Why:
    ///   One Git command asks about every path of a group.
    ///  A path without a recorded tree cannot be answered and fails the lookup.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const groups = groupBy(paths, (path) => treeOf(path));
    /// ```
    fn groups<'a>(
        &self,
        paths: &[&'a [u8]],
    ) -> Result<Vec<(Option<ObjectId>, Vec<&'a [u8]>)>, CandidateError> {
        match &self.source {
            AttributeSource::Index => return Ok(vec![(None, paths.to_vec())]),
            AttributeSource::Tree(tree) => return Ok(vec![(Some(tree.clone()), paths.to_vec())]),
            AttributeSource::PerPath(trees) => {
                let mut groups: Vec<(Option<ObjectId>, Vec<&'a [u8]>)> = Vec::new();
                for path in paths {
                    let Some(tree) = trees.get(*path) else {
                        return Err(attribute_failure(
                            CandidateFailure::StaleCandidate,
                            "a candidate was not listed from any commit or tree.",
                        ));
                    };
                    let mut existing: Option<usize> = None;
                    for (group_index, group) in groups.iter().enumerate() {
                        if group.0.as_ref() == Some(tree) {
                            existing = Some(group_index);
                            break;
                        }
                    }
                    match existing {
                        Some(group_index) => groups[group_index].1.push(*path),
                        None => groups.push((Some(tree.clone()), vec![path])),
                    }
                }
                return Ok(groups);
            }
        }
    }

    /// What:
    ///  The given candidate pathnames whose `text` attribute is unset.
    ///  No path means no Git process.
    /// Why:
    ///   A policy asks only about the files it would otherwise change,
    ///  and a commit of canonical files starts no attribute query.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async textUnset(paths: Buffer[]): Promise<Set<string>>; // throws CandidateError
    /// ```
    pub fn text_unset(&self, paths: &[&[u8]]) -> Result<HashSet<Vec<u8>>, CandidateError> {
        let mut unset: HashSet<Vec<u8>> = HashSet::new();
        if paths.is_empty() {
            return Ok(unset);
        }
        let top: Vec<u8> = self.top_level()?;
        for (tree, group) in self.groups(paths)? {
            for chunk in group.chunks(ATTRIBUTE_CHUNK_SIZE) {
                // Each asked name is the top level, a separator and the candidate's pathname.
                let mut asked: Vec<Vec<u8>> = Vec::new();
                for path in chunk {
                    asked.push([top.as_slice(), b"/", *path].concat());
                }
                let command: Vec<OsString> = self.command(tree.as_ref(), asked.as_slice())?;
                let output: Vec<u8> = self.run(command.as_slice())?;
                unset.extend(parse_text_unset(output.as_slice(), asked.as_slice(), chunk)?);
            }
        }
        return Ok(unset);
    }
}

/// Parsing controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_attribute_tests.rs"]
mod tests;
