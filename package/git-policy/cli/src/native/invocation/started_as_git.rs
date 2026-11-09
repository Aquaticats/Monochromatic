//! What:
//!  The marker that every start of the real Git carries,
//! and the check that stops a monochromatic-git-policy-cli executable which finds it was
//!  started as that Git.
//! Why:
//!  Which file the wrapper selected as Git can be a launcher,
//! a link,
//! a copy or another release of the wrapper,
//! and no test
//!  of that file can recognise every one of them.
//! What no spelling hides is the start itself:
//! the wrapper that starts
//!  "Git" puts its own process number,
//! that process's birth identity and the way it starts the program (replacing itself or
//!  as a child) in the environment of the start.
//! A monochromatic-git-policy-cli executable that finds itself in the very process,
//! or the child,
//!  that a wrapper started as Git has been started as Git,
//! whatever stood in between,
//! and stops at once.
//!  Anything started further down (a hook,
//! an alias,
//! an editor a real Git started) has another process in between and
//!  passes:
//! running `git` there through an installed wrapper stays legitimate.
//! That shape cannot see every way back to the wrapper,
//! because a script between two wrappers puts as many processes
//!  in between as it likes;
//! so the marker also counts the starts of one chain,
//! and a wrapper stops once the chain
//!  holds more than `MOST_STARTS_IN_ONE_CHAIN`,
//! whatever stood between two of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // child_process.spawn(realGit, args, { env: { ...env, CLI_GIT_STARTED_AS_GIT: `${process.pid}\x1f${identity}\x1fchild\x1f${starts}` } });
//! // if (marker && (marker.starts > MOST_STARTS_IN_ONE_CHAIN || marker.pid === process.ppid && sameBirth(marker))) { stop('started as Git by a monochromatic-git-policy-cli process'); }
//! ```

/// The one lookup of a named variable in an injected environment.
use crate::child_environment::{FORWARD_TARGET_VARIABLE, environment_value};
/// What:
///  The start a chain's descendants still see through the descriptor they inherit,
/// which a launcher that clears
///  the environment and replaces itself with the wrapper cannot hide.
/// Why:
///   The environment marker and the environment of the process above are both records a launcher can
/// destroy;
/// an open descriptor survives `exec` and every environment clearing.
use crate::chain_starts_record::ChainRecord;
/// What:
///  The marker's own format and the one lookup of it in an environment,
/// kept in a module both this guard and the chain record
///  read.
/// Why:
///   A record a launcher cannot erase and the guard that reads it speak the same text,
/// and neither may own it alone.
pub use crate::start_marker::{
    FIELD_SEPARATOR, MOST_STARTS_IN_ONE_CHAIN, StartMarker, StartMode, STARTED_AS_GIT_VARIABLE, marker_of, marker_value, parse_marker,
};
/// What:
///  The birth identity of a process,
/// which changes when its number is reused.
/// Why:
///   A process number alone could name a later process after the first one ended;
/// the identity cannot.
use crate::process_identity::{current_birth_identity, process_birth_identity};
/// What:
///  `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes (siblings `&str`/`String` must be UTF-8).
/// Why:
///   Environment values and paths need not be UTF-8.
use std::ffi::{OsStr, OsString};
/// `Path` is a borrowed filesystem path of raw bytes (owned sibling `PathBuf`).
use std::path::Path;
/// The child-process builder that receives the marker.
use std::process::Command;

/// What:
///  How many processes above this one the chain marker is looked for in,
/// when this process's own environment has
///  none.
/// Why:
///   A launcher that clears the environment of the program it starts removes the marker from that
/// program's
///  environment only;
/// the environment the launcher itself was started with still holds it,
/// and neither
///  `env --ignore-environment` nor a shell between the two changes the launcher's own environment.
/// `env` replaces itself with
///  the program it runs,
/// so a launcher that runs `env --ignore-environment` and one that runs
/// `env --ignore-environment sh -c 'exec ...'` put one and two processes between the marker and the arriving
///  wrapper;
/// two levels cover both,
/// and they cost 0.18 ms of a forwarded command measured by alternating two builds that differ only in this walk
///  (`native:build`,
/// 300 pairs,
/// median 4.76 ms without the walk;
/// `doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
///  finding 2 of the round-1 review).
/// The reads are made only when this process's own environment carries no marker,
/// so a
///  start that is already marked costs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ANCESTORS_ASKED_FOR_THE_CHAIN = 2;
/// ```
pub const ANCESTORS_ASKED_FOR_THE_CHAIN: usize = 2;

///  The birth identity of this process,
/// or empty text where none can be read.
/// Why:
///   A start is marked even where the platform cannot name a birth;
/// the check then compares process numbers alone.
///  Only Linux reads it here,
/// from one small file that stays cached for the life of the process;
/// the other platforms
///  would start a program to ask,
/// which every real-Git start must not pay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = process.platform === 'linux' ? readBirth(process.pid) ?? '' : '';
/// ```
fn own_identity() -> String {
    #[cfg(target_os = "linux")]
    {
        // `.unwrap_or_default()` substitutes empty text when the identity cannot be read.
        return current_birth_identity().unwrap_or_default();
    }
    #[cfg(not(target_os = "linux"))]
    {
        return String::new();
    }
}

/// What:
///  Put the marker in the environment of a real-Git start.
///       `&mut Command` lends the builder for changing.
/// Why:
///   Every place that starts the real Git calls this,
/// so a wrapper that arrives as Git can tell.
/// `Command::env` overrides
///  an earlier value of the same name,
/// so a start that replaces the process can override the child marker that the shared
///  builder put there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command.env.CLI_GIT_STARTED_AS_GIT = markerValue(mode, process.pid, ownIdentity());
/// ```
pub fn mark_real_git_start(command: &mut Command, mode: StartMode) {
    let starts: u32 = own_start_count().saturating_add(1);
    let marker: OsString = marker_value(mode, std::process::id(), own_identity().as_str(), starts);
    // The record keeps what this start is, so a launcher that clears the environment of the wrapper it starts cannot
    //  hide it (`chain_starts_record.rs`).
    crate::chain_starts_record::remember_marker(marker.as_os_str());
    command.env(STARTED_AS_GIT_VARIABLE, marker);
}

/// What:
///  How many marked starts of real Git this process's chain records,
/// read from its own environment or,
///  when that
///  carries none,
/// from the nearest process above it that does.
/// Why:
///   Every builder that starts Git calls `mark_real_git_start` with nothing but its command,
/// so the count of the chain is read from this
///  process's environment,
/// which is where a start carries it.
/// A launcher that clears the environment of the wrapper it
///  starts removes it there,
/// and the environment of the launcher itself is the record that survives
///  (`a_launcher_that_clears_the_environment_ends_the_chain`);
/// without it the chain of such a launcher never ends.
/// No policy input is
///  read here;
/// the one variable is the wrapper's own trace of its starts.
/// An environment of another program's making
///  reads as no start,
/// which is what a first start is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const starts = ownMarker()?.starts ?? nearestAncestorMarker()?.starts ?? 0;
/// ```
fn own_start_count() -> u32 {
    if let Some(value) = std::env::var_os(STARTED_AS_GIT_VARIABLE)
        && let Some(marker) = parse_marker(value.as_os_str())
    {
        return marker.starts;
    }
    // The own environment carries none: the nearest ancestor that does, and the record a launcher cannot erase, say
    //  which chain this start continues; the longer of the two is the one that counts.
    let from_above: u32 = ancestor_marker().map_or(0, starts_of);
    let from_record: u32 = record_marker().map_or(0, starts_of);
    return from_above.max(from_record);
}

/// The number of starts a marker records,
/// for the callers that only need that.
fn starts_of(marker: StartMarker) -> u32 {
    return marker.starts;
}

/// The marker of a chain,
/// where the caller has no use for the environment it was read from.
fn marker_of_pair(found: (StartMarker, Vec<(OsString, OsString)>)) -> StartMarker {
    let (marker, environment) = found;
    drop(environment);
    return marker;
}

/// The marker of a chain and the borrowed environment it was read from,
/// for the callers that only need the marker.
fn marker_of_borrowed_pair<'a>(found: (StartMarker, &'a [(OsString, OsString)])) -> StartMarker {
    let (marker, _environment) = found;
    return marker;
}

/// What:
///  The marker the start above this one wrote into the chain record,
/// when that start is this process or the one that started it.
/// Why:
///   A record names the last start of Git of this process tree,
/// and only a start of this very process (a launcher that
///  replaced itself with the wrapper) or of the process that started this one (a launcher that started it as a child)
///  belongs to this chain;
/// a record naming any other process is a command that has ended,
/// and its count starts
///  again at one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordMarker(): StartMarker | undefined { const marker = parseMarker(record.marker); return continuesHere(marker) ? marker : undefined; }
/// ```
fn record_marker() -> Option<StartMarker> {
    let record: &ChainRecord = crate::chain_starts_record::this_chain()?;
    let marker: StartMarker = parse_marker(OsStr::new(record.marker()?))?;
    if !continues_this_start(&marker) {
        return None;
    }
    return Some(marker);
}

/// What:
///  Whether the marker the record names wrote for a start of this process or of the process that started it.
/// Why:
///   Those are the two ways a launcher can hide a start:
/// in its own process by replacing itself with the wrapper,
/// or in a
///  child it started and waits for.
/// Another process's start is no part of this chain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function continuesThisStart(marker) { const candidate = marker.starterPid === process.pid ? process.pid : (marker.starterPid === process.ppid ? process.ppid : undefined); return candidate !== undefined && identityAgrees(candidate, marker); }
/// ```
fn continues_this_start(marker: &StartMarker) -> bool {
    return record_continues(marker, chain_ancestors().as_slice());
}

/// What:
///  The most processes above this one the chain record's start is looked for in:
/// every one up to the first
///  process of the host.
/// Why:
///   A record names the start above this one,
/// and a launcher can put any number of relays between the two;
/// a
///  record whose start is not found starts the count again,
/// so a chain of five relays had no end
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 1 of the round-4
///  review).
/// The walk costs one read of the kernel's process record per level and is made only when a record
/// exists,
/// so a first command pays nothing and a start that follows one pays 1.05 ms on this host
///  (measured against the same start without the walk:
/// paired median +33.7 ms over the 32 starts of one
/// chain,
///  inside the 126 to 143 ms run-to-run spread of either arm).
/// The number is a guard against a tree that cannot be walked,
/// not a
///  limit any chain of finishable depth reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MOST_ANCESTORS_ASKED_FOR_THE_RECORD = 4096;
/// ```
const MOST_ANCESTORS_ASKED_FOR_THE_RECORD: usize = 4096;

/// What:
///  The processes a record may name:
/// this one and the first few above it,
/// nearest first.
/// Why:
///   A record of this process is the launcher that replaced itself with the wrapper;
/// a record of a process above
///  is the launcher that started this one and waits,
/// however many relays it started between the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chainAncestors() { const found = [process.pid]; let pid = process.ppid; for (let level = 0; level < N; level++) { found.push(pid); pid = parentOf(pid); } return found; }
/// ```
fn chain_ancestors() -> Vec<u32> {
    // `mut` allows this process and its ancestors to be collected in order.
    let mut found: Vec<u32> = vec![std::process::id()];
    let mut pid: u32 = parent_pid();
    for _level in 0..MOST_ANCESTORS_ASKED_FOR_THE_RECORD {
        // The first process of the host is where every chain ends, and it is never one of these starts.
        if pid <= 1 {
            break;
        }
        found.push(pid);
        let Some(parent) = parent_of(pid) else { break };
        pid = parent;
    }
    return found;
}

/// What:
///  Whether the record's start is this process,
/// or one of the live processes named here above it.
/// Why:
///   The rule is the seam its controls need:
/// a descriptor record names the start that wrote it,
/// and only a
///  process of this chain continues it,
/// so a record of a command that has ended starts the count again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordContinues(marker, processes) { return processes.includes(marker.starterPid) && identityAgrees(marker.starterPid, marker); }
/// ```
fn record_continues(marker: &StartMarker, processes: &[u32]) -> bool {
    if !processes.contains(&marker.starter_pid) {
        return false;
    }
    // What: A read of the wrapper's own continues the chain from any process of it.
    // Why:  A record of a query start says the start above began its Git to read the repository, with the output captured
    //       and the input closed; a wrapper arriving anywhere under that start came from the file that Git is, which is
    //       the wrapper leading back to itself, exactly as the marker path refuses it. Only accepting the reading process
    //       left the branch a relay starts uncounted (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
    //       finding 2 of the round-3 review).
    if marker.starter_identity.is_empty() {
        return true;
    }
    return match live_identity_of(marker.starter_pid) {
        Some(found) => found == marker.starter_identity,
        None => true,
    };
}

/// The file the start above chose as Git,
/// as the record this process inherited keeps it.
fn chosen_file(record: &ChainRecord) -> Option<&str> {
    return record.chosen();
}

/// What:
///  The environment a refusal is written from,
/// with the file the start chose as Git added to it.
/// Why:
///   The record names that file and this process's environment does not,
/// and the message a person reads names both
///  the file and the `PATH` that offered it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withTheChosenFile(environment) { const chosen = record.chosen; return chosen ? [...environment, [FORWARD_TARGET, chosen]] : environment; }
/// ```
fn with_the_chosen_file(environment: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    // `mut` allows the chosen file to be added after the caller's variables.
    let mut said_in: Vec<(OsString, OsString)> = environment.to_vec();
    let chosen: Option<&str> = crate::chain_starts_record::this_chain().and_then(chosen_file);
    if let Some(target) = chosen {
        said_in.push((OsString::from(FORWARD_TARGET_VARIABLE), OsString::from(target)));
    }
    return said_in;
}

/// What:
///  The marker held by the nearest process above this one,
/// or nothing when none of them holds one.
/// Why:
///   This is the record a launcher that clears the environment cannot reach:
/// it is the launcher's own environment,
///  written by the wrapper that started the launcher as its Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const environment of ancestorEnvironments()) { const marker = parseMarker(environment.CLI_GIT_STARTED_AS_GIT); if (marker) return marker; }
/// ```
fn ancestor_marker() -> Option<StartMarker> {
    return first_marker_above(environment_of).map(marker_of_pair);
}

/// What:
///  The nearest process above this one that holds a chain marker,
/// and the environment it holds it in.
/// Why:
///   The walk stops at the first marker,
/// so a chain that is in progress costs its levels one read each and not
///  the whole bound;
/// the reader is a parameter so a control can name the processes the walk sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstMarkerAbove(environmentOf): [StartMarker, [string, string][]] | undefined;
/// ```
fn first_marker_above(environment_of: impl Fn(u32) -> Option<Vec<(OsString, OsString)>>) -> Option<(StartMarker, Vec<(OsString, OsString)>)> {
    let mut pid: u32 = parent_pid();
    for _level in 0..ANCESTORS_ASKED_FOR_THE_CHAIN {
        if pid == 0 {
            return None;
        }
        let environment: Vec<(OsString, OsString)> = environment_of(pid)?;
        if let Some(marker) = marker_of(environment.as_slice()) {
            return Some((marker, environment));
        }
        let parent: u32 = parent_of(pid)?;
        pid = parent;
    }
    return None;
}

/// What:
///  The environments of the first `ANCESTORS_ASKED_FOR_THE_CHAIN` processes above this one,
/// nearest first,
///  each as name/value pairs.
/// Why:
///   `env --ignore-environment` clears the environment it passes on and nothing else,
/// so the environment of the
///  process that ran it is still there to read,
/// and so is the one of a shell above that.
/// A process whose
///  environment or parent cannot be read ends the walk:
/// nothing further up can belong to a chain this process is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ancestorEnvironments(): [string, string][][] { const found = []; let pid = ppid; for (let level = 0; level < ANCESTORS; level++) { const env = environmentOf(pid); if (!env) break; found.push(env); pid = parentOf(pid); } return found; }
/// ```
#[cfg(unix)]
fn ancestor_environments() -> Vec<Vec<(OsString, OsString)>> {
    let mut found: Vec<Vec<(OsString, OsString)>> = Vec::new();
    let mut pid: u32 = parent_pid();
    for _level in 0..ANCESTORS_ASKED_FOR_THE_CHAIN {
        if pid == 0 {
            break;
        }
        let Some(environment) = environment_of(pid) else { break };
        found.push(environment);
        let Some(parent) = parent_of(pid) else { break };
        pid = parent;
    }
    return found;
}

/// Without a process table there is no ancestor to read (this crate runs on Linux only).
#[cfg(not(unix))]
fn ancestor_environments() -> Vec<Vec<(OsString, OsString)>> {
    return Vec::new();
}

/// Whether this byte separates the entries of a process environment.
fn is_separator(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether this byte separates a name from its value in a process environment.
fn is_equals_sign(byte: &u8) -> bool {
    return *byte == b'=';
}

/// What:
///  The environment of the process with this number,
/// as name/value pairs,
/// or nothing when it cannot be read.
/// Why:
///   `/proc/<pid>/environ` holds the bytes the program was started with,
/// separated by NUL bytes,
/// and a
///  name may be separated from its value by the first `=` only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(`/proc/${pid}/environ`, 'binary').split('\0').filter(Boolean).map((entry) => [entry.slice(0, entry.indexOf('=')), entry.slice(entry.indexOf('=') + 1)])
/// ```
#[cfg(unix)]
fn environment_of(pid: u32) -> Option<Vec<(OsString, OsString)>> {
    use std::os::unix::ffi::OsStrExt;
    let bytes: Vec<u8> = std::fs::read(format!("/proc/{pid}/environ")).ok()?;
    let mut environment: Vec<(OsString, OsString)> = Vec::new();
    for entry in bytes.split(is_separator) {
        if entry.is_empty() {
            continue;
        }
        let Some(equals) = entry.iter().position(is_equals_sign) else { continue };
        let name: OsString = OsString::from(OsStr::from_bytes(&entry[..equals]));
        let value: OsString = OsString::from(OsStr::from_bytes(&entry[equals + 1..]));
        environment.push((name, value));
    }
    return Some(environment);
}

/// What:
///  The number of the parent of the process with this number,
/// or nothing when it cannot be read.
/// Why:
///   `/proc/<pid>/stat` holds the command name in parentheses,
/// which may contain spaces and parentheses of its own,
/// so
///  the fields after it are read from after its last `)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(readFileSync(`/proc/${pid}/stat`, 'utf8').slice(readFileSync(...).lastIndexOf(')') + 2).split(' ')[1])
/// ```
#[cfg(unix)]
fn parent_of(pid: u32) -> Option<u32> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_name: &str = &text[text.rfind(')')? + 1..];
    let mut fields = after_name.split_whitespace();
    // The first field after the name is the process's state; its parent's number is the second.
    let _state = fields.next()?;
    return fields.next()?.parse::<u32>().ok();
}

/// What:
///  The words a person reads when a monochromatic-git-policy-cli executable stops because it was started as Git.
///       `&Path` borrows this executable's path for the message.
/// Why:
///   The message names the executable,
/// the process that started it and how,
/// the Git that process chose when the
///  environment says so,
/// the `PATH` this process saw,
/// why stopping is right,
/// and every way out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalText(marker, ownExecutable, environment): string;
/// ```
pub fn refusal_text(marker: &StartMarker, own_executable: &Path, environment: &[(OsString, OsString)]) -> String {
    let how: String = if marker.mode == StartMode::Query {
        format!(
            "this is the program that a monochromatic-git-policy-cli wrapper that runs as process {} started as Git to read the repository itself, so the `git` that wrapper chose leads back to monochromatic-git-policy-cli",
            marker.starter_pid
        )
    } else if marker.starts > MOST_STARTS_IN_ONE_CHAIN {
        format!(
            "this is start {} of one chain of monochromatic-git-policy-cli starts, so a `git` on the PATH of each of them leads back to monochromatic-git-policy-cli, however many processes stand between two starts",
            marker.starts
        )
    } else if marker.mode == StartMode::Replace {
        format!("this is process {}, which a monochromatic-git-policy-cli wrapper replaced with this program", marker.starter_pid)
    } else {
        format!("this is a child of the monochromatic-git-policy-cli wrapper that runs as process {} and waits for its answer", marker.starter_pid)
    };
    let chosen: String = match environment_value(environment, FORWARD_TARGET_VARIABLE) {
        Some(target) => format!(" The file that process chose as Git is {}.", Path::new(&target).display()),
        None => String::new(),
    };
    let search_path: String = match environment_value(environment, "PATH") {
        Some(value) => format!(" PATH here is {}.", Path::new(&value).display()),
        None => String::from(" PATH is not set here."),
    };
    return format!(
        "this monochromatic-git-policy-cli executable ({}) was started as Git by a monochromatic-git-policy-cli wrapper: {how}.{chosen}{search_path} {WAYS_OUT_TEXT}",
        own_executable.display()
    );
}

/// What:
///  The last part of every refusal:
/// what the wrapper never does,
/// what the shape means,
/// and the ways out.
/// Why:
///   Every refusal ends the same way,
/// so a person who reads one has read them all,
/// and a second wording cannot drift
///  from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WAYS_OUT_TEXT = 'A monochromatic-git-policy-cli wrapper never runs itself as Git, ...';
/// ```
const WAYS_OUT_TEXT: &str =
    "A monochromatic-git-policy-cli wrapper never runs itself as Git, so it stops here instead of starting one more wrapper, which would \
     repeat without end. This happens when the first `git` that PATH offers after the wrapper leads back to monochromatic-git-policy-cli: \
     a link to it, a copy of it, its toolchain launcher, or another release of it. To fix it, put the directory of the \
     real Git (for example /usr/bin) on the PATH of the program that started the first monochromatic-git-policy-cli, or remove the entry that \
     leads back to monochromatic-git-policy-cli from that PATH.";

/// What:
///  The birth identity of the process with a number,
/// read from the operating system now.
///       `.ok().flatten()` turns a read failure and an absent process into the same `None`.
/// Why:
///   The check treats an identity it cannot read as no contradiction,
/// so both cases need no distinction here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveIdentityOf(pid: number): string | undefined { try { return readBirth(pid); } catch { return undefined; } }
/// ```
fn live_identity_of(pid: u32) -> Option<String> {
    return process_birth_identity(i64::from(pid)).ok().flatten();
}

/// What:
///  Whether this process was started as Git by a monochromatic-git-policy-cli wrapper,
/// read from the live process,
/// and if so the words to
///  say so.
///       `Option<String>` is "the message,
/// or nothing when this process was not started as Git".
/// Why:
///   The executable calls it once,
/// before it plans anything.
/// It reads the parent's number and,
/// only when the marker's
///  number matches,
/// one small process file for the birth identity,
/// so an ordinary start costs a variable lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalWhenStartedAsGit(environment, ownExecutable): string | undefined;
/// ```
pub fn refusal_when_started_as_git(environment: &[(OsString, OsString)], own_executable: &Path) -> Option<String> {
    return refusal_when_started_as_git_among(environment, own_executable, ancestor_environments().as_slice());
}

/// What:
///  `refusal_when_started_as_git` with the environments of the processes above this one given,
/// and what the
///  chain marker says there.
/// Why:
///   A launcher that clears the environment of the wrapper it starts is found through the
/// environment
///  the launcher itself was started with,
/// so the control of that shape hands the check made-up ancestors instead of
///  reading a process table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalWhenStartedAsGitAmong(environment, ownExecutable, ancestors): string | undefined;
/// ```
pub fn refusal_when_started_as_git_among(
    environment: &[(OsString, OsString)],
    own_executable: &Path,
    ancestors: &[Vec<(OsString, OsString)>],
) -> Option<String> {
    let facts: ProcessFacts<'_> = ProcessFacts { own_pid: std::process::id(), parent_pid: parent_pid(), identity_of: &live_identity_of };
    if let Some((marker, said_in)) = chain_marker_and_where(environment, ancestors)
        && started_as_git(&marker, &facts)
    {
        // The environment the marker was read from is the one that names the `git` that led back and the PATH that
        //  offered it; a chain's later levels have no marker of their own to read them from.
        return Some(refusal_text(&marker, own_executable, said_in));
    }
    // What: The record is read whatever a marker above says, including one that matches nothing here.
    // Why:  A launcher that clears the environment and replaces itself with the wrapper leaves the marker of that start
    //       only in the record, while the environment above still holds the chain's older marker, which names the outer
    //       start and matches nothing; returning there skipped the record and the wrapper replaced itself with the
    //       launcher without end (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
    //       findings 1 and 2 of the round-3 review).
    let marker: StartMarker = record_marker()?;
    if !started_as_git(&marker, &facts) {
        return None;
    }
    // The record also names the file that start chose as Git, which is the one a person can remove from PATH.
    return Some(refusal_text(&marker, own_executable, with_the_chosen_file(environment).as_slice()));
}

/// What:
///  The chain marker this start continues:
/// the one in this process's own environment,
/// or else the one held by the nearest
///  process above it that holds any.
/// Why:
///   The count of a chain and the identity of the start it began are one record,
/// and a launcher that clears the
///  environment of the wrapper it starts removes it from that wrapper only
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-1
///  review).
/// The nearer record wins,
/// so a wrapper that carries its own says nothing about the processes above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chainMarkerAmong(environment, ancestors) { return parseMarker(environment.CLI_GIT_STARTED_AS_GIT) ?? ancestors.map(parseMarker).find(Boolean); }
/// ```
pub fn chain_marker_among(
    environment: &[(OsString, OsString)],
    ancestors: &[Vec<(OsString, OsString)>],
) -> Option<StartMarker> {
    return chain_marker_and_where(environment, ancestors).map(marker_of_borrowed_pair);
}

/// What:
///  `chain_marker_among` and the environment the marker was read from.
/// Why:
///   The message a person reads names the file that start chose as Git and the PATH that offered it,
/// and those are
///  recorded beside the marker,
/// so a chain whose later levels carry none of their own still says where it came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chainMarkerAndWhere(environment, ancestors): [StartMarker, [string, string][]] | undefined;
/// ```
pub fn chain_marker_and_where<'a>(
    environment: &'a [(OsString, OsString)],
    ancestors: &'a [Vec<(OsString, OsString)>],
) -> Option<(StartMarker, &'a [(OsString, OsString)])> {
    if let Some(marker) = marker_of(environment) {
        return Some((marker, environment));
    }
    for ancestor in ancestors {
        if let Some(marker) = marker_of(ancestor.as_slice()) {
            return Some((marker, ancestor.as_slice()));
        }
    }
    return None;
}

/// What:
///  What the check asks of the live process.
///       `&dyn Fn(u32) -> Option<String>` borrows any function from a process number to its birth identity.
/// Why:
///   A control hands the check made-up numbers and identities;
/// production hands it the process's own facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessFacts = { ownPid: number; parentPid: number; identityOf: (pid: number) => string | undefined };
/// ```
pub struct ProcessFacts<'a> {
    /// This process's number.
    pub own_pid: u32,
    /// The number of this process's parent.
    pub parent_pid: u32,
    /// The birth identity of the process with a number;
    /// `None` when it cannot be read.
    pub identity_of: &'a dyn Fn(u32) -> Option<String>,
}

/// What:
///  Whether the marker names the process this one was started by,
/// in the way its mode says,
/// or says this
///  start is one of a chain too long to be anything but the wrapper leading back to itself.
/// Why:
///   `Replace`:
/// the starter replaced itself,
/// so this is that very process (same number,
/// same birth).
/// `Child`:
/// the starter
///  ran this program as its child,
/// so the starter is this process's parent.
/// In both a identity the platform can read must
///  agree,
/// so a later process that reuses the number is not taken for the starter.
/// A process further down has another
///  process in between and is not the start;
/// a chain longer than `MOST_STARTS_IN_ONE_CHAIN` reached this process whatever stood between
///  its starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startedAsGit(marker: StartMarker, facts: ProcessFacts): boolean {
///   return marker.starts > MOST_STARTS_IN_ONE_CHAIN || sameStarter(marker, facts);
/// }
/// ```
pub fn started_as_git(marker: &StartMarker, facts: &ProcessFacts<'_>) -> bool {
    // A start the wrapper made to read the repository runs nothing of the person's, so whatever stands between it and this
    //  process, a wrapper here came from the file that start chose as Git.
    if marker.mode == StartMode::Query {
        return true;
    }
    if marker.starts > MOST_STARTS_IN_ONE_CHAIN {
        return true;
    }
    let candidate: u32 = if marker.mode == StartMode::Replace { facts.own_pid } else { facts.parent_pid };
    if candidate != marker.starter_pid {
        return false;
    }
    if marker.starter_identity.is_empty() {
        return true;
    }
    // What: `match` on the identity read: a readable identity must be the recorded one; an unreadable one cannot
    //       contradict it, so the numbers decide.
    // Why:  Stopping one start too many is repaired by the person who reads the message; missing a start is not.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = facts.identityOf(candidate); return found === undefined || found === marker.starterIdentity;
    // ```
    match (facts.identity_of)(candidate) {
        Some(found) => return found == marker.starter_identity,
        None => return true,
    }
}

/// What:
///  The process number of this process's parent.
/// Why:
///   A child start leaves the starter as the parent;
/// the standard library does not expose it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parentPid = process.ppid;
/// ```
#[cfg(unix)]
fn parent_pid() -> u32 {
    // SAFETY: `getppid` takes no argument, reads no memory of this process and cannot fail.
    let parent: libc::pid_t = unsafe { libc::getppid() };
    // `u32::try_from(..)` refuses a negative number, which `getppid` never returns; `.unwrap_or(0)` then means no parent.
    return u32::try_from(parent).unwrap_or(0);
}

/// Without a parent number the child check matches nothing (this crate runs on Linux only).
#[cfg(not(unix))]
fn parent_pid() -> u32 {
    return 0;
}

/// Disposable-fact controls of the marker and the check stay out of the release executable.
#[cfg(test)]
#[path = "started_as_git_tests.rs"]
mod tests;
