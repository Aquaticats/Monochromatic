# @monochromatic-dev/agent-harness-shared-truncate

Neutral tool-output truncation and size formatting with pi-identical semantics.

## Why

Pi's coding agent bundles `truncateHead` and `formatSize` inside its own tool layer,
and host adapters must render byte-identical model-visible output whether they run as a pi
extension or as an MCP server.
This package ports those two helpers into a dependency-free module,
and `src/truncate-parity.unit.test.ts` proves the port matches pi across a behavior corpus.

## API

- `truncateHead(content, options?)` keeps whole lines from the start of content
  and reports which ceiling stopped it.
- `formatSize(bytes)` renders byte counts the way tool descriptions quote truncation limits.
- `DEFAULT_MAX_LINES`, `DEFAULT_MAX_BYTES` hold the shared ceilings.

## Usage

```ts
import {
  formatSize,
  truncateHead,
} from '@monochromatic-dev/agent-harness-shared-truncate/ts';

const result = truncateHead(fullResponse,);
```

## Tests

- `src/truncate.unit.test.ts` covers every truncation branch,
  including the empty-first-line ceiling case.
- `src/truncate-parity.unit.test.ts` runs a differential corpus against
  `@earendil-works/pi-coding-agent` and fails on any divergence.

## Lint suppressions

`src/truncate.ts` carries one `no-restricted-syntax/no-nullish-union` disable on
`TruncationResult.truncatedBy: 'lines' | 'bytes' | null`.
Linted value: Pi's exported `TruncationResult` field of the same name,
which is `'lines' | 'bytes' | null` (`@earendil-works/pi-coding-agent` `dist/core/tools/truncate.d.ts`).
The rule's own remedy list offers a scoped disable for external API mirrors,
and no config or allow-list form names a cross-package mirror,
so the disable is the narrowest compliant option and the parity suite keeps it honest.
