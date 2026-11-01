# cli-git logger sink verify timeout

Issue [#573](https://github.com/Aquaticat/Monochromatic/issues/573) was closed at the owner's request
with the comment "can't reproduce".
The investigation identified the Node default sink and validated a timeout detector,
but did not reproduce the reported delay without fault injection.
The cause remains unknown;
no production behavior or verification deadline was changed.
The investigation is concluded.

## Symptom

The original 2026-09-26 report recorded this line on every `git` command in that session.
Issue #573 also records repeated occurrences during commits on 2026-09-25.
The warning was not reproduced by ordinary current-checkout commits on 2026-10-04:


```text
logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify
```

Git itself still completes,
 and commits land normally.
Long tool calls that budget tightly around `git`
 can still time out because the hook path waits out the verify window first.

## Emitter

`@monochromatic-dev/logger-core`,
 from `package/logger/core/src/create-logger.ts`:
 `verifyAndApply` runs each sink's `verify()` under `withHostTimeout`
 with `verifyTimeoutMs` (5000 ms),
 and a timed-out verify reports the quoted diagnostic,
 then drops that sink
 (`markEntryUnavailable`),
 so the records that sink would carry are lost for that process.

## Working diagnosis

In the reported invocations,
entry 3's verification did not complete before its 5000 ms deadline.
That index identifies the fourth sink in the emitting logger instance.
The reporting session described the failure as persistent across invocations.
That observation does not establish whether host load contributed.

The original investigation did not identify the sink or emitting process.
The 2026-10-04 source trace identifies the Node default entry as the file sink,
but does not establish which verification operation exceeded its deadline.
A timeout also does not prove an operation is permanently hung.
This is a separate incident from
 [module-logger-quickjs-console](module-logger-quickjs-console.md),
 whose sink failures came from a QuickJS host without timers.

## Also seen under host load in `mise` / `node` runs

The same emitter fires outside the cli-git hook surface:
 plain `mise run //package/...:test:unit` and `node <file>.unit.test.ts`
 runs printed the identical line during a session where the host was
 swapping under concurrent mutation-test containers
 (`free -m` showed 39 GiB used and 30 GiB swap used),
 and the line vanished once the host load dropped.

On 2026-10-06 one `git commit` printed the line once,
in a session where other work had held the host's load average at about 35 to 60 an hour before
(it was not measured at the moment of the commit,
and read 66 shortly after);
the commit landed,
and the next commits did not print it.

This transient,
load-correlated observation remains separate from issue #573 until their process and trigger boundaries match.
An identical timeout diagnostic proves a shared reporting path,
not a shared root cause.
Do not saturate the host to recreate it;
any load experiment must use a bounded disposable container.

On 2026-10-26 the line appeared twice during `node src/package/mise.container-test.ts`
container-matrix runs in `package/file-enforcer/core`
(before that run's resource bounds landed):

```text
logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify
```

It preceded a SIGKILL of the first run
and a SIGTERM of the second.
The same line appears in pre-change baselines of those runs,
so the matrix edits neither introduced nor removed it.
The runs were host-exhausting at the time
(the first grew to 42 GB RSS,
recorded in `doc/planning/mpm-toolchain-migration.md`),
and the bounded rerun
(`podman run --memory=2g --cpus=2`,
`package/test-harness/core-matrix/src/container.ts`)
was green.
The emitter is again load-correlated and still not root-caused here.

## Fake-timer cross-domain clear deadlock on 2026-10-18

A separate incident with the same emitter,
bounded by a whole-file deadlock instead of a timeout diagnostic line.
During the issue #1762 investigation
(`doc/handover/wall-clock-bound-unit-tests.md`),
a probe against the built harness dist installed sinon fake timers
(`toFake: ['setTimeout', 'clearTimeout']`)
at the start of a test attempt and deterministically deadlocked:
the entire test file hung,
no `[FAIL]` diagnostic was emitted,
and node exited 13 with
`Warning: Detected unsettled top-level await`.
The preceding stderr carried sinon's
`FakeTimers: clearTimeout was invoked to clear a native timer instead of one created by this library. To automatically clean-up native timers, use shouldClearNativeTimers.`
emitted by `clearTimer` in `@sinonjs/fake-timers`,
surfaced through the bundled harness path
`withTimeout` dispose,
`withHostTimeout`,
`verifyAndApply`,
logger `initialize`:
the same `verifyAndApply` sink-verification path this doc names as the emitter.

Mechanism:
bootstrap schedules each sink verification deadline with the native `setTimeout`;
a test attempt that replaces global `clearTimeout` while a verification is still in flight
leaves the completion disposer calling the replacement with the native handle;
`@sinonjs/fake-timers` throws by design on that cross-domain clear;
the rejection escapes the logger initialization path and never surfaces as a test failure.
The host-load section of this doc matters here:
verification in flight for seconds under load
widens the window in which an early fake-timer install can collide with it.
That makes the deadlock a latent,
load-sensitive hazard for the default-run files that install fake timers
(`package/test-harness/core/src/sinon-clock.unit.test.ts`,
`package/logger/core/src/sink/file-batch.unit.test.ts`,
`package/test-harness/core-sandbox/src/sandbox.unit.test.ts`);
no default-run failure has been observed,
and the issue #1762 five-batch 32-process repro was green.

Workaround verified in the probes:
pass `shouldClearNativeTimers: true` to `sinon.useFakeTimers`
so the fake clock forwards native clears instead of throwing;
both probe files then ran green to completion (exit 0).
Root-fix direction,
not implemented at this writing:
`withTimeout`
(`package/async-time/core/src/with-timeout.ts`)
should capture schedule and clear from one clock domain
so a global replacement mid-race cannot cross domains.
The owner paused fix implementation while the issue #1762 remedy design settles.

## Workarounds and rejected interpretations

No fix or workaround for the underlying verification delay is verified.
Increasing a command runner's timeout does not change the logger's 5000 ms limit.
Retrying starts another process with fresh sink verification;
the original assertion that a retry reused the timed-out logger was incorrect.
Before retrying a state-changing Git command,
inspect its result because the commit can have landed despite the warning.

Suppressing the warning or raising the verification deadline without identifying the delayed operation
would hide or postpone the symptom rather than establish its cause.

## Investigation for issue #573 on 2026-10-04

`package/logger/core/src/default-sinks.node.ts:37` constructs the Node default set in this order:

```ts
// package/logger/core/src/default-sinks.node.ts
return [
  createConsoleSink(),
  createSessionStorageSink(),
  createLocalStorageSink(),
  createFileSink(),
];
```

Entry 3 is therefore the file sink for a process using this default set.
The emitting process in the reported commit remains unidentified.

At baseline commit `d62bf6905`,
a disposable repository with its own `node_modules` completed both wrapper `status --short`
and an explicit-path `commit` with exit code 0 and empty stderr.
Both processes created JSONL files containing logger records.
This fixture did not load the repository policies,
so it is a negative control rather than a reproduction of the reported commit.
Current working-repository `status` and `log` commands also did not emit the warning.

A streaming directory enumeration counted 882885 entries in the working repository's
`node_modules/.monochromatic`.
That count alone does not establish latency or causation;
the failing process's actual sink directory must be identified first.
No existing log files were removed or changed by the enumeration.

Follow-up fixtures loaded `markdown/autofix`,
`mono/forbidden-root-context`,
and `security/forbidden-strings`,
using disposable Git repositories and homes on the host filesystem.
Both root and linked-worktree commits completed with exit code 0 and empty stderr.
The linked-worktree fixture invoked the actual shadow `git` executable.
The real documentation commit `10554b905` also completed without the warning.
These successful observations do not prove an intermittent incident is fixed.

The detector's positive control delayed only the file sink's `mkdir` by 5100 ms.
It captured the exact issue diagnostic and failed the probe as expected.
The trace identified the wrapper process,
its temporary log directory,
and the pending `mkdir`.
This validates timeout detection,
not the incident's root cause.

### Historical endpoint investigation

The owner requested bisection after the agent prematurely asked for a fresh reproduction.
The missed step was testing historical revisions before treating current non-reproduction as a blocker.

Candidate endpoints:

- `77992cfed`:
  first-parent revision before 2026-09-21,
  not yet a proven good revision.
- `d322d083e`:
  the recorded concurrent-commit decision commit associated with the report,
  not yet a failing executable probe.
  Its only change from its parent is the decision document,
  so its executable source matches its parent's.

`git diff d322d083e HEAD -- package/logger/core package/async-time/core` is empty.
The logger and timeout source did not change between that reported revision and this investigation.
Commit `ee58222bc` introduced the verification deadline on 2026-09-06,
before the issue's reported onset window.
The September 23 timerless-host changes are not an established cause;
the endpoint and positive-control results constrain that hypothesis.

An isolated sparse worktree at `d322d083e` rebuilt the wrapper with
`mise run //package/git-policy/cli:build:js:node`.
Its configured root and linked-worktree commits also passed without the warning.
Historical source and package-local build configuration are used;
Node 26.10.0,
Git 2.55.0,
installed external dependencies,
and the forbidden-strings scanner are held constant.
This isolates source changes but does not recreate the full September 25 runtime environment.

The completed traced comparison ran four rotated trials per endpoint.
Each trial rebuilt the selected wrapper and committed in both a disposable main repository and linked worktree.
All 16 commits exited successfully with empty stderr.
Every trial produced five JSONL files containing 38 lines,
so file logging was exercised rather than silently absent.

The 5100 ms positive control was also run against freshly rebuilt wrappers at `e8880898d^` and `e8880898d`.
Both produced the exact 5000 ms verification diagnostic.
This establishes that the September 23 change did not introduce the diagnostic's basic timed-Node behavior;
it does not explain the historical operation delay.

A prior repeated-run attempt was interrupted by a 30-second command-runner deadline during trust setup,
without a sink verification diagnostic.
At inspection the host load average was 65.79 and multiple Node processes were waiting on I/O.
The complete comparison then passed with a 90-second setup budget.
The logger's own deadline remained 5000 ms throughout.
The setup timeout is not a reproduction of issue #573.

The uninstrumented comparison repeated the same four rotated trials per endpoint,
with the filesystem tracer disabled.
All 16 commits again had empty stderr,
and every trial again produced five log files with 38 lines.
Across the traced and uninstrumented comparisons,
32 commits passed without the issue diagnostic.

No discriminating endpoint was established,
so no `git bisect good` or `git bisect bad` classification was recorded.
Silence cannot classify an intermittent revision as good,
and the injected delay cannot classify it as bad because it intentionally forces a timeout.
Any renewed investigation would need an actual failing invocation captured with its process and filesystem stage,
or a controlled workload that reproduces a revision-dependent difference.

### Retained historical log window

A read-only inspection enumerated the root log directory and selected all 2225 retained files
whose names fall between 2026-09-26 01:34 UTC and 01:40 UTC,
surrounding the recorded decision commit.
No selected file exceeded the 2 MB read budget,
so every selected file was read.

Four files mention `cli-git-concurrent-commits`:
three concern a main-worktree handover,
and one records root discovery for the linked worktree.
They do not identify the process that emitted the timeout.
The deleted linked worktree's own log directory was not available.
These retained root logs cannot establish that the failing process completed verification.

### Local diagnostic artifacts

The investigation's scripts and raw outputs are retained under `${HOME}/temp/agent/`,
with the `issue-573-` prefix:

- `issue-573-probe.ts`:
  disposable main and linked-worktree commits with the reported policies.
- `issue-573-trace.mjs`:
  filesystem verification tracing and the optional delayed-`mkdir` positive control.
- `issue-573-link-deps.ts`:
  isolated historical workspace source links with current external dependencies held constant.
- `issue-573-history.ts`:
  rotated endpoint builds and probes.
- `issue-573-history-1791123474130.jsonl`:
  completed traced comparison's stdout and stderr.
- `issue-573-history-1791124085415.jsonl`:
  completed uninstrumented comparison's stdout and stderr.
- `issue-573-host-timeout-control.ts` and `issue-573-host-timeout-control.jsonl`:
  positive controls around the September 23 change.
- `issue-573-historical-log-window.ts`:
  read-only selection of retained incident-window logs.

The detached historical worktree was removed after the trials;
recreating it is necessary before rerunning the endpoint scripts.
The scripts and evidence files remain.

The scripts currently record this checkout's absolute path.
They are local diagnostic artifacts,
not portable package tests or a completed regression guard.
The tracer captures invocation arguments and paths;
inspect and redact its output before sharing externally.

### Proposed agent instruction correction

Tighten `AGENTS.md`'s existing `CB1` rather than adding another overlapping rule:

```text
Before handing off, try bridges: shell, browser, compositor/IPC, expect/tokens, hardware CLI;
regressions: test historical endpoints, bisect with a validated signal. State attempts.
```

This is a proposal,
not an applied change to agent instructions.

## Documentation verification

`mise run lint:markdown -- doc/troubleshooting/cli-git-logger-sink-verify-timeout.md` passed.
Sätteri 0.10.5 rendered the document to HTML;
headings,
code spans,
lists,
and diagnostic blocks were inspected in the rendered output.

## Upstream filing decision

The existing repository issue #573 is the tracking location;
no separate upstream issue or duplicate is warranted.

1.  No third-party fault is established;
    the diagnostic comes from this repository's logger.
2.  Repository-owned code can be changed once the delayed operation is identified.
3.  Node command-line logging is a documented supported use case.
4.  The owner explicitly requested this investigation and fix.
5.  There is no established external maintainer decision to evaluate.
6.  No causal fix has been prototyped;
    forcing a timeout validates observation only.

The additive issue update records sink identity,
endpoint and positive-control results,
and the unresolved reproduction requirement.
The owner subsequently requested closure with "can't reproduce".
The issue was closed as not planned,
not as a verified code fix.

## Open questions

- Which process emitted the reported timeout,
 and which actual directory did its file sink select?
- Which filesystem operation or scheduling delay exhausted verification's deadline?
- Does the failure follow this machine or every checkout?
