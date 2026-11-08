//! What:
//!  The resident commit service as a mode of the one executable:
//! `monochromatic-git-policy-cli monochromatic-git-policy-cli service run <options>`.
//! Why:
//!  The release ships one program (owner,
//! 2026-11-04:
//! "Bazel doesn't need to distribute as 2 bins."),
//! so the service is a mode
//!  of the wrapper's executable and not a second one.
//! The operating system starts it on demand and hands it the listening
//!  socket (socket activation);
//! the wrapper never starts it.
//! The mode is chosen by the three leading words of the arguments and
//!  never by the name the program was started as,
//! so the executable's own path in a unit file names nothing but itself.
//!  Development and test runs name an address instead.
//!  Options:
//! `--journal <directory>` (required),
//! `--git <path>` (default `/usr/bin/git`),
//! and one of
//!  `--abstract <name>` or `--path <socket>`;
//! with neither,
//! the socket comes from the service manager.
//! Test runs may also
//!  give `--idle-seconds <n>` and `--release <text>` (the release announced to clients;
//! default this build's).
//! The unit
//!  instance of one worktree passes `--instance <name>` (`service_instance.rs`),
//! and that service refuses messages about
//!  any other worktree;
//! without it a development run serves any.
//!  The service finishes no batch of an earlier release's helper and has no option that does:
//! the look that would guard it cannot
//!  exclude a wrapper of the earlier release that starts the helper meanwhile (cross-vendor reviews of 2026-11-07),
//! so only
//!  `git monochromatic-git-policy-cli service settle-earlier-batches`,
//! run by a person,
//! does it
//!  (`earlier_service_journal_adopted.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (argv.slice(0, 3).join(' ') === 'monochromatic-git-policy-cli service run') process.exit(await runService(argv.slice(3)));
//! ```

use crate::service_journal::{DONE_RETAINED, prune_done};
use crate::wrapper_names_old_and_new::COMMAND_WORD;
use crate::service_land::{LandContext, recover_instance};
use crate::service_protocol::PROTOCOL_VERSION;
use crate::service_timing::{log, started_line};
use crate::service_server::{Bind, IDLE_EXIT, ServeSettings, bind, serve};
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

/// The words,
/// first in the arguments,
/// that select the service mode.
/// They sit inside the wrapper's own `monochromatic-git-policy-cli` namespace,
/// and
///  `run` is a third word apart from `status`,
/// which asks a running service whether it answers.
pub const SERVICE_WORDS: [&str; 3] = [COMMAND_WORD.current, "service", "run"];

/// The words a unit file an earlier release wrote passes;
///  the service mode answers them the same.
pub const EARLIER_SERVICE_WORDS: [&str; 3] = [COMMAND_WORD.earlier, "service", "run"];

/// The most workers and blocking-pool threads the service's runtime has: 4.
/// A service runs per worktree and lives for hours,
/// so the budget-sized default (one worker per CPU less a margin:
/// 25 on a 28-CPU host,
/// 27 idle threads in all) is a standing
/// cost of every service.
/// Measured 2026-11-07 on that host with release binaries that differ only in this constant,
/// four rounds alternating 2,
/// 4,
/// 8 and the uncapped service,
/// 30 samples a scenario (`bin/native-perf.ts`),
/// the added median of a commit through the service in milliseconds:
/// uncapped 19.0,
/// 18.4,
/// 18.4,
/// 18.5 (spread 0.6);
/// 2 workers 25.6,
/// 22.7,
/// 23.0,
/// 22.4 (4.9 above the uncapped mean,
/// outside the spread);
/// 4 workers 18.4,
/// 18.3,
/// 18.1,
/// 18.7 (0.2 below it,
/// inside);
/// 8 workers 18.6,
/// 18.6,
/// 18.2,
/// 18.5.
/// The signed commit and the forwarded commands show the same:
/// only 2 workers is outside the spread.
/// So 4 is the smallest cap inside the spread,
/// and never below two (the owner's "Full on tokio.
/// Multi-thread.").
/// An idle service then has 6 threads whatever the number of connections.
/// The cap does not move the smallest address-space limit under which the service starts (93 MiB at every cap,
/// 27 MiB before T10):
/// that is the runtime plan's reserve (`ADDRESS_SPACE_RESERVE_BYTES`).
/// The numbers are kept in `doc/planning/git-policy-cli-full-tokio.md`,
/// "Step T10,
/// the worker cap".
pub const SERVICE_WORKER_CAP: usize = 4;

/// The exit status of a start that was refused for its arguments or its Git.
pub const EXIT_REFUSED: i32 = 2;

/// The exit status of a start that could not listen.
pub const EXIT_NOT_LISTENING: i32 = 3;

/// What:
///  Whether the arguments (without the program name) select the service mode.
/// Why:
///   The decision reads the words only,
/// before anything else of the process is set up,
/// so the wrapper's own start never runs
///  the service's setup and the service's start never runs the wrapper's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isServiceStart(args) // args[0..3] equal 'monochromatic-git-policy-cli', 'service', 'run'
/// ```
pub fn is_service_start(arguments: &[OsString]) -> bool {
    return arguments.len() >= SERVICE_WORDS.len()
        && (starts_with_service_words(arguments, &SERVICE_WORDS) || starts_with_service_words(arguments, &EARLIER_SERVICE_WORDS));
}

/// The words of the arguments after the service mode words,
/// as text;
/// a word that is not text is refused.
pub fn service_options(arguments: &[OsString]) -> Result<Vec<String>, OsString> {
    // The first word that is not text is the refusal.
    let mut options: Vec<String> = Vec::new();
    for argument in arguments.iter().skip(SERVICE_WORDS.len()) {
        match argument.to_str() {
            Some(text) => options.push(text.to_string()),
            None => return Err(argument.clone()),
        }
    }
    return Ok(options);
}

/// What:
///  Whether the arguments start with the words that select the service mode.
/// Why:
///  `is_service_start` needs at least that many arguments and each of them equal to its word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SERVICE_WORDS.every((word, at) => arguments[at] === word)
/// ```
fn starts_with_service_words(arguments: &[OsString], words: &[&str; 3]) -> bool {
    for (argument, word) in arguments.iter().zip(words) {
        if argument != *word {
            return false;
        }
    }
    return true;
}

/// What:
///  The value of a numeric option,
/// nothing when the option is absent or its value is not a number.
/// Why:
///  The ceiling and idle options are read this way;
/// a value that does not parse leaves the default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const n = Number(option(args, name)); return Number.isInteger(n) && n >= 0 ? n : undefined;
/// ```
fn number_option(arguments: &[String], name: &str) -> Option<u64> {
    let text: String = option(arguments, name)?;
    return text.parse::<u64>().ok();
}

fn option(arguments: &[String], name: &str) -> Option<String> {
    // The word after the first occurrence of the option name.
    for (at, argument) in arguments.iter().enumerate() {
        if argument == name {
            return arguments.get(at + 1).cloned();
        }
    }
    return None;
}

/// What:
///  Refuses a `--git` that would make every Git the service starts start the wrapper again.
/// Why:
///   A launcher of the wrapper,
/// a link to it,
/// a copy of it or the executable itself named as Git makes the service start
///  the wrapper for every Git step (a unit written by an earlier release can hold such a path),
/// and the executable is now
///  the service's own program,
/// so its own file is the first candidate to refuse (owner:
/// "We should never invoke ourselves
///  recursively.").
/// The executable's path is identification only;
/// nothing is started from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (isWrapper(git, ownExecutable)) { exit(2) }
/// ```
fn git_is_wrapper(git: &std::path::Path) -> bool {
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    return match crate::real_git::process_resolution_inputs(environment.as_slice()) {
        Ok(inputs) => matches!(crate::real_git_candidate::classify_candidate(git, inputs.own_executable.as_path()), crate::real_git_candidate::CandidateKind::Wrapper),
        // Without its own path only a launcher's record can be recognised.
        Err(_) => crate::real_git_candidate::is_wrapper_launcher(git, None),
    };
}

/// What:
///  Runs the service until it ends and gives the exit status.
/// Why:
///   The setup of the process comes first and only here:
/// the service's Git starts run with hooks off,
/// a signal that ends the
///  service leaves a line,
/// and the push advice names the service's remedies.
/// The wrapper's own entry never reaches this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = await runService(options)
/// ```
pub fn run(arguments: &[String]) -> i32 {
    // Every Git the service starts (landing, index, push, upkeep) runs with hooks off, as the wrapper's own starts do: a hook
    //  named by a repository's configuration is the person's, run by their own Git, never by the service.
    crate::hooks_off::activate();
    // The runtime is built by the first need (the warm below, or the acceptor): the cap comes before it.
    crate::lazy_runtime::cap_workers(SERVICE_WORKER_CAP);
    // The blocking jobs of the service are held to the same cap: one service runs per worktree, so the pool is bounded as its workers
    //  are. A budget that is already made is not resized, so the call comes before the service's first job.
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(SERVICE_WORKER_CAP);
    // A signal that ends the service (the manager stops a unit with `SIGTERM`) leaves one line with the signal and the uptime.
    crate::service_signal::install();
    // The push messages this process writes name the remedies that reach the service, not a terminal's variable.
    crate::push_deadline_advice::mark_service_process();
    let address: Bind = match (option(arguments, "--abstract"), option(arguments, "--path")) {
        (Some(name), _) => Bind::Abstract(name),
        (None, Some(path)) => Bind::Path(PathBuf::from(path)),
        (None, None) => Bind::Inherited,
    };
    // A person who typed `git monochromatic-git-policy-cli service run` has no socket from the manager and named no address: the refusal comes before
    //  anything is read, created or bound, so no stray server and no stray directory is left.
    if matches!(address, Bind::Inherited) && !crate::service_server::socket_passed_in() {
        crate::service_error!(
            "monochromatic-git-policy-cli service run is started by the service manager, which passes it the listening socket of one worktree's commit service; no socket was passed in and no address was named, so nothing was started. Nothing needs starting by hand: the first commit in a worktree sets the service up. For a development run name an address with --path <socket> or --abstract <name>"
        );
        return EXIT_NOT_LISTENING;
    }
    let Some(journal) = option(arguments, "--journal").map(PathBuf::from) else {
        crate::service_error!("--journal <directory> is required");
        return EXIT_REFUSED;
    };
    let git: PathBuf = match option(arguments, "--git") {
        Some(named) => PathBuf::from(named),
        None => PathBuf::from("/usr/bin/git"),
    };
    if git_is_wrapper(git.as_path()) {
        crate::service_error!(
            "--git {} is the monochromatic-git-policy-cli executable itself, a copy of it or a toolchain launcher of it, so every Git the service started would start monochromatic-git-policy-cli again; pass the real Git executable (for example /usr/bin/git), or remove the unit that names this path and let the next monochromatic-git-policy-cli command write it again",
            git.display()
        );
        return EXIT_REFUSED;
    }
    if let Err(error) = std::fs::create_dir_all(&journal) {
        crate::service_error!("cannot create the journal directory {}: {error}", journal.display());
        return EXIT_REFUSED;
    }
    let listener = match bind(&address) {
        Ok(listener) => listener,
        Err(error) => {
            crate::service_error!("cannot listen: {error}");
            return EXIT_NOT_LISTENING;
        }
    };
    let context: LandContext = LandContext {
        git: GitContext { real_git: git, overlay: crate::child_environment::service_child_overlay(std::env::vars_os().collect::<Vec<_>>().as_slice()), global_prefix: Vec::new(), common_dir: None },
        journal,
        on_objects_written: None,
        // A control gives a short ceiling with `--lock-ceiling-millis`; a service the manager starts never passes it.
        lock_ceiling: number_option(arguments, "--lock-ceiling-millis").map_or(crate::wait_ceilings::LANDING_LOCK_CEILING, Duration::from_millis),
    };
    let idle: Duration = number_option(arguments, "--idle-seconds").map_or(IDLE_EXIT, Duration::from_secs);
    let mut settings: ServeSettings = ServeSettings::new(idle);
    if let Some(release) = option(arguments, "--release") {
        settings.release = release;
    }
    // The unit instance names the worktree this service is for; a message about any other worktree is refused.
    settings.instance = option(arguments, "--instance");
    // Every later entry carries the release and the instance.
    crate::service_timing::set_context(settings.release.as_str(), settings.instance.as_deref());
    // Before the recovery of the batches an earlier service left open, so the time from this line to `ready` is that recovery.
    log(started_line(settings.release.as_str(), PROTOCOL_VERSION, settings.instance.as_deref()).as_str());
    // The single service of the earlier releases kept one journal above every instance's own: this worktree's open batches in
    //  it are finished before the first connection is accepted.
    if let (Some(instance), Some(shared)) = (settings.instance.as_deref(), context.journal.parent()) {
        let earlier: LandContext = LandContext { git: context.git.clone(), journal: shared.to_path_buf(), on_objects_written: None, lock_ceiling: context.lock_ceiling };
        let _reports = recover_instance(&earlier, instance);
        // Nothing reads that journal's finished batches any more; the newest are kept and the rest go, so it stops growing.
        let _pruned = prune_done(shared, DONE_RETAINED);
    }
    crate::service_log!("ready");
    // The repository this service served before warms its compiled rules now, on the blocking pool, so an upgrade's new cache is
    //  filled before the first commit that would compile it.
    crate::service_rules_precompile::warm_remembered(context.journal.as_path());
    serve(listener, &context, &settings);
    return 0;
}

/// Mode controls stay out of the release executable.
#[cfg(test)]
#[path = "service_main_tests.rs"]
mod tests;
