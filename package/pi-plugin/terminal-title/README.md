# pi-terminal-title

Terminal tab title extension for pi:
shows current tool,
session state,
and user prompt in the terminal window title.

## Pi dependency trust

PZD:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Installation

### Global (recommended)

```sh
pi install ./packages/pi-plugin/terminal-title
```

Or add to `~/.pi/agent/settings.json`:

```json
{
  "packages": ["./packages/pi-plugin/terminal-title"]
}
```

### Quick test

```sh
pi -e ./packages/pi-plugin/terminal-title/src/index.ts
```

## How it works

The extension subscribes to pi agent lifecycle events and updates the terminal window/tab title via
`ctx.ui.setTitle()`.
Titles use a `π` prefix,
replace OSC-breaking controls with visible tokens,
and are byte-capped below Ghostty's 256-byte UTF-8 title reject threshold before `ctx.ui.setTitle()`.
There is no separate display-character cap.

## Event title examples

- `tool_execution_start`:
   `π Reading src/index.ts`
- `tool_execution_end`:
   `π Read src/index.ts`
- `session_start`:
   `π Started session: startup`
- `session_shutdown`:
   `π Ended session`
- `agent_end`:
   `π Stopped agent`
- `before_agent_start`:
   `π Received prompt: Refactor the auth module`

## Tool title examples

- `bash`:
   `π Running npm test` while running,
  `π Ran npm test` after completion.
- `read`:
   `π Reading src/index.ts` while running,
  `π Read src/index.ts` after completion.
- `edit`:
   `π Editing src/config.ts` while running,
  `π Edited src/config.ts` after completion.
- `write`:
   `π Writing dist/output.ts` while running,
  `π Wrote dist/output.ts` after completion.
- `grep`:
   `π Searching for TODO` while running,
  `π Searched for TODO` after completion.
- `find`:
   `π Finding *.ts` while running,
  `π Found *.ts` after completion.
- `ls`:
   `π Listing src` while running,
  `π Listed src` after completion.

Custom and MCP tools that are not in the registry display generically:
`π Running mcp__weather` while running and `π Ran mcp__weather` after completion.

## Source structure

```text
src/
  index.ts           # Extension entry point, event handlers
  title-builder.ts   # titleForEvent(), titleForTool(): maps pi events to titles
  tool-titles.ts     # Tool name to title-entry registry using pi tool names
```

Shared terminal-title engine helpers live in `@monochromatic-dev/agent-harness-shared-terminal-title`.
