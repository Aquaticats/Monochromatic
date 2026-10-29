//! Settings of one scripted-server run,
//!  read once from `IDE_SCRIPTED_*` environment variables.

/// What:
///  `Value` is a decoded JSON value of any shape,
///  like TS's `unknown` from `JSON.parse`.
/// Why:
///  Tests hand whole definition and reference results to the server as JSON text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::Value;
/// What:
///  `env` reads process environment variables;
///  `PathBuf` is an owned filesystem path
///       (sibling:
///  borrowed `&Path`).
/// Why:
///  Tests configure the server through the same `environment` table real servers receive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { env } from 'node:process';
/// ```
use std::{env, path::PathBuf};

/// What:
///  A closed set of names for the column unit of protocol positions.
///  `Copy` lets a value
///       be passed like a number.
/// Why:
///  The client must convert positions for whichever unit the server announces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unit = 'utf-8' | 'utf-16' | 'utf-32';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Columns count bytes.
    Utf8,
    /// Columns count UTF-16 code units;
    ///  the protocol's default.
    Utf16,
    /// Columns count characters.
    Utf32,
}

/// How the server answers `initialize`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Init = 'ok' | 'hang' | 'exit' | 'error';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Init {
    /// Answer with capabilities.
    Ok,
    /// Never answer.
    Hang,
    /// Exit with status 3 without answering.
    Exit,
    /// Answer with a protocol error and keep running.
    Error,
}

/// How the server answers hover requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Hover = 'answer' | 'crash' | 'modified' | 'modifiedOnce' | 'silent';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hover {
    /// Answer,
    ///  after the configured delay.
    Answer,
    /// Exit with status 7 without answering.
    Crash,
    /// Always answer with error `-32801` (content modified).
    Modified,
    /// Answer the first request with `-32801`,
    ///  later ones normally.
    ModifiedOnce,
    /// Never answer.
    Silent,
}

/// What:
///  What the server does with one inlay-hint or pull-diagnostics request,
///  chosen by the
///       request's number among the requests of its kind.
///  `Copy` lets a value be passed like a number.
/// Why:
///  A test names exactly which request is left unanswered,
///  held back,
///  or superseded,
///  so the
///      client's reaction is the same however busy the machine is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = 'answer' | 'empty' | 'silent' | 'hold' | 'modified' | 'modifiedThenNotify' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Answer at once.
    Answer,
    /// Answer at once with a list that holds no hint,
    ///  as a server does that is still loading the project.
    Empty,
    /// Never answer.
    Silent,
    /// Keep the answer and write it right after the next answer of the same kind.
    Hold,
    /// Answer with error `-32801` (content modified).
    Modified,
    /// Answer with error `-32801`,
    ///  then send a `window/logMessage` notification.
    ModifiedThenNotify,
    /// Answer with error `-32603` (internal error),
    ///  which the client does not ask again for.
    Failed,
}

/// What:
///  The steps of one variable:
///  words separated by commas,
///  one per request in order.
///       `Vec<Step>` is a growable list (siblings:
///  fixed `[Step; N]`,
///  borrowed `&[Step]`).
/// Why:
///  Requests past the end of the list,
///  and unknown words,
///  are answered at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const steps = (name: string) => read(name, '').split(',').filter(Boolean).map(toStep);
/// ```
fn steps(name: &str) -> Vec<Step> {
    // `Vec::new()` creates an empty list; `mut` allows pushing to it.
    let mut found: Vec<Step> = Vec::new();
    for word in read(name, "").split(',') {
        let step = match word.trim() {
            "" => continue,
            "empty" => Step::Empty,
            "silent" => Step::Silent,
            "hold" => Step::Hold,
            "modified" => Step::Modified,
            "modified-notify" => Step::ModifiedThenNotify,
            "error" => Step::Failed,
            _ => Step::Answer,
        };
        found.push(step);
    }
    return found;
}

/// What:
///  The step for request number `seen`,
///  counted from one.
///  `&[Step]` lends the list.
/// Why:
///  Each kind of request keeps its own count,
///  so a test scripts them independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stepFor = (steps: Step[], seen: number) => steps[seen - 1] ?? 'answer';
/// ```
pub fn step_for(steps: &[Step], seen: u64) -> Step {
    // What: `usize::try_from` converts the count to an index and returns `Result`; `get` returns
    //       `Option<&Step>`; `copied` turns the borrow into a value; `unwrap_or` substitutes "answer".
    // Why: Request zero does not exist, so a count of zero means "answer" too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return steps[seen - 1] ?? 'answer';
    // ```
    let Some(before) = seen.checked_sub(1) else {
        return Step::Answer;
    };
    let Ok(index) = usize::try_from(before) else {
        return Step::Answer;
    };
    return steps.get(index).copied().unwrap_or(Step::Answer);
}

/// What:
///  One record of every setting.
///  `Option<...>` fields are "a value,
///  or nothing";
///       `u64` is an unsigned 64-bit integer (siblings:
///  `u32`,
///  `usize`).
/// Why:
///  Reading the environment once keeps the handlers free of string lookups.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Script = { encoding?: Unit; sync?: number; init: Init; hover: Hover; /* ... */ };
/// ```
#[derive(Clone, Debug)]
pub struct Script {
    /// Announced position encoding;
    ///  absent means none is announced,
    ///  which means UTF-16.
    pub encoding: Option<Unit>,
    /// Announced `textDocumentSync.change` (0 none,
    ///  1 full,
    ///  2 incremental);
    ///  absent means the
    /// capability is omitted entirely.
    pub sync: Option<u64>,
    /// Behavior on `initialize`.
    pub init: Init,
    /// When false,
    ///  only synchronization and hover are announced.
    pub all_features: bool,
    /// Behavior on hover requests.
    pub hover: Hover,
    /// Milliseconds to wait before a hover answer.
    pub hover_delay: u64,
    /// Copy the document version into pushed diagnostics.
    pub versioned_diagnostics: bool,
    /// Push diagnostics after every open and change.
    pub push_diagnostics: bool,
    /// Push diagnostics again after every hover answer,
    ///  as a server with delayed analysis does.
    pub push_after_hover: bool,
    /// What:
    ///  A file whose existence lets the server answer `initialize`;
    ///  `Option<PathBuf>` is "an
    ///       owned path,
    ///  or nothing" (sibling:
    ///  borrowed `&Path`).
    /// Why:
    ///  A test that must act while the server is still starting creates the file only after
    ///      it acted,
    ///  so the start cannot overtake it however busy the machine is.
    pub init_gate: Option<PathBuf>,
    /// Milliseconds to wait before answering `initialize`.
    pub init_delay: u64,
    /// What each inlay-hint request gets,
    ///  in order (`IDE_SCRIPTED_HINT_STEPS`).
    pub hint_steps: Vec<Step>,
    /// What each pull-diagnostics request gets,
    ///  in order (`IDE_SCRIPTED_PULL_STEPS`).
    pub pull_steps: Vec<Step>,
    /// Announce and answer pull diagnostics.
    pub pull_diagnostics: bool,
    /// Announce interest in save notifications.
    pub save: bool,
    /// Send the client-reply probe after the first open.
    pub probe: bool,
    /// File outside the project that the definition answer points at.
    pub outside: Option<PathBuf>,
    /// File every received message and text mirror is appended to,
    ///  one JSON value per line.
    pub report: Option<PathBuf>,
    /// What:
    ///  `Option<Value>` is "a decoded JSON value,
    ///  or nothing".
    /// Why:
    ///  When present,
    ///  the definition answer is exactly this result,
    ///  so a test can name its
    ///      targets;
    ///  absent keeps the fixed list of same-file and unavailable targets.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// definition?: unknown;
    /// ```
    pub definition: Option<Value>,
    /// When present,
    ///  the references answer is exactly this result;
    ///  absent keeps the fixed failure.
    pub references: Option<Value>,
    /// Ignore `exit` and stay alive after the client closed standard input,
    ///  so that only a
    /// kill ends the process.
    pub linger: bool,
    /// One line written to standard error when `shutdown` arrives,
    ///  as a real server reports its
    /// own shutdown there,
    ///  and right before the hover crash ends the process,
    ///  as a crashing
    /// server's last words;
    ///  nothing when absent.
    pub stderr_at_shutdown: Option<String>,
    /// File watchers registered with the client after `initialized`,
    ///  as the protocol's JSON array.
    pub watchers: Option<Value>,
}

/// What:
///  Decode the JSON text of one variable.
///  `Option<Value>` is nothing when the variable is
///       unset or does not hold valid JSON.
/// Why:
///  A test passes a whole protocol result through the server definition's environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function json(name: string): unknown | undefined {
///   try { return JSON.parse(env['IDE_SCRIPTED_' + name] ?? ''); } catch { return undefined; }
/// }
/// ```
fn json(name: &str) -> Option<Value> {
    // `ok()` turns the failed parse of a missing or malformed value into "nothing".
    return serde_json::from_str(&read(name, "")).ok();
}

/// What:
///  Read one variable,
///  or the fallback when it is unset.
///  `&str` parameters are borrowed text.
/// Why:
///  Every setting has a default so a test names only what it changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (name: string, fallback: string) => env['IDE_SCRIPTED_' + name] ?? fallback;
/// ```
fn read(name: &str, fallback: &str) -> String {
    // What: `env::var` returns `Result<String, _>`; `unwrap_or_else` substitutes the closure's
    //       value on failure. `|_|` is a closure that ignores its argument.
    // Why: An unset or non-text variable simply means "use the default".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return env[`IDE_SCRIPTED_${name}`] ?? fallback;
    // ```
    return match env::var(format!("IDE_SCRIPTED_{name}")) {
        Ok(value) => value,
        Err(_) => fallback.to_string(),
    };
}

/// Settings construction.
impl Script {
    /// Read every setting from the environment,
    ///  applying defaults.
    pub fn from_environment() -> Self {
        // What: `match` on borrowed text picks the first arm whose literal equals it; `_` is the
        //       catch-all arm.
        // Why: Unknown values fall back to the protocol default instead of stopping the server.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const encoding = { 'utf-8': 'utf-8', 'utf-16': 'utf-16', 'utf-32': 'utf-32' }[value];
        // ```
        let encoding = match read("ENCODING", "").as_str() {
            // `Some(...)` is the "value present" variant of `Option`.
            "utf-8" => Some(Unit::Utf8),
            "utf-16" => Some(Unit::Utf16),
            "utf-32" => Some(Unit::Utf32),
            // `None` is the "nothing" variant: no encoding is announced.
            _ => None,
        };
        let sync = match read("SYNC", "incremental").as_str() {
            "none" => Some(0),
            "full" => Some(1),
            "absent" => None,
            _ => Some(2),
        };
        let init = match read("INIT", "ok").as_str() {
            "hang" => Init::Hang,
            "exit" => Init::Exit,
            "error" => Init::Error,
            _ => Init::Ok,
        };
        let hover = match read("HOVER", "answer").as_str() {
            "crash" => Hover::Crash,
            "modified" => Hover::Modified,
            "modified-once" => Hover::ModifiedOnce,
            "silent" => Hover::Silent,
            _ => Hover::Answer,
        };
        // What: `parse` converts text to a number and returns `Result`; `unwrap_or(0)` substitutes
        //       zero when the text is not a number.
        // Why: A malformed delay means "no delay".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const hoverDelay = Number(value) || 0;
        // ```
        let hover_delay = read("HOVER_DELAY_MS", "0").parse().unwrap_or(0);
        // `env::var_os` returns `Option<OsString>`; `map(PathBuf::from)` converts a present value.
        let outside = env::var_os("IDE_SCRIPTED_OUTSIDE").map(PathBuf::from);
        let report = env::var_os("IDE_SCRIPTED_REPORT").map(PathBuf::from);
        return Self {
            encoding,
            sync,
            init,
            all_features: read("FEATURES", "all") == "all",
            hover,
            hover_delay,
            versioned_diagnostics: read("DIAG_VERSION", "0") == "1",
            push_diagnostics: read("PUSH", "1") == "1",
            push_after_hover: read("PUSH_AFTER_HOVER", "0") == "1",
            init_delay: read("INIT_DELAY_MS", "0").parse().unwrap_or(0),
            // `map(PathBuf::from)` converts a present value into an owned path.
            init_gate: env::var_os("IDE_SCRIPTED_INIT_GATE").map(PathBuf::from),
            hint_steps: steps("HINT_STEPS"),
            pull_steps: steps("PULL_STEPS"),
            pull_diagnostics: read("PULL", "0") == "1",
            save: read("SAVE", "0") == "1",
            probe: read("PROBE", "0") == "1",
            outside,
            report,
            definition: json("DEFINITION"),
            references: json("REFERENCES"),
            linger: read("LINGER", "0") == "1",
            // `.ok()` turns an unset or unreadable variable into "nothing".
            stderr_at_shutdown: env::var("IDE_SCRIPTED_STDERR").ok(),
            watchers: json("WATCHERS"),
        };
    }

    /// The column unit positions are interpreted in.
    pub fn unit(&self) -> Unit {
        // `unwrap_or` substitutes the protocol default when no encoding is announced.
        return self.encoding.unwrap_or(Unit::Utf16);
    }
}
