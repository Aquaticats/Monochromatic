# Remove stylelint and config-stylelint

## Status

Accepted 2026-10-04.
Implemented in commit `f1bca05`.
Replacement tracked in issue [#587][issue].

## Context

The repository linted CSS with stylelint 17.16.0,
 plus `stylelint-config-standard` 40.0.0 and `postcss-html` 2.0.0,
 through the root `stylelint.config.mjs`
 extending `@monochromatic-dev/config-stylelint` (`package/config/stylelint`).
Active tree held 32 `.css` files (23 more under `package-paused/`),
 so the linter served little code.

On 2026-10-04 `pnpm audit` reported advisory 1240992,
 [GHSA-vfj7-8cjw-p6xm][advisory]
 ("braces vulnerable to stack-exhaustion denial of service through deeply nested patterns"):
 high severity,
 `braces` `<=3.0.3`,
 no patched release published.
All 16 findings reached `braces` through stylelint only
 (`stylelint > micromatch > braces`,
 `stylelint > globby > fast-glob > micromatch > braces`,
 and the same paths via `stylelint-config-standard`).

## Decisions

### Drop stylelint wholesale

- Removed `stylelint`,
  `stylelint-config-standard`,
  `postcss-html`,
  and `@monochromatic-dev/config-stylelint`
  from the workspace and catalog,
  plus `stylelint.config.mjs`,
  the `lint:stylelint` and `format:stylelint` mise tasks,
  the `stylelint.vscode-stylelint` extension,
  the `com.intellij.stylelint` plugin and its inspection,
  the `.stylelintcache` ignore entry,
  and every `stylelint-disable` comment.
- `postcss-html` went with it:
  it existed only for stylelint's HTML custom syntax.
- Kept:
  dprint formats CSS,
  `@monochromatic-dev/css-cli` builds it,
  and `@monochromatic-dev/hyperscript` type-checks CSS property,
  value,
  and at-rule names at the type level.

### Replace with a first-party CSS linter

Issue [#587][issue] carries the full rule inventory of the removed config
 (property,
  unit,
  at-rule,
  and color-function denylists,
  media-query shape,
  notation and whitespace rules),
 the known defects not to repeat,
 and acceptance criteria.
Until it lands,
 CSS has no semantic linting.

### Keep the security overrides

The `fast-uri` override stays as defense in depth
 even though its only consumer (`ajv` via `stylelint > table`) may leave the graph.

## Durable facts carried from `doc/troubleshooting/stylelint.md`

That file was deleted with the tool.
Its root cause and workaround survive here:

- stylelint 16.x and 17.x dynamically import `postcss-html`
  when configured with `customSyntax: 'postcss-html'`
  (via `stylelint/lib/utils/dynamicImport.cjs`)
  but declare it neither as dependency nor peer dependency.
  Under pnpm strict isolation the lookup starts at stylelint's own install
  and fails with `ERR_MODULE_NOT_FOUND`.
- Verified workaround while we used stylelint:
  a pnpm `packageExtensions` entry adding `postcss-html` to stylelint's dependencies.
  npm and yarn ignore `packageExtensions`;
  their consumers had to declare `postcss-html` themselves.
- Not filed upstream under the five-constraint policy:
  existing upstream issues already tracked it.
- The shared config also passed regular expressions as strings,
  so stylelint received `[\w-]+` as `[w-]+`
  (probe results in `doc/planning/learning-rust-formatting-boundary.md`).

## Consequences

- `pnpm audit` reports zero advisories
  and the resolved dependency count drops from 812 to 708.
- `mise run lint` and `mise run format` no longer run stylelint.
  Until [#587][issue] lands they cover CSS only as formatting via dprint.
- `README.md`,
  `doc/philosophy/tool-choices.md`,
  and `doc/todo/package-structure.md` list no stylelint.

[advisory]: https://github.com/advisories/GHSA-vfj7-8cjw-p6xm
[issue]: https://github.com/Aquaticat/Monochromatic/issues/587
