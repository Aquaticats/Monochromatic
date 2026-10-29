//! Parse the notification control verbs.
//!
//! `notifications`,
//!  `notification-log [N]`,
//!  `notification-clear`,
//!  `notification-close ID [REASON]`,
//!  and `notification-action ID KEY`.

/// Imports `no_arguments`,
///  `Command` from `super`.
use super::{no_arguments, Command};

/// The number of records `notification-log` and `job-log` list when no count is given.
const DEFAULT_LOG_COUNT: usize = 64;

/// A parsed notification verb.
///
/// What:
///      `pub enum NotificationCommand { List, Log, Clear, Close, Action }`.
/// Why:
///       The executor matches on one typed value,
///  and the parser never touches the bus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationCommand {
    /// List the notifications currently visible.
    List,
    /// List the newest log records,
    ///  oldest first.
    Log {
        /// How many of the newest records to list.
        count: usize,
    },
    /// Forget the visible notifications and the log.
    Clear,
    /// Close a visible notification and emit `NotificationClosed`.
    Close {
        /// The notification id.
        id: u32,
        /// The reason code in the signal.
        reason: u32,
    },
    /// Emit `ActionInvoked` for a visible notification.
    Action {
        /// The notification id.
        id: u32,
        /// The action key,
        ///  verbatim.
        key: String,
    },
}

/// Parse an optional record count:
///  a whole number of at least one,
///  default 64.
///
/// What:
///      `pub(super) fn parse_count(tokens, verb) -> Result<usize, String>`.
///  Rejects a second argument.
/// Why:
///       A count of zero would answer with no records and hide a mistyped argument.
pub(super) fn parse_count(tokens: &mut std::str::SplitWhitespace, verb: &str) -> Result<usize, String> {
    let count = match tokens.next() {
        Some(text) => match text.parse::<usize>() {
            Ok(number) => number,
            Err(_) => return Err(format!("{verb} count is not a whole number")),
        },
        None => DEFAULT_LOG_COUNT,
    };
    if count == 0 {
        return Err(format!("{verb} count must be at least 1"));
    }
    if tokens.next().is_some() {
        return Err(format!("{verb} takes at most one count"));
    }
    return Ok(count);
}

/// Parse a notification or job id:
///  a whole number of at least one.
pub(super) fn parse_id(text: &str, verb: &str) -> Result<u32, String> {
    let Ok(id) = text.parse::<u32>() else {
        return Err(format!("{verb} id is not a whole number: {text}"));
    };
    if id == 0 {
        return Err(format!("{verb} id must be at least 1"));
    }
    return Ok(id);
}

/// Parse the reason of `notification-close`:
///  a code from 1 to 4 or its name.
///
/// What:
///      `fn parse_reason(text: &str) -> Result<u32, String>`.
///  `expired` is 1,
///  `dismissed` is 2,
///  `closed` is 3,
///  `undefined` is 4.
/// Why:
///       These are the codes the notification specification defines for `NotificationClosed`.
fn parse_reason(text: &str) -> Result<u32, String> {
    return match text {
        "1" | "expired" => Ok(1),
        "2" | "dismissed" => Ok(2),
        "3" | "closed" => Ok(3),
        "4" | "undefined" => Ok(4),
        other => Err(format!(
            "notification-close reason must be 1 to 4 or expired, dismissed, closed, undefined, got: {other}"
        )),
    };
}

/// Parse the verbs that take whitespace-separated arguments.
///
/// What:
///      `pub(super) fn parse(verb, tokens) -> Result<Command, String>`.
///  `notification-action` is parsed by `parse_action`,
///  which keeps the key verbatim.
pub(super) fn parse(verb: &str, tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    match verb {
        "notifications" => return no_arguments(tokens, verb, Command::Notification(NotificationCommand::List)),
        "notification-clear" => {
            return no_arguments(tokens, verb, Command::Notification(NotificationCommand::Clear));
        }
        "notification-log" => {
            let count = parse_count(tokens, verb)?;
            return Ok(Command::Notification(NotificationCommand::Log { count }));
        }
        "notification-close" => {
            let Some(id_text) = tokens.next() else {
                return Err("notification-close requires an id and an optional reason".to_string());
            };
            let id = parse_id(id_text, verb)?;
            let reason = match tokens.next() {
                Some(text) => parse_reason(text)?,
                None => 2,
            };
            if tokens.next().is_some() {
                return Err("notification-close takes an id and an optional reason".to_string());
            }
            return Ok(Command::Notification(NotificationCommand::Close { id, reason }));
        }
        other => return Err(format!("unknown command: {other}")),
    }
}

/// Parse the text after `notification-action `:
///  an id,
///  then the action key as the rest of the line.
///
/// What:
///      `pub(super) fn parse_action(rest: &str) -> Result<Command, String>`.
///  The key keeps its spaces;
///  only the separator run after the id is dropped.
/// Why:
///       Action keys are chosen by the app,
///  and Plasma's own examples include `Action 1`.
pub(super) fn parse_action(rest: &str) -> Result<Command, String> {
    let trimmed = rest.trim_start();
    let (id_text, key_text) = match trimmed.split_once(char::is_whitespace) {
        Some((id_text, key_text)) => (id_text, key_text.trim_start()),
        None => (trimmed, ""),
    };
    if id_text.is_empty() || key_text.is_empty() {
        return Err("notification-action requires an id and an action key".to_string());
    }
    let id = parse_id(id_text, "notification-action")?;
    return Ok(Command::Notification(NotificationCommand::Action { id, key: key_text.to_string() }));
}
