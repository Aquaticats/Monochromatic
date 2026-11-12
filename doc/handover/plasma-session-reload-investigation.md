# Continue the desktop-session reload investigation

## Checkpoint at 2026-11-12T08:00:00-05:00

The user asked to dig fully into both open-source components,
requested a saved checkpoint before a reboot,
and resumed the investigation after it.
The user said no Claude Sonnet or Kimi K3.1 Pro audits were needed this session;
none was started.
The user asked for issues in this repository;
upstream filing was not authorized.

The canonical evidence,
source excerpts,
and every completed reproduction are in
[the incident investigation](../troubleshooting/plasma-session-ended-after-service-manager-timeouts.md).
The wrapper issue is [GitHub issue 6286](https://github.com/Aquaticats/Monochromatic/issues/6286).

## Current state

The wrapper asks the shared user manager for one global reload inside the commit path.
A reload re-reads every unit of the user manager,
and on this host it routinely outlasts the three-second subscription deadline the wrapper
itself uses:

- The retained journal holds 208 reloads.
- Their median duration is 1635 ms.
- 40 are longer than 3000 ms and 12 longer than 10000 ms.
- The longest,
  the incident's own,
  is 139770 ms.
- The requesters are the wrapper (161),
  `git` (41),
  and `systemctl` (6).

When a reload keeps the manager from processing messages past a subscription's deadline,
sd-bus expires an answer the bus had already sent,
`signal_name_owner_changed_install_handler` reads that expiry as a lost bus name,
and a running `Type=dbus` unit is stopped.
That is the desktop session's end,
with no GPU reset anywhere in the record.

An automated harness now reproduces the whole sequence in one run:
the manager alone is held inside its reload,
the bus stays responsive and still reports the service's owner,
the manager expires the captured successful subscription reply,
and the service and its bound target end.
The control arm keeps the same process.
Artifacts:
`${HOME}/temp/agent/systemd-manager-delay.2LP6eZ` (delayed,
packaged `262.1-1`),
`${HOME}/temp/agent/systemd-manager-delay.WSdsKT` (control,
packaged `262.2-1-arch`),
and the same arms against a build of the clone's `v262.1`.

A candidate repair was verified in the disposable clone only.
With it,
the delayed arm keeps the service and target,
the control keeps them,
and a unit whose bus really goes away still ends.
Upstream `main` (`4c213bab55c53682e6e3af6353bd201a4b27ac2a`) and released `v262.2`
carry the unmodified callbacks.
Nothing is deployed in the host or the repository.

## Open work

- Decide and implement the wrapper remedy for issue 6286:
  a commit must not ask the shared manager for a reload that outlives a client's patience,
  and a three-second greeting silence must not be read as a service stopped inside its own process.
- Measure whether alternating wrapper executables force repeated global reloads.
  The shipped template embeds the absolute toolchain path of whichever executable ran the commit
  (`package/git-policy/cli/src/native/service/service_provision.rs`,
  `unit_texts` and `ask_manager`),
  so two releases or a worktree build under
  `.cache/monochromatic-git-policy-cli-bin` can overwrite one shared file back and forth.
  A sampled watch of the template and the reload records is the cheapest first fixture;
  a fifteen-minute watch on 2026-11-12 found the template unchanged and the journal holding no reload,
  so the fixture has to provoke the alternation rather than wait for it.
- Break down a slow reload's phases in a disposable user manager with a representative unit set:
  unit enumeration,
  environment and unit generators,
  deserialization,
  and job and unit re-loading.
  The incident's journal has no phase timings,
  so no share may be assigned to storage,
  generators,
  or another component without a measurement.
- Decide whether the repair's own gap matters:
  it leaves the manager without tracking a name whose subscription it never installed,
  until the next reload installs it again.
  A complete repair re-arms the subscription instead.

## Reconstructible inputs

The retained reload timestamps,
caller names,
process IDs,
and completion durations are saved in
[`plasma-session-reload-evidence.json`](../troubleshooting/plasma-session-reload-evidence.json).
The evidence file omits journal cursor and machine identifiers.

Unrelated documentation edits that were already uncommitted were inspected and snapshotted in
[`plasma-reload-reboot-uncommitted.patch`](plasma-reload-reboot-uncommitted.patch).
Their original files are still unchanged and still uncommitted;
the patch reverse-applies to them.

Systemd baseline is `v262.1`,
commit `73612ea502d53baf9b0df4336cfc296a7ee0d524`.
Upstream `main` is `4c213bab55c53682e6e3af6353bd201a4b27ac2a`.
The clone has also fetched `v262.2`,
which carries the same two callbacks.

The clone is `${HOME}/temp/agent/systemd-reload-prototype.KJTwzaK2`.
Its `build-baseline` directory holds the built manager,
and `src/core/unit.c` holds the candidate repair (two early returns for transient failures).
The candidate is uncommitted in the clone;
`git --no-pager diff` there is the whole change.

The harness,
its fixture files,
and the retained probes are under `${HOME}/temp/agent/plasma-reload-probe-2026-11-12`:
`reproduce-manager-delay.ts` (one arm),
`run-harness-arms.ts` (control,
delayed,
and bus-gone arms),
and `watch-template-rewrites.ts` (the sampled wrapper-template watch).

The manager under test must be mounted read-only at `/probe-build` and reached through
`LD_LIBRARY_PATH=/probe-build/src/core:/probe-build/src/shared:/probe-build/src/basic`,
because a Meson build links the manager against its build tree.
The build image is `localhost/systemd-reload-build-tools:20261112`;
`meson` is reached through `--entrypoint=/usr/bin/meson`.

The original agent run folder is
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

The wrapper's live template is `/run/user/1000/systemd/user/monochromatic-git-policy-cli@.service`;
it names
`/var/home/user/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-git-policy-cli/0.6.1-alpha.12/...`
as the program.

Do not run a diagnostic reload against the live desktop manager.
Keep repository experiments in disposable checkouts and their own homes and runtime directories,
and use the guarded `git` command.
