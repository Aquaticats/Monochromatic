# module-logger verification campaign

## Status

Campaign in progress.
The Node orchestration layer,
 the Node sink boundary layer,
 the coverage-reachability gate,
 and the CI workflow landed on 2026-09-06 in the sidecar package `package/logger/core.fuzz`
 and `.github/workflows/logger-fuzz.yml`.
The toml-edit campaign moved into `package/toml-edit/core.fuzz` on 2026-09-06 (`doc/decision/toml-edit-fuzzing.md`,
 "Sidecar package").
The Playwright browser layer landed the same day,
 so every deliverable of the campaign is on main.
Plan,
 grill records,
 and landed commits:
 `package/logger/core/bulletproofing.plan.md`.
Per-contract design rationale:
 `package/logger/core/DECISIONS.md`.

This document is the decision record for how `@monochromatic-dev/logger-core` is verified,
rewritten on 2026-09-06 from the June design-only record.
The June contracts that the shipped code reversed are listed under "Superseded decisions" so the history stays readable.

## Context

The goal is the verification bar `package/toml-edit/core` already meets:
a budgeted property campaign,
 a reference-model oracle,
 a committed coverage-reachability gate,
a path-filtered CI workflow,
 and a decision record.
The logger differs from a parser in one structural way that drives the whole design:
its bugs are timing and interleaving bugs,
 not input bugs.
So the generator is a scheduler over sink hook settlements,
 not a grammar,
and every oracle is a reference model of the orchestration contract.

Two facts measured before the campaign shaped it:
the logger is imported at 102 sites in this repository and no site guards a log call,
and no colour or escape sequence flows through the logger today.

## Decision: sidecar package

The campaign lives in `package/logger/core.fuzz`,
 a private workspace package,
the convention `jsonc-edit.fuzz` and `css-edit.fuzz` set.
The runtime package's `src` stays pure production code and ships in the tarball;
`fast-check`,
 the scripted fake sinks,
 the reference model,
 the properties,
the run-budget tooling,
 and the coverage gate never publish.
Every property file imports the built runtime artifact through the package name,
never the runtime package's source,
 so the suite crosses the consumer boundary.
The sidecar's `fuzz` task rebuilds the runtime package first for that reason.

The toml-edit campaign,
 which still lives inside its runtime package under `src/fuzz/`,
migrates into `package/toml-edit/core.fuzz` after the logger campaign,
 copying this layout.

## Decision: scheduler, not grammar

Interleavings come from fast-check's `scheduler()`.
Every fake-sink hook settlement (verify,
 write,
 flush) is a scheduled task,
so the scheduler decides the order in which outcomes reach the logger,
and a shrunk counterexample names the exact release order.
Never-settling work is a promise the scheduler never releases.

The logger's own timers stay real.
Deadline properties therefore use short real deadlines and bounded run counts
(`src/harness.ts`:
 verify timeout 25 ms,
 flush deadline 300 ms,
 a 60 ms tolerance below the deadline for the "settled within" verdict).
Faking timers was rejected:
 `withTimeout` and the sinks share the same clock,
and a faked clock would prove nothing about the real deadline path.

## Decision: scripted fake sinks

A fake sink (`src/fake-sink.ts`) is driven by one script per hook.
Each script is a per-call outcome sequence with a repeating tail:
verify outcomes are resolve-true,
 resolve-false,
 reject,
 throw synchronously,
 or never;
write and flush outcomes are resolve,
 reject,
 throw synchronously,
 or never;
flush may be absent.
Every sink carries a stable index,
 so a shrunk counterexample reads as
`sink 2: verify [resolve-true*] write [reject, resolve*] flush absent`.
Every scripted rejection gets its own rejection handler at creation,
because the scheduler holds rejected promises until released and an unhandled rejection would otherwise kill the
test-file process.
A trace records every hook call and settlement with its call index,
 so the model can fold the observed order.

## Decision: reference model scope

`src/model.ts` predicts,
 per sink,
 the exact records attempted and delivered,
 in order,
 and the final availability;
whether each `flush()` settled within its deadline;
the dropped-count marker record after startup overflow;
and the count of `console.warn` breadcrumbs.
The "loud signal once,
 at a boundary" contract is therefore a checked invariant,
 not a comment.
Properties that stub `console.warn` retain the sidecar's sequential setting.
This originally matched the runtime package's breadcrumb suites.
Issue #481 gives the runtime suites context-owned method replacements;
those suites now run concurrently without changing this campaign's scheduling decision.

## Decision: two run layers

The same `*.property.unit.test.ts` files run bounded in `test:unit`
(60 runs per property under a 60 s harness timeout)
and as a time-budgeted campaign through the sidecar `fuzz --budget <ms>` task,
keyed on `LOGGER_FUZZ_BUDGET_MS` (`src/fuzz-budget.ts`).
Campaign mode uses `interruptAfterTimeLimit` with an unbounded run count and a harness timeout of budget plus 30 s,
so fast-check owns the stop and still has room to shrink.

`interruptAfterTimeLimit` abandons the in-flight run rather than awaiting it.
An abandoned run's logger keeps firing timers into the next property's `console.warn` stub,
which surfaced as one extra verify-timeout breadcrumb that appeared only in campaign mode.
The harness tracks every in-flight run and `settleRuns()` is awaited after each `assert`,
so no run outlives its property.

## Landed: scheduled orchestration property

`src/orchestration.property.unit.test.ts` runs a random program of `log`,
 `release`,
 and `flush` steps
(weighted three to two to one,
 at most eight steps)
against a logger over one to four scripted sinks.
Two properties:

- The real sinks and the model agree on attempts,
   deliveries,
   flush verdicts,
   breadcrumb count,
   and whether a log call threw
  (exactly-once delivery,
   startup replay,
   dropout on failed verify,
   write resilience,
   flush totality,
   and the no-backend throw,
   in one oracle).
- Over always-available sinks,
   every record reaches every sink exactly once regardless of release order.

Guard-failure proof on 2026-09-06:
with the startup replay removed from `create-logger.ts` and the artifact rebuilt,
both properties fail and shrink to one sink and one log call;
restored and rebuilt,
 both pass.

Model defects the property found before the logger was ever wrong:

- A write that never settles keeps `flush()` in its write drain until the deadline,
   so no flush hook runs;
  the model had run the hooks anyway.
- A rejecting flush hook retires the sink (the shipped contract),
   so the always-available property restricts flush outcomes to resolve and never.

No logger defect has been found by this layer yet.
That is consistent with the three robustness changes having been built test-first the week before;
the campaign's value so far is that the contract is now executable.

## Landed: sink boundary properties under Node

`src/sink-boundary.property.unit.test.ts` feeds adversarial records through each Node-reachable sink built from the
artifact and inspects what came out.
The corpus (`src/boundary-corpus.ts`) holds JSON delimiters and record-forging text,
 terminal escape sequences (well-formed,
 unterminated,
 nested,
 and 8-bit C1),
 lone and paired surrogates,
 and astral text;
the arbitrary (`src/adversarial-message.ts`) interleaves those tokens with binary text
and with control characters drawn uniformly across C0,
 DEL,
 and C1,
because the default binary string arbitrary reaches DEL and C1 about once per million code units.
Four properties:

- The console neutralizer agrees with an independent code-unit-indexed reference
and leaves no forbidden control in its output.
- The console sink emits exactly the reference prediction of its grouped output:
  one text per contiguous same-level run,
   every message neutralized,
   debug runs on process stderr with the trailing newline the sink adds.
- The sessionStorage sink's persisted batches reparse to the exact records.
- The file sink's appended lines,
   under a throwaway package directory per run,
   reparse to the exact records after its verify probe.

Guard-failure proof on 2026-09-06:
with the neutralizer made identity and the file sink interpolating the message raw into its JSON line,
 and the artifact rebuilt,
the neutralizer,
 console sink,
 and file sink properties fail on their first run;
restored and rebuilt,
 all four pass.

Finding,
 in the campaign rather than the logger:
every boundary body settles through microtasks alone,
 so a campaign held the event loop for its whole budget;
the test harness's own logger,
 still verifying its file sink when the first property started,
saw its verify timer fire before the filesystem answered and wrote a breadcrumb.
Each run now yields once through `setImmediate` (`yieldToEventLoop` in `src/sink-boundary-harness.ts`).

## Landed: coverage-reachability gate

The campaign measures its own reach with the deterministic V8 line-coverage gate toml-edit established
(`doc/decision/toml-edit-fuzzing.md`,
 "Coverage gate"),
 copied into the sidecar as
`src/coverage-v8.ts`,
 `src/coverage-aggregate.ts`,
 and `src/coverage-report.ts`
and pointed at the runtime package's `src` through the `/ts` subpath it resolves.
The driver (`src/coverage-driver.ts` and the `src/coverage-*.ts` scenario modules) imports the runtime source the same
way
and replays fixed inputs through every orchestration branch (scripted fake sinks with the identity gate)
and every Node-reachable sink branch:
the console sink under silent,
 verbose,
 and browser-window modes and under hosts whose `env`,
 `argv`,
 or `stderr` throw;
the file sink with a present,
 absent,
 blocked,
 and vanished log directory;
the sessionStorage and localStorage sinks over Node's real backends and over quota-limited,
 flaky,
 and refusing stand-ins;
the record buffer's every trigger including page-lifecycle events fired through a page stand-in;
the key,
 quota,
 and quota-error helpers under staged Deno and Bun markers.
The `fuzz:coverage` task runs the driver under `--localstorage-file`
so the localStorage sink elects instead of taking the flagless short-circuit;
`--write` refreezes `coverage-baseline.json`.

First baseline,
 2026-09-06:
 3376 of 3623 code lines across the 27 runtime source files.
Every Node-reachable file sits between 97 and 100 percent;
the remaining lines there are unreachable under Node by construction
(the console verify guard for a missing `console`,
 the browser and unknown runtime branches,
 a timer without `unref`)
or internal invariants (a missing sink entry,
 a second `initialize` call).
The browser-only IndexedDB,
 OPFS,
 and IndexedDB utility files sit at 47 to 69 percent and wait for the Playwright layer.
The driver's adequacy was checked against the uncovered-line list,
 not assumed:
each scenario added after the first freeze targeted a named uncovered line,
and one apparent gap (the store's oldest-first sort) turned out to need two prior-run entries before the comparator
runs at all.

## Landed: CI workflow

`.github/workflows/logger-fuzz.yml` mirrors `toml-edit-fuzz.yml`:
path-filtered to `package/logger/core/**`,
 `package/logger/core.fuzz/**`,
 this document,
 and itself,
with the merge-queue scope step,
 `MISE_AUTO_INSTALL` off,
 and only node and pnpm installed.
Its steps build the runtime package,
 type-check the sidecar,
 run the sidecar unit suite (bounded property runs plus the fake-sink and model tests),
run the fuzz smoke at 3000 ms per property,
 and run the coverage gate.
Every step was run locally in that order before the workflow was committed;
the smoke takes about 25 seconds and the gate about 20 seconds on the development machine.

## Landed: Playwright browser property layer

The browser-only backends are checked in real browsers rather than through stand-ins.
`src/browser/properties.ts` is bundled by the sidecar's `bundle:browser-properties` task
(rolldown's client flavor under the browser platform,
 so `@monochromatic-dev/logger-core` resolves to the neutral artifact and fast-check is inlined)
and served to the Playwright harness page from `/dist/module-logger.fuzz/client/properties.js`.
The bundle installs a runner on the page;
 `src/browser/properties.browser.test.ts` loads it,
 runs every property,
 prints each browser's outcomes,
 and fails on any falsified property.
Three properties over the adversarial record arbitrary:

- The localStorage sink's run-scoped batches,
   read back through the key helpers the artifact exports,
   reparse to the exact records.
- The IndexedDB sink's batches in the `monochromatic.log` database reparse to the exact records;
  each run prefixes its messages with a nonce because the store outlives the run.
- The OPFS sink accepts every record and its flush settles with no breadcrumb.
  This oracle is weaker by construction:
  the sink keeps its writable open for the session and the platform commits a file's bytes only on close,
  so the page cannot read the content back.

Measured on 2026-09-06 inside the `monochromatic-playwright` podman image:
Chromium 153 and Firefox 155 run all three properties;
WebKit 26.6 runs the two storage properties and reports OPFS skipped,
because headless WebKit refuses the origin-private file system with `UnknownError`
(the probe turns that refusal into the skip reason rather than letting it escape the run).
Guard-failure proof:
with the shared record buffer dropping the first record of every batch and both the artifact and the bundle rebuilt,
the localStorage and IndexedDB properties fail on their first run and shrink to one debug record;
restored and rebuilt,
 all three pass.

The task is named `bundle:browser-properties`,
 not `build`:
the sidecar ships nothing,
 and the `require-eventual-artifact` lint treats a package with a build task as one whose tests must import built
 output,
which would reject every relative helper import in the sidecar's test files.
No workflow runs the browser suite today;
the layer is a local podman run
(`mise run test:browser -- package/logger/core.fuzz/src/browser/properties.browser.test.ts`) until a browser CI job
exists.

## Superseded decisions

The June record fixed several contracts at design time;
 the shipped code reversed them.
Each is recorded in `package/logger/core/DECISIONS.md`.

- Failure stance:
   log calls never throw.
  Shipped:
   a log call throws `No logging backends available` once initialization has proven that no sink is available.
  The startup-buffer section of `DECISIONS.md` records why a marker with nowhere to go has no consumer;
  the no-backend throw is a property the campaign checks,
   not a bug it hunts.
- Flush deadline default 1000 ms and verify timeout default 1000 ms.
  Shipped:
   `DEFAULT_FLUSH_DEADLINE_MS` and `DEFAULT_VERIFY_TIMEOUT_MS` are 5000,
   each with a measured local settle time of a few milliseconds.
- Retire threshold of ten consecutive sink failures.
  Not shipped:
   a write rejection stays transient with one breadcrumb and a rejecting flush hook retires the sink,
  per "Write failures do not disable a sink;
   only verify failure does".
- Console boundary preserving well-formed SGR colour sequences.
  Shipped:
   every C0 control except newline and tab,
   DEL,
   and every C1 control is rendered as `\uXXXX`;
  the allowlist classifier was rejected because no in-repo call site passes colour through the logger.
- Startup buffer cap as a `createLogger` option.
  Shipped:
   `STARTUP_BUFFER_CAP` is an exported constant (10000),
   dropping oldest with a marker record.
- `@monochromatic-dev/async-time-core` promoted to a runtime dependency.
  Shipped:
   it stays a devDependency imported through its `/ts` subpath and inlined by the build;
  the published `dependencies` map is empty.

## Pending deliverables

None.
 The browser layer runs locally only until a browser CI job exists;
 see "Deferred follow-up" for the rest.

## Rejected alternatives

- A differential oracle comparing the logger against `pino` or `winston`:
  their formats differ by design,
   so a disagreement would be noise.
- A hostile `toString` message family:
   the log API is string-only,
   so no object reaches a sink.
- Fake timers:
   see "Decision:
   scheduler,
   not grammar".

## Reusable fuzz-target checklist

For the logger and any future async target,
 all six must hold before a campaign is called strong:

- Is the tested layer where the logic and bugs live?
- Are all entry points,
   sinks,
   and seams covered?
- Is every oracle a reference model,
   not no-crash or returns-a-string?
- Do generators cover the full operation and message grammar and boundary cases?
- Are real corpus seeds,
   counterexamples,
   and coverage feedback wired in?
- Does the schedule generator explore the async interleavings where timing bugs live,
  including never-settling work?

## Deferred follow-up

Mutation testing,
 to measure oracle strength more directly,
 is deferred to a separate plan,
the same follow-up toml-edit recorded.
File or update an issue for it when the logger campaign lands.
