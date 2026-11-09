//! What:
//! An in-memory workspace for the dependent-version planner's tests,
//! which records
//!       every read.
//! Why:
//! The planner reads through the content seam only,
//! so tests answer from memory and
//!      can prove which files were read and in what order,
//! and what happens when a read
//!      fails.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const workspace = memoryWorkspace([manifest('package/module/a', { name: 'a', version: '1.0.0' })]);
//! ```

/// The content seam this provider implements.
use crate::dependent_version_content::{
    ContentUnavailable, TrackedMode, TrackedPath, WorkspaceContent,
};

/// One file of the in-memory workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryFile {
    /// Repository path.
    pub path: Vec<u8>,
    /// Mode at the candidate state.
    pub mode: TrackedMode,
    /// Candidate bytes.
    pub current: Vec<u8>,
    /// Base bytes,
    /// when the base has the path.
    pub base: Option<Vec<u8>>,
}

/// Which read the planner made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Read {
    /// The tracked-path listing.
    Listing,
    /// Candidate bytes of a path.
    Candidate(
        /// The path.
        Vec<u8>,
    ),
    /// Base bytes of a path.
    Base(
        /// The path.
        Vec<u8>,
    ),
}

/// The in-memory workspace.
#[derive(Clone, Debug, Default)]
pub struct MemoryWorkspace {
    /// Tracked files in listing order.
    pub files: Vec<MemoryFile>,
    /// Paths whose reads fail.
    pub unreadable: Vec<Vec<u8>>,
    /// Whether the listing fails.
    pub listing_unavailable: bool,
    /// Every read,
    /// in order.
    pub reads: Vec<Read>,
}

/// The failure for a path this workspace refuses to read.
fn unavailable(path: &[u8], reason: &str) -> ContentUnavailable {
    return ContentUnavailable {
        path: Some(path.to_vec()),
        reason: String::from(reason),
    };
}

/// Answers from memory,
/// recording each read.
impl WorkspaceContent for MemoryWorkspace {
    /// Every file,
    /// unless the listing is set to fail.
    fn tracked_paths(&mut self) -> Result<Vec<TrackedPath>, ContentUnavailable> {
        self.reads.push(Read::Listing);
        if self.listing_unavailable {
            return Err(ContentUnavailable {
                path: None,
                reason: String::from("listing refused"),
            });
        }
        return Ok(self
            .files
            .iter()
            .map(|file: &MemoryFile| {
                return TrackedPath {
                    path: file.path.clone(),
                    mode: file.mode,
                };
            })
            .collect());
    }

    /// The candidate bytes,
    /// unless reads of the path fail.
    fn candidate_bytes(&mut self, path: &[u8]) -> Result<Vec<u8>, ContentUnavailable> {
        self.reads.push(Read::Candidate(path.to_vec()));
        return self
            .file(path)
            .map(|file: &MemoryFile| return file.current.clone());
    }

    /// The base bytes,
    /// unless reads of the path fail.
    fn base_bytes(&mut self, path: &[u8]) -> Result<Option<Vec<u8>>, ContentUnavailable> {
        self.reads.push(Read::Base(path.to_vec()));
        return self
            .file(path)
            .map(|file: &MemoryFile| return file.base.clone());
    }
}

/// Lookup shared by both byte reads.
impl MemoryWorkspace {
    /// The tracked file at a path,
    /// unless reads of it fail.
    fn file(&self, path: &[u8]) -> Result<&MemoryFile, ContentUnavailable> {
        if self
            .unreadable
            .iter()
            .any(|refused: &Vec<u8>| return refused == path)
        {
            return Err(unavailable(path, "read refused"));
        }
        return self
            .files
            .iter()
            .find(|file: &&MemoryFile| return file.path == path)
            .ok_or_else(|| return unavailable(path, "not tracked"));
    }
}

/// A workspace of the given files.
pub fn workspace(files: Vec<MemoryFile>) -> MemoryWorkspace {
    return MemoryWorkspace {
        files,
        ..MemoryWorkspace::default()
    };
}

/// A regular file whose candidate bytes equal its base bytes.
pub fn unchanged(path: &str, text: &str) -> MemoryFile {
    return MemoryFile {
        path: path.as_bytes().to_vec(),
        mode: TrackedMode::Regular,
        current: text.as_bytes().to_vec(),
        base: Some(text.as_bytes().to_vec()),
    };
}

/// A regular file with separate candidate and base text;
/// `None` for a new file.
pub fn changed(path: &str, text: &str, base: Option<&str>) -> MemoryFile {
    return MemoryFile {
        path: path.as_bytes().to_vec(),
        mode: TrackedMode::Regular,
        current: text.as_bytes().to_vec(),
        base: base.map(|found: &str| return found.as_bytes().to_vec()),
    };
}

/// A manifest as the incumbent's fixtures write one:
/// two-space JSON and a final newline.
pub fn manifest_text(fields: &str) -> String {
    return format!("{{\n{fields}\n}}\n");
}

/// The repository path of the generated registry configuration.
pub const CONFIG_PATH: &str = "package/config/pnpr/config.yaml";

/// One rendered list item of the generator's package list,
/// at the indent the generator writes
///       it.
fn config_item(name: &str) -> String {
    return format!("            - '{name}'");
}

/// The registry configuration the generator writes around one package list.
/// Why:
///  The publish set is the OIDC workload list,
/// inside `auth.oidc[].workloads[]`;
/// every
///       other line is noise the reader has to descend through.
pub fn config_text(names: &[&str]) -> String {
    let items: Vec<String> = names
        .iter()
        .map(|name: &&str| return config_item(name))
        .collect();
    return format!(
        "auth:\n  oidc:\n    - name: github\n      workloads:\n        - identity:\n            subject: 'repo:example/main'\n          registry: r\n          packages:\n{}\n\nweb:\n  enable: false\n",
        items.join("\n")
    );
}

/// The registry configuration listing names in the generator's shape.
pub fn config(names: &[&str]) -> MemoryFile {
    return unchanged(CONFIG_PATH, &config_text(names));
}

/// The registry configuration with a narrower workload list above the publish set.
/// Why:
///  The generator writes one list today;
/// a second,
///       narrower block above it must not hide the publish set.
pub fn config_with_a_narrower_list_above(names: &[&str]) -> MemoryFile {
    let items: Vec<String> = names
        .iter()
        .map(|name: &&str| return config_item(name))
        .collect();
    let text: String = format!(
        "auth:\n  oidc:\n    - name: github\n      workloads:\n        - identity:\n            subject: 'repo:example/release'\n          registry: r\n          packages:\n            - '@s/narrow'\n        - identity:\n            subject: 'repo:example/main'\n          registry: r\n          packages:\n{}\n\nweb:\n  enable: false\n",
        items.join("\n")
    );
    return unchanged(CONFIG_PATH, &text);
}
