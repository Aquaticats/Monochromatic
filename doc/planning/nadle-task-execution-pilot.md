# Nadle task-execution pilot and comparison

Purpose:
 the user directed "Pilot and compare with <https://nadle.dev/> first"
before choosing the task-slice contract,
so this measures Nadle 0.6.1 against the same parity gates as
[`pnpm-task-execution-pilot.md`](pnpm-task-execution-pilot.md)
and compares both with the selected owner,
meow.

Status:
 pilot complete,
comparison complete,
contract choice still open.

## Environment

- Nadle 0.6.1 from npm (MIT,
  `nadlejs/nadle`),
  cloned to `~/temp/agent/nadle-src-2026-10-29`
  at `16414f00379059fa03ba6169cc40a5b252a8f312`
  (`chore: release nadle v0.6.1` commit,
  2026-09-30).
- Fixture `~/temp/agent/nadle pilot spaces.x2`
  (a workspace path containing spaces),
  root `nadle.config.ts` plus `pkgs/lib` and `pkgs/app` workspaces,
  driven by `~/temp/agent/nadle-pilot-runner-2026-10-29.ts`.
- Requires Node 22+ and a `package.json` at the root;
  workspace discovery needs `pnpm-workspace.yaml` (without it only `root` loads).

## Naming grammar is the headline gap

Registering the repo's own vocabulary fails at config load:

```text
# nadle free:test, first probe
Invalid task name: [free:test]. Task names must contain only letters, numbers, and dashes;
start with a letter, and not end with a dash.
```

Exit 2,
`ConfigurationError`,
message defined at
[`messages.ts:24-25`](https://github.com/nadlejs/nadle/blob/16414f00379059fa03ba6169cc40a5b252a8f312/packages/nadle/src/core/utilities/messages.ts#L24-L25)
and asserted by
[`invalid-task-name.test.ts`](https://github.com/nadlejs/nadle/blob/16414f00379059fa03ba6169cc40a5b252a8f312/packages/nadle/test/features/invalid-task-name.test.ts#L15-L23).
Colon-shaped names (`build:js:node`,
`lint:types`,
`test:unit`) and `//package/<path>:<task>` labels are therefore impossible;
the whole vocabulary would become dash names
(`build-js-node`),
which also removes the `:` separator's ambiguity
(`packages/kernel/src/task-identifier.ts:4-12` splits references on the last colon).
Cost:
 every task name,
the 603 measured `depends` edges,
`CM5` addressing,
README and CI call sites,
and the muscle memory
[`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md)
counts as a feature.

## What the pilot measured

Exact edges (the pnpm killer gap) work:

- `stage-lint` with `dependsOn: ["adjudication-build", "corpus-build"]`
  ran exactly those and `stage-lint`;
  the unrelated `bench-build` never ran.
- Cross-workspace edges resolve by workspace id:
  `pkgs:lib:test` and `pkgs:lib:build-js-node` ran the `pkgs/lib` bodies
  before their `pkgs/app` dependents.
- `dependsOn` rejects the CLI's slash labels and package names:
  `Workspace "pkgs/lib" not found. Available workspaces: root, pkgs:app, pkgs:lib`.
  The CLI accepts both (`pkgs/lib:build-js-node` resolved to `pkgs:lib:build-js-node`).

Bodies run in-process,
so the pnpm escaping class is absent:

- The template-literal body printed exactly
  `hello world quote=it's fine dolar=literal-${x}`,
  the same as direct `node -e`,
  while naive shell compilation of that body corrupted silently in the pnpm pilot.
- `ExecTask` with `$(id)`,
  `a;b`,
  and quote-bearing arguments passed them literally
  (`exec ok argv=$(id)`).

Failure semantics:

- Default run stops after the first failure:
  `app-test free-test` reported `1 task failed, 1 task blocked, 1 task not started`,
  so the independent `free-test` never ran.
- `--continue` keeps independent work going:
  `1 task executed, 1 task failed, 1 task blocked`,
  exit 1,
  and `free-test` completed.
- A dependent of a failure is **blocked in both modes**,
  the same shape as pnpm `--no-bail`.
- `--reporter agent` emits a machine-readable cause:
  `{"errorType":"TaskExecutionError","message":"Task lib-test failed: lib test failed","errorCode":1,"task":"lib-test"}`.

Missing optional tasks fail hard:

- `ghost-test` with `dependsOn: ["nope-build"]` exits 3
  (`TaskNotFoundError`),
  unlike pnpm's pass-through skip.
  `onlyIf` covers skips
  (a falsey `onlyIf` skips the body and its dependents still run,
  verified with `skip-dependent`),
  so "missing optional build" needs generated empty tasks or `onlyIf` guards.

Surface that pnpm lacks:

- `--list --json`,
  `--dry-run --json` (ordered plan),
  `--graph --json` (nodes and dependency edges),
  `--explain`,
  `--capabilities` (flags plus task-config JSON Schema),
  `--why` (cache outcome),
  `--doctor`,
  `--since <ref>` affected selection,
  `--watch` (re-run on input change),
  `--continue`,
  `--exclude`,
  `--min-workers`/`--max-workers`,
  per-task `timeout`,
  `retries`,
  `env`,
  `workingDir`,
  and caching via `inputs`/`outputs`/`cacheVerdict`
  (the run reports "not cacheable;
  declare inputs & outputs" when absent).
- Exit codes are distinct:
  1 execution,
  2 config,
  3 not found.

Everything ran under the spaced fixture path,
so "paths containing spaces" passes.

One CLI hazard:
 ambiguous names autocorrect across workspaces
(`app-test2` silently resolved to `root:app-test`,
with the mapping printed).
CI should pass exact ids.

## The September documentation gate still stands

The features page shows `CopyTask` with `to`,
while the guide,
the types,
and the runtime require `into`:

```text
# nadle copy-to (options: { from: "assets", to: "dist-to" })
{"errorType":"TaskExecutionError","message":"Task copy-to failed: The 'into' option is required.","errorCode":1,"task":"copy-to"}
```

The guide's wording is current
([destination sentence, selected](https://nadle.dev/docs/guides/file-operation-tasks/#:~:text=The%20destination%20(into)%20is%20always%20a%20directory));
the features example still uses `to`
([that example, selected](https://nadle.dev/docs/getting-started/features/#:~:text=to%3A%20%22dist%2Fassets%22)).
This is the same documentation confusion that culled Nadle in
[`tech-monorepo-manager-vet-2026-09-16.md`](../audit/tech-monorepo-manager-vet-2026-09-16.md).

## Comparison

Nadle versus pnpm 12.16 with adapters:

- Exact task edges:
   Nadle yes,
  pnpm needs `^name` over-approximation on 476 edges
  plus package-graph reshaping for 38.
- Task bodies:
   Nadle runs typed TS in-process,
  pnpm needs a compiler that owns JSON and shell escaping
  (naive compilation corrupts silently).
- Names:
   Nadle forbids the colon vocabulary outright,
  pnpm keeps names as script keys
  but changes the invocation labels anyway.
- Failure aggregation:
   both continue independent work on request
  (`--continue` versus `--no-bail`)
  and block dependents of failures;
  Nadle reports richer causes.
- Missing optional tasks:
   pnpm passes through,
  Nadle errors unless modeled.
- Caching:
   Nadle ships it built in with `--why`,
  pnpm offers it in the experimental `pnpm pipeline`.
- Maintenance:
   pnpm is a large ecosystem;
  Nadle is 0.6.x with one maintainer,
  36 stars,
  and about 107 weekly downloads,
  though actively developed through 2026-10.

Nadle versus meow (selected 2026-09-17,
plan-only):

- Nadle covers the task slice today with caching,
  machine-readable output,
  `--watch`,
  and `--since`;
  meow must still build all of it.
- meow's design keeps `//package/<path>:<task>` labels,
  argv task bodies with no shell,
  a watch daemon with RPC control,
  and freshness from declared reads;
  Nadle changes the labels and its watch has no control channel.

## Ranking for the task slice

1.  Nadle 0.6.1 with a file-enforcer compiler and a task rename.
2.  pnpm 12.16 with adapters and the three recorded shifts.
3.  meow as designed.
4.  Keep Mise.

- Nadle beats pnpm because exact task edges and typed in-process bodies
  remove two of pnpm's three shifts,
  and its caching and machine-readable surface exceed pnpm's;
  the price is the global rename and the missing-optional modeling.
- pnpm beats meow because it works today,
  while meow is plan-only with open design questions
  and would still need scheduling and caching built.
- meow beats keeping Mise because the accepted endpoint is Mise removal
  and the incumbent carries the measured defects of
  [`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md).

Risks:

- Nadle's one-maintainer,
  low-adoption profile is a continuity risk that pnpm does not carry.
- The features-page `to` example keeps the documentation gate open.
- The rename touches every task name and call site;
  file-enforcer could compile the labels,
  but the migration cost is unmeasured.

Next action:
 the contract is a flagged choice;
adoption waits for explicit acceptance and would reopen the vet
for the chosen candidate.
