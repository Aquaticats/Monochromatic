//! What:
//!  The words one slow-route sentence is made of:
//!  the value a person reads,
//!  the two leads a line starts with,
//!  how the source is called in it,
//!  the closing words of a remedy,
//!  and the place both the terminal and the journal are told.
//! Why:
//!  Every cause produces the same shape of line,
//!  so the shape is one place:
//!  a lead chosen by which checkout wrote the worktree,
//!  the cause in the person's words,
//!  and what makes the next add quick.
//!  `worktree_slow_route_unclean.rs` builds the sentence for a source that cannot be shown clean,
//!  and `worktree_slow_route.rs` builds the sentences for each rung's ending;
//!  both are re-exported there so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = new SlowRoute({ word, lead, detail, sentence }); route.tell(plan);
//! ```

use crate::client_journal::record_entry;
use crate::diagnostic_log::debug;
use crate::journald_native::Priority;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_identity::WorktreeIdentity;
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSlowRoute";

/// The start of the line on standard error and of the journal message;
/// the cause and the remedy follow.
pub const LEAD: &str = "monochromatic-git-policy-cli: this worktree was not made as a snapshot, so it took longer, because ";

/// The start of the line for a rung that declined before it copied anything.
///
/// Why:
///  Nothing of the add had started when a limit or the fence declined it,
/// so the line must not say the add took longer;
/// it says which checkout wrote the worktree,
/// and the cause follows
/// (the integration pass's low L4).
pub const DECLINED_LEAD: &str =
    "monochromatic-git-policy-cli: Git's own checkout wrote this worktree, because ";

/// What:
///  What kept the add from being a snapshot,
/// as a sentence for the person and a word for the journal.
/// Why:
///   One sentence and one word per cause,
/// chosen by what the cause is and never by the ladder's step that found it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlowRoute = { word: string; detail: string; sentence: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlowRoute {
    /// The journal field `CLI_GIT_SLOW_CAUSE`:
    /// a short word a reader can filter on.
    ///
    /// Its four fields are visible to this folder and no further,
    /// because the sentence for an unclean source (`worktree_slow_route_unclean.rs`) and the sentences for each
    /// rung's ending (`worktree_slow_route.rs`) are two files that fill one shape.
    pub(crate) word: &'static str,
    /// The start of the line,
    /// which chooses whether it says the add took longer or which checkout wrote it.
    pub(crate) lead: &'static str,
    /// The journal field `CLI_GIT_SLOW_DETAIL`:
    /// the check's own reason,
    /// as the debug log has it.
    pub(crate) detail: String,
    /// The rest of the line after `LEAD`:
    /// the cause,
    /// then what makes the next add quick.
    pub(crate) sentence: String,
}

impl SlowRoute {
    /// The journal word of the cause.
    pub fn word(&self) -> &'static str {
        return self.word;
    }

    /// The check's own reason.
    pub fn detail(&self) -> &str {
        return self.detail.as_str();
    }

    /// The whole line,
    /// without its newline.
    pub fn line(&self) -> String {
        return format!("{}{}", self.lead, self.sentence);
    }

    /// What:
    ///  Tell the person on standard error unless the add was quiet,
    /// and tell the journal either way.
    /// Why:
    ///   `--quiet` silences the terminal and nothing else;
    /// the journal is where the cause is found afterwards.
    /// A closed
    ///  standard error is no failure of the add.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (!plan.quiet) console.error(route.line()); journal.warn(route.line(), fields);
    /// ```
    pub fn tell(
        &self,
        plan: &AddPlan,
    ) {
        let line: String = self.line();
        debug(
            TAG,
            format!(
                "{}: told ({}; {})",
                plan.destination.display(),
                self.word,
                self.detail
            )
            .as_str(),
        );
        if !plan.quiet {
            // A closed standard error leaves nowhere to print; the journal entry below still records the cause.
            let _written = std::io::stderr().write_all(format!("{line}\n").as_bytes());
        }
        let fields: [(&str, String); 3] = [
            (
                "CLI_GIT_WORKTREE",
                plan.destination.to_string_lossy().into_owned(),
            ),
            ("CLI_GIT_SLOW_CAUSE", self.word.to_string()),
            ("CLI_GIT_SLOW_DETAIL", self.detail.clone()),
        ];
        record_entry(Priority::Warning, line.as_str(), &fields);
    }
}

/// What:
///  How the source is called in a sentence:
/// the main checkout,
/// or the worktree the person ran the add from.
/// Why:
///   The person knows which checkout they sit in;
/// the sentence names it the way they would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'main' ? 'the main checkout' : 'this worktree'
/// ```
pub(crate) fn subject(identity: &WorktreeIdentity) -> &'static str {
    return match identity {
        WorktreeIdentity::MainWorktree {
            ..
        } => "the main checkout",
        _ => "this worktree",
    };
}

/// The closing words of a remedy:
/// what the next add becomes.
pub(crate) const THEN_QUICK: &str = "and the next add is a snapshot that takes seconds";
