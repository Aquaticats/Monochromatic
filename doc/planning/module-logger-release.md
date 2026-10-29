# module-logger first npm release

Status:
 done (2026-09-06);
 0.1.0 is published,
 the trusted publisher exists,
 and the release is tagged.
Owner decisions are recorded here as they land;
 this file is canonical for the release,
 not `package/logger/core/bulletproofing.plan.md`.

## Legacy reference

`package/logger/core/bulletproofing.plan.md` (written 2026-06-15,
 amended by a GPT Pro review) is a verification program,
 not a release plan.
The owner classified it as a historical reference written by an unreliable narrator.
Measured against current code on 2026-09-06:

- It predates the IndexedDB and localStorage sinks,
   the record buffer,
   and the bounded storage retries (all 2026-07-22),
   so its sink inventory and file map are wrong.
- Gap B (log call throws) is reachable only through `createLogger` with custom sinks;
   console `verify` fails only when `console` or `queueMicrotask` is missing.
- Gap A3 (retire after 10 failures) is unreachable for every shipped sink,
   which each swallow their own write errors,
   and it reverses the 2026-06-14 decision in `package/logger/core/DECISIONS.md`.
- Its claim that `%s` in a single-argument `console.info` is interpreted is false on the built artifact:
   `%s%d` reaches stdout literally.
- Its claim that terminal escapes pass through raw is true on the built artifact:
   OSC title-set and CSI clear-screen bytes reach stdout unchanged.

Its post-release value (toml-edit verification bar,
 model oracle,
 scheduler interleavings) is to be rewritten as a separate post-release track.

## Settled decisions

- Release means npm publish under `@monochromatic-dev`,
   the scope's first publish ever.
   The owner confirms the scope exists and they can publish to it.
- Driver:
   `@monochromatic-dev/test-harness-core` is blocked by issue #338 because it depends on the private logger.
   Scope is the logger alone,
   as groundwork;
   `module-fs-path` stays private,
   so module-test remains blocked after this release.
- README import examples change to root imports;
   no `/tagged` or `/logger` subpath exports are added.
- The tuning options from the legacy plan stay out of 0.1.0 except `flushDeadlineMs`.
- The retire-after-N-failures policy is dropped;
   only `verify` failure retires a sink,
   as `DECISIONS.md` 2026-06-14 records.
- The console sink neutralizes every control character except `\n` and `\t` before 0.1.0,
   rendering each as `\uXXXX`;
   no SGR allowlist.
   This is a publish gate (rule `SYB`).
- Tarball:
   `tsconfig.tsbuildinfo` leaves every tarball via the shared tsconfig;
   test files ship (owner accepts the mixed ecosystem practice).
- `./ts` is stripped from the published manifest through `publishConfig.exports` (pnpm override);
   in-repo `/ts` imports are unaffected.
   `module-caught-value` moves to `devDependencies` (the dist bundles it) and is not published in this release.
- `flush()` gets a deadline before 0.1.0,
   exposed as one `createLogger` option `flushDeadlineMs`
   with a named default chosen after measuring local write latency.
- The 25 `require-eventual-artifact` findings gate the publish:
   internals are exported from the entry with an underscore prefix (toml-edit precedent)
   and their tests import the built artifact.
- The no-backend throw stays in 0.1.0:
   reachable only through `createLogger` with sinks that all fail verify,
   documented,
   pinned by tests,
   consistent with rules PP4 and PP8.
- `engines.node` declares `>=24`:
   the dist calls `Error.isError`,
   which arrived with V8 13.6 in Node 24.0.0.
- First version 0.1.0,
   produced by a `minor` changeset.
- Versioning and publishing tool:
   changesets (the owner's standing intent,
   issue #159's recommendation).
   Adoption requires a vet report per the `choosing-technology` skill,
   since it promotes one tool over release-please and semantic-release.
- Publish workflow:
   a changesets release workflow with `id-token: write` and OIDC trusted publishing,
   building selected packages through mise before `pnpm publish -r`.
   The never-run `publish.yml` is deleted;
   issue #307 closes by construction and #306's build and pack checks move to the new workflow.
- Bootstrap:
   the first publish of a package is local and interactive.
   `pnpm pack` the package (rewrites `workspace:` and applies `publishConfig`),
   `npm publish <tarball> --access public` with provenance disabled and the owner's OTP,
   then `npm trust github` for the release workflow.
   Later versions publish from CI with provenance.
   No npm token is ever stored.
- A `doc/decision/` entry supersedes the "bundle,
   do not publish" direction.
- The legacy plan becomes a post-release track after a rewrite
   that fixes its inventory and removes the items decided against here;
   a GitHub issue tracks it.

## Measured state (2026-09-06)

- Nothing in the `@monochromatic-dev` scope exists on npm;
   ten names probed,
   all 404.
   No local npm login (`npm whoami` returns 401).
- `package/logger/core/package.json`:
   `private: true`,
   version 0.0.1,
   no `publishConfig`,
   `dependencies` on unpublished `module-caught-value`,
   exports `.`,
   `./ts`,
   `./ts/*`.
- Built dist bundles caught-value (no `@monochromatic-dev` imports in `dist/final/*/index.mjs`);
   the `./ts` export still imports `@monochromatic-dev/caught-value-core/ts` from source.
- `npm pack --dry-run` on the current dist:
   55 files,
   506,898 unpacked bytes,
   including `dist/final/types/tsconfig.tsbuildinfo` and 21 test files (issue #336).
- Gates:
   `lint:types` passes;
   `test:unit` passes;
   `lint:oxlint` reports 25 errors,
   all `test-import(require-eventual-artifact)`.
- `.github/workflows/publish.yml`:
   hardcoded filter list without logger,
   never run,
   no build step (issue #306),
   token required for dry run (issue #307);
   `gh secret list` shows no `NPM_TOKEN`.
- Consumers in this repo:
   288 `/ts` import sites,
   2 root import sites,
   1 `createLogger` caller outside the package (module-test's unit test).
- Transitive workspace runtime closure of `module-test` (all at 0.0.1):
   `module-test`,
   `module-async-time`,
   `module-caught-value`,
   `module-const`,
   `module-numeric-format`,
   `module-or-throw` (public),
   `module-logger` and `module-fs-path` (private).
- The #338 consumers bundle the logger into their dists;
   their `dependencies` entries are what would break an external install.
- Node refuses `.ts` files under `node_modules` by default (Node TypeScript docs),
   so a published `./ts` subpath serves bundler users only.
- No in-repo file imports both the logger and a color library,
   and no log call carries an escape literal or color helper.
- The dist uses `Error.isError` (twice) and `toSorted`;
   `Error.isError` needs Node 24+.
- `files` negation works under the packer:
   `package/git-policy/cli/package.json` lists `!src/**/*.unit.test.ts`,
   and `npm pack --dry-run` there shows 0 of its 44 unit-test files.
- `tsconfig.tsbuildinfo` lands in `dist/final/types` because the shared
   `package/config/typescript/tsconfig.options.json` sets `composite: true`
   with `outDir` there and no `tsBuildInfoFile`.
   `package/monorepo-task/core-exec/src/tsc-filter.ts` already deletes `dist/**/*.tsbuildinfo`,
   and the root tsconfig redirects its own build info to `.cache/typescript/`.
- pnpm `publishConfig` can override `exports` (and `bin`,
   `main`,
   `types`,
   `module`,
   `browser`,
   `typesVersions`,
   `cpu`,
   `os`,
   `engines`,
   `name`) but not `files`,
   per <https://pnpm.io/package_json>.
   pnpm rewrites every `workspace:` specifier on pack or publish,
   per <https://pnpm.io/workspaces>.
- npm trusted publishing cannot bootstrap:
   `npm trust` requires the package to already exist on the registry.
   pnpm supports OIDC trusted publishing since 10.21.0;
   the workflow installs `pnpm = "latest"` via mise.
   npm's access-token docs steer CI publishing to trusted publishing and away from Bypass-2FA tokens.
- Test files shipped by other packages (`npm pack --dry-run`):
   rambdax 1 of 321,
   zod 191 of 828,
   pino 54 of 195,
   fast-check 0 of 11,
   picocolors 0 of 7.
- `module-caught-value` readiness:
   one source file,
   no workspace runtime dependencies,
   README present,
   all package gates pass.
- The `require-eventual-artifact` rule exempts only test-only modules by name (`fixturePatterns`);
   its README forbids allowlisting behavior modules.
   toml-edit exposes seams as underscore-prefixed entry exports;
   the shared rolldown config builds one entry;
   no `stripInternal` in the shared tsconfig.
- Git tags follow `<package>-v<version>` (17 tags).
   `cargo-publish.yml` combines a tag push trigger with a dispatch choice input.
- The changesets action supports OIDC trusted publishing,
   needs `id-token: write`,
   takes `publish-script` and `version-script` inputs,
   and pnpm's own guide publishes with `pnpm publish -r`.
   No `.changeset/` directory exists yet.
- Prior direction in `doc/handover/cli-git-policies-platform.md`
   says to bundle logger into public artifacts rather than publish it.

## Execution plan

Order matters:
 docs that gate code first,
 code gates next,
 tooling,
 then verification and the manual bootstrap.

1.   Vet report `doc/audit/tech-npm-release-versioning-and-publishing-vet-2026-09-06.md`
     for changesets against release-please,
     semantic-release,
     and manual tags,
     per the `choosing-technology` skill.
2.   Decision doc `doc/decision/npm-publishing.md`:
     supersedes the bundle-rather-than-publish direction;
     records changesets,
     trusted publishing,
     the local bootstrap,
     `./ts` stripped at publish,
     tests shipped,
     `engines.node >=24`.
3.   Logger code gates:
     control-character neutralization module for the console sink with adversarial tests;
     `flushDeadlineMs` with tests that force the deadline;
     underscore seam exports and the 25 tests pointed at the built artifact;
     reword the `logger.ts` throw comment.
     Zero lint findings,
     types and unit tests green.
4.   Manifest:
     drop `private`,
     add `publishConfig` (access public,
     provenance,
     exports without `./ts`),
     `engines`,
     caught-value to `devDependencies`,
     README examples rewritten.
     Version stays 0.0.1 until the changeset (`minor`) bumps it to 0.1.0.
5.   Shared tsconfig:
     `tsBuildInfoFile` outside `dist/final`;
     confirm `tsc-filter.ts` still cleans it.
6.   Changesets:
     `@changesets/cli` as a root devDependency via the catalog,
     `.changeset/config.json` (`access: public`,
     `baseBranch: main`,
     other options at defaults),
     `.github/workflows/npm-release.yml`,
     delete `publish.yml`,
     one changeset file for the logger.
7.   Verification at the consumer boundary:
     `pnpm pack` the logger,
     install the tarball in a disposable non-workspace project on Node 24,
     import the root entry,
     log,
     flush,
     confirm the escape neutralization and the absence of `./ts` on the installed package.
8.   Runbook `doc/runbook/publish-npm-package-first-time.md` for the local bootstrap and `npm trust github`.
9.   Rewrite `bulletproofing.plan.md` as the post-release track and open an issue for it.
10.  Comment on issues #159,
     #306,
     #307,
     and #338 with what changed.

## Progress log (2026-09-06)

- Vet report:
   `doc/audit/tech-changesets-release-tooling-vet-2026-09-06.md` (alternatives waived by owner).
- Decision:
   `doc/decision/npm-publishing.md`.
- Logger code gates landed in commit `1e786539e` with a proven red-to-green guard run;
   manifest and README in `1afde136e`.
- Changesets adopted in `77ba3948a`;
   the release workflow is `.github/workflows/npm-release.yml`.
- Workflow shakedown findings,
   each fixed and recorded in the commit that fixed it:
   the repository Actions allowlist lacked `changesets/*` (repository setting updated);
   the `npm:pnpm` mise backend cannot bootstrap pnpm on a runner (use `mise install node pnpm`);
   tools installed in a step are not on later steps' PATH (exported explicitly);
   `pnpm pack` fails under `dedupeDirectDeps` (pack job installs with the override);
   changesets' auto-format runs the npm dprint this workspace removed (`format: false`);
   the stock changelog line `<hash>: summary` fails the markdown lint (custom `.changeset/changelog.ts`).
- Tarball verified in disposable consumers:
   Node 24 imports,
   logs,
   flushes,
   refuses `./ts`;
   Node 22 warns `EBADENGINE`.
- Runbook:
   `doc/runbook/publish-npm-package-first-time.md`.

## Result (2026-09-06)

- `@monochromatic-dev/logger-core@0.1.0` is on the registry (shasum `f19c6f53`,
   publisher `aquaticat`,
   no provenance);
   a fresh Node 24 project installed it from the registry and logged.
- Trusted publisher registered for `npm-release.yml` (id `4315f78a-aba0-4cdb-9336-b5f6649d5635`).
- Version pull request #478 merged as `dfa0e0d58`;
   tag and GitHub release `@monochromatic-dev/logger-core@0.1.0` created through the GitHub API
   (the workflow reports nothing to do for an already-published version,
   and the cli-git wrapper rejects a manual tag push,
   `doc/troubleshooting/cli-git-tag-push-eagain.md`).
- Lesson recorded in the runbook:
   wait for the npm package index to replicate before merging,
   and create the bootstrap version's tag by hand.

## Next action

None for this release.
The next changeset merged to `main` exercises the CI publish path with provenance for the first time.
Agent work done:
 the post-release track is issue #479;
 issue comments (#159,
 #306,
 #307,
 #338) and the legacy plan rewrite.

## Serialized file writes release, 2026-10-16

The owner requested publication of the logger's pending `0.4.1` release.
This version serializes complete file-sink appends so concurrently queued records larger than a filesystem write chunk
remain valid JSONL in invocation order.
The registry still served `0.4.0` when this request arrived.
The existing `npm-release.yml` attempt
[38837664209](https://github.com/Aquaticats/Monochromatic/actions/runs/38837664209)
failed with `ERR_PNPM_FAILED_TO_PUBLISH` and HTTP 404 for `0.4.1`.
That failure alone does not establish the authentication cause.

The previously verified `npm-publish.yml` workflow now accepts a `module` choice,
`fs-path` or `logger`,
and builds and tests the selected module before packing it with pnpm.
The separate publish job uses npm OIDC and provenance.
Logger repository metadata now points to `Aquaticats/Monochromatic`.
The release kept the pending version `0.4.1`.

Commit `61d38bd7041443b236eea75b0a7f00da7e0d53bc` is the source for
[release run 38838431661](https://github.com/Aquaticats/Monochromatic/actions/runs/38838431661).
Local `lint:types` and `lint:oxlint` passed with zero warnings and errors.
The disposable npm consumer has its own private manifest and an explicit installation prefix.
The workflow succeeded,
including the build and unit tests.
npm accepted the upload,
then made `0.4.1` available at `2026-10-16T02:06:06.290Z`;
the registry's `latest` tag now resolves to it.
The registry identifies GitHub Actions as publisher and
`80363e5d-aede-4258-81ca-d4ee414a90a5` as the trusted publisher configuration.
Its SLSA provenance names `Aquaticats/Monochromatic`,
`.github/workflows/npm-publish.yml`,
and source commit `61d38bd7041443b236eea75b0a7f00da7e0d53bc`.
Tag `@monochromatic-dev/logger-core@0.4.1` preserves that revision.

A fresh npm installation passed version and repository URL assertions.
The installed `createLogger`,
`tagged`,
and `createFileSink` queued four records with 600000-character payloads,
flushed them,
and produced complete JSONL in exact invocation order.
The `/ts` entry remained unavailable to registry consumers as intended.

Next action:
none for `0.4.1`.

## Shared completion and file ownership release, 2026-10-18

The owner authorizes the auto-mode investigation's shared logger fix
and requests a version bump so publication runs.
Version `0.4.2` deduplicates observation of shared pending sink completions,
retains every sink invocation,
and serializes complete file batches without reentrant serialization or same-millisecond filename collisions.
The package-local rationale and runtime limits are in
[`write-completion.md`](../../package/logger/core/doc/write-completion.md).

The built-package unit suite and the type check pass.
Oxlint reports zero warnings and errors across 63 files.
The capacity regression delivers 100,000 records while retaining 48,304 bytes before continuation drainage,
compared with the regression's 93,863,816-byte failing baseline.
Private rebuilt guard mutations fail the completion-identity,
missing-JSON-value,
filename-identity and issued-batch-admission regressions without changing the checked-in source.

Version commit `3befa69c148c609f864fdb200c75cbbb6d9e0cce` triggers
[automatic run 39104844467](https://github.com/Aquaticats/Monochromatic/actions/runs/39104844467).
Its pnpm publish step reports `ERR_PNPM_FAILED_TO_PUBLISH` with HTTP 404,
which does not establish the authentication cause.
The first targeted recovery fails before packing because the generated root Mise configuration has lost
tasks that were never added to its authoritative source.
The [targeted-workflow troubleshooting record](../troubleshooting/npm-targeted-workflow-tasks.md)
records that separate cause and its source-owned repair.

Commit `d6d1d67b069de6dd6cff8dc7543f06bd6fd94caa` is the source named by
[successful targeted run 39106517976](https://github.com/Aquaticats/Monochromatic/actions/runs/39106517976).
The registry serves `0.4.2`,
identifies GitHub Actions and trusted publisher `80363e5d-aede-4258-81ca-d4ee414a90a5`,
and supplies an SLSA statement naming that commit,
`.github/workflows/npm-publish.yml`,
and the successful workflow invocation.

A fresh registry installation in a disposable consumer confirms version and repository metadata,
imports `createLogger`,
`tagged` and `createFileSink` through the published exports,
and confirms the workspace-only `/ts` export remains unavailable.
Independent verified file sinks retain separate files.
Five tagged, large and reentrant records parse as complete JSONL,
with the inner serialized record preceding its outer record.

The release is published and consumer-verified.
The automatic pnpm workflow's HTTP 404 remains a separate unresolved publishing path.
