# music-player: the Windows executable is a GUI-subsystem program that attaches to the console of its parent

Status:
 Implemented on branch `feat/music-player-windows-gui-subsystem`, proven on GitHub's Windows runners.
 One tradeoff accepted by the coordinating session on 2026-10-15 under the owner's standing instruction to make
 other decisions itself, open to the owner's veto: in PowerShell, `music-player --version > out.txt` leaves an
 empty file (it worked with the console-subsystem build of release 0.1.3); see "Consequences for users".
 Reason for accepting: every GUI-subsystem Windows program behaves this way in PowerShell (a probe with no console
 code gave 0 bytes), the working forms (`| Out-File`, `| Out-String`, `Start-Process -RedirectStandardOutput`, and
 `cmd.exe` redirection) are documented in the README and release notes, and the alternative of a second console
 binary adds a fourth Windows file to every release for a rarely used flag.
Decided:
 2026-10-15 (the details; the release agent proposed the approach after release 0.1.3).
Applies to `package/music-player/desktop-app` and the three Windows release files.

## Context

Release `music-player-v0.1.3` shipped three Windows executables linked for the console subsystem.
Windows opens a console window next to the app for every double-click start of such a file.
The release notes, `package/music-player/README.md` and `doc/decision/music-player-releases.md` said so.
The proposed follow-up was `#![windows_subsystem = "windows"]` plus `AttachConsole(ATTACH_PARENT_PROCESS)`,
so `music-player --version` still prints in a terminal.
This record checks that design against the Rust and Windows documentation and the sources, and records the details.

## Decision

- `src/main.rs` carries `#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]`.
  The attribute alone is enough on both linkers; no `-mwindows` flag and no `.cargo/config.toml` change is needed.
  The `not(test)` part keeps the unit-test build of the binary a console program, so nextest output is unchanged.
  The debug build is a GUI program too, so the CI check inspects the same subsystem the release has.
- `main` calls `attach_parent_console()` (`src/console_attach_win32.rs`) before `logging::init()` and before
  `Cli::parse()`, because both print.
  The outcome is logged at debug level after the logger exists, so `--version` and a normal start print no extra line.
- The attach is skipped when both stdout and stderr already hold a usable handle (a pipe, a file, a console the shell
  passed on).
- When a handle is missing, the process attaches to the console of its parent.
  Afterwards, for each of stdout and stderr:
  a handle the process started with is put back when the attach replaced or cleared it;
  a handle that was missing and is still missing after the attach is pointed at `CONOUT$`;
  a handle that was missing and was supplied by the attach is kept.
  The decisions are pure functions in `src/console_attach.rs`, tested on every host (10 tests).
- A double-click start has no parent console (Explorer has none), so the attach fails with `ERROR_INVALID_HANDLE`
  and the program runs without a console. Its log lines and its startup error messages are not shown anywhere.
- Every Windows item is behind `cfg(windows)`; the pure half compiles on Linux and macOS.
  `windows` crate features added: `Win32_Security`, `Win32_Storage_FileSystem`, `Win32_System_Console`.
  `Cargo.lock` did not change.

## Sources checked

- The Rust reference, runtime chapter, `windows_subsystem`: accepted values are `"console"` and `"windows"`;
  the attribute applies to the crate root; it is ignored on non-Windows targets and on crate types other than `bin`;
  the `"windows"` subsystem runs detached from any existing console.
- rustc at commit `e9bbadf6824e` (the commit of the pinned `nightly-2026-10-13`),
  `compiler/rustc_codegen_ssa/src/back/linker.rs`:
  for the gcc driver (`GccLinker`, line 892) `windows_subsystem` passes `--subsystem` and the value to the linker
  (`-Wl,--subsystem,windows`, seen in the printed link line of the GNU job);
  for `MsvcLinker` (line 1151) it passes `/SUBSYSTEM:windows` and `/ENTRY:mainCRTStartup`.
  This is why the attribute alone works with GNU ld.
- Rust standard library at the same commit (`library/std/src/sys/stdio/windows.rs`, `library/std/src/io/stdio.rs`):
  `get_handle` returns `ERROR_INVALID_HANDLE` for a null handle, and `handle_ebadf` turns that error into a successful
  write that goes nowhere. `print!`, `eprintln!`, the panic message and the `tracing` stderr writer therefore never
  panic on a missing standard handle. The handle is read on the first write, so the attach has to run before it.
- Microsoft, `AttachConsole`: `ATTACH_PARENT_PROCESS` is `(DWORD)-1`; the call fails with `ERROR_ACCESS_DENIED`
  when the process already has a console and with `ERROR_INVALID_HANDLE` when the target has none; it is meant
  for programs linked with `/SUBSYSTEM:WINDOWS`, whose standard handles "will likely be invalid on startup".
- Microsoft, `GetStdHandle`, section "Attach/detach behavior": when attaching to a console, standard handles are
  "always replaced with console handles unless `STARTF_USESTDHANDLES` was specified during process creation".
  This is why a handle the process started with is put back after the attach
  (`> out.txt` passed by a parent that did not use `STARTF_USESTDHANDLES` would otherwise end up on the console).
- `windows` crate 0.62.2 (`src/Windows/Win32/System/Console/mod.rs`): `AttachConsole(u32)`, `GetStdHandle`
  (an `Err` for a null and for an invalid handle), `SetStdHandle`, `ATTACH_PARENT_PROCESS`.
- Microsoft, `start` command: "If you run a 32-bit graphical user interface (GUI) application, `cmd` doesn't wait for
  the application to quit before returning to the command prompt."
- PowerShell, `NativeCommandProcessor.cs`: a Windows GUI application at the end of a pipeline is started as a
  background process (`_isRunningInBackground = isWindowsApplication`), so PowerShell does not wait for it
  and does not read its output.

## Consequences for users

- Double-click: no console window.
- `cmd.exe` and PowerShell return to the prompt at once, because they do not wait for a GUI program.
  The program prints on the terminal it attached to, so the text can appear after the prompt.
- `music-player --version > out.txt` in `cmd.exe` and in bash writes the file. In PowerShell `>` gives an empty file for
  every GUI program, because PowerShell never reads the output (the CI step compares `music-player` with a probe program
  that has no console code; both gave 0 bytes). Use `music-player --version | Out-File out.txt`, `| Out-String`,
  or `Start-Process -Wait -NoNewWindow -RedirectStandardOutput out.txt`.
- Ctrl+C and closing the terminal reach the attached program as they reach any process attached to that console.
- A start by double-click that fails shows a Windows message box that names the failed step, what Windows reported and
  the remedies; see "Follow-up: startup failure message". The exit status stays 1 (101 after a panic).
- Log lines and clap's text carry ANSI colors only on a console that interprets them; see "Follow-up: console colors".

## Follow-up: startup failure message

Status:
 Implemented and proven on the Windows runners on branch `fix/music-player-windows-startup-errors` (2026-10-18; see
 "Evidence for the two follow-ups"). Closes the gap "a double-click start that fails shows no message and logs nowhere".
 Not seen: nobody looked at the dialog itself; the runners read its title and text back through Win32 calls.

- The steps that end the program: Slint's winit backend (`Backend::builder().build()`, the connection to the desktop),
  `AppWindow::new()` (the window), `app.run()` (the event loop of the running window), and a panic of the main thread.
  No sound output is not one of them: the window shows its own notice and the program keeps running
  (the runner has no audio device and the program stayed up for 15 seconds in every run).
- `src/startup_failure.rs` holds the pure part: one message per step with the detail Windows reported, the remedies
  that hold for this program, and the path of the executable for a Command Prompt start (tested on every host).
  Only the event-loop message offers `SLINT_BACKEND=software`: without a renderer name the winit backend already tries
  the OpenGL renderer and then the software renderer when it creates the window
  (`i-slint-backend-winit` 1.18.1, `try_create_window_with_fallback_renderer`), but OpenGL itself starts later,
  when the window is shown inside the event loop.
- `src/startup_failure_win32.rs` re-reads `GetStdHandle(STD_ERROR_HANDLE)` at the time of the failure.
  No usable handle (a double-click start, or an attach that could not supply stderr): `MessageBoxW` with
  `MB_ICONERROR | MB_OK | MB_SETFOREGROUND | MB_TASKMODAL`, the flags Alacritty uses for its Windows panic box
  (`alacritty/src/panic.rs`). Any handle (a console, a file, a pipe): the text is printed on stderr and no box appears,
  so a script or a CI job never waits for a click. The panic hook covers panics of the main thread: it runs the previous
  hook first (the standard panic message on stderr), and shows the box only when stderr has no usable handle.
  With stderr on a console, a file or a pipe a panic therefore prints the standard panic message and no extra text.
- Mechanism chosen: the message box alone, not a log file. Precedent: Alacritty shows a `MessageBoxW` for a panic of its
  GUI-subsystem build, and Neovide shows its own error window when stdout is not a terminal
  (`src/error_handling.rs`, `handle_startup_errors`); neither writes a startup log file. A log file under the app data
  directory would still need the person to find it, and the box already tells them how to see the log lines
  (start from Command Prompt). The Microsoft `MessageBoxW` page does not document copying the text with Ctrl+C, so the
  message does not claim it.
- The `windows` crate feature `Win32_UI_WindowsAndMessaging` was added (0.62.2 has `MessageBoxW` there);
  `Cargo.lock` did not change.
- Test hook: a Windows debug build reads `MUSIC_PLAYER_FORCE_STARTUP_FAILURE` (`window-system`, `window`, `event-loop`,
  `panic`) and fails that step through the same code path as a real failure. The reading code is behind
  `#[cfg(all(windows, debug_assertions))]`, so release builds do not contain it.
- Linux and macOS are unchanged: the error goes from `main` to stderr exactly as before.
- The text of the message lives in `src/startup_failure.rs` as pure functions (`failure_title`, `failure_message`,
  `message_box_wanted`, `panic_detail`, `without_nul`), unit-tested on every host. Each message names the step that
  failed in plain words, quotes what Windows reported, lists the remedies that hold for that step, and tells how to see
  the log lines (start from Command Prompt with the quoted path of the executable).
- A second defect, found by the forced event-loop failure and fixed in the same change: the process aborted at exit.
  Run 39094984560 (stderr in a file) ended the forced event-loop failure with exit code 3221226505 (0xC0000409) and the
  text `thread 'main' panicked at library/std/src/thread/lifecycle.rs:230:14: threads should not terminate unexpectedly`
  followed by `fatal runtime error: thread local panicked on drop, aborting`. In a double-click start (run 39094394847)
  the process was still alive 35 seconds after the box was closed; the abort itself was seen only with stderr in a file.
  Cause: `src/ui_search.rs` keeps the last handle to the Search service in a `thread_local!` of the window thread,
  and `SearchService::drop` (`src/search_service.rs`) joins three threads. On Windows the thread-local of the main
  thread is dropped while the process is already ending, after the other threads were ended; the std `JoinInner::join`
  (`library/std/src/thread/lifecycle.rs` at the pinned nightly) then finds its result packet still shared and panics.
  The same drop runs when a person closes the window, so a normal exit on Windows was exposed to it too.
  Fix: `SearchService::shutdown` (idempotent) does the stopping and joining, `Drop` calls it, and a guard in `main`
  (`ui_search::ServiceShutdown`) calls it while every thread is alive, on every return path of `main`.
  The same change applies on Linux and macOS (an earlier, harmless join); the new unit tests
  `shutdown_ends_all_three_threads_and_a_second_call_does_nothing` and
  `a_query_after_shutdown_is_ignored_without_a_panic` run on every host.
  Windows evidence: the forced event-loop case ended with exit status 1 on all three jobs after the fix, and with
  0xC0000409 or no exit before it ("Evidence for the two follow-ups").

## Follow-up: console colors

Status:
 Implemented and proven on the Windows runners on branch `fix/music-player-windows-startup-errors` (2026-10-18; see
 "Evidence for the two follow-ups"). Closes the gap "escape characters on `msvc-arm64`".

Cause, from the sources:

- `tracing-subscriber` 0.3.23 (`src/fmt/fmt_layer.rs`, `Layer::default`) turns ANSI colors on whenever the `ansi`
  feature is on and `NO_COLOR` is empty. It never looks at the handle it writes to, and nothing in it sets the console
  mode (`nu-ansi-term` 0.50.3 has `enable_ansi_support`, which `tracing-subscriber` does not call).
  So every log line carried escape sequences: into files (the `launch.log` of the earlier runs holds them on all three
  jobs) and onto consoles that do not interpret them.
- clap prints through `anstream` 1.0.0. `anstyle-query` 1.1.5 (`src/windows.rs`, `enable_virtual_terminal_processing`)
  sets `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on stdout first and returns at the first error, so with stdout in a file the
  console behind stderr is never switched. With `TERM` set (the runners set `xterm-256color`),
  `AutoStream::always` then picks pass-through over the console attribute API, and the escapes reach a console without
  VT processing as text.
- Why only arm64 showed it: a console that a process creates with `AllocConsole` starts with VT processing off on the
  `windows-11-arm` image (output mode 0x3) and on on the x64 images (0x7); `HKCU\Console\VirtualTerminalLevel` is not
  set on either (measured in runs 38776373884 and 39095649585, which print both).

Fix:

- After the attach, `enable_console_colors` (`src/console_attach_win32.rs`) reads the mode of stdout and of stderr
  separately and adds `ENABLE_VIRTUAL_TERMINAL_PROCESSING` to each console; a file or a pipe has no console mode.
- `src/console_color.rs` decides: the log is colored only when stderr is a console with VT processing and `NO_COLOR` is
  empty (`logging::init_with_ansi`); clap keeps its own detection unless a console refused the mode, and then prints no
  colors (`cli::parse_with_color`).
- Like `anstream`, the program leaves the mode on the console it attached to.
- Linux and macOS are unchanged: `tracing-subscriber`'s default still colors log lines there, also in a file.

## Evidence for the two follow-ups

All runs are `workflow_dispatch` runs of `.github/workflows/music-player-windows.yml` on this branch, jobs `msvc-x64`,
`msvc-arm64` and `gnu-x64-ucrt64`
(that workflow moved into the `test-windows` job of `.github/workflows/music-player-release.yml` on 2026-10-22).
The step "Check console colors and the startup failure message" creates a console of
its own (cleared of VT processing for the color cases), starts the debug build with `CreateProcessW`, reads the console
screen buffer back, and for the double-click cases frees its console, starts the program without handle inheritance,
finds the dialog (class `#32770`) by process id, reads its title and text through `WM_GETTEXT`, presses OK and checks
the exit status.

- Positive control, run 38776373884 at `d1f3475f84` (the checks, none of the program change): all three jobs failed.
  Color cases: the console screen buffer held the left-arrow glyph of a literal ESC in front of every colored span of
  the log line and of clap's error text, and the stderr file held `\x1b[` bytes.
  On `msvc-arm64` a fresh console started with mode 0x3, so the harness had nothing to clear there.
  The x64 jobs failed the same cases because the harness clears VT processing on their console (mode 0x7) to
  reproduce the arm64 state.
  Startup failure cases: no dialog existed (`dialog None`) and the program stayed running. The launch step failed with
  `the log written to a file holds ANSI escape sequences`.
- Run 38778131811 at `29175e0d76` (the program change): the colors passed; the box was found on all three jobs with the
  right title and text, but the check failed because the posted `WM_COMMAND` with `IDOK` did not end the process.
- Run 39094394847 at `611a40ff94` (OK pressed with `BM_CLICK` on the Button child): the process ended with the right
  exit status for `window-system`, `window` and `panic` on all three jobs, and did not end for `event-loop`
  (the abort above).
- Run 39094984560 at `21eaf53422` (all four steps also with stderr in a file): `window-system` and `window` passed;
  `event-loop` ended with exit code 3221226505 and the panic text above; `panic` failed only on the harness expectation
  (a panic with stderr in a file prints the standard panic message, not the box text).
- Run 39095649585 at `a478502ccf` (the Search service shutdown): all three jobs passed. Colors: no ESC in the console
  screen buffer or the stderr file for no handles, stdout in a file, and stderr in a file; the log written by the 15
  second launch holds no ESC byte; `msvc-arm64` started its console at 0x3 and the program left it at 0x7. Startup
  failure: with a console or a file, no dialog, the text printed, exit status 1 (101 after the panic); in a double-click
  start the dialog appeared for each of the four steps, was closed by `BM_CLICK` on its OK button, and the process ended
  with 1, 1, 1 and 101. The Windows nextest step ran 1470 tests with 1470 passed and 1 skipped on each job.
- Final run 39096290321 at `4f8719b489` (the branch with `main` merged in, program code identical to `a478502ccf`
  apart from a test helper): all three jobs passed, the same checks as the previous run passed, and the Windows nextest
  step ran 1470 tests with 1470 passed and 1 skipped on each job.
- Host (Linux), the verification units `winerr-verify-2` and `winerr-verify-3`: `lint:slint` status 0, nextest 1498 run,
  1498 passed, 1 skipped (1496 before the two shutdown tests), `lint:clippy` with `-D warnings` status 0 on the final
  head (the first run of it failed on `unwrap` in the two new tests, which the helper `running_threads` replaced).
- `actionlint` 1.7.12 on the workflow: no findings.

## Evidence

All runs are on GitHub-hosted runners with `.github/workflows/music-player-windows.yml` on this branch,
jobs `msvc-x64`, `msvc-arm64` and `gnu-x64-ucrt64`
(see the note on the move to `.github/workflows/music-player-release.yml` in the section "Evidence for the two
follow-ups").

- Positive control, run 38751509147 at `a9b4a681d2` (the new checks, no attribute): all three jobs failed.
  The PE step printed `Subsystem in the PE header: 3 (IMAGE_SUBSYSTEM_WINDOWS_CUI)` and the link-line check failed.
  The bash and PowerShell `--version` steps and the 15 second launch passed on the console build, which is the baseline.
- First run with the attribute, run 38752464513 at `3434eaa455`: Subsystem 2 on all three jobs.
  The PowerShell step failed: `> out.txt` in PowerShell gave an empty file (see "Consequences for users").
- Run 38756439754 at `2db98160ed` and run 38757787904 at `7e5024d176`: all three jobs passed.
  Subsystem 2 (`IMAGE_SUBSYSTEM_WINDOWS_GUI`) read with `pefile` 2024.8.26;
  the printed link line holds `/SUBSYSTEM:windows` (MSVC) and `-Wl,--subsystem,windows` (GNU);
  `--version` through a bash pipe, `> file` in bash, `2>` of a bad argument (exit status 2),
  PowerShell pipeline capture, `2>&1`, `| Out-File`, `Start-Process -RedirectStandardOutput` and `cmd.exe > file`
  (the file held the text 105 to 117 ms after `cmd.exe` returned) all gave `music-player 0.1.3`;
  the 15 second launch check passed and the log file held the `tracing` lines;
  nextest ran 1432 tests with 1432 passed and 1 skipped on each job.
- The attach branch, run on a real console (runs 38761713560 at `970a1f0b3b` and 38763214487 at `248442611a`,
  all three jobs passed both): a Python process creates a console of its own and starts the program with
  `CreateProcessW`, with `RUST_LOG=debug` so the program prints its own attach report on that console.
  - No standard handle: `--version` and a bad argument (exit status 2) appeared in the console screen buffer;
    the report was `stdout` Keep and `stderr` Keep (the attach supplied both).
  - Stdout an inherited file, stderr missing, no `STARTF_USESTDHANDLES`: the report was `stdout` `Restore` with the
    original handle value and `stderr` Keep. The version stayed in the file with an empty console, and the bad-argument
    text reached the console with an empty file. This is the case the Microsoft sentence describes
    ("always replaced with console handles"), and the measurement shows the replacement happened and was undone.
  - The same with `STARTF_USESTDHANDLES`: the report was Keep and Keep, with the same file and console contents.
- Double-click start, same runs: the Python parent called `FreeConsole()` and started the program without handle
  inheritance, so the attach had no console to join. A bad argument exited with status 2 (a panic would give 101).
  A start without a path and a start on the `fixture` folder were both still running after 15 seconds.
  A sibling no-path start with both output handles in files logged `music folder candidate accepted` for
  `C:\Users\runneradmin\Music` and started the folder walk, so the runner takes the normal start path, not the
  "no saved folder and no music folder" first-run line. Nothing in these cases panicked on a missing standard handle.
- Escape characters on `msvc-arm64`: the first log line of the program (written by `tracing`, whose ANSI colors
  `src/logging.rs` leaves on) and clap's error text in the stdout-file cases contained literal ESC characters in the
  arm64 console screen buffer (shown as left arrows), and the same text was plain on the x64 jobs.
  The arm64 console does not interpret ANSI sequences that the x64 console interprets; the console mode was not read.
  `anstream` 1.0.0 `AutoStream::always` passes ANSI through when `TERM` is set (the runners set `TERM=xterm-256color`),
  and it can enable virtual terminal processing only when stdout is a console,
  so the stdout-file cases keep the escapes.
  These writers do not depend on the attach, but no control run of a console-subsystem build covered these cases.
- Release workflow, dispatch with `dry-run` true from this branch, run 38757804504 at `7e5024d176`: every job passed.
  The detect job printed `dry_run=true`; the step "Package (Windows executable)" printed `subsystem 2` for the x64,
  arm64 and GNU executables of the release profile; the notes carry "The executable opens no console window";
  the release job printed `DRY RUN, would run: gh release create` and created nothing.
- Positive control for that release check: the exact Python text extracted from `music-player-release.yml`
  was run with `pefile` 2024.8.26 on two executables from the windows-gnu image's `x86_64-w64-mingw32-gcc`.
  The `-mconsole` file (Subsystem 3) failed with the `::error::The Subsystem field ... is 3` line and exit status 1;
  the `-mwindows` file (Subsystem 2) passed.
  The release run itself did not build a console-subsystem file.

## Not verified

- Nobody looked at the startup failure dialog. The runners read its title and text through `WM_GETTEXT` and press its OK
  button with `BM_CLICK`; how it looks (size, wrapping of the long lines, icon, whether it comes to the front on a real
  desktop) was not seen. The longest message (event-loop) has 15 lines, blank ones included.
- Whether Windows Error Reporting shows a "stopped working" window for the abort that this change removes was not
  observed; the runner showed only that the process stayed alive for the check.
- A real graphics or window-system failure was not produced; the four failures are forced by the debug-only test hook.
  The exit abort was reproduced only through the forced event-loop case; the same drop path runs when a person closes
  the window, which no run did.
- Nobody looked at a Windows desktop: that a double-click opens no window is inferred from Subsystem 2 and the
  documentation, not seen.
- The attach to an interactive terminal (`cmd.exe`, PowerShell, Windows Terminal) was not run; the runner steps used
  pipes, files and a console created by a Python process.
- The prompt-returns-at-once behavior of `cmd.exe` and PowerShell is documented (`start` page, PowerShell source) and
  matches the measured `cmd.exe` return after about 100 ms, but no interactive session was observed.
- The windows-gnu container (`test:windows-gnu`, under Wine) was not run for this change.
- `clippy` for `x86_64-pc-windows-gnu` (run in the windows-gnu image with the clippy component added)
  reports 9 findings in code this change does not touch
  (`launcher.rs`, `change_set.rs`, `ui_progress.rs`, `ui_files.rs` and one function in `main.rs`)
  and none in the new files.
  The repository has no Windows clippy task, so these findings were never reported.
