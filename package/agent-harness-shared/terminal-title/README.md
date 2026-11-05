# terminal-title

Shared terminal title engine for agent harness integrations.

## Pi dependency trust

PJF:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Purpose

This package owns host-neutral terminal title invariants:
control-character sanitizing,
UTF-8 byte caps,
smart path display,
shell command summaries,
lifecycle-aware tool title entries,
safe registry lookup,
unknown-tool fallback,
and prefix assembly.

Host packages keep event adapters and side effects.
For example,
pi still calls `ctx.ui.setTitle()`,
while Claude Code still writes OSC 0 sequences through its hook CLI.

## Entry points

```ts
import {
  buildToolTerminalTitle,
  pathTitleEntry,
  safeTerminalTitlePayload,
  shellCommandTitleEntry,
} from '@monochromatic-dev/agent-harness-shared-terminal-title';
```

Source imports are also available for workspace packages:

```ts
import { buildToolTitle } from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';
```

## Engine shape

```ts
import {
  buildToolTerminalTitle,
  pathTitleEntry,
  shellCommandTitleEntry,
  type ToolTitleRegistry,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';

const registry: ToolTitleRegistry = {
  bash: shellCommandTitleEntry({ field: 'command' }),
  read: pathTitleEntry({
    field: 'path',
    labels: { pre: 'Reading', post: 'Read' },
    noun: 'file',
  }),
};

const title = buildToolTerminalTitle({
  prefix: 'π',
  registry,
  toolName: 'read',
  input: { path: '/repo/src/index.ts' },
  tense: 'pre',
  context: { cwd: '/repo' },
});
// 'π Reading src/index.ts'
```

## Terminal payload safety

Terminal output boundaries should call `safeTerminalTitlePayload()` on final title payload text
before handing it to a host API or wrapping it in an OSC title sequence.
The helper replaces OSC-breaking controls with visible control pictures,
then caps payload text to 255 UTF-8 bytes by default.

```ts
import { safeTerminalTitlePayload } from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';

const payload = safeTerminalTitlePayload({ value: title });
```

The byte cap stays below Ghostty's current 256-byte reject threshold for title payloads.
