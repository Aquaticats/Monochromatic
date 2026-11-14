//! What:
//!  The edits a tracked file holds under a mark that keeps Git from noticing them.
//! Why:
//!   `git update-index --assume-unchanged` and `--skip-worktree` tell every command of Git
//!  to believe the index entry rather than the file,
//!  so `git status` calls the worktree clean,
//!  `git worktree remove` deletes it without a word,
//!  and the wrapper's own proof of cleanliness declines
//!  (`worktree_clean.rs`:
//! "base.txt is marked assume-unchanged")
//!  and hands the removal to Git,
//! which cannot see the mark
//!  (a Sonnet audit of 0.6.1-alpha.14,
//!  finding F3).
//!  An edit to a tracked file is Git work,
//!  and the owner's rule is that no removal destroys it however it is hidden,
//!  so the file itself is compared with the blob its index entry names:
//!  the bytes,
//! not the stat data Git was told to ignore.
//!  The comparison reads the files in process and under the attributes Git would apply,
//!  exactly as the copy and conversion paths prove a worktree
//!  (`worktree_clean_tree_walk::paths_whose_content_differs`),
//!  so a file whose bytes differ is named,
//!  a file that is absent (a sparse checkout's entry) is not an edit here,
//!  and nothing is started when no entry carries either mark.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const edits = hiddenEditsBeneath(context, worktree, admin, commonDir); // string[] reasons, or the reason nothing was proven
//! ```

use crate::repo_read_index::{IndexEntry, parse_index};
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_index_files::repository_hash_length;
use crate::transaction_git::GitContext;
use crate::worktree_clean_content_attribute_sources::for_context;
use crate::worktree_clean_stat_verdicts::ASSUME_VALID;
use crate::worktree_clean_tree_walk::{ContentTarget, paths_whose_content_differs};
use crate::worktree_clean::{StatEntry, stored_stats};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The extended-flags bit Git uses for a `skip-worktree` entry.
/// Why:
///   The reader keeps a version 3 or 4 entry's extended flags as the number Git wrote,
///  and this bit is the mark itself.
const SKIP_WORKTREE: u16 = 0x4000;

/// How many edited paths one answer names before it counts the rest.
const SHOWN_PATHS: usize = 3;

/// How many digits a SHA-256 object name has,
/// which is the one length that is not SHA-1.
const SHA256_DIGITS: usize = 32;

/// How many bytes of a worktree's index this reads.
/// Why:
///  An index holds one entry per tracked path,
/// so a file past this bound is not one Git wrote for a repository this program meets:
/// on 2026-11-14 this repository's index held 3,333,744 bytes for about 13,500 tracked paths,
/// and the reader of packed references and of one object both stop at 256 MiB as well
/// (`repo_read_refs.rs`, `repo_read_object.rs`).
///  A file past the bound keeps the worktree rather than being read whole,
/// so no file beside a checkout decides how much memory a merge reads.
const INDEX_LIMIT: u64 = 256 << 20;

/// What:
///  The tracked files of `worktree` whose bytes are not the blob their index entry names,
///  under a mark that keeps Git from noticing,
///  as the sentences a person reads,
///  or the reason that could not be proven.
/// Why:
///   The one question a removal asks about hidden edits,
///  and the same answer serves the sweep,
///  the service's removal and the person's own command.
///  A worktree with no marked entry costs one index read and no Git start,
///  which is the ordinary case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reasons = hiddenEditsBeneath(context, worktree, admin, commonDir);
/// ```
///
/// # Errors
///
/// The words for an index or a file this cannot read,
/// which leaves the worktree kept rather than read as clear.
pub(crate) fn hidden_edits_beneath(context: &GitContext, worktree: &Path, admin: &Path, common_dir: &Path) -> Result<Vec<String>, String> {
    let hash_length: usize = repository_hash_length(common_dir).ok_or_else(|| return String::from("the repository configuration cannot be read, so no marked entry can be compared with its file"))?;
    let format: ObjectFormat = if hash_length == SHA256_DIGITS { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    // One open file gives both the bytes and the time, so the time belongs to the bytes that were read.
    //  The open is the one every record of this program uses: no follow of a final link, and no waiting for a
    //  writer when the name holds a named pipe, which would hold a merge or a removal for ever.
    let index_path: PathBuf = admin.join("index");
    let index_file: std::fs::File = crate::private_storage::open_regular_file(index_path.as_path())
        .map_err(|refusal| return format!("the index of {} cannot be read as one plain file ({}), so no marked entry can be compared with its file", worktree.display(), refusal_words(&refusal)))?;
    let index_seconds: u32 = index_file
        .metadata()
        .map_err(|error| return format!("the index of {} cannot be examined ({error})", worktree.display()))?
        .mtime() as u32;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(index_file, INDEX_LIMIT), &mut bytes)
        .map_err(|error| return format!("the index of {} cannot be read ({error})", worktree.display()))?;
    let entries: Vec<IndexEntry> = parse_index(bytes.as_slice(), format).ok_or_else(|| return format!("the index of {} has a layout this program does not know, so no marked entry can be compared with its file", worktree.display()))?;
    let stats: Vec<StatEntry> = stored_stats(bytes.as_slice(), hash_length).ok_or_else(|| return format!("the index entries of {} have a layout this program does not know", worktree.display()))?;
    let mut marked: Vec<StatEntry> = Vec::new();
    let mut indexed: Vec<IndexEntry> = Vec::new();
    for (stat, entry) in stats.into_iter().zip(entries) {
        if stat.flags & ASSUME_VALID != 0 || entry.extended & SKIP_WORKTREE != 0 {
            marked.push(stat);
            indexed.push(entry);
        }
    }
    if marked.is_empty() {
        return Ok(Vec::new());
    }
    let environment = for_context(context);
    let target: ContentTarget<'_> = ContentTarget { root: worktree, git_dir: admin, common_dir, environment: &environment, format };
    let differing: Vec<Vec<u8>> = paths_whose_content_differs(&target, marked.as_slice(), indexed.as_slice(), index_seconds)
        .map_err(|refusal| return format!("nothing proves the marked entries of {} hold what their index entries name, so nothing is removed: {}", worktree.display(), refusal.reason))?;
    let mut said: Vec<String> = Vec::new();
    let mut counted: usize = 0;
    for path in differing {
        counted += 1;
        if said.len() < SHOWN_PATHS {
            said.push(sentence(path.as_slice()));
        }
    }
    if counted > said.len() {
        said.push(format!("and {} more files whose edits no command of Git sees", counted - said.len()));
    }
    return Ok(said);
}

/// What:
///  Why an index could not be read,
///  in words a person reads.
/// Why:
///  A named pipe or a device at the name is the state this read must refuse rather than wait on,
///  and a person has to know which of the two happened to them:
///  nothing there is a file Git wrote,
///  while an input-output error is the disk's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusalWords(new ReadRefusal.NotRegular()) // 'it is not a plain file (a named pipe, a device or a directory)'
/// ```
fn refusal_words(refusal: &crate::private_storage::ReadRefusal) -> String {
    return match refusal {
        crate::private_storage::ReadRefusal::Missing => String::from("nothing exists at that name"),
        crate::private_storage::ReadRefusal::NotRegular => String::from("it is not a plain file (a named pipe, a device or a directory)"),
        crate::private_storage::ReadRefusal::TooLarge => format!("it holds more than {INDEX_LIMIT} bytes"),
        crate::private_storage::ReadRefusal::Io(error) => format!("the file system refused the read ({error})"),
    };
}

/// What:
///  The sentence for one tracked file whose edit a mark hides.
/// Why:
///   The person decides about their own file,
///  so the sentence names it,
///  says why nothing has seen the edit,
///  and gives the two ways forward:
///  keep the bytes somewhere they can be committed,
///  or remove the mark so Git itself refuses the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sentence(path) // 'the tracked file base.txt is marked ..., and its bytes are not ...'
/// ```
fn sentence(path: &[u8]) -> String {
    let shown: String = String::from_utf8_lossy(path).into_owned();
    // The path is quoted where it enters the command the person copies: a name with a space, a quote, a dollar sign
    // or a line break otherwise turns that command into another one.
    let quoted: String = crate::shell_quote::shell_quote(path);
    return format!(
        "the tracked file {shown} is marked assume-unchanged or skip-worktree, and its bytes are not the ones its index entry \
         names, so no command of Git sees that edit. Commit it, move it somewhere else, or remove the mark \
         (`git update-index --no-assume-unchanged -- {quoted}`, or `--no-skip-worktree`), and remove the worktree again"
    );
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_hidden_edits_tests.rs"]
mod tests;
