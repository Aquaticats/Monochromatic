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
 all eight pilots complete 2026-10-29
 (pnpm 12.16,
 Nadle 0.6.1,
 Poe the Poet 0.49.0,
 Buck2 2026-10-14,
 Pants 2.33.1,
 projen 0.104.19,
 Please 17.34.0,
 Tilt 0.37.8);
 the ranking below is refrozen over every candidate;
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
- Please 17.34.0 (`thought-machine/please`,
  Apache-2.0):
  report `~/temp/agent/please-pilot-report-2026-10-29.md`,
  fixture `~/temp/agent/please pilot spaces.x7`,
  clone `~/temp/agent/please-src-2026-10-29` at `a07c97592aaafdb4daa8271796c924a2a14fe184`.
- Tilt 0.37.8 (`tilt-dev/tilt`,
  Apache-2.0):
  report `~/temp/agent/tilt-pilot-report-2026-10-29.md`,
  fixture `~/temp/agent/tilt pilot spaces.x8`,
  clone `~/temp/agent/tilt-src-2026-10-29` at `f3acf27d8459924249776d72a82e1d5d99139c33`.
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
Only Poe the Poet,
projen,
and Tilt accept the colon names verbatim.

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
- Tilt:
  colon resource names run (`build:js:node`),
  while `//package/<path>:<task>` is rejected as a resource name
  (`may not contain '/'`)
  and as a label,
  and one flat namespace hard-errors on duplicates.
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

- Please:
  the colon is reserved in target names
  and label parsing splits on the first colon,
  so `build:js:node` cannot exist in either position:

```text
# plz build //pkgs/lib:build:js:node
Invalid build label: //pkgs/lib:build:js:node
# plz build //pkgs/names:all (genrule name = "build:js:node")
: is a reserved character in build target names
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
 Pants,
 and Please each need a name-mangling compiler over 1,811 task names,
 603 `depends` edges,
 `CM5` addressing,
 and every call site.
Buck2,
 Pants,
 and Please preserve more of the shape than Nadle
(`build.js.node` keeps its separators in all three;
Nadle allows only letters,
numbers,
and dashes).
Poe the Poet,
projen,
and Tilt keep the names
but address tasks as one flat namespace,
so `//package/<path>:<task>` still needs a mapping layer
and Tilt makes the loss explicit in its name validator.

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
- Please:
   partial;
  `//pkg:task` works with one colon,
  colon task names are impossible,
  package namespace survives in the label path.
- Tilt:
   fail;
  colon names work,
  `//package/<path>:<task>` is rejected as name and label,
  duplicates hard-error in one flat namespace.
- projen:
   fail;
  identity is project directory plus flat name.

### Partial invocation

All eight measured candidates pass:
 one package,
 one task,
 one invocation.

### Paths containing spaces

All eight pass;
every pilot ran its whole probe catalog from a spaced fixture root.
Two quoting caveats are measured:
 Poe the Poet word-splits unquoted `${VAR}` expansions,
 and Please word-splits unquoted `$OUT` and `$SRCS`
 (the `build_rules` example idiom breaks verbatim),
 so both compilers must quote.

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
- Please:
   partial;
  `plz run sequential` and `plz run parallel` exist
  but only for targets marked `binary = True`.
- Tilt:
   partial;
  `resource_deps` enforce ordering
  and `allow_parallel = True` enables concurrency,
  no group abstraction.
- projen:
   sequential pass,
  parallel fail
  (`Async commands are not supported. Run a command concurrently in the JS code instead.`).

### Missing optional package tasks

This gate separates the field most sharply.

- pnpm:
   pass;
  a missing script becomes `missingScript: true` and passes through.
- Tilt:
   pass,
  the best measured behavior;
  `resource app:lint specified a dependency on unknown resource optional:check - dependency ignored`
  and the run proceeds.
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
- Please:
   fail;
  `Parsed build file pkgs/opt2/BUILD but it doesn't contain target extra`
  is a hard error;
  lazy parsing spares unrelated single-target runs,
  while wildcard and whole-graph operations break.
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
- Please:
   partial;
  `--keep_going` builds other prerequisites of a doomed target
  with causes retained,
  but the default cancels unstarted work
  and the flag is absent from the website docs.
- Poe the Poet:
   fail;
  `sequence` aborts hide independent siblings,
  `ignore_fail = "return_non_zero"` recovers them
  at the cost of double-running shared deps
  and labeling never-run tasks as failed.
- Tilt:
   fail;
  `tilt ci` aborts at the first failure
  and abandons unstarted resources with no flag to prevent it;
  `tilt up` tolerates failures
  but never exits with an aggregate status.
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
- Please:
   partial;
  with `--keep_going` the independent test ran
  while the build failed;
  without it the schedule decides.
- Buck2:
   partial;
  independent tests run,
  a test depending on a failed build never runs.
- Pants:
   pass for independent tests,
  same dependent-blocked shape.
- Poe the Poet:
   fail;
  the graph aborts before the dependent test.
- Tilt:
   fail;
  the independent test was never attempted under `tilt ci`.
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
- Buck2:
   pass;
  exact command line,
  stdout and stderr,
  JSON `action_error` with digest and error tags.
- Pants:
   pass;
  `ProcessExecutionFailure` with stdout,
  plus `--keep-sandboxes=on_failure`.
- Please:
   pass;
  aggregates name the failing target with error text and stdout,
  xUnit `test_results.xml` retains the cause.
- Tilt:
   pass;
  `buildHistory[0].error` and `conditions` retain the cause,
  `waiting-for-dep` distinguishes skips from pending.
- Poe the Poet:
   partial;
  aggregates name failing subtasks,
  skipped dependents are misreported as failures.
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
- Please:
   partial;
  `plz query graph` is JSON
  with per-target deps,
  outs,
  command,
  and hash,
  but no task catalog shape.
- Tilt:
   partial;
  `tilt get ... -o json` is rich
  but needs a live server,
  and the offline snapshot writer corrupts longer existing files.
- projen:
   partial;
  `.projen/tasks.json` is a stable manifest
  but the runner has no JSON output mode.
- Poe the Poet:
   fail;
  `_list_tasks` prints names only,
  no JSON and no edge dump.

### Correct aggregate exit status

All eight return nonzero on failure;
Buck2 returns 3,
Please returns 2 for builds and 7 for tests,
Pants returns 1 or 241 by failure class,
Tilt and projen collapse child codes to 1.

## Task bodies and escaping

Typed argv bodies are safe wherever they exist;
every shell-string form needs a quoting compiler,
and naive compilation corrupts silently
in pnpm,
Buck2,
Please,
and Tilt.

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
- Please:
   BUILD string,
  Please's own `\$` rewrite,
  and shell quoting;
  Please rewrites `\$` to `$` before bash sees the command,
  so shell-level dollar escaping is silently undone
  (measured),
  while correct single-quote encoding is byte-exact.
- Tilt:
   argv-list form is byte-exact;
  string form runs `sh -c`
  and silently strips quotes and expands `${x}`.
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
 Please `escape_safe`,
 Tilt argv-list,
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
- Please:
   caching pass
  (input-hash skip across separate invocations),
  watch partial
  (build targets rebuild;
  watched test bodies served `[cached]` after source changes),
  affected selection pass-with-caveat
  (`plz query changes --since` checks the ref out in the live worktree).
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
- Tilt:
   caching absent,
  watch pass
  (auto rerun under `tilt up`,
  `tilt trigger` for manual runs),
  affected selection absent.
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
 Please is single-maintainer-leaning
 (Peter Ebden 22 of 28 recent human commits,
 active releases),
 Tilt is concentrated
 (top human contributor about 3.5 times the second,
 monthly releases),
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
   genrule docs
  ([bash sentence, selected](https://github.com/facebook/buck2/blob/9ac63c5839fc5a874e766947e276a5fd0292789e/prelude/decls/genrule_common.bzl#L100))
  say `bash -c`,
  the observed invocation is `bash -e <script>`,
  and `--keep_going` means different things on `build` versus `targets`.
  The sandbox footnote
  ([sandbox footnote, selected](https://github.com/meta/buck2#:~:text=Buck2%20currently%20does%20not%20sandbox))
  is accurate and is not drift:
   probes confirm it
  (a local action read `/etc/os-release`
  and `curl` returned `http_code=200`),
  so the contradiction belongs to a third-party security write-up
  claiming sandbox-by-default,
  not to upstream docs.
- Pants:
   `pants help shell_command` advertises a goal that does not exist,
  the shell docs
  ([outputs sentence, selected](https://www.pantsbuild.org/stable/docs/shell/run-shell-commands#:~:text=list%20them%20in%20the%20outputs%20field))
  name a nonexistent `outputs` field,
  PyPI `pantsbuild.pants` is frozen at 2.17.1.
- Please:
   the commands doc
  ([-a sentence, selected](https://please.build/commands.html#:~:text=arguments%20must%20be%20passed%20one%20by%20one%20via%20the))
  documents an `-a` form that fails verbatim
  (`the required argument 'target (at least 1 argument)' was not provided`),
  `--keep_going` is absent from the website docs entirely,
  and the build-rules example leaves `$SRCS` and `$OUT` unquoted.
- Tilt:
   the `tilt ci` doc
  ([snapshot sentence, selected](https://docs.tilt.dev/cli/tilt_ci.html#:~:text=If%20specified%2C%20Tilt%20will%20dump%20a%20snapshot%20of%20its%20state))
  promises a state snapshot
  while the writer omits `O_TRUNC`
  (invalid JSON over a longer existing file,
  reproduced),
  and `tilt trigger` reports success for resources that never run.
- projen:
   the tasks doc
  ([POSIX sentence, selected](https://github.com/projen/projen/blob/c8b3fee593c07e38a258ccffd2e57c8eba505b8c/docs/concepts/tasks.md?plain=1#L241))
  claims POSIX-style syntax the default shell rejects,
  generated markers say `.projenrc.js` where `.projenrc.ts` is generated.

Decision 2026-10-29 (drift weight):
 documentation drift is a scored deduction this session,
 never a cull.
Unacknowledged contradictions deduct harder than acknowledged ones,
 where upstream is already fixing the text.
This replaces the earlier plan to cull unacknowledged drift outright.

### Upstream acknowledgement audit

- Poe the Poet:
   acknowledged.
  Open [PR #438](https://github.com/nat-n/poethepoet/pull/438)
  ("docs:
  fix misc inaccuracies")
  is a maintainer-authored docs audit against 0.49.0,
  and the include-`cwd` family was acknowledged and fixed
  ([#158](https://github.com/nat-n/poethepoet/issues/158),
  [#160](https://github.com/nat-n/poethepoet/issues/160)).
  Trust score 3 of 4.
- projen:
   partially acknowledged.
  [#4267](https://github.com/projen/projen/issues/4267)
  has the maintainer agreeing the quoting gap is real
  ("projen can either A) do nothing (the current behavior) or B) always quote all args"),
  but the POSIX-syntax claim itself is not retracted.
  Trust score 2 of 4.
- Please:
   partially acknowledged.
  [#522](https://github.com/thought-machine/please/issues/522)
  records the maintainer calling `--keep_going` semantics
  "not the desired behaviour",
  while the documented `-a` form that fails verbatim
  has no upstream record.
  Trust score 2 of 4.
- pnpm:
   unacknowledged.
  [pnpm/pnpm.io#987](https://github.com/pnpm/pnpm.io/issues/987)
  (ours,
  open 2026-10-28) drew zero maintainer response,
  and [pnpm/pnpm.io#519](https://github.com/pnpm/pnpm.io/issues/519)
  shows the docs queue is unattended.
  Trust score 1 of 4.
- Nadle:
   unacknowledged.
  [PR #614](https://github.com/nadlejs/nadle/pull/614)
  merged a correct `into` guide
  while the features page still shows `to`,
  and no issue names the contradiction.
  Trust score 1 of 4.
- Buck2:
   unacknowledged.
  No `meta/buck2` issue covers the genrule `bash -c` claim
  or the `--keep_going` split between `build` and `targets`.
  Trust score 1 of 4.
- Pants:
   unacknowledged.
  The stale `experimental_run_in_sandbox` help reference
  and the nonexistent `outputs` field have no upstream record;
  [#17616](https://github.com/pantsbuild/pants/issues/17616)
  explains the rename that left the stale text behind
  without acknowledging the text.
  Trust score 1 of 4.
- Tilt:
   unacknowledged and the largest deduction.
  Neither the snapshot truncation nor the `tilt trigger` false success
  appears in any issue;
  [#6553](https://github.com/tilt-dev/tilt/issues/6553)
  reports a different snapshot bug (a race),
  and [#4811](https://github.com/tilt-dev/tilt/issues/4811)
  calls the feature "half-finished" without naming either.
  Trust score 0 of 4.

## Ranking for the task slice

Refrozen after all six added pilots returned
and after the drift weight was set to deduction
(see the acknowledgement audit above).
Documentation trust is one scored factor among the parity gates,
so the order below is stable under that weighting
and the margins it narrows are named per pair.

1.  Pants 2.33.1 with a name-mangling compiler and generated BUILD files.
2.  Nadle 0.6.1 with a rename and missing-optional modeling.
3.  Buck2 2026-10-14 with name flattening and a target-generation layer.
4.  Please 17.34.0 with a rename,
 stub emission,
 and a `--keep_going` wrapper contract.
5.  pnpm 12.16 with adapters and the three recorded shifts.
6.  Tilt 0.37.8 with a `tilt up` wrapper that invents the aggregate exit.
7.  Poe the Poet 0.49.0 with a quoting compiler and stub emission.
8.  projen 0.104.19 with hand-written aggregation tasks.
9.  meow as designed.
10. Keep Mise.

### Why each adjacent pair

- Pants beats Nadle because the migration cost is identical
  (both ban the colon vocabulary,
  so both need a compiler over 1,811 names,
  603 edges,
  `CM5` addressing,
  and every call site)
  while Pants covers the acceptance boundary more completely:
   per-package missing-task sweeps pass
  (`./pants test pkgs/free/:: pkgs/bench/::` exits 0),
  independent work continues after a failure
  with `ProcessExecutionFailure` retaining stdout
  and `--keep-sandboxes=on_failure`,
  caching is measured (`(cached locally)`,
  `local_cache_requests_cached: 1`),
  and `--loop`,
  `--changed-since` with `--changed-dependents`,
  and `peek` JSON carrying `execution_dependencies` are all measured.
  Nadle's typed in-process TS bodies are the only form
  that removes the escaping class entirely,
  and `--reporter agent`,
  `--why`,
  and `--since` are real,
  but Nadle hard-errors on any missing task
  (`TaskNotFoundError` exit 3)
  unless `onlyIf` or generated empties model absence,
  and dependents of a failure are blocked.
  Trust ties at 1 of 4,
  so drift does not separate them.
   Flip condition:
  weight typed-body safety above failure tolerance and caching.
- Nadle beats Buck2 because the slice needs watch and caching
  and Buck2 has neither:
   no watch command at all
  (only a protobuf `subscribe` RPC),
  and `Cache hits: 0%` across every daemon restart probed,
  which makes its local cache daemon-incremental only.
  Nadle owns `--watch`,
  input/output caching with `--why`,
  and `--continue`.
  Buck2 is stronger exactly where Nadle is weak:
   gate 6 is a default pass
  (independent targets run in one invocation,
  exact command line plus stdout plus stderr retained in JSON `action_error`,
  aggregate exit 3),
  and `targets --json`,
  `uquery`/`cquery`,
  `--build-report=-`,
  and BXL form the richest machine-readable surface measured.
  But Nadle's missing-optional gap is modelable
  (`onlyIf`,
  generated empty tasks)
  while Buck2's cache gap is architectural for a long-lived dev loop,
  and Buck2 `genrule` bodies cross three escaping layers
  (Starlark literal,
  `$(...)` macro expansion,
  shell quoting)
  where the naive form built green with an empty output file,
  the worst silent corruption measured.
  Trust ties at 1 of 4.
   Flip condition:
  a working remote or RE cache,
  or a watch wrapper,
  puts Buck2 first.
- Buck2 beats Please because the acceptance boundary's
  failure-tolerance gate is a default pass in Buck2
  and only a partial pass in Please behind `--keep_going`,
  a flag absent from the website docs entirely
  whose semantics the maintainer calls
  "not the desired behaviour"
  ([#522](https://github.com/thought-machine/please/issues/522))
  and whose summary text (`Build stopped after ...`)
  misleads even when everything ran.
  Please charges the same rename cost
  (`:` is reserved in target names)
  and its shell-only bodies carry a measured `\$` un-escape:
   Please rewrites `\$` to `$` before bash sees the command,
  silently undoing dollar escaping.
  Buck2 at least offers a typed argv escape hatch.
  Please's wins are real and are the slice's other requirements:
   input-hash caching across invocations measured end to end,
  `plz watch`,
  and `--since` with `--level` reverse-dependents expansion.
  Trust favors Please (2 of 4 versus 1 of 4),
  which narrows this pair without flipping it.
   Flip condition:
  weight cross-invocation caching and watch
  above default failure tolerance.
- Please beats pnpm because `depends` edges are the slice's core semantics
  and pnpm cannot express them:
   476 of 603 cross-package edges become `^name` over-approximations
  (translate-stage would run `build:js:node` in 16 extra packages),
  33 hold only transitively,
  and 5 are inexpressible and need new `package.json` edges.
  Over-approximation changes failure semantics:
   a failure in an unrelated same-named script blocks the dependent.
  Please's `deps` are exact per target
  (measured:
  `lib:build` plus `corpus:build`,
  and never `bench:build`),
  caching is input-hash based across invocations,
  and watch and `--since` exist
  where pnpm has no watch at all
  and experimental pipeline caching.
  pnpm's clean wins are the missing-optional pass-through
  (`missingScript: true`),
  keeping the colon names as per-project script keys without collision,
  and already being the repo's package manager.
  Its failure tolerance is partial
  (`--no-bail` continues independents,
  dependents of a failure never run)
  and tests-after-build-failure fails at the scheduler level.
  Trust favors Please (2 of 4 versus 1 of 4).
   Flip condition:
  accept the 476 over-approximations as harmless in practice
  (the translate-stage measurement is the test)
  and weight no-rename above edge exactness.
- pnpm beats Tilt because `tilt ci` cannot own the failure-tolerance gate
  at all:
   it aborts at the first failure
  and abandons unstarted resources
  with no flag to prevent it,
  and the only tolerant mode (`tilt up`)
  never exits with a status,
  so a wrapper must invent the aggregate exit.
  `--no-bail` at least continues independent work,
  pnpm caches where Tilt has no caching whatsoever
  (every `tilt ci` is a fresh engine),
  and pnpm's colon names sit in per-project script keys
  where Tilt's one flat namespace hard-errors on duplicates,
  which 222 packages of `test` and `lint` and `build` will hit
  without mangling.
  Tilt's strengths are the strongest in the field
  where they exist:
   exact `resource_deps`,
  the best missing-optional behavior measured
  (unknown dependency warns and is ignored,
  run proceeds,
  exit 0),
  `buildHistory[0].error` with `waiting-for-dep` reasons,
  and watch.
  What sinks it is trust:
   0 of 4,
  the only zero in the field.
  `--output-snapshot-on-exit` silently corrupts its own JSON
  over a longer existing file
  (missing `O_TRUNC`,
  reproduced 99,999 bytes with a stale tail)
  and `tilt trigger` reports success for resources that never ran.
  Both are unacknowledged,
  and both land on the listing and watch surfaces Tilt would own.
  pnpm's own drift is one stale sentence (1 of 4).
   Flip condition:
  a keep-going mode for `tilt ci` plus any caching
  would put Tilt ahead of pnpm;
  its gate-5 and edge-exactness wins are otherwise decisive.
- Tilt beats Poe the Poet because Poe fails the missing-optional gate
  hardest in the field:
   one invalid name poisons every run in the project,
  including unrelated ones
  (`Error: Invalid task 'needs:missing'`),
  which for 222 packages with optional builds is an operational blocker
  unless the generator emits a stub for every absent task.
  Poe's failure tolerance is the weakest measured
  (`sequence` abort hides independent siblings;
  `ignore_fail = "return_non_zero"` recovers them
  at the cost of double-running shared deps
  and labeling never-run tasks as failed),
  and Poe has no watch,
  no caching,
  and no JSON listing at all
  where Tilt has watch and partial JSON
  and the best gate 5 in the field.
  Poe's own strengths are the cleanest body model after Nadle's
  (`cmd` has no shell;
  POSIX single-quote encoding is byte-exact
  against the measured body),
  exact deduping `deps` edges,
  and the field's only acknowledged drift (3 of 4).
  Against Tilt's 0 of 4 that makes this the closest pair in the ranking,
  but Poe's missing surfaces are slice requirements Tilt partially covers.
   Flip condition:
  stub emission that cleans gate 5
  plus a watch shim
  lets Poe's trust and body model carry it past Tilt.
- Poe the Poet beats projen because projen has no package namespaces at all
  (identity is project directory plus flat name,
  so `//package/<path>:<task>` cannot be expressed),
  no parallel step type
  (`Async commands are not supported`),
  no keep-going (first failure aborts everything after),
  and its typed cross-project `spawn` silently resolves
  to the same-named task in the same project:
   the measured probe ran `APP build`
  where `LIB build` was the edge,
  a wrong-task-runs-green trap.
  Missing tasks are either hard errors
  or false-green empty stubs.
  Poe's graph dedups,
  offers parallel and `ignore_fail` modes,
  and its failures at least fail loudly;
  Poe also holds the trust edge (3 of 4 versus 2 of 4).
  projen's `execArgs` byte-exactness
  and versioned `.projen/tasks.json` manifest do not compensate
  for an inexpressible label namespace.
   Flip condition:
  projen gaining namespaces,
  parallel steps,
  and honest spawn resolution.
- projen beats meow because it runs today:
  measured,
  installable,
  with a manifest the file-enforcer model can generate toward.
  meow is plan-only
  with open design questions in scheduling,
  caching,
  and the watch daemon.
   Flip condition:
  meow existing and passing the parity gates.
- meow beats keeping Mise because the accepted endpoint is Mise removal
  (`doc/decision/monorepo-manager-all-rust.md`
  selected meow as the owner)
  and the incumbent carries the measured defects of
  [`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md):
   no caching at all,
  and root `mise run test` misses 15 packages.

### Drift-weight sensitivity

Scoring documentation trust instead of culling it
changes no position in the ranking above.
It narrows two margins:
 Buck2 to Please
 (Please holds 2 of 4 against Buck2's 1 of 4
 on top of its caching and watch wins)
 and Tilt to Poe the Poet
 (Poe holds 3 of 4 against Tilt's 0 of 4).
Under a weighting that puts documentation trust
above watch and machine-readable listing,
Tilt drops below Poe the Poet
and that is the only inversion available from the drift factor alone.

## Risks

- Name migration is structural for Nadle,
  Buck2,
  Pants,
  and Please:
   1,811 names,
  603 edges,
  and the `//package/<path>:<task>` call sites.
- Buck2,
  Pants,
  and Please are build systems,
  not task runners;
  each needs a target-generation layer
  and carries heavier daemons and config surfaces than `mise.toml`.
- Silent body corruption is demonstrated for pnpm,
  Buck2 `genrule`,
  Please (both quoting and the `\$` un-escape),
  Tilt string commands,
  and double-quoted Poe `cmd`;
  any shell-string owner must ship the quoting compiler with tests.
- Buck2 local actions have ambient filesystem and network access
  (probe-confirmed),
  matching Mise's trust model but weaker than Pants' declared-input sandbox.
- Exit-code handling must treat any nonzero as failure
  (Buck2 uses 3,
  Please 2 and 7,
  Pants mixes 1 and 241).
- Tilt's batch mode cannot own failure tolerance at all;
  its only tolerant path (`tilt up`) never exits with a status,
  so a wrapper must invent the aggregate exit.
- Please's failure tolerance hides in `--help` only,
  and its `--since` checks revisions out in the live worktree,
  refusing dirty ones.
- The Pants pilot committed once inside its scratch fixture
  (`8448e0e`,
  no remote)
  because `--changed-since` needs git history.

## Next action

The drift weight is decided
(unacknowledged documentation drift deducts points,
it does not cull),
and the ranking is refrozen with per-pair justifications and flip conditions.
The contract question remains open over the full candidate set
and is asked once the user has read the ranking;
adoption still waits for explicit acceptance
(`doc/decision/` record plus a reopened vet for the chosen candidate).
