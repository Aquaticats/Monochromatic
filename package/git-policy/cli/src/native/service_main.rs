//! What:
//!  The executable of the resident commit service.
//! Why:
//!  A separate program, never started by the wrapper: the operating system starts it on demand and hands it the
//!  listening socket (socket activation). Development and test runs name an address instead.
//!  Options: `--journal <directory>` (required), `--git <path>` (default `/usr/bin/git`), and one of
//!  `--abstract <name>` or `--path <socket>`; with neither, the socket comes from the service manager.

use git_policy_cli::service_land::LandContext;
use git_policy_cli::service_server::{Bind, IDLE_EXIT, bind, serve};
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
    };
    let idle: Duration = option(&arguments, "--idle-seconds")
        .and_then(|text| { return text.parse::<u64>().ok(); })
        .map_or(IDLE_EXIT, Duration::from_secs);
    eprintln!("cli-git-service: ready");
    serve(listener, &context, idle);
}
