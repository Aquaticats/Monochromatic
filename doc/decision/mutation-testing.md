# mutation testing

## Status

Accepted.
First consumer:
 `package/file-enforcer/core`.
Second consumer:
 `package/jsonc-edit/core-ts` (parser,
 serializer,
 immutable edit API,
 comment-as-data API).
 Its unit tests are organized by API surface,
 not per source file,
 so it runs with
 `--full-suite`;
 its non-runtime tooling lives in sidecar packages so a whole-package run
 stays scoped to real runtime files.
Reusable package:
 `package/dev-script/mutation-test`.

## Context

`file-enforcer` has parser,
 transform,
 filesystem,
 package-manager,
 and watcher
logic where ordinary example tests can pass while equivalent or weak assertions
survive.
 Property tests already cover broad input generation,
 but they do not
answer whether existing assertions kill small semantic changes.
 Mutation testing
fills that gap.

The workspace uses isolated pnpm linking,
 package-local `node_modules`,
 and
workspace `/ts` imports that Node resolves through real paths.
 Stryker's default
sandbox copy excludes `node_modules`,
 which breaks that layout.
 The repo-pinned
latest Node can run this repo's erasable TypeScript directly,
 so adding `tsx` or
another loader would add a second runtime path that production tests do not use.

## Decision

Use Stryker with the command runner,
 run inside one restricted Podman container
per source file.
 The host owns source-file enumeration,
 outer concurrency,
runtime image readiness,
 and weighted JSON aggregation.
 Each container owns one
Stryker session for one source file.

Dependencies are baked into a local runtime image tagged by lockfile hash and
platform.
 The image starts from `fedora:latest`
and installs the repo-pinned latest Node plus pnpm through mise rather than corepack.
 Bun is intentionally
absent while the repository migrates away from it;
 selected package tests run fixture config files through
Node.
 At container
startup the entrypoint rsyncs current repository source from `/src-ro` into
writable `/work`,
 excludes `node_modules`,
 `dist`,
 and `.git`,
 then recreates
root and package-local `node_modules` from `/baked`.
 Stryker runs in
place from `/work`,
 not from the read-only source mount.

The Stryker TypeScript checker stays enabled with
`prioritizePerformanceOverAccuracy: false` by default.
 Plain Node strips types
but does not type-check,
 so type-invalid mutants must become `CompileError`
rather than ordinary runtime results.
 The CLI has `--typescript-performance-mode`
for measured follow-up runs where the accuracy tradeoff is explicitly accepted.

The default test selection runs related sibling tests,
 related regression tests,
 and
integration tests.
 `--full-suite` runs every unit test for every source file when
stricter coverage is worth the runtime.
Selection also includes sibling sidecar packages (directories named
`<package>.<concern>`) so tooling moved out of a runtime package for source purity
still kills mutants:
 their `*.unit.test.ts` files import the package under test through its
workspace `/ts` subpath,
 which the container resolves through a direct relative symlink to the
mutated work-tree source.

## Rejected alternatives

- Host Stryker with one container per mutant.
   This adds thousands of container
  cold starts,
   creates host/container dependency overlay problems,
   and makes
  Podman timeouts compete with Stryker's own timeout classification.
- Docker fallback.
   The repository's containerized test infrastructure is Podman
  first,
   rootless Podman is the supported local path,
   and a second runtime would
  double the hardening surface.
- `tsx`.
   The repo-pinned latest Node already runs the test files directly,
   and the mutation preflight
  proves that path inside the container.
- Stryker sandbox mode.
   Under isolated pnpm linking,
   sandbox mode drops the
  package-local dependency layout this repo needs.
   `inPlace: true` against a
  disposable writable work tree keeps real resolution without mutating the host.
- Averaging per-file mutation scores.
   Files have different mutant counts,
   so the
  package score is computed from raw status totals.

## Consequences

Mutation testing is opt-in through
`mise run //package/file-enforcer/core:test:mutation`.
 The default run can
be expensive because it type-checks mutants and runs one Stryker session per
source file.
 The one-file command remains the first measurement gate before a
whole-package run.

Reports are written to a host temp directory printed by the task.
 A per-file
container failure does not delete completed reports;
 aggregation reads whatever
JSON reports exist and lists missing reports separately.
