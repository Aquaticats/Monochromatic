# oxlint 1.92.0 reports no `prefer-readonly-parameter-types` findings once its TypeScript bridge child exits with "Unexpected EOF"

## Symptom

`mise run //package/typeface/aquaticat:lint:oxlint` exits 0 and prints
`Found 0 warnings and 0 errors` while the package still holds findings of the
repository's own readonly rule.
The run's stderr carries two warnings and then one warning per linted file:

```text
[warn] [prefer-readonly-parameter-types] [retireSemanticBridgeAfterChannelFailure] retiring semantic bridge after terminal channel failure: Error: Unexpected EOF while reading from child process (unknown reason)
[warn] [preference] semantic evidence unavailable for .../package/typeface/aquaticat/task.ts: api-unavailable: SemanticBridgeError: TypeScript 7.1.0-beta synchronous API stopped accepting requests: Error: Unexpected EOF while reading from child process (unknown reason)
[warn] [preference] semantic evidence unavailable for .../package/typeface/aquaticat/src/build-font-resolve-points.ts: api-unavailable: SemanticBridgeError: TypeScript 7.1.0-beta synchronous API was retired after a terminal channel failure; this process does not start a replacement channel.
```

The exit status stays 0 because the rule reports nothing,
not because the code
is clean.
A run that loses its bridge and a run that finds nothing look identical in the
summary line.

## Root cause

1.  The plugin keeps one process-local bridge.
    `package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts:63` declares it:

    ```ts
    export const bridgeState: {
      api: API | typeof NO_API;
      ...
      channelRetired: boolean;
    } = {
    ```

2.  `getApi` creates the TypeScript 7 synchronous API once and registers a
    process-lifetime cleanup hook (`semantic-bridge-lifecycle.ts:121` and `:143`):

    ```ts
    const api = new API({
      cwd: process.cwd(),
      fs: overlayFileSystem({ overlays: bridgeState.overlays, },),
    },);
    ...
    process.once(
      'beforeExit',
      closeSemanticBridge,
    );
    ```

3.  That API talks to a native child process,
    which the plugin reaches through
    TypeScript's unstable channel
    (`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/typescript-sync-native-shutdown.ts:94`):

    ```ts
    nativeApiChild(api).kill('SIGKILL');
    ```

    When the child exits,
    its pipe yields the `Unexpected EOF while reading from
    child process` message above.

4.  The first terminal failure retires the bridge for the rest of the process
    (`semantic-bridge-lifecycle.ts:184` and `:199`):

    ```ts
    export function retireSemanticBridgeAfterChannelFailure({
      error,
    }: {
      readonly error: unknown;
    },): void {
      if (bridgeState.channelRetired)
        return;
      bridgeState.channelRetired = true;
      ...
      rl.warn(`retiring semantic bridge after terminal channel failure: ${String(error,)}`);
    ```

    Every later request fails closed instead of starting a replacement channel
    (`semantic-bridge-lifecycle.ts:101`):

    ```ts
    if (bridgeState.channelRetired)
      throw new SemanticBridgeError({
        reason: 'api-unavailable',
        message: `TypeScript ${typescriptVersion} synchronous API was retired after a terminal channel failure; this process does not start a replacement channel.`,
      },);
    ```

5.  The rule catches that per file and skips the file,
    so nothing is reported
    (`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/readonly-rule-visitor.ts:109`):

    ```ts
    catch (error) {
      rl.warn(
        `semantic evidence unavailable for ${context.filename}: ${
          error instanceof SemanticBridgeError
            ? `${error.reason}: ${caughtValueStack(error,)}`
            : caughtValueStack(error,)
        }`,
      );
    }
    ```

What makes the child exit is not established here.
The measured correlation was multi-threaded runs,
and it did not hold later
(see Verification),
so no cause is claimed.

An earlier reading of this incident was wrong and is recorded so it is not
re-derived:
the first four failures all had 13 linted files and a fifth run with
12 files was clean,
which suggested a file-count threshold.
A later tree with 18 linted files ran clean five times in a row at three workers,
so the count is not the trigger.

## Verification

Versions under test:

- `oxlint` 1.92.0,
  from `node_modules/oxlint/package.json`.
- `typescript` 7.1.0-beta,
  from `node_modules/typescript/package.json`;
  the
  bridge asserts this major (`typescript-sync-native-shutdown.ts`,
  `assertTypeScriptSeven`).
- The plugin and the root task configuration at commit `a1effd4dc5`.

Harness,
one run per invocation,
capped in its own cgroup:

```bash
systemd-run --user --scope -p CPUQuota=300% -p MemoryMax=8G -p TasksMax=512 -p IOWeight=10 \
  -- timeout 900 mise run //package/typeface/aquaticat:lint:oxlint
```

`OXLINT_THREADS` is read by the wrapper
(`package/monorepo-task/core-exec/src/oxlint-wrapper.ts`,
"Thread count override
from environment") and passed to oxlint as `--threads`.

Catalog that lost the bridge,
all on a tree holding 13 linted files and two real
`prefer-readonly-parameter-types` findings:

- four consecutive runs with the default worker count (3 on this 4-CPU host),
  each printing `Found 0 warnings and 0 errors` and the EOF warning.
- one run with the third file replaced by a trivial suite that imported nothing,
  which also printed the EOF warning.
- one run with a non-test probe module added instead,
  which also printed the EOF
  warning.

Catalog that kept the bridge:

- `OXLINT_THREADS=1` on the same 13-file tree,
  which reported both findings
  (`Found 0 warnings and 2 errors`).
- the same tree with 12 linted files at the default worker count,
  which reported
  both findings.
- after later source edits,
  an 18-file tree at the default worker count,
  which
  printed no EOF warning in five consecutive runs.

## Verified workarounds

Run the package's lint with one oxlint worker:

```bash
OXLINT_THREADS=1 mise run //package/typeface/aquaticat:lint:oxlint
```

Tradeoffs:
the run uses one worker (measured 1.1 s to 2.5 s for 18 files against
0.8 s to 1.1 s with three workers on this host),
and the variable only reaches
oxlint through `oxlint-wrapper.ts`.
It makes the rule's findings appear;
it does not explain or prevent the child's
exit.

The repository-wide `lint` task already sets the same variable for its own
reasons (`mise.toml:863`),
so `mise run lint` never showed this symptom.
A per-package `lint:oxlint` task inherits nothing from that task's environment,
because `mise.toml` declares the variable on `[tasks."lint".env]` alone.

## What does not work

- Raising the runner's limits:
  the EOF appeared with `MemoryMax=20G`,
  `TasksMax=2048`,
  and `CPUQuota=350%`,
  so the child is not being killed by the
  runner's cgroup.
- Reading the child's own diagnostic:
  the sync API reports only
  `Unexpected EOF while reading from child process (unknown reason)`;
  nothing
  names the child's exit status or signal.
- Blaming a source construct in the linted files:
  a trivial third test file and a
  non-test probe module both reproduced it.
- Blaming the file count:
  the 13-file runs failed and the later 18-file runs were
  clean at the same worker count.

## Upstream filing decision

The tool is this repository's own
`package/oxlint-plugin/prefer-readonly-parameter-type`,
not a third-party
project,
so the upstream-filing constraints do not apply and nothing is filed
anywhere.
The fix belongs in that package,
and possibly in the root `lint:oxlint` task
template so a per-package run passes `OXLINT_THREADS` the way `mise run lint`
already does.

Open questions for whoever owns that package:

- What makes the `tsgo --api` child exit,
  and does a worker thread's teardown
  close a child another worker is still using?
- Should a retired bridge make the run fail rather than report zero findings?
  Silent success after a lost channel is the failure mode this document exists
  for.
