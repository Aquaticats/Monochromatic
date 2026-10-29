> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/cli/fy` is now `package/fy/cli`.

<!--
status: final; recommendation recorded, adoption not authorized
lifecycle-phase: recommended (retain incumbent); candidate B exited on proven hard-gate failures
subject: pnpm store-loader
scope: Decide whether the Monochromatic monorepo can migrate dependency linking
 from the pnpm isolated node_modules linker to the pnpm experimental loaded linker (store loader).
started: 2026-10-18
last-updated: 2026-10-18
governing-skill: .agents/skills/choosing-technology/SKILL.md
governing-skill-commit: 37117e36397e30233062828581a5e38217da9458
governing-skill-sha256: 552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b
compatibility-fingerprint: af53173d69e8b1f95654626ec891c8c57dea60e9b096d0464d9c8a05a4bc0aba
active-audit-owner: /var/home/user lead session (DSH web GUI)
prior-compatible-report: none; no report in doc/audit shares the tech-pnpm-store-loader subject slug
unresolved-preferences: none required for the feasibility verdict; adoption authority stays with the user
criteria-and-weights: six criteria frozen at weight 1 each; scoring runs only for finalists that survive hard gates
hard-constraints:
  - CI installs remain possible from the committed frozen lockfile with store caching
  - Electron packaging and Playwright-in-podman browser and e2e runs keep working
  - Mise-run workflows keep working: tsc --build type check, bare-node TypeScript unit tests,
   rolldown node/browser/client builds, oxlint with tsgolint, watch and dev servers
  - Native addon, napi, WASM and platform-binary dependencies stay loadable and buildable
  - No regression of the strict isolation guarantees behind nodeLinker isolated with hoist false
  - Node surfaces that must run stored code meet the loader Node floor of 24.18+ or 26.2+
  - Supply-chain policy keeps effect: .pnpmfile readPackage substitution, overrides including link shims,
   allowBuilds approvals, strict minimumReleaseAge, trustPolicy no-downgrade
  - The committed lockfile stays portable across supportedArchitectures darwin/linux, x64/arm64, glibc/musl
base-components:
  - Inspectable open-source local technology
overlays:
  - High-trust execution in CI and every spawned Node process
  - Incumbent dependency replacement
  - Multi-platform claim
  - Native, Wasm and prebuilt binary boundary
-->
# pnpm store-loader migration vet

## Purpose and authorization

The user asked whether this repository can migrate to the pnpm experimental loaded linker,
documented at [pnpm.io/store-loader][store-loader].
The request is a decision verb,
so this evaluation produces an answer plus this report.
The main worktree receives documentation writes only.
Experiments run in a disposable git worktree that is removed after this report completes.
Adoption,
if ever authorized,
is a separate later action with its own decision record.

## Context (measured facts)

Host toolchain,
measured on 2026-10-18 in `/var/home/user/Monochromatic`:

- pnpm 12.14.0 (`pnpm --version`),
  installed by mise (`pnpm = "latest"`,
  [mise.toml](../../mise.toml));
- Node v26.12.0 (`node --version`),
  installed by mise (`node = "latest"`);
- host is Fedora Silverblue-class Linux x64,
  Btrfs `/var/home`,
  935 GiB free,
  61 GiB RAM (`df`,
  `free`).

Workspace linker and supply-chain configuration in [pnpm-workspace.yaml](../../pnpm-workspace.yaml):

- `nodeLinker: isolated` (line 220),
  `hoist: false` (line 196),
  `hoistWorkspacePackages: false` (line 197);
- `packageImportMethod: clone-or-copy` (line 424),
  `modulesCacheMaxAge: 0` (line 13);
- `allowBuilds` build-script approvals with `sharp: true` (lines 1 to 6);
- `.pnpmfile.mjs` `readPackage` hook substituting blocklisted packages with workspace stubs;
- `overrides` including `link:package/shim/*` substitutions (lines 222 to 403);
- strict `minimumReleaseAge: 1440` with excludes (lines 199 to 218),
  `trustPolicy: no-downgrade` (line 460);
- `supportedArchitectures` darwin/linux,
  x64/arm64,
  glibc/musl for lockfile portability (lines 447 to 459),
  rationale in [doc/decision/pnpm-supported-architectures.md](../decision/pnpm-supported-architectures.md);
- catalogs with `catalogMode: strict` (lines 16 to 193).

Task execution model,
measured in [mise.toml](../../mise.toml) (generated from `mise.no-env.toml`):

- unit tests run as bare `node <file.test.ts>` subprocesses spawned by the `run_test_files` var
  (Node native TypeScript type-stripping),
  never through `pnpm run` or `pnpm exec`;
- package entries dispatch through `execFileSync('node', ...)` in the `dispatch_workspace_node` var;
- builds call `rolldown` directly;
  type checks call the workspace `tsc-filter` wrapper with `--build`;
- browser and e2e tests run Playwright inside a podman image over a bind mount of the repo;
- `prepare` runs `pnpm install` first because every later child needs `node_modules`.

The loaded linker,
as documented at [pnpm.io/store-loader][store-loader] (fetched 2026-10-18):

- enabled via `nodeLinker: { type: loaded, excluded: [...] }`;
  install writes
  `node_modules/.pnpm/.store-manifest.json` and `node_modules/.pnpm/.store-loader.mjs`;
- `pnpm run`,
  `pnpm test` and `pnpm exec` preload the loader;
  Node children inherit it through
  `NODE_OPTIONS`;
  generated JS bin shims register it;
  bare `node` outside pnpm needs an explicit
  `--import` preload;
- requires Node 24.18+ (24.x) or 26.2+,
  else every preloaded process stops with
  `ERR_PNPM_LOADER_UNSUPPORTED_NODE`;
- unsupported from the store:
  native addons,
  WebAssembly,
  TypeScript files,
  package-internal symlinks;
  `import.meta.url`,
  `__filename`,
  `__dirname` and `require.resolve()` of stored modules are virtual,
  so direct `fs` reads of package assets fail;
- independent test-runner resolvers and direct filesystem reads need materialization
  (`excluded` packages land in the global virtual store with their whole dependency tree)
  or resolver integration;
- `symlink: false` and `nodePackageMap.mode` other than `off` cannot combine with the mode;
- the standalone `@pnpm/esm-loader` API exists for manual integration.

Prior related repository evidence (to be re-verified by the repository inventory):

- [doc/troubleshooting/pnpm-symlink-false-isolated.md](../troubleshooting/pnpm-symlink-false-isolated.md)
  analyzed `nodeLinker` modes against virtual-store layout;
- [package/dev-script/catalog-tighten.matrix](../../package/dev-script/catalog-tighten.matrix/README.md)
  already exercises isolated,
  hoisted and pnp scenarios;
- [doc/troubleshooting/napi-loader-bundled-out-of-package.md](../troubleshooting/napi-loader-bundled-out-of-package.md)
  documents why strict isolation matters for napi resolution;
- [doc/troubleshooting/pnpm-modules-cache.md](../troubleshooting/pnpm-modules-cache.md) and
  [doc/handover/pnpm-modules-cache-benchmark.md](../handover/pnpm-modules-cache-benchmark.md)
  record modules-cache benchmarks under the isolated linker.

## Hard constraints (frozen)

Each constraint lists its measured source.

1. Mise-run workflows keep working:
   `tsc --build` type check,
   bare-node TypeScript unit tests,
   rolldown node/browser/client builds,
   oxlint with tsgolint,
   watch and dev servers.
   Source:
   [mise.toml](../../mise.toml) task templates and vars.
2. Native addon,
   napi,
   WASM and platform-binary dependencies stay loadable and buildable.
   Source:
   catalog entries in [pnpm-workspace.yaml](../../pnpm-workspace.yaml)
   (sharp,
   electron,
   Playwright,
   libsql/turso,
   ripgrep,
   esbuild,
   wasm font packages),
   `allowBuilds` list,
   napi incident record
   [depot-git-host-failure evidence](depot-git-host-failure-mapping-2026-10-17.evidence.json)
   and [napi-loader troubleshooting](../troubleshooting/napi-loader-bundled-out-of-package.md).
3. Supply-chain policy keeps effect:
   `.pnpmfile.mjs` substitution,
   `overrides` including `link:` shims,
   `allowBuilds` approvals,
   strict `minimumReleaseAge`,
   `trustPolicy: no-downgrade`.
   Source:
   [pnpm-workspace.yaml](../../pnpm-workspace.yaml),
   [.pnpmfile.mjs](../../.pnpmfile.mjs).
4. The committed lockfile stays portable across `supportedArchitectures` (darwin/linux,
   x64/arm64,
   glibc/musl) for mixed development and CI environments.
   Source:
   [pnpm-workspace.yaml](../../pnpm-workspace.yaml) lines 437 to 459.
5. CI installs remain possible from the committed frozen lockfile with store caching.
   Source:
   `.github/workflows` (exact invocations recorded by the repository inventory).
6. Electron packaging and Playwright-in-podman browser and e2e runs keep working.
   Source:
   `@electron/packager` catalog entry,
   `run_playwright` var in [mise.toml](../../mise.toml).
7. Node surfaces that must run stored code meet the loader Node floor (24.18+ or 26.2+).
   Source:
   [pnpm.io/store-loader][store-loader];
   host Node is v26.12.0,
   other surfaces pending inventory.
8. No regression of the strict isolation guarantees behind `nodeLinker: isolated` with `hoist: false`.
   Source:
   [pnpm-workspace.yaml](../../pnpm-workspace.yaml) line 409 comment,
   [napi-loader troubleshooting](../troubleshooting/napi-loader-bundled-out-of-package.md).

## Candidate ledger

### Candidate A: pnpm isolated node_modules linker (incumbent)

- Discovery source:
  repository configuration,
  [pnpm-workspace.yaml](../../pnpm-workspace.yaml) line 220.
- Base category:
  inspectable open-source local technology (pnpm 12.14.0).
- Overlays:
  high-trust execution,
  native/Wasm/prebuilt boundary,
  multi-platform.
- Screening result:
  incumbent;
  required candidate under the replacement-parity overlay.

### Candidate B: pnpm loaded linker (store loader)

- Discovery source:
  user request naming [pnpm.io/store-loader][store-loader].
- Base category:
  inspectable open-source local technology (embedded in pnpm;
  standalone
  `@pnpm/esm-loader`).
- Overlays:
  incumbent replacement,
  high-trust execution (preload injected into every spawned Node
  process),
  native/Wasm/prebuilt boundary,
  multi-platform claim.
- Screening result:
  promoted to finalist and validated at equal depth (install,
  layout,
  runtime,
  toolchain probes);
  **exited on hard-gate failures** (constraints 1 and 6) recorded in the
  Hard-gate outcomes section.

### Candidate C: pnpm hoisted linker

- Discovery source:
  repository evidence class;
  prior mode trials in
  [doc/troubleshooting/pnpm-symlink-false-isolated.md](../troubleshooting/pnpm-symlink-false-isolated.md)
  and [doc/troubleshooting/pnpm-pack-dedupe-direct-deps.md](../troubleshooting/pnpm-pack-dedupe-direct-deps.md)
  (hoisted used as a control,
  discarded).
- Base category:
  inspectable open-source local technology.
- Overlays:
  incumbent replacement,
  native/Wasm boundary,
  multi-platform.
- Screening result:
  **exited at screening**.
  A hoisted flat layout makes undeclared dependencies
  resolvable by design,
  failing constraint 8,
  the guarantee this repository bought with
  `hoist: false` + isolated after the napi incident recorded in
  [doc/troubleshooting/napi-loader-bundled-out-of-package.md](../troubleshooting/napi-loader-bundled-out-of-package.md);
  prior repository trials used hoisted only as a control and discarded it
  ([doc/troubleshooting/pnpm-pack-dedupe-direct-deps.md](../troubleshooting/pnpm-pack-dedupe-direct-deps.md)).

### Candidate D: pnpm pnp linker

- Discovery source:
  repository evidence class;
  scenarios in
  [package/dev-script/catalog-tighten.matrix](../../package/dev-script/catalog-tighten.matrix/README.md)
  and [version-pnp.ts](../../package/dev-script/catalog-tighten/src/version-pnp.ts).
- Base category:
  inspectable open-source local technology.
- Overlays:
  incumbent replacement,
  high-trust execution (Yarn-PnP-style resolver),
  native/Wasm boundary,
  multi-platform.
- Screening result:
  **exited at screening**,
  failing the decision scope twice:
  pnpm's pnp keeps
  per-importer `node_modules` symlinks (a hybrid,
  not node_modules-free;
  [catalog-tighten.matrix README](../../package/dev-script/catalog-tighten.matrix/README.md)),
  and
  with `symlink: false` the repository's own record concludes PnP is viable only for PnP-aware
  toolchains ([doc/troubleshooting/pnpm-symlink-false-isolated.md](../troubleshooting/pnpm-symlink-false-isolated.md)),
  which this repository's tsc bundler-resolution,
  native rolldown/oxlint resolvers and containerized
  `npx` Playwright are not;
  both pnp variants were already exercised in the 21-scenario matrix.

### Candidate E: `@pnpm/esm-loader` standalone preload over the isolated layout

- Discovery source:
  [pnpm.io/store-loader][store-loader],
  standalone integration section.
- Base category:
  inspectable open-source local technology.
- Overlays:
  high-trust execution (manual preload),
  incumbent augmentation rather than replacement.
- Screening result:
  **exited at screening** on scope mismatch:
  the standalone loader is an
  integration API over an already-materialized layout,
  not a linker switch,
  so it delivers no
  layout or file-count benefit for this decision;
  recorded because it is the documented manual
  integration path and the likely vehicle for any future resolver-integration work.

## Frozen query schedule

Recorded before executing discovery.
Saturation rule per source class:
two consecutive complete pages
add no screening survivor,
or the provider reports exhaustion;
a cap reached earlier marks the class
blocked,
not saturated.

Source class 1,
package registry (npm):

1. `npm view @pnpm/esm-loader versions time --json`
2. fetch `https://registry.npmjs.org/@pnpm/esm-loader`
3. npm search `pnpm loader` (record count;
   leads only,
   not evidence)

Source class 2,
repository host (GitHub,
`gh`):

1. `gh release list --repo pnpm/pnpm --limit 30`
2. release notes of every 12.x release for `loaded`,
   `store loader`,
   `store-loader`,
   `CAS`,
   `global virtual store`,
   `GVS`,
   `esm-loader`
3. `gh search issues --repo pnpm/pnpm "loaded linker"` (open and closed)
4. `gh search issues --repo pnpm/pnpm "store loader"` (open and closed)
5. `gh search issues --repo pnpm/pnpm "store-manifest"` (open and closed)
6. `gh search issues --repo pnpm/pnpm "esm-loader"` (open and closed)
7. `gh search issues --repo pnpm/pnpm "nodeLinker loaded"` (open and closed)
8. `gh search prs --repo pnpm/pnpm "loaded linker"` (open and closed)
9. `gh repo clone pnpm/pnpm` at depth 1 (record revision),
   source read of the loader,
   manifest
   generation,
   `excluded` materialization,
   preload injection,
   Node floor gate,
   and interactions with
   pnpmfile hooks,
   overrides,
   `link:` deps,
   workspace packages,
   build approvals,
   `supportedArchitectures`
   and frozen-lockfile installs

Source class 3,
broader web:

1. `pnpm store loader`
2. `pnpm loaded linker`
3. `pnpm node_modules-free install`
4. `https://pnpm.io/store-loader` (all version tabs:
   12.x,
   11.x,
   10.x)
5. `https://pnpm.io/settings` (`nodeLinker`,
   `nodePackageMap`,
   `frozenStore`,
   GVS sections)
6. `https://pnpm.io/pnpr` (relation to the loaded mode)

Expansion round (de-duplicated,
from taxonomy already observed on the docs page;
frozen with the
schedule):
`global virtual store pnpm`,
`CAS install pnpm`,
`ERR_PNPM_LOADER_UNSUPPORTED_NODE`,
`enhanced-resolve pnpm loader`,
`nodePackageMap pnpm`,
`frozenStore pnpm`,
`.store-manifest.json`.

Source class 4,
this repository (incumbent,
parallel systems,
prior reports):

1. `rg "nodeLinker|node-linker"` across the repo (consumers,
   configs,
   docs)
2. `rg "node_modules"` across `package/`,
   `.github/`,
   container files,
   editor and linter configs
3. prior vet reports:
   `doc/audit/tech-monorepo-manager-vet-2026-09-16.md`,
   `doc/audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`
4. troubleshooting and handover records listed in the context section
5. `rg "TODO|FIXME|HACK|workaround"` filtered to pnpm/linker/symlink/node_modules mentions

Query execution completed 2026-10-18 through the delegated upstream audit and repository inventory.
Literal queries,
providers,
filters,
result counts and access dates are recorded in
`${HOME}/temp/agent/store-loader-research-20261018/upstream.md` (query-log section:
gh release,
search and api commands with exact-phrase totals;
npm registry views;
HTTP probes of the versioned
docs) and `repo-inventory.md` (repository source-class sweeps with uncapped counts).
The expansion-round terms were exercised inside the source audit and tracker searches;
they produced
taxonomy evidence (GVS force-enable,
`nodePackageMap` incompatibility,
`frozenStore` registration
behavior,
the Node-floor error code) but no additional candidates.
Terminal discovery result:
every source class saturated;
screening survivors were candidates A and B;
candidates C,
D and E exited at screening for the reasons recorded in the ledger.
No new candidate emerged from any class,
so the two-survivor rule is satisfied by the incumbent plus
the user-named subject,
with three recorded exits.

## Frozen criteria and weights

No user preference was stated,
so every criterion carries weight 1 and scoring only runs if at least
two finalists survive hard gates.
Criteria apply to every finalist at equal depth:

1. Workflow continuity:
   current mise,
   CI,
   editor and packaging workflows keep functioning
   without per-tool reengineering.
2. Migration effort:
   size and risk of the change set required to adopt.
3. Dependency-graph compatibility:
   coverage of this graph's native,
   napi,
   WASM,
   TypeScript-source and
   bundler-consumed surfaces.
4. Upstream maturity:
   experimental versus stable status,
   release history,
   maintainer trajectory,
   open defect surface.
5. Resource benefit:
   disk usage,
   node_modules file count,
   install and branch-switch time
   (motivation class recorded by prior modules-cache work).
6. Supply-chain posture:
   preservation of substitution,
   build-approval,
   release-age and trust policies.

Ratings use the 0 to 4 scale with evidence and confidence per the governing skill;
sensitivity reruns
move every medium/low-confidence rating and each weight one at a time.

## Execution manifest for the disposable-worktree experiment

Recorded before running the candidate command tree.

- Candidate and revision:
  pnpm 12.14.0 (loader embedded in the pnpm executable);
  repository HEAD
  `6889b9f273` checked out detached in `${HOME}/temp/agent/store-loader-research-20261018/cas-worktree`.
- Top-level commands:
  `pnpm install` (object-form then,
  if rejected,
  string-form `nodeLinker`),
  optional repeat with `excluded` entries for build-script packages;
  probe commands `node -e`,
  `node --import ./node_modules/.pnpm/.store-loader.mjs -e`,
  `pnpm exec node -e`,
  package `mise run`
  tasks,
  `node_modules/.bin` shims,
  one rolldown package build,
  one tsc type check.
- Statically discovered reachable tree:
  pnpm resolution against the public npm registry (HTTPS),
  `.pnpmfile.mjs` readPackage hook execution by pnpm,
  store blob clone-or-copy into the worktree,
  GVS materialization for excluded packages,
  approved build scripts per `allowBuilds` (sharp fetches
  prebuilt binaries),
  generation of `.store-manifest.json`,
  `.store-loader.mjs` and bin shims,
  `NODE_OPTIONS` injection into pnpm-run children.
- Files inspected to establish the tree:
  [pnpm-workspace.yaml](../../pnpm-workspace.yaml),
  [.pnpmfile.mjs](../../.pnpmfile.mjs),
  [pnpm.io/store-loader][store-loader].
- Expected reads:
  shared pnpm store under the mise-managed home,
  public registry metadata and
  tarballs.
  Expected writes:
  the disposable worktree only,
  plus pnpm's normal store metadata
  (project registration for store pruning;
  a stale registration may outlive worktree removal and is
  harmless until the next store prune).
  No network endpoints beyond the registry and sharp's
  release download.
  No credentials in the environment.
- Environment bounds:
  Fedora host,
  Btrfs,
  61 GiB RAM,
  935 GiB free;
  no container cap.
  Deviation
  justification:
  pnpm is the incumbent package manager performing its routine install class;
  the new
  code path is pnpm's own loader generation;
  isolation is provided by the disposable worktree;
  a
  memory/CPU cap would distort the install-time and layout measurements the evaluation needs.
- Success condition:
  install completes and probes classify the documented behavior on this graph.
  Failure and stop conditions:
  unsupported-configuration diagnostic (record,
  stop install attempts),
  install error (record,
  one variant retry with `excluded`),
  probe failures recorded not retried.
- Cleanup:
  `git worktree remove` after this report reaches a terminal outcome.

## Evidence log

Delegated evidence arrives in `${HOME}/temp/agent/store-loader-research-20261018/`
(`upstream.md`,
`repo-inventory.md`,
`experiment.md`).
Decisive records are folded into this report
with citations when the delegations return;
the scratch files are the raw backing.

### Upstream source audit (pnpm clone, tracker, registry, docs; 2026-10-18)

Raw backing:
`${HOME}/temp/agent/store-loader-research-20261018/upstream.md` (234 lines with the
query log) plus 38 evidence files in the sibling `evidence/` directory;
pnpm clone at
`${HOME}/temp/agent/pnpm-src-20261018`,
commit `07ab9fa881733154670f803fb4aa96dfd97e6cd9`
(2026-10-18).
The pnpm monorepo now holds `pnpm/` (the v12 Rust CLI,
loader under `pnpm/esm-loader/`,
docs under `pnpm/docs/`),
`pnpm11/` (legacy TypeScript line) and `pnpr/` (registry server).

Provenance and stability:

- Introduced in pnpm v12.10.0 (published 2026-10-06) by PR `pnpm/pnpm#16482`
  (`feat(install): add loaded linker with GVS exclusions`,
  @zkochan,
  merged 2026-10-01).
  The feature does not exist in pnpm 10 or 11:
  `https://pnpm.io/10.x/store-loader` returns 404
  while `/10.x/settings` returns 200,
  and `pnpm11/docs/settings/node-modules.md` lists `nodeLinker`
  types as isolated/hoisted/pnp only.
- v12.10.1 (same day) moved the generated files from the project root into `node_modules/.pnpm/`
  (a layout break between consecutive patch releases;
  stale files needed manual deletion) and fixed
  three severe day-one bugs:
  any package bundling its own `node_modules` (e.g. `npm`) stopped every
  Node.js process from starting;
  a `devEngines.runtime` shim recursion;
  startup overhead cut from
  67 ms to 18 ms per process.
- v12.11.0 added the `ERR_PNPM_LOADER_UNSUPPORTED_NODE` gate (`^24.18.0 || >=26.2.0`;
  Node 25.x is
  rejected;
  `pnpm/esm-loader/index.mjs:25-34`).
  Before the gate,
  old-Node failure was silent CJS
  breakage.
- No loaded-linker changes from v12.11.1 through v12.14.0 (2026-10-15,
  current).
  The feature was
  **12 days old** at audit time and experimental in every source (docs title,
  settings reference,
  release notes);
  the design task `pnpm/tasks#64` is OPEN,
  the config-surface rework
  `pnpm/tasks#72` (scope linker settings under the `nodeLinker` object) is OPEN,
  perf work `#77` is
  OPEN,
  and no stabilization roadmap or graduation milestone was found.
- Bus factor 1:
  `pnpm/esm-loader` 8/8 commits,
  `deps-restorer/src/cas.rs` 2/2,
  docs 7/8,
  all
  @zkochan;
  eight fix PRs landed in release week,
  all by the same author.

Architecture (clone citations at commit `07ab9fa8`):

- pnpm 12 is a Rust CLI;
  the loader is an esbuild bundle of `@pnpm/esm-loader` (npm 0.1.0
  2026-10-06,
  0.1.1 2026-10-15;
  sole runtime dep `enhanced-resolve`) embedded via `include_str!`
  (`pnpm/crates/deps-restorer/src/cas.rs:23`);
  the npm `pnpm@12.14.0` package has zero dependencies,
  only 14 platform `@pnpm/exe.*` optional dependencies.
- Registration uses Node's synchronous `module.registerHooks`;
  stored package roots are virtual
  `.pnpm-loader/<sha256(pkgId)>` directories that never exist on disk;
  every read is SHA-512
  verified (`store.mjs:174-177`,
  `ERR_PNPM_LOADER_INTEGRITY`);
  the manifest is built from the
  resolved lockfile (`cas.rs:45-79`).
- Format gate:
  `.mjs`/`.cjs`/`.json` plus extensionless JS bins only;
  **`.ts`,
  `.d.ts`,
  `.node`,
  `.wasm` throw `ERR_PNPM_LOADER_UNSUPPORTED_FORMAT`** (`index.mjs:110-129`;
  loader test
  `loader.test.mjs:178-181` proves native addons fail through both `import` and `createRequire`).
- Resolution is strictly the manifest dependency map with no `node_modules` fallback
  (`resolver.mjs:21-51`,
  `ERR_PNPM_LOADER_UNDECLARED_DEPENDENCY`;
  a `node_modules` bundled inside
  the importing package still wins,
  fix #16643).
- Build scripts:
  any `requires_build` snapshot not materialized hard-fails the install with
  `ERR_PNPM_CAS_REQUIRES_MATERIALIZATION` (`create_virtual_store/cas.rs:167-183`);
  **build denial
  does not exempt** (matches the experiment's `core-js` gate and the third-party `tesseract.js`
  observation).
- Workspace packages are not symlinked;
  importers become manifest `root` entries pointing at their
  real directories (`cas.rs:99-111`;
  test `cas_linker.rs:234-286` asserts every `node_modules`
  holds only dot-entries).
  Bare imports from workspace roots still pass through the strict
  dependency map.
- Preload injection sites:
  `pnpm exec` (`exec.rs:352`),
  run/test (`run/execution.rs:184`),
  recursive run,
  pipelines,
  install lifecycle scripts,
  and frozen-lockfile installs
  (`install_frozen_lockfile.rs:264`),
  all through `Config::add_cas_loader_env` appending
  `--import=file://.../.store-loader.mjs` to `NODE_OPTIONS`.
  **Bare `node` outside pnpm never
  receives the preload** (docs `store-loader.md:26`).
  CAS bin shims register the loader and
  re-export `NODE_OPTIONS` to children (`cas/bins.rs:112-120`);
  GVS-materialized bins get plain
  shims.
  `verify-deps-before-run` (default Install) regenerates deleted manifest/loader files on
  the next `pnpm exec` (`discovery.rs:99-110`,
  test `cas_linker.rs:340-355`).
- The mode force-enables the global virtual store (`config/src/cas.rs:10-13`) and validate-errors on
  `symlink: false` and `nodePackageMap` other than off (`create_virtual_store/cas.rs:185-199`).
  Patched dependencies and `node@runtime:` trees auto-materialize.
  `frozenStore` skips the
  store-prune project registration (`fast_path.rs:220-222`).
  Repeat installs re-verify the manifest
  instead of using the optimistic shortcut (`optimistic_repeat_install.rs`
  `config_blocks_fast_path`);
  generated files are machine-local.
- Frozen-lockfile CAS installs are supported end to end and tested (`cas_linker.rs:136-166`,
  `289-358`;
  `--prod --frozen-lockfile` prunes the manifest and `.bin`,
  `:154-166`).
- Supply-chain interaction:
  pnpmfile hooks and overrides act at resolution stage;
  the CAS manifest
  derives from the resolved lockfile;
  `rg` over all CAS code files found zero override/pnpmfile
  specific handling,
  i.e. no known incompatibility.
  `supportedArchitectures` platform-skipped
  snapshots are dropped from the manifest (`cas.rs:63-66`);
  the lockfile stays the portable artifact.
- Store-prune safety:
  loaded installs register the project durably because CAS blobs have no
  project hardlinks protecting them (`fast_path.rs:210-232`);
  prune walks registered projects and
  keeps manifest-referenced blobs and `resolution: "node"` roots
  (`store-dir/src/loader_references.rs:14-32`).
- Platforms:
  loader JS is platform-neutral;
  Windows shim surface (`.cmd`/`.ps1`,
  CRLF,
  BOM) is
  generated through `pnpm_cmd_shim`;
  the `cas_linker` suite runs on windows-2025,
  ubuntu-2404 and
  macos-latest in pnpm CI (`.github/workflows/pacquet-ci.yml:151-161`);
  a third-party macOS arm64
  field test corroborates loader function.
  **Zero Electron-specific handling exists anywhere in the
  loader or CAS code** (`rg 'electron'`:
  0 matches).
- `pnpr` is a separate experimental Rust registry server (0.1.0-alpha.15) with no relation to the
  loaded linker beyond a `frozenStore` incompatibility note.

Bug and ecosystem-compatibility surface:

- Zero open loaded-linker-specific issues in the pnpm tracker as of 2026-10-18 (exact-phrase API
  totals:
  `"loaded linker"` 0,
  `"nodeLinker loaded"` 0;
  phrase hits for `"store loader"`/
  `"esm-loader"` are GVS-hook or historical).
  Adjacent GVS resolve-hook bugs #16436 (ts-node on
  Node < 22.15) and #16979 (synthetic parentURL) were opened and closed same-day.
  The original
  feature request #5039 (2022) closed 2026-09-26,
  days before the PR merged.
- Upstream's own 30-workload compatibility matrix (`pnpm/tasks#74`,
  closed 2026-10-05,
  Linux x64,
  Node 26.10.0,
  full table in `evidence/tasks-74-full.txt`):
  pure compiler/unit-test corpora pass
  all-CAS (Svelte compiler 200 compilations with matching hashes,
  Astro plugin suites,
  Kysely Mocha
  units,
  Prisma coverage tools);
  **every Vite,
  Jest and native-Rolldown workload fails all-CAS**
  (runner fs-reads of virtual paths;
  native bindings);
  GVS opt-outs recover some but many still
  fail with their runner excluded (VueUse via `msw`,
  Nuxt via `@nuxt/kit`,
  SvelteKit six suites,
  Prisma foundation "workspace tsconfig not locatable",
  Svelte stores via `esm-env`,
  Turborepo
  conflicting Babel GVS contexts).
- Third-party field test (lilting.ch,
  2026-10-13,
  M4 macOS,
  pnpm 12.11.2;
  corroborating only):
  Astro+Vite dev and build fail even with `astro` excluded (Vite reads its own virtual
  `package.json` through fs;
  `Cannot find module 'astro/config'` inside Vite's resolver while
  `pnpm exec node` imports the same module fine);
  exclusion widening flipped the manifest from 381
  CAS packages to 134 (materialization snowball);
  `sharp` + `better-sqlite3` force exclusion;
  build-denied `tesseract.js` still required exclusion.

### Repository inventory (read-only sweep, 2026-10-18)

Raw backing:
`${HOME}/temp/agent/store-loader-research-20261018/repo-inventory.md`
(234 lines,
every claim cited path:line;
scale metrics measured in the main worktree).

Execution model (decisive for the preload mechanism):

- Nothing in the repository runs workloads through `pnpm run/test/exec`;
  repository policy
  (AGENTS.md rule CM3,
  `AGENTS.md:609-611`) bans `pnpm exec` for task work.
  pnpm appears only as
  `pnpm install`,
  `pnpm approve-builds sharp`,
  `pnpm install --lockfile-only`,
  `pnpm list`,
  `pnpm update` (`mise.no-env.toml:1077-1083,1224-1227,1280-1307`).
- All 1076 `*.unit.test.ts` files run as bare `node <file.ts>` spawned by the `run_test_files` var
  (`mise.no-env.toml:553-585,647-678`);
  270 task bodies run inline TypeScript through bare node;
  dev servers and dev-scripts (`ssg/aquati.cat`,
  `webapp-productivity/done`,
  file-enforcer,
  catalog-tighten,
  deps-update) are bare node on TS source.
- Native tooling resolves through `node_modules/.bin` shims injected into PATH by the generated
  mise `_.path` (root plus 47 package-level `.bin` dirs;
  `mise.toml:1342-1393`,
  generator
  `file-enforcer.config.ts:678-696`):
  `tsc` (the tsc-filter wrapper spawns bare `tsc --build`),
  `rolldown` (137 configs:
  88 node,
  40 browser,
  9 client),
  `oxlint` + `oxlint-tsgolint`,
  `changeset`,
  `build-css`,
  `h3`.
- Consequence:
  the loader preload would reach zero current workflows;
  adoption requires re-plumbing
  every node spawn and every shim-mediated native tool,
  and preload inheritance breaks at fixtures
  that deliberately clear `NODE_OPTIONS` (`package/module/test/src/rejection-fixture-process.ts:89`)
  and at containers that bind-mount the repo without the host store.

Native,
napi,
WASM and platform-binary surface (all would need `excluded` materialization):

- `satteri@0.10.5` + `@bruits/satteri-*` (9 platforms incl.
  `wasm32-wasi`;
  consumer
  `package/pi-plugin/auto-mode`),
  `@yuku-core/binding-*` (12),
  `sharp` + `@img/sharp-libvips-*` (25),
  `@rolldown/binding-*` (15),
  `@oxlint/binding-*` (18),
  `@oxlint-tsgolint/*` (6,
  Go),
  `@typescript/typescript-*` (12+;
  TypeScript 7 is a native launcher),
  `@tursodatabase/database-*`
  (4),
  `@mitata/counters`,
  `hb-subset-wasm`,
  `woff2-encode-wasm`,
  `wawoff2`,
  `@napi-rs/wasm-runtime`,
  `@emnapi/*` (lockfile counts in the inventory file,
  section 3.2).
- `allowBuilds` entries for `esbuild`,
  `@vscode/ripgrep`,
  `protobufjs` are vestigial (zero lockfile
  hits);
  `core-js@3.50.0` is present with builds refused,
  yet CAS demanded its materialization
  (experiment install gate).
  No `patchedDependencies` exist.
  `electron@44.7.0` postinstall never ran
  (no `dist/`,
  no `path.txt`);
  packaging downloads Electron zips via `@electron/packager`
  (`package/desktop-app/electron-infra/src/packager-target.ts:77-91`).
- Precedent:
  `doc/troubleshooting/napi-loader-bundled-out-of-package.md` documents that napi loaders
  anchor binding lookup at `createRequire(import.meta.url)`,
  exactly the anchoring the loader's
  virtual URLs break,
  with `NAPI_RS_NATIVE_LIBRARY_PATH` as the only path-based escape hatch.

Direct filesystem readers of the install layout (break by construction under virtualized paths):

- `require('electron/package.json')` and `require('electron')` binary-path resolution
  (`electron-infra/src/electron-version.ts:79`,
  `src/wayland-process.ts:82-86`);
- auto-mode `realpath(node_modules/@earendil-works/pi-coding-agent)` plus
  `require.resolve('typescript/package.json')` and spawned `node <ts>/bin/tsc`
  (`package/pi-plugin/auto-mode/native/build-source.ts:69-95`);
- hardcoded `node_modules/.pnpm` imports (`package/cli/forbidden-strings/bin/render-docs.mjs:9-12`,
  `package/linter/monochromatic-lint/mise.toml:221-224`);
- layout readers:
  catalog-tighten (`src/version-resolve.ts:205`,
  `src/store-probe.ts:6-9`,
  pnp fallback `src/version-pnp.ts`),
  `cli/fy` resolver walks (`src/resolve.ts:206-300`),
  build-tool/css package-resolver walks (`src/package-resolver.ts:5-80`);
- mutation-test recreates `node_modules` symlink farms from a baked container layer
  (`src/container/worktree.ts:123-232`,
  `runtime/Containerfile:6`);
- path-segment heuristics:
  test-diagnostic frame classification (`src/harness-frames.ts:90-109`),
  oxlint-plugin workspace-vs-external detection (`workspace-source-path.ts:4-22`),
  tsc-filter
  diagnostics suppression (`src/tsc-output-filter.ts:160-179`);
- the desktop IDE (Rust) launches the nearest `node_modules/typescript/bin/tsc` as its TS language
  server (`package/desktop-app/ide/src/language/config/typescript.rs:50,53,186`).

TypeScript topology:

- Shared config uses `moduleResolution: "bundler"`,
  `types: ["node"]`,
  `composite` +
  `declaration`,
  extends resolved through node_modules links
  (`package/config/typescript/tsconfig.options.json`;
  every package tsconfig extends
  `@monochromatic-dev/config-typescript/*` by package name);
  2709 `/ts` subpath import lines
  across 2251 files;
  per-package `tsc --build`,
  no project references.
- tsgo classifies files reached through a node_modules search path as external libraries even after
  realpath (`doc/planning/monorepo-manager-route-research/bazel-typescript.md:259-273`),
  which is
  what keeps `/ts` source imports free of TS6307/TS6059;
  a layout without node_modules-resolved
  paths removes the verified basis of that behavior.
- Node refuses type stripping under any `node_modules` path segment;
  the repository relies on both
  directions of that rule (workspace `/ts` realpath escapes,
  publish tooling strips TS exports:
  `file-enforcer.config.ts:1894-1975`,
  `package/config/pnpr/src/publish-plan.ts:128-255`).

Containers and CI:

- Playwright runs inside podman (`mcr.microsoft.com/playwright:v1.63.0-noble` image) over a repo
  bind mount without the host store;
  `npx playwright test` (npm's npx,
  not pnpm) and the webServer
  `node playwright/serve.ts` resolve `@playwright/test` and `h3` from the mounted repo
  `node_modules` (`mise.no-env.toml:592-607`,
  `playwright.Dockerfile:13`).
  A CAS/GVS layout on the
  host is invisible inside the container.
  Incidental drift noted,
  not fixed (decision-verb scope):
  image tag v1.63.0 versus lockfile 1.64.0;
  `docker-compose.yml` pins a stale `store/v10` volume.
- CI shape:
  every workflow installs with `mise exec node pnpm -- pnpm install --frozen-lockfile`
  (7 invocations across `npm-publish.yml`,
  `pnpr-publish.yml`,
  `npm-release.yml`,
  `logger-fuzz.yml`,
  `readonly-semantic-bridge.yml`,
  `toml-edit-fuzz.yml`);
  Node 26.12.0 via mise everywhere except
  `fs-id.yml` (setup-node 26.5.0);
  OS matrix ubuntu/macos-15/windows for semantic checks.
  No pnpm store or node_modules cache exists by measured decision
  (`doc/decision/ci-caching.md:75-83`,
  `doc/planning/depot-developer-pool-cache-research.md:191-202`).
- Publish workflows set `pnpm_config_dedupe_direct_deps: 'false'` job-wide because `pnpm pack`
  fails with `ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL` under the deduped layout
  (`.github/workflows/npm-release.yml:191-199`,
  `doc/troubleshooting/pnpm-pack-dedupe-direct-deps.md`).

Unaffected surfaces:
mise-managed non-npm tools (dprint native,
harper,
monochromatic-lint,
slopo,
caddy,
llama.cpp,
cargo/zig/gradle families),
scanner ignore-lists (path-pattern based),
logger
`.monochromatic` sinks and the file-enforcer staleness cache (they need only a `node_modules`
directory to exist;
the experiment observed the sink working),
and install-time supply-chain
machinery (`.pnpmfile.mjs` substitutions,
overrides,
`packageExtensions`,
stubs).

Scale metrics (main worktree,
measured 2026-10-18):

- 180 workspace packages;
  root `node_modules`:
  20 GB / 1,761,234 files,
  of which
  `node_modules/.monochromatic` logger JSONL is **19 GB / 1,718,094 files** (157 package-level log
  dirs add to this);
  the dependency tree proper (`node_modules/.pnpm`) is **1020 MB / 40,534 files /
  583 dirs / 1,103 symlinks**,
  mostly Btrfs reflinks (prior benchmark:
  about 372 KiB exclusive data).
- Recorded warm-install benchmarks under the isolated linker (pnpm 11.9.0,
  `doc/handover/pnpm-modules-cache-benchmark.md:285-413`):
  branch-switch checkout+install 939 ms,
  frozen install 727 ms,
  lockfile-only 231 ms,
  catalog-edit about 3.8 s.
- Debt signals (EL4):
  package sources show zero real TODO/FIXME/workaround hits about
  pnpm/linker/node_modules (17 raw hits,
  all base64 image false positives);
  `doc/` carries 45 hits
  across 22 files concentrated in 16 pnpm troubleshooting records;
  the live workaround inventory
  the linker change must preserve is configuration-shaped (26 blocklist stubs,
  4 `link:` shims,
  10 removal overrides,
  `packageExtensions`,
  dedupe-direct-deps CI override).

Prior mode trials (parallel implementations,
EVL):
`package/dev-script/catalog-tighten.matrix`
already spans 21 container scenarios (isolated with hoist on/off,
hoisted,
pnp,
pnp +
`symlink: false`,
renamed `modulesDir`,
relocated `virtualStoreDir`/`storeDir`,
stale-orphan
seeding);
`doc/troubleshooting/pnpm-symlink-false-isolated.md` proved `symlink: false` + isolated
builds an unresolvable store and that hoisted is the fallback for a symlink-free runnable tree;
`doc/audit/tech-monorepo-manager-vet-2026-09-16.md` and
`doc/audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md` contain no linker-relevant
conclusions;
`doc/audit/depot-linker-experiment-2026-10-16.md` is Rust lld-versus-mold despite the
name (recorded here to prevent future confusion).

### Experiment results (disposable worktree, 2026-10-18)

Raw backing:
`${HOME}/temp/agent/store-loader-research-20261018/experiment.md` (execution manifest,
484 lines) with artifacts `install-1.log`,
`install-2.log`,
`install-3.log`,
`probe-timings.md`
(19 timed probes via `probes.mjs`),
`rolldown-build-phase2.log`,
`rolldown-build-phase2-preload.log`,
`store-manifest.phase1.json`,
`main-nm-baseline.log`.
Worktree `cas-worktree`,
detached at `a1dfc3ab92a03ce6e647b1a5ccafab11e6cfd070`,
pnpm 12.14.0,
Node v26.12.0,
mise 2026.10.0.
Environment deviation:
the worktree lives on the encrypted volume,
so pnpm fell back from the home
store to `/var/mnt/encrypted/.pnpm-store/v11` (hard links cannot cross devices);
recorded in every
install log and harmless to the classification results.

Configuration acceptance and install gates:

- `nodeLinker: { type: loaded }` (object form) is accepted by pnpm 12.14.0;
  `node_modules/.modules.yaml` records `"nodeLinker": "loaded"`.
- Install attempt 1 failed in 0.433 s with `ERR_PNPM_CAS_REQUIRES_MATERIALIZATION`:
  `Package core-js needs materialization with the loaded linker`,
  even though `allowBuilds` maps
  `core-js: false`.
  The materialization gate fired despite disabled build scripts;
  recorded as an
  upstream-evidence wrinkle (lead-verified in `install-1.log`).
- With `excluded: [core-js]`,
  the warm install completed in 0.804 s (`Done in 770ms`);
  lockfile and supply-chain gates passed (`Lockfile passes supply-chain policies`).
- Phase 2 excluded 8 names (`core-js`,
  `rolldown`,
  `oxlint`,
  `oxlint-tsgolint`,
  `typescript`,
  `sharp`,
  `satteri`,
  `@bruits/satteri`):
  install 0.685 s,
  `+57` packages materialized into the GVS.

Layout facts (phase 1 unless noted;
lead-verified directly against the worktree):

- Root `node_modules` holds only `.bin` (32 shims),
  `.modules.yaml`,
  `.pnpm/` (`.store-loader.mjs` 115821 B,
  `.store-manifest.json` 6268011 B,
  `cas-bins/` with 51
  hashed JS bin wrappers,
  `lock.yaml`) and `.pnpm-workspace-state-v1.json`.
  No package directories and no `@monochromatic-dev/` symlinks exist anywhere in the project.
- Manifest v1 phase 1:
  762 entries = 181 `root` (project root,
  179 workspace directories,
  core-js GVS) + 581 CAS `files` entries;
  the lockfile carries 652 packages over 181 workspace
  projects.
  Phase 2:
  238 roots / 524 files / 58 `resolution: "node"` entries.
- The `link:` override shims appear as `root` entries pointing at their real `package/shim/*`
  directories (lead-verified in `store-manifest.phase1.json`:
  `node-domexception`,
  `proper-lockfile`,
  `readable-stream`,
  `ungap-structured-clone`),
  so the substitution mechanism survives at loader level.
- The GVS lives outside the project at `/var/mnt/encrypted/.pnpm-store/v11/links`
  (shared directory measured 535 M across projects).
- CAS bin shims self-register the loader and export
  `NODE_OPTIONS=--import=file://.../node_modules/.pnpm/.store-loader.mjs` to children;
  GVS-materialized bin shims do not register the loader.
- The logger sink directory `node_modules/.monochromatic` was still created and written by the
  unit-test probe run,
  so that first-party writer tolerates the layout.

Runtime probes (phase 1,
then phase 2 delta):

- Bare `node` third-party import:
  FAIL (`Cannot find package 'execa'`);
  with the `--import` preload:
  PASS;
  `pnpm exec node`:
  PASS via the NODE_OPTIONS mechanism.
  The repository's mise tasks spawn bare
  `node` and `.bin` shims directly,
  never `pnpm run/test/exec`,
  so no current workflow receives the
  preload (corroborated by the repository inventory).
- CJS (`yaml`),
  JSON (require and ESM `with { type: 'json' }` on `@changesets/config/schema.json`),
  cross-package `createRequire`,
  exports gating:
  PASS.
- Virtual URLs behave as documented:
  `import.meta.resolve('execa')` returns
  `.pnpm-loader/<sha256>/index.js`;
  `fs` reads of it fail ENOENT;
  the directory never exists.
- Workspace TypeScript unit test (`package/module/or-throw/src/non-nullish-or-throw.unit.test.ts`):
  bare node FAIL (`Cannot find package '@monochromatic-dev/module-test'`);
  under preload PASS
  (exit 0) once the built `dist` exists for the self-reference import;
  sibling `/ts` workspace imports
  and their third-party dependencies resolve under the loader because workspace roots are real
  directories and Node's own type stripping applies.
- Native/napi packages from CAS:
  FAIL (`rolldown`,
  `sharp`,
  `satteri` with
  `Cannot find native binding`;
  TypeScript 7's JS launcher ENOENTs reading its own virtual
  `package.json`).
  Under phase-2 exclusions all six import/CLI probes PASS
  (`tsc --version` 7.0.2,
  `rolldown` v1.2.13,
  `oxlint` 1.88.0,
  `sharp`,
  `satteri`).
  The loader also enforces declared-dependency edges
  (`Package package/ssg/aquati.cat does not declare @bruits/satteri`).
- `tsc --build` on `package/module/or-throw` (phase 2,
  typescript materialized):
  exit 2 with 83
  errors;
  TS2307 for every workspace `/ts` import and the shared config packages,
  cascading
  TS5097/TS2550 from the lost base tsconfig.
  tsc's own fs-based resolution sees neither CAS nor GVS.
- rolldown build on `package/module/async-iter` (phase 2):
  without preload,
  config resolution fails
  (`Cannot find package '@monochromatic-dev/config-rolldown'`);
  with NODE_OPTIONS preload the config
  loads but the build fails with Rust-side `TSCONFIG_ERROR` on
  `@monochromatic-dev/config-typescript/dom` plus a transitive napi package
  (`@yuku-core/binding-linux-x64-gnu` via `rolldown-plugin-dts`) whose `.node` blob cannot load from
  CAS.
  The napi exclusion tail is discovered at runtime,
  not statically.
- Bin shims:
  pure-JS bins PASS standalone (`browserslist` 4.29.3);
  napi bins FAIL in phase 1 and
  PASS in phase 2.

Scale deltas (measured):

- Main worktree `node_modules` under the isolated linker:
  20 G,
  1761060 files
  (`du`/`find`,
  wall 1 m 11.6 s for the count).
- CAS worktree `node_modules`:
  6.9 M / 96 files (phase 1),
  6.7 M / 97 files (phase 2).
- Warm-store CAS install under 1 s for the whole 181-project graph.
- Correction from the repository inventory (same day):
  of the main worktree's 20 GB / 1,761,234
  files,
  **19 GB / 1,718,094 files are first-party logger JSONL** under
  `node_modules/.monochromatic`,
  a retention concern orthogonal to the linker;
  the dependency tree
  proper (`node_modules/.pnpm`) is 1020 MB / 40,534 files / 583 dirs,
  mostly Btrfs reflinks with
  about 372 KiB exclusive data by prior benchmark,
  and recorded warm isolated-linker installs are
  already sub-second (frozen install 727 ms;
  branch-switch install about 0.93 s on pnpm 11.9.0,
  [doc/handover/pnpm-modules-cache-benchmark.md](../handover/pnpm-modules-cache-benchmark.md)).
  The honest benefit delta of the loaded layout is therefore about 1 GB and about 40,000
  project-local dependency files removed with negligible warm-install time gain;
  the 20 GB headline
  belongs to the logger sink,
  not to the linker.

Experiment conclusion (gate outcomes recorded in the Hard-gate outcomes section):
pure-JS runtime consumption works as documented today;
the type-check and bundle workflows are broken
under the loaded layout even with the toolchain materialized,
because fs-based resolvers outside
Node's loader hooks (tsc,
rolldown's Rust side) cannot see CAS or GVS packages,
and the napi
materialization tail is long and only discovered at runtime.

## Hard-gate outcomes

Candidate B (loaded linker),
final dispositions against the constraints frozen in the metadata block:

- Constraint 1 (mise-run workflows keep working):
  **FAIL,
  proven;
  eliminates the candidate**.
  Evidence:
  experiment probes h and j (`tsc --build` exit 2 with 83 errors on one small package
  even with `typescript` materialized;
  rolldown build fails on the Rust-side tsconfig lookup plus a
  hidden napi dependency even with preload and exclusions);
  repository inventory (zero workload
  invocations pass through `pnpm run/test/exec`;
  AGENTS.md rule CM3 bans `pnpm exec`;
  1076
  bare-node test files,
  270 inline TypeScript task bodies,
  137 rolldown configs,
  tool PATH
  injection through `.bin` shims);
  upstream source (bare node never receives the preload;
  stored
  `.ts`/`.d.ts` throw `ERR_PNPM_LOADER_UNSUPPORTED_FORMAT`;
  GVS bin shims do not register the
  loader);
  upstream compatibility matrix (every Vite/Jest/native-Rolldown workload fails all-CAS;
  Prisma-class "workspace tsconfig not locatable" failures under GVS opt-out).
- Constraint 2 (native/napi/WASM stay loadable and buildable):
  **conditional pass with recorded
  risk**.
  `excluded` materialization restored every probed native package,
  but the required set is
  not statically enumerable:
  `@yuku-core` surfaced only during a build,
  build-refused packages
  (`core-js` here,
  `tesseract.js` upstream) still force materialization,
  and exclusion widening
  snowballed 381 to 134 CAS packages in the third-party test.
  Runtime-discovered tails conflict
  with reproducible-CI expectations.
- Constraint 3 (supply-chain policy keeps effect):
  **PASS**.
  Resolution-stage mechanisms are
  untouched (manifest derives from the resolved lockfile;
  zero CAS-specific pnpmfile/override code
  paths in the source audit);
  experiment installs ran the policy checks
  (`Lockfile passes supply-chain policies`);
  `link:` shims resolve as manifest root entries
  (lead-verified against `store-manifest.phase1.json`).
- Constraint 4 (lockfile portability):
  **PASS**.
  The lockfile is retained unchanged by the mode;
  platform-skipped snapshots drop from the manifest (`cas.rs:63-66`);
  frozen-lockfile CAS installs
  are supported and tested upstream.
- Constraint 5 (CI frozen installs):
  **PASS at mechanism level,
  moot at workflow level**.
  Frozen
  CAS install paths exist and inject the preload (`install_frozen_lockfile.rs:264`;
  tests
  `cas_linker.rs:136-166,289-358`),
  and CI Node versions (26.12.0 via mise,
  26.5.0 in `fs-id.yml`)
  meet the floor;
  but every CI job proceeds to typecheck/test/build steps that hit the
  constraint-1 failure,
  so no workflow can adopt the mode.
- Constraint 6 (Electron and Playwright surfaces):
  **FAIL**.
  Playwright runs in a podman container
  bind-mounting only the repository:
  the host CAS store and GVS are invisible inside,
  the runner is
  npm's `npx` (no preload),
  and the image Node version is unverifiable locally (image absent;
  the
  tag already drifts from the lockfile,
  v1.63.0 versus 1.64.0).
  Electron metadata reads
  (`require('electron/package.json')`,
  `require('electron')` binary-path resolution) break by
  construction on virtual paths,
  and upstream has zero Electron-specific handling or tests.
  The
  packager operating on a staged dependency-free `dist/app` mitigates packaging only,
  not the build
  and test tooling around it.
- Constraint 7 (Node floor):
  **PASS with reservation**.
  Host and CI floors met;
  Node 25.x is
  rejected upstream,
  and mise tracks `latest` (currently 26.x),
  so a future Node 25-class skip
  release would fail loudly with `ERR_PNPM_LOADER_UNSUPPORTED_NODE` rather than silently;
  the
  Playwright container Node version is unknown and moot under constraint 6.
- Constraint 8 (isolation guarantees):
  **PASS,
  improved**.
  The loader enforces declared-dependency
  edges strictly (`ERR_PNPM_LOADER_UNDECLARED_DEPENDENCY`,
  no node_modules fallback;
  observed in
  the experiment as `Package package/ssg/aquati.cat does not declare @bruits/satteri`) and
  verifies SHA-512 on every stored read.

Net:
candidate B fails constraints 1 and 6 on proven evidence;
per the governing skill a hard-gate
failure removes the candidate,
and soft benefits cannot offset it.

Candidate A (incumbent isolated linker):
passes every constraint by measurement;
every workflow is
green under it today,
and its recorded costs (about 1 GB / about 40,000 project-local dependency
files,
mostly reflinked;
warm installs already sub-second) are the baseline the alternatives had to
beat.
Candidates C,
D and E exited at screening with the reasons recorded in the ledger.

## Scoring and sensitivity

Not executed.
Weighted scoring and the sensitivity matrix require at least two validated finalists;
only the incumbent survived hard gates,
and soft points cannot rescue a hard-gate failure.
The
frozen criteria and weights (six criteria,
weight 1 each) remain recorded for any future
re-evaluation triggered below.

## Ranking and recommendation

Complete ledger order with the reason for each adjacent ordering:

1. **Candidate A,
   keep `nodeLinker: isolated` (recommended)**:
   the only candidate passing every
   frozen hard constraint;
   measured costs are bounded (about 1 GB / about 40,000 project-local
   dependency files,
   mostly Btrfs reflinks;
   warm installs already sub-second).
2. Candidate E (standalone `@pnpm/esm-loader`) ranks below A because it changes nothing for this
   decision:
   packages stay materialized,
   so no layout or file-count benefit exists;
   it exited at
   screening on scope mismatch.
   A beats E because A is the working status quo and E is the same
   status quo plus a manually maintained preload.
3. Candidate D (pnp) ranks below E because it fails the decision scope twice:
   pnpm's pnp keeps
   per-importer `node_modules` symlinks (not node_modules-free),
   and with `symlink: false` the
   repository's own troubleshooting record and 21-scenario matrix show the fs-resolver blindness
   class that also sank candidate B,
   with no compensating benefit.
   E beats D because E at least
   preserves the layout guarantees while D trades them away for a resolver model this toolchain
   does not speak.
4. Candidate C (hoisted) ranks below D because it additionally fails constraint 8 by design:
   phantom dependencies become resolvable,
   the exact guarantee this repository bought with
   `hoist: false` + isolated after a documented napi incident.
   D beats C because D preserves
   strict declared-dependency resolution while C abandons it.
5. Candidate B (loaded linker) ranks last despite the largest measured upside (project-local
   dependency state about 1 GB / about 40,000 files down to about 7 MB / about 100 files;
   strict
   declared-dependency enforcement;
   per-read SHA-512 verification;
   sub-second warm installs),
   because it fails constraints 1 and 6 on proven evidence,
   its exclusion tail is
   runtime-discovered rather than statically enumerable,
   and it is a 12-day-old experimental
   feature with
   a same-week layout break,
   an open config-schema rework (`pnpm/tasks#72`),
   no stabilization
   roadmap,
   and bus factor 1.
   C beats B because C at least runs every current workflow,
   while B
   breaks the typecheck/build/container surfaces outright.

**Recommendation:
do not migrate.
Keep `nodeLinker: isolated` with `hoist: false`.**
A runtime-JS-only workload could adopt the loaded linker today (the experiment proved the pure-JS
path works exactly as documented),
but this repository has no such workload:
every consumer surface
(type check,
builds,
type-aware lint,
bare-node tests,
containers,
packaging) is precisely what the
mode cannot serve yet.

Pros and cons of the recommended path:

- Keep isolated.
  Pros:
  all workflows stay green;
  strictness guarantees preserved;
  zero migration
  work;
  supply-chain machinery (`.pnpmfile`,
  overrides,
  `allowBuilds`,
  `minimumReleaseAge`,
  `trustPolicy`) untouched;
  multi-OS CI unaffected.
  Cons:
  about 1 GB / about 40,000 project-local
  dependency files remain (mostly reflinked,
  about 372 KiB exclusive);
  no per-read runtime hash
  verification;
  the separate 19 GB logger-JSONL volume under `node_modules/.monochromatic` stays a
  retention-policy problem for the logger,
  which the linker cannot solve.

Revisit triggers (any one reopens this evaluation with a fresh report):

- `pnpm/tasks#64` (loaded-linker feature task) closes with a stabilization or graduation statement;
- `pnpm/tasks#72` lands and the `nodeLinker` object schema stops churning;
- upstream ships resolver integration for fs-based tools (type checkers and native bundlers consume
  the store manifest or GVS roots),
  or the loader gains `.d.ts`/`.ts` support;
- build-refused packages (`allowBuilds: false`,
  e.g. `core-js`) stop forcing materialization;
- Electron handling and a container story (store visibility plus preload injection for
  Playwright-in-podman) exist and are tested upstream;
- this repository gains a runtime-JS-only deployable that could pilot the mode in isolation.

Terminal outcome:
report complete with a recommendation (retain incumbent);
no discovery or
validation blockage;
scoring not applicable with a single surviving finalist.
The adoption boundary
was respected:
no product,
configuration,
dependency or decision-record mutation occurred during
the evaluation,
and adoption of any candidate waits for a separate authorizing request.

Cleanup record:
the disposable worktree
`${HOME}/temp/agent/store-loader-research-20261018/cas-worktree` (detached
`a1dfc3ab92a03ce6e647b1a5ccafab11e6cfd070`) was removed with `git worktree remove --force` on
2026-10-18 after path verification;
all probe outputs,
install logs,
timed re-runs and the phase-1
manifest copy are preserved in the scratch directory backing this report.
The shared pnpm store
retains only a stale project registration whose manifest no longer exists;
pruning tolerates it
because registrations are consumed through the manifest (`store-dir/src/loader_references.rs:14-32`).

Incidental observations outside this evaluation's scope (recorded,
not fixed;
the request was a
decision verb):
Playwright image tag v1.63.0 versus lockfile `@playwright/test` 1.64.0 drift
(`playwright.Dockerfile:11-13`);
`docker-compose.yml` pins a stale `pnpm-store` volume at
`store/v10` while the store is v11 and references an undefined `ai-proxy:start` task;
`allowBuilds`
entries for `esbuild`,
`@vscode/ripgrep` and `protobufjs` are vestigial (zero lockfile hits);
the
19 GB logger JSONL volume under `node_modules/.monochromatic` deserves its own retention discussion.

[store-loader]: https://pnpm.io/store-loader
