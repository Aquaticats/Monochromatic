# Plasma 6.8.2 session ends after service-manager timeouts while the user is away

## Incident at 2026-11-12T01:24:54-05:00

The user returned from sleeping to what looked like the lock screen.
After entering the desktop,
the applications from the previous session were gone.
The logs establish loss of the graphical session,
with no recorded GPU reset in the incident window.
The reload requester was recorded,
but the reason the reload and bus subscriptions timed out remains unknown.

## Symptom and recorded sequence

Times are local EST on 2026-11-12.
`journalctl --output=short-iso` reported:

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

## Root cause boundary

The service-manager timeout and target shutdown precede the display-client failures.
The recorded reload requester belonged to an agent service.
This establishes a session-ending sequence but does not prove why the manager timed out,
or establish an upstream defect.
The user subsequently requested investigation of the underlying cause.
No source patch has been applied.

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

## Upstream filing decision

- Upstream fault is unproven:
  the initiating event was not established.
- Fixability cannot be assessed without that cause.
- The installed packages provide a supported desktop session,
  but no reproducible failing input was isolated.
- Contribution policies were not investigated because no attributable upstream defect was established.
- Maintainer intent and duplicate reports were not investigated for the same reason.
- No source fix was prototyped.

No upstream filing or fileable draft is supported by these incident records.
Further causal investigation would need to trace the recorded agent-service reload request
and why that reload and the subsequent bus subscriptions timed out.
