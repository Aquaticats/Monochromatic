# Task-slice migration to Pants

Status:
 proposal,
 written 2026-10-29 after the user answered
 "2"
 to the adoption question
 (adopt the owner,
 write the migration plan first,
 adopt after reading it),
 and added the scope statement
 "we're still going to migrate off pants to meow later when we have capacity".
No `doc/decision/` record yet,
 no task file changed.

## Scope

Two hops,
 not one.

- Hop 1:
  Mise to Pants 2.33.1,
  the task-execution slice,
  now.
- Hop 2:
  Pants to meow,
  later,
  when capacity allows.

Because hop 2 is planned,
 hop 1 is written for cheap reversal
 and cheap second migration.
That constraint shapes every decision below
 and it is the reason the plan preserves
 a vocabulary that costs something to keep.

## What must survive both hops

### The `//package/<path>:<task>` vocabulary

Keep it as the addressing form
 through both hops.

This is not incumbent fidelity.
`meow`'s accepted command surface is already
 `meow run //package/cow:test`
 ([`monorepo-manager-from-scratch-design.md`](monorepo-manager-from-scratch-design.md)),
 so the label shape is the *target* shape
 and keeping it makes hop 2
 a leading-token change at call sites
 instead of a second re-addressing.

Measured call-site surface today:

- 104 code call sites in 78 files
  match `mise run //...`
  (`.ts`,
  `.toml`,
  `.rs`,
  `.mjs`,
  `.kts`);
- 2,418 total matches including docs and fixtures.

### Task bodies

Keep each body as the same argv plus inline TypeScript
 it is today,
 so hop 2 moves the text
 rather than re-deriving it.

### The acceptance boundary

`package/monorepo-task/core-exec/src/build-and-test.unit.test.ts`
 (moved 2026-10-29 by concurrent work to
 `package/monorepo-task/core-exec/src/build-and-test.unit.test.ts`,
 staged and not yet committed)
 stays green after every wave.

## Integration boundary

Pants owns,
 for the task slice only:

- task discovery and per-package addressing;
- `depends` edges;
- task bodies;
- watch;
- failure tolerance with retained causes and aggregate exit;
- machine-readable listing.

Pants does not take:

- file-enforcer and root-file generation;
- toolchain provisioning (`toolchain:sync`);
- package publishing,
  secrets,
  or the age-encrypted `env.local.json` path;
- the `meow` build itself.

Mise's `sync:files` tasks leave with Mise,
 as the endpoint decision already records.

## Name and label mapping

Measured 2026-10-29 over `mise.toml` and `mise.no-env.toml`
 outside worktrees:
 508 distinct task names,
 of which 408 contain a colon
 (`build:js:node`,
 `lint:types`,
 `watch:build`).
Pants target names and address specs both reject the colon
 (Buck2's wording is "the colon is the label separator";
 Pants' error is `expected EOF`),
 so the generated layer must mangle.

Decision:
 keep the colon names
 as the source of truth,
 mangle only inside the generated Pants layer,
 and record the map
 so hop 2 can invert it.
Mangling is `:` to `.`
 (`build:js:node` to `build.js.node`),
 the form Buck2,
 Pants,
 and Please all accept
 (Nadle is the outlier,
 allowing only letters,
 numbers,
 and dashes,
 which is one reason the map is kept
 and not baked into names).

Open item for hop 2:
 `meow`'s label grammar does not yet decide
 whether a task name may contain a colon
 (the design keeps `meow run //package/cow:test`
 without deciding what the label refers to).
If `meow` rejects colons,
 the mangling becomes permanent
 and the map retires.

## Task source of truth

Keep the current task metadata
 where it lives today
 (root `mise.toml` plus the package `mise.toml` files)
 as the source of truth during the interim,
 and generate the Pants layer from it.

Reason:
 `meow`'s task model is still unsettled
 ("We need to settle on 'what is a task' first too",
 and task ordering is explicitly undecided),
 so writing `meow.hcl` now
 would bake in a model the user has not approved.
Writing `meow.hcl` is hop 2's own work.

Consequence:
 the generated Pants layer is disposable
 and the source files never move during hop 1.

## Target generation

One generator,
 owned by `monorepo-task`
 (`RCI`:
 extend the package that already owns task orchestration,
 `@monochromatic-dev/monorepo-task-core-exec`
 and `@monochromatic-dev/monorepo-task-cli`),
 emitting one `BUILD` file per package
 from that package's task metadata.

`file-enforcer.config.ts` already owns `mise.toml` generation,
 so the generator is added beside it
 and the two share the parse
 rather than each re-reading the corpus.

Generator duties:

- read the task headers,
  `extends` chains,
  `run` arrays,
  `depends`,
  `dir`,
  `hide`,
  and `{{vars.}}` values;
- expand `{{vars.}}` at generation time,
  since Pants has no template layer
  (1,159 of 1,836 tasks carry `{{vars.`);
- mangle names and write the name map;
- emit `dependencies`
  (or `execution_dependencies` where order is required)
  from the `depends` lists;
- emit `timeout`,
  `environment`,
  and `output` fields
  from the body shape.

## Task bodies

- `adhoc_tool` argv lists,
  one entry per argv element,
  no shell strings.
- Use a local-workspace environment target
  (`LocalWorkspaceCompatiblePlatformsField`)
  so the process is a pure exec.
  Without it `prepare_adhoc_process` wraps argv
  in `bash -c "cd <workdir> && <shlex.join(argv)>"`,
  which is safe against injection
  but puts a `bash` in the tree
  and is the wrong default for hop 2.
- `runnable_dependencies` always generate bash shims;
  keep them out of the slice
  unless a body truly needs them.
- The 41 tasks that use `depends`
  and the fan-out bodies
  in `monorepo-task/core-exec`
  keep their tests-after-build-failure behavior,
  which is the acceptance boundary.

## The four carry-in items

These are what adoption must build,
 because upstream does not provide them.

- Summary line for the dependents of a failed build.
  Build it in the interim dispatcher,
  not in Pants:
   compute the blocked set from `peek`
  and the run result,
  and print it after the run.
- Aggregate exit that survives a raised failure.
  The engine's `concurrently` path is fail-fast
  (`future::try_join_all`),
  so one raised `ProcessExecutionFailure`
  aborts before any summary.
  Keep bodies failing as captured results
  (nonzero exit from `adhoc_tool`),
  which is the path that continues independents,
  and make the dispatcher derive the aggregate exit.
- Task timeout above 30 seconds.
  `AdhocToolTimeoutField` defaults to 30.
  Emit a repo default
  (proposal:
  900 seconds)
  on every generated target
  and allow per-task overrides.
- Watch and cache configuration the repo owns.
  `pants.toml` carries `loop_max`,
  `local_cache`,
  `--local-store-dir`,
  and must opt out of anonymous telemetry.

## Command surface

Open item,
 ranked.

1.  A `meow`-shaped stub:
    `meow run //package/<path>:<task>`
    dispatched to Pants,
    deleted when real `meow` lands.
    Call sites are rewritten once
    and never again.
    Con:
     a binary named `meow` that is not `meow`.
2.  A neutral interim command:
    `task //package/<path>:<task>`
    in `monorepo-task/cli`.
    Call sites change once to `task`
    and once more to `meow run` at hop 2.
    Con:
     two mechanical rewrites.
3.  `pants` goals directly.
    Call sites learn the Pants address form
    and the whole vocabulary moves twice.
    Con:
     worst for hop 2.

Ranking 1 > 2 > 3,
 because hop 2 is already planned
 and the only cost that compounds
 is the call-site token.

## Rollout

Each wave is gated on the acceptance boundary
 and on a dual-run diff.

- Wave 0:
   the generator,
   one pilot package,
  dual-run against Mise
  and diff the outputs.
- Wave 1:
   the root aggregates
  (`build`,
  `test`,
  `lint`,
  `watch:*`),
  dual-run,
  diff.
- Wave 2:
   per-package cutover by category,
  dual-run per package,
  diff.
- Wave 3:
   rewrite the call-site leading token,
  retire `mise run`,
  and delete the generated layer's name map
  only at hop 2.

Docs (`AGENTS.md` `CM3`,
 `CM5`,
 and the 2,418 mentions)
 follow wave 3,
 not before.

## Reversal

- Rollback is deletion.
   The generated `BUILD` files are disposable,
  the task metadata stays authoritative
  until wave 3 completes,
  and reverting is deleting the generated files
  plus restoring the call-site token.
- Exit to meow
  (hop 2):
   the call sites change the leading token,
  the name map inverts,
  bodies move into meow's argv model,
  and the generator's input becomes `meow.hcl`.

## Revisit triggers

- Capacity for the meow hop.
- `meow`'s task model settled
  (what is a task,
  what creates ordering).
- `meow`'s label grammar settled
  for a colon inside a task name.

## Risks carried into hop 2

Named here so hop 2 starts with them open,
 not discovered.

- Task ordering is undecided in the meow design,
  while this corpus carries 603 cross-package `depends` edges
  (the figure the pnpm pilot measured),
  41 tasks with a non-empty `depends`,
  and a `run`-array convention
  that the design's `depends_on` entry retracted.
  A live recount on 2026-10-29
  over `mise.toml` and `mise.no-env.toml`
  outside worktrees finds 182 `depends` lines
  expanding to 673 entries,
  so the migration re-measures the edge set
  at wave 0
  rather than trusting either figure.
- meow's cross-package edges
  come from native manifests
  (pnpm workspace links,
  Cargo path dependencies),
  which do not express task-level ordering.
- meow's freshness model
  hashes declared reads
  and assumes `meow watch`
  keeps everything up to date,
  so unbuilt dependencies
  "do not arise";
  Pants runs on demand.
  The two models differ
  and the hop-2 plan must reconcile them.
- The name map must survive hop 1
  in the generated layer
  or be re-derived.

## Open items to settle at adoption

- The command surface
  (ranked above).
- The timeout default
  (proposal:
  900 seconds).
- Where the generator lives
  (proposal:
  `monorepo-task`,
  beside the existing task orchestration).
- Whether docs rewrite in wave 3
  or in a separate wave
  (proposal:
  wave 3,
  with `AGENTS.md` `CM3` and `CM5`
  updated in the same commit).
