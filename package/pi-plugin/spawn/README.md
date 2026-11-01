# Pi Spawn

Pi Spawn provides `spawn-pi`,
 a visible child Pi session launcher with first-result forwarding
back into the parent Pi session.

## Prerequisites

- [terminal-exec](../../terminal-exec/cli/) installed on `PATH`.
- Node for the `spawn-pi` CLI.
- This package loaded as a Pi extension in the parent session,
   so session mappings and result delivery exist.

## Install

From this workspace:

```bash
pi install ./package/pi-plugin/spawn
```

The repository root depends on this package as a workspace dev dependency,
 so `pnpm install`
links `spawn-pi` into `node_modules/.bin` for normal repo shells.

For local development without installing:

```bash
pi -e ./package/pi-plugin/spawn/src/index.ts
```

When the extension starts in an interactive Pi session,
 it symlinks `spawn-pi` to
`~/.local/bin/spawn-pi` if no `spawn-pi` command is already on `PATH`.
Built package installs expose `dist/final/node/cli.mjs` as the command;
source-mode development uses `src/cli.ts`,
 which has a Node shebang.

## Usage

```bash
spawn-pi "implement feature X"
spawn-pi --cwd /some/path "fix the bug"
spawn-pi --extra-arguments "--model openai/gpt-5.1 --thinking high" "review this module"
```

The command prints:

```json
{"spawnId":"<uuid>"}
```

A new terminal window opens with `pi` running the given prompt.
After the child Pi agent loop finishes its first response,
 the parent extension
uses Pi's `sendMessage` API to inject a visible custom message containing the child session id,
session file,
 and last assistant message.

If no parent Pi session mapping exists,
 `spawn-pi` prints a warning,
 launches the child Pi
without result forwarding,
 and prints:

```json
{"resultForwarding":false}
```

Load or reload the spawn-pi extension in the parent Pi session to restore result forwarding.

## How it works

The package has two pieces.

### `spawn-pi` CLI

The CLI resolves the calling Pi session by walking the process tree and matching
ancestor PIDs against files written by the extension under:

```text
~/.pi/agent/spawn-results/.by-pid/
```

It pre-creates a spawn state file under:

```text
~/.pi/agent/spawn-results/spawns/{spawnId}.json
```

Then it launches a visible child terminal through `terminal-exec`:

```text
terminal-exec --title="spawn-pi <short-id>" -- pi ...args "prompt"
```

`--extra-arguments` is split on whitespace,
 matching `spawn-claude`.
Use it for Pi flags such as `--model`,
 `--thinking`,
 `--tools`,
 or `--name`.

### Pi extension

The extension runs in both parent and child Pi sessions.

On `session_start`,
 it:

- writes a PID mapping for parent discovery;
- exports its extension path through `PI_SPAWN_EXTENSION_PATH`,
   so child Pi loads the same extension;
- claims a spawn state when `PI_SPAWN_ID` is present;
- symlinks the CLI into `~/.local/bin` when interactive and needed.
- starts a lightweight monitor that consumes completed child state and calls `pi.sendMessage`.

On child `agent_end`,
 it writes the first assistant result into the spawn state.

The completed-child monitor consumes results by renaming `{spawnId}.json` to `{spawnId}.reported`,
then delivers the formatted result as a `spawn-pi` custom message with `triggerTurn: true`.

## Limitations

- If no parent session mapping exists,
   `spawn-pi` falls back to an unlinked child Pi session.
  The child terminal still opens,
   but no spawn state is written and no result is injected back
  into the parent session.
- Result forwarding is first-result forwarding.
   After the parent consumes a child result,
  later manual conversation inside that child terminal is not reported through the same spawn id.
- The process-tree lookup uses Linux `/proc`.
   If that fails,
   the CLI falls back to the most
  recently written PID mapping,
   which is suitable for one active parent Pi session.
- The child must load this extension.
   The parent extension passes `--extension <current extension path>`
  to the child,
   which covers local development and package installs.
- `terminal-exec` opens a real terminal window.
   Do not use `spawn-pi` when a background
  noninteractive result is desired;
   use Pi print or JSON mode instead.

## Development

```bash
mise run //package/pi-plugin/spawn:build
mise run //package/pi-plugin/spawn:test:unit
mise run //package/pi-plugin/spawn:lint
mise run //package/pi-plugin/spawn:verify:extension
```
