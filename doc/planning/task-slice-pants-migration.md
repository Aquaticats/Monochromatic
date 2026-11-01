# Task-slice migration to Pants

Status:
 proposal,
 written 2026-10-29 and revised the same day after two corrections.
No `doc/decision/` record yet,
 no task file changed.

## Corrections applied 2026-10-29

Two corrections from the user
 re-derived the decisions below.

- "We don't care about how much work hop 2 would take."
 Hop 2's cost is not a design input.
 Every decision that was justified
 by making hop 2 cheap is re-derived
 on capability for the target workload alone.
- "`task-run` (task CLI) is an extremely ugly hack we did because Mise was bugging out."
 `@monochromatic-dev/monorepo-task-cli`
 and its `task.ts` mirrors
 are a stopgap,
 not an owner to extend.
 The plan retires them
 instead of building on them.

## Scope

Two hops.

- Hop 1:
  Mise to Pants 2.33.1,
  the task-execution slice,
  now.
- Hop 2:
  Pants to meow,
  later,
  when capacity allows.

Hop 2 still happens.
Its cost simply does not shape hop 1.

## What the target workload needs

### Addressing

Keep `//package/<path>:<task>`.

This is not incumbent fidelity
 and not a hedge for hop 2.
It is the target's own grammar:
 a Pants spec is `path:target_name`
 with an optional `//` prefix
 that `src/rust/engine/src/externs/address.rs` strips.
After the name change below,
 `//package/cow:build.js.node`
 is a literal Pants spec.

Measured call-site surface today
 (2026-10-29,
 `rg --hidden`,
 excluding worktrees and `node_modules`):

- 2,422 `mise run //...` matches;
- 81 `task-run //...` matches,
  all in CI workflows and handover notes.

Root tasks are bare names today
 (`lint:monochromatic-lint`,
 `prepare:pnpm`,
 `changeset:*`).
They become root-level specs
 (`//:lint.monochromatic-lint`).

### Task names

Rename the task-name separator from `:` to `.`
 at the source.
One vocabulary,
 no map,
 no translation layer.

`BANNED_CHARS_IN_TARGET_NAME`
 (`src/rust/engine/src/externs/address.rs:73`)
 bans `: # ! @ ? / \ =`,
 so the colon form is not addressable
 in Pants at all.

Measured 2026-10-29 over `mise.toml` and `mise.no-env.toml`
 outside worktrees:
 508 distinct task names,
 408 of which contain a colon,
 0 of which contain a dot,
 and `:` to `.` collides on no pair.
The rename is therefore injective.

Why rename rather than mangle behind a map:

- criterion `G`,
  machine-readable listing,
  reports one vocabulary.
  With a map,
  `peek` prints `build.js.node`
  while humans and call sites say `build:js:node`,
  and every consumer needs the map.
- criterion `I`,
  maintenance depth,
  drops a permanent translation artifact.

The hierarchy survives.
`build.js` fans out over `build.js.*`
 exactly as `build:js` fans out over `build:js:*` today,
 so umbrella fanout
 and the `category[--modifier].child` grammar
 are unchanged apart from the separator.

The naming convention at `mise.toml:624-625`
 (`${category}${optional(`--${modifier}`)}:${child}`)
 is documented in the incumbent's own config
 and retires with it.

This is the largest single change in the plan
 (408 names,
 2,422 call sites,
 `AGENTS.md` `CM5`).
It is recorded here as decided.
Veto is cheap now
 and expensive after wave 3.

### Task bodies

- `adhoc_tool` argv lists,
  one entry per argv element,
  no shell strings.
- Use a local-workspace environment target
  (`LocalWorkspaceCompatiblePlatformsField`)
  so the process is a pure exec.
  Without it `prepare_adhoc_process` wraps argv
  in `bash -c "cd <workdir> && <shlex.join(argv)>"`,
  which is safe against injection
  but puts a `bash` in the tree.
- `runnable_dependencies` always generate bash shims;
  keep them out of the slice
  unless a body truly needs them.
- Intentional failure tolerance
  stays expressible in the body.
  `adhoc_tool` has no `allowFailure` field,
  so a tolerant task exits 0 from its own body
  rather than being marked tolerant by the runner.

### The acceptance boundary

`package/monorepo-task/core-exec/src/build-and-test.unit.test.ts`
 is the gate.
It checks that tests still run after build failure,
 that either phase failing makes the aggregate fail,
 and that explicitly tolerant tasks remain tolerant.

It currently drives generated `mise.toml`
 through real Mise subprocesses.
Under hop 1 its subject changes
 to the generated Pants layer
 and the dispatcher,
 and its assertions stay.

## What retires with Mise

### `task-run` and `task.ts`

`@monochromatic-dev/monorepo-task-cli`
 (`package/monorepo-task/cli`,
 the `task-run` bin)
 is a stopgap built because Mise was misbehaving,
 per the user on 2026-10-29.
It resolves `//package/<path>:<task>`,
 honours `depends`,
 `extends`,
 `usage_*` and umbrella fanout
 over 21 `task.ts` files
 that mirror each package's `mise.toml`.

None of that is a platform to extend.
The 21 `task.ts` files are a derived mirror
 of the `mise.toml` corpus
 and retire with the CLI.

Consequence:
 `doc/decision/mise-free-ci.md`
 (accepted 2026-10-25,
 in force)
 is superseded on the runner half
 once the Pants layer serves CI.
Its other half,
 that no workflow installs or invokes Mise,
 stands unchanged.
That supersession is a `doc/decision/` edit
 and waits for explicit acceptance.

### `monorepo-task/core-exec`

`command.ts` (`task-command --allowFailure`)
 and the `depends-*.ts` make-style checker
 exist for the same reason,
 to paper over Mise limitations.
`allowFailure` is a real duty
 ("intentional failure tolerance"),
 so that behavior survives,
 but as a body concern rather than a runner shim.
The `depends` staleness machinery
 is superseded by Pants caching.

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

## Source of truth

The `mise.toml` corpus
 (root `mise.no-env.toml` plus the package `mise.toml` files)
 stays the source of truth through hop 1.
`task.ts` is derived and retires.
The Pants layer is generated.

Reason on target-workload grounds:
 `meow`'s task model is still unsettled
 ("We need to settle on 'what is a task' first too",
 and task ordering is explicitly undecided),
 so `meow.hcl` cannot be the source yet.
Nothing here is chosen to spare hop 2.

## Target generation

One generator,
 extending the pipeline that already emits task config:
 `file-enforcer.config.ts:691` `generateMiseToml`
 already generates root `mise.toml`.
That is the nearest real owner of task-config emission,
 and extending it is what `RCI` asks.

The stopgap CLI is not a candidate,
 per the correction above.

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
- apply the `:` to `.` name change;
- emit `dependencies`
  (or `execution_dependencies` where order is required)
  from the `depends` lists;
- emit `timeout`,
  `environment`,
  and `output` fields
  from the body shape.

## The four carry-in items

These are what adoption must build,
 because upstream does not provide them.

- Summary line for the dependents of a failed build.
  Compute the blocked set from `peek`
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
  and derive the aggregate exit outside the engine.
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

The first two need a thin repo-owned reporter
 that runs with the job.
The last two are generator and config work.

## Command surface

The old ranking
 (`meow`-shaped stub first)
 rested entirely
 on saving a hop-2 token change,
 so it is void.

Re-derived:
 the label is already a Pants spec,
 so a dispatcher adds nothing to addressing.
Its only remaining jobs
 are the two reporting duties
 (dependents summary,
 aggregate exit),
 which need a thin runner in the loop
 either way.

1.  A thin `task` dispatcher
    that resolves the spec,
    invokes `pants`,
    prints the dependents summary,
    and derives the exit.
    One surface for humans and CI.
    Con:
     one more binary.
2.  `pants` goals directly,
    with the reporter as a separate step
    on the root aggregates and in CI.
    Fewest moving parts.
    Con:
     call sites that need the summary
     must remember the reporter.
3.  A `meow`-shaped stub.
    Con:
     its only merit was hop-2 token stability,
     which is now not a design input,
     and it names a binary after a tool it is not.

Ranking 1 > 2 > 3,
 because the two reporting duties
 are wanted on every call
 and a dispatcher is the only shape
 that guarantees them;
 2 leaves that to caller discipline;
 3 has no remaining justification.

## Rollout

Each wave is gated on the acceptance boundary
 and on a dual-run diff.

- Wave 0:
   re-measure the corpus,
  build the generator,
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
   rewrite call sites to the new names,
  repoint CI from `task-run` to the dispatcher,
  retire `task.ts` and the `task-run` bin,
  retire `mise run`.

Docs (`AGENTS.md` `CM3`,
 `CM5`,
 and the 2,422 mentions)
 follow wave 3,
 not before.

## Reversal

Rollback is deletion.
The generated `BUILD` files are disposable,
 the `mise.toml` corpus stays authoritative
 until wave 3 completes,
 and reverting is deleting the generated files
 plus restoring the call sites.

Nothing here is designed around hop 2.
When meow takes the slice,
 it replaces the dispatcher,
 reads the same corpus,
 and the call sites keep their labels.

## Revisit triggers

- Capacity for the meow hop.
- `meow`'s task model settled
  (what is a task,
  what creates ordering).
- `meow`'s label grammar settled
  for a dot inside a task name.

## Open items to settle at adoption

- The timeout default
  (proposal:
  900 seconds).
- Whether the generator extends `file-enforcer.config.ts`
  or gets its own package
  (proposal:
  extend file-enforcer,
  which already emits task config).
- Whether the dispatcher's reporting
  is worth its own binary
  or is a root-aggregate and CI-only step
  (ranking above).
- Whether docs rewrite in wave 3
  or in a separate wave
  (proposal:
  wave 3,
  with `AGENTS.md` `CM3` and `CM5`
  updated in the same commit).
