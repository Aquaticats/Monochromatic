# TypeScript constant-loop ban handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/css-edit` is now `package/css/core-edit`.

Last updated:
`2026-07-22T22:06:35-04:00`.

Current repository HEAD when this file was created:
`2c0d0c52ff02f8e700673ecf40128d8e56029527`.

## User requirements

- Ban TypeScript `while (true)` through option 1 from the policy review.
- Option 1 is Oxlint's built-in `eslint/no-constant-condition` rule with
  `checkLoops: 'all'`.
- Do not add a literal-only custom `while (true)` rule.
- Create this handover now and update it from time to time during implementation.

## Settled design

Configure the shared rule in
`package/config/oxlint/src/rule/correctness.ts`:

```ts
// package/config/oxlint/src/rule/correctness.ts
'eslint/no-constant-condition': [
  'error',
  { checkLoops: 'all', },
],
```

Use explicit continuation state or bounds for ordinary loops:

- Token scans use array bounds and preserve EOF or absent-token behavior after the loop.
- Ancestor-directory scans use an inclusive ancestor iterator that yields the filesystem root.
- AST cursor walks use node-shape or parent-boundary conditions.
- Pagination records whether another page exists.
- Polling records the current completion state or uses the real deadline where semantics permit.
- Storage retries record whether another owned entry permits a retry.
- Stream reads record the current `done` state or use async iteration when behavior is equivalent.

The two scripts whose purpose is deliberate nontermination or exhaustion may use `for (;;)`,
which `no-constant-condition` intentionally accepts:

- `package/runtime-error/bun/src/infinite-loop.ts`
- `package/runtime-error/bun/src/oom.ts`

The existing production `for (;;)` at
`package/ssg/aquati.cat/src/lib/content.ts:110`
is a known gap in option 1.
Do not expand this task into a custom conditionless-`for` rule unless the user separately requests it.

## Verified Oxlint behavior

Oxlint version under test:
`1.74.0`.

The installed source and a disposable Mise-run probe established:

- Default `checkLoops: 'allExceptWhileTrue'` accepts `while (true)`.
- `checkLoops: 'all'` rejects `while (true)`.
- `checkLoops: 'all'` rejects `for (; true;)`.
- `checkLoops: 'all'` rejects `do {} while (true)`.
- `checkLoops: 'all'` rejects `while (1)`.
- `checkLoops: 'all'` accepts `for (;;)`.
- Dynamic conditions such as `while (keepGoing())` remain accepted.

Durable source trace and reproduction:
`doc/troubleshooting/oxlint-no-constant-condition-loop-options.md`.

That troubleshooting document passes:

```bash
mise run lint:markdown -- doc/troubleshooting/oxlint-no-constant-condition-loop-options.md
```

## Current inventory

The initial complete scan found 25 lint-scoped executable `while (true)` statements.
All 25 lint-scoped executable statements are migrated.
The final text inventory returns no matches.

### Parser and analysis loops

All 14 parser and analysis occurrences are migrated.
No `while (true)` remains in:

- `package/module/css-edit/src/`.
- `package/oxlint-plugin/tsdoc/src/`.
- `package/oxlint-plugin/prefer-readonly-parameter-type/src/`.

### Retry, polling, stream, and fixture loops

All 11 retry,
polling,
stream,
and fixture occurrences are migrated:

- Logger storage retries use owned-entry attempt bounds.
- MVM pagination and polling use explicit continuation state.
- File-enforcer lock acquisition uses handle absence as its boundary.
- Tofu stream consumption uses `ReadableStreamReadResult.done`.
- Runtime-error fixtures alone preserve deliberate infinity as `for (;;)`.

Re-run the inventory after every migration group:

```bash
rg --line-number --multiline \
  --glob '*.ts' \
  --glob '*.tsx' \
  --glob '!package-paused/**' \
  --glob '!package-deprecated/**' \
  --glob '!**/test-fixture/**' \
  --glob '!**/perf-test-data/**' \
  --glob '!**/*.generated.ts' \
  'while\s*\(\s*true\s*\)' \
  .
```

## Concurrent worktree state

At handover creation,
these unrelated changes existed and must not be staged,
restored,
or rewritten:

- Modified:
  `doc/handover/css-edit.md`.
- Untracked under
  `package/oxlint-plugin/prefer-readonly-parameter-type/src/`:
  - `csstools-css-tokenizer-package-effect-catalog.unit.test.ts`.
  - `prefer-readonly-parameter-types/csstools-css-tokenizer-package-effect-catalog.ts`.

The original concurrent files were later committed by their owning work.
Latest unrelated state includes fuzz coverage work,
a root `mise.toml` edit,
and a Done database WAL.
Re-run `git status --short` before every scoped commit,
and stage only explicit paths owned by this task.

## Task state

- Task 6,
   enable constant-loop linting:
  completed.
- Task 7,
   migrate parser and analysis loops:
  completed.
- Task 8,
   migrate retry and polling loops:
  completed.
- Task 9,
   verify constant-loop ban:
  completed.
- Task 10,
   create this handover:
  completed.
- Task 11,
   clear concurrent CSS test blocker:
  completed after source migration settled.
- Task 12,
   resolve PostCSS catalog test drift:
  completed by owning commit `e2cda4e35`.

Only one task may be actively implemented at a time.
Update this section whenever a task changes state.

## Verification plan

For every changed TypeScript package:

- Run its focused unit tests when a `test:unit` task exists.
- Run `mise run //package/<path>:lint:oxlint`.
- Run `mise run //package/<path>:lint:types` manually.

Required package tasks include:

- `//package/config/oxlint:lint:oxlint`
- `//package/config/oxlint:lint:types`
- `//package/module/css-edit:test:unit`
- `//package/module/css-edit:lint:oxlint`
- `//package/module/css-edit:lint:types`
- `//package/oxlint-plugin/prefer-readonly-parameter-type:test:unit`
- `//package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint`
- `//package/oxlint-plugin/prefer-readonly-parameter-type:lint:types`
- `//package/oxlint-plugin/tsdoc:test:unit`
- `//package/oxlint-plugin/tsdoc:lint:oxlint`
- `//package/oxlint-plugin/tsdoc:lint:types`
- package-equivalent lint and test tasks for logger,
  mvm,
  file-enforcer,
  tofu,
  and runtime-error.

Final boundary checks:

- The lint-scoped inventory command returns no `while (true)` matches.
- A disposable Oxlint fixture proves `while (true)` fails under the shared config.
- The two runtime-error scripts still express deliberate unbounded execution as `for (;;)`.
- No unrelated worktree paths appear in task commits.

## Commits and evidence

Research and troubleshooting commits already on `main`:

- `eb0e00071`,
   initial Oxlint behavior document.
- `1be1e84c5`,
   semantic line-break fixes.
- `0566088cf`,
   reproducible probe and inventory.
- `9ee37aa29`,
   pinned source and compact inventory.
- `ba84ca9e8`,
   initial constant-loop migration handover.
- `397b24a44`,
   handover Markdown formatting.
- `f4c02781b`,
   final handover creation state.
- `0abe1ea7d`,
   shared Oxlint `checkLoops: 'all'` configuration.
- `f901fe2de` and `4cbf886f1`,
   bounded CSS parser loops and narrowed trivia cursors.
- `8aaff507e` and `0c8d0a367`,
   root-inclusive ancestor traversal plus explicit AST conditions.
- `fc6535276`,
  `7af601fbe`,
  and `dd30f430d`,
   explicit TSDoc wrapper traversal.
- `aeba92bdf`,
  `209d13058`,
  and `b870b4576`,
   bounded logger storage retries.
- `9dfd631d6`,
  `654a7139c`,
  `9fc9a8b54`,
  and `ded751f51`,
   explicit MVM pagination and polling state.
- `e7fab0f6e`,
   multi-page traversal regression coverage.
- `8b59983fc`,
   explicit file-enforcer lock acquisition state.
- `edd2f0273`,
   explicit Tofu stream completion state.
- `3913ec8a6`,
   deliberate runtime-error `for (;;)` fixtures.
- `a8f6354bf`,
   executable-only final text inventory.
- `fc1b39e65`,
   shared guest-exec status polling boundary.
- `d8c241786`,
  `793581d37`,
  and `725133ac4`,
   first-attempt,
   retry,
   timeout,
   and completion polling tests.
- `08397ba5e` and `dc024f240`,
   root-inclusive ancestor traversal tests.
- `6e5cfe99d` and `7e0b0c4f4`,
   package-scoped logger unit task using existing storage retry suites.

Config verification passed:

```bash
mise run //package/config/oxlint:build
mise run //package/config/oxlint:lint:oxlint
mise run //package/config/oxlint:lint:types
```

Parser verification passed for `module-css-edit` and `oxlint-plugin-tsdoc`:
unit tests,
Oxlint,
and TypeScript lint are green.

`oxlint-plugin-prefer-readonly-parameter-type` build,
Oxlint,
and TypeScript lint are green.
Its full unit suite passed after owning commit `e2cda4e35` retired stale PostCSS catalog tests.

Retry and polling verification passed:

- Logger build,
  unit tests covering reactive eviction,
  terminal give-up,
  and non-quota failures,
  Oxlint,
  and TypeScript lint.
- MVM build,
  unit tests including fake-virsh first-attempt,
  retry,
  timeout,
  and completion paths,
  Oxlint,
  and TypeScript lint.
- File-enforcer build,
  unit tests,
  Oxlint,
  and TypeScript lint.
- Tofu local tests,
  Oxlint,
  and TypeScript lint.
- Runtime-error Oxlint and TypeScript lint.
  Dangerous fixtures were not executed.

Final verification passed:

- The lint-scoped `while`-true inventory returns zero matches.
- Config build,
  Oxlint,
  and TypeScript lint pass.
- A disposable package-consumer fixture exits with status 1 and reports
  `eslint(no-constant-condition): Unexpected constant condition`.
- Ordinary migrated sources contain no `for (;;)` statements.
- The two runtime-error fixtures use `for (;;)` as approved.
- Root-inclusive ancestor traversal now has focused start-to-root and root-only tests.
- The pre-existing production statement at `package/ssg/aquati.cat/src/lib/content.ts` remains unchanged
  and out of scope.
- Scoped commit path inspection contains only task files.
- An optional repository-wide Oxlint sweep was also attempted.
  It failed on unrelated baseline input with 697 errors and 3,870 warnings,
  including unsafe JavaScript benchmark diagnostics,
  plus TypeScript-Go tuple-type panics emitted through `prefer-readonly-parameter-types`.
  This does not replace or invalidate the passing package-scoped acceptance checks.

## Next action

No implementation remains.
Keep the known conditionless `for` policy gap out of scope unless the user requests a separate rule.
