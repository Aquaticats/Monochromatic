# Current CI baseline register

Status:
implementation work item 1,
opened 2026-10-18.
This register records what the current static,
unit and build profiles actually do.
It is not a passing baseline.
No profile here is a gate.

## Pinned revision and owner map

The baseline revision is `4f3d5a65a776355f55d32e1334109c05debfaa09`.
The worktree was clean for the measurement aside from dependencies.
The owner map at that revision is 225 package directories,
210 active,
13 paused and 2 archived,
with 14,301 tracked files.
The source-owned task declarations are 2,600,
expanded by `mise` to 2,464 records.
The evaluation's 223,
208,
13,860 and 2,425 counts are historical for its frozen cohort.
They are not a current starting scope.

## Static profile

Command:
`mise run lint:monochromatic-lint`.
Exit code: 1.
Affected files:
1,046.
Findings:
85,719.
The dominant classes are:

- `markdown/semantic-line-breaks`:
  81,700.
- `rust/require-rustdoc`:
  2,077.
- `markdown/no-bare-urls`:
  1,894.
- `rust/max-lines`: 27.
- `markdown/heading-increment`: 5.
- `markdown/no-trailing-punctuation`: 4.
- `markdown/link-image-style`: 3.
- `markdown/link-image-reference-definitions`: 2.
- `markdown/fenced-code-language`: 2.
- `markdown/single-h1`: 1.

Exact per-finding identities are in
[the compressed static log](../audit/depot-current-baseline-static-2026-10-18.log.gz).
Disposition:
the fast static subset is changed-path scoped.
The whole-tree result is an advisory baseline,
not a gate.
The selector and the exact lint configuration are work item 2.

## Unit profile

Command:
`mise run test:unit`.
Exit code: 1.
Named failing test files: 751.
The exact repository-relative file identities are in
[the unit failure list](../audit/depot-current-baseline-unit-files-2026-10-18.json).
Distribution by package family:
`package/module` 284,
`package/pi-plugin` 260,
`package/cli` 51,
`package/oxlint-plugin` 40,
`package/dev-script` 27,
`package/agent-harness-shared` 25,
`package/pi-shared` 19,
`package/config` 10,
`package/mcp` 8,
`package/claude-code-plugin` 6,
`package/cloudflare-worker` 6,
`package/desktop-app` 6,
`package/build-tool` 3,
`package/git` 3,
`package/music-player` 2,
`package/rolldown-plugin` 1.
The raw log is in
[the compressed unit log](../audit/depot-current-baseline-unit-2026-10-18.log.gz).
Observed failure classes:
1,979 `ERR_MODULE_NOT_FOUND`,
987 `Cannot find`,
354 `Unexpected token ... is not valid JSON` and
94 AggregateErrors.
The first failures reference missing `dist/final/node/index.mjs`.
The run executed without built prerequisites,
so the 751 names are a missing-prerequisite cascade,
not product failure evidence.
A sampled JSON-parse failure comes from
`package/oxlint-plugin/test-support/src/index.ts:331`,
where `JSON.parse(stdout)` receives a `Failed to ...` child-output string.
That is a harness or child-command boundary,
not a product assertion.
No unit case is converted to a pass by this register.

Disposition:
this run is invalid as a product baseline.
Rerun the unit profile after a successful build,
then partition the remaining failures by cause.
No product failure is attributed from this run.
This is work item 1 and its acceptance artifact.

## Host boundary

The first bounded build rerun was started on the shared development host and then stopped.
That was outside the plan's execution routing and stressed the host.
Full build and unit baselines run only in the bounded CI environment or a resource-capped
container,
never on the shared development host.
The aborted attempt produced no failure classification.

## CI baseline runs

The first bounded CI baseline run is GitHub run `39196063655` on the Depot-managed
`depot-ubuntu-24.04-8` runner.
It failed in the dependency-preparation step after 3 minutes,
before any build or unit work.
`mise install` installed 58 of 59 tools and failed on `pipx:slopo@latest` because the dependency
sidecar `.mise/locks/pipx-slopo/0.9.0` is absent on a fresh checkout.
That is an environment and tool-install failure,
not a product failure.

The corrected run is GitHub run `39196377166`.
It installs the repository tools with `continue-on-error`,
then prepares dependencies,
builds and
runs unit with automatic tool installation off.
The slopo sidecar failure is retained as tooling noise.
It completed in 13 minutes 18 seconds.
The build failed and the unit step was skipped.
Observed build failure classes include a `glib-sys` custom build-command failure,
a Podman image build failure,
a `cxx-qt` panic,
`cargo vendor` download failures,
`gh repo clone` and `gh api` exit 4,
and an unresolved `canvg` import in `jspdf`.
Those are environment,
tooling and prerequisite failures,
not product attribution.
The build step is marked `continue-on-error`,
so the run conclusion is success while its annotations report exit code 1;
the workflow was corrected to fail honestly in the next run.

Run `39197594870` provisioned `pkg-config`,
`libglib2.0-dev`,
`qt6-base-dev`,
`qt6-declarative-dev`,
Podman and the workflow token,
and removed `continue-on-error`.
It failed at Build after 14 minutes 26 seconds;
the unit step was skipped.
The install step still failed on `pipx:slopo` and `vfox:cmake`,
and installed 57 of 59 tools.
The build failure classes were:

- system-library build scripts for `glib-sys`,
  `gdk-pixbuf-sys` and `cairo-sys-rs`;
- a Podman image build failure;
- `cargo vendor` download failures for `anstyle-wincon` and `serde_json`;
- unresolved workspace imports of `@monochromatic-dev/logger-core` and
  `@monochromatic-dev/async-time-core`;
- Android and design build failures;
- an unresolved `canvg` import in `jspdf`.

The full log is in
[the CI baseline log](../audit/depot-current-baseline-ci-39197594870-2026-10-18.log.gz).
These are environment,
toolchain,
build-order and dependency failures,
not product attribution.
A generic Depot GitHub runner is insufficient for this build profile.
The baseline needs the prepared environment described by the evaluation or a documented
prerequisite image.
The unit baseline remains invalid until that build succeeds.

The prepared native complete baseline is run `2ls35xfpds`.
It installed glib,
gdk-pixbuf,
cairo,
pango,
GTK,
Qt6 and Podman before the build.
The build still failed on 13 owning tasks,
including rootless Podman user namespaces,
cargo vendor downloads,
the music player design capture precondition,
the Android build in the complete fanout,
the desktop application,
the native plugin,
the static site,
the terminal release library order,
the `canvg` dependency and the Wayland session build.
The unit step was skipped because the build failed.
The failure classes and dispositions are in
[the prepared native baseline evidence](../audit/depot-native-baseline-2ls35xfpds-2026-10-19.evidence.json).
The complete build is still not known-good.
The unit baseline remains invalid.

## Fast subset

The first fast subset is five owning build tasks and their tests:

- `//package/logger/core:build` and `:test:unit`;
- `//package/upstream-fork/p-map:build` and `:test:unit`;
- `//package/toml-edit/core:build` and `:test:unit`;
- `//package/forbidden-strings/cli:build` and `:test`;
- `//package/linter/monochromatic-lint:build` and `:test`.

This subset avoids the desktop,
Android,
Qt and Podman packages that make the complete `mise run build` profile fail on a generic
runner.
The complete profile stays a weekly,
prepared-environment workload.
GitHub run `39199930641` completed in 10 minutes 17 seconds.
The `Build fast subset` and `Unit fast subset` steps both succeeded.
The only annotation is the tolerated `Install repository tools` exit 1 on `pipx:slopo` and
`vfox:cmake`.
This makes the five-owner fast subset the first known-good build and unit baseline.
It is not a whole-repository baseline and it is not a gate.
The log is in
[the fast-subset CI log](../audit/depot-fast-subset-baseline-39199930641-2026-10-18.log.gz).

## Advisory profile

`.github/workflows/advisory-core.yml` implements the default-branch advisory profile.
It runs on a push to `main`,
on pull requests to `main`,
and on dispatch.
The gate admits 2 starts per rolling minute,
4 per hour and 8 per day,
and records the newest skipped tip as pending.
Pull-request runs do not consume the push budget;
manual dispatch bypasses it for qualification.
The core job runs on GitHub-hosted Linux,
uses the changed-path static selector,
and runs the five-owner fast subset build and unit tasks.
Results are advisory;
no required checks and no branch-protection change follow.
The first admitted run was `39201617300`.
Its changed-path static step failed on pre-existing semantic-line-break findings in
`doc/handover/slint-ide-0x.md`,
a changed file in the pushed range.
The old workflow stopped there and skipped build and unit.
The workflow now lets static and build continue,
runs unit after a successful build,
and fails once at the end if any profile failed.
The first run is therefore an observed changed-file lint failure,
not a product assertion or a workflow success.
Manual qualification run `39202293518` completed successfully:
the gate admitted the dispatch,
the changed-path static step reported no changed static files,
the build and unit fast subsets passed,
and the final enforce step passed.
That verifies the all-profile execution and the manual qualification path.
Push admission remains limited to the 2/4/8 window;
the first admitted push is the lint finding.

## Build profile

Command:
`mise run build`.
Exit code: 1.
Failed tasks:

- `//package/config/lfs-r2-worker:build:js:browser`.
- `//package/wg/cli-allowedips:build:js`.
- `//package/file-enforcer/core:build:js`.
- `//package/monorepo-task/core-exec:build:js`.

Observed errors include `rolldown exited with non-zero status: killed by SIGABRT`,
`node exited with non-zero status: killed by SIGABRT` and
`Assertion failed: (0) == (uv_thread_create(t.get(), start_thread, this))`.
The raw log is in
[the compressed build log](../audit/depot-current-baseline-build-2026-10-18.log.gz).

Disposition:
the SIGABRT and thread-creation results are consistent with host-resource or
concurrency pressure under the parallel fanout.
They are not attributed to product source.
A rerun in the bounded CI environment is required before any build failure is classified.

## Triage queue

1.  Re-run the build in the bounded CI environment and capture per-task logs.
    Classify each failed task as product,
    harness,
    environment or resource.
2.  Rerun the unit profile after a successful build,
    then partition the remaining failures by cause.
    Separate the 354 JSON-parse errors and 94 AggregateErrors from product assertions.
3.  Select the changed-path static selector and its exact lint rules.
    Confirm the whole-tree baseline remains advisory.
4.  Record the refreshed owner map and the three baselines in the implementation evidence.
5.  Set the relative target baseline from these measurements;
    do not claim a speed target from a
    failed baseline.

## Acceptance and limits

Acceptance:
every observed failure class has a disposition,
the baseline commands reproduce the
recorded exit codes,
and the exact identities are linked.
This register does not claim product failure attribution,
complete coverage or a passing CI state.
It records the state the rollout starts from.
