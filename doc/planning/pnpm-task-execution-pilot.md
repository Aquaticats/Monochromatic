# pnpm task-execution pilot for the Mise task slice

Purpose:
 answer whether pnpm can replace Mise's task discovery and execution,
the option the user chose on 2026-10-28
("Run disposable pnpm pilot"),
by measuring the parity gates that the September record left open:
 graph reshaping,
declaration compilation,
paths with spaces,
missing optional scripts,
and failure tolerance under `pnpm pipeline`.
The failure-semantics gate was already measured in
[`pnpm-recursive-no-bail-semantics.md`](../troubleshooting/pnpm-recursive-no-bail-semantics.md).

Status:
 pilot complete,
recommendation conditional,
no adoption performed.

## Environment

- pnpm 12.16.1 linux-x64,
  the artifact `toolchain.lock.json` records,
  SHA-256 `565ff81174a14f6f7bd5d84c1b17e51031a80230d370a1fe24be5d16bd02a825`.
- Disposables under `~/temp/agent/`:
  `pnpm-pilot-graph-2026-10-28.ts` (repo measurement),
  `pnpm-pilot-decl-runner-2026-10-28.ts` (declaration fixture),
  `pnpm-no-bail-probe.x3IU7LBt` (failure chain),
  `pnpm pilot spaces.x1` (spaces and missing-script fixture).
- Commands ran with the extracted binary;
  the fixtures are not git repositories,
  so pipeline caching reported "without a cache key".

## Graph reshaping

`node ~/temp/agent/pnpm-pilot-graph-2026-10-28.ts` classifies every `depends`
edge in active `package/**/mise.toml` against pnpm's two `dependsOn` forms
(`name` for the same project,
`^name` for the same-named task in each workspace dependency,
per [workspace-task-orchestration, selected](https://pnpm.io/workspace-task-orchestration#:~:text=Each%20dependsOn%20entry%20has%20one%20of%20these%20forms%3A)):

- 222 packages carry `mise.toml`;
  603 `depends` edges total.
- 89 edges are same-project (5 of them use an absolute `//<own path>:<task>` label);
  these map to `name` entries exactly.
- 0 root-target edges in active packages.
- 476 edges are cross-package and land on a direct workspace dependency;
  every one of them becomes a `^name` over-approximation.
  Example:
   `translate-stage`'s edge to `adjudication:build:js:node` would also run
  `build:js:node` in 16 other dependencies.
- 33 edges reach only transitive dependencies,
  so they hold only when intermediate packages run the same-named task first.
- 5 edges are inexpressible:
   `source-scan` depends on `bench`,
  `corpus-pass`,
  `coverage-census`,
  `probe`,
  and `slice-cache` `build:js:node`,
  and none of those packages is a dependency of `source-scan`
  (they would need new `package.json` edges).

Consequences of `^name` over-approximation are semantic:
 extra tasks run,
and a failure in an unrelated dependency now blocks the dependent,
which the parity gate's failure-tolerance items forbid.

## Declaration compilation

The fixture compiles one representative inline-TS body
(template literal with `${name}`,
a single-quoted string with an escaped quote,
and a literal `literal-${x}` string)
into `package.json` scripts:

- Baseline `node --input-type=module-typescript -e <body>` prints
  `hello world quote=it's fine dolar=literal-${x}`.
- Naive double-quoted compilation **silently corrupts**:
  the shell expands backticks and `${...}`,
  prints `sh: line 1: hello: command not found`,
  and exits 0 with the intended output gone.
- Single-quoted compilation with `'"'"'` escaping reproduces the baseline.
- A body file (`node scripts/hello.ts`) reproduces the baseline trivially.
- `pnpm --filter @pilot/app run <script>` works as the partial-invocation form
  of `mise run //package/<path>:<task>`.
- `run = ["seq-a", "seq-b"]` maps to a same-project `dependsOn` chain;
  `pnpm -r run --dry-run --json seq-b` shows the edge and the run order holds.

A file-enforcer compiler is therefore viable,
but it must own shell and JSON encoding;
naive interpolation corrupts silently.

## Spaces and missing scripts

In `~/temp/agent/pnpm pilot spaces.x1` (a workspace path containing spaces):

- install,
  `pnpm -r run --dry-run --json test`,
  and `pnpm -r run test` all work,
  and task `cwd` output shows the spaced path intact.
- A dependency without the `test` script appears as `missingScript: true`
  and passes through as skipped,
  keeping the chain between the remaining projects,
  which covers "missing optional package builds".

## Pipeline failure tolerance

`pnpm pipeline --full` on the failing chain
(`lib#test` fails,
`app#test` depends on it,
`free#test` independent):

```text
# ~/temp/agent/pnpm-no-bail-probe.x3IU7LBt
Pipeline "default": 3 tasks — 1 passed (0 from cache), 1 failed, 1 skipped.
Error: ERR_PNPM_PIPELINE_FAIL
```

The dependent is skipped,
exactly like `pnpm -r --no-bail run test`.
"Keeps going after a task fails,
so one run reports every failure"
therefore means independent failures aggregate,
not that dependents run.
Both execution paths fail the gate item
"tests continuing after build failure where required".

## Parity gate verdicts

- package namespaces:
   partial;
  `--filter` and `<project>#<script>` replace `//package/<path>:<task>`.
- partial invocation:
   pass.
- paths containing spaces:
   pass.
- sequential groups:
   pass via same-project `dependsOn`.
- parallel groups:
   partial;
  `--parallel` ignores `tasks` declarations,
  `concurrency` and `concurrencyGroups` cover limits.
- missing optional package builds:
   pass.
- intentional failure tolerance:
   partial;
  independent projects continue under `--no-bail`,
  dependents of a failure never run.
- tests continuing after build failure where required:
   fail without graph reshaping or in-body tolerance.
- retained failure causes:
   partial;
  aggregate "failed in N packages" plus per-task output,
  pipeline writes a run report,
  skipped dependents carry no cause.
- machine-readable task listing:
   partial;
  `--dry-run --json` gives the graph,
  no full `mise tasks --json` catalog equivalent.
- correct aggregate exit status:
   pass.

## Pros and cons versus meow

pnpm 12.16 with repo-owned adapters:

- Pro:
   a working scheduler today,
  caching and affected selection where this repo caches nothing,
  spaces-safe execution,
  missing-script pass-through,
  graph JSON,
  and working sequencing.
- Con:
   5 inexpressible edges,
  33 fragile transitive edges,
  476 over-approximated edges with changed failure semantics,
  a compiler that must own shell and JSON escaping,
  no watch equivalent,
  different labels,
  and experimental `pnpm pipeline`.

meow (selected owner 2026-09-17,
plan-only):

- Pro:
   argv task bodies avoid the escaping class,
  arbitrary task edges,
  `//package/<path>:<task>` labels,
  watch daemon with RPC,
  freshness from declared reads.
- Con:
   no code exists,
  open questions remain in its design,
  and it must build scheduling and caching pnpm already ships.

## Ranking

Under the parity gate as written:
 meow > pnpm 12.16 with adapters > keep Mise.

- meow beats pnpm because 38 task edges need reshaping and the
  failure-tolerance gate fails at the scheduler level,
  which is exactly meow's designed surface;
  a pilot cannot remove those facts.
- pnpm with adapters beats keeping Mise because the incumbent's task
  surface carries the measured defects of
  [`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md)
  (no caching,
  root `mise run test` misses 15 packages)
  while the accepted endpoint is Mise removal.

Conditional ranking if the user accepts three semantic shifts
(new `package.json` edges for the 5 unreachable targets,
`^name` over-approximation,
and failure tolerance inside single task bodies):
 pnpm 12.16 with a file-enforcer compiler > meow > keep Mise,
because pnpm works today and adds caching.

Risks:

- `pnpm pipeline` is documented as experimental;
  its configuration and output may change.
- The over-approximation changes aggregate failure semantics in ways the
  current Node fanout does not.
- The compiler path is proven only on one representative body;
  a full audit of template bodies is still open.

Next action:
 the three semantic shifts are a flagged choice;
adoption waits for explicit acceptance
(`doc/decision/` per `DRR`).
