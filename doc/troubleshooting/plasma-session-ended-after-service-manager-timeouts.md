# Plasma 6.8.2 session ends after service-manager timeouts while the user is away

## Incident at 2026-11-12T01:24:54-05:00

The user returned from sleeping to what looked like the lock screen.
After entering the desktop,
the applications from the previous session were gone.
The logs establish loss of the graphical session after a service-manager reload requested by the Git wrapper,
with no recorded GPU reset in the incident window.
A disposable container reproduced the same subscription-timeout diagnostic,
followed by termination of an unrelated service and its dependent target.
The reason the original reload took 139770 ms remains unknown.
The wrapper defect is tracked in [GitHub issue 6286][].

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
`src/core/unit.c:3576` defines `signal_name_owner_changed_install_handler`.
After emitting the diagnostic observed on the host,
the error branch discards the pending name lookup and reports a missing owner as a definitive signal:

```c
// systemd v262.1, src/core/unit.c:3593.
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

The verified driver is retained here so the probe does not depend on scratch storage.
It requires the disposable container named `plasma-reload-timeout-20261112`,
with the fixture service already started and its bus policy installed.
Run the control without arguments;
restart the fixture service before running the failing case with `--reload-during-pause`.

```ts
// ${HOME}/temp/agent/plasma-reload-probe-2026-11-12/reproduce-timeout.ts
import { execFileSync } from 'node:child_process';
import { setTimeout } from 'node:timers/promises';
import { strict as assert } from 'node:assert';

const container = 'plasma-reload-timeout-20261112';
const commandCeilingMilliseconds = 60000;
const reloadDuringPause = process.argv.includes('--reload-during-pause');

function inside(args: string[]): string {
  return execFileSync('podman', ['exec', container, ...args], {
    encoding: 'utf8',
    timeout: commandCeilingMilliseconds,
  }).trim();
}

function states(): string {
  return inside(['systemctl', 'show', 'desktop-probe.service', 'desktop-probe.target',
    '--property=Id,ActiveState,MainPID,Result']);
}

const baseline = states();
console.log('Baseline after ordinary reload:\n' + baseline);
assert.equal(inside(['systemctl', 'show', 'desktop-probe.service', '--property=ActiveState', '--value']), 'active');
const brokerPid = Number(inside(['systemctl', 'show', 'dbus.service', '--property=MainPID', '--value']));
assert(Number.isSafeInteger(brokerPid) && brokerPid > 1, 'Expected a container-local broker PID');
const configuredTimeout = inside(['systemctl', 'show', 'desktop-probe.service', '--property=TimeoutStartUSec', '--value']);
assert.match(configuredTimeout, /^\d+s$/);
const configuredTimeoutMilliseconds = Number(configuredTimeout.slice(0, -1)) * 1000;

let actualState;
try {
  inside(['/usr/bin/kill', '--signal', 'STOP', String(brokerPid)]);
  if (reloadDuringPause) inside(['systemctl', 'daemon-reload']);
  await setTimeout(configuredTimeoutMilliseconds * 2);
  console.log('After broker response deadline during reload:\n' + states());
  console.log(inside(['journalctl', '--unit=desktop-probe.service', '--unit=desktop-probe.target',
    '--no-pager', '--output=short-iso']));
  actualState = inside(['systemctl', 'show', 'desktop-probe.service', '--property=ActiveState', '--value']);
} finally {
  inside(['/usr/bin/kill', '--signal', 'CONT', String(brokerPid)]);
}
assert.equal(actualState, 'active',
  'An unrelated service should survive a reload whose bus subscription could not be confirmed');
```

The actual invocations were:

```sh
# Host driver; every service operation is confined to the named disposable container.
node "${HOME}/temp/agent/plasma-reload-probe-2026-11-12/reproduce-timeout.ts"
node "${HOME}/temp/agent/plasma-reload-probe-2026-11-12/reproduce-timeout.ts" --reload-during-pause
```

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

## Further source findings before reboot at 2026-11-12T03:47:48-05:00

The user requested a full investigation of both open-source components,
then requested preservation and an issue update before reboot.
The open work is recorded in
[the reboot handover](../handover/plasma-session-reload-investigation.md).

### Original wrapper recovery output recovered

The original agent's saved session contains the untruncated tool result,
which its driver feed had shortened.
The tool result was itself a tail of command output,
not the complete command log.
The tool result at `2026-11-12T06:25:08.295Z` includes:

```text
# Original agent tool output, recovered from its saved session.
monochromatic-git-policy-cli: Waiting for the commit service of this worktree... nothing has answered at /run/user/1000/monochromatic-git-policy-cli/586bb5f650de27366c16146c727c8866.sock yet (the service manager says monochromatic-git-policy-cli@586bb5f650de27366c16146c727c8866.service is active); this wait ends as soon as it answers.
monochromatic-git-policy-cli: the commit service of this worktree has answered nothing for 3 seconds, so it is stopped inside its own process; it is asked to restart (monochromatic-git-policy-cli@586bb5f650de27366c16146c727c8866.service), and this commit goes on once it answers.
The commit service did not start, so this commit was not made. Its log shows why: `journalctl --user --unit 'monochromatic-git-policy-cli@*.service'`. Run the commit again after fixing that. Nothing in the repository changed.
```

This establishes that the wrapper took its no-greeting restart path before the failed recovery.
The diagnostic's claim that the service was stopped inside its own process is the wrapper's inference,
not an independently established process state.
The precise reload call site still needs correlation with the original command and incident-version source.
The output also printed preparation commit `c200b01e0f`;
that alone does not establish that this commit reached the branch.
The subsequent service log establishes landing of `c2b4d5a7fd`.

The transcript is recoverable from the session named
`2026-11-12T05-58-16-907Z_01a1ce55-e049-76d8-9fc0-fe8eb6f98f8e.jsonl`
under the Pi session directory for
`pi-subagent-driver-a-real-batch-start-works-and-the-remaining-commands-are-usable`.
The durable excerpt in this document does not depend on that local file surviving reboot.

### A delayed manager can expire replies before reading them

In the same systemd `v262.1` commit,
`src/libsystemd/sd-bus/sd-bus.c:3063` defines `process_running`.
Its processing order is:

```c
// systemd v262.1, src/libsystemd/sd-bus/sd-bus.c:3070.
r = process_timeout(bus);
if (r != 0)
        goto null_message;

r = dispatch_wqueue(bus);
if (r != 0)
        goto null_message;

r = dispatch_track(bus);
if (r != 0)
        goto null_message;

r = dispatch_rqueue(bus, &m);
```

`process_timeout` at line 2691 constructs `SD_BUS_ERROR_NO_REPLY` with `Method call timed out`
when the callback deadline has elapsed.
This identifies a concrete hypothesis:
the manager can process an expired subscription before reading an already queued successful reply.
The bus daemon need not have been stopped for that hypothesis to apply.
The completed reproduction paused the bus daemon;
the stronger reproduction that pauses only the manager has not yet been run.

The packaged container library exports `manager_reload` and `unit_install_bus_match`,
verified with `nm --dynamic`.
`manager_ready` is static.
A next probe can use GDB to break in `manager_reload`,
finish that call,
hold the manager before it returns to event processing,
and query the still-running bus independently.
This is a proposed experiment,
not a completed result.

### Source and prototype state

- Installed-version source remains systemd `v262.1`,
  commit `73612ea502d53baf9b0df4336cfc296a7ee0d524`.
- Upstream `main` resolved to `4c213bab55c53682e6e3af6353bd201a4b27ac2a` during this continuation.
  Its `src/core/unit.c` was fetched,
  but a complete comparison and maintained-release check remain open.
- A fresh disposable prototype clone was created and remained clean.
  No systemd patch or build was started.
- The wrapper's shared template embeds the calling executable and Git paths in
  `package/git-policy/cli/src/native/service/service_provision.rs:112`.
  Whether alternating executable paths cause repeated global reloads still needs a real-manager fixture.
- No Sonnet or Kimi audit was launched.
  The user explicitly waived those audits for this session.

## Manager-only delay reproduced at 2026-11-12T04:18:02-05:00

After reboot,
the host retained systemd `262.1-1.fc45`,
KWin `6.8.2-3.fc45`,
and Plasma `6.8.2-1.fc45`.
The kernel changed from `7.2.15-ogc1.1.fc45.x86_64` to `7.3.3-ogc1.1.fc45.x86_64`.
The container image was still
`75045b70216eff1bca32eef9add76c9cf02153f11d9c6dab01290ea90df4c6fd`.

The normal reload control left `desktop-probe.service` and its bound target active,
with service PID 146 and bus owner `:1.1`.
GDB then paused only the container manager after `manager_reload` returned:

```text
# Disposable container debugger commands, /probe/pause-manager.gdb.
set pagination off
set confirm off
set debuginfod enabled off
set breakpoint pending on
break manager_reload
printf "RELOAD_BREAKPOINT_ARMED\n"
continue
finish
printf "MANAGER_PAUSED_AFTER_RELOAD\n"
shell /usr/bin/timeout 120 /usr/bin/cat /probe/resume-manager
detach
quit
```

The container had `SYS_PTRACE` capability for this debugger.
Its systemd process was PID 1 inside the container,
not the host's PID 1.
While that process was held,
`ps` reported `ts` before and after a successful independent bus query:

```text
# Observations while only the disposable manager was paused.
PID STAT COMMAND
  1 ts   systemd
s ":1.1"
PID STAT COMMAND
  1 ts   systemd
```

The query was `GetNameOwner` for `org.example.DesktopProbe`.
The manager remained paused for a further 6.1109 seconds,
exceeding the fixture's 3-second method timeout.
After release through the FIFO,
GDB detached and the reload caller exited successfully.
The service and target became inactive,
the service PID became zero,
and `Result=success`.
At `2026-11-12T09:18:02+00:00`,
the container journal emitted the same subscription-timeout diagnostic and then
`desktop-probe.service: Deactivated successfully.`

This proves that a manager delay can produce the shutdown while the bus remains responsive
and the service still owns its name during the delay.
It does not identify which operation delayed the original host manager.

### Additional bus-monitor evidence and its limit

A subsequent run captured the manager's `AddMatch` request for the fixture's `NameOwnerChanged` signal,
cookie 136 from `:1.0`,
and the bus's successful `method_return` with `reply_cookie=136` destined for `:1.0`.
Both were present in a log copied while the manager was still in tracing-stop state.
The independent name-owner query returned `:1.4` in that run.

That run reached the container's 1800-second lifetime limit before the final release and state check.
Its termination is excluded as evidence for the subscription-timeout shutdown.
The next run must combine the request/reply capture and final-state assertion in an automated harness.

### Incident wrapper source identified

The original invocation was a guarded `git commit`,
started through the agent's process tool at `2026-11-12T06:21:10.236Z`.
It did not explicitly request a manager reload.
Its output was piped through `tail -20`,
and a later command read a tail of that retained output.

The incident worktree's lock selected `monochromatic-git-policy-cli` `0.6.1-alpha.12`.
The release tag resolves to `418ef369a6bb34e829ea90a50c64d9bb5d353d1a`.
Its Linux x64 archive checksum is
`b3c92d60bd3acdbb08f23f2ff8ebbb2882f5e416516727224c360632d8520667`.
That release's `service_client_reach_service.rs` contains the restart and fallback reload paths,
and `wait_ceilings.rs` sets `SERVICE_GREETING_SILENCE` to 3 seconds
and `START_CEILING` to 60 seconds.
These bounds align with the service starting at 01:21:33,
the restart at 01:21:36,
and the global reload request at 01:22:34.
The remaining call-chain review must distinguish this evidence from proof of every intermediate branch.

The current checkout's `git` link resolves to a toolchain launcher.
Its resolver and the recorded executable used by the service still need tracing;
do not assume that the launcher path is the service executable.

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

[GitHub issue 6286][] records the wrapper defect,
the incident evidence,
the controlled reproduction,
and the remaining unknowns.
No message was posted to systemd's tracker.

[GitHub issue 6286]: https://github.com/Aquaticats/Monochromatic/issues/6286
