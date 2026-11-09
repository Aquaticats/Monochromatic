# The git wrapper 0.5.4 commit service fails to start after an upgrade: the provisioned unit names a removed program, `status=203/EXEC`, and the first signed commit waits ten seconds

Affected:
`monochromatic-git-policy-cli` 0.5.2 through 0.5.4 on a Linux machine with a systemd user manager,
upgraded in place (the toolchain directory of the old release is removed on install).
Fixed in the release that follows 0.5.4.
The manager is systemd 259 (`systemd[5165]`,
the user manager);
the wrapper's side is
`package/git-policy/cli/src/native/service_route.rs`,
`service_provision.rs` and `service_client.rs`.
This is a defect in the wrapper's decision,
not in systemd:
the manager did what its unit file said.

Note 2026-11-09T16:55:17-05:00:
`package/git-policy/cli/src/native/service_route.rs` is now
`package/git-policy/cli/src/native/landing/service_route.rs`,
moved 2026-11-03 in `f18b7c2464`.

## Symptom

The first commit that goes through the service after an upgrade stops after about ten seconds with,
verbatim:

```text
Starting the commit service...
The commit service did not start, so this commit was not made. Its log shows why: `journalctl --user --unit cli-git.service`.
Run the commit again after fixing that. Nothing in the repository changed.
```

The measured first failure took 10178 ms.
Release 0.5.3 hid it,
because it declined signed commits before the service;
0.5.4 sends them to the service.
The user journal (`journalctl --user --unit cli-git.service --unit cli-git.socket`),
2026-11-01,
verbatim:

```text
cli-git.service: Unable to locate executable '/var/home/user/.local/share/monochromatic-dev/toolchain/cargo/monochromatic-git-policy-cli/0.5.2/bin/cli-git-service': No such file or directory
cli-git.service: Main process exited, code=exited, status=203/EXEC
cli-git.service: Scheduled restart job, restart counter is at 5.
cli-git.service: Start request repeated too quickly.
Failed to start cli-git.service - Commit service for the git wrapper.
cli-git.socket: Failed with result 'service-start-limit-hit'.
```

The five restarts happened within two seconds (12:17:51 to 12:17:53);
the wrapper still waited its whole ten second budget.

## Root cause

Provisioning writes the unit into `$XDG_RUNTIME_DIR/systemd/user` with the absolute path of the `cli-git-service` that sits
beside the wrapper that provisioned it (`ExecStart=<toolchain>/0.5.2/bin/cli-git-service --journal ...`).
Installing a later release removes that directory.
The unit file stays,
because the runtime directory lasts until logout.
The route decision returned early whenever the manager's socket existed:

```rust
// package/git-policy/cli/src/native/service_route.rs (0.5.4)
let socket: PathBuf = runtime_dir.join(SOCKET_RELATIVE);
if socket.exists() {
    return Route::Service { address: Bind::Path(socket), provision: None };
}
```

The socket belongs to the manager (`ListenStream=%t/cli-git/service.sock`) and outlives every upgrade,
so provisioning
never ran again and the unit kept naming the removed program.
The ten seconds are `START_WAIT` in `service_client.rs`:
the connection is accepted by the manager,
no service answers with
a welcome,
and the client waits until its bound.
The wrapper had no way to tell a start that is slow from one that cannot happen.

An earlier reading,
that provisioning ran before every commit in 0.5.2 and that this was the 10.2 s wait,
is not what
the code shows:
in release 0.5.2 provisioning is gated on the route returning a host,
which only happened with no socket.
The only fixed waits in a commit's path are `START_WAIT` (client,
reached only when no welcome arrives) and the server's
`REQUEST_READ_TIMEOUT`;
a healthy start answers at the first welcome (measured with the stand-in:
a commit that replaces a
service of another release takes under one second end to end).
The user journal does not reach back to the 0.5.2 install,
so the 10.2 s first commit under 0.5.2 is not tied to a logged
cause;
the same wait is what any failed start costs.

## Fix

From the release after 0.5.4:
- Every commit that goes to the service compares the two installed unit files with what the wrapper would write (two reads
  and one `stat` of the socket;
  measured eleven system calls and 20.6 microseconds in a debug build,
  in
  `doc/planning/cli-git-resident-service.md`) before it opens the bus.
  A difference rewrites the units,
  reloads the manager,
  clears failed
  state with `ResetFailedUnit`,
  and starts the socket.
  Matching units cost no bus call.
- The service announces its release in the welcome.
  A service from another release,
  older or newer,
  is asked to
  retire (it finishes its batch and exits),
  and the next connection makes the manager start the program the rewritten unit
  names.
  The asking is bounded by the start wait;
  a service that does not retire gives a plain message with
  `systemctl --user restart cli-git.service`.

## Verified manual recovery for a machine still on 0.5.4

The commands below are the recovery that worked on 2026-11-01;
run them in a terminal,
then commit again:

```sh
# stop both units and clear failed state
systemctl --user stop cli-git.socket cli-git.service
systemctl --user reset-failed cli-git.socket cli-git.service
# delete the stale runtime units and the socket path, then reload
rm --force "${XDG_RUNTIME_DIR}/systemd/user/cli-git.socket" "${XDG_RUNTIME_DIR}/systemd/user/cli-git.service"
rm --force "${XDG_RUNTIME_DIR}/cli-git/service.sock"
systemctl --user daemon-reload
```

The next commit provisions the units for the installed release,
starts the service,
and lands (324 ms for a signed commit
in the measured run).

## What does not work

- Waiting:
  the failed units stay failed (`service-start-limit-hit`) until `reset-failed`.
- Deleting only the unit files without `daemon-reload`:
  the manager keeps the loaded unit.
- Running the commit again:
  the unit still names the removed path,
  so each attempt costs the whole wait.

## Not repairable by the wrapper

An `ExecStart` that exists but cannot be executed for another reason (a `noexec` mount,
a security policy;
compare
`doc/troubleshooting/selinux-denies-system-service-exec-of-home-files.md` for a system manager) still gives `status=203/EXEC`
and the ten second wait.
Reading the unit's exit status from the manager needs `Properties` replies with variant types,
which
the wrapper's minimal bus client (`systemd_dbus.rs`) deliberately does not read,
so the failure stays loud and clear
(`journalctl --user --unit cli-git.service`) instead of guessed.

## Upstream filing decision

Nothing to file:
the manager behaved as documented,
and the defect was in this repository.
