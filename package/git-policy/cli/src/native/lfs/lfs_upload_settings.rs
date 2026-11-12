//! What:
//!  The configuration the Git LFS upload plan reads,
//! and the answers derived from it:
//! the default remote,
//! the branch
//!  `HEAD` names,
//! and whether a value reads as true.
//! Why:
//!  The plan and the manual-push probe must agree on the destination of a bare push,
//! so both read it here.
//! The entries
//!  come from a snapshot the caller already holds or from one `config --get-regexp`,
//! never one start per key.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settings = readSettings(query); const remote = defaultRemote(branch, settings);
//! ```

/// The configuration snapshot the caller may already hold.
use crate::config_snapshot::ConfigSnapshot;
/// Owned OS text for arguments.
use std::ffi::OsString;

/// What:
///  A read-only Git query:
/// its trimmed standard output when it finished (status zero,
/// or status one with no output,
///  which is how `config --get-regexp` and `symbolic-ref --quiet` say "nothing"),
/// or what went wrong.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The plan asks Git questions through this one method.
/// Each caller that starts Git is a small struct with the state
///  its questions need (the real Git,
/// the prefix,
/// the overlay),
/// and a test can still pass a closure,
/// which the
///  blanket `impl` below turns into a query,
/// so no closure is written in shipped code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface GitQuery { ask(args: string[]): string; } // throws string
/// ```
pub trait GitQuery {
    /// Ask Git one question and read its answer.
    fn ask(&mut self, arguments: &[&str]) -> Result<String, String>;
}

/// What:
///  Every function-like value with the right shape is a query:
/// `ask` just calls it.
///  `impl<Answer: FnMut(..)> GitQuery for Answer` reads "for every type `Answer` that can be called like this".
/// Why:
///   The tests of the plan stub Git with closures,
/// and they keep doing so without a struct per stub.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asQuery = (answer: (args: string[]) => string): GitQuery => ({ ask: answer });
/// ```
impl<Answer: FnMut(&[&str]) -> Result<String, String>> GitQuery for Answer {
    fn ask(&mut self, arguments: &[&str]) -> Result<String, String> {
        return self(arguments);
    }
}

/// A borrowed query that the plan functions share.
pub type Query<'a> = &'a mut dyn GitQuery;

/// What:
///  A read-only Git query awaited by a caller that is itself a task of the runtime,
/// with the answer the synchronous `GitQuery` gives.
/// Why:
///   The landing round's push runs as a task,
/// and the synchronous queries enter the runtime and are refused from there;
/// a trait with an `async` method cannot become a `dyn` value,
/// so the question is
///  handed back as a boxed future,
/// as the ordered runner's awaited work is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface GitQueryAwaiting { ask(args: string[]): Promise<string>; } // rejects with string
/// ```
pub trait GitQueryAwaiting {
    /// Ask Git one question and await its answer.
    fn ask<'a>(&'a mut self, arguments: &'a [&'a str]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>>;
}

/// A borrowed awaited query that the plan functions share.
pub type QueryAwaiting<'a> = &'a mut (dyn GitQueryAwaiting + Send);

/// Configuration entries the plan reads:
/// `(key, value)` with Git's normalised key.
pub type Settings = Vec<(String, String)>;

/// Why the plan could not be worked out;
/// the push is refused with this text rather than run without its upload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlanError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanError(pub String);

/// The `config --get-regexp` pattern that selects exactly the keys `wanted_key` accepts.
pub const SETTINGS_REGEXP: &str =
    "^(branch\\..*\\.(pushremote|remote)|remote\\.pushdefault|remote\\..*\\.(push|pushurl|mirror|url)|push\\.default|url\\..*\\.pushinsteadof)$";

/// What:
///  The last dotted word of a key after a prefix:
/// `remote.origin.url` after `remote.` gives `url`.
///  `Option<&str>` is "a borrowed piece of the key,
/// or no answer".
/// Why:
///   `wanted_key` asks it for the `remote.` and `branch.` families,
/// and a function stands where nested closures stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastVariable = (key: string, prefix: string) => key.startsWith(prefix) ? key.slice(prefix.length).split('.').at(-1) : undefined;
/// ```
fn last_variable<'a>(key: &'a str, prefix: &str) -> Option<&'a str> {
    let rest: &str = key.strip_prefix(prefix)?;
    let (_name, variable) = rest.rsplit_once('.')?;
    return Some(variable);
}

/// Whether the plan reads this configuration key (Git's normalised spelling).
pub(crate) fn wanted_key(key: &str) -> bool {
    let remote_variable: bool = match last_variable(key, "remote.") {
        Some(variable) => ["push", "pushurl", "mirror", "url"].contains(&variable),
        None => false,
    };
    let branch_variable: bool = match last_variable(key, "branch.") {
        Some(variable) => variable == "pushremote" || variable == "remote",
        None => false,
    };
    let push_rewrite: bool = match key.strip_prefix("url.") {
        Some(rest) => rest.ends_with(".pushinsteadof"),
        None => false,
    };
    return branch_variable
        || push_rewrite
        || key == "remote.pushdefault"
        || key == "push.default"
        || remote_variable;
}

/// The settings the plan reads,
/// out of a snapshot the caller already holds,
/// so no Git start is needed for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.filter(wantedKey).map(({ key, value }) => [key, value])
/// ```
pub fn settings_from_snapshot(snapshot: &ConfigSnapshot) -> Settings {
    let mut settings: Settings = Vec::new();
    for entry in snapshot.entries() {
        if !wanted_key(entry.key.as_str()) {
            continue;
        }
        // A key written without a value reads as the empty text.
        let value: String = match entry.value.as_deref() {
            Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            None => String::new(),
        };
        settings.push((entry.key.clone(), value));
    }
    return settings;
}

/// The settings the plan reads,
/// from one `config --get-regexp` start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['config', '--get-regexp', SETTINGS_REGEXP])
/// ```
pub fn read_settings(query: Query<'_>) -> Result<Settings, PlanError> {
    let listing: String = match query.ask(&["config", "--get-regexp", SETTINGS_REGEXP]) {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git config --get-regexp: {why}"))),
    };
    return Ok(settings_of(listing.as_str()));
}

/// What:
///  The same read,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push plans its upload as a task,
/// and the synchronous read enters the runtime and is refused from there.
/// The question and the reading of its answer are the synchronous face's,
/// so the two cannot plan from different settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['config', '--get-regexp', SETTINGS_REGEXP])
/// ```
pub async fn read_settings_awaiting(query: QueryAwaiting<'_>) -> Result<Settings, PlanError> {
    let listing: String = match query.ask(&["config", "--get-regexp", SETTINGS_REGEXP]).await {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git config --get-regexp: {why}"))),
    };
    return Ok(settings_of(listing.as_str()));
}

/// What:
///  The settings one finished listing answers.
/// Why:
///   Both faces of the read parse the same listing here,
/// so a key either face reads is read the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\n').map((line) => line.split(/ (.*)/))
/// ```
fn settings_of(listing: &str) -> Settings {
    let mut settings: Settings = Vec::new();
    for line in listing.lines() {
        // A key written without a value is listed alone.
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        settings.push((key.to_owned(), value.trim().to_owned()));
    }
    return settings;
}

/// The last value of a setting,
/// the last one written winning as in Git.
pub(crate) fn setting<'a>(all: &'a [(String, String)], key: &str) -> Option<&'a str> {
    for (name, value) in all.iter().rev() {
        if name == key {
            return Some(value.as_str());
        }
    }
    return None;
}

/// Every value of a setting,
/// in the order written.
pub(crate) fn values<'a>(all: &'a [(String, String)], key: &str) -> Vec<&'a str> {
    let mut found: Vec<&'a str> = Vec::new();
    for (name, value) in all {
        if name == key {
            found.push(value.as_str());
        }
    }
    return found;
}

/// The branch `HEAD` names,
/// from the full ref so that a tag with the same name cannot change the answer.
pub(crate) fn current_branch(query: Query<'_>) -> Result<Option<String>, PlanError> {
    let full: String = match query.ask(&["symbolic-ref", "--quiet", "HEAD"]) {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git symbolic-ref HEAD: {why}"))),
    };
    return Ok(branch_of(full.as_str()));
}

/// What:
///  The same answer,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push reads the branch `HEAD` names as a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['symbolic-ref', '--quiet', 'HEAD'])
/// ```
pub(crate) async fn current_branch_awaiting(query: QueryAwaiting<'_>) -> Result<Option<String>, PlanError> {
    let full: String = match query.ask(&["symbolic-ref", "--quiet", "HEAD"]).await {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git symbolic-ref HEAD: {why}"))),
    };
    return Ok(branch_of(full.as_str()));
}

/// What:
///  The branch a full reference names,
/// or nothing for a `HEAD` that names no branch.
/// Why:
///   Both faces of `current_branch` read one answer here,
/// so a tag of the same name cannot change either answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// full.startsWith('refs/heads/') ? full.slice('refs/heads/'.length) : undefined;
/// ```
fn branch_of(full: &str) -> Option<String> {
    return full.strip_prefix("refs/heads/").map(str::to_owned);
}

/// The value of one `branch.<name>.<variable>` setting of the current branch;
/// nothing when there is no branch.
fn branch_setting(branch: Option<&str>, all: &[(String, String)], variable: &str) -> Option<String> {
    let name: &str = branch?;
    return setting(all, &format!("branch.{name}.{variable}")).map(str::to_owned);
}

/// The remote Git pushes to when the caller names none:
/// the branch's push remote,
/// the default push remote,
/// the
///  branch's remote,
/// then `origin`.
/// Each later source is read only when the earlier ones gave nothing.
pub(crate) fn default_remote(branch: Option<&str>, all: &[(String, String)]) -> String {
    if let Some(found) = branch_setting(branch, all, "pushremote") {
        return found;
    }
    if let Some(found) = setting(all, "remote.pushdefault") {
        return found.to_owned();
    }
    if let Some(found) = branch_setting(branch, all, "remote") {
        return found;
    }
    return String::from("origin");
}

/// What:
///  The remote a push goes to:
/// the one the caller named,
/// or the one Git picks for a bare push.
/// Why:
///   The manual-push probe and the upload plan must agree on the destination,
/// so both read it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named ?? defaultRemote(branch, settings)
/// ```
pub fn resolve_push_remote(named: Option<OsString>, query: Query<'_>) -> Result<OsString, PlanError> {
    if let Some(found) = named {
        return Ok(found);
    }
    let branch: Option<String> = current_branch(query)?;
    let all: Settings = read_settings(query)?;
    return Ok(OsString::from(default_remote(branch.as_deref(), all.as_slice())));
}

/// What:
///  Whether Git reads the configuration key `key` as true,
/// asked of Git rather than of a list of spellings kept
///  here.
/// Why:
///   Git's boolean grammar is Git's:
/// `2`, `-1` and `0x10` are true,
/// an empty value is false,
/// `TRUE` is true,
///  and a value it cannot read stops the push it belongs to.
/// A list here disagreed with all four of those, and a
///  boolean read wrongly is a push that publishes what it did not upload.
/// The key is asked only when the settings hold a
///  value for it,
/// so a push in a repository without it costs no Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setting(all, key) === undefined ? false : query(['config', '--type=bool', '--get', key]) === 'true'
/// ```
pub(crate) fn git_says_true(all: &[(String, String)], key: &str, query: Query<'_>) -> Result<bool, PlanError> {
    if setting(all, key).is_none() {
        return Ok(false);
    }
    return boolean_of(key, query.ask(&["config", "--type=bool", "--get", key]));
}

/// What:
///  The same answer,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round of the service runs as tasks,
/// and a question asked on the synchronous face is refused from
///  there;
/// the question and the reading of its answer are the synchronous face's,
/// so the two cannot
///  disagree about a boolean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await gitSaysTrue(all, key, query)
/// ```
pub(crate) async fn git_says_true_awaiting(all: &[(String, String)], key: &str, query: QueryAwaiting<'_>) -> Result<bool, PlanError> {
    if setting(all, key).is_none() {
        return Ok(false);
    }
    return boolean_of(key, query.ask(&["config", "--type=bool", "--get", key]).await);
}

/// What:
///  The boolean one finished answer states,
/// or the refusal that ends the push.
/// Why:
///   `git config --type=bool --get` prints `true` or `false` and nothing else;
/// any other answer and any
///  failure mean Git could not read the value,
/// and a push Git would refuse must not be planned as one that
///  skips its objects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answer === 'true' ? true : answer === 'false' || answer === '' ? false : throw new PlanError(key)
/// ```
fn boolean_of(key: &str, answer: Result<String, String>) -> Result<bool, PlanError> {
    return match answer {
        Ok(text) if text == "true" => Ok(true),
        Ok(text) if text == "false" || text.is_empty() => Ok(false),
        Ok(text) => Err(PlanError(format!("git config --type=bool {key}: Git answered {text:?}"))),
        Err(why) => Err(PlanError(format!("git config --type=bool {key}: {why}"))),
    };
}
