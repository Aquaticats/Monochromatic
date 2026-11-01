//! What:
//!  The hook plan of one commit transaction:
//!  which commit events the user disabled, whether the configuration can define a hook at all, the hook lock,
//!  and the preparation lease.
//! Why:
//!  Git runs the configured hooks of the preparation events itself inside the native `git commit`, and the wrapper
//!  runs `post-commit` itself after landing, so the plan only carries what the wrapper must decide around those
//!  runs.
//!  `<tx>/hooks/plan.json` is the durable record of the lease: a nested wrapper invocation validates its inherited
//!  lease against it (`transaction_lease.rs`), and the tearing table pins its recovery.
//!  No plan names an executable that Git would start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan: HookDispatchPlan = await computeHookDispatchPlan({ cwd, commonDir, lease, ... });
//! ```

/// The one configuration read, and its lookup answers.
use super::config_snapshot::{ConfigSnapshot, Lookup};
/// Config reads and the per-transaction Git context.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// Typed writers over the plan's JSON object.
use super::json_record::{ObjectWriter, string_array};
/// Unguessable tokens for preparation leases.
use super::random_id::random_uuid;
/// Lease spelling shared with the transaction lease validator.
use super::transaction_lease::format_preparation_lease;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Component, Path, PathBuf};

/// Plan schema version written.
pub const PLAN_SCHEMA_VERSION: i64 = 1;

/// Name of the plan inside a transaction's `hooks` directory.
pub const PLAN_FILENAME: &str = "plan.json";

/// Events Git runs inside the native commit; `post-commit` runs once after landing instead.
pub const PREPARATION_HOOK_EVENTS: [&str; 3] = ["pre-commit", "prepare-commit-msg", "commit-msg"];

/// Every commit event whose user switch (`hook.<event>.enabled`) the plan records.
pub const SILENCED_HOOK_EVENTS: [&str; 4] =
    ["pre-commit", "prepare-commit-msg", "commit-msg", "post-commit"];

/// What:
///  Why planning or encoding a plan failed.
/// Why:
///   The caller stops the commit with the message;
///  a plan is never partially written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new TypeError(`git config --get ${key} failed: ${stderr.trim()}`);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HookPlanError {
    /// A `git config --get` read failed with a code other than 1; holds Git's stderr text.
    ConfigRead(String),
    /// A value the plan records is not UTF-8, so it cannot be written as JSON text.
    NotUtf8(&'static str),
    /// The random source for a preparation lease token failed.
    Random(String),
}

/// What:
///  One line for each planning failure.
/// Why:
///   The caller prints it as the diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for HookPlanError {
    /// Writes one sentence naming what failed.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigRead(stderr) => {
                return write!(formatter, "git config failed: {stderr}");
            }
            Self::NotUtf8(what) => {
                return write!(formatter, "the {what} is not UTF-8 and cannot be recorded");
            }
            Self::Random(message) => {
                return write!(formatter, "no random token for the preparation lease: {message}");
            }
        }
    }
}

/// What:
///  The plan written to `<transaction>/hooks/plan.json`.
/// Why:
///   The wrapper decides from it whether the native commit and the `post-commit` run need the hook lock, and the
///  lease validator reads the lease back from the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HookDispatchPlan = { schemaVersion: 1; disabledEvents: string[]; lease: string; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookDispatchPlan {
    /// Events the user disabled with `hook.<event>.enabled=false`, in the silenced list's order.
    pub disabled_events: Vec<String>,
    /// Preparation lease exported to the native commit and the hooks it runs.
    pub lease: String,
    /// Absolute hook lock directory.
    pub hook_lock_path: String,
    /// Whether hook runs skip the hook lock.
    pub skip_hook_lock: bool,
    /// Whether the configuration may define hooks by name (`hook.<name>.event` or `.command`), which run for their
    ///  event whatever the hooks directory holds; `true` when the configuration was not inspected.
    ///  Not recorded in `plan.json`: nothing reads a plan back but the lease check.
    pub named_hooks_possible: bool,
}

/// What:
///  The inputs of one plan computation.
/// Why:
///   Named fields keep the inputs apart; the context carries the caller's global options
///  as its global prefix, so every config read sees them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// computeHookDispatchPlan({ cwd, commonDir, lease, ... });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct HookPlanInputs<'a> {
    /// Real Git and the caller's global options.
    pub context: &'a GitContext,
    /// Owning worktree directory where the config reads run.
    pub cwd: &'a Path,
    /// Absolute common Git directory.
    pub common_directory: &'a Path,
    /// Preparation lease token text, formatted by `new_preparation_lease`.
    pub lease: &'a str,
    /// `hooks.concurrentCommits`.
    pub concurrent_commits: bool,
    /// Whether this invocation runs under a valid outer preparation lease.
    pub inherited_lease_valid: bool,
    /// The configuration read once with the caller's global options;
    ///  `None` makes each key start its own `git config`.
    pub snapshot: Option<&'a ConfigSnapshot>,
}

/// What:
///  Normalize a path lexically, the way Node's `path.join` and `path.resolve` do.
/// Why:
///   Recorded paths must match the incumbent's spelling byte for byte,
///  and `PathBuf::join` keeps `.` and `..`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.resolve(path);
/// ```
pub fn normalized_path(path: &Path) -> PathBuf {
    // `mut` lets the result grow and shrink as components arrive.
    let mut result: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                result.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                // A `..` above the root stays at the root, as Node clamps it.
                let at_root: bool = result.parent().is_none();
                if !at_root && result.file_name().is_some() {
                    result.pop();
                }
            }
            Component::Normal(name) => {
                result.push(name);
            }
        }
    }
    return result;
}

/// What:
///  The hook lock directory under the common Git directory.
/// Why:
///   The incumbent's `join(commonDir, 'cli-git', 'hook.lock')`; the same lock serializes both wrappers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'cli-git', 'hook.lock')
/// ```
pub fn hook_lock_directory(common_directory: &Path) -> PathBuf {
    return normalized_path(common_directory.join("cli-git").join("hook.lock").as_path());
}

/// What:
///  The silenced events the user disabled with `hook.<event>.enabled`,
///  in the silenced list's order,
///  from one Git process.
/// Why:
///   The incumbent reads each event's switch with its own `git config --get`;
///  one `--get-regexp` over the same four keys answers all of them,
///  and `--type=bool` canonicalizes every spelling (`False`, `no`, `0`, a valueless key) per line.
///  Git prints repeated keys in precedence order with `-c` values last,
///  so the last value of a key is the one `--get` returns.
///  Exit code 1 means no event has a switch;
///  any other failure stops planning, as it does for a single read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = (await git(['config', '--type=bool', '--get-regexp', pattern])).split('\n');
/// SILENCED_HOOK_EVENTS.filter((event) => lastValue(lines, `hook.${event}.enabled`) === 'false');
/// ```
fn read_disabled_events(context: &GitContext, cwd: &Path) -> Result<Vec<String>, HookPlanError> {
    let pattern: String = format!("^hook\\.({})\\.enabled$", SILENCED_HOOK_EVENTS.join("|"));
    let arguments: Vec<String> = vec![
        String::from("config"),
        String::from("--type=bool"),
        String::from("--get-regexp"),
        pattern.clone(),
    ];
    let request: GitRequest = GitRequest::new(cwd, arguments.as_slice());
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(HookPlanError::ConfigRead(format!(
                "git config --get-regexp {pattern} could not start: {error}"
            )));
        }
    };
    if output.code == Some(1) {
        return Ok(Vec::new());
    }
    if !output.succeeded() {
        return Err(HookPlanError::ConfigRead(format!(
            "git config --get-regexp {pattern} failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )));
    }
    let Ok(text) = String::from_utf8(output.stdout) else {
        return Err(HookPlanError::ConfigRead(format!(
            "git config --get-regexp {pattern} printed bytes that are not UTF-8"
        )));
    };
    // Each line is `<key> <canonical bool>`; a later line replaces an earlier one for the same key.
    let mut switches: Vec<(&str, &str)> = Vec::new();
    for line in text.lines() {
        if let Some((key, value)) = line.rsplit_once(' ') {
            switches.retain(|(known, _)| return *known != key);
            switches.push((key, value));
        }
    }
    let mut disabled: Vec<String> = Vec::new();
    for event in SILENCED_HOOK_EVENTS {
        let key: String = format!("hook.{event}.enabled");
        let enabled: Option<&str> = switches
            .iter()
            .find(|(known, _)| return *known == key.as_str())
            .map(|(_, value)| return *value);
        if enabled == Some("false") {
            disabled.push(String::from(event));
        }
    }
    return Ok(disabled);
}

/// What:
///  The silenced events the user disabled with `hook.<event>.enabled`, from the configuration snapshot.
/// Why:
///   The snapshot answers exactly what the `--get-regexp` read answers when every entry of the four switches is a
///  boolean it models; one it does not model (a bad boolean, a unit suffix) is `None`,
///  and the caller starts Git, which reports the same error it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SILENCED_HOOK_EVENTS.filter((event) => snapshot.boolValue(`hook.${event}.enabled`) === false);
/// ```
fn snapshot_disabled_events(snapshot: &ConfigSnapshot) -> Option<Vec<String>> {
    let mut disabled: Vec<String> = Vec::new();
    for event in SILENCED_HOOK_EVENTS {
        match snapshot.bool_value(format!("hook.{event}.enabled").as_str()) {
            Lookup::Unset | Lookup::Found(true) => {}
            Lookup::Found(false) => disabled.push(String::from(event)),
            Lookup::Unhandled => return None,
        }
    }
    return Some(disabled);
}

/// What:
///  Compute the hook plan of one transaction from the real repository.
/// Why:
///   The event switches (only the exact value `false` disables an event),
///  whether the configuration can define a hook by name, the lock path and whether the lock is skipped;
///  traditional hooks are unsupported, so no hooks path is resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function computeHookDispatchPlan(inputs): Promise<HookDispatchPlan>;
/// ```
pub fn compute_hook_dispatch_plan(
    inputs: &HookPlanInputs<'_>,
) -> Result<HookDispatchPlan, HookPlanError> {
    let disabled_events: Vec<String> = match inputs.snapshot.and_then(snapshot_disabled_events) {
        Some(events) => events,
        None => read_disabled_events(inputs.context, inputs.cwd)?,
    };
    let plan: HookDispatchPlan = HookDispatchPlan {
        disabled_events,
        lease: String::from(inputs.lease),
        hook_lock_path: path_text(
            hook_lock_directory(inputs.common_directory).as_path(),
            "hook lock path",
        )?,
        skip_hook_lock: inputs.concurrent_commits || inputs.inherited_lease_valid,
        named_hooks_possible: inputs.snapshot.is_none_or(ConfigSnapshot::defines_named_hook),
    };
    return Ok(plan);
}

/// What:
///  A path as UTF-8 text, or the refusal naming what it was.
/// Why:
///   The plan's JSON strings hold text; lossy conversion would name another path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // paths are strings in TypeScript
/// ```
fn path_text(path: &Path, what: &'static str) -> Result<String, HookPlanError> {
    let Some(text) = path.to_str() else {
        return Err(HookPlanError::NotUtf8(what));
    };
    return Ok(String::from(text));
}

/// What:
///  Spell a plan as `JSON.stringify` writes it, without the trailing newline.
/// Why:
///   The lease validator reads the `lease` field back from these bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(plan)
/// ```
pub fn encode_hook_dispatch_plan(plan: &HookDispatchPlan) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer.integer("schemaVersion", PLAN_SCHEMA_VERSION);
    writer.raw(
        "disabledEvents",
        string_array(plan.disabled_events.as_slice()).as_str(),
    );
    writer.string("lease", plan.lease.as_str());
    writer.string("hookLockPath", plan.hook_lock_path.as_str());
    writer.boolean("skipHookLock", plan.skip_hook_lock);
    return writer.finish();
}

/// What:
///  A fresh preparation lease for one transaction directory.
/// Why:
///   The incumbent's `formatPreparationLease({ directory, token })` with an unguessable token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// formatPreparationLease({ directory, token: randomUUID() });
/// ```
pub fn new_preparation_lease(transaction_directory: &Path) -> Result<String, HookPlanError> {
    let directory: String = path_text(transaction_directory, "transaction directory")?;
    let token: String = random_uuid().map_err(|error| { return HookPlanError::Random(format!("{error:?}")); })?;
    return Ok(format_preparation_lease(directory.as_str(), token.as_str()));
}

#[cfg(test)]
#[path = "hook_dispatch_plan_tests.rs"]
mod tests;
