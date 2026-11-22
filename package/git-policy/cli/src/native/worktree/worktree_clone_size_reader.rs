//! What:
//!  The fence's size read as three readers behind one answer:
//! the one `git ls-tree` the shipped fence takes,
//! one `git cat-file --batch-check` over the ids the fence needs,
//! and one in-process read of the pack entries and the loose objects,
//! which asks Git for the few objects it cannot size.
//! Why:
//!  **A prototype for measurement, not for merging.**
//!  The project asked for a faster reader of the sizes
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 10's notes,
//! the answer of 2026-11-21 which takes "A:
//! merge,
//! then faster reader"),
//! and a report has to time the options before the project picks one;
//! nothing here is shipped,
//! and the selector is reachable only from a measurement build.
//! The answer is one map,
//! so a control can compare the readers entry by entry,
//! and the only thing an option changes is what it costs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sizes = readSizes(reader, read, wanted); // throws the reason the sizes cannot be read
//! ```

use crate::add_timing;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_object::to_hex;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

/// The variable that names the reader a measurement build takes.
pub const READER_VARIABLE: &str = "MONOCHROMATIC_CLONE_SIZE_READER";

/// Which reader answers the fence's sizes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SizeReader {
    /// The shipped read:
    /// one `git ls-tree -r -l`.
    LsTree,
    /// One `git cat-file --batch-check` over the distinct ids the fence needs.
    CatFile,
    /// The pack entries' and loose objects' own headers,
    /// read in this process.
    ObjectDatabase,
}

impl SizeReader {
    /// Every reader this build has,
    /// in the order the report names them.
    pub const ALL: [SizeReader; 3] = [SizeReader::LsTree, SizeReader::CatFile, SizeReader::ObjectDatabase];

    /// The reader a name stands for,
    /// or nothing for a name no reader has.
    pub fn named(name: &str) -> Option<SizeReader> {
        return match name {
            "ls-tree" => Some(SizeReader::LsTree),
            "cat-file" => Some(SizeReader::CatFile),
            "object-database" => Some(SizeReader::ObjectDatabase),
            _ => None,
        };
    }

    /// The reader this process is asked to take.
    /// Why:
    ///  A measurement build picks one by name,
    /// and no name means the shipped read,
    /// so an unset variable times what ships.
    pub fn chosen() -> SizeReader {
        let named: String = std::env::var(READER_VARIABLE).unwrap_or_default();
        return SizeReader::named(named.as_str()).unwrap_or(SizeReader::LsTree);
    }

    /// The reader's name as the record and the report write it.
    pub fn name(self) -> &'static str {
        return match self {
            SizeReader::LsTree => "ls-tree",
            SizeReader::CatFile => "cat-file",
            SizeReader::ObjectDatabase => "object-database",
        };
    }
}

/// What:
///  Where one size read reads from.
/// Why:
///   Every reader answers from the same facts:
/// the repository's root,
/// its common Git directory,
/// and the index entries that name the blobs,
///  which the clean check already proved equal to `HEAD`'s tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SizeRead = { context: GitContext; root: string; commonDir: string; format: ObjectFormat; headCommit: string };
/// ```
pub struct SizeRead<'a> {
    /// The Git runs' context,
    /// which names real Git and the global options.
    pub context: &'a GitContext,
    /// The source checkout's root,
    /// where a run starts.
    pub root: &'a Path,
    /// The repository's common Git directory,
    /// where the object database and its alternates are.
    pub common_dir: &'a Path,
    /// The object format,
    /// which fixes both the name length and the pack reader's strides.
    pub format: ObjectFormat,
    /// The commit whose tree the shipped read lists,
    /// as hexadecimal.
    pub head_commit: &'a str,
}

/// The object format a name length stands for,
/// so a caller holding only the index's hash length can still open the database.
pub fn format_of(hash_length: usize) -> Option<ObjectFormat> {
    return match hash_length {
        20 => Some(ObjectFormat::Sha1),
        32 => Some(ObjectFormat::Sha256),
        _ => None,
    };
}

/// What:
///  The paths whose blob sizes the fence needs,
/// with the id each index entry records.
/// Why:
///   The size proves a file holds its blob,
/// so the read is asked only about the entries the attributes leave to that proof:
/// a converting path,
///  a filter,
///  an `ident`,
///  an encoding or a link is decided without a size.
/// The ids are the index's own,
/// which the clean check proved equal to the tree's,
/// so a reader keyed by id answers what a reader keyed by path does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wanted = entries.filter(needsSize).map((entry) => ({ path: entry.path, id: entry.id }));
/// ```
pub fn wanted_paths(index_bytes: &[u8], format: ObjectFormat, needs_size: &dyn Fn(&[u8], u32) -> bool) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
    let Some(entries) = parse_index(index_bytes, format) else {
        return Err(String::from("the source's index changed shape"));
    };
    let mut wanted: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(entries.len());
    for entry in entries {
        if needs_size(entry.path.as_slice(), entry.mode) {
            wanted.push((entry.path, entry.id));
        }
    }
    return Ok(wanted);
}

/// The distinct ids of `wanted`,
/// in the order they first appear.
fn distinct_ids(wanted: &[(Vec<u8>, Vec<u8>)]) -> Vec<&[u8]> {
    let mut seen: std::collections::HashSet<&[u8]> = std::collections::HashSet::with_capacity(wanted.len());
    let mut ids: Vec<&[u8]> = Vec::with_capacity(wanted.len());
    for (_path, id) in wanted {
        if seen.insert(id.as_slice()) {
            ids.push(id.as_slice());
        }
    }
    return ids;
}

/// What:
///  The blob size of every wanted path,
/// by path,
/// as the reader named answers it.
/// Why:
///   One map is the whole contract,
/// so the fence's decision is the same whatever answers,
///  and a control compares the readers by comparing this.
/// A path the reader has no answer for is absent,
/// which the fence reads as "the tree lists no blob for it".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readSizes(reader, read, wanted): Map<string, number>; // throws
/// ```
pub fn read_sizes(reader: SizeReader, read: &SizeRead, wanted: &[(Vec<u8>, Vec<u8>)]) -> Result<HashMap<Vec<u8>, u64>, String> {
    return match reader {
        SizeReader::LsTree => read_by_path(read, wanted),
        SizeReader::CatFile => {
            let entered: bool = add_timing::begin_step("fenceSizesAskingGit");
            let answered: Result<HashMap<Vec<u8>, u64>, String> = ask_git(read, &distinct_ids(wanted));
            add_timing::end_step(entered);
            return Ok(join(wanted, &answered?));
        }
        SizeReader::ObjectDatabase => read_in_process(read, wanted),
    };
}

/// The shipped read:
/// one `git ls-tree -r -l` over the commit's tree,
/// by path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sizes = lsTree(read.headCommit);
/// ```
fn read_by_path(read: &SizeRead, wanted: &[(Vec<u8>, Vec<u8>)]) -> Result<HashMap<Vec<u8>, u64>, String> {
    let entered: bool = add_timing::begin_step("fenceSizesListingTheTree");
    let listed: Result<HashMap<Vec<u8>, u64>, String> = list_tree(read);
    add_timing::end_step(entered);
    let sizes: HashMap<Vec<u8>, u64> = listed?;
    // Only the wanted paths are handed on, so every reader answers one shape of map.
    let mut answer: HashMap<Vec<u8>, u64> = HashMap::with_capacity(wanted.len());
    for (path, _id) in wanted {
        if let Some(size) = sizes.get(path.as_slice()) {
            answer.insert(path.clone(), *size);
        }
    }
    return Ok(answer);
}

/// One `git ls-tree -r -l -z` of the commit,
/// parsed to path -> blob size.
fn list_tree(read: &SizeRead) -> Result<HashMap<Vec<u8>, u64>, String> {
    let output: GitOutput = run(read, &["ls-tree", "-r", "-l", "-z", read.head_commit], None)?;
    if output.code != Some(0) {
        return Err(String::from("the sizes of the committed blobs cannot be read: `git ls-tree` failed"));
    }
    let mut sizes: HashMap<Vec<u8>, u64> = HashMap::new();
    for record in output.stdout.split(|byte| return *byte == 0) {
        // `mode SP type SP object SP size TAB path`
        let Some(tab) = record.iter().position(|byte| return *byte == b'\t') else { continue };
        let mut fields = record[..tab].split(|byte| return *byte == b' ').filter(|word| return !word.is_empty());
        let (Some(_mode), Some(_kind), Some(_object), Some(size)) = (fields.next(), fields.next(), fields.next(), fields.next()) else { continue };
        let Ok(size_text) = std::str::from_utf8(size) else { continue };
        let Ok(size_value) = size_text.parse::<u64>() else { continue };
        sizes.insert(record[tab + 1..].to_vec(), size_value);
    }
    return Ok(sizes);
}

/// What:
///  The sizes of `ids` as one `git cat-file --batch-check` answers them,
/// by id.
/// Why:
///   Git looks each id up in the object database and reads only what a size needs,
/// and it fetches a promised object itself,
///  so a partial clone answers as its `git ls-tree` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byId = oneCatFile(ids);
/// ```
fn ask_git(read: &SizeRead, ids: &[&[u8]]) -> Result<HashMap<Vec<u8>, u64>, String> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut input: Vec<u8> = Vec::with_capacity(ids.len() * (to_hex(ids[0]).len() + 1));
    for id in ids {
        input.extend_from_slice(to_hex(id).as_bytes());
        input.push(b'\n');
    }
    let output: GitOutput = run(read, &["cat-file", "--batch-check=%(objectname) %(objectsize)"], Some(input))?;
    if output.code != Some(0) {
        return Err(String::from("the sizes of the committed blobs cannot be read: `git cat-file` failed"));
    }
    let mut by_id: HashMap<Vec<u8>, u64> = HashMap::with_capacity(ids.len());
    for line in output.stdout.split(|byte| return *byte == b'\n') {
        let mut fields = line.split(|byte| return *byte == b' ');
        let (Some(name), Some(size)) = (fields.next(), fields.next()) else { continue };
        let Some(name_raw) = crate::repo_read_object::from_hex(name, read.format) else { continue };
        let Some(size_value) = std::str::from_utf8(size).ok().and_then(|text| return text.parse::<u64>().ok()) else { continue };
        by_id.insert(name_raw, size_value);
    }
    return Ok(by_id);
}

/// The in-process read:
/// the object database's own headers,
/// and one Git run for the ids it cannot size,
/// which is what a promisor,
/// a replace ref,
/// a shallow history or an alternate it cannot follow leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sizes = inProcess(ids, askGitForTheRest);
/// ```
fn read_in_process(read: &SizeRead, wanted: &[(Vec<u8>, Vec<u8>)]) -> Result<HashMap<Vec<u8>, u64>, String> {
    let entered: bool = add_timing::begin_step("fenceSizesOpeningTheDatabase");
    let opened: Option<ObjectDatabase> = ObjectDatabase::open(read.common_dir, None, read.format);
    add_timing::end_step(entered);
    let ids: Vec<&[u8]> = distinct_ids(wanted);
    let Some(database) = opened else {
        // An object database this process must not read whole:
        //  Git answers every id, and the fence still decides.
        add_timing::note("fenceSizesFellBack", "the object database refuses this repository");
        let answered: Result<HashMap<Vec<u8>, u64>, String> = ask_git(read, ids.as_slice());
        return Ok(join(wanted, &answered?));
    };
    let entered: bool = add_timing::begin_step("fenceSizesReadingTheDatabase");
    let mut by_id: HashMap<Vec<u8>, u64> = HashMap::with_capacity(ids.len());
    let mut unknown: Vec<&[u8]> = Vec::new();
    for id in &ids {
        match database.size(id) {
            Some(size) => {
                by_id.insert(id.to_vec(), size);
            }
            None => unknown.push(id),
        }
    }
    add_timing::end_step(entered);
    if !unknown.is_empty() {
        add_timing::note("fenceSizesAskedGit", format!("{} ids", unknown.len()).as_str());
        let answered: Result<HashMap<Vec<u8>, u64>, String> = ask_git(read, unknown.as_slice());
        for (id, size) in answered? {
            by_id.insert(id, size);
        }
    }
    return Ok(join(wanted, &by_id));
}

/// The wanted paths' sizes,
/// joined from the ids' answers.
fn join(wanted: &[(Vec<u8>, Vec<u8>)], by_id: &HashMap<Vec<u8>, u64>) -> HashMap<Vec<u8>, u64> {
    let mut answer: HashMap<Vec<u8>, u64> = HashMap::with_capacity(wanted.len());
    for (path, id) in wanted {
        if let Some(size) = by_id.get(id.as_slice()) {
            answer.insert(path.clone(), *size);
        }
    }
    return answer;
}

/// One Git run of this module,
/// in the source,
/// with the same request shape the fence's own runs use.
fn run(read: &SizeRead, words: &[&str], input: Option<Vec<u8>>) -> Result<GitOutput, String> {
    let arguments: Vec<OsString> = words.iter().map(|word| return OsString::from(*word)).collect();
    let request: GitRequest = GitRequest {
        cwd: read.root.to_path_buf(),
        arguments,
        without_prefix: true,
        environment: Vec::new(),
        input,
        ..GitRequest::default()
    };
    return run_git(read.context, &request).map_err(|error| return format!("a Git run of the size reader could not start ({error})"));
}

#[cfg(test)]
#[path = "worktree_clone_size_reader_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "worktree_clone_size_reader_measure_tests.rs"]
mod measure_tests;
