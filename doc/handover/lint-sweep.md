# Lint sweep handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/doodle-widget` is now `package/doodle-widget/app`;
> `package/webapp-productivity/done-postcss` is now `package/done/variant-postcss`;
> `package/desktop-daemon/hall-monitor` is now `package/hall-monitor/daemon`;
> `package/webapp-productivity/done` is now `package/done/app`;
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`;
> `package/webapp-productivity/rss` is now `package/rss/app`;
> `package/dev-script/page-weight` is now `package/page-weight/cli`;
> `package/dev-script/vm-builder` is now `package/vm-builder/cli`;
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`;
> `package/build-tool/css` is now `package/css/cli`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/catalog-tighten` is now `package/dependency-catalog/cli-tighten`;
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `@monochromatic-dev/module-or-throw` is now `@monochromatic-dev/or-throw-core`;
> `package/module/kv-store` is now `package/kv-store/core`;
> `package/module/memoize` is now `package/kv-store/memoize`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/fy` is now `package/fy/cli`;
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/terminal-exec` is now `package/terminal-exec/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

STATUS:
 COMPLETE (2026-05-26).
 All 84 active packages pass `:lint` (oxlint + types) with zero warnings/errors,
 and the repo-wide `lint:dprint`,
 `lint:stylelint`,
 `lint:markdownlint` all pass.
 `module/es` remains excluded (still fails by design;
 being split in a later session).
 The cli-git guard was verified intact on a throwaway repo after its own lint fix.
 Remaining uncommitted working-tree changes are out-of-scope `mise run format` churn in `package-paused/*`,
 `package/module/es/*`,
 and root `oxlint-require-tsdoc.ts`,
 plus 11 harmless `dist/final` build-artifact files and `pnpm-lock.yaml`;
 left for the user to revert or handle (not reverted,
 to avoid clobbering paused/split-session work).
 The sections below are the historical resume record.
Readonly-specific allowlist and diagnostic-filter guidance is superseded by the project-owned semantic rule and must not
be reused.

Resume point for the workspace-wide lint sweep.
 A fresh agent should be able to continue from here with no prior conversation context.

## Goal

Fix all lint issues across active packages so `mise run lint` exits 0.
`denyWarnings` is on,
 so warnings count as failures.
`module/es` is EXCLUDED from this sweep (it is being split in a later session);
 leave its violations alone.

## What is already done (committed)

- Formatting:
   `chain-per-line` autofix and `mise run format` converged (style commits).
- Two new custom oxlint rules in `package/oxlint-plugin/no-restricted-syntax`,
   both enabled at `error` in `package/config/oxlint/src/rule/restriction.ts`:
    - `no-nullish-union`:
       bans `T | undefined` and `T | null` (commit 5954a771).
    - `no-optional-escape`:
       bans `| void`,
       `| never`,
       `| unknown`/`| any`,
       `| {}`,
       falsy literals (`| ""`,
       `` | `` ``,
       `| 0`,
       `| -1`,
       `| false`,
       gated on a non-literal union member),
       empty/optional/named-optional/rest-only tuples,
       `Partial<T>`,
       `Record<K, never>`,
       `Pick<T, never>`,
       optionality-adding mapped types (commit e2741a08).
- `AGENTS.md` rule forbidding preemptive cli-git bypass (commit 17f79a17).
   `CLAUDE.md` is gitignored and regenerated from `AGENTS.md` via `mise run sync:files`.
- Research doc `doc/research/optionality-enforcement.md` (commit 039ec6db).
- GitHub issue #213 filed for future improvement of the two rules (the syntax-only rules miss
runtime/binding-resolution escapes;
   those are tractable with more effort,
   not unbannable).
- All 12 shared `module/*` packages re-fixed under the final rules with proper patterns:
   kv-store (b76ec75f),
   toml-edit (a7e908d8),
   logger (60325d59),
   fs-path (c039ca94),
   hyperscript (843a9745),
   image-diff (2631fb84),
   dom (3826946b),
   or-throw (70888390),
   test (097c10c7),
   observable (49e30157),
   i18n-compose (58dab75a).
- Leaves wave L1:
   deps-cube (0764b13c),
   done-postcss (926ef363),
   messages-demo (b1d3be9a),
   auto-mode (e8c5ba80),
   inference-canary (ea4f2041).
- Leaves wave L2 (verified `:lint` exit 0):
   module/memoize (1665ed5a lint + 5d3e148c test migration to ABSENT contract),
   dev-script/watch-restart (5ff2776d + 1c6418e7 OVERSIZED sentinel re-export),
   pi/morph-compact (dcd5e344 + 592b6aaf).
   All used real Symbol/domain sentinels and justified external-boundary disables,
   no nullish dodges.

## Done: webapp-productivity/done (committed 9741947c) + reusable oxlint findings

Green and committed.
 The optionality/readonly sweep landed as `fix(done): resolve lint issues` (`9741947c`),
 mirroring the lint-clean sibling `package/webapp-productivity/done-postcss` (commit 926ef363,
 the gold blueprint).
 Patterns applied:
 domain types `T | null` -> `readonly foo?: T`;
 `TaskRow` kept `T | null` under a justified `@tursodatabase/database` boundary block-disable with null->absent
 conversion in `mapTask`;
 symbol sentinels (`TASK_NOT_FOUND`,
 `SETTING_ABSENT`,
 `INVALID`,
 `ARGUMENT_ABSENT`,
 `METADATA_UNSET`,
 `NO_TIMER`/`NO_ABORT`,
 `CHIP_NOT_FOUND`);
 `AutofillManager` class -> `createAutofillController` factory;
 readonly param sweeps;
 hoisted non-capturing functions;
 non-async click handler.

Three reusable findings came out of the two stubborn `prefer-readonly-parameter-types` warnings (the earlier "2
remaining" count was an undercount;
 see lesson below):

1. **h3 `app: H3` exposed a native-rule symbol-name mismatch.**
   h3's dist declared `class H3$1` and re-exported `type H3 = H3$1`,
   so the retired native allowlist could miss the surface name.
   Do not recreate that allowlist entry.
   The replacement uses exact declaration provenance and caller effects.

2. **`focusOutline({ offset?: CssValue })` exposed a native-rule nested-brand limitation.**
   The semantic replacement classifies the primitive and readonly brand constituents directly.
   A 2026-07-13 package run produced no `focusOutline` or `CssValue` diagnostic while preserving unrelated findings.
   See
   [oxlint-prefer-readonly-branded-nesting.md](../troubleshooting/oxlint-prefer-readonly-branded-nesting.md).

3. **Readonly diagnostics must remain authoritative.**
   Do not add output filtering for the replacement rule.
   Resolve exact external effects in the tested catalogue,
   propagate real ownership through `ForeignBorrowed` boundaries,
   or add an honest `@mutates` contract for caller-observable effects.
   Oxlint reports unnecessary disable directives itself.

Lessons for future sweeps:

- **Count from authoritative `lint:oxlint` output,
   not a prior handover claim.
  ** This package was handed off as "2 remaining" but had `chain-per-line` (9)
  and `tsdoc(multiline-blocks)` (1) warnings too,
   plus a third `prefer-readonly` site (`requireParam`,
   fixed by typing `event: H3Event`).
   `lint:oxlint` surfaces every rule,
   not just the one you are chasing;
   grep the full summary line.
- **`rg -rln '<pat>'` is `--replace=ln`,
   not recursive.
  ** It rewrites matches to `ln`/`n` in output (ripgrep recurses by default).
   Bit this sweep twice (`focusOutline` -> `n`,
   a bin-entry key -> `ln`).
   Use `rg -n`.

## Critical context: the optionality discipline

Agents repeatedly tried to dodge the bans by swapping one escape hatch for another (`| undefined` to `| null` to
tuple-as-Maybe to `''` sentinels).
 Do not do this.

Forbidden ways to model optional or absent,
 even though some are not lint-detectable:

- `T | undefined`,
   `T | null`,
   `T | void` (and `| never`/`| unknown`/`| any`/`| {}`).
- tuple or array as Maybe:
   `[]`,
   `[T] | []`,
   `[T?]`,
   returning `[value]`/`[]`,
   or a plain `T[]` whose length encodes presence.
- disguised-empty sentinels:
   `''`,
   `0`,
   `-1`,
   `NaN`,
   empty array or object used to mean absent.
   An empty string is NOT a sentinel.
- `Partial<T>` used to dodge required-ness.
- adding an `oxlint-disable` to turn the rule off,
   except for a genuine external-library boundary (for example mirroring `Map.get`
   or DOM `querySelector`) with a justification naming the external API.

Allowed ways (use these):

- `?:` optional property or field or param (under `exactOptionalPropertyTypes`,
   `foo?: T` already means absent or T).
- an `if`-guard,
   early return,
   or narrowing so the value is always present where it is typed.
- throw at the boundary with `nonNullishOrThrow` from `@monochromatic-dev/module-or-throw`.
- a real sentinel:
   a unique `Symbol` (for example `const ABSENT = Symbol('absent')`,
   narrowed by identity) or a distinct,
   meaningful,
   non-empty domain value.

kv-store contract note:
 `module/kv-store` exports `ABSENT` (a `unique symbol`) and `get()` returns `T | typeof ABSENT`.
 Consumers import `ABSENT` and narrow by `=== ABSENT`.

## Remaining work

Dispatch each leaf package to a `spawn-claude` child that fixes it to `mise run //package/<PATH>:lint` exit 0.
 Counts are warnings/errors as of the last enumeration (`/tmp/lint-all4.log`),
 so treat them as approximate;
 the child should re-check its own package.

Leaf packages still to fix:

- `webapp-productivity/done`:
   this historical package status predates the semantic replacement.
   A 2026-07-13 package run returned status `1` with 35 replacement-rule findings,
   but no `focusOutline` or `CssValue` finding.
   See the branded-nesting troubleshooting document for the resolved native-rule false positive.
- `pi/advisor` (33e)
- `oxlint-plugin/tsdoc` (28w 37e):
   a config package;
   child was launched but PAUSED by the user.
   The "no config edits while `webapp-productivity/done` is uncommitted" constraint is now LIFTED (done is committed);
   resume only with zero leaf children running.
- `webapp-productivity/doodle-widget` (45w 16e)
- `dev-script/page-weight` (20w 12e)
- `pi/terminal-title` (23w 3e)
- `typeface/aquaticat` (18w 1e)
- `pi/thinking-defaults` (15w 5e)
- `webapp-productivity/rss` (19w 2e)
- `pi/current-time-context` (6w 4e)
- `cli/mvm` (14e),
   `cli/terminal-exec` (13e),
   `cli/vmsync` (11e),
   `cli/git` (8e),
   `cli/fy` (3e)
- `mcp/stdio` (12w 6e),
   `mcp/mvm` (2w)
- `oxlint-plugin/stylistic` (12e),
   `oxlint-plugin/no-restricted-syntax` (5w,
   pre-existing require-unicode-regexp in `prefer-describe-function-ref-name.ts`)
- `dev-script/file-enforcer` (10e),
   `dev-script/task-util` (11w 6e),
   `dev-script/catalog-tighten` (4w 6e),
   `dev-script/vm-builder` (7w)
- `rolldown-plugin/import-attributes` (8e),
   `build-tool/css` (4e)
- `desktop-daemon/hall-monitor` (7w 6e)
- `claude-code-plugin/hook-type` (3e),
   `claude-code-plugin/source` (3w 24e)
- `dev-script/inference-canary-viewer` (2e),
   `ssg/aquati.cat` (4e)
- `module/memoize` (2e):
   import `ABSENT` from kv-store and narrow by identity;
   it broke when kv-store stopped returning `undefined`.

Open behavioral decision discovered during the sweep (NOT a lint issue):

- `module/memoize` lint is green (commit 1665ed5a),
   but the kv-store -> `ABSENT` migration (commit b76ec75f) changed runtime behavior:
   a memoized function returning `undefined` now round-trips through kv-store and gets cached,
   instead of being treated as a miss and recomputed.
   This breaks 4 unit tests (`memoize.unit.test.ts:180,207,244`,
   `memoize-async.unit.test.ts:195`) which assert the old "undefined is a miss" contract.
   Whether memoize should now cache `undefined` (arguably more correct)
   or keep treating it as a miss is a product decision for the user,
   so the tests are left untouched.
   Resolve the semantics,
   then update the 4 tests accordingly.

Numeric / non-`''` sentinel audit candidate (out of scope,
 noted for the user):

- The optionality ban also covers `0`,
   `-1`,
   `NaN`,
   empty array/object used to mean "absent",
   but those are NOT lint-detectable and were NOT swept (this sweep's manual pass targeted `''` specifically,
   per the user's explicit ruling).
   The `inference-canary-viewer` child flagged surviving `?? 0` numeric defaults (e.g. score-display defaults,
   `build.ts`'s `ModelSummary.threshold ?? 0`) where `0` may be encoding "absent threshold" rather than a real value.
   A future pass could audit `?? 0` / `=== 0` / `-1` / `=== ''`-style absence encodings across packages the same way
   the `''` rework did.
   Distinguish a genuine default (0 is a valid value) from a disguised sentinel (0 means missing) by reading each
   consumer.

Post-sweep consolidation candidate (out of scope for lint-zero,
 noted for the user):

- The `Maybe<T> = T | typeof ABSENT` alias (with a local `const ABSENT: unique symbol = Symbol('absent')`,
   narrowed by `=== ABSENT`) has been independently recreated as a per-package `src/maybe.ts` in at least `deps-cube`,
   `pi/advisor`,
   `cli/terminal-exec`,
   and `dev-script/page-weight`.
   Each copy is correct and matches the sanctioned sentinel pattern (NOT a banned Option/Maybe monad:
   it is a bare union with a unique symbol,
   no boxing).
   The duplication is a DRY candidate:
   a shared `@monochromatic-dev/module-maybe` (or extending `module-or-throw`/reusing kv-store's exported `ABSENT`)
   would consolidate them.
   Deferred because it is a refactor,
   not a lint fix.

Manual rework that no lint rule catches (the type annotation is honest):

- `ssg/aquati.cat` and `dev-script/inference-canary-viewer` used `''` as an absence sentinel in earlier passes.
   The user ruled that `''` is not a valid sentinel.
   Hunt these by reading the files (grep for `''` defaults and returns in slots that mean absent) and convert to `?:`,
   an `if`-guard,
   or a real Symbol or domain sentinel.

After all leaves:
 rebuild (`mise run build`),
 then run the full `mise run lint` and confirm exit 0 across active packages (excluding `module/es`).

## How to resume (mechanics)

- Spawn children with `spawn-claude --cwd /var/home/user/Monochromatic "<prompt>"`.
   It prints `{"spawnId":"<uuid>"}`.
   Results are forwarded back automatically;
   poll `~/.claude/spawn-results/spawns/<id>.json` for `status`,
   or watch `git log` for the child's commit.
- PROMPT ESCAPING:
   the prompt is a double-quoted bash argument,
   so any backtick inside it is command substitution and gets EVALUATED,
   silently deleting that span from the prompt (e.g. an example like a backtick-wrapped code snippet runs as a shell
   command and vanishes).
   Do NOT put backticks in the prompt.
   Use single quotes for code examples,
   or single-quote the whole prompt argument.
   Bit this sweep on the two `''`-rework prompts;
   verify the child's work independently when the prompt may have been degraded.
- The child prompt template is at `/tmp/lint-rules.txt`.
   If that file is gone,
   reconstruct it from the forbidden/allowed lists above plus:
   each child fixes ONE package (or a small named batch) to lint-zero,
   commits with a SCOPED pathspec (`git add package/<PATH>` then `git commit package/<PATH> -m ...`),
   uses NO `--no-enforce-bulk-add` or `--no-enforce-only` flags,
   and reports tersely.
- NEVER run a `config/oxlint*` package (`oxlint`,
   `oxlint-tsdoc`,
   `oxlint-stylistic`,
   `oxlint-no-restricted-syntax`) concurrently with any leaf child.
   Those packages DEFINE the rules every other child lints against;
   editing them mid-flight shifts the ground under every running child and produces unstable,
   non-reproducible results.
   Fix the config packages in their own isolated wave,
   one at a time,
   only when zero leaf children are running.
- Order:
   polish the `config/oxlint*` packages EARLY,
   before launching further leaf waves,
   so later leaves lint against settled rules.
   The user prefers the oxlint config stable first.
   (The in-flight leaves already running on the current ground are allowed to finish first;
   the rule is "no NEW leaf wave until the config packages are done.
  ") Config packages are an early isolated serial phase,
   not the final one.
- Use a Monitor (background/until-loop) to wait on child completion,
   not a foreground polling loop in the main session:
   repeated poll output pollutes the parent context.
   Watch a condition (new `fix(`/`test(` commits,
   or the child's PID exiting, via `timeout 3600 tail --pid=<pid> --follow /dev/null`)
   and only surface the result.
   Never loop on `pgrep` by name: the match includes the waiting shell itself.
- Concurrency is NOT capped at a small number by resources.
   This machine has 16 cores (`os.availableParallelism()` == 16) and runs 16 sessions fine.
   Scale child concurrency toward `os.availableParallelism()`,
   not a guessed ceiling;
   check it rather than assuming.
- Empirical burst-size finding (2026-05-25 sweep),
   CORRECTED:
   a wave of 9 children launched in one burst on top of 7 existing terminals (16 total) appeared stalled/husked at 46
   minutes (spawn-result JSON frozen at `status:running`,
   ~5 with no claude process,
   ~4 alive but idle,
   zero source edits,
   zero commits).
   I concluded they were dead and escalated.
   That conclusion was WRONG:
   all 9 recovered and committed green over the next several hours with no intervention.
   Lesson:
   these children can stall for a very long time (hours) under a large simultaneous burst and still recover;
   46 minutes of no progress is NOT proof of death.
   Do not kill or escalate on a stall alone.
   Still,
   large bursts are slow and risky,
   so the practical rule stands:
   keep spawn BURSTS small (about 5 at a time),
   re-check live count between bursts,
   and be patient rather than declaring children dead.
- There was one incident where new `ghostty -e claude` sessions hung at startup (Claude exited before initializing;
   the terminal became an empty husk) with around 12 lingering terminals.
   Do NOT treat this as a reproducible resource limit:
   it happened once and did not recur.
   It is debuggable in principle (Claude Code is closed-source but its compiled JS can be extracted and read),
   just not worth that effort for a non-recurring one-off.
   If new children stop producing live sessions,
   that single failure mode is a candidate,
   and the user must close stale terminal windows (killing them needs explicit user authorization;
   the auto-mode classifier blocks mass-kills),
   but it is an unexplained one-off,
   not a cap to design around.
- Do NOT trust the `status` field in `~/.claude/spawn-results/spawns/<id>.json` to find live children:
   it does not reset to `stopped`/`reported` on abrupt termination,
   so hundreds of stale files read `"running"`.
   For an actual live count use `find ~/.claude/spawn-results/spawns/ -name '*.json' -mmin -30` plus
   `pgrep -af ghostty`,
   not the JSON status.
- `cli/git` implements the cli-git enforcement guard itself.
   Fix it LAST (or,
   right after its commit lands,
   confirm the guard still rejects `git add -A` in a throwaway `git init` repo before launching more children);
   a regression there breaks every later child's scoped commit and stalls the sweep.
- Do NOT fix shared oxlint config packages concurrently with anything else.
   `oxlint-plugin/tsdoc`,
   `oxlint-plugin/stylistic`,
   `oxlint-plugin/no-restricted-syntax`,
   and `config/oxlint` define the rules every other package lints against;
   editing them mid-flight shifts the lint baseline under every concurrent child (they end up working on unstable
   ground).
   Fix these SERIALLY,
   after all consumer packages are green,
   one at a time,
   re-running a full fanout lint between each.
   Treat them like `cli/git`:
   a final,
   isolated,
   single-child phase.
- Other recurring lint categories and their fixes:
   `prefer-readonly-parameter-types` (mark params readonly,
   find the structural root in shared types;
   if `TS2540` "cannot assign to read-only property" appears,
   use a mutable internal type with a readonly public view rather than widening);
   `require-unicode-regexp` (add `u` to the existing guarded regex);
   `max-lines` (split the file into siblings,
   never raise the limit);
   `no-class` (prefer factory closures);
   `consistent-function-scoping` (hoist non-capturing inner functions).
- `claude-code-plugin/*/dist/*.mjs` showing as modified is harmless build-artifact churn from rebuilds.
   Leave it uncommitted for the user;
   do not commit hook infrastructure during the autonomous sweep.

## References

- Optionality rules:
   `package/oxlint-plugin/no-restricted-syntax/src/rule/no-nullish-union.ts`,
   `no-optional-escape.ts`.
- Research:
   `doc/research/optionality-enforcement.md`.
- Future rule work:
   GitHub issue #213.
- The development guidelines in `AGENTS.md` govern everything (commit eagerly per logical unit,
   scoped commits,
   no rule loosening,
   log extensively,
   comprehensive TSDoc).
