# pi-morph-compact

Morph Compact integration for pi:
 replaces default LLM summarization with line-deletion compression at 33K tok/s.

## Setup

Set the `MORPH_API_KEY` environment variable:

```sh
export MORPH_API_KEY="sk-your-key-here"
```

Get a key from [morphllm.com](https://morphllm.com).

If the env var is not set,
 the extension reads the key from `~/.pi/agent/mcp.json`:
any MCP server entry with `env.MORPH_API_KEY` is used.
 This is convenient when
the Morph MCP server is already configured.

## Installation

### Global (recommended)

```sh
pi install ./packages/pi-plugin/morph-compact
```

Or add to `~/.pi/agent/settings.json`:

```json
{
  "packages": ["./packages/pi-plugin/morph-compact"]
}
```

### Quick test

```sh
pi -e ./packages/pi-plugin/morph-compact/src/index.ts
```

## How it works

When pi's context window fills up and compaction triggers,
 this extension intercepts the `session_before_compact` event
 and calls the Morph Compact API
 instead of pi's default LLM summarization.

**Key differences from default compaction:
**

- Morph uses line deletion,
   not summarization:
   it removes irrelevant lines from the serialized conversation while keeping exact excerpts
- Output is wrapped in `<morph-compacted-history>` tags with an explanatory header so the LLM understands the format
- Previous compaction context is preserved via `<keepContext>` tags
- File tracking (`<read-files>`,
   `<modified-files>`) is appended to the Morph output

**Fallback behavior:
**

- Missing `MORPH_API_KEY`:
   warns once per session,
   falls through to pi default
- Split-turn compactions:
   always uses pi default (Morph can't produce coherent turn prefix summaries)
- API errors:
   notifies and falls through to pi default

## Adaptive compression

Compression ratio adjusts based on context pressure.
 Ratios are deliberately
higher than traditional summarization would use for two reasons:

1. **Speed**:
    Morph Compact runs at 33K tok/s,
    so re-triggering compaction is
   nearly instant.
    There is no penalty for keeping more context and compacting
   again sooner.
2. **Drift reduction**:
    every compaction cycle loses some information.
    Preserving
   more context per cycle keeps the model's working memory closer to the full
   conversation,
    reducing accumulated drift across repeated compactions.

<table>
<thead>
<tr>
<th>Context usage</th>
<th>Ratio</th>
<th>Lines kept</th>
</tr>
</thead>
<tbody>
<tr>
<td>>80%</td>
<td>0.3</td>
<td>30%</td>
</tr>
<tr>
<td>>60%</td>
<td>0.4</td>
<td>40%</td>
</tr>
<tr>
<td>else</td>
<td>0.5</td>
<td>50%</td>
</tr>
</tbody>
</table>

## Commands

- `/morph-compact`:
   trigger compaction manually (optionally with custom instructions)

## Source structure

```text
src/
  index.ts         # Extension entry point, event handlers
  compaction.ts    # Morph API client, compression ratio logic
  api-key.ts       # API key resolution (env var + mcp.json fallback)
  formatting.ts    # Query extraction, input/output wrapping
  file-tracking.ts # Reimplemented file tracking (not in pi public API)
  types.ts         # Shared type definitions
```
