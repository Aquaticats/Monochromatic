# Task-slice owner vet report

Status:
 complete;
 reopened vet for the chosen candidate
 closed with no disqualifying finding.
Scoring is complete in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md);
 the adoption-depth audit below
 carries no hard-gate failure.

Lifecycle phase:
 validated,
 scored,
 recommended.
The adoption-depth audit
 (license,
 provenance,
 maintenance,
 production source,
 human auditability)
 is filled from the two audits
 recorded on 2026-10-29
 (`~/temp/agent/pants-maintenance-audit-2026-10-29.md`,
 `~/temp/agent/pants-source-audit-2026-10-29.md`).

Subject:
 task slice owner.

Decision scope:
 select the owner for Mise's task-execution slice:
 task discovery,
 per-package task addressing,
 dependency edges,
 task bodies,
 watch,
 failure tolerance,
 and machine-readable listing,
 at 222 packages and 1,811 tasks.

Start date:
 2026-10-29.

Last updated:
 2026-10-29.

Governing skill commit:
`37117e36397e30233062828581a5e38217da9458`.

Governing skill SHA-256:
`552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.

Compatibility fingerprint:
`e4a30abd2c6ffb3c3bb2dfa178aae27a546962df505282d609158352d361a123`.
Built from schema version 1,
 subject,
 decision scope,
 hard constraints,
 deployment,
 trust boundary,
 incumbent,
 base categories,
 and overlays,
 canonicalized by sorted-key JSON
 and hashed with SHA-256.

Active audit owner:
 the agent session that ran the eight pilots
 on 2026-10-29.

Prior compatible report:
 none.
The 2026-09-16 monorepo manager vet
 ([`tech-monorepo-manager-vet-2026-09-16.md`](tech-monorepo-manager-vet-2026-09-16.md))
 carries a different subject
 and a different decision scope
 (file-enforcer,
 watch daemon with RPC,
 cgroup sandboxing,
 one-file shipping),
 so its fingerprint does not match
 and its "no candidate is recommendable" result
 is not reusable here.

## Context

The task slice is one slice of the monorepo manager.
The accepted endpoint
 ([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md))
 names `meow` as the monorepo-manager owner,
 and `meow` is plan-only for the task slice,
 so the slice has no implemented owner today.
The incumbent is Mise,
 measured with two named holes:
 nothing is cached at all
 ("Nothing is cached,
 so the aggregates are slow by construction"),
 and root `mise run test` runs only the three root suites
 and misses 15 packages' tests
 ([`doc/planning/monorepo-manager-route-research/mise-task-usage.md`](../planning/monorepo-manager-route-research/mise-task-usage.md)).

The scoring axis is capability for the slice job.
The user rejected parity with the current status quo
 as an axis
 ("Parity with the current status quo is poor scoring axis.
Assume we're fine with huge huge refactors and everything."),
 so migration and shape cost score nothing
 and are recorded as implementation notes.
The tightening that records this lives in `AGENTS.md` as `OPT`:
 "Score capability for the target workload,
 never fidelity to incumbents."

## Hard constraints

The slice's capability catalog,
 each a hard requirement of the dev loop:

- partial invocation of one package's one task;
- paths containing spaces;
- sequential and parallel groups;
- missing optional package builds;
- intentional failure tolerance;
- tests continuing after build failure where required;
- retained failure causes;
- machine-readable task listing;
- correct aggregate exit status.

The acceptance boundary
 `package/dev-script/task-util/src/build-and-test.unit.test.ts`
 carries the failure-tolerance and tests-after-build-failure duties.

## Base components and overlays

- Base:
  inspectable open-source local technology
  (build tool or task runner,
  installed locally,
  source inspectable).
- Overlay:
  incumbent dependency replacement
  (Mise's task-execution slice).
- Overlay:
  high-trust execution
  (the owner executes arbitrary package task bodies
  in the developer loop and in CI).

No managed service,
 no proprietary component,
 no native or prebuilt artifact without source mapping
 is under consideration.

## Frozen criteria and weights

Nine criteria,
 weight 1 each,
 because the user named no priority.
Full definitions and thresholds are in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md),
 section "Scoring axis".

- A,
  edge fidelity;
- B,
  body model;
- C,
  failure semantics;
- D,
  optional-task robustness;
- E,
  watch;
- F,
  caching and affected selection;
- G,
  machine-readable introspection;
- H,
  scale at 222 packages;
- I,
  maintenance depth.

Documentation trust is a scored deduction of `4 - trust` points,
 not a criterion.

Weight sensitivity is published in the comparison doc
 (duty only,
 duty dominant,
 equal).
Pants is first under every weighting tested.

## Candidate ledger and where its evidence lives

Discovery,
 screening,
 and validation ran as eight measured pilots
 against the task-slice catalog,
 in disposable fixtures whose paths contain spaces.
The per-gate measurements,
 score arithmetic,
 sensitivity runs,
 and per-adjacent-pair reasons are in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md).

- Pants 2.33.1:
  validated,
  scored 32,
  recommended,
  chosen by the user 2026-10-29.
  Adoption-depth audit in this report.
- Please 17.34.0:
  validated,
  scored 26,
  runner-up.
- Buck2 2026-10-14:
  validated,
  scored 26,
  third on trust tie-break.
- Nadle 0.6.1:
  validated,
  scored 25.
- Tilt 0.37.8:
  validated,
  scored 21.
- pnpm 12.16:
  validated,
  scored 19.
- projen 0.104.19:
  validated,
  scored 13.
- Poe the Poet 0.49.0:
  validated,
  scored 12.
- meow as designed:
  not scorable,
  plan-only,
  and the custom-implementation gate
  is not met while Pants clears every named duty.
- Keep Mise:
  incumbent baseline,
  scored 24,
  trust not audited.

## Ranking and adjacent reasons

The full ranking with every adjacent reason
 and its flip condition is in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md),
 section "Why each adjacent pair".
Condensed order:

1.  Pants 2.33.1
2.  Please 17.34.0
3.  Buck2 2026-10-14
4.  Nadle 0.6.1
5.  Keep Mise
6.  Tilt 0.37.8
7.  pnpm 12.16
8.  projen 0.104.19
9.  Poe the Poet 0.49.0

## Adoption-depth audit for Pants 2.33.1

Filled from the maintenance and source audits
 recorded below.
Every claim cites a primary source,
 a repository path,
 or a reproduced command.

### License and provenance

- License:
  Apache License,
  Version 2.0
  (`LICENSE:1-3` of the clone,
  "Apache License / Version 2.0 / January 2004"),
  with the matching header on every source file
  and no `NOTICE` file.
  Compatible with use as an externally executed build tool
  in an Apache-2.0 repository;
  nothing in the Pants tree is linked into our product code
  (the native engine is a standalone `.so`
  loaded by Pants itself).
- Distribution:
  GitHub Releases only,
  10 assets on `release_2.33.1`:
   four wheels
  (`pantsbuild_pants-2.33.1-cp314-cp314-manylinux2014_x86_64.whl`
  and its aarch64 and macosx_14_0_arm64 siblings,
  plus `pantsbuild_pants_testutil-2.33.1-py3-none-any.whl`),
  three pex files
  (`pants.2.33.1-cp314-linux_x86_64.pex`
  and siblings),
  and three scie binaries
  (the same names without the `.pex` suffix).
  PyPI is dead-ended at 2.17.1
  (`https://pypi.org/pypi/pantsbuild.pants/2.33.1/json`
  returned 404 on 2026-10-29).
- Bootstrap:
  the `scie-pants` launcher
  (v0.13.2 assets plus `.sha256` companions)
  reads `pants_version` from `pants.toml`
  and fetches the matching pex
  from GitHub Releases,
  plus a Python Build Standalone interpreter.
- Provenance:
  every release asset carries a GitHub sha256 digest,
  and `.github/workflows/release.yaml`
  (generated from
  `src/python/pants_release/generate_github_workflows.py`)
  runs `actions/attest-build-provenance@v3.0.0`
  over the wheels and the pex and scie binaries.
  The attestations API
  returns a published sigstore-backed bundle
  for the wheel digest
  (Rekor log index 2618865919).
  No cosign signatures
  and no detached checksum manifests,
  so offline verification is weaker
  than the GitHub-hosted path.
  The pex bootstrap tool
  is itself pinned
  (`pex-tool/pex` v2.104.1
  checked against a hard-coded sha256)
  and the Rust toolchain is pinned to 1.99.1.
- Dependency surface:
  36 declared Python requirements
  resolving to 79 locked projects
  with sha256 artifact hashes,
  plus 523 `Cargo.lock` entries
  around a 186-file,
  58,572-line production Rust core.
  The `3rdparty/python/requirements.txt` header
  records a supply-chain policy
  against adding dependencies.
  No same-author Python packages;
  the same-author surface is the two shipped wheels
  and the `scie-pants` launcher repository.
  The explorer quartet
  (`fastapi`,
  `starlette`,
  `strawberry-graphql`,
  `uvicorn`)
  is excluded from the shipped pex.
- Lifecycle:
  no install hooks in the artifacts themselves.
  The repo `pants` script
  bootstraps a venv
  and builds `native_engine.so`
  from `src/rust/engine`.

### Maintenance

- Sample:
  the 10 most recently updated issues
  and the 10 most recently updated pull requests
  of `pantsbuild/pants`,
  plus the 30 most recently merged pull requests.
- Sampling caveat:
   9 of the 10 most recently updated issues
  were touched by `github-actions[bot]`
  adding the `stale` label
  (`.github/workflows/nudgebot.yaml`,
  `actions/stale`,
  366 days,
  `bug` exempt,
  never closes),
  so the issue sample reads
  the inactive non-bug backlog,
  not bug triage.
- Maintainer concentration:
   8 accounts performed maintainer-class actions.
  `MAINTAINERS.md` lists 18 active maintainers,
  but `benjyw` merged 24 of the 30 most recently merged
  pull requests (80 percent)
  and authored 19 of them.
  The release path is single-person dominated.
- Merge latency:
   bimodal.
  Median 0.24 days
  and 26 of 30 merges land within 8 days,
  while external work waits
  47,
  51,
  172,
  and 421 days.
  Every external fix that merged
  was reviewed first,
  so the defect is latency
  and unanswered review or CI-approval requests
  (pull request 23710,
  "Could a maintainer approve the CI run?",
  unanswered),
  not unreviewed merges.
- Release recency:
   `release_2.33.1`
  published 2026-08-27
  and still flagged "Latest"
  at the audit date,
  63 days old.
  Maintenance merges do reach releases
  through `auto-cherry-picker`
  (the October 2026 CA-certificate compromise fix
  landed the same week)
  and through dev releases,
  but nothing from the last two months
  is in a stable release.
- Upstream acknowledgement
  of the three pilot-measured defects:
   none found for any of the three.
  The literal search queries
  are recorded in
  `~/temp/agent/pants-maintenance-audit-2026-10-29.md`.
  Nearest neighbours only:
   issue 11391
  (cancelled workunits not reported,
  open since 2020,
  zero maintainer comments),
  the 2021 to 2022 error-message pull requests
  (15751,
  15788,
  15790,
  11996)
  which acknowledge the `ResolveError` text
  but not the `::` poisoning,
  and no report at all
  that the 30 second default is a defect.
  The 30 second default is confirmed in source:
   `AdhocToolTimeoutField`
  in `src/python/pants/backend/adhoc/target_types.py`
  carries `default = 30`.
  Under the session's drift policy
  this keeps Pants' trust at 1 of 4,
  the deduction already applied
  in the scoring.

### Production source

Evidence from the clone of `pantsbuild/pants`
 at `release_2.33.1`,
 commit `c94f9f4ccfd43342b8f9379a139a328555bc5a5b`
 (`~/temp/agent/pantsbuild-pants-2026-10-29`).
No Pants build or lifecycle script was run.

- Body execution (`adhoc_tool`):
   `AdhocToolArgumentsField`
  is a plain string sequence
  (`src/python/pants/backend/adhoc/target_types.py:153`),
  concatenated element-wise
  with the runnable's own argv
  and never shell-tokenized
  (`src/python/pants/core/util_rules/adhoc_process_support.py:516`),
  then spawned exec-style
  (`Command::new(&req.argv[0])`,
  `.args(&req.argv[1..])`,
  `env_clear()`,
  `PATH=""`;
  `src/rust/process_execution/src/local.rs:303`).
  Caveat on "no shell":
   in the default configuration
  the argv is rewritten to
  `bash -c "cd <workdir> && <shlex.join(argv)>"`
  (`adhoc_process_support.py:899`),
  so a `bash` process is in the tree
  but every argv element
  is POSIX-quoted and cannot inject syntax.
  A local-workspace environment target
  (`LocalWorkspaceCompatiblePlatformsField`)
  removes the wrapper entirely
  and yields a pure exec.
  `shell_command` is shell-based by design.
- Listing (`peek`):
   `peek` emits indented JSON
  (`src/python/pants/backend/project_info/peek.py:184`)
  whose elements carry
  `address`,
  `target_type`,
  `sources`,
  `sources_fingerprint`,
  and `dependencies`
  as expanded dependency addresses
  (`peek.py:152`),
  written to stdout
  or `--output-file`.
  This is the natural discovery feed
  at 222 packages.
- Watch (`--loop`):
   `loop` is a global boolean option
  (`src/python/pants/option/global_options.py:177`);
  `local_pants_runner.py:192`
  reruns the goal body in a bounded loop
  with a fresh run id per iteration,
  and the Rust side
  blocks on filesystem invalidation
  (`notify` inotify or FSEvents,
  `src/rust/watch/src/lib.rs:22`)
  then recomputes only dirty memoized nodes.
- Caching and affected selection:
   `local_cache` defaults to true
  (`src/python/pants/option/bootstrap_options.py:1319`),
  backed by `cache` and `sharded_lmdb`
  (`src/rust/process_execution/src/cache.rs:34`).
  Cache hits are reported,
  not silent:
   `ProcessResultMetadata.source`
  distinguishes `RAN`,
  `HIT_LOCALLY`,
  `HIT_REMOTELY`,
  `MEMOIZED`
  (`src/python/pants/engine/process.py:276`),
  and the `test` and `check` summaries
  print "cached locally" or "memoized"
  per result.
  Cross-run reuse is explicit
  (`source(current_run_id)` returns `MEMOIZED`).
  Affected selection:
   `--changed-since`
  with `--changed-dependents=none|direct|transitive`
  (`src/python/pants/vcs/changed.py:32`),
  shelling out to git
  through `GitWorktree`,
  with the `--all-changed` alias
  recorded in `pants.toml:97`.
- Failure reporting:
   a failed process raises
  `ProcessExecutionFailure`
  retaining exit code,
  stdout,
  and stderr
  (`src/python/pants/engine/process.py:313`),
  with a `--keep-sandboxes=on_failure` hint.
  The measured omission is confirmed in source:
   results whose `result_metadata` is `None`
  (a backend whose own discovery hit an error,
  such as a compilation failure)
  set the aggregate exit code
  and are then `continue`d past
  by the `test` summary loop
  (`src/python/pants/core/goals/test.py:1026`),
  so a dependent of a failed build
  gets no summary line
  while its cause survives
  in the workunit stream and the rerun list.
  The raised-exception path is worse:
   `concurrently` is fail-fast
  (`src/rust/engine/src/nodes/task.rs:181`,
  `future::try_join_all`),
  so one raised `ProcessExecutionFailure`
  aborts the goal before any summary prints,
  surfaced as `ExecutionError`
  rendering only the first throw
  plus "(and N more)"
  (`src/python/pants/engine/internals/scheduler.py:522`).
  Aggregate exit semantics:
   `test` reports the nonzero result
  with the largest magnitude
  (`test.py:1020`),
  `check` the first nonzero per tool
  (`check.py:102`).
  Relevance at 1,811 tasks:
   captured per-task failures
  aggregate an exit code
  but can vanish from the summary,
  and one raised setup failure
  kills the whole run's summary.
  Both matter for the failure-tolerance duty
  and neither is acknowledged upstream.

### Human auditability

- Size:
   1,240 non-test Python files
  and 160,537 lines,
  plus 186 non-test Rust files
  and 58,572 lines.
  The Rust side adds 64 test files
  and 26,037 test lines.
- Dependencies:
   36 declared Python requirements,
  79 locked projects,
  523 `Cargo.lock` entries.
  Zero same-author Python packages.
- Fuzzing and mutation:
   none.
  Searches for `cargo-fuzz`,
  `libfuzzer`,
  `proptest`,
  `quickcheck`,
  `hypothesis`,
  `mutagen`,
  and `cargo-mutants`
  over `src/rust`,
  `build-support`,
  `.github`,
  `3rdparty`,
  and `pyproject.toml`
  returned no declared use.
  Absence is reported inline
  rather than scored.
- Tests and CI:
   521 `*_test.py` files
  (135 integration),
  709 Rust test annotations,
  full Python suites on
  Linux x86_64 (10 shards),
  Linux arm64,
  and macOS 14 arm64;
  Windows is a partial crate list.
  The merge gate `merge_ok`
  requires every substantive job.
  Coverage runs in CI
  and aggregates in Coveralls.
- Type and lint:
   mypy with selective strictness
  (`no_implicit_optional`,
  `strict_equality`,
  `warn_unreachable`,
  not `strict`),
  ruff plus flake8 with repo-owned plugins,
  shellcheck and shfmt,
  and license headers enforced
  by `pants.backend.tools.preamble`.
- Capability concentration:
   process execution
  in `engine/process.py`,
  `core/util_rules/adhoc_process_support.py`,
  `backend/adhoc/`,
  and `src/rust/process_execution/`;
  filesystem and stores
  in `src/rust/fs/` and `src/rust/cache`;
  network and remote
  in `engine/download_file.py`,
  `src/rust/grpc_util/`,
  and `src/rust/process_execution/remote/`;
  credentials
  in `option/bootstrap_options.py`
  (remote oauth bearer token,
  remote client certs,
  remote store headers)
  and the OpenTelemetry exporter header;
  listing and watch
  in `backend/project_info/peek.py`,
  `bin/local_pants_runner.py`,
  and `src/rust/watch/`.
- Trust notes:
   the default `adhoc_tool` path
  carries a `bash -c` wrapper
  (shlex-quoted),
  `runnable_dependencies`
  always generate bash shims,
  and this repo's own `pants.toml`
  enables anonymous telemetry
  with a repo id,
  a consideration for an offline dev loop.
  Source-to-artifact mapping
  for `native_engine.so` is complete in tree,
  while the scie-pants launcher
  and Python Build Standalone
  are external artifacts
  whose provenance lives
  in their own repositories.
- Tradeoff named:
   the tree is large
  and has no fuzzing or mutation testing,
  so verifiability rests
  on the reviewed slice
  (adhoc processes,
  peek,
  loop,
  cache,
  failure reporting)
  rather than on the whole.

## Confidence and evidence limits

- The pilot measurements are fixture-scale
  except pnpm and Keep Mise,
  which were exercised at 222 packages.
- Category judgments on scale
  (Pants,
  Please,
  Buck2,
  Tilt)
  are not corpus probes.
- The Buck2 and Please totals tie at 26
  and the tie breaks on documentation trust;
  a reader who does not weigh trust
  should treat them as co-second.
- The issue sample
  in the maintenance audit
  is dominated by stale-bot label events,
  so it speaks to the inactive non-bug backlog,
  not to bug triage or security response.
- `gh attestation verify`
  produced no output in this environment;
  the attestation evidence
  rests on the GitHub attestations API.
- `wheels.pantsbuild.org`
  returned 404 for both probed paths,
  so the project-hosted index
  is recorded but not enumerated.
- Fuzzing and mutation testing
  are absent from the Pants tree;
  that absence is reported,
  not scored.

## Recommendation

Pants 2.33.1.
The user chose it on 2026-10-29
 when asked which owner he wants
 for the task slice.
This vet finds no disqualifying problem:
 license and provenance pass,
 maintenance is alive
 but concentrated on one merger
 with long external latency,
 the consumed source boundary
 is exactly the slice we need
 and is inspectable,
 and the three pilot-measured defects
 remain unacknowledged upstream,
 which is already priced
 into the trust deduction.

What adoption must carry in,
 because upstream does not:
 a summary line for
 the dependents of a failed build,
 an aggregate exit path
 that survives a raised failure,
 a task timeout above 30 seconds,
 and a watch and cache story
 that the repo configures
 (the launcher,
 Python Build Standalone,
 and anonymous telemetry
 are external or on by default).
Remaining gate before migration:
 explicit acceptance,
 which unlocks the `doc/decision/` record
 and the 1,811-name migration.
