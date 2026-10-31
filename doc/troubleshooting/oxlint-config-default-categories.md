# Oxlint 1.89 rules-only configs still apply default correctness fixes, so focused sweeps need `categories` off

A config that sets only `rules` does not limit a run to those rules.
 `oxlint --fix` under such a
config also applies every fixable default-category rule,
mixing unrelated rewrites into what should be a single-rule normalization.

## Symptom

Run `oxlint --fix` with a config whose only rule is one JS plugin rule
(the shape used for the issue 577 indentation sweep):

```json path=probe-rules-only.json
{
  "jsPlugins": ["package/oxlint-plugin/stylistic/dist/final/node/index.mjs"],
  "rules": {
    "stylistic/indentation": "error"
  }
}
```

The sweep rewrites lines the configured rule does not govern.
 During the
2026-10-22 indentation sweep of this repository,
ten lines in eight files changed content rather than leading whitespace,
for example `unicorn(no-useless-spread)` rewriting
`return [...text.slice(0, index)].length + within;` to
`return text.slice(0, index).length + within;` in
`package/ide/app/bin/inspect-language-confinement.mjs:39`
(now `package/ide/app/bin/inspect-language-confinement.ts`,
renamed 2026-10-22),
and `unicorn(prefer-string-starts-ends-with)` rewriting a regex `.test`
call to `String.prototype.startsWith` in
`package/music-player/design/template-editor-scenes-test.mjs:70`.
 `--print-config` for the same config prints `categories: {}`
while listing 111 active built-in rules.

## Root cause

1.  The `categories` field defaults to an empty map,
    so a config that omits it (or sets only `rules`) reports nothing configured.
    The installed schema records the default in
    `node_modules/.pnpm/oxlint@1.89.0_oxlint-tsgolint@7.0.2004/node_modules/oxlint/configuration_schema.json`:

    ```json path=configuration_schema.json
    "categories": {
      "default": {},
      "allOf": [
        {
          "$ref": "#/definitions/OxlintCategories"
        }
      ]
    },
    ```

2.  An empty `categories` map does not disable the category defaults.
    The oxlint configuration guide states the design
    (fetched 2026-10-22 from
    <https://oxc.rs/docs/guide/usage/linter/config.html>):
    "Categories let you enable or disable sets of rules with similar intent.
    By default,
    Oxlint enables rules in the `correctness` category."
    The correctness category spans the default plugins
    (`typescript`,
    `unicorn`,
    `oxc`),
    which is why `unicorn/no-useless-spread` fires under a rules-only config.

3.  The `rules` map overrides categories per rule,
    not wholesale.
    The same schema describes `OxlintCategories`:
    "Rules enabled or disabled this way will be overwritten by individual
    rules in the `rules` field."
    Entries in `rules` therefore win for the rules they name,
    while every unlisted rule keeps its category severity.

4.  A rules-only config consequently runs
    the listed rules plus every default correctness rule,
    and `--fix` applies all fixable ones.

An earlier reading of this behavior was wrong:
"the empty `categories: {}` in `--print-config` means no category rules are
enabled."
 The mirror sweep disproved it
(10 content edits appear next to 21,441 whitespace edits),
and `--print-config` itself lists the 111 default rules beside that map.

## Verification

Version under test:
`oxlint` 1.89.0,
npm package `oxlint@1.89.0` under
`node_modules/.pnpm/oxlint@1.89.0_oxlint-tsgolint@7.0.2004`;
`oxlint --version` prints `Version: 1.89.0`.
 Config semantics were read from the installed
`configuration_schema.json` cited in "Root cause"
and the configuration guide fetched 2026-10-22.

Harness:
two probe configs differing only in `categories`,
run with the repository's `oxlint` binary from the repository root:

```json path=probe-categories-off.json
{
  "jsPlugins": ["package/oxlint-plugin/stylistic/dist/final/node/index.mjs"],
  "categories": {
    "correctness": "off"
  },
  "rules": {
    "stylistic/indentation": "error"
  }
}
```

```sh path=shell
# Resolved rule sets
node_modules/.pnpm/oxlint@1.89.0_oxlint-tsgolint@7.0.2004/node_modules/oxlint/bin/oxlint \
  --print-config --config probe-rules-only.json
node_modules/.pnpm/oxlint@1.89.0_oxlint-tsgolint@7.0.2004/node_modules/oxlint/bin/oxlint \
  --print-config --config probe-categories-off.json

# Default-category diagnostics on the eight files the sweep touched
node_modules/.pnpm/oxlint@1.89.0_oxlint-tsgolint@7.0.2004/node_modules/oxlint/bin/oxlint \
  --config probe-rules-only.json \
  package/ide/app/bin/inspect-language-confinement.mjs \
  package/ide/app/bin/inspect-language-lifecycle.mjs \
  package/ide/app/bin/inspect-language-mise.mjs \
  package/ide/app/bin/inspect-language.mjs \
  package/ide/app/design/wrap-annotation-shots.mjs \
  package/linter/monochromatic-lint/bin/test-container.mjs \
  package/music-player/design/template-editor-scenes-test.mjs \
  package/music-player/truepeak-core.bench/analysis/tags-sweep.mjs
```

The `package/ide/app` files in that command were renamed on 2026-10-22
to TypeScript files of the same base names
(`bin/inspect-language-confinement.ts`,
`bin/inspect-language-lifecycle.ts`,
`bin/inspect-language-mise.ts`,
`bin/inspect-language.ts`,
`design/wrap-annotation-shots.ts`),
see issue 1794;
the command is kept as it was run.
`package/linter/monochromatic-lint/bin/test-container.mjs` was renamed on 2026-10-30 to
`bin/test-container.ts` (and no longer uses the banned `spawnSync`).

Works cleanly with `probe-categories-off.json`:
`--print-config` prints `"categories": {"correctness":"allow"}`
(`"allow"` is the schema's disabled severity,
equal to `"off"`) and zero built-in rules;
the indentation-only sweep over 7,333 candidates then changes
21,441 leading-whitespace lines,
zero content lines,
and zero line counts,
converging in one fix pass.

Fails with `probe-rules-only.json`:
`--print-config` prints `categories: {}` with 111 built-in rules
(`constructor-super`,
`for-direction`,
`getter-return`,
and so on),
and the eight-file run reports 15 diagnostics:
`eslint(no-control-regex)` 2,
`eslint(no-useless-escape)` 4,
`unicorn(no-useless-fallback-in-spread)` 3,
`unicorn(no-useless-spread)` 5,
`unicorn(prefer-string-starts-ends-with)` 1.
 Ten of them are fixable and produced the 10 content edits
(5 spread removals,
2 useless-escape quote fixes,
2 useless-fallback-in-spread collapses,
1 `startsWith` rewrite);
the two `eslint(no-control-regex)` diagnostics and three more are report-only.

## Verified workarounds

- Add `"categories": { "correctness": "off" }` to the sweep config,
  as in `probe-categories-off.json` above.
  Tradeoff:
  every correctness rule is disabled for the run,
  including ones a sweep might legitimately want,
  so a clean single-rule diff is bought by giving up that coverage;
  `--print-config` then shows the category as `"allow"`.
- Re-enable the wanted default rules individually under `rules`,
  which override categories per rule.
  Tradeoff:
  each rule must be named one by one,
  so a category-sized wish list becomes a hand-maintained rule map.

## What does not work

- Treating `rules` as an allow-list.
  The mirror sweep and `--print-config` both show the default rules running
  beside the listed one.
- Reading `--print-config`'s `categories: {}` as "no categories enabled".
  The empty map is the user-set field;
  the resolved rule list beside it is the truth.
- Leaving the sweep config rules-only and reviewing the diff afterwards.
  The 10 content edits hid among 21,441 whitespace edits across 1,845 files,
  so review is where they escape.

## Upstream filing artifact

No upstream issue or comment is kept,
because the observed behavior is the documented default.
 The decision walk records why.

### Upstream filing decision

`doc/troubleshooting/` sibling `oxlint-config-load-failure-exit-code.md`
covers config loading,
and `.out-of-scope/` was checked before this decision:
no exemption covers config-default semantics.
 The nearest entry,
`.out-of-scope/low-impact-typescript-formatting.md`,
covers investment in formatter rules
("This project does not invest in low-impact TypeScript formatter rules"),
not category resolution,
so it does not match.

Duplicate search on 2026-10-22:
`gh search issues --repo oxc-project/oxc 'default correctness category rules config'`
returned nothing open or closed.
 The probe works:
`gh search issues --repo oxc-project/oxc 'oxlint'` returns current issues.

#### Is it really upstream's fault?

No.
 The behavior matches the documented design
("By default,
Oxlint enables rules in the `correctness` category"),
so this is a usage pitfall,
not a defect.
 This constraint fails and the default policy applies:
do not file.

#### Can upstream fix it?

Not applicable.
 There is no defect to fix,
and changing the default would break documented semantics.

#### Are they supporting this use case?

Yes.
 The `categories` field exists exactly to enable or disable rule groups,
and the guide shows per-rule overrides beside it.

#### Would the repo welcome our contribution?

Not applicable.
 Nothing is being filed,
so contribution policy was not consulted.

#### Will they likely fix it?

Not applicable.
 No change is requested.

#### Have we prototyped a minimal fix compatible with their architecture?

Not applicable upstream.
 The consumer-side minimal change is the `categories` entry in
`probe-categories-off.json`,
verified in "Verification".

Outcome:
nothing to add;
no upstream issue or comment is kept,
because the observed behavior is the documented default.
