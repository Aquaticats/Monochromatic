//! What:
//!  The tree Git reads `.gitattributes` from in a merge that has no worktree,
//!  and where the word-level second pass of such a merge reads the `merge` attribute.
//! Why:
//!  Git takes that tree from `--attr-source`,
//!  else from `GIT_ATTR_SOURCE`,
//!  else from the configuration `attr.tree` (`attr.c`,
//!  `compute_default_attr_source`,
//!  lines 1202 to 1231),
//!  and a merge computed from objects has no worktree to read a rule file from.
//!  The pass names the tree the merge produced only where the person named none,
//!  and never when the merge left a `.gitattributes` conflicted:
//!  a rule read from between Git's markers is no side's rule.
//!  The in-process readers decline a repository whose environment holds a `GIT_*` variable they do not know,
//!  so a declined repository is asked of Git rather than silently losing the person's tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const variables = await attributeTreeVariables(tip, probe); // [['GIT_ATTR_SOURCE', tip]] | []
//! const source = await mergeAttributeSource(tree, conflicts, probe); // 'invocation' | 'unknown' | { tree }
//! ```

use crate::child_environment::environment_value;
use crate::diagnostic_log::debug;
use crate::merge_second_pass::{
    AttributeSource,
    Conflict,
    GitRunner,
    is_attributes_path,
};
use crate::repo_read_config_list::{
    canonical_key,
    list_config,
};
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_layout::read_layout_after_success_with;
use std::ffi::OsString;
use std::path::Path;

/// The tag every line about this part of the second pass is tagged with.
/// Why:
///  The subject is one,
/// so every file it was split into carries the same word,
///  and a person's search of the log keeps finding all of it.
pub(super) const TAG: &str = "wordMergeTree";

/// The variable that names the tree Git reads `.gitattributes` from when there is no worktree.
pub const ATTRIBUTE_SOURCE_VARIABLE: &str = "GIT_ATTR_SOURCE";

/// The configuration key that names that tree when neither `--attr-source` nor the variable does (`Documentation/config/attr.adoc`).
const ATTRIBUTE_TREE_KEY: &str = "attr.tree";

/// What the attribute-tree choice needs to know about where a merge runs.
pub struct AttributeProbe<'a> {
    /// The worktree (or a directory inside the repository) the merge belongs to.
    pub directory: &'a Path,
    /// The invocation's global options,
    /// among which `--attr-source` may be.
    pub global_prefix: &'a [OsString],
    /// The complete environment the Git children get:
    /// the process's own and the overlay.
    pub variables: &'a [(OsString, OsString)],
    /// How to start Git,
    /// to resolve a configured `attr.tree`.
    pub runner: &'a dyn GitRunner,
}

/// What:
///  The value of `attr.tree`,
/// read in process when the readers take the repository and from Git when they decline it;
/// `None` when
///  neither can say,
/// and `Some(None)` when it is not configured.
/// Why:
///   The in-process reader declines an environment that holds a `GIT_*` variable it does not know (`GIT_EDITOR` is one many
///  people set),
/// and a declined repository must not silently lose the person's attribute tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configuredAttributeTree(probe: AttributeProbe): string | null | undefined;
/// ```
async fn configured_attribute_tree(probe: &AttributeProbe<'_>) -> Option<Option<String>> {
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(probe.variables);
    let listed = match read_layout_after_success_with(probe.directory, probe.variables, &scope) {
        Some(layout) => list_config(
            layout.common_dir.as_path(),
            layout.git_dir.as_path(),
            &scope,
            probe.variables,
        ),
        None => None,
    };
    if let Some(entries) = listed {
        // The last entry of the key is the value Git uses; empty text and bytes that are not text count as no value.
        let mut value: Option<String> = None;
        for item in entries.iter().rev() {
            if canonical_key(item.entry.key.as_str()).as_deref() == Some(ATTRIBUTE_TREE_KEY) {
                if let Some(bytes) = item.entry.value.as_deref()
                    && let Ok(text) = std::str::from_utf8(bytes)
                    && !text.is_empty()
                {
                    value = Some(text.to_owned());
                }
                break;
            }
        }
        return Some(value);
    }
    debug(
        TAG,
        "the in-process readers decline the repository, so Git is asked for attr.tree",
    );
    let reply = probe
        .runner
        .run(&["config", "--get", ATTRIBUTE_TREE_KEY], None, None)
        .await
        .ok()?;
    return match reply.code {
        Some(0) => {
            let text: String = String::from_utf8_lossy(&reply.stdout).trim().to_owned();
            Some(if text.is_empty() { None } else { Some(text) })
        }
        Some(1) => Some(None),
        _ => None,
    };
}

/// What:
///  The value the invocation's own global options give `attr.tree`,
/// the last one winning,
/// or nothing when the command line names no attribute tree.
/// Why:
///   Git takes a `-c name=value` and a `--config-env=name=variable` above every configuration file
/// (`git_config_from_parameters`),
/// so a value given there is the one Git uses
///  whether a file sets the key or not;
/// a value no tree resolves to is ignored,
/// as a configured one is,
/// and is told apart from
///  none at all here.
/// `--config-env` takes its value from the environment,
/// and Git stops before it reads anything when
///  the variable it names is not there,
/// so that case is told apart from "nothing names the key".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandLineAttributeTree(prefix: string[], variables: [string, string][]): 'not-set' | { value: string | null } | 'unknown';
/// ```
fn command_line_attribute_tree(
    prefix: &[OsString],
    variables: &[(OsString, OsString)],
) -> CommandLineTree {
    let mut named: CommandLineTree = CommandLineTree::NotSet;
    let mut tokens = prefix.iter();
    while let Some(token) = tokens.next() {
        let Some(text) = token.to_str() else { continue };
        // The entry an option carries: `-c` and a bare `--config-env` take the next token, and `--config-env=` carries it.
        let (entry, from_environment): (&str, bool) =
            if let Some(carried) = text.strip_prefix("--config-env=") {
                (carried, true)
            } else if text == "-c" || text == "--config-env" {
                let Some(next) = tokens.next() else { continue };
                let Some(carried) = next.to_str() else {
                    continue;
                };
                (carried, text == "--config-env")
            } else {
                continue;
            };
        // `-c name=value` splits at the first `=` and `--config-env=name=variable` at the last (`config.c` lines 495 and 511):
        // the variable name may hold no `=` while the key may, and Git refuses a `--config-env` entry with none at all.
        let split: Option<(&str, &str)> = if from_environment {
            entry.rsplit_once('=')
        } else {
            Some(entry.split_once('=').unwrap_or((entry, "")))
        };
        let Some((key, given)) = split else { continue };
        if canonical_key(key).as_deref() != Some(ATTRIBUTE_TREE_KEY) {
            continue;
        }
        if !from_environment {
            // `-c name` alone sets `true`, which no tree resolves to.
            named = if given.is_empty() {
                CommandLineTree::Set(None)
            } else {
                CommandLineTree::Set(Some(given.to_owned()))
            };
            continue;
        }
        let Some(value) = environment_value(variables, given) else {
            named = CommandLineTree::Unknown;
            continue;
        };
        let from_the_variable: String = value.to_string_lossy().into_owned();
        // An empty value is no value, which Git ignores as it ignores a configured one.
        named = if from_the_variable.is_empty() {
            CommandLineTree::Set(None)
        } else {
            CommandLineTree::Set(Some(from_the_variable))
        };
    }
    return named;
}

/// What the invocation's global options say about `attr.tree`.
enum CommandLineTree {
    /// No option sets the key.
    NotSet,
    /// An option sets it to this value,
    /// or to no value Git can resolve (`-c attr.tree` alone,
    /// or an empty value).
    Set(Option<String>),
    /// An option names the key,
    /// but takes its value from a variable the environment does not hold,
    /// so Git stops before it reads anything.
    Unknown,
}

/// What the attribute-tree precedence came to for a merge that must name a tree.
enum AttributeChoice {
    /// The person named one (`--attr-source`,
    /// `GIT_ATTR_SOURCE`,
    /// a resolving `attr.tree`),
    /// or nothing can say whether one is named.
    ThePerson,
    /// Nothing names one.
    Free,
}

/// What:
///  The attribute-tree precedence Git itself applies,
/// without naming a tree.
/// Why:
///   Git takes the attribute tree from `--attr-source`,
/// else from `GIT_ATTR_SOURCE`,
/// else from the configuration `attr.tree`
///  (`attr.c`,
/// `compute_default_attr_source`,
/// lines 1202 to 1231);
/// a value of the configuration that does not resolve to a tree
///  is ignored,
/// as Git ignores it,
/// while one of the other two that does not resolve is an error of Git's own.
/// A layout or a
///  configuration the in-process readers decline counts as a person's choice that cannot be seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributeChoice(probe: AttributeProbe): 'the-person' | 'free';
/// ```
async fn attribute_choice(probe: &AttributeProbe<'_>) -> AttributeChoice {
    for (name, _value) in probe.variables {
        if name == ATTRIBUTE_SOURCE_VARIABLE {
            return AttributeChoice::ThePerson;
        }
    }
    for token in probe.global_prefix {
        if let Some(text) = token.to_str()
            && (text == "--attr-source" || text.starts_with("--attr-source="))
        {
            return AttributeChoice::ThePerson;
        }
    }
    let read: Option<Option<String>> = match command_line_attribute_tree(
        probe.global_prefix,
        probe.variables,
    ) {
        // The command line names the key and Git stops on the value it cannot find, so no tree is named over it.
        CommandLineTree::Unknown => {
            debug(
                TAG,
                "an attr.tree of the command line takes its value from a variable the environment does not hold, and Git stops on it, so no attribute tree is named",
            );
            return AttributeChoice::ThePerson;
        }
        CommandLineTree::Set(from_the_command_line) => Some(from_the_command_line),
        CommandLineTree::NotSet => configured_attribute_tree(probe).await,
    };
    let Some(configured) = read else {
        debug(
            TAG,
            "attr.tree cannot be read, so no attribute tree is named",
        );
        return AttributeChoice::ThePerson;
    };
    if let Some(value) = configured {
        let spec: String = format!("{value}^{{tree}}");
        let resolves: bool = match probe
            .runner
            .run(
                &["rev-parse", "--verify", "--quiet", spec.as_str()],
                None,
                None,
            )
            .await
        {
            Ok(reply) => reply.code == Some(0),
            Err(_) => false,
        };
        if resolves {
            return AttributeChoice::ThePerson;
        }
        debug(
            TAG,
            format!(
                "attr.tree names {value}, which is no tree; Git ignores it, so the tree is named"
            )
            .as_str(),
        );
    }
    return AttributeChoice::Free;
}

/// What:
///  The variable pair that makes a merge from objects read the attributes of `tip`,
/// only where Git itself would read no
///  attribute tree.
/// Why:
///   So the first side's tip is named only when none of the three sources applies:
/// the person's choice is never
///  overridden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attributeTreeVariables(tip: string, probe: AttributeProbe): Promise<[string, string][]>;
/// ```
pub async fn attribute_tree_variables(
    tip: &str,
    probe: &AttributeProbe<'_>,
) -> Vec<(OsString, OsString)> {
    return match attribute_choice(probe).await {
        AttributeChoice::ThePerson => Vec::new(),
        AttributeChoice::Free => vec![(
            OsString::from(ATTRIBUTE_SOURCE_VARIABLE),
            OsString::from(tip),
        )],
    };
}

/// What:
///  The same variables for a caller that is not inside the runtime,
/// entered on the runtime once.
/// Why:
///   The isolated merge,
/// the commit replay merge and the shadow replay read the attribute tree
/// on a thread of their own,
/// and the face above is refused there only inside the runtime;
/// the choice,
/// the questions and the words are the awaited face's,
/// so the two cannot name different trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(() => attributeTreeVariables(tip, probe))
/// ```
pub fn attribute_tree_variables_on_the_runtime(
    tip: &str,
    probe: &AttributeProbe<'_>,
) -> Vec<(OsString, OsString)> {
    return match crate::lazy_runtime::run_on_runtime(attribute_tree_variables(tip, probe)) {
        Ok(found) => found,
        Err(refusal) => {
            crate::diagnostic_log::warn(
                TAG,
                format!("the attribute tree could not be read, so none is named: {refusal}")
                    .as_str(),
            );
            Vec::new()
        }
    };
}

/// What:
///  Whether one of the conflicts of a merge from objects is a `.gitattributes` file.
/// Why:
///   The tree the merge produced carries Git's markers in a rule file both sides changed,
/// so no rule of it can be trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anyAttributesConflict = (conflicts: Conflict[]) => conflicts.some((conflict) => isAttributesPath(conflict.path));
/// ```
fn any_attributes_conflict(conflicts: &[Conflict]) -> bool {
    for conflict in conflicts {
        if is_attributes_path(conflict.path.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Where the second pass of a merge from objects reads the `merge` attribute of its candidates.
/// Why:
///   The tree the merge produced carries the `.gitattributes` of both sides as the merge left them,
/// so a rule the
///  merge itself introduces applies to the files of that same merge;
/// reading the first side's tip alone let such a
///  file be merged by word and committed.
/// The tree is named only where the person named none,
/// and only when the merge
///  left no `.gitattributes` conflicted:
/// a conflicted one holds Git's markers,
/// and a rule read from
///  between them is no side's rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeAttributeSource(tree: string, conflicts: Conflict[], probe: AttributeProbe): AttributeSource;
/// ```
pub async fn merge_attribute_source<'a>(
    tree: &'a str,
    conflicts: &[Conflict],
    probe: &AttributeProbe<'_>,
) -> AttributeSource<'a> {
    return match attribute_choice(probe).await {
        AttributeChoice::ThePerson => AttributeSource::Invocation,
        AttributeChoice::Free if any_attributes_conflict(conflicts) => AttributeSource::Unknown,
        AttributeChoice::Free => AttributeSource::Tree(tree),
    };
}
