# catalog-tighten repair and portability decision log

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/catalog-tighten.matrix` is now `package/dependency-catalog/cli-tighten.matrix`;
> `package/module/pnpm-workspace-catalog` is now `package/dependency-catalog/core-pnpm`;
> `@monochromatic-dev/module-pnpm-workspace-catalog` is now `@monochromatic-dev/dependency-catalog-core-pnpm`;
> `package/dev-script/catalog-tighten` is now `package/dependency-catalog/cli-tighten`;
> `package/module/matrix` is now `package/test-harness/core-matrix`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

Living record of the work to fix `catalog-tighten` (GitHub issues #258 and #195)
and make it portable across package managers,
updated across sessions as decisions crystallise.
Tracks what we settled,
why,
what stays open,
and the planned shape of the deliverable.

## Status

pnpm phase complete;
 paused as planned.
The #258 three defects and the #195 hardening are fixed,
 linted,
 type-checked,
 unit-tested,
 and
verified end-to-end on the real repo (38 entries tighten,
 minimal single-quoted diff).
 Reading uses
the `yaml` library;
 resolution reads on-disk install state only (node_modules linkers plus a pnp
reader),
 never the lockfile.
 The `catalog-tighten.matrix` sidecar runs all six layout combinations
in containers and passes (isolated and hoisted with hoist on/off,
 pnp,
 and the stale-orphan
regression).
 All work is committed and pushed to `main`.

The four deferred-manager issues are filed:
 Bun (#260),
 Yarn Berry (#261),
 Deno (#262),
 vlt (#263).
The catalog reader consolidation (#264) is now complete;
next sessions pick up the manager-specific issues.

## Progress

- Done:
   shared `@monochromatic-dev/module-pnpm-workspace-catalog` reader;
  yaml-library parsing;
  string-scan name validator + `Object.create(null)` maps (#195);
  on-disk resolver (root + workspace `node_modules`,
   exports-bypassing),
   Bun-store scan removed;
  surgical single-quote-preserving write-back;
   no-install guard;
   clear error on a missing
  `pnpm-workspace.yaml`;
   README rewrite;
   host unit tests.
- Done:
   pnpm pnp reader (`version-pnp.ts`,
   via the `.pnp.cjs` Yarn-style API).
   Note:
   default pnpm pnp
  is a hybrid that also keeps per-importer `node_modules` symlinks,
   so the node_modules walk-up
  resolves it.
   With `symlink: false` (pnpm's recommended pnp config) the symlinks are gone and it is a
  true no-node_modules layout,
   where the `.pnp.cjs` reader is the only resolution path;
   that scenario
  exercises `version-pnp.ts` through the matrix.
- Done:
   effective `modulesDir` support via `pnpm config get modules-dir` (covers cli/env/workspace/
  global config).
   `virtualStoreDir`,
   `enableGlobalVirtualStore`,
   and `storeDir` need no code:
   the
  resolver follows the active symlink wherever the store moved,
   including the global store.
   All four
  verified in the matrix.
- Done:
   the `catalog-tighten.matrix` sidecar,
   now 20 containerised scenarios:
   five layouts,
   two
  pnp `symlink: false` shapes,
   four store-settings,
   stale-orphan,
   and eight missing-X robustness
  cases (missing lockfile / store / some-node_modules / pnpm / default-pnp .
  `pnp.cjs` still tighten;
  missing virtual store and symlink-off .
  `pnp.cjs` -> MISS;
   missing all node_modules / missing
  workspace yaml -> clean error).
- Filed as issues:
   Bun (#260),
   Yarn Berry (#261),
   Deno (#262),
   vlt (#263),
   and the deps-cube reader
  consolidation (#264).

## The two issues

- #258:
   `catalog-tighten --dry-run` resolves nothing.
  The issue's investigation notes blame single-quote parsing only.
- #195:
   prototype-pollution hardening of the catalog parser
  (crafted `__proto__:` keys could mutate the result map).

## Diagnosis (broader than the issues stated)

#258 is three defects,
 not one:

1.  Parse (`package/module/pnpm-workspace-catalog/src/parse.ts`):
    the shared YAML parser handles single quotes,
    double quotes,
    comments,
    default catalogs,
    and named catalogs while retaining raw values for callers that need them.
2.  Resolve (`package/dev-script/catalog-tighten/src/version-resolve.ts`,
    `package/dev-script/catalog-tighten/src/version-read.ts`):
    even after the quote fix,
     122 of 134 entries still miss.
    They are sub-package deps not hoisted to root `node_modules`,
    and `require.resolve('<pkg>/package.json')` throws `ERR_PACKAGE_PATH_NOT_EXPORTED`
    for packages whose `exports` map does not expose `./package.json`.
    The store-scan fallback exists for this case
    but scans `node_modules/.bun/` (Bun),
     which does not exist in this pnpm repo (`.pnpm/`).
3.  Write-back (`package/dev-script/catalog-tighten/src/index.ts`):
    the rewrite only matches `"name": "range"` (double quotes),
    so on the single-quoted file the real (non-dry-run) command silently writes nothing.
    Invisible to the issue's acceptance criteria,
     which only test `--dry-run`.

## Measurements (throwaway worktree, real `.pnpm` store)

- Single-quote fix alone:
   8 tighten,
   4 already tight,
   122 not found.
- Plus direct symlink read across workspace roots
  (read `<wsRoot>/node_modules/<name>/package.json`,
   bypassing exports):
   38 tighten,
   61 already tight,
   35 not found.
- The remaining 35 are consumed only by `package-paused/**` packages (not installed,
   no active symlink)
  or are transitive-only.
  Neither should be tightened:
   there is no active installed version to tighten a floor against.

## Locked decisions

1.  Scope:
     full end-to-end repair of all three #258 defects plus #195.
    The tool must actually tighten,
     not just pass `--dry-run`.
2.  Resolution source:
     actual on-disk install state,
     never the lockfile.
    Rationale:
     a lockfile persists after `node_modules` is deleted,
    so it reports versions that are not installed.
    - nm-linker layouts (pnpm isolated/hoisted,
       and other managers' node-modules linkers):
      read `node_modules/<name>/package.json`,
       walking up from each importer to the root,
      bypassing exports-gated `require.resolve`.
    - no-`node_modules` layouts (pnpm `nodeLinker: pnp`):
      read the manager's real install artifact via the PnP API (`.pnp.cjs`),
       still on-disk state.
    - `node_modules` absent entirely:
       MISS-and-skip with a loud warning.
3.  Drop the version-store scan entirely
    (`readVersionFromBunStore` and the `.bun`/`.pnpm` directory scan).
    It is package-manager-specific and hazardous:
    `doc/troubleshooting/pnpm-modules-cache.md` shows `.pnpm` retains stale higher-version orphans,
    so scan-and-pick-highest could tighten a floor above the active version and break installs.
4.  #258 fixes:
    - Parse:
       `unquote()` strips a matching single or double quote pair.
    - Resolve:
       redesign per decision 2.
    - Write-back:
       replace only the version token within the matched entry's line,
      preserving the existing quote characters and spacing (minimal diff).
5.  #195 hardening:
    - Name validation:
       string-scan predicate,
       no regex
      (repo lint rule `no-restricted-syntax/no-regex`,
       and this parser was deliberately rewritten from regex to string scans).
      Check non-empty,
       first char `@` or `[a-z0-9]`,
       chars in the npm-name set,
       at most one `/`.
    - Container:
       `Object.create(null)` plus `Object.hasOwn`.
    - Rejection:
       log a warning and skip the entry,
       preserving the rest of the catalog.
6.  README rewrite:
    `package/dev-script/catalog-tighten/README.md` currently misdescribes the source
    as the root `package.json` `workspaces.catalog`;
    the code reads `pnpm-workspace.yaml` `catalog:`.

## Phasing

- pnpm first:
   one PR closes #258 and #195,
   including a PnP-API reader (pnp is in the matrix).
- Pause after pnpm.
  For Bun,
   Yarn Berry,
   Deno,
   and vlt,
   open one self-contained GitHub issue per manager
  carrying the source-format and layout findings below as its spec,
   instead of implementing.

## Catalog support across managers (mid-2026, source-verified)

Reference protocol `catalog:` / `catalog:<name>` is uniform;
 the install layout is not.

- pnpm (9.5+,
   GA):
   source `pnpm-workspace.yaml` `catalog:`/`catalogs:`.
  Default linker is the symlinked isolated store;
   also `hoisted` and `pnp`.
- Bun (1.2.14+,
   May 2025):
   source `package.json` `workspaces.catalog`/`catalogs` (or top level).
  Default hoisted (real dirs);
   opt-in isolated (`node_modules/.bun`).
- Yarn Berry (4.10+,
   Sept 2025):
   source `.yarnrc.yml` `catalog:`/`catalogs:`.
  Default PnP has no `node_modules` (`.pnp.cjs`,
   `.yarn/cache` zips);
  switchable to `nodeLinker: node-modules` or `pnpm`.
- Deno (2.8+,
   May 2026):
   source `deno.json` or `package.json`,
   npm-only entries.
  `.deno` store in auto mode;
   `nodeModulesDir: none` has no `node_modules`.
- vlt (young,
   pre-1.0):
   source `vlt.json` `catalog`/`catalogs`.
  `node_modules/.vlt/` symlink store.
- npm:
   no catalogs (RFC #528 open and unmerged since 2022).
- Yarn Classic v1:
   no catalogs.

The layouts that break any `node_modules`-reading strategy:
 Yarn Berry PnP and Deno `none`.

## E2e test infrastructure

- Mechanism:
   a bespoke sidecar package `package/dev-script/catalog-tighten.matrix`
  (`.<suffix>` is the repo's sidecar convention,
   as in `forbidden-regex.fuzz` and `truepeak-core.bench`;
  those are Rust with no `package.json`,
   so this is the first TypeScript sidecar
  and gets a `package.json` with `workspace:*` deps per AP1 to AP5),
  borrowing `package/module/matrix`'s cartesian-product and `describe`/`it` reporting shape
  and `package/dev-script/mutation-test`'s isolation
  (baked node+pnpm image,
   fixture mounted read-only,
   tmpfs `node_modules`,
   dropped caps,
   no network).
- Why not reuse `module-matrix` directly:
   its model mounts the real monorepo read-write
  (`-v ${monorepoRoot}:/workspace:Z`) and runs a file with a curl-installed bun/deno runtime,
  with no package-manager/settings axis.
  Our e2e needs the opposite:
   a throwaway fixture installed with pnpm under a chosen setting,
  `catalog-tighten` run via node,
   never touching the real tree.
  Extract a shared pm-install-matrix module later,
   once a second manager proves the pattern.
- Test isolation policy:
   containerise the install-based resolution tests only
  (the host's own install would contaminate a hoisted or stale-orphan fixture).
  Pure-function tests (parser,
   name-validator,
   write-back) stay as fast host unit tests
  via `@monochromatic-dev/module-test`.
- pnpm matrix axes (selected):
   `nodeLinker` isolated,
   hoisted,
   and pnp;
   `hoist` true and false.
- Required correctness scenario:
   seed a higher-version stale orphan into `node_modules/.pnpm`
  (simulating a post-downgrade leftover,
   per `doc/troubleshooting/pnpm-modules-cache.md`)
  and assert the resolver tightens to the active lower version,
   never the orphan.
- `packageImportMethod` is not an axis:
   it changes how files are placed,
   not the structure read.

## Open items

- Fixture workspace shape and how pnpm settings are applied per container combo.
- pnpm pnp reader:
   exact PnP-API call sequence.
- Commit slicing for the pnpm work (committing to `main`,
   eagerly).
- `catalog-tighten` remains default-catalog-only;
   `deps-cube` explicitly opts into named
  catalogs through the shared raw-entry flattener.

## Key files

- `package/module/pnpm-workspace-catalog/src/parse.ts`:
   shared catalog parser and #195 hardening.
- `package/module/pnpm-workspace-catalog/src/read.ts`:
   located-file reader retaining raw content.
- `package/dev-script/catalog-tighten/src/version-resolve.ts`:
   installed-version resolution (#258 defect 2).
- `package/dev-script/catalog-tighten/src/version-read.ts`:
   store scan to be removed.
- `package/dev-script/catalog-tighten/src/index.ts`:
   top-level run and write-back (#258 defect 3).
- `package/dev-script/catalog-tighten/README.md`:
   to rewrite.
- `doc/troubleshooting/pnpm-modules-cache.md`:
   stale-orphan hazard rationale.
- `pnpm-workspace.yaml`:
   the catalog source,
   single-quoted,
   `nodeLinker: isolated`.
