# Translation repair history, first handover document: repository idioms and traps learned

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/fs-id` is now `package/fs-id/core`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

The lint-rule idioms,
test harness conventions and tooling traps learned while building the package,
including the build-before-test rule,
the misleading exit codes of background pipelines and the fixture rule against real-person data.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Repo idioms and traps learned this session

- micromark emits `footnoteReference` nodes only when a matching definition exists;
  an undefined `[^n]` survives as literal text.
  `scanGfmReferenceLiterals` in `footnote-graph.ts` turns exactly those literals into
  unresolved-reference findings.
- `prefer-readonly-parameter-types`:
  mdast-typed parameters need
  `ForeignBorrowed<...>` (from `@monochromatic-dev/ownership-marker-foreign-borrowed/ts`)
  at the ownership boundary;
  never repeat the marker on descendants.
  Error constructors passing `cause` to `super` need an
  `@mutates cause - ...getter or proxy trap...` TSDoc line (idiom from `package/module/fs-id/src/errors.ts`).
- `unicorn/custom-error-definition` wants literal `this.name = 'ClassName';` strings.
- `no-restricted-syntax/no-nullish-union`:
  model absence with optional properties plus conditional spread
  (`exactOptionalPropertyTypes`),
  never `T | undefined`.
- `eslint/init-declarations`:
  no `let` plus try-assign;
  extract a throwing helper returning the value.
- `prefer-describe-function-ref-name`:
  `describe` name must be `fnUnderTest.name`.
- Test harness:
  `await describe({ name, children: [it({ name, fn, },),], },)`
  from `@monochromatic-dev/module-test/ts`;
  matchers include `toBe`,
  `toEqual`,
  `toStrictEqual`,
  `toContain`,
  `toHaveLength`,
  `toThrow`.
- mise task wrappers swallow findings into inherited stdio:
  capture full output to a scratchpad file and `rg` it;
  tails alone mislead.
- **Ghostty windows cannot be launched while the screen is locked**,
  which is
  most of the time an overnight session runs.
  `ghostty -e <command>` starts a
  live process that never spawns the command:
  the pty child is created on the
  first GLArea resize,
  and a locked compositor never maps the window.
  There is
  no error anywhere.
  Full diagnosis,
  the positive control that found it,
  and the
  source citations:
  `doc/troubleshooting/ghostty-locked-session-no-command.md`.
  So an interactive sol review in a visible window is something to launch while
  the user is AWAKE;
  overnight,
  run `pi --print` into a file and read the file.
- **A background command's exit code reports its LAST stage,
  not the task's.**
  A verification pipeline ending in `... | rg 'FAIL' || echo 'no failures'`
  exits 0 whether the suite passed or the lint failed,
  because `rg` finding
  nothing and `echo` succeeding are what the shell reports.
  Measured rather than suspected:
  commit `e7f635e0d` was made on exactly that
  reading and carried THREE type errors
  (`artifact-build.ts` reading two fields its own parameter type did not
  declare,
  and a benchmark test stub missing them),
  found the next morning by
  opening the captured output instead of the notification.
  Read the captured file for the `Found N warnings and N errors` line and the
  suite's own FAIL count;
  never accept the wrapper's exit code as the verdict.
- **`test:unit` alone tests the PREVIOUS BUILD,
  and `lint:types` reads it too.**
  Every `*.unit.test.ts` here imports `../dist/final/node/index.mjs`,
  the built
  bundle,
  not the source beside it.
  `lint:types` DOES check the test files;
  it checks them against the DECLARATIONS OF THAT SAME STALE BUNDLE,
  which is
  the trap rather than an exemption.
  (An earlier version of this note said the type-check skipped test files
  entirely.
  It does not.
  Later evidence:
  adding an export and running
  `lint:types` before rebuilding reported
  `Module "../dist/final/node/index.mjs" has no exported member`,
  from a test
  file.
  The practical rule is unchanged;
  where you look when a test disagrees
  with the source in front of you is not.)
  So a green `test:unit` straight after a source edit is evidence about the
  build from before that edit,
  and a test calling a function without a newly required argument passes both
  tasks until the bundle is rebuilt.
  Always use `mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:buildAndTest`).
  Measured rather than suspected:
  two green `test:unit` runs were collected in the 2026-08-09 session before
  this was noticed,
  and neither had executed a line of the new code;
  the first `buildAndTest` after it failed immediately on a real assertion.
- Run `pnpm install` after every `package.json` dependency edit (TS2307 otherwise).
- Fixtures must never contain real-person data or recognizable source content;
  cat-themed invention mirroring structure only (user corrected this twice;
  treat as hard rule).
- Parser deps all from the pnpm catalog:
  `unified`,
  `remark-parse`,
  `remark-mdx`,
  `remark-gfm`,
  `yaml`,
  `@types/mdast`.
  `remark-math` is not in the catalog;
  math survives as text nodes (accepted milestone-one gap).
- `no-mixed-operators` idioms:
  parenthesize comparisons under `&&`/`||`,
  `(typeof value) !== 'string'`,
  and `index === (-1)` for indexOf non-existence
  (`unicorn/consistent-existence-index-check` simultaneously demands `=== -1`).
- `chain-per-line`:
  chains of two or more member steps plus a call split one step per
  line (`claim` / `.spans` / `.map(...)`);
  extract consts for chained conditions.
- `typescript/no-unsafe-type-assertion` bans narrowing casts:
  replace with `find` over the closed list plus `nonNullishOrThrow`.
- `isolatedDeclarations`:
  exported consts need explicit type annotations;
  `as const satisfies ...` alone fails TS9010.
