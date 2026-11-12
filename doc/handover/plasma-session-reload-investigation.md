# Continue the desktop-session reload investigation

## Checkpoint at 2026-11-12T09:59:14+00:00

The user asked to dig fully into both systemd and the Git wrapper,
requested a saved checkpoint before reboot,
and then explicitly resumed the investigation after reboot.
The user said no Claude Sonnet or Kimi K3.1 Pro audits were needed this session.
None was started.

The canonical evidence,
source excerpts,
and completed reproduction are in
[the incident investigation](../troubleshooting/plasma-session-ended-after-service-manager-timeouts.md).
The wrapper issue is [GitHub issue 6286](https://github.com/Aquaticats/Monochromatic/issues/6286).

## Current state

The wrapper requested a user-manager reload at 01:22:34 EST on 2026-11-12.
It lasted 139770 ms.
KWin's D-Bus subscription then timed out,
the desktop targets stopped,
and applications lost their display connections.
No GPU reset or ring timeout was recorded for the incident.

A rootless container with systemd `262.1-1-arch` reproduced termination of a running D-Bus service
and its bound target when its subscription timed out during reload.
The control that paused the bus without a reload preserved both.
That container was stopped and removed.
No host lifecycle fix or source patch has been deployed.

The original agent transcript has now supplied the wrapper's three-second no-greeting restart diagnostic.
The incident document preserves it verbatim.
The wrapper inferred a stuck process from the missing greeting;
the actual blocking operation remains unproven.

## Open work

- Trace the exact wrapper recovery branch using the original command,
  saved session,
  and incident-version source.
  Distinguish a preparation commit printed to stdout from a commit that reached the branch.
- Automate the now-reproduced delayed-manager case,
  combining a successful subscription reply observed on the bus with the final service-state assertion.
  The first manager-only pause reproduced the shutdown with a responsive bus.
  The subsequent monitored run reached its container lifetime limit before its final check and is excluded.
- Compare current upstream and maintained systemd releases,
  then prototype and verify a repair in a disposable clone.
  Preserve detection of real bus-name loss and startup failure;
  merely ignoring the subscription error is not a verified repair.
- Exercise the wrapper's shared service template with alternating executable paths in an isolated user manager.
  Determine whether otherwise independent worktrees repeatedly overwrite the template and reload the manager.
- Break down reload cost using source instrumentation and representative isolated inputs.
  The original journal has no phase timings;
  do not assign the 139770 ms to storage,
  generators,
  or another component from coincidence.
- Update issue 6286 and file separate repository issues only for independently established defects.
  Upstream filing has not been authorized or performed.

## Reconstructible inputs

The retained reload timestamps,
caller names,
process IDs,
and completion durations are saved in
[`plasma-session-reload-evidence.json`](../troubleshooting/plasma-session-reload-evidence.json).
The evidence file omits journal cursor and machine identifiers.

Unrelated documentation edits that were already uncommitted were inspected and snapshotted in
[`plasma-reload-reboot-uncommitted.patch`](plasma-reload-reboot-uncommitted.patch).
Their original files were left unchanged.
Before applying that recovery patch after reboot,
compare it with the current worktree and history:
the originating work may have committed meanwhile.

Systemd baseline is `v262.1`,
commit `73612ea502d53baf9b0df4336cfc296a7ee0d524`.
The inspected upstream `main` was `4c213bab55c53682e6e3af6353bd201a4b27ac2a`.
Fetch them again if local scratch is unreadable after reboot.

The local baseline clone was `${HOME}/temp/agent/systemd-plasma-reload-2026-11-12`.
The fresh,
unmodified prototype clone was `${HOME}/temp/agent/systemd-reload-prototype.KJTwzaK2`.
The existing fixture inputs and driver were under `${HOME}/temp/agent/plasma-reload-probe-2026-11-12`.
The driver and test units are retained in the incident document.

The original agent run folder was
`${HOME}/temp/agent/pi-runs/subagent-driver-a-real-batch-start-works-and-the-remaining-commands-are-usable-1`.
Its `summary.json` names the full session file;
its `feed.jsonl` truncates tool results.
The session filename is
`2026-11-12T05-58-16-907Z_01a1ce55-e049-76d8-9fc0-fe8eb6f98f8e.jsonl`.
Relevant results are between `2026-11-12T06:19:00Z` and `2026-11-12T06:27:00Z`.
The original command began at `2026-11-12T06:21:10.236Z` through the `process` tool,
not the `bash` tool.
Its release is `0.6.1-alpha.12`,
source commit `418ef369a6bb34e829ea90a50c64d9bb5d353d1a`.

The post-reboot host kernel is `7.3.3-ogc1.1.fc45.x86_64`;
the systemd and KDE package versions did not change.
The prototype clone has also fetched `v262.2`.
Its changes in the relevant files do not alter the failure callback or timeout-before-reply order,
but that release has not yet been exercised in the reproducer.

The build-dependency container `systemd-reload-build-deps-20261112` installed Meson,
Ninja,
and related dependencies,
but pacman hooks failed under its inner sandbox.
Use the already documented repair in
[`pacman-hook-sandbox-rootless-podman-build.md`](../troubleshooting/pacman-hook-sandbox-rootless-podman-build.md)
before treating a derived image as ready.
The current build source remains unmodified.

The successful probe used `localhost/monochromatic/nested-wayland-session:latest`
with its entrypoint changed to `/usr/lib/systemd/systemd`,
no network or host bind mounts,
and explicit CPU,
memory,
task,
and I/O limits.
The container needed the documented `dbus-daemon` override and a policy allowing the test bus name.
It did not exercise the host's live manager.

The image has GDB and GCC,
but `meson` and `ninja` were not found on its `PATH`.
The earlier [scope investigation](../troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md)
contains a working systemd build recipe and the unambiguous Meson target `./systemd:executable`.

Do not run a diagnostic reload against the live desktop manager.
Keep repository experiments in disposable checkouts and their own homes and runtime directories,
and use the guarded `git` command.
