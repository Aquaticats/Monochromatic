# Translation repair history 2026-08-05 to 2026-08-07: the branch rebased onto main, and a lint rule left alone

The rebase onto main and what it changed in building and checking,
and a lint rule left alone by the owner's decision.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The branch rebased onto main, and a lint rule left alone

- BRANCH REBASED onto main (main was 1228 commits ahead;
  276 branch commits
  replayed).
  Conflict surface was five files.
  `pnpm-lock.yaml` was never
  hand-resolved (LFW):
  upstream taken at each conflict and the lockfile
  regenerated afterwards by `//:prepare:pnpm:install`.
  `git-policy/cli/src/index.ts`:
  main restructured it to a barrel;
  resolution
  keeps the barrel PLUS this branch's `resolveGit` export.
  `forbidden-strings.append.txt`:
  main's restructured version kept,
  with this
  branch's unique retired-benchmark-module rename guard re-appended.
  The
  retired name is deliberately not spelled here:
  the guard exists to keep it
  out of the tree,
  and writing it in prose about the guard makes the document
  its own violation.
  `mise.toml` conflicted in the AUTOSTASH,
  not in a commit,
  and the stashed copy
  was STALE generated output that would have deleted PATH entries for packages
  main added.
  Taken from the rebased tree;
  the original is preserved in
  `stash@{0}` if it is ever wanted.
  BUILD BREAKAGE the rebase surfaced:
  main repointed `git-policy-cli/ts` at
  `authoring.ts`,
  which does not export `resolveGit`,
  so the build failed with
  MISSING_EXPORT.
  Per user decision the export was NOT added to `authoring.ts`.
  The first fix,
  importing the bare package specifier,
  was wrong and broke every
  test in the package:
  that specifier resolves to `dist/final/node/index.mjs`,
  which is also the `bin` entry,
  so it is the whole policy CLI.
  Neutral builds
  bundle workspace deps inline by design (`NEUTRAL_ALWAYS_BUNDLE`),
  so the CLI
  trust validator and its `yuku-parser` NATIVE BINDING landed inside the
  translation-repair artifact,
  and all 52 test files died on
  `Cannot find module @yuku-parser/binding-linux-x64-gnu/yuku-parser.node`
  because pnpm only links that binding inside `yuku-parser`'s own store dir.
  It also violated AGENTS.md ST3,
  which requires cross-package imports to
  resolve to TypeScript SOURCE.
  Fixed in `f48fde57c` by giving `git-policy-cli` the `"./ts/*": "./src/*"`
  wildcard that 58 other packages already have,
  and importing
  `@monochromatic-dev/git-policy-cli/ts/resolve-git.ts`.
  `resolve-git.ts` pulls
  in two node builtins and two small workspace modules;
  the artifact dropped
  from 651 kB to 167 kB.
  TRAP FOR NEXT TIME,
  and the first version of this note got it WRONG.
  It said
  `lint:types` does not cover the unit tests.
  It does.
  What actually happened is
  that the tests type-check against `dist`,
  and `dist` was STALE:
  it still
  carried the old `RepairModels` with `editorModelId`,
  so tests referencing the
  removed field checked out clean against the old API.
  The correct rule is
  BUILD FIRST,
  then `lint:types`,
  then `test:unit`;
  a green type-check over a
  stale `dist` proves nothing about either.
- `prefer-readonly-parameter-types` IS BEING IGNORED ON THIS BRANCH by user
  decision,
  and is filed as
  <https://github.com/Aquaticat/Monochromatic/issues/414>.
  It fires on ordinary array methods (`filter`,
  `map`,
  `find`,
  `flatMap`,
  `reduce`) called on parameters that are already deeply `readonly`,
  and its four
  printed remediations name no action that fits a built-in array method.
  107
  findings in this package;
  206 in `git-policy/cli`,
  which predates this branch
  and is what shows it is a rule question rather than a per-package cleanup.
  Do NOT spend branch time conforming to it,
  and do not suppress it either.
