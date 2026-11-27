//! What:
//!  The provider that asks real Git for a repository fact:
//!  the values it holds,
//!  how it is built,
//!  the one query every fact goes through,
//!  and the small messages and readings its questions are made of.
//! Why:
//!   The interface beside this file (`repository_facts.rs`) states which facts a rule may ask for;
//!  this file is the one implementation the executable uses,
//!   and the trait's implementation for it lives beside the interface,
//!   because one type implements a trait in one block.
//!  A provider owns copies of the executable path,
//!  the caller's global options and the child environment additions,
//!   so it can be handed around without borrowing rules,
//!   and its query count lets a test prove how many Git processes an invocation started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type GitFacts = { realGit: string; globalPrefix: string[]; overlay: [string, string][]; location?: Result; queries: number };
//! ```

use crate::git_metadata::{
    MetadataOutput,
    read_stopped_text,
    run_metadata_git,
};
use crate::git_system_config::with_real_git;
use crate::kept_names_in_the_git_directories::settle_kept_names;
use crate::repo_read_layout::{
    RepoLayout,
    read_layout,
};
use crate::repository_location::RepositoryLocation;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The provider that asks real Git.
///  A `struct` is a record with named fields;
///       `Option<T>` is "a value or nothing";
///  `usize` is the unsigned integer of counts.
/// Why:
///   It owns copies of the executable path,
///  the caller's global options and the child
///       environment additions,
///  so it can be handed around without borrowing rules.
///  The
///       query count lets a test prove how many Git processes an invocation started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitFacts = { realGit: string; globalPrefix: string[]; overlay: [string, string][]; location?: Result; queries: number };
/// ```
#[derive(Clone, Debug)]
pub struct GitFacts {
    /// The selected real Git executable.
    pub(super) real_git: PathBuf,
    /// The arguments before the subcommand,
    ///  without wrapper controls.
    pub(super) global_prefix: Vec<OsString>,
    /// Variables added to every Git child.
    pub(super) overlay: Vec<(OsString, OsString)>,
    /// The remembered location answer,
    ///  once asked.
    pub(super) location: Option<Result<RepositoryLocation, String>>,
    /// The in-process layout,
    /// read once:
    /// `Some(None)` means the reader declined and Git answers.
    pub(super) layout: Option<Option<RepoLayout>>,
    /// How many Git processes this provider started.
    pub queries: usize,
}

/// What:
///  Build a provider that has asked nothing yet.
///  `&Path` and `&[...]` borrow the
///       caller's values;
///  the provider keeps its own copies.
/// Why:
///   Creating a provider costs no process;
///  only a rule that needs a fact pays for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitFacts(realGit: string, globalPrefix: string[], overlay: [string, string][]): GitFacts;
/// ```
pub fn git_facts(
    real_git: &Path,
    global_prefix: &[OsString],
    overlay: &[(OsString, OsString)],
) -> GitFacts {
    return GitFacts {
        // `.to_path_buf()` and `.to_vec()` copy the borrowed values into owned ones.
        real_git: real_git.to_path_buf(),
        global_prefix: global_prefix.to_vec(),
        overlay: overlay.to_vec(),
        // `None` is the "absent" case of `Option`: nothing has been asked.
        location: None,
        layout: None,
        queries: 0,
    };
}

/// What:
///  Say that real Git could not be started for a fact.
///  `&std::io::Error` borrows the
///       operating system's error.
/// Why:
///   The message names the executable and the fact,
///  so the reader knows what failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = `monochromatic-git-policy-cli could not start ${realGit} to read ${fact}: ${error}`;
/// ```
fn not_started(
    real_git: &Path,
    fact: &str,
    error: &std::io::Error,
) -> String {
    // `format!` builds owned text; `.display()` shows a path, replacing undecodable bytes.
    return format!(
        "monochromatic-git-policy-cli could not start {} to read {fact}: {error}",
        real_git.display()
    );
}

/// Private helpers of the real-Git provider.
impl GitFacts {
    /// What:
    ///  Run one captured Git query after the caller's global options and count it.
    ///       `&mut self` lends the provider for writing;
    ///  `Vec<OsString>` is the owned
    ///       argument list,
    ///  already starting with the global options.
    /// Why:
    ///   Every fact goes through here,
    ///  so the count cannot miss a process.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async query(args: string[], fact: string): Promise<MetadataOutput>;
    /// ```
    pub(super) fn query(
        &mut self,
        arguments: Vec<OsString>,
        fact: &str,
    ) -> Result<MetadataOutput, String> {
        self.queries += 1;
        // `match` unpacks the start result; `Ok`/`Err` are the success/failure variants.
        match run_metadata_git(
            self.real_git.as_path(),
            arguments.as_slice(),
            self.overlay.as_slice(),
        ) {
            Ok(output) => return Ok(output),
            // A read the wrapper ended at its ceiling says what blocked it and what stayed as it was.
            // `&error` borrows the error for the message.
            Err(error) => {
                return Err(match read_stopped_text(fact, &error) {
                    Some(stopped) => stopped,
                    None => not_started(self.real_git.as_path(), fact, &error),
                });
            }
        }
    }

    /// What:
    ///  The repository layout read from the file system,
    /// once per provider;
    ///  `None` when the reader declined and Git must answer.
    /// Why:
    ///   Location and sequencer questions share one read and start no Git process when the layout is plain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// layout(): RepoLayout | undefined; // memoized
    /// ```
    pub(super) fn layout(&mut self) -> Option<&RepoLayout> {
        if self.layout.is_none() {
            self.layout = Some(read_layout(
                self.global_prefix.as_slice(),
                with_real_git(self.overlay.as_slice(), self.real_git.as_path()).as_slice(),
            ));
            // The first time the Git directories are known, a name an earlier release kept in them is moved to its current name.
            if let Some(Some(known)) = &self.layout {
                settle_kept_names(known.git_dir.as_path(), known.common_dir.as_path());
            }
        }
        return self.layout.as_ref().and_then(Option::as_ref);
    }
}

/// What:
///  The text one unreadable location answer is reported as.
///  `std::io::Error` is the operating system's own error.
/// Why:
///   Callers of the facts interface report failures as text,
///  and a named function keeps the conversion out of a closure the linter cannot keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errorAsMessage = (error: Error) => error.message;
/// ```
pub(super) fn error_as_message(error: std::io::Error) -> String {
    return error.to_string();
}
