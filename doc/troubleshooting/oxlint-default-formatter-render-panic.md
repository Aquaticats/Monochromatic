# Oxlint 1.87.0 aborts in its default formatter for some files that have type-aware warnings

## Symptom

`oxlint --format=default <file>` (the format `mise run //package/git-policy/cli:lint:oxlint:paths` uses) aborts with
status `134` and prints, instead of the diagnostics:

```text
thread 'tokio-rt-worker' panicked at apps/oxlint/src/output_formatter/default.rs:181:14:
called `Result::unwrap()` on an `Err` value: Error
```

The task then fails with `[task-oxlint] failed to execute oxlint: Command was terminated with SIGABRT`.
The same file with `--format=unix` prints every diagnostic and exits normally,
so the lint run itself succeeds and only the rendering fails.
Files without diagnostics, and most files with diagnostics, are unaffected.

## Reproduction

Observed on Oxlint `1.87.0` (`mise exec -- oxlint --version`) with the repository's configuration,
by saving this file as `package/git-policy/cli/bin/scratch.mjs` and running
`mise exec -- oxlint --format=default package/git-policy/cli/bin/scratch.mjs` from the repository root:

```js
// package/git-policy/cli/bin/scratch.mjs: aborts with --format=default, prints 12 problems with --format=unix
/// <reference types="node" />
import { once } from 'node:events';
import { spawn } from 'node:child_process';

async function readText(stream) {
  const chunks = await Array.fromAsync(stream);
  return Buffer.concat(chunks.map(function toBuffer(chunk) {
    return Buffer.from(chunk);
  }))
    .toString('utf8');
}

export async function observe(command) {
  const child = spawn(command, [], { stdio: ['ignore', 'pipe', 'pipe'] });
  const [stdout, stderr, closed] = await Promise.all([
    readText(child.stdout),
    readText(child.stderr),
    once(child, 'close'),
  ]);
  const code = /** @type {number | null} */ (closed[0]);
  if (code === null) throw new Error(stderr);
  return { code, stdout, stderr };
}
```

Removing either function leaves a file that renders normally (twelve warnings for `readText` alone,
five for the `once` call alone), so the trigger needs both together.
The diagnostics are type-aware warnings about values typed `error` or `any`:
`Array.fromAsync` is unknown to the TypeScript library the linter loads here.

## What is known and what is not

The unwrap at the reported line is in the formatter's rendering of the collected diagnostics.
On the upstream `main` branch the same function calls `self.handler.render_reports(...)` and unwraps its `Result`
(`apps/oxlint/src/output_formatter/default.rs`, `render_errors`),
so a rendering error from the report handler becomes an abort.
Which diagnostic makes the handler fail was not isolated;
the minimal files above render normally, and the failure appears only for the combination.

## Verified workaround

Run the same paths with the unix format and read the diagnostics from it:

```sh
# doc/troubleshooting/oxlint-default-formatter-render-panic.md: same lint, different output format
mise exec -- oxlint --format=unix package/git-policy/cli/bin/scratch.mjs
```

A file whose rendering aborts is also a file with warnings the task would have shown,
so fix the warnings in the unix output and rerun the task; once the warnings are gone the default format has nothing
to render and the task passes.
