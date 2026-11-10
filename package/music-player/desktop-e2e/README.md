# music-player-desktop-e2e

Keyboard end-to-end runner for the desktop music player.
It hosts the release binary of `package/music-player/desktop-app` in a nested Wayland compositor,
presses every shortcut as a real key event through the compositor's control socket,
and reads what happened from the app's own `ui_state` log lines,
 from screenshots,
 from files and from the nested clipboard.

## Run it

The desktop app's task builds the release binary and then calls this package's command:

```sh
# shell
mise run //package/music-player/desktop-app:test:e2e
```

The command can also run alone against an existing release build.
It looks for `target/release/music-player` in the current directory,
 or in the directory named by `--app-dir`:

```sh
# shell, from the repository root
node package/music-player/desktop-e2e/src/cli.ts --app-dir package/music-player/desktop-app
```

The run is headless:
the compositor renders without any display,
so nothing appears on the desktop.
A person who wants to watch passes `--visible` (or sets `E2E_VISIBLE=1`),
which opens a nested window in the current graphical session:

```sh
# shell, from the repository root, in a graphical session
node package/music-player/desktop-e2e/src/cli.ts --app-dir package/music-player/desktop-app --visible
```

The output has one `PASS`,
 `FAIL` or `SKIP` line per check,
 then a table and a summary such as
`<passed> passed, 0 failed, 1 not injectable; <seconds> s`.
The command exits with a non-zero status when a check fails or a safety stop ends the run.
Run it alone:
 it refuses to start while another run of this suite is active.

## What the suite needs

- With `--visible` only,
  a graphical session (`WAYLAND_DISPLAY`) for the nested compositor to open its window in.
  A headless run needs no display and a GPU render node (see "Headless by default").
- PipeWire with `pw-dump` and `pactl`,
   a systemd user session with `systemd-run` and `systemctl`,
   `pgrep`,
   ImageMagick (`magick`),
   and `wl-paste`.
- The nested compositor `monochromatic-nested-wayland-session`,
  installed through mise (`"cargo:monochromatic-nested-wayland-session" = "latest"` in the root `mise.toml`),
  or any executable named by `E2E_COMPOSITOR`.
  The run starts the install that mise reports for `mise where cargo:monochromatic-nested-wayland-session`,
  whatever its version:
  the runner pins no version,
  and the path it uses is absolute,
  because the mise shim needs a trusted config that the compositor's systemd unit does not have.
  Before it starts,
  the run asks the compositor for `--version` and requires 0.6.0 or newer for a headless run (`FIRST_VERSION_WITH_HEADLESS_GPU_CHOICE` in `src/compositor-version.ts`,
  the first version with `--headless` together with `--gpu` and `--list-gpus`) and 0.4.0 or newer for a visible one (`FIRST_VERSION_WITH_RUNNER_CONTROLS`,
  the first version with the controls below);
  a newer version,
  including a pre-release build,
  is always accepted.
  An older version,
  or output with no version,
  is refused with a plain message that names `mise upgrade cargo:monochromatic-nested-wayland-session` and `E2E_COMPOSITOR` as the ways out.
  Version 0.4.0 added the key names the checks press (Menu,
  F10 and the other F keys,
  chords,
  `hold` with the client's key repeat),
  Unicode typing,
  exact odd sizes,
  focus return after a dialog,
  and the `windows` and `repeat-info` commands.
  One check stays recorded as not injectable:
  a held Shift with repeats cannot occur,
  because modifier keys never repeat on Wayland.

## Headless by default

Nothing a run starts shows anything on the desktop unless a person asked for it.
The compositor starts with `--headless --gpu <device>`,
and the unit that holds it gets none of the variables that name a display of the host (`WAYLAND_DISPLAY`,
`WAYLAND_SOCKET`,
`DISPLAY`,
`XAUTHORITY`):
they are left out of the unit's environment and listed in `UnsetEnvironment`,
so even a compositor started without `--headless` finds no parent display and refuses to start instead of opening a window.
A failed device probe is an error,
never a fall back to a visible window.
`src/session-launch.ts` builds the one argument vector every session runs (`sessionLaunchArguments`),
and its unit tests assert on that vector for the headless and the visible case.

The device is not fixed.
The run asks the compositor for `--list-gpus` (which opens every render node Mesa offers),
passes the node that `auto uses:` names to `--gpu`,
and falls back to the first node Mesa could open.
The software renderer is never chosen by itself,
because a start on it was slow to map a window in an earlier measurement;
a host with no usable render node fails with a message that quotes the compositor,
everything its listing printed and the ways out (`E2E_GPU` and `--visible`).
That call also loads the compositor's libraries before the first session waits for them.

## Start-up waits and where a start stops

A session start waits for the control socket and then for the app's first state line.
Both waits are bounded by `COLD_START_BOUND_MS` in `src/timing.ts`,
sized from the measured start times recorded beside it:
a start with the files in the page cache takes about a second,
and a start after they were evicted took up to 57 s on the development host,
because the time goes to reading Mesa,
LLVM and the binaries from a busy disk,
not to computing (the doc `doc/troubleshooting/music-player-e2e-search-answer-and-headless-start.md` has the numbers).
When a start does not finish in time,
the message names how far it got from the unit's log (`startStageOf` in `src/log-scan.ts`):
the compositor wrote nothing,
the compositor started but opened no socket,
the socket is open but the app was not started,
the app was started but wrote no line,
or the app logs but wrote no state line.

## Failure evidence

Each failing check writes `<E2E_OUT>/failures/<group>--<name>.failure.txt` with the check's id and title,
its failure text,
the name of the session it ran in,
the last ten `ui_state` changes (`seq`,
`reason`,
`zone`,
`focus`,
`route` and `unfolded`) without the position ticks of a playing track (their count is stated),
and the final state.
Beside it sit the files that say what the session itself was doing,
read while the session still runs because the scratch directory and the unit go away with it:
`<group>--<name>.unit.log` (the unit's standard output and error,
the compositor's lines and the app's),
`<group>--<name>.cpu.stat` and `<group>--<name>.io.pressure` (the accounting of the unit's control group,
which says whether the unit was throttled on its CPU quota and how long its tasks waited for the disk).
The failure file names each one with the path it came from,
and says why a file was not kept when the unit had already stopped.
The paths are printed on the `FAIL` line,
on the closing `FAILED` line and in `e2e-summary.json`.
The folder is emptied at the start of each run.
A session that stops between two checks (a page it expects never came back) is recorded as `session/<name>` with the same evidence,
and the run goes on with the next session instead of ending without a report.

## Isolation

- Audio goes only into a private PipeWire null sink that the run creates and unloads.
  After playback starts,
   every output stream of the app is looked up by process id in the PipeWire graph;
  a link to any node other than the private sink ends the run at once with a safety stop.
- The compositor and the app run inside a transient systemd user unit per session,
   named `music-player-e2e-<pid>-<label>`.
  The app process is found by its control group,
   so another `music-player` process,
   such as your own player,
   is never matched.
  The runner signals no process by id or by name;
   sessions end through the compositor's `quit` and the unit.
- The run refuses to start while any `music-player-e2e-*` unit is active.
- Every session gets scratch XDG directories,
   a stub `xdg-open` and a scratch trash,
   under the scratch root.

## Environment

- `E2E_ONLY`:
   a check id or group prefix to run alone,
   such as `seek` or `row/delete`.
- `E2E_OUT`:
   where screenshots and `e2e-summary.json` go;
   default `target/e2e` in the app directory.
- `E2E_TMP`:
   scratch root;
   default `temp/agent/music-player-e2e` under the home directory,
   created with mode 0700.
- `E2E_VISIBLE`:
  exactly `1` opens a nested window on the desktop (the same as `--visible`);
  unset,
  empty and every other value keep the run headless.
- `E2E_GPU`:
  the render device of a headless run,
  `software` or a render node path such as `/dev/dri/renderD128`,
  used instead of the `--list-gpus` probe.
- `E2E_COMPOSITOR`:
   path of the compositor executable;
   default the active mise install,
  found with `mise where` from the app directory.
   Use it to run against a build that is not installed yet,
   such as `package/nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session`.
- `E2E_LIBRARY_GENERATOR`:
   path of the library generator script that materializes the one-thousand-track library;
   default `temp/agent/agent-scale/make-scale-library.ts` under the home directory.
   That script is not part of this repository,
   and its name corpus is a private file;
   it is the one input of the suite that still lives outside the repository.
   When the file is absent,
   the run builds a fallback library of the same shape in `src/library.ts`:
   thirty loose files,
   one artist folder of thirty short tracks between the two tone folders,
   and the tone folders themselves.
   The full suite passes with the fallback (every check,
  and 1 not injectable),
   so the suite needs no out-of-repository input.

## Layout

- `src/cli.ts` is the command:
  it reads `--app-dir`,
  `--visible` and the environment,
  runs the suite,
  and fails the command when checks fail.
- `src/compositor-path.ts` finds the installed compositor with `mise where`,
   and `src/compositor-version.ts` refuses one without the controls the checks use.
- `src/gpu-choice.ts` decides where the compositor draws and,
  for a headless run,
  probes the render node.
- `src/run.ts` checks the preconditions,
   builds the scratch library,
   creates the sink,
   runs the sessions the filter selects,
   and writes the report.
- `src/session.ts`,
   `src/session-launch.ts` and `src/control.ts` start a session in its own unit and drive the compositor.
- `src/ui-state.ts`,
   `src/ui-state-log.ts` and `src/log-scan.ts` read the app's state from its log.
- `src/pipewire.ts` holds the null sink and the routing proof.
- `src/recorder.ts`,
  `src/failure-evidence.ts`,
  `src/session-evidence.ts`,
  `src/report.ts` and `src/selection.ts` record outcomes,
  keep each failure's evidence,
  print the table and summary,
  and apply the filter.
- `src/library-folders.ts` reads the folder shape of the scratch library it built:
   one entry per top-level folder with the tracks it holds,
   the fullest folder,
   and the step bound of a walk through the folder list.
- `src/folder-choice.ts` opens a folder by its name through the folder list's own keys
   (`Alt+1`,
   the first letter,
   `Down` rows,
  `Enter`),
   waiting for the app to report the folder it shows.
  Naming the folder keeps a check independent of a row position,
   of the window size and of the font size.
- `src/wide-interaction.ts` and `src/narrow-interaction.ts` hold the click-through checks:
   while Search covers the player,
  a click on its empty area changes nothing under it (wide and narrow window),
   and a click on the body of a toast does not reach the track row under it.
  Each proves its click points are live first
   (the same click selects a row,
   or toggles play on the selected one,
   or changes the mode,
   once the page or the toast is gone).
  The wide checks need a track row under every click point,
   so they open the library's fullest folder by name before they click,
   and the failure text names that folder and its track count.
- `src/wide-select.ts` holds the select-only checks (D124,
  D127,
  D128):
   a Search result for another track selects it paused and stops the playing track,
  the same result again plays it,
   and a row whose file cannot be opened (an unreadable file the check writes into the scratch library) stays the
   selected row until a click loads it once the file has been repaired.
- `src/wide-menu.ts` holds the menu placement checks:
   a right click opens the track menu at the pointer in the two-pane layout,
   and the menu is wider than the 280 px that cut its longest shortcut hint.
   The app reports the menu card's position and size
   in its `ui_state` lines
   (`menu_x`,
  `menu_y`,
  `menu_width`,
  `menu_height`).
- `src/wide-*.ts`,
   `src/narrow-session.ts`,
   `src/dialog-session.ts`,
   `src/quit-session.ts` and `src/visual-session.ts` are the scenarios.
  Each check is declared where it runs,
   with its id (`group/name`) and title.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/music-player/desktop-e2e:<task>`.

The unit tests cover everything that needs no display:
 the log parser,
 the filter,
 the scenario table (every check id belongs to a session the filter reaches,
 and the suite keeps its published count of checks that run and its 1 not-injectable one),
 the command lines sent to the compositor,
 the routing proof with allowed and rejected graphs,
 the unit and process guards,
 the report and the summary file.
The folder selection,
 the library folder count and the session loop
 (a session that stops is recorded as `session/<name>` and the run goes on;
 a safety stop ends the run at once)
 are unit tested too.
The scenarios and the session start-up run only in the real end-to-end run.
