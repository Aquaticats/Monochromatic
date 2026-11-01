# Node 26.4.0 execFileSync ENOBUFS when mise task JSON exceeds default buffer

## Pandoc renderer recurrence on Node 26.12.0

On 2026-10-18,
the private Depot research renderer invoked Pandoc through `execFileSync`
with its default output buffer.
The JSON AST for `doc/handover/depot-trial-grilling.md` exceeded that bound,
and Node reported `Error: spawnSync pandoc ENOBUFS` with child signal `SIGTERM`.
This was a parent capture limit,
not a Markdown parse failure or a reason to accept a stale rendered output.

The read-only Node 26.12.0 source checkout resolves to
`26e72d85a611e222e3f06025de8a3c496923bacd`.
`lib/child_process.js:99` sets the bound:

```js
// lib/child_process.js:99
const MAX_BUFFER = 1024 * 1024;
```

`lib/child_process.js:931` supplies that default to synchronous spawning:

```js
// lib/child_process.js:931
function spawnSync(file, args, options) {
  options = {
    __proto__: null,
    maxBuffer: MAX_BUFFER,
    ...normalizeSpawnArguments(file, args, options),
  };
```

`src/spawn_sync.cc:648` detects aggregate captured-output overflow and terminates capture:

```cpp
// src/spawn_sync.cc:648
void SyncProcessRunner::IncrementBufferSizeAndCheckOverflow(ssize_t length) {
  buffered_output_size_ += length;
  if (max_buffer_ > 0 && buffered_output_size_ > max_buffer_) {
    SetError(UV_ENOBUFS);
    Kill();
  }
}
```

The renderer follows the existing bounded-buffer handling in this document:
both HTML and AST calls use a named 16 MiB bound,
then the complete AST is parsed before link/emphasis checks.

```js
// Private research mise.toml, render-root-docs task.
const PANDOC_MAX_BUFFER = 16 * 1024 * 1024;
const astBody = execFileSync('pandoc', ['--from=commonmark', '--to=json', file], {
  cwd: root,
  encoding: 'utf8',
  maxBuffer: PANDOC_MAX_BUFFER,
});
const ast = JSON.parse(astBody);
```

The tradeoff remains a bounded in-memory capture;
it does not make arbitrary output safe or remove the need to verify complete parsing.
The default-bound failing catalog and explicit-bound passing catalog concern the same unchanged Markdown input.
The bounded rerun exits zero for all fourteen documents.
The handover AST is 1,067,303 bytes and its HTML is 215,831 bytes;
complete JSON parsing finds no unintended emphasis or missing relative links.
Actual rendered text is still read as a separate document verification step.
No upstream patch is needed:
the caller owns its capture policy and the supported option addresses this failure.
The existing upstream-filing decision remains applicable.

## Symptom

Package fanout tasks failed while running a package build through mise:

```text
Error: spawnSync mise ENOBUFS
spawnargs: [ 'tasks', '--all', '--hidden', '--json' ]
```

The failing surface was:

```sh
mise run //package/agent-harness-shared/terminal-title:buildAndTest
```

The failure happened before `tsdown` ran because the task fanout helper could not read
`mise tasks --all --hidden --json` through Node `execFileSync`.

## Root cause

The workspace task graph is large enough that `mise tasks --all --hidden --json` is over
Node's default `execFileSync` stdout buffer.
The measured output size was:

```sh
cd /var/home/user/Monochromatic && mise tasks --all --hidden --json | wc --bytes
# 1060629
```

The default Node behavior was reproduced with a one-byte-over-1-MiB child stdout:

```sh
node --input-type=module --eval "import { execFileSync } from 'node:child_process'; try { execFileSync(process.execPath, ['--eval', 'process.stdout.write(\"a\".repeat(1048577))']); } catch (error) { console.log(error.code); console.log(error.message.split('\n')[0]); }"
# ENOBUFS
# spawnSync /var/home/user/.local/share/mise/installs/node/26.4.0/bin/node ENOBUFS
```

`mise.no-env.toml:302` and `mise.no-env.toml:303` now set a larger buffer for the package-local
fanout helper:

```toml
const MISE_TASKS_JSON_MAX_BUFFER = 16 * 1024 * 1024
const tasks = JSON.parse(execFileSync('mise', ['tasks', '--all', '--hidden', '--json'], { encoding: 'utf8', maxBuffer: MISE_TASKS_JSON_MAX_BUFFER }))
```

`mise.no-env.toml:334` and `mise.no-env.toml:335` apply the same fix to cross-package fanout.
`mise.no-env.toml:766` and `mise.no-env.toml:767` apply it to the `buildAndTest` package lookup.
`mise.no-env.toml:934` and `mise.no-env.toml:935` apply it to `prepare` fanout.

`mise.toml` is generated from `mise.no-env.toml` by file-enforcer,
so `mise run sync:files` must run after edits to `mise.no-env.toml`.

## Verification

The minimal Node reproduction fails without `maxBuffer` and succeeds with the workspace buffer:

```sh
node --input-type=module --eval "import { execFileSync } from 'node:child_process'; const output = execFileSync(process.execPath, ['--eval', 'process.stdout.write(\"a\".repeat(1048577))'], { maxBuffer: 16 * 1024 * 1024 }); console.log(output.length);"
# 1048577
```

The workspace verification that originally failed now passes:

```sh
cd /var/home/user/Monochromatic && mise run buildAndTest -- package/agent-harness-shared/terminal-title/src/index.unit.test.ts
# Build complete
# PASS truncate, shortPath, stringField, field, pathFormat, quotedFormat, shortCommand,
# stripCommandNoise, lookupToolTitleEntry, formatToolTitle, prefixedTitle
```

## Verified workarounds

### Set `maxBuffer` at every task-list scan

Use a named buffer constant in each Node task body that reads the full mise task graph:

```js
const MISE_TASKS_JSON_MAX_BUFFER = 16 * 1024 * 1024;
const tasks = JSON.parse(execFileSync('mise', ['tasks', '--all', '--hidden', '--json'], {
  encoding: 'utf8',
  maxBuffer: MISE_TASKS_JSON_MAX_BUFFER,
}));
```

Tradeoff:
large task JSON is still buffered in memory.
The current task graph is about 1.01 MiB,
so 16 MiB leaves growth room without making this an unbounded read.

## What does not work

### Leaving `execFileSync` at its default buffer

The default buffer fails once stdout exceeds 1 MiB,
as shown by the minimal Node reproduction in the root cause section.

### Fixing only package-local fanout

The same task-list scan pattern also appears in cross-package fanout,
`buildAndTest` lookup,
and `prepare` fanout.
Leaving any of those at the default buffer keeps a delayed ENOBUFS failure path.

## Upstream filing decision

Nothing to file upstream.

- Is it really upstream's fault?
  No.
  Node exposes `maxBuffer`,
  and the workspace task helper omitted it for a large stdout-producing command.
- Can upstream fix it?
  Not applicable because the workspace script owns the buffer policy.
- Are they supporting this use case?
  Not needed for an upstream issue because this is a caller-side buffer setting.
- Would the repo welcome our contribution?
  Not applicable.
- Will they likely fix it?
  Not applicable.
- Have we prototyped a minimal fix compatible with their architecture?
  Yes,
  the workspace-side fix was applied in `mise.no-env.toml`,
  regenerated into `mise.toml`,
  and verified with `mise run buildAndTest -- package/agent-harness-shared/terminal-title/src/index.unit.test.ts`.
