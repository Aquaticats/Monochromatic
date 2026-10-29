# Lint fix handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/forbidden-regex.bench` is now `package/forbidden-strings/core.bench`.

## Current human instructions

- Fix repo-wide `mise run lint` failures until the root lint task passes.
- The earlier `package/dev-script/catalog-tighten` skip is superseded.
  The human said catalog-tighten work is finished and asked to fix all lint issues.
  Issue #259 is historical context only now.
- Do not stage unrelated human changes.
  Earlier known human-owned files were
  `package/dev-script/catalog-tighten.matrix/README.md` and
  `package/dev-script/catalog-tighten.matrix/src/combos.ts`.
  `git status --short` was clean immediately before this handover edit,
  but check again on resume.
- Do not satisfy `catch-binding` with an unused `catch (error)`.
  Log the caught value or rethrow it.
- `package/module/test` remains a special stdout/stderr case:
  for expected fallback catches,
  use `catch (error: unknown) { if (!Error.isError(error,)) throw error; ... }`
  instead of logging expected failures.
- Keep this handover updated from time to time.
- Current pause request:
  update this handover and pause.

## Current repository state

- Current HEAD before this handover edit:
  `71e49ce78 feat(no-restricted-syntax): catch object tag suffix checks`.
- `git status --short` printed no paths before this handover edit.
- Root lint now reports `475` rules.
  Earlier passing targeted packages may fail again under the newer rule set.

## Commits made during this lint sweep

- `7a0d8c9a7`:
  mutation-test catch bindings and Symbol descriptions.
- `3197fd056`:
  logged expected catch paths in mutation-test and vm-builder,
  added the AGENTS catch-value rule.
- `9d85256c0`:
  vm-builder catch bindings.
- `97a574293`:
  deps-cube lint diagnostics,
  including logged catch paths.
- `359640a17`:
  page-weight lint diagnostics,
  including logged catch paths.
- `61b724390`,
   `1c459b445`,
   and `8502d31c4`:
  handover refreshes.
- `f36c3efd3`:
  split figma-kiwi into focused modules,
  replaced sync decompression,
  and refreshed unit seams.
- `ff6490a2a`:
  fixed figma-penpot and mcp/stdio lint.
- `c18f8a975`:
  first module lint batch.
- `9e9c7d340`:
  preserved i18n-compose locale spec inference.
- `2289510b3`:
  next module lint batch.
- `fdc876b5a`:
  kept module-test chai re-export after imports.
- `f279c59c6`:
  parallelized toml-edit fuzz coverage reads.
- `b8a050712`:
  split toml-edit coverage aggregation.

Commits above `b8a050712` were already present when this pause handover was written.
They changed no-restricted-syntax behavior and explain why root lint now surfaces more packages.

## Verified package progress

- `mise run //package/dev-script/mutation-test:lint` passed after mutation-test fixes.
- `mise run //package/dev-script/vm-builder:lint` passed after vm-builder fixes.
- `mise run //package/dev-script/deps-cube:lint` passed after deps-cube fixes.
- `mise run //package/dev-script/page-weight:lint` passed after page-weight fixes.
- `mise run //package/figma/kiwi:lint` and
  `node package/figma/kiwi/src/index.unit.test.ts` passed after `f36c3efd3`.
  Current root lint lists kiwi failing again under newer rules.
- `mise run //package/figma/to-penpot:lint //package/mcp/stdio:lint` passed after `ff6490a2a`.
- Targeted module lints passed for dom,
  fs-path,
  i18n-compose,
  image-diff,
  kv-store,
  logger,
  or-throw,
  pipe,
  test,
  toml-edit,
  and zip-writer.
- `mise run //package/module/toml-edit:lint` passed immediately before commit `b8a050712`.

## Latest root lint results

A full root lint was started after the toml-edit split as process
`root-lint-after-next-module-fixes` (`proc_14`).
It failed.
Visible failures in that run included:

- `//package/oxlint-plugin/tsdoc:lint`:
  `method-signature-style` and `unicorn/prefer-export-from` warnings.
- `//package/oxlint-plugin/stylistic:lint`:
  one `method-signature-style` warning and seven `catch-binding` errors in
  `src/oxlint-stylistic.unit.test.ts`.
- `//package/pi-plugin/linkup:lint`:
  many `unicorn/prefer-export-from` warnings,
  `readFileSync` in `src/config.ts`,
  and an unbound catch in `src/domain-policy.ts`.

Process logs for that run:

- `/tmp/pi-processes-1782759438972/proc_14-stdout.log`
- `/tmp/pi-processes-1782759438972/proc_14-stderr.log`

The human then reran `mise run lint`.
That command failed with full output at:

```txt
/tmp/pi-bash-f1a6058111d0bdff.log
```

The aggregate failed package list from that run was:

```txt
//package/config/tofu:lint
//package/figma/kiwi:lint
//package/oxlint-plugin/tsdoc:lint
//package/pi-shared/model-selection:lint
//package/pi-plugin/current-time-context:lint
//package/oxlint-plugin/stylistic:lint
//package/pi-plugin/morph-compact:lint
//package/pi-plugin/auto-mode:lint
//package/pi-plugin/terminal-title:lint
//package/pi-plugin/statusline:lint
//package/pi-plugin/spawn:lint
//package/rust-module/forbidden-regex.bench:lint
//package/rolldown-plugin/import-attributes:lint
//package/pi-plugin/linkup:lint
//package/pi-plugin/thinking-default:lint
//package/typeface/aquaticat:lint
//package/ssg/aquati.cat:lint
//package/webapp-productivity/done-postcss:lint
//package/webapp-productivity/done:lint
//package/webapp-productivity/rss:lint
//package/webapp-productivity/doodle-widget:lint
```

Visible diagnostic themes in the human run:

- New `catch-binding` errors in many packages.
  Bind caught values and either log them through the package logger or rethrow unexpected values.
- New `no-sync` errors for APIs such as `readFileSync`,
  `writeFileSync`,
  `existsSync`,
  `mkdirSync`,
  `rmSync`,
  `statSync`,
  `symlinkSync`,
  and `zstdCompressSync`.
- New `no-low-information-symbol-description` warnings.
  Symbol descriptions must be self-explanatory phrases,
  not short tags such as `discard`,
   `no-timer`,
   or `task-not-found`.
- `unicorn/prefer-export-from` warnings for import-then-re-export patterns.
- `package/webapp-productivity/done` and `done-postcss` have both oxlint and type failures.
  The visible type errors come from calling `.get`,
   `.all`,
   or `.run` on `db.prepare(...)`
  where `prepare` is now typed as returning `Promise<Statement>`.
  Await the prepared statement before calling statement methods.
- `package/webapp-productivity/rss` has unbound catches,
  sync filesystem checks,
  and low-information Symbol descriptions.
- `package/webapp-productivity/doodle-widget` has unbound catches,
  low-information Symbol descriptions,
  and `prefer-number-coercion` warnings for `Number.parseFloat`.

## Suggested resume plan

1. Run `git status --short` and protect any unrelated human changes.
2. Fix failing packages in small batches with targeted package lint after each batch.
   Start with the plugin and Pi package failures already visible in `proc_14`,
   then work through the webapp-productivity packages from the human rerun.
3. For database calls where `prepare` returns a Promise,
   introduce named prepared statement constants before `.get`,
    `.all`,
    or `.run`.
   That should address both `TS2339` and downstream unsafe-call diagnostics.
4. For catch blocks that intentionally tolerate a failure,
   narrow `error: unknown` and rethrow unexpected values.
   Do not leave unused bindings.
5. Commit after each meaningful batch with explicit pathspecs.
6. Rerun `mise run lint` only after targeted failures pass,
   because the root fanout is large and currently reports many independent packages.
