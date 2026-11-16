/**
 The git wrapper's commit service, which works inside a worktree for a while after a run's last command by design and is no work the
 run left running.

 The wrapper starts one service per worktree as a systemd user unit of its own (`monochromatic-git-policy-cli@<instance>.service`,
 and `cli-git@<instance>.service` from the release before it), and that service runs the Git processes that land a commit inside the
 worktree. Those Git processes work in the run's folder and sit in the service's control group rather than the run's unit, so a scan
 that knows only folders and the run's own unit would report them as the run's work: it would keep a finished run open for them, and
 announce them as work that ended.

 The owner decided on 2026-11-15 (`doc/handover/cli-git-rust-implementation.md`, "Owner decisions in force", "Answers of the evening,
 the long sitting"): "Do not count the commit service". So the unit a process runs in is what tells the two apart.

 A service that runs no manager refuses to start, so a Linux host with a user manager is the only place one of these units exists
 (`package/git-policy/cli/src/native/service/service_provision_host_and_manager.rs`, `platform_without_manager`).

 @module
 */

/**
 The unit names the wrapper's commit service is started under, this release's and the one before it, each with the instance's name
 after the at-sign (`package/git-policy/cli/src/native/foundation/wrapper_names_old_and_new.rs`, `COMMAND_WORD`).
 */
const COMMIT_SERVICE_UNIT_STARTS: readonly string[] = [
  'monochromatic-git-policy-cli@',
  'cli-git@',
];

/**
 The end of every commit service's unit name.
 */
const UNIT_SUFFIX = '.service';

/**
 Says whether a control group is one the git wrapper's commit service runs in.

 @param cgroup - the control group text the process folder holds for a process

 @returns whether that process is the wrapper's commit service, or one of the Git processes it runs

 @example
 ```ts
 const theirs = isTheCommitServiceIn({ cgroup: '0::/user.slice/user-1000.slice/user@1000.service/app.slice/monochromatic-git-policy-cli@0123456789abcdef.service', },);
 ```
 */
export function isTheCommitServiceIn({ cgroup, }: { readonly cgroup: string },): boolean {
  /**
   The components of the control group's path, of which the last is the unit a process runs in when it runs in one.
   */
  const components = cgroup.trim()
    .split('/',);
  /**
   The unit's name, or empty text when the control group names none.
   */
  const unit = components.at(-1,) ?? '';
  return COMMIT_SERVICE_UNIT_STARTS.some(function isTheStart(start,) {
    return unit.startsWith(start,) && unit.endsWith(UNIT_SUFFIX,);
  },);
}
