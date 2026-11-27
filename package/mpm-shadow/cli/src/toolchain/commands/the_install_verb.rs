//! What:
//! `mpm-shadow install` (alias `i`):
//! the tools the command line names are added to the plan first,
//! then the verb either writes or checks the lock (`--lockfile-only`)
//! or installs every tool the lock names that is not at its version.
//! Why:
//! pnpm's `install` is the verb this mirrors,
//! and it is the only way a new tool is declared here (`pnpm i -g <package>`):
//! an id the plan already holds is installed as it is and the plan is not written at all.
//! Freezing follows pnpm's rules,
//! frozen when asked or by default in CI,
//! and `--lockfile-only` never installs a tool,
//! so a frozen check writes nothing and needs no network.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function installVerb(matches: ParsedArgs, scope: Scope, env: EnvMap): number
//! ```

/// Imports the `clap` type for reading a parsed command line.
use clap::ArgMatches;

/// Imports `is_ci` from `super::super::ci_detect`.
use super::super::ci_detect::is_ci;
/// Imports what this module uses from `super::super::context`.
use super::super::context::{
    Context,
    Host,
};
/// Imports `connect_or_explain` from `super::super::http`.
use super::super::http::connect_or_explain;
/// Imports what this module uses from `super::super::install_argument`.
use super::super::install_argument::{
    AddRefusal,
    AddReport,
    add_named_tools,
    nothing_stops_the_run,
};
/// Imports what this module uses from `super::super::lock_cmd`.
use super::super::lock_cmd::{
    LockMode,
    run_lock,
};
/// Imports `ProcessRunner` from `super::super::runner`.
use super::super::runner::ProcessRunner;
/// Imports what this module uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports what this module uses from `super::super::sync`.
use super::super::sync::{
    FrozenReason,
    LockPolicy,
    run_lockfile_only,
    run_sync_with_named,
};
/// Imports the lines `install` prints for a sync.
use super::super::sync_report_text::{
    format_sync_report,
    locked_line,
    rewritten_line,
};
/// Imports what this module uses from `super::what_every_verb_prints`.
use super::what_every_verb_prints::{
    print_lock_outcome,
    report,
    warn_older_locks,
};

/// What:
/// Runs `install` with the arguments `matches` in `scope` and returns the exit code.
/// Why:
/// The plan is written before the lock,
/// so a run killed in between leaves the new entry with no lock row,
/// which is the state a plain `install` finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installVerb(matches: ParsedArgs, scope: Scope, env: EnvMap): number
/// ```
pub(super) fn install_verb(
    matches: &ArgMatches,
    scope: &Scope,
    env: &EnvMap,
) -> i32 {
    let frozen_flag: bool = matches.get_flag("frozen-lockfile");
    let lockfile_only: bool = matches.get_flag("lockfile-only");
    let offline: bool = matches.get_flag("offline");
    // pnpm: frozen when asked, or by default in CI; `--no-frozen-lockfile` turns it off.
    let policy: LockPolicy = if frozen_flag {
        LockPolicy::Frozen(FrozenReason::Flag)
    } else if matches.get_flag("no-frozen-lockfile") || !is_ci(env) {
        LockPolicy::Resolve
    } else {
        LockPolicy::Frozen(FrozenReason::Ci)
    };
    let arguments: Vec<String> = match matches.get_many::<String>("tool") {
        Some(given) => given.cloned().collect(),
        None => Vec::new(),
    };
    let http = connect_or_explain();
    let host: Host = Host::current();
    let mut named: Vec<String> = Vec::new();
    if !arguments.is_empty() {
        let refusal: Option<AddRefusal> =
            if lockfile_only && matches!(policy, LockPolicy::Frozen(_)) {
                Some(AddRefusal::FrozenCheck)
            } else if offline {
                Some(AddRefusal::Offline)
            } else {
                None
            };
        let context: Context = Context {
            scope,
            env,
            runner: &ProcessRunner,
            http: http.as_ref(),
            host: &host,
            tools: &[],
        };
        let additions: AddReport =
            match add_named_tools(&context, &arguments, refusal, nothing_stops_the_run) {
                Ok(report) => report,
                Err(error) => return report(&error),
            };
        for declaration in &additions.added {
            println!(
                "toolchain install: added {} to {}: request {}, route {}, bins {}",
                declaration.id,
                scope.plan_file.display(),
                declaration.request.text(),
                declaration.route.as_str(),
                declaration.bins.join(", ")
            );
            println!(
                "toolchain install: no comment says why {} is there; write one beside its entry in {}",
                declaration.id,
                scope.plan_file.display()
            );
        }
        named = additions.added_ids();
    }
    if lockfile_only {
        // A frozen lock check writes nothing and needs no network.
        if matches!(policy, LockPolicy::Frozen(_)) && scope.lock_file.exists() {
            return print_lock_outcome("install", scope, run_lock(scope, LockMode::Check));
        }
        if offline {
            return print_lock_outcome("install", scope, run_lock(scope, LockMode::Write));
        }
        return match run_lockfile_only(scope, env, &ProcessRunner, http.as_ref(), &host, policy) {
            Ok(prepared) => {
                for (id, version) in &prepared.resolved {
                    print!("{}", locked_line(id, version));
                }
                for description in &prepared.rewritten {
                    print!("{}", rewritten_line(description));
                }
                if prepared.resolved.is_empty() && prepared.rewritten.is_empty() {
                    println!(
                        "toolchain lock already current: {}",
                        scope.lock_file.display()
                    );
                } else {
                    println!(
                        "toolchain install: wrote {}; nothing was installed",
                        scope.lock_file.display()
                    );
                }
                warn_older_locks("install", scope);
                0
            }
            Err(error) => report(&error),
        };
    }
    return match run_sync_with_named(
        scope,
        env,
        &ProcessRunner,
        http.as_ref(),
        &host,
        policy,
        &named,
    ) {
        Ok(sync_report) => {
            println!(
                "toolchain install: platform {}, root {}, exposure {}",
                host.platform,
                scope.toolchain_root.display(),
                scope.exposure_dir.display()
            );
            print!("{}", format_sync_report(&sync_report));
            i32::from(!sync_report.failed.is_empty())
        }
        Err(error) => report(&error),
    };
}
