# Bazel 9 migration design for the TypeScript side

## Status and evidence rules

- Status:
   design research only;
   nothing was installed,
   built,
   or run with Bazel.
- Date:
   2026-09-16.
- Label convention:
   **verified** means a cited repository path,
   ruleset `path:line`,
   or a command whose output is quoted;
   **unverified** means inference from reading source without executing the integration,
   and says why.
- Ruleset clones under `~/temp/agent/`
   (all `--depth 1`, `main` as of the date):
  - `rules_js-2026-09-16` at `3f5022d`
     (latest release v3.4.1, 2026-08-21, from `gh release list`);
  - `rules_ts-2026-09-16` at `bfbf401`
     (latest release v3.10.1);
  - `rules_lint-2026-09-16` at `4cb8c45`
     (latest release v2.9.0);
  - `bazel-2026-09-16` at `8e90a0d`;
  - `bazel-lib-2026-09-16` at `7ca5f6d`;
  - `typescript-go-2026-09-16` at `89d5d5b2` (cloned for this document).
- Repository measurements come from scratch scripts
   (`lockstats.ts`, `bareimports.ts`, `misecensus.ts`, `testcensus.ts`, `targetcount.ts`)
   in the session scratchpad;
   their outputs are quoted where used.
  Regex-based census numbers are marked as heuristics.

## Summary

No single finding proves the Bazel route impossible.
Three findings need a prototype before the route can be trusted,
because each one either breaks analysis for part of the dependency graph
or forces the most expensive action (oxlint with the readonly rule) out of the sandbox:

- pnpm `overrides` and `.pnpmfile.mjs` substitutions create five third-party-to-workspace `link:` edges
   that rules_js appears to reference by package-store targets it never generates;
- native tools (tsgo, tsgolint, `tsgo --api` inside the readonly rule) resolve real paths outside the sandbox,
   while the Node side of the same oxlint process is patched to stay inside it;
- the readonly rule's persistent cache cannot survive a sandboxed action,
   so every Bazel lint rerun is a cold run.

The details and the full candidate list are in "Yikes candidates".

## Area 1: dependency install

### Today

- Package manager:
   pnpm 12.4.2
   (`mise.lock`, `[[tools.pnpm]] version = "12.4.2"`),
   Node 26.8.2 (`mise.lock`, `[[tools.node]]`).
- `pnpm-workspace.yaml`:
   `packages: ['package/*/*']` (lines 367 to 368),
   `nodeLinker: isolated` (173),
   `hoist: false` (163),
   `catalogMode: strict` with a `catalog:` block (15, 160),
   `allowBuilds` with only `sharp: true` (1 to 6),
   `overrides` including four `link:package/shim/*` substitutions (229, 297, 313, 334),
   `packageExtensions` for `mitata` (355),
   `supportedArchitectures` for darwin and linux, x64 and arm64, glibc and musl (389).
- `.pnpmfile.mjs` `readPackage` hook (line 190) rewrites blocked transitive dependencies to
   `workspace:@monochromatic-dev/stub-throwing@*` or `stub-silent` (`STUB_SPECIFIER`, line 90),
   driven by `.pnpmfile.policies.json` (for example `js-yaml` at line 56 and `sax` at line 80).
- No `.npmrc` at the root (`ls --all` of the repository root).
- `prepare:pnpm:install` runs `pnpm install` (`mise.no-env.toml:1012`).

### Lockfile facts (verified by `lockstats.ts` and `rg`)

- `lockfileVersion: '9.0'` (`pnpm-lock.yaml:1`).
- Single YAML document:
   `rg '^---' pnpm-lock.yaml` found no separator.
- 156 importers
   (root plus 155 under `package/`),
   772 `packages` entries,
   772 `snapshots`.
- No `patchedDependencies` key.
- `pnpmfileChecksum` present (`pnpm-lock.yaml:358`),
   so the hook output is baked into the lockfile.
- Importer edges:
   706 `link:` (workspace) and 476 registry.
- 160 packages carry `os` or `cpu` constraints;
   43 carry `hasBin`;
   every registry package has an `integrity`.
- Five snapshot dependencies point at workspace directories with no importer linking that directory under the same name:
  - `cosmiconfig@9.0.2(typescript@7.0.2)` to `js-yaml: link:package/stub/throwing` (`pnpm-lock.yaml:9217`);
  - `xml2js@0.6.2` to `sax: link:package/stub/throwing` (`:11120`);
  - `@earendil-works/pi-coding-agent@0.85.1` to `proper-lockfile: link:package/shim/proper-lockfile` (`:8027`);
  - `mdast-util-to-hast@13.2.1` and `rehype-autolink-headings@7.1.0` to
     `@ungap/structured-clone: link:package/shim/ungap-structured-clone` (`:10025`, `:10554`).

### How rules_js would do it

- `npm.npm_translate_lock(name = "npm", pnpm_lock = "//:pnpm-lock.yaml")` in `MODULE.bazel`,
   with `update_pnpm_lock` left off so pnpm keeps owning resolution outside Bazel.
- Lockfile parsing:
  - YAML is converted with `yq eval-all '. as $d ireduce (null; $d)'`,
     keeping the last document of a multi-document lockfile
     (verified: `npm/private/npm_translate_lock_state.bzl:414-434`);
  - only lockfile version 9.0 is accepted
     (verified: `npm/private/pnpm.bzl:338-360`),
     which this lockfile is;
  - pnpm 12's lockfile is covered by the `e2e/pnpm_lockfiles/v120` case on `main`
     (verified: `e2e/pnpm_lockfiles/v120/pnpm-lock.yaml`),
     but that case is absent at tag v3.4.1
     (verified: `gh api repos/aspect-build/rules_js/contents/e2e/pnpm_lockfiles?ref=v3.4.1` lists only `v101`, `v110`,
     `v90`).
- Catalogs:
   no special handling exists or is needed,
   because importer entries already carry the resolved `version`
   (verified: `pnpm.bzl:143-164` reads `attributes["version"]`;
   `rg 'catalog|workspace:' npm/private --glob '*.bzl'` returned nothing).
- `workspace:*`:
   lockfile records these as `link:` versions,
   converted to link keys
   (verified: `pnpm.bzl:148-151`).
- `patchedDependencies`:
   supported for the pnpm 11+ hash-only form with paths read from `pnpm-workspace.yaml`
   (verified: `npm_translate_lock_state.bzl:479-498`);
   not used here.
- Build scripts:
   `allowBuilds` is read and its `true` keys become `only_built_dependencies`
   (verified: `npm_translate_lock_state.bzl:501-508`);
   lifecycle hooks run only for names in that list
   (verified: `npm_translate_lock_helpers.bzl:596`),
   and a missing list is an error
   (verified: `npm_translate_lock_helpers.bzl:801-821`).
  So only `sharp` gets a lifecycle action.
- `.pnpmfile.mjs`:
   rules_js never reads it
   (verified: `rg --ignore-case pnpmfile` over the clone returned nothing);
   its effects arrive through the lockfile only.
- `.npmrc`:
   an undeclared `.npmrc` beside the lockfile is an error
   (verified: `npm_translate_lock_state.bzl:192-198`);
   none exists.
- Nested `node_modules` from the pnpm install must be hidden from Bazel with
   `ignore_directories(["**/node_modules"])` in `REPO.bazel`
   (verified: `npm/private/npm_translate_lock.bzl:323-331`;
   Bazel `docs/versions/9.1.0/rules/lib/globals/repo.mdx:12-15`).
- libc:
   rules_js maps `os` and `cpu` to constraints
   (verified: `pnpm.bzl:55-98`)
   and has no libc handling
   (verified: `rg --ignore-case 'libc|musl' npm/private --glob '*.bzl'` matched only pnpm binary checksums in
   `versions.bzl`),
   so glibc and musl variants of native bindings both link on Linux.
  Unverified whether any binding misloads because of that.
- Native pnpm 12 executables for `bazel run @pnpm` exist on `main`
   (verified: `npm/private/pnpm_repository.bzl:27-31`)
   but not at v3.4.1
   (verified: `gh api ...pnpm_repository.bzl?ref=v3.4.1` shows no native branch).
  Only needed if Bazel runs pnpm.

### Target counts

- External repositories:
   two per registry package,
   `npm__<pkg>__<ver>` and `...__links`
   (verified: `npm/private/npm_import.bzl:1263`, `:1293-1296`; `utils.bzl:409`),
   so 1,544 repositories for 772 packages.
- BUILD files:
   `npm_link_all_packages()` may only be called in the root package or an importer package
   (verified: `npm_translate_lock_generate.bzl:210-222`),
   and each importer that links anything needs one:
   up to 156 `BUILD.bazel` files for linking alone.
- Link targets:
   one `:node_modules/<name>` per importer edge,
   1,182 in total (706 workspace plus 476 registry).
- First-party package targets:
   every workspace directory that another importer links needs a target named by `npm_package_target_name`,
   default `pkg`
   (verified: `npm_translate_lock.bzl:211-217`, `npm_translate_lock_generate.bzl:360-363`):
   78 directories (`targetcount.ts`).

### Risk found: substituted workspace links without an importer alias

- First-party package stores are generated only from importer `link:` edges and `directory` resolutions
   (verified: `npm_translate_lock_generate.bzl:88-181`, stores emitted at `:354-391`).
- A `link:` key's store name is `<alias>@0.0.0`
   (verified: `utils.bzl:96-103`).
- Third-party packages reference their dependencies' stores by that name
   (verified: `npm_import.bzl:741-745` and `:783-786`).
- No importer links `package/stub/throwing`,
   `package/shim/proper-lockfile`,
   or `package/shim/ungap-structured-clone`
   (verified: `targetcount.ts` output "importer aliases: none" for all five edges).
- Inference, unverified by execution:
   `cosmiconfig`, `xml2js`, `pi-coding-agent`, `mdast-util-to-hast`, and `rehype-autolink-headings`
   would reference `.aspect_rules_js/node_modules/js-yaml@0.0.0` and similar targets that do not exist,
   failing analysis for anything depending on stylelint,
   `@homebridge/dbus-native`,
   the pi packages,
   or the aquati.cat markdown pipeline.
- `npm_replace_package` cannot paper over it:
   it replaces only packages present in the lockfile's `packages`
   (verified: `npm/extensions.bzl:74-82`, `npm_translate_lock_helpers.bzl:500-572`),
   and these names are absent there.
- Candidate fixes, all unprototyped:
   root `devDependencies` aliases such as `"proper-lockfile": "link:package/shim/proper-lockfile"`;
   for `stub/throwing`,
   which serves two aliases while stores are keyed by directory (`npm_translate_lock_generate.bzl:143-181`),
   one stub directory per alias;
   or a rules_js patch.
  No upstream issue was found (`gh search issues --repo aspect-build/rules_js 'override link'` and `'pnpmfile'`).

## Area 2: workspace `/ts` source imports

### Today

- 91 of 155 packages export `"./ts": "./src/index.ts"` (`lockstats.ts`);
   `@monochromatic-dev/config-rolldown` uses `./.ts`, `./.node.ts`, `./.client.ts`
   (`package/config/rolldown/package.json`).
- 1,594 files contain an `@monochromatic-dev/<name>/ts` import (`rg --count-matches` census, heuristic;
   the decision doc counted 1,789 sites on 2026-09-06).
- Bare specifiers still resolve through each package's `"."` export,
   which is built output for most packages:
   181 bare self-reference import statements (mostly tests importing their own package),
   and 28 cross-package bare import statements resolving to `dist/final/...`,
   16 of them to `@monochromatic-dev/module-logger`,
   whose `"."` export is `./dist/final/node/index.mjs` (`package/module/logger/package.json`) (`bareimports.ts`).
  The `claude-code-plugin-source` subpath imports (17 statements) resolve to `src/*.ts` instead.
- Package-internal `imports` also point at source,
   for example `#default-sinks` to `./src/default-sinks.node.ts` (`package/module/logger/package.json`).
- The decision and its measured costs:
   `doc/decision/workspace-ts-source-imports.md`.
  It relies on pnpm links resolving to real paths outside `node_modules`
   so Node type-strips sibling `.ts` ("Node runs it").

### How Bazel would do it

- Each linked package gets
`js_library(name = "pkg", srcs = ["package.json"], types = glob(["src/**/*.ts"]), deps = [":node_modules/..."])`.
  `js_library` copies sources to the output tree
   (verified: `js/private/js_library.bzl:43-45`, `:156-158`).
  `.ts` files in `srcs` are classified as sources, not types;
   only `.d.ts`, `.json`, and directories count as types
   (verified: `js_library.bzl:160-179`);
   the `types` attribute forces any file to be a type
   (verified: `js_library.bzl:79-91`, `:204-213`).
- A first-party package store backed by `JsInfo` is a directory symlink to the target's package directory in
`bazel-bin`,
   and carries the target's transitive sources and types
   (verified: `npm/private/npm_package_store.bzl:320-340`);
   the store also links the package's own dependencies
   (verified: `npm_package_store.bzl:342-362`, issue #2915).
  So `@monochromatic-dev/x/ts` resolves through `bazel-bin/.../node_modules/@monochromatic-dev/x/package.json` exports
   to `bazel-bin/package/<cat>/x/src/index.ts`.
- Node runtime (js_test, js_run_binary):
   the launcher runs `node --require <launcher> ... -- <entry>`
   (verified: `js/private/js_binary.sh.tpl:281-308`);
   Node patches for `lstat`, `readlink`, `realpath`, `readdir`, `opendir` keep resolution inside the sandbox
   (verified: `js/private/js_binary.bzl:147-159`),
   but ESM imports skip those patches
   (verified: `README.md:44-45`, `js_binary.bzl:212-227`).
  Unverified:
   whether resolved sibling paths ever keep a `/node_modules/` segment,
   which would trip Node's refusal to type-strip `.ts` under `node_modules`.
  By the symlink layout above the realpath lands in `bazel-bin/package/...`,
   so the expectation is that it works;
   no rules_js example or test uses a `.ts` entry point
   (verified: `rg 'entry_point = "[^"]*\.m?ts"'` over the clone returned nothing).
- TypeScript (tsgo):
   see Area 3.
  Files reached through a `node_modules` search are classified as external library files
   even after realpath
   (verified: `typescript-go internal/compiler/fileloader.go:880-902`, `filesparser.go:474-476` and `:539-541`),
   and external library files are excluded from the composite file-list check and the rootDir check
   (verified: `internal/compiler/emitter.go:483-486`, `program.go:1046-1063`, `program.go:1728-1735`).
  This is why `/ts` imports do not raise TS6307 or TS6059 today,
   and it carries over under Bazel as long as imports go through `node_modules`.
- Documentation gap:
   rules_ts troubleshooting still tells users to verify a `.d.ts` exists for dependencies
   (verified: `rules_ts docs/troubleshooting.md:96`).
  The `types` route above is source-derived,
   not a documented pattern.

## Area 3: type checking

### Today

- `lint:types` runs `task-util tsc-filter --build`
   (`mise.no-env.toml:605`),
   which deletes `.tsbuildinfo` caches and filters `node_modules` diagnostics
   (`package/dev-script/task-util/src/tsc-filter.ts`, header and `removeStaleBuildInfo`),
   adding `--singleThreaded` under `TSC_SINGLE_THREADED`
   (`package/dev-script/task-util/src/tsc-args.ts:32-53`).
- TypeScript is 7.0.2 native
   (`pnpm-lock.yaml` importer `typescript` resolves 7.0.2;
   the npm package's `lib/tsc.js` execs the native binary through `process.execve` or `execFileSync`,
   verified by reading `node_modules/.pnpm/typescript@7.0.2/node_modules/typescript/lib/tsc.js`).
- Shared options (`package/config/typescript/tsconfig.options.json`):
   `include` uses `${configDir}/src/**/*.ts` and `${configDir}/*.config.ts`;
   `noEmit: true`,
   `composite: true`,
   `declaration: true`,
   `isolatedDeclarations: true`,
   `allowImportingTsExtensions: true`,
   `module: preserve`,
   `moduleResolution: bundler`,
   `tsBuildInfoFile: ${configDir}/.cache/typescript/tsconfig.tsbuildinfo`,
   `paths` for `@/*` and `@_/*`,
   `types: ["node"]`.
- 152 package `tsconfig.json` files,
   all extending `@monochromatic-dev/config-typescript` subpaths except the config package itself (`targetcount.ts`).
- oxlint already type-checks:
   `options.typeAware: true` and `options.typeCheck: true`
   (`package/config/oxlint/src/config-base.ts`),
   and the installed schema describes `typeCheck` as
   "experimental type checking (includes TypeScript compiler diagnostics)"
   (verified: `oxlint@1.81.0/configuration_schema.json:17058-17061`).

### How Bazel would do it

- rules_ts supports TypeScript 7 native:
   `NATIVE_TYPESCRIPT_VERSIONS` contains `7.0.2`
   (verified: `rules_ts ts/private/versions.bzl:10`, `:36`),
   platform packages are downloaded and linked
   (verified: `ts/private/npm_repositories.bzl:62-125`),
   and the options validator spawns `tsc --showConfig` instead of requiring the JS API
   (verified: `ts/private/ts_project_options_validator.cjs:4-50`).
- Version source:
   `ts_version_from = "//:package.json"` reads the `devDependencies.typescript` string literally
   (verified: `npm_repositories.bzl:15-26`)
   and would read `"catalog:"`,
   failing the mirror lookup at `:44-49`.
  Use `ts_version = "7.0.2"` or the rules_js `resolved.json` form accepted at `:17-19`.
- Per package:
   `ts_project(name = "types", srcs = glob(["src/**/*.ts", "*.config.ts"]), no_emit = True, tsconfig = ":tsconfig", deps = [":node_modules/@types/node", ":node_modules/@monochromatic-dev/..."])`.
  - `no_emit` produces no outputs,
     so the root-module transpiler requirement at `ts/private/ts_project.bzl:211-212` does not fire
     (verified).
  - A no-output target runs one `TsProjectCheck` action with `--noEmit`
     (verified: `ts_project.bzl:225-262`).
  - Inputs are the target's own sources,
     the tsconfig chain,
     and dependencies' `transitive_types` plus `npm_sources`,
     never dependencies' sources
     (verified: `ts_project.bzl:17-29`, `:81-94`).
     Sibling `.ts` therefore reaches tsgo only through the `types` attribute from Area 2.
  - `--project` and `--rootDir` are always passed
     (verified: `ts_project.bzl:160-166`).
  - `tsconfig` is a `ts_config` target whose `deps` include
     `:node_modules/@monochromatic-dev/config-typescript`,
     so `extends` resolves through a link.
- Composite and build info:
   the `composite` attribute makes rules_ts expect a `.tsbuildinfo` and one `.d.ts` per source
   (verified: `ts/defs.bzl`, `composite` argument doc).
  Unverified how the validator treats inherited `composite: true` with `noEmit: true`,
   and where tsgo writes `tsBuildInfoFile` under `--project --noEmit`.
  The low-risk design is a Bazel-only tsconfig layer turning off `composite` and `incremental`.
- `isolatedDeclarations`:
   required only for `isolated_typecheck`
   (verified: `ts/private/ts_lib.bzl:91-103`);
   a `no_emit` check does not need it,
   and the repository already sets it.
- Alternative:
   drop `ts_project` and rely on oxlint `typeCheck` inside the lint aspect.
  That removes a duplicate tsgo program per package but depends on an option oxlint labels experimental.
- Hermeticity:
   see Yikes candidate 2;
   tsgo's realpath is native
   (verified: `typescript-go internal/vfs/osvfs/os.go:154-173`)
   and used by module resolution
   (verified: `internal/module/resolver.go:1881-1886`).

## Area 4: lint

### Today

- `lint:oxlint` template:
   builds `config-oxlint` when stale,
   then runs `task-util oxlint-wrapper` from the package directory
   (`mise.no-env.toml:589`;
   staleness logic in `dispatch_workspace_node`, `:357`).
- Root `oxlint.config.ts` spreads `@monochromatic-dev/config-oxlint`,
   whose default export is built `dist/final/node/index.mjs`
   pointing `jsPlugins` at five co-located sidecar bundles
   (`package/config/oxlint/src/index.node.ts`, `rolldown.node.config.ts`).
- `config-base.ts`:
   `typeAware`, `typeCheck`, `denyWarnings`, and `ignorePatterns` (including `**/bundle`, `**/fixture/**`).
- The wrapper forces `--format=default`,
   honours `OXLINT_THREADS`,
   and loops `--fix` to a fixpoint
   (`package/dev-script/task-util/src/oxlint-wrapper.ts`).
- Per-package cwd is load-bearing:
   type-aware mode finds the nearest `tsconfig.json` upward from cwd,
   and `ignorePatterns` resolve from cwd
   (`doc/troubleshooting/oxlint.md`, Bug 1 and the `ignorePatterns` report).
- `prefer-readonly-parameter-type` (`package/oxlint-plugin/prefer-readonly-parameter-type`):
  - uses the TypeScript 7 native API
     (`typescript/unstable/sync`,
     `typescript/unstable/ast` imports across `src/prefer-readonly-parameter-types/`;
     renamed `typescript/sync` and `typescript/ast` in 7.1.0-beta,
     2026-10-20),
     spawning `tsgo --api` children
     (`typescript-sync-native-shutdown.ts:137-147`);
  - analyzes workspace callees live from sibling source,
     classifying "workspace source" as any real path without `/node_modules/`
     (`workspace-source-path.ts:1-40`),
     and installed packages by the last `/node_modules/` segment
     (`installed-package-identity.ts:66-116`);
  - reads the nearest `pnpm-lock.yaml` for package eligibility
     (`lockfile-package-eligibility.ts:36-56`);
  - persists summaries to `<nearest pnpm-lock.yaml ancestor>/node_modules/.cache/prefer-readonly-parameter-type`
     (`effect-summary-cache-identity.ts:223-271`),
     silently skipping failed writes
     (`effect-summary-persistent-cache.ts:336-379`);
  - has no environment override for the cache root:
     `rg 'process\.env'` over its non-test sources returned nothing,
     and `cacheRootOverride` is a programmatic test parameter.
- Measured costs with source siblings (`doc/decision/workspace-ts-source-imports.md`, "Context"):
   warm per package 1.2 to 3.1 s (`mcp/mvm`, `file-enforcer`, `pi-plugin/auto-mode`);
   cold per package 7.4 to 22.0 s for the same three;
   cold whole repository 261 to 305 s with the rule on,
   13 to 15 s with it off;
   warm whole repository 63 to 105 s.

### rules_lint aspect API

- rules_lint ships no oxlint integration
   (verified: `rg --ignore-case oxlint` over the clone returned nothing;
   `gh issue view 445 --repo aspect-build/rules_lint` is open with 0 comments).
- Helpers for a custom aspect
   (verified: `lint/private/lint_aspect.bzl`):
   `should_visit` (`:31-53`, honours `no-lint`),
   `output_files` (`:57-129`, human and machine outputs plus exit codes, `_validation` group),
   `patch_and_output_files` (`:147-175`),
   `filter_srcs` (`:177-181`, source files from `srcs`),
   `noop_lint_action` (`:183-217`),
   `parse_to_sarif_action` (`:222`).
- Pattern to copy:
   `lint/eslint.bzl`.
  Inputs are the bin-tree copy of `srcs` plus `JsInfo` sources, types, and npm sources of `deps`,
   the rule's `tsconfig`,
   and config files
   (verified: `eslint.bzl:63-91`, `lint/private/js_linter_inputs.bzl:15-45`).
  It runs the linter twice per target,
   once for human output and once for the machine report
   (verified: `eslint.bzl:208-228`).
- Fix mode:
   `run_patcher` copies the files to diff into a writable tree,
   runs the linter,
   and emits a patch
   (verified: `lint/private/patcher.mjs:1-8`, `:52-80`);
   applying patches is left to Aspect CLI or a wrapper script
   (verified: `docs/linting.md:50-80`).

### Design

- One aspect action per package target (`ts_project` or the `pkg` `js_library`),
   so lint stays per package;
   nothing forces a repository-wide action.
- Action shape:
   `js_binary` for `oxlint` with `chdir` to the package's bin directory,
   env `OXLINT_TSGOLINT_PATH` pointing at the tsgolint binary
   (verified that oxlint 1.81.0 reads it:
   `strings` over `@oxlint/binding-linux-x64-gnu/*.node` shows
   "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to"),
   `--format json` once,
   human output rendered from JSON to avoid the double run.
- Inputs per package, derived from the current behavior above:
  - own `src/**/*.ts` and `*.config.ts`;
  - package `tsconfig.json` plus `config-typescript` `tsconfig.options.json` and `tsconfig.dom.json`;
  - transitive sibling `.ts` sources through `/ts` links (needed by tsgolint programs and live effect analysis);
  - npm packages of the whole transitive closure as tree artifacts
     (types for tsgolint, shipped JavaScript for the readonly rule's implementation inference);
  - root `oxlint.config.ts` copied to bin plus `config-oxlint` built output (index and five sidecars),
     or the `/ts` source variant (`package/config/oxlint/src/index.ts`) with plugin sources and their dependencies;
  - `oxlint`, `oxlint-tsgolint`, and TypeScript 7 native platform binaries;
  - `pnpm-lock.yaml` copied to the bin root,
     because the readonly rule reads it and derives its cache root from it.
- Invalidation fan-out (from `doc/research/bazel-migration-dx.md`, "Never running a warm whole-repo oxlint again"):
   `config-typescript` 146 direct dependents,
   `module-test` 119,
   `config-rolldown` 111,
   `module-logger` 66;
   320 of 2,328 TypeScript commits since 2026-06-18 touched plugin or oxlint config source,
   each invalidating every lint action.
  Adding `pnpm-lock.yaml` as an input also invalidates every lint action on any dependency change.
- Sandbox interaction:
   Yikes candidates 2 and 3.

## Area 5: build

### Today

- 129 `rolldown.*.config.ts` files in 113 packages
   (`rg --files --glob 'rolldown.*.config.ts'`, `targetcount.ts`);
   no `tsdown.*.config.ts` remains
   (`package/config/tsdown` holds only `dist` and `node_modules`, no `package.json`).
- Task templates run `rolldown --configLoader native --config rolldown.<flavor>.config.ts`
   (`mise.no-env.toml:524`, `mise.toml:519-526`);
   two plugin packages invoke `node ../../../node_modules/rolldown/bin/cli.mjs` directly
   (`package/oxlint-plugin/prefer-readonly-parameter-type/mise.toml`, `build:js:node`).
- `@monochromatic-dev/config-rolldown` (`package/config/rolldown/src`):
  - bundles `@monochromatic-dev/**` from source into every artifact
     (`index.ts` `NEUTRAL_ALWAYS_BUNDLE`, `index.node.ts` `NODE_ALWAYS_BUNDLE`);
  - `dts({ generator: 'oxc' })` for declarations;
  - `cleanDir: true` for `dist/final/<flavor>`,
     except `perEntryNodeConfig`, which relies on the task pre-cleaning (`index.node.ts`);
  - reads `process.cwd()/package.json` for externals (`package-externals.ts:191-204`);
  - reads `../../../../.browserslistrc.resolved.local.json` relative to its own source file,
     a gitignored file generated by file-enforcer,
     falling back to live Browserslist resolution from `process.cwd()`
     (`browserslist-targets.ts:298-301`, `:925-945`).
- Outputs outside `dist/`:
  - eight Claude Code plugins write `bundle/node`,
     which is committed
     (`package/claude-code-plugin/guardrail/rolldown.node.config.ts`;
     `git ls-files package/claude-code-plugin | rg --count /bundle/` returned 18);
  - `package/ssg/aquati.cat`:
     `build` runs `rm -rf dist`, a client bundle, `node src/build.ts`, in-place postprocess, and compression
     (`package/ssg/aquati.cat/mise.toml:1-42`);
     it persists `.cache/build-manifest.json` in the package (`src/lib/cache.ts:105`),
     runs `git log --follow` per content file (`src/lib/git-dates.ts:224-310`),
     spawns `pagefind` from the mise tool set (`mise.toml:170-173`),
     and rewrites `dist` files in place (`src/build/postprocess.ts:326-352`).
- Other TypeScript-side custom builds:
   Electron staging in `desktop-app/electron-counter` and `desktop-app/file-manager-electron`,
   Node SEA in `kwin/key-helper` (`sea`, `binary`),
   `typeface/aquaticat` `build:font`,
   `webapp-productivity/done-postcss` `build:css`,
   `module/logger.fuzz` `bundle:browser-properties`,
   and `desktop-daemon/hall-monitor` using `bun build --compile`
   (`misecensus.ts` and `rg '^\[tasks\."?(build|...)'` over package `mise.toml` files).

### How Bazel would do it

-
`js_run_binary(tool = ":rolldown", srcs = [...], out_dirs = ["dist/final/node"], chdir = package_name(), args = ["--configLoader", "native", "--config", "rolldown.node.config.ts"])`
   (verified attributes: `js/private/js_run_binary.bzl:19-45`, `chdir` guidance at `:95-123`).
- `srcs` must include `package.json`,
   the config file,
   own `src`,
   transitive sibling sources (bundled inline),
   `.browserslistrc` at the bin root (Browserslist searches upward from cwd),
   and either the generated `.browserslistrc.resolved.local.json` or an accepted fallback.
- Fresh declared output directories make `cleanDir` and the per-entry pre-clean moot.
- Committed bundles:
   `write_source_files` from bazel-lib updates checked-in files and generates a drift test
   (verified: `bazel-lib lib/write_source_files.bzl:1`);
   the 18 plugin bundle files become `bazel run` outputs.
- aquati.cat needs restructuring:
   git history is not a declarable input,
   the `.cache` manifest must become an output or be dropped,
   and each in-place step must emit a new tree.
  Unverified whether stamping (`js_run_binary(stamp = ...)`, `js_run_binary.bzl:208-221`) can carry per-file git dates;
   status files are key-value lines,
   so per-file history would need a separate generator.
- Native rolldown bindings resolve with their own resolver;
   unverified whether they realpath outside the sandbox the way tsgo does.

## Area 6: tests

### Today

- `test:unit` globs `**/*.unit.test.ts`,
   excluding `.expensive.` unless `--all`,
   and runs `node <file>` per file with cwd at the nearest `mise.toml`,
   bounded by `availableParallelism()`
   (`mise.no-env.toml:454`, `:548`; generated `mise.toml:455-487`, `:549-580`).
  No Node flags are passed,
   so tests rely on Node 26's default type stripping.
- Harness:
   `@monochromatic-dev/module-test` is a self-executing `describe` and `it` API over chai and sinon;
   failures throw and set a non-zero exit
   (`package/module/test/README.md`, "API").
  Its logger's file sink writes `node_modules/.monochromatic/<timestamp>.log.jsonl` under the nearest ancestor
  `node_modules`
   and degrades to unavailable on any error
   (`package/module/logger/src/sink/file.ts:168-238`).
- The `test-import` oxlint rule requires tests to import the shipped artifact
   (`package/config/oxlint/src/index.ts` comment on `oxlint-plugin-test-import`).
- Census (`testcensus.ts`, regex heuristics):
   694 active unit test files in 109 packages (4 `.expensive.`);
   234 files in 50 packages import `dist/final` or `../dist` paths;
   116 files in 35 packages use `mkdtemp`;
   62 files in 30 packages spawn children.
- Environment-bound tests (targeted `rg`, heuristic):
  - network:
     `oxlint-plugin/prefer-readonly-parameter-type/src/external-consumer.unit.test.ts` runs `pnpm pack`
     and `pnpm install --ignore-workspace` (lines 77-110);
     `config/pnpr/src/publish-plan.unit.test.ts` matched the network pattern;
  - containers or other runtimes:
     `module/matrix` (`container.unit.test.ts`, `runtime.unit.test.ts`, `self.unit.test.ts`,
     podman plus bun and deno per its README),
     `vmsync/cli/src/lifecycle.expensive.unit.test.ts`,
     `mvm/cli/src/cli.unit.test.ts`;
  - real home:
     `homedir()` or `process.env.HOME` in `claude-code-plugin/statusline`, `figma/to-penpot`, `git-policy/cli`,
     `pi-plugin/auto-mode`, `wg/core-allowedips`, `module/fs-path`;
  - repository layout:
     35 files use `findRoot` or `MISE_MONOREPO`,
     including the shared oxlint plugin test root (`package/oxlint-plugin/test-support/src/index.ts:37-40`);
  - PATH-resolved tools:
     `oxlint` spawned by name (`test-support/src/index.ts:370-385`),
     `pnpm` by name in the external consumer test;
  - writes into the source tree through `import.meta.dirname`:
     none matched.

### How Bazel would do it

-
`js_test(name = "<file>", entry_point = "src/<file>.unit.test.ts", chdir = package_name(), data = [":pkg", ":dist", ":node_modules/..."])`,
   one per file to keep today's process isolation (694 targets),
   or a per-package runner (109 targets).
- Built-artifact imports make each test depend on its package's rolldown action,
   and on other packages' builds for cross-package bare imports.
- Environment-bound tests get `tags = ["manual"]` or `["requires-network", "no-sandbox"]`,
   plus data dependencies for root markers and env such as a `PATH` entry for `oxlint`.
- Node toolchain:
   rules_nodejs mirrors Node up to 24.21.0 and no 25 or 26 release
   (verified: `gh api repos/bazel-contrib/rules_nodejs/contents/nodejs/private/node_versions.bzl` searched for `"25.`
   and `"26.` with no match);
   the toolchain tag accepts `node_repositories` and `node_urls` for unmirrored versions
   (verified: `rules_nodejs nodejs/extensions.bzl:108-136`),
   so Node 26.8.2 needs hand-maintained checksums.
- Type stripping under `js_test` is unverified,
   since rules_js has no `.ts` entry point example (Area 2).

## Area 7: formatting

### Today

- `format` runs `dprint fmt`,
   then Stylelint, oxlint, and markdown fixes
   (`mise.no-env.toml:932`);
   `lint:dprint` runs `dprint check --list-different` (`:869`).
- dprint comes from mise (`mise.toml:58`);
   the npm package is removed by override (`pnpm-workspace.yaml` `dprint: '-'`).
- dprint plugins, fetched by URL:
   TOML, markup_fmt, malva (CSS), pretty_yaml, JSON;
   TypeScript and Markdown plugins are disabled
   (`package/config/dprint/index.json`).
  TypeScript layout comes from `oxlint-plugin-stylistic`.

### How rules_lint would do it

- `format_multirun` offers one tool per language attribute
   (verified: `format/private/formatter_binary.bzl:5-38`)
   and allows `<lang>_fix_args` and `<lang>_check_args`
   (verified: `format/defs.bzl:25-40`, `:236-245`).
- The `javascript` attribute also receives JSON, JSON5, JSONC, TSX, TypeScript, and Vue batches
   (verified: `format/private/format.sh:350-354`),
   and each language batch is invoked separately (`format.sh:291-339`).
- dprint 0.57.4 exits 14 when every given path lacks a plugin
   and exits 0 for a mixed batch
   (verified by running `dprint check package/module/logger/src/index.ts`,
   "No files found to format with the specified plugins",
   and `dprint check package/module/logger/src/index.ts package/module/logger/package.json` with no output).
  So mapping dprint to `javascript` needs `--allow-no-files` in both argument sets.
- Language attributes exist for CSS, HTML, YAML, TOML
   (verified: `formatter_binary.bzl:5-38`).
- Simpler alternative:
   a `bazel run` target that runs `dprint fmt` in `BUILD_WORKSPACE_DIRECTORY`,
   keeping dprint's own file selection and `excludes`;
   `format.sh` already `cd`s there (verified: `format.sh:16-35`).
- oxlint fixes stay in the lint aspect's patch flow (Area 4).
- dprint downloads plugins at run time;
   under `bazel run` that is outside the sandbox and unchanged from today.

## Area 8: editor

- rules_js tells editor users to keep running `pnpm install` for a source-tree `node_modules`
   because Bazel's tree lives under `bazel-out`
   (verified: `rules_js docs/faq.md:9-26`).
  The design therefore keeps two installs from one lockfile:
   pnpm for editors and ad hoc Node scripts,
   rules_js for Bazel actions,
   separated by `ignore_directories(["**/node_modules"])`.
- TypeScript LSP:
   tsgo resolves `/ts` through the pnpm tree exactly as today;
   no `paths` mapping is needed because pnpm links already resolve.
- oxlint LSP:
   repository comments say the language server does not run JS plugins
   (`package/config/oxlint/src/index.ts`, `jsPlugins` comment;
   plugin `README.md`, "CLI diagnostics are authoritative").
  Both cited upstream issues are now closed
   (verified: `gh issue view 14402 --repo oxc-project/oxc` closed 2025-10-07;
   `gh issue view 14826` closed 2026-03-02),
   so current LSP behavior is unverified.
  Either way Bazel does not change it,
   because the editor uses the pnpm tree.
- Unverified:
   whether Bazel convenience symlinks (`bazel-bin`, `bazel-out`, `bazel-<workspace>`)
   are walked by root-level oxlint, dprint, or file watchers;
   a `--symlink_prefix` under an ignored directory avoids the question.

## Yikes candidates

Ordered by how likely each is to block or reshape the design.

### 1. Substituted workspace links produce references to package stores that are never generated

- Evidence:
   Area 1, "Risk found";
   `npm_translate_lock_generate.bzl:88-181`,
   `utils.bzl:96-103`,
   `npm_import.bzl:741-745` and `:783-786`,
   five edges listed with `pnpm-lock.yaml` line numbers.
- Why it matters:
   the pnpm policy layer (`overrides` shims, `.pnpmfile.mjs` stubs) is repository governance,
   and the affected consumers include stylelint, the pi packages, and the aquati.cat markdown pipeline.
- Status:
   source-read inference;
   the first `bazel build` over a stylelint consumer would confirm or refute it.
  Fixable by aliases plus one stub directory per alias, or a rules_js patch.

### 2. Native tools escape the sandbox that the Node side of the same process honours

- Evidence:
  - the sandbox is a symlink tree pointing at original files
     (verified: Bazel `docs/docs/sandboxing.mdx:69-75`);
  - rules_js confines only Node's `fs` through patches
     (verified: `js_binary.bzl:147-159`),
     and ESM imports already skip them
     (verified: rules_js `README.md:44-45`);
  - TypeScript 7's `tsc` replaces Node with the native binary (`typescript/lib/tsc.js`),
     and tsgo module resolution calls native realpath
     (verified: `typescript-go internal/vfs/osvfs/os.go:154-173`, `internal/module/resolver.go:1881-1886`);
  - the readonly rule runs inside oxlint's Node process but asks a `tsgo --api` child for file identities,
     then classifies files by path shape
     (`workspace-source-path.ts:1-40`, `installed-package-identity.ts:66-116`).
- Why it matters:
   the plugin's Node side sees sandbox paths while its tsgo child can report real execroot paths for sibling files.
  Unverified whether path identity mismatches cause missing summaries,
   opaque-effect diagnostics,
   or nothing.
  Running lint unsandboxed removes the mismatch but also removes Bazel's input enforcement for the slowest action,
   which was Route A's main stated advantage in `doc/planning/monorepo-manager-build-routes.md`, "Pros".
  The user already accepted lint-level rather than OS-level enforcement ("Answered questions"),
   so this erodes Route A's case rather than disqualifying it.

### 3. The readonly rule's persistent cache cannot persist, so Bazel lint reruns are cold

- Evidence:
   cache root derivation and silent write skipping (Area 4, "Today");
   linux-sandbox makes everything outside the sandbox directory read-only
   and discards undeclared outputs
   (verified: `sandboxing.mdx:69-81`);
   no environment override exists in the plugin.
- Cost:
   every rerun pays cold numbers,
   7.4 to 22.0 s per measured package instead of 1.2 to 3.1 s warm,
   and a whole-repository rerun costs 261 to 305 s instead of 63 to 105 s
   (`doc/decision/workspace-ts-source-imports.md`, "Context").
  Every lint action also needs `pnpm-lock.yaml` as an input,
   so any dependency bump reruns all of them cold.
  An eslint-style aspect would double that by running the linter twice (`eslint.bzl:208-228`).
- Escape hatches, all unprototyped:
   `no-sandbox` execution for the lint mnemonic,
   or `--sandbox_writable_path` plus a new cache-root environment variable in the plugin.
  Both make lint results depend on state Bazel does not track;
   the plugin's cache envelope validation is what keeps that safe.

### 4. Tests are downstream of bundles, and bundles inline all workspace source

- Evidence:
   234 test files in 50 packages import `dist` paths (heuristic),
   and 181 bare self-import statements resolve through `"."` exports that are mostly `dist` (Areas 2 and 6);
   builds inline `@monochromatic-dev/**` (Area 5).
- Why it matters:
   an edit to a widely used module rebuilds every bundling dependent and reruns its tests.
  This is correct behavior Bazel would make explicit,
   not a blocker,
   but it bounds the "affected-only" savings.

### 5. Some builds read state Bazel cannot declare or write where Bazel cannot

- Evidence:
   aquati.cat reads git history per file and keeps a source-tree cache;
   18 committed plugin bundle files;
   `config-rolldown` reads a gitignored root file relative to its own source path (Area 5).
- Why it matters:
   aquati.cat needs a real redesign (precomputed git dates as a tracked or generated input);
   the rest are `write_source_files` and input declarations.

### 6. Source-typed dependencies in rules_ts are undocumented

- Evidence:
   rules_ts docs expect `.d.ts` (`docs/troubleshooting.md:96`);
   the `js_library(types = ...)` route and tsgo's external-library classification are source-derived (Areas 2 and 3);
   composite plus `noEmit` interplay with the validator is unverified.
- Why it matters:
   a per-package `ts_project` may need a Bazel-only tsconfig layer,
   or type checking moves entirely into oxlint `typeCheck`, which oxlint labels experimental.

### 7. Toolchain and ruleset gaps that are work, not blockers

- Node 26 is not mirrored by rules_nodejs (Area 6).
- `ts_version_from` cannot read `catalog:` (Area 3).
- No oxlint aspect upstream and no dprint formatter mapping (Areas 4 and 7).
- Native pnpm 12 and the pnpm 12 lockfile test case exist only on rules_js `main`, not v3.4.1 (Area 1).
- Two `node_modules` installs must stay in sync for editors (Area 8).
- rules_lint fix mode is one patcher pass per action,
   while the repository's wrapper loops `--fix` to a fixpoint (`oxlint-wrapper.ts`, `fixUntilStable`),
   so the wrapper, not bare oxlint, must be the patcher's command.

## Prototype order suggested by the findings

- `npm_translate_lock` over the real lockfile,
   then `bazel build` of one stylelint consumer (candidate 1).
- One `ts_project(no_emit)` plus one `js_test` for a package with `/ts` siblings,
   for example `package/dev-script/file-enforcer` (candidates 6 and Area 6 type stripping).
- One oxlint aspect action for `package/pi-plugin/auto-mode` under `linux-sandbox` and under `no-sandbox`,
   diffing JSON findings against today's `mise run //package/pi-plugin/auto-mode:lint:oxlint`
   and timing both (candidates 2 and 3).
