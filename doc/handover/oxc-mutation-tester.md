# Handover: oxc mutation tester (issue 247)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/fs-path` is now `package/fs-path/core`;
> `package/module/test` is now `package/test-harness/core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mutation-test.fixture` is now `package/mutation-test/cli.fixture`;
> `package/cli/mutation-test` is now `package/mutation-test/cli`;
> `@monochromatic-dev/cli-mutation-test` is now `@monochromatic-dev/mutation-test-cli`.

Living doc for the ground-up rewrite of the mutation tester.
Update this file whenever a milestone lands or a decision changes.

## State

- Plan drafted and posted:
   <https://github.com/Aquaticat/Monochromatic/issues/247#issuecomment-4887670850>.
  The plan comment is the authoritative design record;
   issue body Status links to it.
- Implementation not started yet;
   task breakdown exists in the session task list and mirrors the checklist below.

## Decisions (from the design interview, all user-confirmed)

- Deliverable:
   plan,
   then implement in the same arc;
   close issue 247 explicitly when done.
- Full parity including the TypeScript type-check filter,
   not MVP-without-filter.
- Ground-up rewrite of everything;
   `package/dev-script/mutation-test` deleted at the end.
- New home:
   `package/cli/mutation-test`,
   npm `@monochromatic-dev/cli-mutation-test`,
   bin `mutation-test`,
  single package (engine plus orchestrator together).
- Architecture:
   sharded disposable containers with taint-aware re-runs.
  Trust model:
   once the first mutant executes in a container,
   that container is untrusted.
  Shards grouped by source file,
   chunk max initially 16 (engineering guess,
   benchmark later).
  Taint (per-mutant timeout,
   runtime/infra error,
   restore failure,
   container nonzero exit)
  re-runs the shard remainder as fresh half-size shards (bisection;
   position-1 results always final).
- Confirmation always on,
   no flag:
   every Survived and final Timeout re-verified as mutant number 1
  in a fresh container.
   Killed accepted from any position.
- Type-check filter:
   spawn `tsgo --noEmit --incremental --project` per mutant in-container
  (warm `.tsbuildinfo` from baseline).
   Measured 0.125 s warm on `package/module/test`.
  Watch-daemon approach rejected on evidence:
   tsgo 7.0.1-rc watch output has no completion terminator
  and fanotify failed on the dev host.
- No mutation score anywhere.
   Native versioned JSON report:
   statuses killed/survived/timeout/compileError/runtimeError,
  per-mutant provenance (shard id,
   position,
   rerun count,
   confirmed flag).
   Exit 0 unless infra failure.
- Suppression support is a must-have (user request mid-session):
   comment directives
  `mutation-test-disable-next-line [families] [-- reason]` and `mutation-test-disable-file`,
  validated family names,
   suppressed mutants reported in an `ignored` bucket with reasons
  (partially reverses the earlier drop-Ignored decision).
- Mutant enumeration happens host-side with `oxc-parser`;
   containers receive a span manifest
  and never need oxc.
- CORRECTION to the posted plan (probe-verified,
   oxc-parser 0.138.0):
   the JS bindings return
  UTF-16 string offsets,
   not UTF-8 byte offsets.
   `source.slice(start, end)` is exact,
   including past
  astral characters;
   Buffer.subarray at those offsets is WRONG.
   Splice on JS strings,
   never Buffers.
  Mention this when closing the issue so the plan comment's Buffer claim doesn't mislead.
- Operators:
   port semantics of the ~15 Stryker mutator families as parity spec,
   fresh implementation.
- Deleting with Stryker:
   all `@stryker-mutator/*`,
   the whole `@babel` subtree,
   and the `typescript6`
  (`catalog:classic`) alias whose only consumer is Stryker's checker.

## Checklist

- [x] Stryker reference run captured:
       `package/module/fs-path` (NOT async-time:
       old tool needs
      module-logger+module-test deps).
       Reports for 7 of 8 files in scratchpad
      `stryker-reference-fs-path/` (totals:
       9 killed / 319 survived / 2 timeout / 109 compileError);
      `find-monorepo-root.ts` has environment-dependent tests that fail in-container (red baseline),
      so no tool can reference it.
       Two old-tool preflight fixes landed on the way (commits
      a28b34e41,
       862e4ac66).
- [x] oxc-parser probed (UTF-16 spans confirmed) and added to pnpm catalog.
- [x] Package scaffolded per AP1-AP5 (commit 88a611826).
- [x] Engine complete with suppression must-have (user request):
       string-slice splicer,
      deterministic ids,
       work-stack walker,
       15 families,
       mutation-test-disable-next-line/-file
      directives.
       Unit tests pass,
       oxlint 0/0,
       types clean.
       fs-path enumeration:
       523 mutants
      vs Stryker 439 (superset;
       empty.ts exact match 41=41).
- [x] Container-side shard runner (worktree port,
       baseline gates,
       per-mutant loop with
      process-group kill and await-using restore,
       taint semantics).
- [x] Host orchestrator (selection incl.
       integration-test default,
       sharding,
       bisection re-runs,
      always-on confirmation,
       red baseline fails only its shard,
       test-less files short-circuit
      to confirmed survivors,
       native report without score,
       CLI with dry-run).
- [x] Runtime image module (content-hash tag over pnpm-lock + Containerfile,
       podman build reuse);
      root `mise.toml` PATH update folded into the deletion task.
- [x] Fixture sidecar `package/cli/mutation-test.fixture` (naming rule:
       fixture packages live in
      the same category under the same name plus `.fixture` suffix,
       NOT package/test-fixture/)
      plus `test:integration` mise task asserting kill/survive/short-circuit expectations.
- [x] Verification on `fs-path` (not async-time):
       end-to-end run green (14 killed / 470 survived /
      2 timeout / 37 compileError / 0 runtimeError,
       17 shards,
       0 infra).
       Parity vs Stryker reference:
      find-package-root killed 9=9,
       survivors 3=3 with identical lines;
       enumeration superset per file;
      find-monorepo-root newly covered (old tool red-baselined it).
       Known divergence:
       test-less files
      short-circuit to survivors without tsgo,
       where Stryker's checker still classified compileErrors.
      Fixes landed en route:
       containerignore (context bloat + sops secrets),
       source-aware image hash,
      diagnosable baselines,
       .git marker in work tree,
       timeout taint not an infra failure.
- [x] Old package deleted;
       Stryker catalog entries,
       typescript6 alias,
       qs override,
       mise PATH entry purged;
       file-enforcer dead dep dropped;
       file-enforcer and jsonc-edit test:mutation tasks repointed to the new CLI;
       both Stryker troubleshooting docs deleted (described the removed system).
       Lockfile:
       stryker/typescript6 gone,
       @babel down to 7 unrelated leaf packages.
- [x] Issue 247 closed with summary comment.

## Container runner design notes (task in progress)

- Mount conventions to keep (from old `container-args.ts`):
   `/src-ro` read-only repo,
  `/work` writable tmpfs worktree,
   `/out` report mount,
   whole repo baked at `/baked`,
  container entry runs from baked source with plain node.
- Worktree mechanics:
   adapt old `container-worktree.ts` mostly as-is (rsync `/src-ro/` to `/work/`
  with node_modules excludes,
   then recreate root and per-package node_modules symlink farms
  from `/baked`);
   this is proven infrastructure,
   not Stryker-shaped.
- Shard manifest (host writes,
   container reads via mounted file):
   schemaVersion,
   packagePath,
  mutants (id/file/start/end/replacement),
   selected tests,
   timeout floor + factor.
- Shard report (container writes to `/out`):
   schemaVersion,
   shardId,
   baseline timings and result,
  per-mutant results (id,
   status,
   position,
   durationMs,
   detail),
   `unrun` id list for taint-aborted
  remainder,
   optional anomaly description.
- Per-mutant loop:
   splice file in `/work` (string offsets),
   run `tsgo --noEmit --incremental
  --project <pkg tsconfig>` (nonzero = compileError,
   skip tests),
   else run each selected test via
  `node <file>` with process-group kill on timeout (timeout = max(floor,
   factor x baseline test ms)),
  restore original file bytes,
   record result.
   First anomaly (timeout/spawn failure/restore failure)
  stops the loop;
   remainder goes to `unrun`.
- New package's engine exports live in `package/cli/mutation-test/src/index.ts`;
   container code
  must import via relative paths (same package),
   not the package name,
   to run from baked source.

## Late design pivot: in-container build per mutant (user directive)

- Repo convention (user-confirmed):
   tests always test built output,
   except Rust;
   source-importing
  tests are defects to fix (fs-path's were fixed as part of this work).
- Consequence:
   the container mutant loop is splice -> `mise run build` (missing task = skip,
  failure = compileError) -> tsgo whole-project check -> selected tests.
   Baseline builds once.
  The build also materialises dist declarations,
   which fixes TS2307 red baselines from
  package self-reference imports without any tsconfig surgery.
- The old tool never built in-container,
   so its runs on output-importing packages could not
  kill mutants at all;
   parity comparisons must account for that.
- Watchouts:
   /work mise configs are untrusted in the image (set MISE_TRUSTED_CONFIG_PATHS
  or equivalent);
   network=none so mise must never fetch tools (they are preinstalled in /mise).

## Gotchas for future sessions

- The reference run must happen while the old tool still works;
   do not delete `package/dev-script/mutation-test`
  before the report is captured and the comparison is done.
- `rg` reminder:
   `-r` is `--replace`,
   not recursive;
   use long-form flags.
- Old tool invocation surface:
   bin `mutation-test` from `package/dev-script/mutation-test/node_modules/.bin`
  (on PATH via root `mise.toml`);
   requires its dist to be built (`mise run //package/dev-script/mutation-test:build`).
- tsgo probe artifacts:
   a stray `dist/final/types/tsconfig.tsbuildinfo` write happened in
  `package/module/test` during the timing probe (gitignored build cache,
   harmless).
