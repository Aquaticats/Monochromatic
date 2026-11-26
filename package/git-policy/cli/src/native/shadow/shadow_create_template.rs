//! What:
//!  The facts a shadow repository is made from,
//!  resolved once,
//!  and the bytes of the shadow's `config` and `objects/info/alternates` for them.
//! Why:
//!   The pool's fill and a commit's claim compare those same facts,
//!  so a member is only ever taken by a commit that would have written it,
//!  and the alternates name the real object directory byte for byte:
//!  a spelling of a path that is not text sends the shadow to a store that does not exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ShadowTemplate = { commonDir; gitDir; repositoryRoot; objectDirectory; objectFormat; refFormat; format };
//! ```

/// The invocation layout the facts are read from.
use crate::capture_invocation::InvocationLayout;
/// The bytes of a path,
/// for a file that names one.
use crate::path_bytes::path_bytes_of;
/// Format copy and the config bytes.
use crate::shadow_config::{
    RepositoryFormat,
    format_shadow_config,
    read_repository_format,
    repository_format_from_snapshot,
};
/// Process context and why reading the facts can fail.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// What:
///  The facts a shadow repository is made from,
///  resolved once so a member made ahead and a creation made here cannot disagree.
/// Why:
///   The creation writes the repository format,
/// the common directory,
/// the worktree root,
/// the object directory and the ref
/// backend into the shadow;
///  the pool's fill and the pool's claim compare those same facts,
/// so a member is only ever taken by a commit that would have written it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShadowTemplate = { commonDir; gitDir; repositoryRoot; objectDirectory; objectFormat; refFormat; format };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShadowTemplate {
    /// The repository's common Git directory,
    /// whose `config` the shadow includes.
    pub common_dir: PathBuf,
    /// The worktree's own Git directory,
    /// whose per-worktree state is copied.
    pub git_dir: PathBuf,
    /// The worktree root the shadow's `core.worktree` names.
    pub repository_root: PathBuf,
    /// The real object directory the shadow's alternates name.
    pub object_directory: PathBuf,
    /// The object hash algorithm the shadow is made with.
    pub object_format: String,
    /// The ref storage backend of the repository.
    pub ref_format: crate::transaction_journal::RefFormat,
    /// The format facts the shadow's `config` copies from the real common config.
    pub format: RepositoryFormat,
}

impl ShadowTemplate {
    /// What:
    ///  The facts of one invocation.
    /// Why:
    ///   The format facts come from the configuration snapshot the invocation already read when that read proves them,
    /// and from the common config file alone otherwise.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ShadowTemplate.of(context, layout)
    /// ```
    pub fn of(
        context: &GitContext,
        layout: &InvocationLayout,
    ) -> Result<ShadowTemplate, GitFailure> {
        let format: RepositoryFormat = match layout
            .config_snapshot
            .as_ref()
            .and_then(repository_format_from_snapshot)
        {
            Some(from_snapshot) => from_snapshot,
            None => read_repository_format(context, layout.common_dir.as_path())?,
        };
        return Ok(ShadowTemplate {
            common_dir: layout.common_dir.clone(),
            git_dir: layout.git_dir.clone(),
            repository_root: layout.repository_root.clone(),
            object_directory: layout.object_directory.clone(),
            object_format: layout.object_format.clone(),
            ref_format: layout.ref_format,
            format,
        });
    }

    /// What:
    ///  The bytes of the shadow's `config` for these facts.
    /// Why:
    ///   The creation writes it and a claim compares a member's own file with it,
    /// so one function says what the file holds;
    ///  the included path and the worktree it names are the bytes the layout holds,
    ///  because a spelling of either loses a name that is not UTF-8.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// template.configBytes()
    /// ```
    pub fn config_bytes(&self) -> Vec<u8> {
        return format_shadow_config(
            &self.format,
            self.common_dir.as_path(),
            self.repository_root.as_path(),
        );
    }

    /// What:
    ///  The bytes of the shadow's `objects/info/alternates` for these facts.
    /// Why:
    ///   The alternates are what lets the shadow's Git read the real object store,
    ///  and a claim checks that a member names this invocation's object directory;
    ///  the object directory's own bytes name it,
    ///  because a spelling of it loses a name that is not UTF-8 and would send the shadow to a store that does not exist.
    ///  A pathname a line would break is written C-quoted,
    /// which Git reads back exactly:
    ///  `parse_alternates` in `odb.c` hands a line that begins with a double quote to
    ///  `unquote_c_style` in `quote.c` of Git 2.56.0.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Buffer.concat([alternatesLine(pathBytes), Buffer.from('\n')])
    /// ```
    pub fn alternates_bytes(&self) -> Vec<u8> {
        let path: &[u8] = path_bytes_of(self.object_directory.as_path());
        let mut text: Vec<u8> = match alternates_line_needs_quoting(path) {
            true => c_quoted(path),
            false => path.to_vec(),
        };
        text.push(b'\n');
        return text;
    }
}

/// What:
///  Whether a pathname has to be written as a quoted line of the shadow's alternates.
/// Why:
///   The file holds one pathname per line and Git reads a line that begins with a double quote as a C-quoted pathname,
///  so a line feed would split the entry in two and a leading quote or hash would change how the line is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.includes('\n') || path.startsWith('"') || path.startsWith('#')
/// ```
fn alternates_line_needs_quoting(path: &[u8]) -> bool {
    return path.contains(&b'\n') || path.starts_with(b"\"") || path.starts_with(b"#");
}

/// What:
///  A pathname as one C-quoted line of the shadow's alternates,
/// which Git unquotes byte for byte.
/// Why:
///   A quote inside a quoted line would end the pathname early and a backslash would start an escape,
///  so the quote,
/// the backslash and the control bytes are written as the escapes Git reads back
///  (`unquote_c_style` in `quote.c`),
/// and every other byte is written as itself,
///  which keeps a name that is not text the name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `"${path.replace(/[\\"\n\r\t]/g, escapeC)}"`
/// ```
fn c_quoted(path: &[u8]) -> Vec<u8> {
    let mut text: Vec<u8> = Vec::with_capacity(path.len() + 2);
    text.push(b'"');
    for byte in path {
        match byte {
            b'"' => text.extend_from_slice(b"\\\""),
            b'\\' => text.extend_from_slice(b"\\\\"),
            b'\n' => text.extend_from_slice(b"\\n"),
            b'\r' => text.extend_from_slice(b"\\r"),
            b'\t' => text.extend_from_slice(b"\\t"),
            _ => text.push(*byte),
        }
    }
    text.push(b'"');
    return text;
}
