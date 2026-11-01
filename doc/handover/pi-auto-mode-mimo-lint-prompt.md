# MiMo lint handoff

## Objective and scope

MiMo's task is to clear the complete maintained lint scope for `package/pi-plugin/auto-mode`
while preserving the verified native execution and review behavior.
MiMo runs through a supervised Pi RPC host rather than requiring a manual handoff.
The live worker has actual lint-editing scope.
Its first coherent checkpoint is the CLI network guard,
the CLI process consumer and `src/native-process-network.unit.test.ts`.
The supervising agent owns native cancellation,
compaction,
history and content-storage sources until their current verification finishes.
The complete replay corpus passes behavioral verification on `native-release-mmaT8k`;
later native input repairs are built as `native-release-EWv4wy`.
See the [current frontier](pi-auto-mode-current-frontier.md) for the active incident and evidence location.

The 2026-10-18 full Oxlint run reports **83 errors and 13,834 warnings**
across **818 files and 486 rules**, using one thread.
This is the measured pre-control-regression snapshot,
so run full lint again after the assigned checkpoint before selecting remaining clusters.
The previous intermediate snapshot reported 30 errors and 13,163 warnings.
Phase extraction and its genuine host fixtures add findings while removing retired protocols;
this is an explicit increase in diagnostic debt, not a clean-lint claim.

The exact diagnostics and per-file error index are private:

- `${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/lint-logger042-mmaT8k.log`.
- `${HOME}/temp/agent/pi-mimo-lint-approval-20261017/final-lint-errors.json`.
- `${HOME}/temp/agent/pi-mimo-lint-approval-20261017/final-lint-summary.json`.

## Starting point

1.  Read the [package architecture](../../package/pi-plugin/auto-mode/doc/architecture.md)
    and the [implemented compact design](../../package/pi-plugin/auto-mode/doc/compact-review-design.md).
    Each major responsibility links to its callers, invariants, alternatives, and verification.
2.  Select a coherent error cluster from the measured error index.
    Regenerate the private index from the fresh full diagnostic log rather than treating a historical index as current.
    Native phase fixtures,
    the shared dispatcher and phase callbacks are separate candidate clusters.
    The linter rule source and actual borrowed API determine a valid restructuring.
3.  After a coherent change, the package type check and affected consumer tests establish preservation.
    Execution changes also need a native build before their consumer tests.
4.  Completion requires a fresh full-package lint result with zero errors and warnings,
    passing type checking and affected native tests,
    and a report of commits and exercised boundaries.

The configured commands are:

```sh
# Repository: package/pi-plugin/auto-mode/mise.toml
mise run //package/pi-plugin/auto-mode:lint:oxlint
mise run //package/pi-plugin/auto-mode:lint:types
mise run //package/pi-plugin/auto-mode:build
mise run //package/pi-plugin/auto-mode:test:unit
mise run //package/pi-plugin/auto-mode:test:native:phases
mise run //package/pi-plugin/auto-mode:test:native:decision-fallback
mise run //package/pi-plugin/auto-mode:test:native:execution
```

The lint task already bounds Node and Go workers.
Verification uses disposable state and the repository's resource limits.
An explicit filename subset supports local iteration;
the no-argument lint command is the completion measurement.
Rule severity and coverage remain unchanged.

## Verified behavior to preserve

The verified complete replay build is `native-release-mmaT8k`.
The current lint host pins `native-release-EWv4wy`.
The canonical frontier records its live fixture and controller;
do not reuse a closed diagnosis host pinned to an earlier release.
Both direct and child groups use Pi's actual preparation, execution, and finalization bodies.
The same prepared group owns automatic review, manual confirmation, membership, and execution maps.
Native slots expose phase retention; the retired `invokeProgramNative` API and whole-call adapters are absent.
Restoring them to satisfy fixture typing would restore removed architecture.

The complete `mmaT8k` baseline has passing full units and type checking.
Its native lifecycle catalog passes all 22 modes,
including actual child preparation, hidden-property rejection, hook blocking, and cancellation.
The newer progress/fallback catalog also passes on `EWv4wy`,
including exact-group Allow after native progress and source/cancellation rejection controls.
The current full-unit run fails on the known stale network fixture's missing private receipt path;
that is part of the worker's first editing checkpoint.
The exact committed-source type check and new storage,
compaction and persisted-content consumers pass.
Logger destination tests and the full logger unit suite also pass.

The faithful configured replay `pi-mimo-failure-rpc-7vxnGu` completes all 52 original calls,
including both reported groups,
their preceding conversation and intermediate process output,
with zero selectors and 62 actual live-Luna requests.
The separate fragmented-thinking stress replay `pi-mimo-failure-rpc-LD83ST` passes the same call and request counts.
Its synthetic character fragments are not claimed to reproduce unknown original network chunk boundaries.
The actual transmitted requests pass positive-controlled payload inspection:
raw structured commands, thinking, and signatures are absent.
The prohibition replay `pi-hyper-rpc-allow-cU2EQs` withholds both process/Bash members
with zero selectors and three actual Luna responses.
The private replay retains configured extensions, skills, tool catalogs, and preceding turns.
Configured Stop,
cancel and abort controls each show one selector,
withhold both members and finish normally.

The broader affected native matrix passes,
but the additional session-action catalog fails at the tree event's real `AbortSignal` field.
`native-event-controls.test.mjs` reproduces `SessionInputCaptureError` after the original callbacks succeed.
Repair must preserve the original signal and callback identity,
ordered immutable control observations,
and strict ordinary message/tool data validation.
Navigation completion with a retained capture-error diagnostic does not pass this boundary.

The process-local denial mutation removes both independent disposition barriers.
The real native test then executes three harmless fixture commands and fails with `3 !== 0`.
Production file hashes remain unchanged.
Private replay and mutation tasks live in the investigation scratch `mise.toml`.

Luna remains `openai/gpt-6-luna-decisions` with cutoff `0.60`.
Assessments retain three sequential bundles of up to three parallel calls,
at most 200 questions per call, and the existing eligible one-retry allowance.
Fifteen seconds remains advisory.
Clause, heading, unit, and whole-operation fallback semantics remain distinct.
Source identity, exact membership, tool freshness, cancellation, callbacks, event ordering,
scoped Allow/Stop, headless withholding, the fixed program guard, and issued-work drain remain required.

The root `TCV` rule is unchanged by the user's decision.
The package-local [review-input lesson](../../package/pi-plugin/auto-mode/native/review-input-verification.md)
records actual provider-payload verification.

## Storage and adjacent change

The post-cleanup snapshot measures 22,933,936 regular-file bytes,
including the active 8,214,937-byte native output and a verified rollback.
The measurement excludes shared dependency symlink targets.
The pre-control-regression source inventory contains 66,392 parser-counted code lines,
including 29,291 verification lines.
The user explicitly did not adopt 13k as a line target.

Historical logs are losslessly archived outside the package.
Earlier maintenance removes 43 inactive release copies after consumer checks;
the later reviewed cleanup removes eight more inactive releases.
New package tasks route logs through the existing shared logger using `MONOCHROMATIC_LOG_DIRECTORY`.
The shared logger repair is released as `0.4.2`.
Its build,
unit suite,
type check and zero-diagnostic Oxlint run pass.
The published registry consumer verifies tagged,
large and reentrant JSONL records and independent file ownership.
The version bump triggers automatic publishing;
the known automatic pnpm HTTP 404 requires the existing targeted npm trusted-publisher recovery,
which succeeds with registry provenance.
Automatic release eviction and log retention are not implemented.

The [current frontier](pi-auto-mode-current-frontier.md) retains incident evidence,
commits, failed experiments, and historical measurements.
The original second group's discarded transport cause remains unknown.
Desktop `notify-send` termination is a separate incident.

## Supervisor completion boundary

The worker reports a coherent scoped commit and fresh zero-finding lint result.
The supervisor reviews the actual patch,
rebuilds its native artifact,
runs the actual CLI process and network unit,
then continues the full-package queue without a manual handoff.
Lint success alone does not establish native behavior.
Final completion still requires full lint,
appropriate complete tests,
both configured replays,
required/excluded request audits,
reviewed artifact cleanup,
dependency-aware measurement and rendered package documentation.
