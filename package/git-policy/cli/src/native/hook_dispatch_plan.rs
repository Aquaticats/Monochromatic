//! What:
//!  The dispatch plan of one commit transaction:
//!  which hooks run, how they run, and under which lock.
//! Why:
//!  Inside native `git commit` the hook entries see only the transaction's overrides,
//!  so the plan is computed by the wrapper from the real repository and the caller's environment.
//!  The incumbent (`src/hook-dispatch/hook-dispatch-plan.ts`) and both dispatchers read the same
//!  `plan.json`, so its field order and spelling are part of the on-disk format.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan: HookDispatchPlan = await computeHookDispatchPlan({ gitPath, cwd, globalArgs, commonDir, ... });
//! ```

/// The one configuration read, and its lookup answers.
use super::config_snapshot::{ConfigSnapshot, Lookup};
/// Config reads and the per-transaction Git context.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// Lenient typed readers over the plan's JSON object.
use super::json_record::{ObjectWriter, parse_object, safe_integer_field, string_array, string_field};
/// Unguessable tokens for preparation leases.
use super::random_id::random_uuid;
/// Lease spelling shared with the transaction lease validator.
use super::transaction_lease::format_preparation_lease;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Component, Path, PathBuf};

/// Plan schema version written and required on read.
pub const PLAN_SCHEMA_VERSION: i64 = 1;

/// Name of the plan inside a transaction's `hooks` directory.
pub const PLAN_FILENAME: &str = "plan.json";

/// Events the shim dispatches during preparation; `post-commit` runs once after landing instead.
pub const PREPARATION_HOOK_EVENTS: [&str; 3] = ["pre-commit", "prepare-commit-msg", "commit-msg"];

/// Every commit event the transaction silences inside native `git commit`.
pub const SILENCED_HOOK_EVENTS: [&str; 4] =
    ["pre-commit", "prepare-commit-msg", "commit-msg", "post-commit"];

/// Environment variable that carries Git's config parameters into hooks.
pub const CONFIG_PARAMETERS_VARIABLE: &str = "GIT_CONFIG_PARAMETERS";

/// Environment variable naming the worktree root for hooks.
pub const WORK_TREE_VARIABLE: &str = "GIT_WORK_TREE";

/// Largest plan a dispatcher reads.
pub const PLAN_LIMIT: u64 = 1024 * 1024;

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
///  The value the dispatcher restores as `GIT_CONFIG_PARAMETERS`, or its absence.
/// Why:
///   Git reads the variable as text; an absent variable must stay absent in hooks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigParameters = { kind: 'absent' } | { kind: 'present'; value: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigParameters {
    /// No config parameters were inherited or given.
    Absent,
    /// The combined parameter text.
    Present(String),
}

/// What:
///  The plan written to `<transaction>/hooks/plan.json`.
/// Why:
///   Both dispatchers and the lease validator read this one record,
///  so every field keeps the incumbent's name and meaning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HookDispatchPlan = { schemaVersion: 1; gitPath: string; hooksPath: string; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookDispatchPlan {
    /// Absolute real Git executable the dispatcher runs `git hook run` with.
    pub git_path: String,
    /// Repository's own absolute hooks directory.
    pub hooks_path: String,
    /// Events the user disabled with `hook.<event>.enabled=false`, in the silenced list's order.
    pub disabled_events: Vec<String>,
    /// Caller's config parameters including its global `-c` options.
    pub config_parameters: ConfigParameters,
    /// Absolute worktree root exported as `GIT_WORK_TREE`.
    pub worktree_root: String,
    /// Preparation lease exported to hook descendants.
    pub lease: String,
    /// Absolute hook lock directory.
    pub hook_lock_path: String,
    /// Whether hooks skip the hook lock.
    pub skip_hook_lock: bool,
    /// Whether the configuration may define hooks by name (`hook.<name>.event` or `.command`), which run for their
    ///  event whatever the hooks directory holds; `true` when the configuration was not inspected.
    ///  Not recorded in `plan.json`: the dispatchers never read it, and a plan read back says `true`.
    pub named_hooks_possible: bool,
}

/// What:
///  The inputs of one plan computation.
/// Why:
///   Named fields keep the many inputs apart; the context carries the caller's global options
///  as its global prefix, so every config read sees them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// computeHookDispatchPlan({ gitPath, cwd, globalArgs, commonDir, worktreeRoot, lease, ... });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct HookPlanInputs<'a> {
    /// Real Git and the caller's global options.
    pub context: &'a GitContext,
    /// Owning worktree directory where the config reads run.
    pub cwd: &'a Path,
    /// Caller's global options, whose `-c` values join the config parameters.
    pub global_arguments: &'a [OsString],
    /// Absolute common Git directory.
    pub common_directory: &'a Path,
    /// Absolute worktree root.
    pub worktree_root: &'a Path,
    /// Preparation lease token text, formatted by `new_preparation_lease`.
    pub lease: &'a str,
    /// `hooks.concurrentCommits`.
    pub concurrent_commits: bool,
    /// Whether this invocation runs under a valid outer preparation lease.
    pub inherited_lease_valid: bool,
    /// Caller environment, whose `GIT_CONFIG_PARAMETERS` is inherited.
    pub environment: &'a [(OsString, OsString)],
    /// The configuration read once with the caller's global options;
    ///  `None` makes each key start its own `git config`.
    pub snapshot: Option<&'a ConfigSnapshot>,
}

/// What:
///  Quote one `key=value` entry for Git's `GIT_CONFIG_PARAMETERS`.
/// Why:
///   Git's `sq_quote_buf` closes the quote around `'` and `!`;
///  the parser accepts that form in every supported version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// quoteConfigParameter("user.name=O'Brien"); // "'user.name=O'\\''Brien'"
/// ```
pub fn quote_config_parameter(value: &str) -> String {
    // `mut` lets the quoted text grow one character at a time.
    let mut body: String = String::with_capacity(value.len() + 2);
    body.push('\'');
    for character in value.chars() {
        match character {
            '\'' => body.push_str("'\\''"),
            '!' => body.push_str("'\\!'"),
            other => body.push(other),
        }
    }
    body.push('\'');
    return body;
}

/// What:
///  Collect the value of every global `-c` option, in order.
/// Why:
///   The incumbent pairs each `-c` with the token after it, even when that token is itself `-c`.
///  A non-UTF-8 value cannot be recorded, so it is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// globalArgs.flatMap((token, i) => (token === '-c') && globalArgs[i + 1] !== undefined ? [globalArgs[i + 1]] : []);
/// ```
pub fn global_config_overrides(global_arguments: &[OsString]) -> Result<Vec<String>, HookPlanError> {
    // `mut` lets the list grow one value at a time.
    let mut values: Vec<String> = Vec::new();
    for (index, token) in global_arguments.iter().enumerate() {
        let Some(next) = global_arguments.get(index + 1) else {
            continue;
        };
        if token != "-c" {
            continue;
        }
        let Some(text) = next.to_str() else {
            return Err(HookPlanError::NotUtf8("global -c option"));
        };
        values.push(String::from(text));
    }
    return Ok(values);
}

/// What:
///  Read the caller's inherited `GIT_CONFIG_PARAMETERS`, last entry winning.
/// Why:
///   An empty value counts as none, as the incumbent's `?? ''` and the joining rule do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// environment.GIT_CONFIG_PARAMETERS ?? ''
/// ```
pub fn inherited_config_parameters(
    environment: &[(OsString, OsString)],
) -> Result<String, HookPlanError> {
    // `Option` starts absent; each matching entry replaces the earlier one.
    let mut found: Option<&OsString> = None;
    for (name, value) in environment {
        if name == CONFIG_PARAMETERS_VARIABLE {
            found = Some(value);
        }
    }
    let Some(value) = found else {
        return Ok(String::new());
    };
    let Some(text) = value.to_str() else {
        return Err(HookPlanError::NotUtf8("inherited GIT_CONFIG_PARAMETERS"));
    };
    return Ok(String::from(text));
}

/// What:
///  Join the inherited parameters and the quoted global `-c` values.
/// Why:
///   Git hands hooks the inherited entries first, then the command-line options;
///  no entry means the variable is absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// combineConfigParameters({ inherited, overrides });
/// ```
pub fn combine_config_parameters(
    inherited: &str,
    overrides: &[String],
) -> Result<ConfigParameters, HookPlanError> {
    // `mut` lets the entry list grow in order.
    let mut entries: Vec<String> = Vec::new();
    if !inherited.is_empty() {
        entries.push(String::from(inherited));
    }
    for value in overrides {
        entries.push(quote_config_parameter(value.as_str()));
    }
    if entries.is_empty() {
        return Ok(ConfigParameters::Absent);
    }
    return Ok(ConfigParameters::Present(entries.join(" ")));
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
///  The hooks directory Git uses when `core.hooksPath` is unset.
/// Why:
///   The incumbent's `join(commonDir, 'hooks')`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'hooks')
/// ```
pub fn default_hooks_directory(common_directory: &Path) -> PathBuf {
    return normalized_path(common_directory.join("hooks").as_path());
}

/// What:
///  The hooks directory a configured `core.hooksPath` names.
/// Why:
///   A relative value resolves against the worktree root,
///  where Git runs hooks; absolute values stay absolute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAbsolute(configured) ? configured : resolve(worktreeRoot, configured);
/// ```
pub fn resolve_hooks_path(worktree_root: &Path, configured: &Path) -> PathBuf {
    if configured.is_absolute() {
        return normalized_path(configured);
    }
    return normalized_path(worktree_root.join(configured).as_path());
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
///  Read one config value through the caller's Git context; exit code 1 means unset.
/// Why:
///   The incumbent's `readConfig`: unset is not an error, any other failure stops planning,
///  and one terminating newline is removed from the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readConfig({ gitPath, cwd, globalArgs, type, key }): Promise<string | typeof CONFIG_UNSET>;
/// ```
fn read_config(
    context: &GitContext,
    cwd: &Path,
    kind: &str,
    key: &str,
) -> Result<Option<String>, HookPlanError> {
    // The context's global prefix precedes `config`, as the incumbent's `[...globalArgs, 'config']`.
    let arguments: Vec<String> = vec![
        String::from("config"),
        format!("--type={kind}"),
        String::from("--get"),
        String::from(key),
    ];
    let request: GitRequest = GitRequest::new(cwd, arguments.as_slice());
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(HookPlanError::ConfigRead(format!(
                "git config --get {key} could not start: {error}"
            )));
        }
    };
    if output.code == Some(1) {
        return Ok(None);
    }
    if !output.succeeded() {
        return Err(HookPlanError::ConfigRead(format!(
            "git config --get {key} failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )));
    }
    let Ok(text) = String::from_utf8(output.stdout) else {
        return Err(HookPlanError::ConfigRead(format!(
            "git config --get {key} printed bytes that are not UTF-8"
        )));
    };
    return Ok(Some(String::from(text.strip_suffix('\n').unwrap_or(text.as_str()))));
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
///  Compute the dispatch plan of one transaction from the real repository.
/// Why:
///   The incumbent's `computeHookDispatchPlan`:
///  the hooks path, the event switches (only the exact value `false` disables an event),
///  the combined config parameters, the lock path and whether the lock is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function computeHookDispatchPlan(inputs): Promise<HookDispatchPlan>;
/// ```
pub fn compute_hook_dispatch_plan(
    inputs: &HookPlanInputs<'_>,
) -> Result<HookDispatchPlan, HookPlanError> {
    let configured_hooks: Option<String> = match inputs.snapshot.map(|snapshot| { return snapshot.path_value("core.hookspath"); }) {
        Some(Lookup::Found(value)) => Some(value),
        Some(Lookup::Unset) => None,
        Some(Lookup::Unhandled) | None => read_config(inputs.context, inputs.cwd, "path", "core.hooksPath")?,
    };
    let hooks_path: PathBuf = match configured_hooks {
        None => default_hooks_directory(inputs.common_directory),
        Some(value) => resolve_hooks_path(inputs.worktree_root, Path::new(value.as_str())),
    };
    let disabled_events: Vec<String> = match inputs.snapshot.and_then(snapshot_disabled_events) {
        Some(events) => events,
        None => read_disabled_events(inputs.context, inputs.cwd)?,
    };
    let inherited: String = inherited_config_parameters(inputs.environment)?;
    let overrides: Vec<String> = global_config_overrides(inputs.global_arguments)?;
    let config_parameters: ConfigParameters = combine_config_parameters(
        inherited.as_str(),
        overrides.as_slice(),
    )?;
    let plan: HookDispatchPlan = HookDispatchPlan {
        git_path: path_text(inputs.context.real_git.as_path(), "real Git path")?,
        hooks_path: path_text(hooks_path.as_path(), "hooks path")?,
        disabled_events,
        config_parameters,
        worktree_root: path_text(inputs.worktree_root, "worktree root")?,
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
///   The incumbent writes `JSON.stringify(plan)` followed by one newline,
///  and both dispatchers and the lease validator read the same bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(plan)
/// ```
pub fn encode_hook_dispatch_plan(plan: &HookDispatchPlan) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer.integer("schemaVersion", PLAN_SCHEMA_VERSION);
    writer.string("gitPath", plan.git_path.as_str());
    writer.string("hooksPath", plan.hooks_path.as_str());
    writer.raw(
        "disabledEvents",
        string_array(plan.disabled_events.as_slice()).as_str(),
    );
    writer.raw(
        "configParameters",
        encode_config_parameters(&plan.config_parameters).as_str(),
    );
    writer.string("worktreeRoot", plan.worktree_root.as_str());
    writer.string("lease", plan.lease.as_str());
    writer.string("hookLockPath", plan.hook_lock_path.as_str());
    writer.boolean("skipHookLock", plan.skip_hook_lock);
    return writer.finish();
}

/// What:
///  Spell the config parameters as the incumbent's tagged object.
/// Why:
///   The dispatcher distinguishes an absent variable from an empty one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// configParameters.kind === 'absent' ? { kind: 'absent' } : { kind: 'present', value }
/// ```
fn encode_config_parameters(parameters: &ConfigParameters) -> String {
    // `match` picks the spelling by variant.
    match parameters {
        ConfigParameters::Absent => {
            let mut writer: ObjectWriter = ObjectWriter::new();
            writer.string("kind", "absent");
            return writer.finish();
        }
        ConfigParameters::Present(value) => {
            let mut writer: ObjectWriter = ObjectWriter::new();
            writer.string("kind", "present").string("value", value.as_str());
            return writer.finish();
        }
    }
}

/// What:
///  Read a plan from its JSON text, refusing any other shape.
/// Why:
///   The dispatcher and the lease validator must agree on what a plan is;
///  an unknown event or schema makes the plan unusable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = JSON.parse(text) as HookDispatchPlan; // checked fields
/// ```
pub fn parse_hook_dispatch_plan(text: &str) -> Option<HookDispatchPlan> {
    let object = parse_object(text)?;
    if safe_integer_field(&object, "schemaVersion") != Some(PLAN_SCHEMA_VERSION) {
        return None;
    }
    let events_value = object.get("disabledEvents")?.as_array()?;
    // `mut` lets the event list grow in file order.
    let mut disabled_events: Vec<String> = Vec::new();
    for item in events_value {
        let name: &str = item.as_str()?;
        if !SILENCED_HOOK_EVENTS.contains(&name) {
            return None;
        }
        disabled_events.push(String::from(name));
    }
    let parameters = object.get("configParameters")?.as_object()?;
    let config_parameters: ConfigParameters = match string_field(parameters, "kind")? {
        "absent" => ConfigParameters::Absent,
        "present" => ConfigParameters::Present(String::from(string_field(parameters, "value")?)),
        _ => return None,
    };
    return Some(HookDispatchPlan {
        git_path: String::from(string_field(&object, "gitPath")?),
        hooks_path: String::from(string_field(&object, "hooksPath")?),
        disabled_events,
        config_parameters,
        worktree_root: String::from(string_field(&object, "worktreeRoot")?),
        lease: String::from(string_field(&object, "lease")?),
        hook_lock_path: String::from(string_field(&object, "hookLockPath")?),
        skip_hook_lock: object.get("skipHookLock")?.as_bool()?,
        named_hooks_possible: true,
    });
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
