# module-fs-path first npm release

Status:
 done (2026-09-07);
 0.1.0 is published,
 the trusted publisher exists,
 and the release is tagged.
Owner decisions from the release grilling are recorded here as they land;
 this file is canonical for the release.
Mechanism and prior art:
 `doc/decision/npm-publishing.md` and `doc/planning/module-logger-release.md`.

## Driver

`@monochromatic-dev/test-harness-core` cannot publish while it depends on private packages.
The logger shipped on 2026-09-06 (0.4.0 today);
 `module-fs-path` is the other private dependency,
 so it gets the same treatment next.
After this release,
 `module-test` still waits on `module-async-time`,
 `module-caught-value`,
 `module-const`,
 and `module-numeric-format`.

## Settled decisions (2026-09-07)

- The published artifact follows the logger's platform split
   (`package/logger/core/DECISIONS.md`,
   "Platform-specific sinks live behind `./node` and `./browser`"):
   the root entry `.` is platform-neutral (POSIX path ops,
   trims,
   the three root finders,
   `GitRepositoryRootNotFoundError`);
   `./node` ships `ensureDir`,
   `ensureFile`,
   `ensurePath`,
   `emptyDir`,
   `emptyFile`,
   `emptyPath`,
   `removeEmptyFilesInDir`,
   `findPackageRoot`,
   and `findPackageRootCached` with static `node:` imports.
   The owner rejected a Node-only 0.1.0 (the package was created to run in browsers,
   confirmed by the extraction commit `894c84a5d`,
   the `dom` tsconfig,
   and the OPFS backend)
   and a single entry with dynamic imports
   (the logger decision measured that every `import()` leaks into every downstream bundle).
- Backends are chosen at resolution time through `package.json` `imports` entries:
   `#posix-path` (`node`:
   `node:path/posix`;
   `default`:
   the pure-JS implementation)
   and `#root-filesystem` (`node`:
   `node:fs/promises`;
   `default`:
   OPFS or an empty backend).
   Neither built artifact contains `import(`;
   the neutral artifact names no `node:` module;
   the node artifact touches no `navigator.storage`.
- The OPFS backend is rewritten on `navigator.storage.getDirectory()`,
   as the logger's OPFS sink already is.
   `happy-opfs` leaves the manifest,
   the pnpm catalog,
   and the lockfile:
   929 kB unpacked plus five transitive packages that every Node installer would download for a path no consumer runs.
- `normalize` is public and delegates to `node:path/posix` under the `node` condition like its siblings.
   `dirnameFallback`,
   `joinFallback`,
   and `resolveFallback` leave the public entry:
   in the neutral artifact they are the implementation,
   so tests import `dist/final/neutral/index.mjs` directly.
   The top-level `await import` in `index.ts` goes with them.
- Logger and caught-value stay inlined (standing decision).
   Accepted consequence:
   a consumer using both `module-logger` and `module-fs-path` runs two logger instances with separate sink discovery.
- A real Chromium run is a release gate:
   the Playwright harness (podman,
   local only,
   no browser CI job yet)
   loads the neutral artifact in the page,
   writes marker files into OPFS,
   and calls the finders.
   Firefox and WebKit run too and may report skipped where the platform refuses OPFS.
   The chunk-text leak test from the logger is adopted with positive controls and a proven red run.
- Tests for the seven untested fs helpers and the error class,
   through the built `./node` artifact.
   The `self.unit.test.ts` aggregator is replaced by the root `test:unit` task,
   which clears the 3 `require-eventual-artifact` findings.
- Manifest:
   drop `private`;
   `publishConfig` with `access: public`,
   `provenance: true`,
   and `exports` without `./ts`;
   `engines.node >=24` (both builds call `Error.isError` through the inlined logger);
   `sideEffects: false` (module-level state is loggers and caches);
   `files` gains `CHANGELOG.md`;
   inlined workspace dependencies move to `devDependencies`;
   the stale `dist/final/types/tsconfig.tsbuildinfo` (2026-07-21,
   110 kB) is removed before packing.
   The description keeps "Node/Bun and browser":
   Bun honors the `node` condition (`bun`,
   `node-addons`,
   `node`,
   `require`,
   `import`,
   `default`).
- First version 0.1.0 from a `minor` changeset;
   the first publish is local and interactive in this session,
   the owner typing the one-time password;
   later versions publish from CI with provenance.
- Records:
   `package/fs-path/core/DECISIONS.md` for the split (package docs stay beside code),
   this file for the release,
   README rewritten.

## Measured state (2026-09-07)

- `package/fs-path/core/package.json`:
   `private: true`,
   version 0.0.1,
   404 on the registry;
   `dependencies` on `module-caught-value` (private),
   `module-logger` (published),
   and `happy-opfs`.
- Both built artifacts inline the logger and caught-value (no `@monochromatic-dev` import),
   statically import `node:fs/promises` and `node:path` (from `ensure.ts`,
   `empty.ts`,
   `find-package-root.ts`),
   and carry `import(` of `happy-opfs`,
   `node:fs/promises`,
   and `node:path`.
   The `default` export condition therefore hands a browser bundler Node builtins.
- Consumers:
   34 import lines in 30 files,
   every one a Node context;
   no built client bundle in the repo contains fs-path code;
   `doc/troubleshooting/css-tooling.md` records done-postcss routing around fs-path
   because its imports dragged Node builtins into the client bundle.
   Only `dependency-catalog/cli-cube` (2 files) imports a Node-only helper (`findPackageRootCached`);
   nothing in the repo imports the `ensure*` or `empty*` helpers.
- Gates:
   `lint:types` green;
   `buildAndTest` green;
   `lint:oxlint` red with 3 `require-eventual-artifact` errors on `self.unit.test.ts`.
   `ensureDir`,
   `ensureFile`,
   `ensurePath`,
   `emptyDir`,
   `emptyFile`,
   `emptyPath`,
   `removeEmptyFilesInDir`,
   and `GitRepositoryRootNotFoundError` have no test.
- README claims the package is source-only and lists `./find-monorepo-root`
   and `./find-package-root` subpaths removed in commit `06cceaa67` (2026-05-29).
- `npm pack --dry-run`:
   22 files,
   347 kB unpacked,
   including the stale tsbuildinfo.
- `doc/todo/packages.md` carried the open item "decide whether `module-fs-path` should publicly export `normalize()`";
   decided above.

## Execution plan

1.   This record.
2.   Platform split with the native OPFS backend;
     deps-cube imports move to `/ts/node.ts`;
     both builds and `lint:types` green.
3.   Tests:
     fs helpers and the error class through `./node`;
     pure-JS path ops and the no-filesystem behavior through the neutral artifact;
     the leak guard with positive controls and a proven red run;
     root `test:unit` replaces the aggregator;
     zero lint findings.
4.   Browser gate:
     harness route,
     `*.browser.test.ts`,
     Chromium pass,
     proven red run.
5.   Manifest,
     README,
     `DECISIONS.md`,
     `npm pack --dry-run` inspected.
6.   Changeset (`minor`),
     Version Packages pull request,
     tarball verified in a disposable Node 24 consumer.
7.   Bootstrap publish per `doc/runbook/publish-npm-package-first-time.md` with fs-path names,
     `npm trust github`,
     merge,
     tag and release,
     registry install verified.
8.   Records closed;
     `doc/todo/packages.md` item resolved.

## Progress log

- 2026-09-07:
   grilling complete;
   this record written.
- 2026-09-07,
   split landed:
   `e3badc3d7` (platform split,
   native OPFS backend,
   happy-opfs out of manifest,
   catalog,
   and lockfile;
   deps-cube imports moved to `/ts/node.ts`).
   The lockfile was regenerated with the committed pnpm 11.21.0 through `mise exec pnpm@11.21.0`:
   the uncommitted pnpm 12.3.4 bump in `mise.lock` (another session's change)
   fails every `pnpm install` in this checkout,
   pristine or not,
   while re-verifying the lockfile:
   `ERR_PNPM_META_FETCH_FAIL` fetching `@jsr/std__path` from `registry.npmjs.org`.
   Recorded in `doc/troubleshooting/pnpm-12-jsr-scope-lockfile-verification.md`.
   The deps-cube lint findings (11 warnings,
   21 errors in `controller.ts` and two test files) predate this change.
- 2026-09-07,
   tests landed:
   `27ad96116`.
   Two defects surfaced and fixed in the same commit:
   `ensureDir` and `ensureFile` repaired permissions with `chmod(path, R_OK | W_OK)` (mode `0o006`,
   owner locked out);
   the pure-JS `dirname` skipped one trailing slash where `node:path/posix` skips the run.
   Guard-failure proof for the platform-split guard:
   root entry made to re-export `ensureDir`,
   rebuilt,
   two cases red;
   restored,
   green.
- 2026-09-07,
   browser gate landed:
   `36400f8c8`.
   Chromium 2 passed,
   Firefox 2 passed,
   WebKit 1 passed and 1 skipped (OPFS writes refused).
   Guard-failure proof:
   OPFS backend made to report every path absent,
   rebuilt,
   Chromium 1 failed;
   restored,
   2 passed.
- 2026-09-07,
   manifest and docs:
   `fdc0302df`;
   changeset and todo item:
   `bbb395445`.
   The manifest commit reached `main` before the changeset,
   so the release workflow saw a public package at 0.0.1 missing from the registry and tried to publish it:
   `E404` from `PUT .../module-fs-path` (no trusted publisher can exist before the bootstrap).
   Harmless;
   lesson for the runbook:
   land the changeset in the same push as dropping `private`.
- 2026-09-07,
   tarball verified in a disposable Node 26 consumer:
   `dependencies` empty,
   no `./ts` export,
   no `workspace:` string,
   no tsbuildinfo;
   `findMiseMonorepoRoot`,
   `ensureDir`,
   and `findPackageRootCached` work from the installed package;
   `/ts` refused with `ERR_PACKAGE_PATH_NOT_EXPORTED`.
   The consumer's fixture put `[monorepo]` on line 1 of `mise.toml`
   and the finder missed it (the marker required a newline before the header):
   fixed in `a7876a2a2` with three regression cases.

## Result (2026-09-07)

- `@monochromatic-dev/fs-path-core@0.1.0` is on the registry
   (`npm view` prints the version and a `registry.npmjs.org` tarball URL);
   published from a linked worktree of the version branch with `npm publish --otp`,
   no provenance.
- The owner registered the trusted publisher for `npm-release.yml` from their own terminal
   (`npm trust github` needs browser web authentication;
   `npm trust list` too,
   so the registration is verified by the next CI publish rather than here).
- Version pull request #499 merged as `3e46e38df`;
   the release run it triggered (34111678810) found nothing to version or publish;
   tag and GitHub release `@monochromatic-dev/fs-path-core@0.1.0` created through the API on the merge commit.
- A throwaway Node 26 project installed 0.1.0 from the registry and ran the mise finder,
   `ensureDir`,
   and `findPackageRootCached`;
   `/ts` refused with `ERR_PACKAGE_PATH_NOT_EXPORTED`.
- Runbook amended with the linked-worktree checkout,
   `--otp`,
   the web-auth constraint on `npm trust`,
   and the changeset-with-manifest ordering.

## Next action

Historical next action after the 2026-09-07 release:

None for this release.
The next changeset merged to `main` for this package exercises the CI publish path
with provenance for the first time and proves the trusted publisher.
`module-test` still waits on `module-async-time`,
 `module-caught-value`,
 `module-const`,
 and `module-numeric-format`.

## Trusted publisher validation release, 2026-10-16

The owner requested a version bump and npm publication to validate the newly added trusted publisher
for `Aquaticats/Monochromatic` and `npm-publish.yml`.
The registry already held `0.2.0`,
published on 2026-09-08 with provenance;
the package directory had no changes since its release tag.
A disposable registry installation passed root discovery on disk and in memory,
Node directory and file helpers,
path joining,
and rejection of the unpublished `/ts` entry.

Release `0.2.1` updates the repository URL to the transferred GitHub owner and keeps the public API unchanged.
Commit `499ab653de` holds the version and changelog update on `release/fs-path-npm-20261016`.
The dedicated manual workflow builds and tests only this package,
packs it with pnpm,
and publishes the tarball with npm OIDC and provenance in a separate job.
It runs from the release branch so the workflow revision matches the source revision.
Publishing before bringing the bump to `main` prevents the existing `npm-release.yml` workflow
from publishing the version before the new trusted publisher can validate.

Local checks passed:
`mise run //package/fs-path/core:buildAndTest`,
`lint:types`,
and `lint:oxlint` (zero warnings and errors).
[Release run 38837297263](https://github.com/Aquaticats/Monochromatic/actions/runs/38837297263)
builds commit `704f1a1586a5ef37c8dc597d1fcb0434dfb7a71a`.
The run succeeded and npm made `0.2.1` available at `2026-10-16T01:53:24.576Z`.
The registry's `latest` tag resolves to `0.2.1`.
Its publisher is GitHub Actions,
with trusted publisher configuration `b5559568-84c4-4539-93bd-c623048cd565`.
The SLSA provenance identifies `Aquaticats/Monochromatic`,
`.github/workflows/npm-publish.yml`,
and source commit `704f1a1586a5ef37c8dc597d1fcb0434dfb7a71a`.
This successful OIDC publication exercised the newly configured publisher.

A disposable consumer with its own `package.json` installed `0.2.1` from npm and passed the same runtime checks,
plus assertions of the installed version and repository URL.
Commit `d71ed3b6d4` records the matching version and changelog on `main`.
Tag `@monochromatic-dev/fs-path-core@0.2.1` preserves the exact published source commit.
The initial consumer fixture omitted its own manifest and caused npm to select the parent scratch directory;
the affected temporary Markdown dependencies were restored and compiled successfully.
Details:
[npm parent prefix selection](../troubleshooting/npm-scratch-parent-prefix.md).

Next action:
none for `0.2.1`.
