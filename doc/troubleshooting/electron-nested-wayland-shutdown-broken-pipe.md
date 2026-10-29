# Electron 43 under nested-wayland-session: quit prints a Fatal Wayland broken-pipe error and a Failed-to-shutdown FATAL; benign in passing runs, with one unreproduced nonzero-exit flake

When the nested compositor (`package/nested-wayland-session/cli`) exits on the control
socket's `quit` while hosting an Electron app,
 the app's Wayland connection dies under it and
Chromium logs a fatal-looking pair of lines during teardown.
In every reproduced case the boundary test still passes (compositor exit code 0);
 one early
run failed with a nonzero exit that did not recur across five immediate re-runs.
Recorded so future sessions do not mistake the stderr noise for a real failure,
 and so the
one flake has a trail.

## Symptom

At quit,
 on the hosted Electron app's stderr:

```txt
[pid:...] ERROR:ui/events/platform/wayland/wayland_event_watcher.cc:78] Fatal Wayland communication error: Broken pipe.
[pid:...] FATAL:electron/shell/browser/electron_browser_main_parts.cc:527] Failed to shutdown.
```

The GTK-based apps hosted the same way print a milder
`Gdk-Message: ... Lost connection to Wayland compositor.` and exit cleanly.
Observed once (first-ever run of `file-manager-electron`'s boundary test via mise):
 the whole
task failed with a nonzero exit immediately after these lines;
 five subsequent runs (three
direct,
 two via mise) all passed,
 as has every run since.

## Root cause

Partially diagnosed;
 recorded honestly as such.

What is established:
 the compositor tears down on `quit`,
 closing the hosted client's Wayland
socket;
 Chromium's Wayland event watcher treats a dead display as fatal
(`wayland_event_watcher.cc:78` in the Chromium tree bundled by electron 43.1.0) and Electron's
browser main parts then abort with `Failed to shutdown`
(`electron_browser_main_parts.cc:527`).
The boundary harness (`package/electron-infra/core/src/wayland-process.ts`,
`waitForSuccessfulExit`) judges only the COMPOSITOR's exit code;
 in passing runs that code is
0 despite the app's noisy death.

What is not established:
 the mechanism of the single nonzero-exit flake.
The plausible path is a shutdown race in which the compositor reaps the hosted client and
propagates its abort status if the client's death lands before the compositor finishes its own
quit path,
 but this was not traced in the compositor's `child.rs`,
 and the flake never
recurred to test against (0 recurrences in 5 immediate retries plus every later run).

## Verification

Environment:
 electron 43.1.0,
 nested-wayland-session 0.1.1,
 boundary test
`package/file-manager/variant-electron/src/wayland-boundary-test.ts`.

- The stderr pair appears on EVERY run,
   passing or failing:
  `mise run //package/file-manager/variant-electron:test:wayland 2>&1 | grep -E 'Broken pipe|Failed to shutdown'`
  prints both lines while the task exits 0.
- Flake reproduction attempts:
   five consecutive runs immediately after the one failure,
   all
  exit 0;
   all subsequent full-suite runs green.

## Verified workarounds

- None needed for the noise:
   treat the two lines as expected teardown output under this
  harness;
   assert on exit codes and observed state,
   never on stderr contents.
  Tradeoff:
   a real Wayland communication failure mid-test prints the same first line;
   the
  distinguishing signal is WHEN (mid-steps vs at quit) and the exit code.

## What does not work

- Grepping the log for `FATAL` as a failure signal:
   false-positives on every passing run.

## Upstream filing decision

`.out-of-scope/` was checked:
 no Electron exemption exists.
The compositor half is repo-owned,
 so the actionable half is internal.

1. Really upstream's fault?
    Split:
    Chromium aborting on a dead Wayland display during an
   externally initiated teardown is arguably correct-if-loud;
    the harness-visible flake,
    if it
   recurs,
    is about OUR compositor's exit-code propagation during quit.
2. Can upstream fix it?
    Electron/Chromium could downgrade the teardown path;
    speculative.
3. Supported use case?
    A compositor dying under a running client is not a supported Electron
   scenario anyone documents.
4. Would the repo welcome the contribution?
    Not evaluated;
    constraints 1-3 do not hold
   cleanly.
5. Will they likely fix it?
    No signal.
6. Prototyped minimal fix?
    Not applicable without a reproducible failure.

Decision:
 nothing to file externally.
Internal follow-up recorded for a future session:
 if the flake recurs,
 trace
`nested-wayland-session`'s `quit` path in `src/child.rs` for how a child abort racing the
quit affects the propagated exit code,
 and consider having `quit` always exit 0 once all
control-socket work completed.
