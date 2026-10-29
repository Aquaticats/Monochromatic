# Oxlint rule: tests must import the eventual artifact

Proposal for `@monochromatic-dev/oxlint-plugin-test-import`,
whose single rule `require-eventual-artifact` bans test files from importing their own package's source
instead of the built artifact that package ships.

Status:
 implemented and registered.
The package,
 its rule,
 its fixture package,
 and its unit tests are in the tree,
and `@monochromatic-dev/config-oxlint` enables the rule at `error`.
Migration of the 697 reported sites has not started.

One decision changed during implementation,
 on measurement rather than preference:
the `**/*-helpers.ts` and `**/*-harness.ts` fixture globs were removed from the default allowlist.
The reasoning is recorded under "Fixture allowlist".

## Problem

`.claude/skills/testing-practices/SKILL.md` states that unit tests import package behavior from built `dist`,
never from sibling source files,
so that tests exercise the artifact users consume and catch export-map,
 build,
 and bundling errors.
No automated check enforces this.

Measured against active `package/**` (paused and deprecated trees are already out of lint scope
per `package/config/oxlint/src/config-base.ts`):

- 557 test files matching `**/*.{test,bench}.ts`
- 174 already compliant
- 383 violating,
   spread across 63 packages

Violating files reach their own package's code through 721 resolvable import sites.

## Definition of an eventual artifact

An import target is eventual when either condition holds.

-   The path,
     resolved and normalized,
     lies under `<package-root>/dist/final`.
    Substring matching on `dist/final` is not sufficient:
    it would accept `src/dist/final/fake.ts` as an artifact.
-   The path lies inside a directory that the owning package declares as a shipping entry,
    via `exports` (excluding the `./ts` and `./ts/*` keys),
    `main`,
     or `bin`,
    with any target under `src/` discarded,
    and with the bare `dist` root never counted as such a directory.

The bare-root exclusion matters because `package/typeface/aquaticat` declares `./dist/Aquaticat-Regular.otf`.
Taking its containing directory literally would make all of `dist/**` eventual for that package,
including `dist/temp/**`,
 which is the intermediate output this rule exists to reject.

Discarding `src/` targets is required because 30 packages currently declare entries pointing into source.
Without that clause the rule would bless exactly the imports it exists to ban.
The repo position is that `/ts` is the sanctioned source channel per ST3,
so a package's `main`,
`bin`,
 or default `exports` entry should never point at `src/`.

Directory granularity,
 rather than exact-file matching,
 is what admits the Electron packages:
`package/desktop-app/file-manager-electron` declares `main: dist/app/main.mjs`,
so `dist/app/**` is eventual and its tests may import `../dist/app/strip.js`.

## Rule behavior

Applies to files matching `**/*.{test,bench}.ts`,
the same glob the existing test override already uses at `package/config/oxlint/src/overrides.ts`.

For each static `import` declaration,
 including `import type`:

-   Relative specifier resolving inside an eventual directory:
     allowed.
-   Relative specifier resolving anywhere else:
     violation,
     unless the target matches the fixture allowlist.
-   Bare specifier equal to the owning package's own name:
     allowed,
    because it resolves through the exports map to an eventual entry.
-   Bare specifier of the owning package's own `/ts` subpath:
     violation.
-   Bare specifier of any other workspace package:
     ignored,
     including its `/ts` subpath,
     because ST3 mandates that form.

The own-package versus cross-package distinction is the reason this is a custom rule rather than configuration.
Native `no-restricted-imports` matches specifier strings only,
so banning `@monochromatic-dev/*/ts` would also hit the 83 legitimate cross-package sites
and the 537 `@monochromatic-dev/module-test/ts` harness imports.

### Fixture allowlist

A configurable rule option holds globs for test-only helpers that are not package behavior.
Default list,
 derived from conventions actually in use:

```text
**/fixture.*
**/*-fixture*.ts
**/test-support.ts
**/test-setup.ts
**/test-fixtures.ts
```

The three literal `test-` names are listed individually rather than as a `**/test-*.ts` prefix glob.
A prefix glob would also match `package/mutation-test/cli/src/container/test-run.ts`,
which is real package behavior imported by `mutant-loop.ts` and `main.ts`,
and would silently exempt it.

`**/*-helpers.ts` and `**/*-harness.ts` were in the agreed list and were removed during implementation,
on measurement rather than preference.
Because an allowlist match both exempts a module as an import target and puts that module under the rule,
a glob catching package behavior fails in both directions at once:
it exempts real behavior from tests,
 and it reports ordinary source for importing its own siblings.
Of the 23 files in this repository carrying those two suffixes,
22 are imported by package behavior and none are test-only.
`cli-helpers.ts` is imported by `cli.ts`,
`render-helpers.ts` by four i18n modules,
`tasks-helpers.ts` by three database modules.
Keeping the globs produced 19 files of reported production code that had done nothing wrong.
The suffix describes what a module does,
 not who may load it.

The earlier claim that the list "covers the 38 helper import sites found across 11 inconsistent naming
conventions" counted names without checking whether those helpers were package behavior.
 They are.
The option remains configurable,
 so restoring either glob is a one-line config change.

Allowlisted helper modules are themselves subject to the same source-import restriction.
Without that,
 a test can import a permitted helper that re-exports directly from source,
bypassing the rule without any change to the test's own import.
Matching runs against canonical resolved target paths,
 never raw specifier text.

### Packages that build nothing

A package defining no build task in its `mise.toml` is exempt entirely.
Such a package produces no artifact,
 so the rule is vacuous there rather than merely inconvenient,
and the exemption self-heals:
adding a build task re-arms the rule automatically.

This covers 35 test files across 14 packages,
mostly satellites whose purpose is exercising another package
(`css-edit.bench`,
`css-edit.fuzz`,
`jsonc-edit.conformance`,
`test-fixture/*`),
plus `config/rolldown`,
 which ships source through `/ts` by design,
and `claude-code-plugin/source`,
 whose `exports` and `bin` point entirely at `./src/**`.

The criterion is deliberately "defines no build task" rather than "has no `src/index.ts`".
The latter keys on an accident of file layout;
the former keys on whether the package ships anything at all.

### Forms not checked

Dynamic `import()`,
`require()`,
 and `export ... from` are out of scope by decision.
Consequence to accept knowingly:
`await import('./toml-set.ts')` remains a legal bypass,
and 3 such sites exist today in `package/module/toml-edit/src/toml-get-raw.unit.test.ts`,
`package/module/toml-edit/src/toml-get-node.unit.test.ts`,
and `package/pi-plugin/thinking-default/src/index.unit.test.ts`.
Adding the dynamic form later is a small change.

## Migration

Measured by running the implemented rule across `package/`,
which supersedes the pre-implementation estimates:

-   697 violation sites across 357 files in 58 packages
-   686 sites are relative imports of package source
-   11 sites are the package's own `/ts` subpath
-   353 of the files are `.test.ts` or `.bench.ts`
-   4 are allowlisted test-support modules,
     in scope because the rule checks them too

The pre-implementation estimate was 662 sites across 344 files in 54 packages.
It was produced by a line-based scanner rather than the rule's own AST walk,
and it did not count allowlisted modules,
which the anti-laundering clause brings into scope.

Largest concentrations by violating file count:
`git-policy/cli` 42,
`dev-script/file-enforcer` 36,
`module/toml-edit` 24,
`pi-plugin/auto-mode` 19,
`module/logger` 16.

Stage-one categorization below predates implementation and is retained
as the shape of the work rather than as a current count.

Stage one,
 before any exemption,
 found 721 sites in three exhaustive categories:

-   259 target a module already reachable from the package's public entry,
     so the fix is a path rewrite.
-   411 target an internal module,
     so the fix is exporting that symbol,
     which XPT permits,
     or restructuring the test.
-   51 sit in packages having no `src/index.ts`,
     so no public entry exists to measure reachability against.

The 411 group is the dominant cost and the reason this is not a mechanical migration.
`package/git-policy/cli` alone accounts for 88 of those sites.

The build bundles rather than preserving module structure,
verified as one emitted `.mjs` from 75 source files in `package/module/toml-edit`
and one from 209 in `package/git-policy/cli`.
Consequently no internal module exists as a separate file under `dist/final`,
and exporting the symbol from the bundle entry is the only route to it.

Agreed sequence:
 land the rule at `error` first,
 then migrate,
using the rule's own output as the worklist.
No allowlist and no `warn` stage;
a `warn` parking lot would conflict with oxlint `denyWarnings`.
Staged registration,
 holding the rule out of standard lint until the tree passes,
 was raised and declined.

Each symbol widened to satisfy the rule carries a TSDoc `@internal` tag.
The repo already uses that tag in 13 files
and models it in `package/oxlint-plugin/tsdoc/src/tsdoc-doc-model.ts`.
`stripInternal` is set nowhere,
so annotated declarations still reach `.d.mts` and type imports in tests keep resolving.

Widening exports does not create false unused-export findings.
`package/mutation-test/cli-unused-export/src/resolve.ts` defines `DIST_MARKER = '/dist/final/'`
and maps built-dist imports back to source precisely so test usage counts.
That marker is hardcoded to `dist/final`,
so imports under `dist/app` do not map back;
this affects the 4 Electron test files.

### Why internal modules are in scope at all

The policy is empirical,
 not aesthetic.
Defects have previously survived the test suite by existing only in built output.
An internal module tested through source therefore proves nothing about what ships.
This is why proposals to let internal unit tests import source were declined:
they reintroduce the exact failure mode the convention exists to prevent.

Consequence to accept knowingly:
from the moment the rule registers until migration completes,
`mise run lint` at repo root fails,
which affects anyone running the standard verification loop.

Type imports resolve declarations alongside the artifact for relative specifiers,
with no `exports` `types` condition required.
Measured across every package having a build task:
 80 emit both code and declarations,
 5 emit code only.
Those 5 are `kwin/key-helper`,
`ssg/aquati.cat`,
`webapp-productivity/done`,
`webapp-productivity/done-postcss`,
 and `webapp-productivity/wc`.
Only 4 type-import sites are actually at risk,
 all in `ssg/aquati.cat`;
they have no compliant form until that package emits declarations.

Relative artifact imports prove that an emitted file is importable by path.
They do not prove that the export map is correct or that consumers can reach the module.
Only the bare self-reference form exercises the export map,
so claims about catching export-map errors apply to that form alone.

## Naming

Package `package/oxlint-plugin/test-import`,
 published as `@monochromatic-dev/oxlint-plugin-test-import`.
Rule `require-eventual-artifact`,
 so diagnostics render as `test-import/require-eventual-artifact`.

Both segments are clear of `forbidden-strings.append.txt` and `forbidden-strings.append.local.txt`,
and unused elsewhere in the repo.
No ecosystem rule enforces this concept,
so there is no established name to inherit;
the closest precedents are `eslint-plugin-ava`'s `no-import-test-files`
and `eslint-plugin-ember`'s `no-test-import-export`.

Noted at decision time:
"eventual" carries an eventual-consistency first reading,
accepted because it is the established local term for the artifact a package eventually ships.

## Rejected alternatives

-   Native `no-restricted-imports` configuration,
     hand-written.
    Verified working for the bulk case on throwaway fixtures,
    but cannot express the own-package `/ts` distinction at all,
    and its eventual-directory list would be hand-maintained and drift from `package.json`.
-   Generated per-package `no-restricted-imports` overrides.
    Achieves full fidelity without custom rule code,
    but oxlint overrides are last-match-wins with option replacement,
    so the generated block is position sensitive and any broad override added later silently disables it.
-   Export-graph reachability as the exemption mechanism.
    Most faithful to the wording "not part of package behavior",
    but needs per-file whole-package graph walking,
    and the uniform-law decision removed the need for it.
-   Exempting packages whose declared entries point at `src/`.
    Would permanently exempt 43 or more test files
    and make coverage depend on manifest style.
-   Exempting packages with no `src/index.ts`.
    Rejected because the criterion keys on file layout rather than on whether the package ships an artifact.
    The buildless-package exemption replaces it on a sounder criterion.
-   Per-file or per-package allowlist of currently-failing paths.
    Rejected in favour of landing at `error` and using the rule's own output as the migration worklist.
-   Repointing the 30 `src/`-declaring manifests at `dist`.
    Correct in spirit but couples this rule's delivery to a manifest migration,
    and several of those packages ship source deliberately.

## External review outcomes

An external model reviewed this design.
 Its findings resolved as follows.

Sustained and folded into this document:
count reconciliation into exhaustive categories;
normalized path matching in place of `dist/final` substring matching;
subjecting allowlisted helper modules to the same restriction,
 closing the re-export laundering path;
scoping the declaration-availability claim to measurement;
and dropping the claim that relative artifact imports validate the export map.

Raised and declined by the owner,
 with reasons recorded here:
letting internal unit tests import source,
 and emitting a private test artifact,
both rejected because defects have previously survived tests by living only in built output;
staged registration,
 rejected in favour of immediate enforcement.

Rebutted with measurement:

-   The claim that accepting any file under `dist/final` makes the rule looser than the migration it demands.
    True in the abstract,
     unreachable in practice,
     because the build bundles rather than preserving modules.
-   The claim that relative imports of JSON,
     CSS,
     SQL,
     and similar assets would be caught unintentionally.
    No such import exists in any test file;
    the single apparent match is text inside a template literal.

Open for implementation,
 not design:
precise resolution semantics for conditional and wildcard exports,
`bin` string versus object forms,
symlinks,
manifest parse failures,
and `.js` specifiers corresponding to `.ts` sources.

## Side findings, out of scope here

-   `package/config/oxlint/src/overrides.ts` documents its array as ordered "from most specific to least specific",
    but oxlint resolves overrides last-match-wins with rule options replaced,
     not merged.
    Verified on throwaway fixtures with oxlint 1.75.0.
    Under last-wins,
     listing most-specific first means the least specific entry wins.
    Worth a separate audit of whether any existing override pair is affected.
-   30 packages declare `exports`,
    `main`,
     or `bin` entries pointing into `src/`.
    By the stated principle that no package should ship `src/*`,
     these are misconfigurations.
    Separate cleanup.

## Implementation notes

Resolution is purely lexical:
nothing is read from disk for an import target and no extension probing happens,
so a specifier naming a not-yet-built artifact classifies identically before and after a build.
This settles the review's concern that diagnostics could depend on whether a build preceded the lint.

The rule gates itself on `context.filename` and is registered globally rather than through an override.
Two reasons.
Allowlisted modules are not `.test.ts` files,
 yet must be checked,
so an override keyed on the test glob could not reach them.
And oxlint resolves overrides last-match-wins with options replaced,
so a generated block would be position sensitive.

Package identity,
 build-task presence,
 and artifact directories are memoized per directory.
They are properties of a package rather than of the file under lint,
so unlike per-file visitor state they must not be cleared between files.

The 30 packages declaring `src/` entries never collide with the own-bare-name allowance in practice:
measured across every test file,
 no bare self-reference occurs in a package whose `.` export points into `src/`.

## Shapes that satisfy the effect rule

`prefer-readonly-parameter-type/prefer-readonly-parameter-types`
rejects any parameter reaching a call whose implementation it cannot inspect,
which covers `Object.keys`,
`Object.entries`,
`Array.prototype.filter`,
`Map.prototype.set`,
 and every Oxlint host method.
These shapes in this package exist for that reason and should not be flattened back:

-   `readFixturePatterns` narrows through `isUnknownArray` rather than bare `Array.isArray`.
    `Array.isArray` widens its subject to `any[]`,
     presenting the mutable `Array` interface at the later `filter`;
    landing on `ReadonlyArray` instead keeps the call on a view TypeScript declares free of receiver mutation.

-   `createOnce` takes `ForeignHostCapability<Context>`,
     not `ForeignBorrowed<Context>`.
    Oxlint's context is runtime-owned and its `report` really does change diagnostic state,
     which the existing `@mutates context` documents.

-   The rule reports through `loc` rather than `node`.
    `report` only reads a node's position,
     so marking the node itself would document an effect that does not happen;
    copying the four line and column numbers into a fresh literal sends nothing sharing identity with the host AST.
    Oxlint's `Diagnostic` accepts either key.

-   `manifestFacts` takes manifest text and returns only the package name and its declared target strings.
    Parsing inside keeps the parsed tree from ever crossing a function boundary,
    so the `Object.entries` walk runs on a value nothing else owns.
    `eventualDirectories` takes those strings for the same reason,
    and `owningPackage` writes its verdict to the cache in place rather than through a helper.

Measured after these changes:
 zero findings from every configured rule in the package,
and the repo-wide report is unchanged at 697 sites across 357 files.
Old and new shipping-target derivation were compared across all 165 package manifests in the repo with no mismatch,
so the restructure is behavior-neutral.

`package/oxlint-plugin/tsdoc` still carries 35 findings of the same rule
and is not covered by the self-hosting override
in `package/config/oxlint/src/overrides.ts`.
Measured breakdown:
 23 name the `context` parameter and reach only `context.*` methods,
 which is exactly the shape `ForeignHostCapability` cleared here;
the remaining 12 do not,
 so the `loc` change is no help there.
Those 12 reach bodyless callables,
`functionNodes.set`,
 and `fixer` methods,
 each needing its own remedy.
The shape match has not been confirmed by applying the marker in that package.

## Next action

Migrate the 697 reported sites,
 using the rule's own output as the worklist.
Widened exports carry a TSDoc `@internal` tag.
