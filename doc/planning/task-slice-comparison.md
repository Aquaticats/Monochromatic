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
 the ranking below is refrozen over every candidate
 on the capability axis
 the user set 2026-10-29
 (parity with the current status quo
 is a poor scoring axis;
 assume unlimited refactors),
 with migration and shape cost excluded from score;
 contract choice still open,
 no adoption performed.

## Candidates and evidence

Each pilot ran against the task-slice catalog of
[`mise-removal-coverage.md`](mise-removal-coverage.md)
in a disposable fixture whose path contains spaces,
under `~/temp/agent/`.
The catalog items were called parity gates until 2026-10-29;
they now read as capability measurements.

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

## Capability measurements

Formerly "Parity gate results".
The measurements stay
 as evidence of what each engine does.
Since 2026-10-29 they are not a scoring axis:
 parity with the current status quo
 is a poor axis,
 the refactor budget is unlimited,
 and shape results
 (names,
 labels)
 score nothing
 and are recorded as implementation notes.

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

## Scoring axis

The user corrected the axis on 2026-10-29:
 parity with the current status quo is a poor scoring axis,
and the refactor budget is unlimited.
Migration and shape cost therefore score nothing.

What scores is intrinsic capability
 for the slice's job at 222 packages.

### Requirements that stay

The slice must still deliver
 the behavior the acceptance boundary and the corpus demand:

- partial invocation of one package's one task,
- paths containing spaces,
- sequential and parallel groups,
- missing optional package builds,
- intentional failure tolerance,
- tests continuing after build failure where required,
- retained failure causes,
- machine-readable task listing,
- correct aggregate exit status.

### Cost that no longer scores

- label shape and the `//package/<path>:<task>` grammar,
- byte-exact preservation of today's inline TypeScript bodies,
- renames,
  adapters,
  codegen,
  stubs,
  quoting compilers,
  name mangling,
  target-generation layers.

Each remains an implementation note,
 and none is charged against a candidate.

### Criteria

Weight 1 each,
 because the user named no priority.

- A,
  edge fidelity:
  exact `depends` edges and groups.
- B,
  body model:
  safe execution of task logic;
  a typed form scores over a string form that needs encoding.
- C,
  failure semantics:
  intentional tolerance,
  tests continuing after build failure,
  retained causes,
  aggregate exit.
- D,
  optional-task robustness:
  what the engine does
  when a package has no such task.
- E,
  watch.
- F,
  caching and affected selection.
- G,
  machine-readable introspection.
- H,
  scale at 222 packages.
- I,
  maintenance depth.

Documentation trust is not a criterion.
It is a scored deduction of `4 - trust` points,
 per the drift policy set this session:
 unacknowledged drift deducts points,
 it never culls.

Thresholds used while scoring:
 F counts as covered at 2 or more,
 D counts as covered at 3 or more,
 and B is scored on the best measured safe form
 while naming the naive form's failure mode.
Where evidence is thin the cell is marked low-signal.

### Ratings

Format is A B C D E F G H I,
 then the total.

- Pants 2.33.1:
  4 4 3 2 4 4 4 4 3,
  total 32.
   Typed argv bodies (`adhoc_tool`)
  and typed edges (`execution_dependencies`)
  remove the encoding class entirely;
  caching and `--changed-since`
  with `--changed-dependents` are measured.
- Please 17.34.0:
  4 2 3 1 3 4 3 4 2,
  total 26.
   `escape_safe` is byte-exact
  but Please rewrites `\$` to `$`,
  so the intuitive escape is silently undone;
  the input-hash cache
  is the only cross-invocation cache measured end to end.
- Buck2 2026-10-14:
  4 2 4 2 0 2 4 4 4,
  total 26.
   Best failure semantics in the field
  and the richest introspection,
  no watch command at all,
  and `Cache hits: 0%` across every daemon restart probed.
- Nadle 0.6.1:
  4 4 2 1 4 4 4 1 1,
  total 25.
   Typed in-process TypeScript bodies
  remove the escaping class entirely;
  dependents of a failure are blocked,
  and every pilot ran at fixture scale.
- Keep Mise:
  3 2 3 2 4 0 3 4 3,
  total 24.
   Watch and listing work in daily use,
  nothing is cached at all,
  and root `mise run test` misses 15 packages.
- Tilt 0.37.8:
  4 3 1 4 3 1 2 2 1,
  total 21.
   Exact `resource_deps` and the best optional-task behavior measured,
  no caching,
  and batch mode abandons unstarted work.
- pnpm 12.16:
  1 1 2 4 0 2 2 4 3,
  total 19.
   Proven at 222 packages with pass-through missing scripts,
  but 476 of 603 cross-package edges
  become `^name` over-approximations.
- projen 0.104.19:
  2 4 0 1 0 0 2 2 2,
  total 13.
   `execArgs` is byte-exact,
  first failure aborts everything after it,
  and typed cross-project `spawn`
  silently resolves to the wrong task.
- Poe the Poet 0.49.0:
  4 3 1 0 0 0 0 2 2,
  total 12.
   Exact `deps` and a shell-free `cmd`,
  no watch,
  no caching,
  no listing,
  and one missing reference poisons every run.

Low-signal cells:
 Keep Mise B is inferred from corpus bodies
 rather than probed,
 so its total spans 23 to 25;
 Nadle H rests on fixture pilots only;
 projen H and Poe H rest on single-project pilots.

## Ranking for the task slice

Capability ranking,
 equal weights,
 ordered on the A to I totals.
Documentation trust is the scored deduction
 the session asked for
 (unacknowledged drift deducts points),
 so it breaks ties here
 and is reported beside each total.

1.  Pants 2.33.1
2.  Please 17.34.0
3.  Buck2 2026-10-14
4.  Nadle 0.6.1
5.  Keep Mise
6.  Tilt 0.37.8
7.  pnpm 12.16
8.  projen 0.104.19
9.  Poe the Poet 0.49.0
10. meow as designed,
    not scorable

Capability totals
 (A to I):
 Pants 32,
 Please 26,
 Buck2 26,
 Nadle 25,
 Keep Mise 24,
 Tilt 21,
 pnpm 19,
 projen 13,
 Poe the Poet 12.

Totals after the trust deduction
 (`4 - trust` points):
 Pants 29,
 Please 24,
 Buck2 23,
 Nadle 22,
 Tilt 17,
 pnpm 16,
 projen 11,
 Poe the Poet 11.
Keep Mise is not in the audit,
 so its deduction is unknown;
 at the field's median trust (1 of 4)
 it lands 21,
 fifth,
 and at 3 of 4 it lands 23,
 level with Buck2.
The deduction alone
 lifts Keep Mise above Nadle and Buck2,
 which is why each adjacent pair below
 names which axis decides it.

### Why each adjacent pair

- Pants beats Please because both cover every named duty
  and Pants covers each one at the higher grade:
   typed bodies where Please needs `escape_safe`,
  typed edges in `peek` JSON
  where Please reports them through `plz query graph`,
  `--loop` watch without Please's stale-`[cached]` bug,
  measured cache hits,
  and `--changed-since`
  with `--changed-dependents`
  as affected selection
  where Please's `--since` only lists the diff.
  Pants' warts are real
  (a dependent of a failure is silently omitted from the summary,
  a named missing address poisons `::`,
  the 30s default timeout),
  but each is a reporting or configuration wart
  on a duty the tool otherwise owns.
  Please's wins are D and C grade-offs
  plus the field's best cache.
   Flip condition:
  weight caching depth
  above body safety and watch cleanliness.
- Please and Buck2 tie on the totals
  (26 each),
  and the tie breaks toward Please
  on documentation trust
  (2 of 4 against 1 of 4)
  and on two of the three weightings
  (duty only 15 to 14,
  duty dominant 41 to 40).
  Buck2 is the better engine
  where it reaches:
   default failure tolerance
  with JSON `action_error` causes
  and aggregate exit 3 is the best in the field,
  and `targets --json`,
  `uquery`/`cquery`,
  `--build-report=-`,
  and BXL
  are the richest introspection measured.
  But Buck2 owns no watch command at all
  (only a protobuf `subscribe` RPC),
  which is a named duty,
  and its local cache measured `Cache hits: 0%`
  across every daemon restart,
  so caching is daemon-incremental only.
  Please owns watch and caching,
  with the two named warts
  (`plz watch` serving stale `[cached]` test results,
  `--since` checking revisions out in the live worktree).
   Flip condition:
  treat watch as wrapper-fillable
  and weight failure semantics plus introspection
  above watch and caching.
  That is the controlling preference for this pair.
- Buck2 beats Nadle
  by one point on the equal weighting
  (26 to 25),
  and the pair is the second load-bearing flip:
   both weightings that favor the named duties
  put Nadle second
  and drop Buck2 to fourth or fifth
  (duty only 18 to 14,
  duty dominant 43 to 40).
  Nadle's gaps are structural:
   it blocks every dependent of a failure
  at the scheduler level,
  so tests-after-build-failure fails outright,
  and `TaskNotFoundError` exits 3
  on any absent task
  unless `onlyIf` or generated empties model absence.
  Nadle's wins are the two strongest single cells
  in the comparison:
   typed in-process TypeScript bodies
  with zero escaping class,
  and a clean native watch
  plus input/output caching with `--why`.
  But Nadle is the only candidate
  whose scale evidence is fixture-scale alone
  and whose maintenance is one maintainer at 36 stars,
  while Buck2 carries Meta backing,
  monorepo-grade scale,
  the best failure semantics measured,
  and the richest introspection.
   Flip condition:
  weight the dev loop
  (watch and caching)
  above scale and maintenance,
  which the duty weightings do
  and which the equal weighting does not.
- Nadle beats Keep Mise
  because the incumbent's two measured holes
  are the ones the comparison set out to close:
   nothing is cached at all
  ("Nothing is cached,
  so the aggregates are slow by construction"),
  and root `mise run test` runs only the three root suites
  and misses 15 packages' tests.
  Keep Mise is genuinely strong elsewhere
  (watch in daily use across 104 `watch:build` tasks,
  `mise tasks --json` at full corpus size,
  proven operation at 222 packages).
  Nadle's own gaps
  (fixture scale,
  blocked dependents)
  are why the margin is one point
  rather than a tier.
   Flip condition:
  fix the two incumbent holes in place
  (add caching,
  fix the root sweep)
  and Keep Mise retakes this pair
  without any migration.
- Keep Mise beats Tilt
  because Tilt cannot own the failure-tolerance duty
  in the mode that exits:
   `tilt ci` aborts at the first failure
  and abandons unstarted resources
  with no flag to prevent it,
  and `tilt up` tolerates failures
  but never exits with an aggregate status,
  so a wrapper must invent the exit.
  Tilt's strengths are the sharpest in the field
  where they exist
  (exact `resource_deps`,
  the best optional-task behavior measured:
   unknown dependency warns and is ignored,
  run proceeds,
  exit 0),
  and Tilt does own watch.
  What sinks it is trust:
   0 of 4,
  the only zero,
  and both unacknowledged bugs
  land on the surfaces Tilt would own
  (`--output-snapshot-on-exit` corrupting its own JSON,
  `tilt trigger` reporting success for resources that never ran),
  plus zero caching.
   Flip condition:
  give `tilt ci` a keep-going mode
  and any caching at all.
- Tilt beats pnpm
  because pnpm cannot express the corpus graph:
   476 of 603 cross-package `depends` edges
  become `^name` over-approximations
  (the translate stage would run `build:js:node`
  in 16 extra packages),
  33 hold only transitively,
  and 5 are inexpressible.
  Over-approximation is not a migration cost,
  it is wrong work scheduled
  and wrong failure semantics:
   a failure in an unrelated same-named script
  blocks the dependent.
  Tilt's edges are exact
  and Tilt owns watch,
  which pnpm does not.
  pnpm's wins are the adoption side
  (proven at 222 packages,
  `missingScript: true` pass-through,
  experimental pipeline caching,
  `[since]` filters,
  a large ecosystem),
  and its body model has no measured byte-exact form:
   naive double-quoted compilation exits 0
  with the body gone.
   Flip condition:
  weight proven-scale and missing-optional pass-through
  above edge exactness and watch.
- pnpm beats projen
  because projen aborts the whole graph
  at the first failure
  and resolves typed cross-project `spawn`
  to the wrong task silently
  (the probe ran `APP build`
  where `LIB build` was the edge),
  a wrong-task-runs-green trap
  sitting on the edge-resolution duty itself.
  projen's `execArgs` byte-exactness
  and versioned `.projen/tasks.json` manifest
  are real,
  and it has no parallel step type
  (`Async commands are not supported`).
  pnpm at least keeps going on independent projects
  under `--no-bail`
  and schedules the right work plus extra.
   Flip condition:
  none on the measured evidence;
  projen would need honest spawn resolution
  and a keep-going mode.
- projen beats Poe the Poet
  on the totals by one point
  and the pair is a tie
  under the duty-only weighting.
  projen's decisive defect is silent wrongness:
   its typed cross-project `spawn`
  resolves to the same-named task in the same project
  (the probe ran `APP build`
  where `LIB build` was the edge),
  a wrong-task-runs-green trap
  sitting on the edge-resolution duty itself,
  and its first failure aborts everything after it.
  Poe's core is clean:
   exact `deps` that dedupe,
  and `cmd` with no shell at all.
  But three named duties are simply absent
  (watch,
  caching,
  listing)
  and optional-task robustness is the worst measured
  (one missing reference poisons every run
  including unrelated ones),
  and its failure semantics are the weakest measured
  (`sequence` aborts hide independent siblings,
  `ignore_fail = "return_non_zero"`
  double-runs shared deps
  and labels never-run tasks as failed).
  Poe holds the field's best trust
  (3 of 4,
  the only acknowledged drift),
  which is why the deduction closes the gap.
   Flip condition:
  emit stubs for every absent task
  and add watch and a listing shim,
  and Poe's body model plus trust
  carry it past projen and pnpm.
- meow as designed cannot be ranked
  because nothing exists to measure.
  Its design targets exactly the duties
  this axis rewards
  (typed argv bodies,
  watch daemon,
  caching,
  freshness),
  and the accepted endpoint already names it
  as the monorepo-manager owner.
  The custom-implementation gate still applies:
   a hand-written owner is eligible
  only when every existing tool
  fails a named hard constraint,
  and Pants does not.

### What the axis change moved

Scoring capability instead of incumbent fidelity
 reordered the field's middle
 and its bottom.

- Please rises from fourth to second.
  Its old penalty was the rename
  (`:` is reserved in target names),
  which the new axis charges nothing for,
  while its capability
  (best measured cache,
  real `--keep_going`,
  monorepo-grade scale)
  was always there.
- Keep Mise rises from last to fifth.
  Last place was an endpoint argument
  plus two defects,
  not a capability verdict.
  On capability it is mid-field,
  and the two defects
  (no caching,
  root sweep misses 15 packages)
  are named and fixable.
  The accepted endpoint
  (`doc/decision/monorepo-manager-all-rust.md`)
  still retires it as an owner;
  that is a decision,
  not a score.
- Nadle falls from second to fourth.
  Its old second place rested on
  typed bodies plus dev loop
  weighed against a rename cost
  it shared with Pants.
  With cost removed,
  what remains are its own gaps:
   fixture-scale evidence,
  one maintainer,
  blocked dependents,
  `TaskNotFoundError` on absent tasks.
- pnpm falls from fifth to seventh
  and lands last on the named duties.
  Its old case was incumbency,
  colon-name retention,
  and no rename.
  The first is an adoption fact,
  the second and third are shape,
  and both now score nothing.
  What remains is an edge model
  that cannot express the corpus graph.
- Poe the Poet falls from seventh to ninth
  for the same reason in reverse:
   its old case was shape retention
  and the best trust.
  Four named duties are absent.

### Weight sensitivity

Three orderings over the same cells.

- Duty only
  (A B C E G):
   Pants 19,
  Nadle 18,
  Please 15,
  Keep Mise 15,
  Buck2 14,
  Tilt 13,
  Poe the Poet 8,
  projen 8,
  pnpm 6.
- Duty dominant
  (named duties doubled):
   Pants 51,
  Nadle 43,
  Please 41,
  Buck2 40,
  Keep Mise 39,
  Tilt 34,
  pnpm 25,
  projen 21,
  Poe the Poet 20.
- Equal
  (the ranking above):
   Pants 32,
  Please 26,
  Buck2 26,
  Nadle 25,
  Keep Mise 24,
  Tilt 21,
  pnpm 19,
  projen 13,
  Poe the Poet 12.

What holds and what flips.

- Pants is first under every weighting.
  This is the only position
  that no weighting disturbs.
- The band from Please through Keep Mise
  (Please,
  Buck2,
  Nadle,
  Keep Mise)
  reorders freely with the weighting,
  and the controlling preference
  in each case is named in the pair notes.
  Duty weighting favors Nadle
  because watch and body model are named duties
  and Nadle grades highest on both.
- Nadle versus Please is the load-bearing flip:
   Nadle second on duty weighting,
  Please second on equal weighting.
  The lever is scale evidence and maintenance
  (Please)
  against body model and native watch quality
  (Nadle).
- Buck2 versus Nadle flips the same way:
   Buck2 third on equal weighting,
  fifth on duty weighting,
  because watch is a named duty
  and Buck2 has no watch command.
- Poe the Poet versus projen is a tie
  on duty weighting
  and a one-point gap on equal weighting,
  closed entirely by the trust deduction.
- pnpm is last on the named duties
  and seventh overall.
  Its total is carried by H and I
  (proven at 222 packages,
  large ecosystem),
  which are adoption factors,
  not slice duties.

## Risks

Migration and shape work is real
but scores nothing
(see the Scoring axis section):
Nadle,
Buck2,
Pants,
and Please each need
a rename,
a label mapping,
or a generation layer
over 1,811 names,
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
