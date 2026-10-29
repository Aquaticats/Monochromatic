# `typescript@7.1.0-beta` semantic bridge: a dead native channel crashed lint runs with uncaught EBADF at teardown

## Symptom

An oxlint run whose plugin drives the TypeScript semantic bridge
(`package/oxlint-plugin/prefer-readonly-parameter-type`)
ends with exit 1 and a bare shutdown crash instead of lint results.
The crash arrives after the rule's own work is done,
in `beforeExit`,
and it repeats on every run while the native channel stays dead.

Verbatim strings from the failing runs and their reproducing fixtures:

```text
Error: EBADF: bad file descriptor, write
Error: write EPIPE
Error: EIO
Error: ERR_STREAM_DESTROYED
SyncRpcChannel is closed
Unexpected EOF while reading from child process
```

The first failing request reports `write EPIPE` (or `EIO`,
`ERR_STREAM_DESTROYED`);
the uncaught one is `Error: EBADF: bad file descriptor, write`,
raised while shutdown itself writes to the stale channel.
The run then prints the Node.js shutdown banner and stack frames
(`Node.js v`,
`    at `),
which no plugin cleanup may emit.

The upstream trigger is a native worker death.
One measured instance kills the worker with `SIGSEGV si_addr=0x8`
(strace record,
2026-10-22);
its reduced input is unknown.
A second,
reduced instance is the `transpileModule` panic on `.d.*` file names,
which surfaces as a per-call error
`Error: panic: Debug failure. False expression: Output generation failed`
and does not kill the channel.

## Root cause

### The incident chain: dead worker, then unguarded teardown

The bridge wraps the classic synchronous API that `typescript@7.1.0-beta` ships
at `typescript/sync`
(the root `.` export of the package is only a version stub).
The installed types declare the surface directly:

```ts
// node_modules/typescript/dist/api/sync/api.d.ts:165
get transpileModule(): { ... };
```

```ts
// node_modules/typescript/dist/api/async/api.d.ts:131
transpileModule(input: string, options?: TranspileOptions): Promise<TranspileOutput>;
```

A claim that "7.1 has no `transpileModule`" is retracted:
the classic API (including `transpileModule`,
`createSourceFile`,
`close`,
and `[Symbol.dispose]`) is present and working,
verified by installed types and real invocation.

The failing chain,
measured with strace and reproduced by the fixture suite:

1. The native worker dies
   (one measured instance:
   `SIGSEGV si_addr=0x8`;
   the fixture stands in for the death with `SIGKILL`).
1. The channel's reader sees EOF and every later request fails with `write EPIPE`.
   The worker's own markers for this state are
   `SyncRpcChannel is closed`
   (`node_modules/typescript/dist/api/syncChannel.js:210`) and
   `eofError()`
   (`node_modules/typescript/dist/api/syncChannel.js:408`).
1. The plugin's `beforeExit` handler runs `closeSemanticBridge()`,
   which called `snapshot.dispose()` on the dead channel.
   The dispose issues an `apiRequest("release")`
   (`node_modules/typescript/dist/api/sync/api.js:466`,
   `node_modules/typescript/dist/api/sync/api.js:1026`)
   that writes to the stale file descriptor
   (`writeSync(21, ...)`)
   and fails with `-1 EBADF`.
1. That failure escaped as an uncaught `Error: EBADF: bad file descriptor, write`,
   exiting 1 with the shutdown banner.

Steps 1 and 2 are upstream behavior under a dead worker.
Steps 3 and 4 were the plugin's bug:
shutdown must never let a release or close request throw a bare error.

### Upstream cluster: debug.Fail panics in the native API

The native worker turns internal assert failures into Go panics.

```go
// tsc/internal/transpile/transpile.go:198 (microsoft/TypeScript, cloned 2026-10-22)
debug.Assert(hasOutputText, "Output generation failed")
```

```go
// tsc/internal/debug/debug.go:45
func Assert(value bool, message ...any) {
	if value {
		return
	}
	assertSlow(message...)
}
```

```go
// tsc/internal/debug/debug.go:7
func Fail(reason string) {
	if len(reason) == 0 {
		reason = "Debug failure."
	} else {
		reason = "Debug failure. " + reason
	}
	panic(reason)
}
```

`assertSlow`
(`tsc/internal/debug/debug.go:52`)
formats `False expression: <message>` and calls `Fail`,
so `transpileModule` on a `.d.*` input panics with
`Debug failure. False expression: Output generation failed`.

Request handling recovers such panics and returns them as call errors
(`tsc/internal/api/session.go:1337`,
`response.Error = fmt.Sprintf("panic: %v\n%s", recovered, debug.Stack())`),
which upstream's own `tsc/internal/api/session_batch_test.go:52` asserts.
That is why the measured call error reads
`Error: panic: Debug failure. False expression: Output generation failed`
and a later call on the same API instance succeeds.

Classic TypeScript 5.9 behaves differently on the same input:
it throws a catchable `Error: Debug Failure. Output generation failed`
(`typescript/lib/typescript.js:145460`,
measured with `typescript@5.9` in scratch).
The native port escalates a predictable input condition into panic control flow
with a runtime stack in the response,
while the classic surface raises an ordinary error.
The panic is recovered per request,
so this instance does not kill the channel;
the channel-killing member of the cluster is the separate `SIGSEGV`-class death,
whose input is unreduced.

## Verified fix

The consumer-side fix landed as `72773af2f5`
(8 files,
914 insertions and 205 deletions):

- `openSemanticFile`
  classifies terminal channel failures at the adapter boundary
  (`isTerminalSemanticChannelFailure`),
  retires the bridge exactly once
  (`retireSemanticBridgeAfterChannelFailure`),
  and rethrows a `SemanticBridgeError` with reason `api-unavailable`
  whose message keeps the first line of the panic text for diagnosis.
- `closeSemanticBridge()` is failure-safe:
  a snapshot release or API close failure during shutdown logs a tagged warning
  (`ignoring snapshot release failure during shutdown`,
  `ignoring API close failure during shutdown`) and never throws.
  This is the CXL requirement:
  cleanup emits no bare shutdown errors and does not filter noise,
  it logs each swallowed failure with its reason.
- The lifecycle state
  (`bridgeState`,
  `getApi`,
  `initializeSemanticBridge`)
  moved to
  `package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts`
  so both the adapter and the rule can see retirement state.

Tradeoffs,
named:

- The process fails closed.
  Once retired,
  no replacement channel starts,
  and later opens report `SemanticBridgeError` with `api-unavailable`.
  A run cannot silently continue on a half-dead channel;
  it reports the bridge as unavailable instead.
- The retirement logs one tagged line
  (`retiring semantic bridge after terminal channel failure: ...`)
  at the boundary,
  so the cause stays visible while teardown stays quiet.

Two adjacent workarounds also landed:

- The kill race in the lifecycle fixtures:
  after `SIGKILL`,
  each killed pid is waited out with a bounded
  `tail --pid=<pid> --follow /dev/null`
  before the fixture closes its stdin.
  Without it the fixture asserts against process teardown that is still running.
- The `lint:parse` gate dodge for the `.d.*` panic
  (`97e7252959`):
  declaration content is parsed under the matching non-declaration suffix
  (`parseName` in `package/monorepo-task/core-exec/mise.toml`),
  because the parse grammar is identical and parse diagnostics still report.
  Tradeoff:
  the parse probe never exercises emit for declaration file names,
  which the gate does not need.

## Verification

Versions under test:
`typescript@7.1.0-beta` (the workspace's only TypeScript),
plus `typescript@5.9` installed in scratch for the classic parity probe.
Upstream source:
a `--depth 1` clone of `microsoft/TypeScript` (2026-10-22,
`go.work`,
Go sources under `tsc/`).

Harness 1,
the lifecycle suite
`package/oxlint-plugin/prefer-readonly-parameter-type/src/semantic-bridge-teardown.unit.test.ts`:
three tests drive a real child process
(`node --input-type=module-typescript` running the built
`dist/final/node/index.mjs`),
kill the native channel the way the incident did,
and assert the CXL contract
(`not.toContain('Node.js v')`,
`not.toContain('    at ')`,
the retirement log line present).
Green run:
all three `[PASS]` in
`${HOME}/temp/agent/ebadf-teardown-2026-10-22/full-run.log` and `green-run.log`.

Harness 2,
the transpile catalogs
(`${HOME}/temp/agent/ebadf-teardown-2026-10-22/dotd-repro.ts`):

- Fails:
  `a.d.ts` and `a.d.mts` both return
  `Error: panic: Debug failure. False expression: Output generation failed`.
- Works:
  `a.ts` returns `{"outputText":"\"use strict\";\n"}`;
  after the panicking call,
  a `b.ts` call on the same instance returns
  `\"use strict\";\nconst y = 1;\n`,
  which is the measured proof that the channel keeps serving.

Guard proof for the fix
(GFP):
the guard was committed,
then its assertions were neutered
(`${HOME}/temp/agent/ebadf-teardown-2026-10-22/gfp-neuter-classifier.patch`),
and the suite reds were captured:
`red-run-classifier.log`
(exit 1,
`expected false to equal true` against
`isTerminalSemanticChannelFailure({ error: { message: 'write EPIPE', code: 'EPIPE' } })`)
and `red-run.log`
(exit 1,
`expected 'RESTART-ALLOWED\n...' to include 'RESTART-BLOCKED: api-unavailable'`).
Restoring the guards returned the suite green.

Guard proof for the upstream prototype
(see "Upstream filing decision"):
the targeted Go test
`tsc/internal/transpile/dotd_panic_test.go`
in the disposable clone panicked pre-patch
(`panic: Debug failure. False expression: Output generation failed`
through `transpile.go:198`,
`debug.go:14`)
and passed post-patch.

Topology and kill-race measurements used for the fixtures
(two `tsc` children per fixture,
`pgrep --parent <pid>` rather than `pgrep --ppid`,
and the changed-overlay text requirement for channel-liveness probes)
were measured by the session's subagent
(`f991d06f-...`,
2026-10-22) and are recorded in
`topology-probe.ts` and `kill-probe.ts` under the same scratch directory.

## What does not work

- Filtering the shutdown noise.
  CXL forbids it;
  hiding the banner hides the cause.
  The fix logs swallowed failures with reasons instead.
- Retiring the bridge from the rule's visitor catch block.
  Superseded by adapter-boundary retirement (PP4);
  the visitor wiring stood down and the visitor stays at HEAD.
- `pgrep --ppid`.
  Unrecognized on procps-ng here;
  `pgrep --parent <pid>` works.
- Reopening an identical overlay text as a channel-liveness probe.
  The change is cache-served;
  the probe needs changed overlay text.
- Assuming `typescript@7.1.0-beta` has no `transpileModule`.
  Retracted;
  the classic surface is at `typescript/sync` and `typescript/async`.
- Treating the `.d.*` transpile panic as the incident's trigger.
  Separate incidents (XIC):
  the panic is recovered per request and the channel keeps serving,
  while the incident's worker death killed the process.
- Waiting for a quiet tree before sweeps.
  The tree is never quiet;
  sweep git-clean paths from a pre-run `git status --short` snapshot.

## Upstream filing decision

Upstream is `microsoft/TypeScript`
(the native preview compiler and the `typescript/sync` API surface).

### Exemption and duplicate checks

`.out-of-scope/` holds `typescript-project-references.md`
(project references) and `low-impact-typescript-formatting.md`
(formatter rules);
neither covers compiler panics,
so no exemption applies.

Duplicate searches over the tracker
(2026-10-22,
unlimited open/closed):

- `gh search issues --repo microsoft/TypeScript "native preview segfault api"`:
  no results.
- `gh search issues --repo microsoft/TypeScript "segfault native preview sync"`:
  no results.
- `gh search issues --repo microsoft/TypeScript "transpileModule panic"`:
  no results.
- `gh search issues --repo microsoft/TypeScript "transpileModule crash"`:
  one hit,
  issue 20333 (closed 2018,
  a fixed unrelated crash).

Contribution policy
(`CONTRIBUTING.md`,
read 2026-10-22):
AI-assisted work is allowed with disclosure in the PR description;
bulk agent-driven contributions are banned;
automated comments are banned;
autonomous agents may open a PR only when a specific human operator chose
that issue and shepherds review themselves.

### Constraint walk: the .d.* transpile panic

1. Upstream's fault:
   yes.
   A documented input shape
   (`Options.FileName` documents extension semantics)
   triggers a worker panic instead of a normal error.
1. Upstream can fix it:
   yes.
   Two fix shapes are demonstrated below.
1. Are they supporting this use case:
   yes.
   The classic API surface ships in 7.1 and classic accepts the input
   with a catchable error.
1. Would they fix it and welcome the contribution:
   soft yes,
   conditional.
   AI assistance must be disclosed,
   a human operator must choose the issue and shepherd it,
   and bulk or automated filing is banned.
1. Will they likely fix it:
   soft yes.
   A panic on accepted input is a robustness fix with no `wontfix` signal
   in the searches above.
1. Did we prototype the minimal fix:
   yes,
   in a disposable clone
   (`${HOME}/temp/agent/typescript-prototype-2026-10-22`,
   `--depth 1`).
   The targeted test panicked pre-patch and passed post-patch.

All six hold,
so the draft below is fileable once a human operator reviews it,
discloses the AI assistance as the policy requires,
and shepherds the discussion.
Filing itself stays a human action.

The prototype patch
(two shapes fix the panic;
maintainers choose the contract):

```diff
// tsc/internal/transpile/transpile.go (prototype shape: empty output)
-	debug.Assert(hasOutputText, "Output generation failed")
+	// Declaration inputs have no JavaScript to emit, so emit produces no output
+	// file for them; return empty output text instead of asserting one exists.
+	if !hasOutputText {
+		return &Output{
+			OutputText:    "",
+			Diagnostics:   allDiagnostics,
+			SourceMapText: sourceMapText,
+		}
+	}
```

The alternative shape preserves classic parity exactly:
keep the Debug Failure as the call error,
but raise it as an ordinary error return instead of `panic`
(or recover at the request boundary and map it to a typed error),
so the response carries the failure without runtime panic semantics.
Both shapes remove the worker panic.

### Constraint walk: the unreduced worker death

1. Upstream's fault:
   yes,
   for the death itself
   (`SIGSEGV si_addr=0x8` in the native worker,
   strace-measured).
1. Upstream can fix it:
   presumably yes,
   but the failing code path is unknown.
1. Are they supporting this use case:
   soft yes,
   the sync API surface is shipped and documented.
1. Would they fix it and welcome the contribution:
   same conditional soft yes as above.
1. Will they likely fix it:
   soft yes,
   a segfault fix,
   no `wontfix` signal.
1. Did we prototype the minimal fix:
   no,
   and prototyping was attempted then abandoned.
   Reduction was attempted with strace and the fixture suite;
   the fixtures reproduce the teardown chain
   by standing in for the death with `SIGKILL`,
   not the death itself.
   The signal record gives `si_addr=0x8` and no symbolized stack,
   so the failing path cannot be located,
   and a patch could not be verified pre-patch and post-patch.

Constraint 6 does not hold,
so nothing is filed for this member and the decision is
do not file as-is.
The verified containment is the consumer-side fix in
`72773af2f5`.

### Draft issue

The draft for the `.d.*` panic,
kept as an auditable record:

~~~md
title: `transpileModule` panics the sync API worker for `.d.*` file names where classic `ts.transpileModule` throws a catchable error

Affected version: `typescript@7.1.0-beta` (native preview, `typescript/sync` API).

Disclosure: this report is AI-assisted (DeepSeek Harness session).
The reproduction commands,
source trace,
prototype patch,
and workaround checks below are mechanical and reproducible;
they were not yet reviewed by the human operator who would shepherd the issue.

### Reproduction

```ts
import { API } from 'typescript/sync';
const api = new API();
const out = api.transpileModule('declare const x: number;\n', {
  fileName: 'a.d.ts',
  reportDiagnostics: true,
});
```

Observed:
the call errors with
`Error: panic: Debug failure. False expression: Output generation failed`,
and the response embeds a runtime stack.
`a.d.mts` and `a.d.cts` behave the same.
A later call on the same instance succeeds,
so the panic is recovered per request
(`tsc/internal/api/session.go:1337`).

### Expected

Classic `typescript@5.9` throws
`Error: Debug Failure. Output generation failed`
at `transpileWorker`
(`typescript/lib/typescript.js:145460`)
for the same input:
an ordinary catchable error,
no panic semantics,
no runtime stack in the message.
The native surface should raise the same failure as a normal call error.
Whether `.d.*` inputs should instead return empty output text
(a declaration file has no JavaScript to emit)
is a contract question for you;
either shape removes the panic.

### Source trace

`tsc/internal/transpile/transpile.go:198`
`debug.Assert(hasOutputText, "Output generation failed")`
for an emit that produces no output file,
via `tsc/internal/debug/debug.go:45` (`Assert`),
`:52` (`assertSlow`),
`:7` (`Fail`),
`:14` (`panic(reason)`).

### Minimal fix prototype

Tested in a `--depth 1` clone on 2026-10-22:
`tsc/internal/transpile/dotd_panic_test.go`
panics pre-patch through `transpile.go:198` and passes post-patch.

```diff
-	debug.Assert(hasOutputText, "Output generation failed")
+	if !hasOutputText {
+		return &Output{
+			OutputText:    "",
+			Diagnostics:   allDiagnostics,
+			SourceMapText: sourceMapText,
+		}
+	}
```

### Workaround in use

Parse `.d.*` content under the matching non-declaration suffix
(`.d.ts` to `.ts`),
which dodges the panic because the parse grammar is identical.
~~~

## Out-of-scope check

The failure mode is not a documented limitation of this repository.
`.out-of-scope/` holds no exemption covering TypeScript compiler panics
or semantic-bridge teardown;
the two TypeScript entries cover project references and formatter rules
only.
The consumer-side containment is therefore in scope and landed
(`72773af2f5`),
and the upstream items stay on record above.
