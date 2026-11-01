# Writer calibration unit scope: lint and formatter resource blockers met while verifying the input runner

Tasks 48 and 49:
the Oxlint memory kills and heap limits found while verifying the runner,
the per-run profiles that completed,
and the retained formatter proof with its cleanup.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Lint and formatter resource blockers of tasks 48 and 49

Current package verification is blocked on task49.
`devformat-TYkzp1` reaches its 600-second deadline after six observed Oxlint process lifetimes,
leaving 21 source files changed and no final convergence result.
Its final sample has no OOM event.
A separate read-only `devlint-ONOp7h` is memcg-killed instead,
with the kernel naming its tsgolint process.
These remain separate incidents.
No rules,
source files or type-aware checks are disabled.
The source checkpoint is committed as `7c106717c`;
it includes native `Error.isError` replacements and still needs semantic auto-fix review and fresh builds.
The Node `192MiB` control aborts;
its instrumented repeat identifies an Oxlint JavaScript heap OOM,
not a cgroup OOM.
Report PID `36` is matched to the actual Oxlint entry and heap/process ledgers.
The Node `224MiB` read-only control completes analysis:
48 warnings and 8 errors,
1538 files and 484 rules,
with no OOM/PID events or source changes.
Its findings are addressed through module extraction,
readonly callback ownership,
unknown abort-reason validation,
asynchronous CLI fixtures and bootstrap-configuration formatting.
Changes are recorded in `6e2ad798a`,
`3ce4ece58` and `96d58ac9e`.
Fresh normal/bootstrap builds,
types and the focused CLI/names-only suite pass.
`devformat224-2vLWZD` finishes analysis without OOM/PID events,
but still reports the inspection-module line limit,
a mapping callback layout and the CLI fixture's `promisify(execFile)` void-return mismatch.
Commit `f57cf9c6e` extracts bind validation,
formats the callback and gives native CLI completion explicit ownership of exit/stdio results.
Fresh normal/bootstrap builds,
types and `devtest-40QnXP` pass.
The next full formatter run,
`devformat224-3ssI7u`,
also finishes without a memory failure.
It reports a named callback declaration outside the permitted callback-expression syntax,
an explicit nullish error type and a deprecated Node type alias.
The correction passes the callback expression directly to `execFile`,
accepts its error value as unknown and narrows it before use.
Unexpected non-Error values become a fixed names-only fixture error.
Native spawn-failure and signal-termination rejection controls are added beside the existing exit-zero/exit-six cases.
Fresh normal/bootstrap builds,
types and focused `devtest-WY17PT` pass,
including actual `ENOENT` and `SIGTERM` rejection controls.
The full-package read-only `devlint224-7spwFf` run finishes without a memory failure,
but `promise/prefer-await-to-callbacks` still rejects the adapter.
The fixture now uses the existing `nano-spawn` asynchronous utility instead of custom callbacks.
It clears inherited environment names before adding the fixture environment,
checks ordinary exits separately from native failures,
and documents the utility's final-newline normalization.
An environment-canary control joins the spawn/signal tests.
Fresh builds,
types and focused `devtest-RNODLX` pass,
including environment isolation and actual native failures distinguished from deadline cancellation.
The subsequent full `devlint224-rTGEaU` run is memcg-killed,
so the Node-heap setting is not treated as a reliable general remedy.
The `GOMAXPROCS=1` control completes full-package lint with zero warnings and errors.
A disposable source-only overlay detects the designated TSDoc and unawaited-promise findings under the same settings.
The unchanged-source run then returns zero findings again.
`input-current-lint-proof-20260913.json` reconciles source/configuration hashes and the single changed control file.
Task49's lint blocker is verified without suppressions or repository-wide memory defaults.
Task47 resumes current artifact/full-suite verification,
remaining native boundary controls,
proof retention and owned diagnostic cleanup.
The independently prepared full-unit harness is `run-current-input-full-unit-20260913.mts`;
it supplies the default-HOME corpus mount plus a test-entry runtime ledger.
Managed process `proc_4603` now runs the exact current types,
normal build,
bootstrap build,
runtime seal and complete default unit suite in sequence.
Its owning log is `input-final-verification-20260913.out`;
per-stage logs are `input-final-<stage>-20260913.out`.
Read the actual `unitExit` and expected/observed test-entry ledger result,
not only the outer process status.
The original memory/CPU/PID bounds remain unchanged.
The passing native lifecycle artifacts predate these source changes and do not verify their exact bytes.

Remaining task47 work includes repeated interruption,
filesystem races/collisions,
final current-artifact and complete-suite verification,
then proof retention,
independent review and owned diagnostic-container cleanup.
The owner `mise.lock` and all corpus files remain outside the edit/staging scope.

Task 48 addresses a separate formatter-resource verification blocker.
The first new type check reports `TS2339` after guards narrow away additional runtime-file fields;
reordering the checks repairs those errors and the next type check passes.
The subsequent formatter reports ASCII-string spreading,
a mutable mount callback view,
a record assertion and single-line callback findings.
Structural fixes are committed at `c00525e056fe39d2a8b995ccd6260507fcd1c136`.

The bounded `producer-input-gate-check-1PX5aV` run then fails in `format:oxlint --fix`,
not the previously measured lint-only operation.
It records `memory.peak=2147483648`,
`oom_kill=2` and the Oxlint wrapper's `SIGKILL` diagnostic.
It also lacks the read-only Mise plugin mount and logs attempted `vfox-cmake` discovery.
No exact kernel/container victim correlation has yet been recorded for this incident.
The process stops before its type-check and build phases.
Its partial source diff contains formatter edits and is reviewed separately;
`mise.lock` remains the owner's unrelated change.
The disposable follow-up adds the known plugin mounts,
fresh owned config and package builds,
source hashes,
retained container inspection/events,
host PID maps and kernel evidence.
Its first unchanged-source 512 MiB lint control hits the named container memory limit.
The 256 MiB lint run and subsequent 512 MiB reversal both complete with style warnings and no OOM.
The reversal therefore does not prove that lowering the Go target alone remedies the failure.

With one Oxlint worker and `GOMEMLIMIT=256MiB`,
`format256-lcie4M` completes all 1515 files and 484 rules with zero findings,
peak 1927979008 bytes and no OOM/PID events.
The three formatter-modified files are committed as `522f7dc62`;
committed blob hashes independently match the transferred results.
The exact development type check passes at peak 984588288 bytes.
Deliberate missing parameter documentation and an unawaited sync operation produce
`tsdoc(require-param)` and `typescript(no-floating-promises)`.
Exact restoration precedes `format256-60YMRl`,
which succeeds with unchanged source hashes,
zero findings and peak 1893507072 bytes.
This is a measured per-run profile,
not a new repository default or proof of an optimal or deterministic memory remedy.

`input-formatter-proof-20260913` retains 171 verified copies and 21 separately indexed supplemental records.
The pre-removal audit checks 12979 filesystem entries,
49 empty directories and no root Git sentinels or foreign ownership.
Nine stopped owned containers are removed and checked absent.
Native temporary Git configuration and descendant trust are reviewed,
disclosed and revoked before cleanup.
The worktree directory and Git registration are both absent at `2026-09-13T17:56:04.692Z`.
Task 48's verification is complete.
Task 48 finished before the production host/child bootstrap was implemented;
the later task 47 CLI evidence is recorded in this section.
No runner tests,
full suite,
actual CLI launch or paid preparation follows from the formatter proof.
