# Helium 0.20.2.1 launcher sessions intermittently freeze the entire window

## Status at 2026-11-03T01:54:00-05:00

The root cause is unconfirmed.
The user reports whole-window freezes despite the existing I/O-weight mitigations,
and says Firefox does not show the problem.
The user specifically suspects excessive `fsync` calls
and requested a comparison with ordinary Chromium.
Browser restarts are authorized.

## Symptom

The entire Helium window becomes unresponsive,
including browser controls.
This is distinct from the historical Plasma and Helium incidents in
[`bazzite-desktop-input-stalls.md`](bazzite-desktop-input-stalls.md).
No contemporaneous freeze interval has been marked in this investigation yet.

## Root cause

Unknown.
Do not attribute the freeze to storage or graphics without a matching incident capture.

The launcher is `${HOME}/.local/share/applications/helium.desktop`.
Its main `Exec` starts `${HOME}/AppImages/helium.appimage` directly.
The live service had no explicit `IOWeight`,
CPU quota,
or memory cap.
Helium reports `0.20.2.1`,
using Chromium `156.0.8078.73`.
The distribution Chromium reports the same Chromium version.

The persisted Helium flags include `enable-vulkan@1`,
`default-angle-vulkan@1`,
`skia-graphite@1`,
and `force-enable-webgpu-interop@1`.
The GPU command line contained `--gpu-recent-crash-count=3`.
The session journal records repeated GPU initialization warnings:

```text
ui/ozone/platform/wayland/gpu/wayland_surface_factory.cc:249
'--ozone-platform=wayland' is not compatible with Vulkan. Consider switching to '--ozone-platform=x11' or disabling Vulkan
```

These are leads,
not an explanation of the reported window freezes.
Graphics configuration remains unchanged while storage is measured.

## Verification

The kernel is `7.2.15-ogc1.1.fc45.x86_64`.
The NVMe scheduler is BFQ.
The first host I/O PSI snapshot had `some avg10=34.32` and `full avg10=25.91`.
That snapshot was not tied to a user-marked freeze.

A private tracefs instance records `fsync` and `fdatasync` entry and exit events
for Helium and comparison-browser threads.
The collector uses kernel timestamps,
resolves file descriptors where they remain available,
and records procfs state and host I/O pressure.
It does not attach with `ptrace`.

The diagnostic source is temporarily at
`${HOME}/temp/agent/helium-freeze-source-files/fsync-trace.ts`.
The capture directory is
`${HOME}/temp/agent/helium-fsync-capture-20261103`.
The system service is `helium-fsync-trace.service`,
bounded to a day,
one CPU,
512 MiB of memory,
64 tasks,
and `IOWeight=10`.
Each log retains a bounded set of rotations.
The kernel instance is removed on normal service shutdown.

The collector emits a single real disk flush as a positive control.
The initial parser rejected the kernel's rendered `sys_fsync(...)` syntax;
this was caught before trusting the empty count and corrected.
The first root service launch also failed with `status=203/EXEC`.
The verified `/usr/bin/env` workaround from
[`selinux-denies-system-service-exec-of-home-files.md`](selinux-denies-system-service-exec-of-home-files.md)
allowed it to start.

## Verified workarounds

No freeze fix has been verified.
The current action is evidence collection.

## What does not work

- A normal `poll` wait in a single browser-main-thread snapshot cannot establish UI responsiveness.
- Host I/O pressure alone cannot identify the operation blocking Helium.
- GPU initialization errors alone cannot identify the cause of a later window freeze.
- Comparing a populated Helium profile with a fresh Chromium profile cannot isolate browser implementation differences.

## Upstream filing decision

No upstream filing or draft is appropriate yet.

- Upstream fault is unconfirmed.
- Fixability is unknown without a captured blocking path.
- Desktop browsing is supported;
  the specific failing operation remains unknown.
- The inspected Helium `CONTRIBUTING.md` forbids AI-generated issue and pull-request descriptions.
- No maintainer decision about this incident has been established.
- There is no demonstrated cause or candidate patch to prototype.

## Next action

Validate the syscall collector with its positive control,
run matched disposable browser comparisons,
and retain incident evidence from the normal Helium session.
