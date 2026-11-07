# pi-bash-poke

Pi extension that runs `!` commands in the background and pokes the model when they finish.

## Pi dependency trust

PJU:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Behavior

Native `!` is foreground:
Pi's TUI submit handler awaits the command,
streams its output into the transcript,
and refuses a second command while one runs.
This extension takes execution over through Pi's `user_bash` event,
so `!` returns control to the editor immediately,
records a placeholder in context,
and sends the model a poke carrying the real output when the process exits.
The motivating case is `! sleep 7200` as a delayed continue after a provider usage limit resets.

Commands run concurrently,
and each finished command produces exactly one poke.

`!!` is left untouched.
It stays Pi's own foreground,
streaming,
context-hidden bash,
which is the escape hatch for a command whose output should never reach the model.

### What the model receives

A poke is a custom message with the `bash-poke` type,
sent with `triggerTurn` and `deliverAs: "followUp"`,
so it fires immediately when the agent is idle and queues behind current work otherwise.
Its content is:

`````text
[bash finished] $ make test (exit 2)
[complete output: /tmp/pi-bash-poke/6f1c....log]
```
FAILED tests/foo.test.ts
```
`````

No trailing instruction is appended.
The delivery options trigger the turn on their own,
so the card ends at its output and the transcript carries no machine-written imperative.

The transcript renders the same text behind a labelled card,
so a poke never looks like something you typed.

Output is middle-out truncated:
the head and the tail are kept and the middle is elided with a marker naming the loss.
The complete output is spooled to an owner-only file below the OS temp directory,
and the poke names that path whenever elision happened,
so a truncated result is never a dead end.

Output is also sanitized before it is rendered or fenced:
control characters other than tab,
line feed,
and carriage return are dropped,
as are unpaired surrogates and invisible formatting characters.
The fence around the output is always longer than any backtick run inside it,
so command output cannot terminate its own code block.

### Progress and cancellation

While a job runs,
a row appears above the editor with the command,
its elapsed time,
and its most recent output lines:

```text
! make test (1m30s)
  FAIL tests/foo.test.ts
```

The elapsed clock advances on a tick,
so a silent command such as a long sleep still shows progress.

A lone Escape cancels every running job.
The keystroke is observed,
never consumed,
and it is claimed only when the agent is idle and the editor is not in bash mode,
so Pi keeps aborting a streaming turn,
leaving bash mode,
and opening the double-escape selectors.
A cancelled job never pokes.

Pi's stdin buffering already disambiguates a lone Escape from an alt combination,
and both the bare escape byte and the Kitty keyboard protocol form are recognized.

### What this package owns

Pi's `shellPath` setting is unreachable from an extension,
and its execution and truncation helpers are deliberately not used here.
This package resolves its own shell (`/bin/bash`,
then `bash` on `PATH`,
then `sh`),
spawns the command detached so cancellation reaches the whole process group,
bounds output in memory while spooling it in full,
truncates and sanitizes it,
and escalates from `SIGTERM` to `SIGKILL` after a grace period.

Pi is used only for its extension contract:
the `user_bash` and `session_start` events,
the returned bash result shape,
`sendMessage`,
`registerMessageRenderer`,
and the context's UI and idle state.

## Settings

Settings live in `~/.pi/agent/extensions/pi-bash-poke.json`.
Every key is optional,
and an unknown key is rejected with a diagnostic rather than ignored.

- `pokeInstruction`:
   instruction appended to each poke.
  Default empty,
   which appends nothing.
  Set a value such as `continue` to restore one.
- `pokeHeadChars`:
   characters kept from the start of output.
  Default `2000`.
- `pokeTailChars`:
   characters kept from the end of output.
  Default `6000`.
- `progressWidget`:
   whether running jobs are drawn above the editor.
  Default `true`.
- `progressTailLines`:
   output lines shown per running job.
  Default `3`.
- `progressRefreshMs`:
   minimum milliseconds between output-driven redraws.
  Default `250`.
- `progressTickMs`:
   milliseconds between elapsed-time redraws.
  Default `1000`.
  Zero disables ticking.
- `killGraceMs`:
   milliseconds between `SIGTERM` and `SIGKILL` on cancel.
  Default `2000`.
  Zero sends both at once.

## Installation

Add the package to global Pi settings:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/bash-poke"
  ]
}
```

Restart Pi or run `/reload` after updating the setting.
If the stopgap `~/.pi/agent/extensions/bash-poke.ts` is present,
delete it first,
because Pi auto-discovers that directory and both copies would intercept `!`.

## Known limitations in v0.x

- Jobs are not tracked across session transitions.
  Pi invalidates an extension runtime on reload and on session replacement,
  and Pi exits through `process.exit` without killing children it does not own,
  so a job still running when Pi exits becomes an orphan and its poke is lost.
  Delivery in that state is caught and logged rather than thrown.
- The extension knows nothing about provider usage limits.
  It can be used to wait for a reset,
  but it does not detect one,
  classify a failure,
  or retry a poke.
- There is no live output streaming into Pi's own bash component.
  Pi renders a returned result once,
  and the alternative `operations` path awaits the command,
  which is the foreground behavior being replaced.

## Development

Run package validation from the repository root:

```sh
mise run //package/pi-plugin/bash-poke:build
mise run //package/pi-plugin/bash-poke:lint
mise run //package/pi-plugin/bash-poke:test:unit
```

Unit tests import the built artifact,
so build before testing.

The fake-terminal suite drives a real Pi TUI inside `tmux` and is excluded from the default run:

```sh
mise run //package/pi-plugin/bash-poke:test:unit -- --all
```

It skips itself inside a Pi session,
because Pi sets `PI_CODING_AGENT` for every child process.
Force it with the override flag:

```sh
BASH_POKE_TUI_VERIFY=1 mise run //package/pi-plugin/bash-poke:test:unit -- --all
```

The nested Pi it starts uses an empty `PI_CODING_AGENT_DIR`,
`--no-extensions` apart from this package,
`--no-session`,
and `PI_OFFLINE=1`,
so it has no credentials and cannot reach a provider.

### Manual check with a real model

The automated suites stop at the poke,
because the isolated agent directory has no provider.
To verify the turn itself:

1. Add the package path to `~/.pi/agent/settings.json` under `packages`.
2. Run `/reload` in Pi.
3. Type `! printf poked` and confirm the placeholder appears at once,
   the progress row shows above the editor,
   and a labelled `bash-poke exit 0` card arrives with the output.
4. Confirm the model then takes a turn without further input.
5. Type `! sleep 300`,
   press Escape once,
   and confirm the cancellation notice appears and no card follows.
