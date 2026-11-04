//! What:
//!  The executable of the resident commit service.
//! Why:
//!  A separate program, never started by the wrapper: the operating system starts it on demand and hands it the
//!  listening socket (socket activation). Development and test runs name an address instead.
//!  Options: `--journal <directory>` (required), `--git <path>` (default `/usr/bin/git`), and one of
//!  `--abstract <name>` or `--path <socket>`; with neither, the socket comes from the service manager. Test runs may also
//!  give `--idle-seconds <n>` and `--release <text>` (the release announced to clients; default this build's). The unit
//!  instance of one worktree passes `--instance <name>` (`service_instance.rs`), and that service refuses messages about
//!  any other worktree; without it a development run serves any.

use git_policy_cli::service_journal::{DONE_RETAINED, prune_done};
use git_policy_cli::service_land::{LandContext, recover_instance};
use git_policy_cli::service_protocol::PROTOCOL_VERSION;
use git_policy_cli::service_timing::{log, started_line};
use git_policy_cli::service_server::{Bind, IDLE_EXIT, ServeSettings, bind, serve};
use git_policy_cli::transaction_git::GitContext;
use std::path::PathBuf;
use std::time::Duration;

fn option(arguments: &[String], name: &str) -> Option<String> {
    return arguments.iter().position(|argument| { return argument == name; }).and_then(|at| { return arguments.get(at + 1).cloned(); });
}

fn main() {
    // Every Git the service starts (landing, index, push, upkeep) runs with hooks off, as the wrapper's own starts do: a hook
    //  named by a repository's configuration is the person's, run by their own Git, never by the service.
    git_policy_cli::hooks_off::activate();
    // A signal that ends the service (the manager stops a unit with `SIGTERM`) leaves one line with the signal and the uptime.
    git_policy_cli::service_signal::install();
    // The push messages this process writes name the remedies that reach the service, not a terminal's variable.
    git_policy_cli::push_deadline_advice::mark_service_process();
    let arguments: Vec<String> = std::env::args().collect();
    // The internal flusher mode (plan step 2, owner decision 22): a transient unit the user manager starts runs the receipt flusher on one
    //  lane and exits with its code. It listens to nothing and serves nothing; without the flag this program is the service as before.
    if let Some(lane) = option(&arguments, git_policy_cli::commit_flusher_unit::FLUSH_LANE_FLAG) {
        if let Err(reason) = git_policy_cli::commit_flusher_unit::validate_lane(std::path::Path::new(&lane)) {
            git_policy_cli::service_error!("--flush-lane {lane}: {reason}");
            std::process::exit(2);
        }
        std::process::exit(git_policy_cli::commit_flusher::flusher_entry(std::path::Path::new(&lane)));
    }
    let Some(journal) = option(&arguments, "--journal").map(PathBuf::from) else {
        git_policy_cli::service_error!("--journal <directory> is required");
        std::process::exit(2);
    };
    let git: PathBuf = option(&arguments, "--git").map_or_else(|| { return PathBuf::from("/usr/bin/git"); }, PathBuf::from);
    // A launcher of the wrapper named as Git would make every Git start of the service start the wrapper (a unit written by an
    //  earlier release can hold such a path); the service stops at once and says what to pass instead.
    if git_policy_cli::real_git_candidate::is_wrapper_launcher(git.as_path(), None) {
        git_policy_cli::service_error!(
            "--git {} is a toolchain launcher of cli-git, so every Git the service started would start cli-git again; pass the real Git executable (for example /usr/bin/git), or remove the unit that names this path and let the next cli-git command write it again",
            git.display()
        );
        std::process::exit(2);
    }
    let address: Bind = match (option(&arguments, "--abstract"), option(&arguments, "--path")) {
        (Some(name), _) => Bind::Abstract(name),
        (None, Some(path)) => Bind::Path(PathBuf::from(path)),
        (None, None) => Bind::Inherited,
    };
    if let Err(error) = std::fs::create_dir_all(&journal) {
        git_policy_cli::service_error!("cannot create the journal directory {}: {error}", journal.display());
        std::process::exit(2);
    }
    let listener = match bind(&address) {
        Ok(listener) => listener,
        Err(error) => {
            git_policy_cli::service_error!("cannot listen: {error}");
            std::process::exit(3);
        }
    };
    let context: LandContext = LandContext {
        git: GitContext { real_git: git, overlay: git_policy_cli::child_environment::lockfile_pid_overlay(std::env::vars_os().collect::<Vec<_>>().as_slice()), global_prefix: Vec::new(), common_dir: None },
        journal,
        on_objects_written: None,
        // A control gives a short ceiling with `--lock-ceiling-millis`; a service the manager starts never passes it.
        lock_ceiling: option(&arguments, "--lock-ceiling-millis").and_then(|text| { return text.parse::<u64>().ok(); }).map_or(git_policy_cli::wait_ceilings::LOCK_CEILING, Duration::from_millis),
    };
    let idle: Duration = option(&arguments, "--idle-seconds")
        .and_then(|text| { return text.parse::<u64>().ok(); })
        .map_or(IDLE_EXIT, Duration::from_secs);
    let mut settings: ServeSettings = ServeSettings::new(idle);
    if let Some(release) = option(&arguments, "--release") {
        settings.release = release;
    }
    // The unit instance names the worktree this service is for; a message about any other worktree is refused.
    settings.instance = option(&arguments, "--instance");
    // Every later entry carries the release and the instance.
    git_policy_cli::service_timing::set_context(settings.release.as_str(), settings.instance.as_deref());
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
    // The lane switch (plan step 3): only a service started with `--lane <dir>` sweeps and serves that lane's receipts; the shipped unit files
    //  do not pass it, so the shipped service touches no lane. The loop runs on a thread of its own and ends with the process.
    if let Some(lane) = option(&arguments, "--lane").map(PathBuf::from) {
        if let Err(reason) = git_policy_cli::commit_flusher_unit::validate_lane(&lane) {
            git_policy_cli::service_error!("--lane {}: {reason}", lane.display());
            std::process::exit(2);
        }
        let interval: Duration = option(&arguments, "--lane-interval-millis").and_then(|text| { return text.parse::<u64>().ok(); }).map_or(git_policy_cli::commit_lane_service::SYNC_INTERVAL, Duration::from_millis);
        if let Err(error) = git_policy_cli::commit_lane_service::start_lane_service(&lane, interval) {
            git_policy_cli::service_error!("--lane {}: {error}", lane.display());
            std::process::exit(2);
        }
    }
    git_policy_cli::service_log!("ready");
    serve(listener, &context, &settings);
}
