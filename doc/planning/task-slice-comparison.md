# Task-slice candidate comparison

Purpose:
 join every measured task-execution pilot the user directed
 ("Pilot and compare with Pants first",
 "Pilot and compare Poe the Poet,
 projen,
 Buck2 too",
 "Pilot and compare Please too",
 "Pilot and compare Tilt too")
 with the recorded pnpm and Nadle pilots
 and the selected owner,
 meow,
 so the task-slice contract question can be asked with every candidate measured.

Status:
 Poe the Poet 0.49.0,
 Buck2 2026-10-14,
 Pants 2.33.1,
 and projen 0.104.19 pilots complete 2026-10-29;
 Please and Tilt pilots in flight;
 the ranking below is provisional until they return;
 contract choice still open,
 no adoption performed.

## Candidates and evidence

Each pilot ran against the parity-gate catalog of
[`mise-removal-coverage.md`](mise-removal-coverage.md)
in a disposable fixture whose path contains spaces,
under `~/temp/agent/`.

- pnpm 12.16.1:
  [`pnpm-task-execution-pilot.md`](pnpm-task-execution-pilot.md)
  and [`pnpm-recursive-no-bail-semantics.md`](../troubleshooting/pnpm-recursive-no-bail-semantics.md).
- Nadle 0.6.1:
  [`nadle-task-execution-pilot.md`](nadle-task-execution-pilot.md).
- Poe the Poet 0.49.0 (`poethepoet`,
  MIT):
  report `~/temp/agent/poe-pilot-report.md`,
  fixture `~/temp/agent/poe pilot spaces.x4`,
  clone `~/temp/agent/poethepoet-2026-10-29` at `b8e96a9c244b6dd1d8b8b46f36957f4420cfa7fd`.
- Buck2 2026-10-14 (`meta/buck2`,
  MIT OR Apache-2.0):
  report `~/temp/agent/buck2 pilot spaces.x6/REPORT.md`,
  fixture `~/temp/agent/buck2 pilot spaces.x6`,
  release binary SHA-256 `89452c62c5222d4231bbb76429ec3bc1e0a7c46252d765be12a00a2be703d5ab`.
- Pants 2.33.1 (`pantsbuild/pants`,
  Apache-2.0):
  report `~/temp/agent/pants-pilot-logs/REPORT-pants-pilot.md`,
  fixture `~/temp/agent/pants pilot spaces.x3`.
- projen 0.104.19 (`projen/projen`,
  Apache-2.0):
  report `~/temp/agent/projen-pilot-evidence/`,
  fixture `~/temp/agent/projen pilot spaces.x5`,
  clone `~/temp/agent/projen-src-2026-10-29` at `c8b3fee`.
- Please and Tilt:
  pilots in flight,
  sections land when their reports return.
- meow (selected 2026-09-17,
  plan-only):
  [`monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md).
- Keep Mise:
  the incumbent,
  with its measured defects in
  [`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md).

## Naming grammar

The repo's vocabulary is colon-shaped
(`build:js:node`,
`lint:types`)
and its labels are `//package/<path>:<task>`.
Only Poe the Poet and projen accept the names verbatim.

- Poe the Poet:
  `poe build:js:node` runs;
  a dotted name is rejected,
  and one invalid name poisons every run in that project:

```text
# ~/temp/agent/evidence/probe-b.txt
Error: Invalid task 'build.js.node'
     | Task names must start with a letter or underscore and contain only alphanumeric characters, colon, underscore, dash, or plus.
```

- projen:
  `npx projen build:js:node` and `npx projen build.js.node` both run;
  `addTask` rejects only empty names.
- Buck2:
  dots and dashes are legal target names,
  colons are not
  (the colon is the label separator):

```text
# buck2 targets //pkgs/colon:
error: Invalid target name `build:js:node`. Target names are non-empty strings and can only contain alpha numeric characters, and symbols `,`, `.`, `=`, `-`, `/`, `~`, `@`, `!`, `+`, `$`, and `_`. No other characters are allowed.
```

- Pants:
  dots are legal in address names,
  colons are banned in both the name and the address spec:

```text
# ./pants list "pkgs/lib:build:js:node"
[ERROR] Failed to parse address spec `pkgs/lib:build:js:node`: error at 1:16: expected EOF
```

- Nadle:
  rejects colons at config load,
  as recorded in
  [`nadle-task-execution-pilot.md`](nadle-task-execution-pilot.md).
- pnpm:
  script keys keep the names,
  but invocation labels change to `--filter` and `<project>#<script>`.

Consequences:
 Nadle,
 Buck2,
 and Pants each need a name-mangling compiler over 1,811 task names,
 603 `depends` edges,
 `CM5` addressing,
 and every call site.
Buck2 and Pants preserve more of the shape than Nadle
(`build.js.node` keeps its separators;
Nadle allows only letters,
numbers,
and dashes).
Poe the Poet and projen need no rename,
but address tasks as a flat namespace,
so `//package/<path>:<task>` still needs a mapping layer.

## Parity gate results

### Package namespaces and labels

- pnpm:
   partial;
  `--filter` and `<project>#<script>` replace the labels.
- Nadle:
   partial;
  workspace ids (`pkgs:lib:test`) replace the labels.
- Poe the Poet:
   partial;
  colon names survive,
  `/` is rejected,
  packages are `include` files in one flat namespace.
- Buck2:
   pass with naming migration;
  labels keep `//pkg:name` shape.
- Pants:
   pass with naming migration;
  address paths keep the shape,
  `tags` plus `--filter-tag-regex` cover grouping.
- projen:
   fail;
  identity is project directory plus flat name.

### Partial invocation

All six measured candidates pass:
 one package,
 one task,
 one invocation.

### Paths containing spaces

All six pass;
every pilot ran its whole probe catalog from a spaced fixture root.
Poe the Poet word-splits unquoted `${VAR}` expansions
(demonstrated),
so its compiler must quote.

### Sequential and parallel groups

- pnpm:
   sequential pass via same-project `dependsOn`;
  parallel partial.
- Nadle:
   pass via `dependsOn` chains.
- Poe the Poet:
   pass with caveats;
  `sequence` and `parallel` exist,
  parallel aborts on first failure with racy sibling output.
- Buck2:
   partial;
  no first-class groups,
  sequential is one command body or a dep chain,
  parallel is independent targets in one invocation.
- Pants:
   partial;
  parallel is the engine default,
  sequential is dep edges or one multi-goal invocation.
- projen:
   sequential pass,
  parallel fail
  (`Async commands are not supported. Run a command concurrently in the JS code instead.`).

### Missing optional package tasks

This gate separates the field most sharply.

- pnpm:
   pass;
  a missing script becomes `missingScript: true` and passes through.
- Nadle:
   fail;
  `TaskNotFoundError` exit 3 unless `onlyIf` guards or generated empty tasks model absence.
- Poe the Poet:
   fail hardest;
  one missing reference breaks every run in the project,
  even unrelated ones:

```text
# ~/temp/agent/evidence/probe-e.txt
Error: Invalid task 'needs:missing'
     | 'deps' option includes reference to unknown task: 'ghost:build'
```

- Buck2:
   partial;
  `--skip-missing-targets` rescues command-line gaps
  (`Skipped 1 missing targets: root//pkgs/bench:test`),
  but a dep edge to a missing address is always a hard error.
- Pants:
   partial;
  per-package sweeps pass
  (`./pants test pkgs/free/:: pkgs/bench/::` exits 0),
  while a declared missing address is a `ResolveError`
  that also poisons `::` queries.
- projen:
   fail;
  a missing name hard-errors,
  while stock empty stub tasks pass green with no work,
  so absence is either a break or a false green.

### Intentional failure tolerance

- pnpm:
   partial;
  `--no-bail` continues independent projects,
  dependents of a failure never run.
- Nadle:
   partial;
  `--continue` continues independent work,
  dependents of a failure are blocked.
- Poe the Poet:
   fail;
  `sequence` aborts hide independent siblings,
  `ignore_fail = "return_non_zero"` recovers them
  at the cost of double-running shared deps
  and labeling never-run tasks as failed.
- Buck2:
   pass;
  independent targets run in the same invocation,
  the cause is retained,
  the aggregate exit is 3.
- Pants:
   pass for independent work;
  `lib:test` and `free:test` both report,
  the dependent is blocked and,
  unlike Buck2,
  silently omitted from the summary.
- projen:
   fail;
  the first failure aborts everything after it.

### Tests continuing after build failure where required

The acceptance boundary in
`package/dev-script/task-util/src/build-and-test.unit.test.ts`
requires that a missing or failing package build
still runs that package's tests.

- pnpm:
   fail without graph reshaping or in-body tolerance.
- Nadle:
   fail at scheduler level;
  a dependent of a failure is blocked.
- Poe the Poet:
   fail;
  the graph aborts before the dependent test.
- Buck2:
   partial;
  independent tests run,
  a test depending on a failed build never runs.
- Pants:
   pass for independent tests,
  same dependent-blocked shape.
- projen:
   fail;
  sequential abort is the only semantics.

### Retained failure causes

- pnpm:
   partial;
  per-task output plus pipeline run report,
  skipped dependents carry no cause.
- Nadle:
   pass;
  `--reporter agent` emits a typed cause.
- Poe the Poet:
   partial;
  aggregates name failing subtasks,
  skipped dependents are misreported as failures.
- Buck2:
   pass;
  exact command line,
  stdout and stderr,
  JSON `action_error` with digest and error tags.
- Pants:
   pass;
  `ProcessExecutionFailure` with stdout,
  plus `--keep-sandboxes=on_failure`.
- projen:
   pass with caveat;
  both layers name command and `cwd`,
  but only the first failure is reported.

### Machine-readable task listing

- pnpm:
   partial;
  `--dry-run --json` gives the graph only.
- Nadle:
   pass;
  `--list`,
  `--dry-run`,
  `--graph` with `--json`.
- Poe the Poet:
   fail;
  `_list_tasks` prints names only,
  no JSON and no edge dump.
- Buck2:
   pass;
  `targets --json`,
  `uquery` and `cquery` JSON,
  `--build-report=-`,
  BXL.
- Pants:
   pass;
  `peek` JSON with `execution_dependencies`,
  `help-all` JSON.
- projen:
   partial;
  `.projen/tasks.json` is a stable manifest
  but the runner has no JSON output mode.

### Correct aggregate exit status

All six return nonzero on failure;
Buck2 returns 3,
Pants returns 1 or 241 by failure class,
projen collapses every child code to 1.

## Task bodies and escaping

Typed argv bodies are safe wherever they exist;
every shell-string form needs a quoting compiler,
and naive compilation corrupts silently in pnpm and Buck2.

- Nadle:
   typed TS in-process,
  no escaping class.
- Buck2:
   `genrule` `bash` strings cross three layers
  (Starlark literal,
  Buck2 macro expansion of `$(...)`,
  shell quoting);
  the naive form built green with an empty output file.
  The typed fix is a custom rule with `ctx.actions.run([argv...])`,
  unproven end to end.
- Pants:
   `adhoc_tool` typed argv is byte-exact;
  `shell_command` needs BUILD-string plus bash single-quote encoding.
- projen:
   `execArgs` argv is byte-exact;
  `exec` shell strings are dialect-specific
  (the default dax shell rejects POSIX `'\''` encoding,
  a double-quote dialect works).
- Poe the Poet:
   `cmd` has no shell;
  POSIX single-quote encoding is byte-exact,
  naive pastes fail loudly on unmatched quotes
  but silently expand `${...}` when double-quoted.
- pnpm:
   script strings need shell and JSON encoding;
  naive double-quoted compilation exits 0 with the body gone.

Measured body:
 `hello world quote=it's fine dolar=literal-${x}`,
 reproduced byte-exact by Poe the Poet `cmd`,
 Buck2 `escape-quoted`,
 Pants `adhoc_tool`,
 and projen `execArgs`.

## Caching, watch, and affected selection

- Nadle:
   pass;
  `inputs`/`outputs` caching with `--why`,
  `--watch`,
  `--since`.
- Pants:
   pass;
  measured cache hits
  (`(cached locally)`,
  `local_cache_requests_cached: 1`),
  `--loop` watch,
  `--changed-since`.
- pnpm:
   partial;
  experimental `pnpm pipeline` caching,
  no watch,
  `[since]` filters.
- Buck2:
   weak;
  daemon-incremental only,
  `Cache hits: 0%` across every daemon restart probed,
  no watch command
  (`subscribe` is an RPC channel),
  affected selection composes from `owner()` and `rdeps()`.
- Poe the Poet:
   absent;
  `uses` re-executes within a run,
  no watch,
  no affected selection.
- projen:
   absent;
  `--watch` re-synthesizes `.projenrc` only.

## Maintenance and documentation drift

Maintenance:
 Pants is org-maintained
 (18 maintainers,
 stable 2.33.1),
 Buck2 is Meta-backed
 (4,503 stars,
 bi-monthly pre-releases,
 no stable tag),
 pnpm is a large ecosystem,
 projen is org-owned with about two main humans behind heavy automation,
 Poe the Poet is single-maintainer
 (`nat-n`,
 2,093 stars,
 active releases),
 Nadle is single-maintainer
 (36 stars,
 about 107 weekly downloads).

Documentation drift,
the class that culled Nadle in
[`tech-monorepo-manager-vet-2026-09-16.md`](../audit/tech-monorepo-manager-vet-2026-09-16.md),
reproduces in every measured candidate:

- Nadle:
   features page `to` versus runtime `into`.
- Poe the Poet:
   the include guide
  ([cwd sentence, selected](https://poethepoet.natn.io/guides/include_guide.html#:~:text=the%20task%20will%20actually%20execute%20as%20if%20it%20had%20been%20run%20from%20the%20my_subproject%20subdirectory))
  claims included tasks run in the include `cwd`,
  which is lost for dependency nodes
  (reproduced),
  `_describe_tasks` docstring promises `name:help`,
  open [PR #438](https://github.com/nat-n/poethepoet/pull/438) admits further inaccuracies.
- Buck2:
   the README
  ([sandbox footnote, selected](https://github.com/meta/buck2#:~:text=Buck2%20currently%20does%20not%20sandbox))
  is contradicted by probes
  (a local action read `/etc/os-release`
  and `curl` returned `http_code=200`),
  genrule docs
  ([bash sentence, selected](https://github.com/facebook/buck2/blob/9ac63c5839fc5a874e766947e276a5fd0292789e/prelude/decls/genrule_common.bzl#L100))
  say `bash -c`,
  the observed invocation is `bash -e <script>`,
  `--keep-going` means different things on `build` versus `targets`.
- Pants:
   `pants help shell_command` advertises a goal that does not exist,
  the shell docs
  ([outputs sentence, selected](https://www.pantsbuild.org/stable/docs/shell/run-shell-commands#:~:text=list%20them%20in%20the%20outputs%20field))
  name a nonexistent `outputs` field,
  PyPI `pantsbuild.pants` is frozen at 2.17.1.
- projen:
   the tasks doc
  ([POSIX sentence, selected](https://github.com/projen/projen/blob/c8b3fee593c07e38a258ccffd2e57c8eba505b8c/docs/concepts/tasks.md?plain=1#L241))
  claims POSIX-style syntax the default shell rejects,
  generated markers say `.projenrc.js` where `.projenrc.ts` is generated.

Finding:
 the drift class cannot cull any candidate now
 without also culling the rest,
 so it is recorded per candidate here
 and the contract question must resolve how it is weighed.

## Ranking for the task slice

Provisional until the Please and Tilt reports return.

1.  Pants 2.33.1 with a name-mangling compiler and generated BUILD files.
2.  Nadle 0.6.1 with a rename and missing-optional modeling.
3.  Buck2 2026-10-14 with name flattening and a target-generation layer.
4.  pnpm 12.16 with adapters and the three recorded shifts.
5.  Poe the Poet 0.49.0 with a quoting compiler and stub emission.
6.  projen 0.104.19 with hand-written aggregation tasks.
7.  meow as designed.
8.  Keep Mise.

- Pants beats Nadle because both need the same name migration,
  while Pants passes missing-optional sweeps
  and intentional failure tolerance with retained causes
  and ships measured caching and watch;
  Nadle hard-errors on missing tasks
  and carries the open documentation gate.
- Nadle beats Buck2 because both need a name migration
  and a generation or compiler layer,
  while Nadle owns watch and input/output caching,
  which Buck2 lacks outright
  (no watch command;
  zero cache hits across daemon restarts),
  and Nadle's missing-optional gap is modelable with `onlyIf`
  while Buck2's cache gap is architectural.
- Buck2 beats pnpm because edges are exact by construction,
  against 476 over-approximated `^name` edges that change failure semantics,
  and Buck2 runs independent work after a failure with retained causes
  while pnpm dependents of a failure never run.
- pnpm beats Poe the Poet because a missing optional script passes through,
  while one missing Poe task reference poisons every run in the project,
  and pnpm ships graph JSON where Poe has no machine-readable listing.
- Poe the Poet beats projen because its dependency graph dedups
  and it offers parallel and `ignore_fail` modes,
  while projen has no parallel step type
  and its typed cross-project spawn silently resolves to the wrong task.
- projen beats meow because it runs today,
  while meow is plan-only with open design questions.
- meow beats keeping Mise because the accepted endpoint is Mise removal
  and the incumbent carries the measured defects of
  [`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md).

## Risks

- Name migration is structural for Nadle,
  Buck2,
  and Pants:
   1,811 names,
  603 edges,
  and the `//package/<path>:<task>` call sites.
- Buck2 and Pants are build systems,
  not task runners;
  both need a target-generation layer
  and carry heavier daemons and config surfaces than `mise.toml`.
- Silent body corruption is demonstrated for pnpm,
  Buck2 `genrule`,
  and double-quoted Poe `cmd`;
  any shell-string owner must ship the quoting compiler with tests.
- Buck2 local actions have ambient filesystem and network access
  (probe-confirmed),
  matching Mise's trust model but weaker than Pants' declared-input sandbox.
- Exit-code handling must treat any nonzero as failure
  (Buck2 uses 3,
  Pants mixes 1 and 241).
- The Pants pilot committed once inside its scratch fixture
  (`8448e0e`,
  no remote)
  because `--changed-since` needs git history.

## Next action

Please and Tilt pilots return into this comparison,
then the task-slice contract question is asked
with every candidate,
the ranking refrozen,
and adoption still waits for explicit acceptance
(`doc/decision/` record plus a reopened vet for the chosen candidate).
