# nested-wayland-session 0.1.1: keystrokes sent before the hosted app maps its window are silently dropped,
plus two operational traps when driving the compositor by hand

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Cluster of footguns hit while writing keyboard-driven boundary tests against
`package/cli/nested-wayland-session` (repo-owned tool) for
`package/desktop-app/file-manager-gtk-sticky` and during manual driving of the compositor.

## Symptom

- Keystroke drop:
   the boundary test polled the app's observed-state file,
   saw the boot state,
  sent `key enter`,
   and the app never reacted (test timed out waiting for `paneCount: 2`),
  while the same `key enter` sent a few seconds after launch worked every time.
  The control socket answered `ok` for the dropped key;
   there is no error anywhere.
- AF_UNIX path length:
   placing the control socket under this session's deep scratchpad path
  failed with `OSError: AF_UNIX path too long` (client side) and,
   compositor side,
  `Error: binding control socket ...: path must be shorter than SUN_LEN`.
- Self-kill:
   `pkill -f nested-wayland-session` inside a compound shell command killed the
  invoking shell itself (exit 144),
   because `-f` matches the full command line and the shell's
  own command line contains the pattern;
   the cleanup then half-ran and left a stale state.

## Root cause

Keystroke drop:
 Wayland keyboard events are delivered to a focused surface.
Until the hosted client creates and maps its toplevel,
 the compositor has no surface to focus,
and a `key` control command synthesizes input that reaches nothing.
The `ok` response only acknowledges the injection,
 not delivery.
The window for this race is real in GTK apps:
 `file-manager-gtk-sticky` writes its first
observed state during controller construction,
 BEFORE `window.present()` completes and the
surface maps,
 so a test that gates on "state file exists" fires keys too early.
The Electron counterpart never hit it because its renderer reports state only after the page
loads inside an already-shown window.

AF_UNIX:
 kernel limit;
 `sockaddr_un.sun_path` is ~108 bytes on Linux.
The scratchpad prefix alone
(`/tmp/claude-1000/-var-home-user-Monochromatic/<uuid>/scratchpad/...`) exceeds it.

pkill:
 documented `pkill -f` semantics;
 the invoking shell's `-c` command string contains the
pattern text.

## Verification

Environment:
 nested-wayland-session 0.1.1 (release build),
 GTK app above,
 Wayland host
session.

- Drop reproduction:
   pre-fix `file-manager-gtk-sticky` (observed `ready: true` written
  pre-map) plus a test that sends `key enter` immediately after the first state read:
   times
  out deterministically (10s deadline,
   5 consecutive runs during diagnosis).
  Same binary,
   `sleep 3` before the key:
   works every time (manual driving).
- Fix verification:
   app gates `ready` on GTK `connect_map`
  (`file-manager-gtk-sticky/src/strip.rs`,
   `wire_window_map`);
   the boundary test waits for
  `ready: true` before its first key.
   Test passes repeatedly in under a second
  (`mise run //package/desktop-app/file-manager-gtk-sticky:test:wayland`).
- AF_UNIX:
   `python3 -c "import socket; s=socket.socket(socket.AF_UNIX); s.connect('<108+ byte path>')"`
  raises `AF_UNIX path too long`;
   a socket at `/tmp/agent/fms.sock` works.

## Verified workarounds

- Have the app expose an observable "surface mapped" fact and gate the first synthetic key on
  it.
   GTK:
   `window.connect_map(...)` flips the mirrored `ready` field.
  Tradeoff:
   every keyboard-driven GUI under test needs such a fact;
   a bare "process started"
  or "state file exists" signal is NOT sufficient.
- Short socket paths:
   create sockets under `~/temp/agent/`
  (`mkdir --parents "${HOME}/temp/agent"; chmod 700 "${HOME}/temp/agent"`).
   Tradeoff:
   current account home makes the prefix longer;
  use unique short basenames and verify the complete socket path remains within `SUN_LEN`.
- Process cleanup while driving manually:
   prefer asking the compositor to exit over signals
  (`quit` on the control socket);
   if a signal is unavoidable,
   match on the exact binary path
  with `pkill -f '/target/release/monochromatic-nested-wayland-session'` from a command line
  that does not itself contain that string,
   or use `pgrep` first and `kill` by pid.
  Tradeoff:
   none beyond care.

## What does not work

- Gating on the state file's existence or on `ready` written before map:
   the file appears
  while the surface still does not exist;
   keys sent then are gone.
- Sleeping a fixed delay in tests:
   works but is the flakiness generator the map-gate removes.
- Treating the control socket's `ok` as delivery confirmation:
   it confirms injection only.

## Scrolling by hand: the steps cap and the client's scaling of fast scrolls (2026-10-15)

Two traps met while driving the music player's lists with the compositor's `scroll` command for the release-build
measurements (`doc/audit/music-player-release-build-measurements-2026-10-15.md`).

- The `steps=` argument of `scroll` (and of `drag`) is limited to 1 through 1000
  (`package/cli/nested-wayland-session/src/protocol_click.rs`, `rest.number("steps", 1, (1, 1_000))`).
  A larger value answers `err scroll steps= must be from 1 to 1000`.
  To travel farther, send several `scroll` commands, each with at most 1000 steps.
- The distance a pixel scroll moves a Slint list is not the distance requested when the events are fast.
  A scroll of 50 logical pixels per event at 16 ms intervals moved the track list about 1.8 times the requested
  distance, and 100 pixels per event at 5 ms intervals moved it between about 1.1 and 2.3 times,
  varying from run to run.
  A single large jump (one `scroll` with a large DY) landed on the end of a 501-folder panel instead of the middle.
  The scaling depends on the event rate, which this repository did not measure further.

Workaround that held: reach a deep position with chunks of 50 logical pixels per event, then take a screenshot and
check the position from the picture, and verify movement with a before and after screenshot compared with
`magick compare -metric AE`.
Tradeoff: the landing row is only known from the picture, so a measurement that needs an exact row cannot use this.

## A wheel before the first frame is refused, and `windows` listed the window too early (2026-10-25)

`wheel` and `scroll` answer `err no nested client surface at wheel position X,Y` while no hosted window covers the point.
Until the app commits its first buffer no window covers any point,
which lasted 0.3 to 0.5 s after the toplevel was created for the IDE and much longer under load.
A driver that ignored the answer lost the wheel with no trace in the app's log:
50 of 276 IDE sessions,
all of them refused,
none of the 226 accepted ones lost (see `slint-flickable-wheel-jump-outruns-the-tile.md`,
"Where the early wheel is lost").

- `click` and `move` over no window answer `ok` and reach nobody;
  only `wheel` and `scroll` refuse.
- `windows` used to list a toplevel from its creation,
  so waiting for `windows` was no gate.
  It lists a toplevel only once its client has committed a buffer in the next release of the compositor
  (the change is on the branch `worktree-agent-a763037d1a881724e`;
  0.6.1 still lists it early).
- Workaround for 0.6.1:
  read the answer of `wheel` and `scroll` and send them again on `err`;
  before the first `click` or `key`,
  which never answer `err`,
  wait for the app's own "first frame drawn" fact.

## A click whose log line is not in the file yet is not a lost click (2026-10-29)

A driver sent `click` to the IDE and read the application log a fixed time later:
2 of 210 sessions had no `source pointer press` line,
and a rerun of the same reading rule (1.5 s after the reply) gave 8 of 400 on the installed IDE.
No press was dropped by the compositor or the IDE in any trial where the session outlived the latency.
A reader that ended the session about 1.5 s after the click did lose records: 7 of those 400 had no press line
even after the session ended.
With a driver that waited for the line,
3,350 of 3,350 clicks were logged,
every `click` answered `ok`,
and the client's own protocol log showed the press and the release on the wire in 1,900 of 1,900 sessions
(full account with counts: `doc/handover/slint-ide-0x.md`, section "Two of 210 press events that logged nothing").
The line was in the file up to 2.75 s late (20 of 3,350 later than 1.5 s).

- The IDE's log writer thread can sit in `write(1, ...)` in state `D` (wait channel `folio_wait_bit_common`)
  for more than a second on a busy disk,
  while the window thread handles the press at once (8 of 8 sampled late lines; a line still queued when the session ends can be lost, 1 s exit flush);
  the record exists and is not in the file yet.
- `WAYLAND_DEBUG=client` writes to the client's standard error from the window thread,
  so on the same busy disk it stalls the window thread and delays the press itself
  (31 of 32 sampled late presses, later than 450 ms, were in `write(2, ...)`).
  Keep that output on memory-backed storage,
  or leave it off when the question is timing.
- Workaround for a driver:
  wait for the line with a deadline well above 3 s,
  read the file again after the session has ended,
  and count a click as lost only when `WAYLAND_DEBUG=client` shows no `wl_pointer.button` for it.
- Positive controls that make the harness show a click that did not reach the text area:
  `click` at a point no window covers (no `wl_pointer.button` on the wire),
  and `click` over the minimap (the button reaches the client, no `source pointer press`).
  Both were seen in 8 of 8 sessions, and neither line appeared in the file within 20 s.

## Upstream filing decision

`.out-of-scope/` was checked:
 no matching exemption.
The compositor is this repository's own package,
 so "upstream" is this repo:
 no external
filing is applicable (constraint 1 resolves internally).
Improvement idea recorded for a future session instead of an issue filed elsewhere:
 the
control protocol could optionally report whether a hosted surface currently holds keyboard
focus (e.g. a `focused` query,
 or `key` answering `err no-surface`),
 which would let harnesses
gate on the compositor side instead of requiring every app to expose a mapped fact.
The kernel AF_UNIX limit and `pkill` semantics are documented platform behavior;
 nothing to
file.
