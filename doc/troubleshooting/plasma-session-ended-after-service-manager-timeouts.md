# Plasma 6.8.2 session ends after service-manager timeouts while the user is away

## Incident at 2026-11-12T01:24:54-05:00

The user returned from sleeping to what looked like the lock screen.
After entering the desktop,
the applications from the previous session were gone.
The logs establish loss of the graphical session after a Git-wrapper-requested service-manager reload,
with no recorded GPU reset in the incident window.
A disposable container reproduced the same subscription-timeout diagnostic,
followed by termination of an unrelated service and its dependent target.
The reason the original reload took 139770 ms remains unknown.
The wrapper defect is tracked in [GitHub issue 6286].

## Symptom and recorded sequence

Times are local EST on 2026-11-12.
`journalctl --output=short-iso` reported:

- At 01:21:33,
  the commit service for instance `586bb5f650de27366c16146c727c8866` started.
  It was being stopped at 01:21:36 and reached `State 'stop-sigterm' timed out. Aborting.` at 01:22:11.
- At 01:22:34,
  systemd PID 5697 recorded a reload request from client PID 750614,
  named `monochromatic-g`,
  in `pi-subagent-driver-a-real-batch-start-works-and-the-remaining-commands-are-usable.service`.
- At 01:23:19,
  systemd PID 1 emitted
  `user@1000.service: Reload operation timed out. Killing reload process.`
- At 01:24:54,
  that user manager emitted `Reloading finished in 139770 ms.`
- At 01:24:54,
  systemd PID 5697 emitted
  `plasma-kwin_wayland.service: Unexpected error response on installing NameOwnerChanged signal match: Method call timed out`.
  Equivalent messages named Plasma Shell and other desktop services.
- At 01:24:54,
  that manager logged
  `Stopped target plasma-workspace.target - KDE Plasma Workspace.`,
  together with the Wayland workspace and Plasma core targets.
- At 01:24:54,
  the replacement commit service announced `release="0.6.1-alpha.12"` and `protocol=11`.
  It subsequently landed the agent's commit at 01:25:50.
- At 01:24:55,
  X11 clients reported broken connections.
- At 01:25:04,
  Wayland clients emitted
  `The Wayland connection broke. Did the Wayland compositor die?`
- At 01:25:07,
  logind recorded the old session logging out and a new login-greeter session starting.
- At 02:39:46,
  logind recorded a new user Wayland session.
  KWin started at 02:39:49.

The machine had remained booted for 6 days and 17 hours at the first probe.
The screen the user returned to was consistent with the login greeter recorded in the journal.
It was not a surviving desktop session with its applications merely hidden behind a lock screen.

## Verified failure mechanism

The service-manager timeout and target shutdown precede the display-client failures.
The recorded reload requester belonged to an agent service.
The loaded KWin service is `Type=dbus`,
owns `org.kde.KWinWrapper`,
has `TimeoutStartUSec=45s`,
and has `Restart=no`,
verified with `systemctl --user show plasma-kwin_wayland.service`.

The exact systemd `v262.1` source was cloned from `systemd/systemd` with `gh repo clone`.
The tag resolved to commit `73612ea502d53baf9b0df4336cfc296a7ee0d524`.
`src/core/unit.c:3574` defines `signal_name_owner_changed_install_handler`.
After emitting the diagnostic observed on the host,
the error branch discards the pending name lookup and reports a missing owner as a definitive signal:

```c
// systemd v262.1, src/core/unit.c:3592.
u->match_bus_slot = sd_bus_slot_unref(u->match_bus_slot);
u->get_name_owner_slot = sd_bus_slot_unref(u->get_name_owner_slot);

if (UNIT_VTABLE(u)->bus_name_owner_change)
        UNIT_VTABLE(u)->bus_name_owner_change(u, NULL, /* from_signal= */ true);
```

`src/core/service.c:5964` defines `service_bus_name_owner_change`.
For a running D-Bus service,
that combination selects the stop path:

```c
// systemd v262.1, src/core/service.c:5975.
if (s->type == SERVICE_DBUS) {
        if (IN_SET(s->state, SERVICE_RUNNING, SERVICE_RUNNING_REVALIDATING)) {
                if (!new_owner && from_signal)
                        service_enter_exited_or_stop(s);
                else
                        service_enter_running(s, SERVICE_SUCCESS);
```

`src/core/service.c:2787` defines `service_enter_exited_or_stop`:

```c
// systemd v262.1, src/core/service.c:2792.
if (s->remain_after_exit)
        service_set_state(s, SERVICE_EXITED);
else
        service_enter_stop(s, SERVICE_SUCCESS);
```

Thus the service can be stopped with a successful result rather than a crash result.
The container reproduction in Verification exercised this path and its target-lifetime consequence.
The host's `/usr/lib/systemd/user/plasma-workspace-wayland.target`
declares `BindsTo=plasma-kwin_wayland.service`.

### Wrapper operations that reach the shared manager

`package/git-policy/cli/src/native/service/service_provision.rs:295`
reloads when provisioning decides the installed templates or their loaded-state record require it:

```rust
// package/git-policy/cli/src/native/service/service_provision.rs:296
if reload {
    manager.reload()?;
}
```

`package/git-policy/cli/src/native/service/service_client_reach_service.rs:224`
also requests a reload after startup or greeting failure:

```rust
// package/git-policy/cli/src/native/service/service_client_reach_service.rs:224
if reloads == 0
    && let (Some(asking), Some(directory)) = (&config.start_watch, config.units_record_dir.as_ref())
    && (asking.reload)(asking.bus.as_path())
```

That condition does not establish that the loaded executable or unit text is stale.
`package/git-policy/cli/src/native/service/systemd_dbus.rs:340`
sends the global manager method:

```rust
// package/git-policy/cli/src/native/service/systemd_dbus.rs:340
pub fn reload(&mut self) -> Result<(), CallError> {
    return self.call_manager("Reload", ()).map(drop);
}
```

The host journal does not identify which wrapper branch selected this request.
It also contains no breakdown of the 139770 ms reload into serialization,
generators,
unit loading,
or restoration.
Do not assign that duration to a particular operation or resource without further measurement.
No wrapper or systemd implementation change was made during this investigation.

The kernel emitted AMDGPU messages such as
`VM memory stats for proc kwin_wayland(6106) task kwin_wayla:cs0(5985) is non-zero when fini`
at 01:25:05.
These messages alone do not establish a GPU crash.
The existing [desktop-stall investigation](bazzite-desktop-input-stalls.md)
documents the warning's teardown context.
Keep the session-ending incident separate from that investigation's input-stall incidents.

The retained kernel journal contained no GPU reset,
GPU fault,
or ring timeout during the overnight interval inspected.
A broader current-boot query did return AMDGPU `REG_WAIT` timeouts from November 5,
confirming that the query could retrieve driver timeout messages.
Those older messages do not establish the cause of this incident.

Memory-pressure events also exist,
but the inspected records do not identify KWin or the desktop session as an OOM victim at 01:25.
The 01:57:58 kernel OOM record names a Python process in a separate agent service.
The 02:39:48 user-slice OOM notifications must not be retroactively assigned as the cause of the earlier logout.

## Verification

Installed packages were `plasma-workspace-6.8.2-1.fc45.x86_64`,
`kwin-6.8.2-3.fc45.x86_64`,
and `systemd-262.1-1.fc45.x86_64`.
`uname --kernel-release` returned `7.2.15-ogc1.1.fc45.x86_64`.
`lspci -nnk -d ::03xx` identified AMD Navi 33 and Raphael devices using `amdgpu`.

The diagnostic commands were read-only:

```sh
# Inspect the recorded session loss and a surrounding interval.
journalctl --since '2026-11-12 01:20:00' --until '2026-11-12 01:26:00' \
  --no-pager --output=short-iso --no-hostname --truncate-newline \
  --grep='plasma|Wayland connection|Reload operation timed out|Session 2|greeter'
```

```sh
# Inspect kernel GPU and memory diagnostics across the overnight interval.
journalctl --dmesg --since '2026-11-11 14:00:00' \
  --no-pager --output=short-iso --no-hostname \
  --grep='oom|out of memory|killed process|amdgpu|drm|segfault' --case-sensitive=no
```

The working state after the user's login was `ActiveState=active`,
`SubState=running`,
and `NRestarts=0` for `plasma-kwin_wayland.service`.
`ps` showed the current KWin and Plasma Shell processes beginning around 02:39:48.
This verifies a replacement session,
not recovery of the original applications or prevention of recurrence.

### Controlled service-lifetime reproduction at 2026-11-12T03:09:13-05:00

The probe ran stock systemd `262.1-1-arch`
from the existing CachyOS-based `localhost/monochromatic/nested-wayland-session:latest` image,
with its entrypoint overridden to `/usr/lib/systemd/systemd`.
The rootless container had no network or host bind mounts,
and caps of 2 CPUs,
2 GiB RAM,
256 tasks,
and block-I/O weight 10.

The container's `dbus-broker` could not start in this environment.
The existing [scope investigation](systemd-scope-adopting-an-exiting-pid-stays-running-empty.md)
records that failure and the verified `dbus-daemon` unit override,
which this probe reused.
A bus policy explicitly allowed the fixture name `org.example.DesktopProbe`.

The fixture service was:

```ini
# Disposable container: /etc/systemd/system/desktop-probe.service
[Unit]
Description=Disposable desktop lifetime probe

[Service]
Type=dbus
BusName=org.example.DesktopProbe
ExecStart=/usr/bin/dbus-test-tool echo --system --name=org.example.DesktopProbe
TimeoutStartSec=3s
Restart=no
```

Its target was:

```ini
# Disposable container: /etc/systemd/system/desktop-probe.target
[Unit]
Description=Disposable target bound to desktop lifetime probe
Requires=desktop-probe.service
BindsTo=desktop-probe.service
```

The timeout is a controlled test input,
not a proposed production setting.
The harness read that configured timeout and paused the container's bus daemon for twice its duration.
All signals targeted the container-local bus daemon,
which was resumed in the harness's cleanup.

- An ordinary reload preserved the running service and active target.
- Pausing the bus daemon without a reload preserved the same service process and active target.
  The survival assertion passed.
- Pausing the bus daemon while reloading the manager made the service and target inactive.
  `MainPID` became zero and `Result` was `success`.
  The survival assertion failed with `ERR_ASSERTION`,
  actual `inactive`,
  expected `active`.

The failing run emitted:

```text
# Disposable container journal, 2026-11-12T08:09:13+00:00.
desktop-probe.service: Unexpected error response on installing NameOwnerChanged signal match: Method call timed out
desktop-probe.service: Deactivated successfully.
Stopped target Disposable target bound to desktop lifetime probe.
```

This reproduces the subscription-timeout shutdown mechanism.
It does not reproduce the original reload's delay,
prove that the host bus daemon itself paused,
or exercise KWin rendering.
The probe container was stopped and removed after verification.

### Retained reload history

The user-manager journal contained 208 reload requests and 208 completion records between
2026-11-11T14:00:00-05:00 and 2026-11-12T02:39:00-05:00.
Source timestamps were checked against that interval after parsing the JSON records.
The caller names were `monochromatic-g` for 161 requests,
`git` for 41,
and `systemctl` for 6.
These counts do not prove that every request used the same wrapper branch or was unnecessary.

### Surviving processes at 2026-11-12T02:47:09-05:00

The user asked whether the disconnected applications were still running.
`ps --user 1000 --format pid,ppid,lstart,stat,comm --sort=start_time`
showed a mixed outcome.

The old KWin,
Plasma Shell,
Xwayland,
CoolerControl,
terminal,
music-player,
System Monitor,
and Firefox process IDs checked were absent.
The journal explicitly recorded exits for the old terminal application units,
music player,
qBittorrent,
Fastmail's main application unit,
and other graphical applications.

Survivors included Dolphin PID 939385,
started on November 11 at 05:11:13;
Fastmail child processes 2322043 and 2322056,
started on November 7;
and Pi and Codex processes with start times preceding the incident.
These were live process states,
not zombie entries.
Process survival does not prove usable windows,
recoverable application state,
or progress of an agent's work.

## Verified workarounds

The user's login established a new desktop session.
No preventive workaround was established,
and no desktop configuration or running workload was changed during this investigation.

## What does not work

- Restricting the investigation to the minutes immediately before the question misses the 01:25 session loss.
- A query ending at 01:25:00 missed messages displayed as 01:24:54.
  JSON output showed source timestamps near 01:24:54 and journal receipt near 01:25:01.
  Include surrounding time and inspect both timestamp fields before trusting an empty query.
- `journalctl --kernel` emitted `journalctl: unrecognized option '--kernel'`.
  The installed help documents `--dmesg`,
  which was used successfully.
- Treating nearby OOM records or AMDGPU teardown warnings as a demonstrated cause exceeds the evidence.
- The first test harness collected the expected inactive state but its cleanup invocation of `podman exec ... kill`
  timed out.
  The container bus was explicitly resumed with `/usr/bin/kill`.
  The corrected harness used that explicit binary and completed cleanup in both the passing control and failing case.
- Browser verification of the systemd selection URLs did not complete:
  `agent-browser` emitted an IPC read timeout,
  a navigation load timeout,
  and then `Software caused connection abort (os error 103)` from its direct read command.
  No browser-verified external source selection link is claimed here.
  Source claims use the fetched tag,
  commit,
  paths,
  and quoted code.

## Upstream filing decision

- The wrapper's shared-manager operation and systemd's timeout-to-stop mechanism are established.
  This investigation does not allocate the whole incident to an upstream defect.
- A wrapper remedy must address its own service-lifecycle boundary.
  A systemd patch was not evaluated.
- A running D-Bus service and dependent target were exercised in a disposable manager.
  Host reload latency and KWin rendering were not reproduced.
- Upstream contribution policies were not investigated;
  the user requested issues in this repository.
- Upstream searches for `"NameOwnerChanged" "timed out"` and `reload dbus services killed`
  returned no issues on 2026-11-12.
  That narrow result does not establish that no related report exists.
- No source fix was prototyped or deployed.

[GitHub issue 6286] records the wrapper defect,
the incident evidence,
the controlled reproduction,
and the remaining unknowns.
No message was posted to systemd's tracker.

[GitHub issue 6286]: https://github.com/Aquaticats/Monochromatic/issues/6286
