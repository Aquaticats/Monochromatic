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
    let Some(journal) = option(&arguments, "--journal").map(PathBuf::from) else {
        eprintln!("cli-git-service: --journal <directory> is required");
        std::process::exit(2);
    };
    let git: PathBuf = option(&arguments, "--git").map_or_else(|| { return PathBuf::from("/usr/bin/git"); }, PathBuf::from);
    let address: Bind = match (option(&arguments, "--abstract"), option(&arguments, "--path")) {
        (Some(name), _) => Bind::Abstract(name),
        (None, Some(path)) => Bind::Path(PathBuf::from(path)),
        (None, None) => Bind::Inherited,
    };
    if let Err(error) = std::fs::create_dir_all(&journal) {
        eprintln!("cli-git-service: cannot create the journal directory {}: {error}", journal.display());
        std::process::exit(2);
    }
    let listener = match bind(&address) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("cli-git-service: cannot listen: {error}");
            std::process::exit(3);
        }
    };
    let context: LandContext = LandContext {
        git: GitContext { real_git: git, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: None },
        journal,
        on_objects_written: None,
        index_wait: git_policy_cli::wait_limit::live_wait_limit(),
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
    // Before the recovery of the batches an earlier service left open, so the time from this line to `ready` is that recovery.
    log(started_line(settings.release.as_str(), PROTOCOL_VERSION, settings.instance.as_deref()).as_str());
    // The single service of the earlier releases kept one journal above every instance's own: this worktree's open batches in
    //  it are finished before the first connection is accepted.
    if let (Some(instance), Some(shared)) = (settings.instance.as_deref(), context.journal.parent()) {
        let earlier: LandContext = LandContext { git: context.git.clone(), journal: shared.to_path_buf(), on_objects_written: None, index_wait: context.index_wait };
        let _reports = recover_instance(&earlier, instance);
        // Nothing reads that journal's finished batches any more; the newest are kept and the rest go, so it stops growing.
        let _pruned = prune_done(shared, DONE_RETAINED);
    }
    eprintln!("cli-git-service: ready");
    serve(listener, &context, &settings);
}
