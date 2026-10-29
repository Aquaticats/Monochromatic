# Planning: preventing the "remove before replacement verified" mistake

Status:
 open.
 Picking back up another day.

## The problem

Previous session was extracting the logger out of `package/module/es` into a new
`package/logger/core`.
 Claude started modifying `module/es` before
`logger/core` was verified end-to-end and committed as a standalone addition.
When interrupted,
 Claude's reflex was to shim `module/es` rather than revert it.

User's framing:
 **basic git hygiene**.
 The correct order is

1. Land the replacement as an additive,
    verified,
    committed change.
2. Migrate consumers;
    each commit leaves the tree green.
3. Delete the old code only once no consumer references it.

Prose rules buried in `AGENTS.md` / `CLAUDE.md` (already ~50k tokens) are not reliable
enough to prevent this.
 We want a mechanical guardrail.

## Options considered and rejected

### Reject: prose addition to AGENTS.md

The previous session had access to existing commit guidelines and still made the
mistake.
 More prose in a long file will not change the outcome.

### Reject: block `git commit` on a red tree (PreToolUse on Bash)

User's objection:
 Claude can sidestep this by bundling the additive change,
 the
consumer migration,
 and the deletion into one giant edit sweep.
 Final tree is
green,
 commit passes,
 but there was never an additive-first checkpoint.

### Partially reject: PostToolUse `:lint` hook on Edit/Write

`AGENTS.md`:
78-79 already says:

> A PostToolUse hook for Edit/Write on `.ts` files will run the package-specific
> `lint:types` task automatically;
>  until that hook exists,
>  run
> `mise run //package/<path>:lint:types` manually after editing TypeScript.

Extended idea:
 run `:lint` (not just `:lint:types`;
 user asked for oxlint too) on
the edited package **plus all dependents** via pnpm workspace filter,
 so removing
logger from `module/es` immediately surfaces `ssg-test` typecheck errors.

**Real value:
** catches refactor regressions mid-session.
**Limitation:
** does not actually enforce commit sequencing.
 A careful edit sweep
that updates both ends simultaneously stays green the whole time,
 never trips the
hook,
 and still bundles into one commit.
 Same anti-pattern the commit-gate failed
to prevent.

So this is worth building,
 but must be scoped as "catch regressions" and not sold
as "enforce git hygiene.
"

### Not yet chosen: targeted `UserPromptSubmit` nudge

Small hook that prepends a one-line sequencing reminder when the user prompt
contains extract / split / move / rename / refactor keywords.
 Advisory,
 not
mechanical.
 Low complexity.
 Hits the exact situation where the mistake happens.

## Empirical findings (worth keeping)

### `pnpm` filter direction

Empirically verified in this repo (pnpm 11.0.0-rc.
1):

- `pnpm --filter "...{<pkg-path>}" list --depth -1 --parseable`:
   returns the
  package + **dependents** (who depends on it).
   Used for "what might break if I
  edit this package?
  ".
  - `...{package/module/es}` → 40 packages (module/es + 39 dependents).
  - `...{package/logger/core}` → 39 packages (because `module/es` already
    lists `@monochromatic-dev/logger-core` in its `package.json:50`,
     so
    logger transitively inherits all of module/es's dependents).
- The trailing-ellipsis form `{<pkg-path>}...` is the opposite direction
  (dependencies / upstream) and must not be used for this hook.

### `:lint` task timings

Repo deletes `.tsbuildinfo` on every `tsgo` invocation,
 so every measurement is
cold-run.
 No warm/cold variance to reason about.

- Single package (`module/es` or `ssg-test`):
   ~1.2s wall.
   `tsgo --build`
  (not stock `tsc`) is why it's this fast from cold.
- Full workspace `mise run lint`:
   ~12s for the TS portion,
   ~15s total
  including root-level dprint / stylelint / markdownlint.
   All 73 packages
  currently lint clean (exit 0,
   zero warnings or errors);
   no pre-existing
  false-positive noise to filter.

Running `:lint` on `module/es + 39 dependents` in parallel is well under the 12s
full-workspace number.
 Perf is not a blocker,
 and there's no hidden slow-path on
the first invocation of the session.

## Open question (unanswered)

The user was asked:
 which direction to take?

- Option A:
   build the PostToolUse hook,
   reframe as catch-regressions.
- Option B:
   build a targeted `UserPromptSubmit` nudge for extract/refactor
  keywords.
- Option C:
   both.
- Option D:
   drop it;
   treat the previous session as a one-off that self-corrected.

User paused to sleep before answering.
 Resume here.

## Critical files if we do implement

- `package/module/es/src/mise.post-edit-typecheck.ts`:
   new script
  (`AGENTS.md` requires `mise.<action>.ts` in this dir,
   no shell).
- `package/module/es/mise.toml`:
   new `post-edit-typecheck` task.
- `.claude/settings.local.json`:
   new entry under `hooks.PostToolUse`
  (around the existing `Bash`-matched entries near line 657).
- `AGENTS.md:78-79`:
   replace the "hook does not yet exist" paragraph.
- `CLAUDE.md`:
   regenerated by file-enforcer
  (`file-enforcer.config.ts:59-63`);
   never hand-edited.

## Gotchas to remember

- `AGENTS.md` is the source of truth;
   `CLAUDE.md` is generated.
   Edit `AGENTS.md` and
  re-run file-enforcer.
- `AGENTS.md`:
  78-79 originally scoped the hook to the edited package only.
   The
  dependents-extension is a design change,
   not just implementation:
   call this
  out when editing `AGENTS.md`.
- `pnpm --filter` ellipsis direction is easy to get wrong;
   keep the leading-
  ellipsis form documented in any comments.
- Claude Code hook schema field names (matcher shape,
   stdin format,
   blocking
  exit codes) have moved across releases;
   use the `update-config` skill when
  wiring the hook entry rather than copying an older example.
- `AGENTS.md` says "never skip hooks (--no-verify)" already,
   so any git-level
  enforcement we add later (tracked `githooks/` via `core.hooksPath`) inherits
  that protection.

## Plan artifact

The detailed plan file from the planning session lives at
`~/.claude/plans/in-the-previous-session-zazzy-cupcake.md`.
 Still accurate for
the hook-implementation details;
 just missing the final direction choice.
