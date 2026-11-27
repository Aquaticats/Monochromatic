//! The watcher child itself:
//! the roots it scans,
//! the policy engine it builds,
//! and the commit-gated start of its loop.

/// The words of the handshake,
/// and the account lookup it shares with the parent side.
use super::{
    COMMIT,
    COMMITTED,
    READY,
    user_install_roots,
};
/// Production target roots and app slice path.
use crate::application_targets::{
    ScanRoots,
    user_app_slice,
};
/// Watch loop preparation and execution.
use crate::application_watch::{
    ReporterIdentity,
    prepare_application_watch,
    run_application_watch,
};
/// The sweep of the scopes of the earlier design.
use crate::leftover_sweep::SweepSetup;
/// Non-panicking output.
use crate::log_line::log_line;
/// Moves matched processes into the updates slice.
use crate::scope_adoption::PrivateSocketAdopter;
/// The start notice and the refused install roots.
use crate::update_report::{
    policy_notice,
    refused_root_failures,
};
/// The policy engine and its setup.
use crate::update_routing::{
    RoutingSetup,
    UpdateRouting,
};
/// Handshake output and input.
use std::io::{
    self,
    BufRead,
    Write,
};
/// Watcher application slice path.
use std::path::{
    Path,
    PathBuf,
};

/// Runs hidden watcher child with production roots and commit-gated persistence.
pub fn run_application_watch_child(
    mark: u32,
    uid: u32,
    app_slice_override: Option<&Path>,
) -> io::Result<()> {
    let production_app_slice = user_app_slice(uid);
    let app_slice = app_slice_override.unwrap_or(&production_app_slice);
    let roots = ScanRoots {
        app_slice,
        proc_root: Path::new("/proc"),
        cgroup_root: Path::new("/sys/fs/cgroup"),
    };
    let (account, install_root_set) = user_install_roots(uid)?;
    log_line(&policy_notice(&install_root_set.roots, &account.home));
    for line in refused_root_failures(&install_root_set.refused) {
        log_line(&line);
    }
    let routing = UpdateRouting::new(RoutingSetup {
        uid,
        install_roots: install_root_set.roots,
        proc_root: PathBuf::from("/proc"),
        cgroup_root: PathBuf::from("/sys/fs/cgroup"),
        adopter: Box::new(PrivateSocketAdopter::new()),
    });
    let mut watch = prepare_application_watch(
        roots,
        mark,
        routing,
        ReporterIdentity {
            uid,
            gid: account.gid,
        },
        Some(SweepSetup::production(uid, account.gid)),
    )?;
    writeln!(io::stdout(), "{READY}")?;
    io::stdout().flush()?;
    let mut commit = String::new();
    io::stdin().lock().read_line(&mut commit)?;
    if commit.trim_end() != COMMIT {
        return Ok(());
    }
    writeln!(io::stdout(), "{COMMITTED}")?;
    io::stdout().flush()?;
    return run_application_watch(&mut watch);
}
