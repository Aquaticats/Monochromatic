//! What:
//!  The `postCommit` configuration key: the actions the wrapper runs itself after a commit lands.
//! Why:
//!  The wrapper supports no external hook program, so what a `post-commit` hook did is declared here as data: a name,
//!  an argument array run directly (never through a shell) with fields from a closed set, and a time bound.
//!  Nothing here is executed at load; the arguments are parsed and the fields checked, so a typo fails the load.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // { "postCommit": { "actions": [{ "name": "review", "argv": ["mise", "run", "review-{shortHead}"] }] } }
//! ```

/// The configuration error type and the typed readers.
use super::config_error::ConfigError;
use super::config_values::{member_keys, member_path, safe_integer, text, wrong_kind};
/// The argument templates and their field set.
use super::post_commit_template::{FIELD_LIST, Template, parse_template};
/// The parsed value model.
use monochromatic_jsonc_edit::JsoncValue;

/// The most seconds an action may run when it states none; the wrapper stops it and the commit stays landed.
pub const DEFAULT_ACTION_TIMEOUT_SECONDS: u64 = 60;

/// The most actions one repository declares, so a configuration cannot make every commit start a crowd.
pub const MAXIMUM_ACTIONS: usize = 16;

/// What:
///  One declared action.
/// Why:
///   The runner needs the name for its notices, the parsed arguments and the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PostCommitAction = { name: string; argv: Template[]; timeoutSeconds: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostCommitAction {
    /// The action's name, unique among the declared actions.
    pub name: String,
    /// The program (first) and its arguments, as parsed templates.
    pub argv: Vec<Template>,
    /// The most seconds the action may run.
    pub timeout_seconds: u64,
}

/// What:
///  Parse the `postCommit` value.
/// Why:
///   The only accepted member is `actions`, an array of objects with `name`, `argv` and an optional
///  `timeoutSeconds`; every other shape is refused with the key path and what is accepted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parsePostCommit(value); // PostCommitAction[]
/// ```
pub(crate) fn parse_post_commit(value: &JsoncValue) -> Result<Vec<PostCommitAction>, ConfigError> {
    let Some(entries) = value.entries() else {
        return Err(wrong_kind(value, "postCommit", "an object"));
    };
    let keys: Vec<String> = member_keys(entries, "postCommit")?;
    let mut actions: Vec<PostCommitAction> = Vec::new();
    for (index, key) in keys.iter().enumerate() {
        if key != "actions" {
            return Err(ConfigError::new(
                format!("Unknown configuration key: postCommit.{key}. The only accepted key under postCommit is actions.")
                    .as_str(),
            ));
        }
        actions = parse_actions(&entries[index].value)?;
    }
    return Ok(actions);
}

/// What:
///  Parse the `actions` array.
/// Why:
///   Names must be unique, so a notice about an action points at one declaration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseActions(value); // PostCommitAction[]
/// ```
fn parse_actions(value: &JsoncValue) -> Result<Vec<PostCommitAction>, ConfigError> {
    let path: &str = "postCommit.actions";
    let Some(elements) = value.elements() else {
        return Err(wrong_kind(value, path, "an array of objects"));
    };
    if elements.len() > MAXIMUM_ACTIONS {
        return Err(ConfigError::new(
            format!("Configuration key {path} lists {} actions; at most {MAXIMUM_ACTIONS} are accepted.", elements.len())
                .as_str(),
        ));
    }
    let mut actions: Vec<PostCommitAction> = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let action: PostCommitAction = parse_action(element, format!("{path}[{index}]").as_str())?;
        if actions.iter().any(|known| { return known.name == action.name; }) {
            return Err(ConfigError::new(
                format!("Configuration key {path}[{index}].name repeats the action name {:?}; names are unique.", action.name)
                    .as_str(),
            ));
        }
        actions.push(action);
    }
    return Ok(actions);
}

/// What:
///  Parse one action object.
/// Why:
///   `name` and `argv` are required; an empty `argv` has no program to run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseAction(value, 'postCommit.actions[0]');
/// ```
fn parse_action(value: &JsoncValue, path: &str) -> Result<PostCommitAction, ConfigError> {
    let Some(entries) = value.entries() else {
        return Err(wrong_kind(value, path, "an object with name and argv"));
    };
    let keys: Vec<String> = member_keys(entries, path)?;
    let mut declared_name: Option<String> = None;
    let mut declared_argv: Option<Vec<Template>> = None;
    let mut timeout_seconds: u64 = DEFAULT_ACTION_TIMEOUT_SECONDS;
    for (index, key) in keys.iter().enumerate() {
        let member: String = member_path(path, key.as_str());
        let member_value: &JsoncValue = &entries[index].value;
        match key.as_str() {
            "name" => {
                let given: String = text(member_value, member.as_str())?;
                if given.is_empty() {
                    return Err(ConfigError::new(format!("Configuration key {member} must not be empty.").as_str()));
                }
                declared_name = Some(given);
            }
            "argv" => declared_argv = Some(parse_argv(member_value, member.as_str())?),
            "timeoutSeconds" => timeout_seconds = safe_integer(member_value, member.as_str(), 1)?,
            _ => {
                return Err(ConfigError::new(
                    format!("Unknown configuration key: {member}. Accepted keys: name, argv, timeoutSeconds.").as_str(),
                ));
            }
        }
    }
    let Some(name) = declared_name else {
        return Err(ConfigError::new(format!("Configuration key {path} needs a name.").as_str()));
    };
    let Some(argv) = declared_argv else {
        return Err(ConfigError::new(format!("Configuration key {path} needs an argv array.").as_str()));
    };
    return Ok(PostCommitAction { name, argv, timeout_seconds });
}

/// What:
///  Parse the `argv` array of templates.
/// Why:
///   The first element is the program, so it must exist; each element is checked against the field set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseArgv(value, path); // Template[]
/// ```
fn parse_argv(value: &JsoncValue, path: &str) -> Result<Vec<Template>, ConfigError> {
    let Some(elements) = value.elements() else {
        return Err(wrong_kind(value, path, "an array of strings"));
    };
    if elements.is_empty() {
        return Err(ConfigError::new(
            format!("Configuration key {path} is empty; its first element is the program to run.").as_str(),
        ));
    }
    let mut templates: Vec<Template> = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let element_path: String = format!("{path}[{index}]");
        let source: String = text(element, element_path.as_str())?;
        let parsed: Template = parse_template(source.as_str()).map_err(|reason| {
            return ConfigError::new(
                format!(
                    "Configuration key {element_path} is not a valid argument: {reason}. The argument is a plain string \
                     (no shell runs it) and may use the fields {FIELD_LIST}."
                )
                .as_str(),
            );
        })?;
        templates.push(parsed);
    }
    return Ok(templates);
}

/// Parsing is controlled in the test module.
#[cfg(test)]
#[path = "config_post_commit_tests.rs"]
mod tests;
