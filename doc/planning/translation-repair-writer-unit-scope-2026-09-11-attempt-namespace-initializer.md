# Writer calibration unit scope: the attempt namespace initializer and its verifier

The exclusive attempt directory with its root plan and identity marker,
the verifier with observed file stability,
the Node runtime ledger that corrected which version the proofs ran on,
and the namespace guard proof.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Attempt namespace initializer and its verification

The journal will retain the approved per-parent seam rather than purchase the broader whole-entry section operation.
Its initial closure is the measured 45 parents,
with 42 queried inputs and 3 structural records.
Xing's unmatched source definitions receive explicit source/namespace accounting only;
that record is neither correspondence nor automatic prompt context.
Future reference-based relabeling still requires qualified current group evidence
and freshly materialized conditional inputs.
The namespace initializer is now implemented in `create-preparation-attempt.ts`,
with fixed-file native I/O in `preparation-attempt-storage.ts` and
`PreparationAttemptError` diagnostics.
It creates a fresh private directory,
writes and file-content-syncs exact `root-plan.json` bytes,
then generates and writes the matching `attempt.json` identity marker.
It never opens or resumes a prior attempt and retains incomplete files on failure.
The storage adapter accepts only the two fixed filenames;
it is not a generic journal record-store interface.

Initial build,
types and focused tests pass.
Formatter findings are named permission constants and a missing handle comment.
Independent review identified input-bearing `SyntaxError` causes,
uncontextualized UUID failure,
circular digest checking and missing completion/failure controls.
The syntax diagnostic now retains only the parser class name,
identity generation is inside the contextualized post-plan operation,
and tests independently hash stored bytes,
retain partial marker failures,
hold write-completion gates and observe native file sync.
The initializer promises file-content sync only,
not containing-directory power-loss durability.
R2 passes every verification phase under Node 26.8.2.
The actual full-suite `unit exit 0` was inspected at line 9840 of
`preparation-attempt-unit-r2-20260912.out`.
The tests include separately held root/identity writes,
native file-sync invocation and completion,
independent SHA-256 of stored bytes,
private rendered syntax failures and retained partial identity bytes.
The native `chmod` is additional permission enforcement;
these tests establish final private modes,
not that removing a redundant chmod alone would change them.
Namespace guard proof remains pending;
no reviewed-phase or crash-durability authority follows from this initializer.
A follow-up moves plan hashing before directory allocation,
and makes concurrent-allocation and retained-directory witnesses fail through ordinary assertions.
R3 passes every phase;
`unit exit 0` was inspected at line 9841 of `preparation-attempt-unit-r3-20260912.out`.
The next namespace operation will verify independently supplied attempt identity and exact root-plan bytes
without treating a marker's own values as expectations.
`verifyPreparationAttempt` now performs that identity/integrity check,
with streaming raw-byte hashing in `preparation-attempt-read-file.ts`.
It checks private directory/file identities,
rejects symlink files and final-directory indirection,
limits each stream to its observed file extent and verifies descriptor/path metadata afterward.
The identity marker's expected size is checked before content reading.
No stored marker values are adopted as expectations,
and no plan JSON is parsed or emitted by the verifier.
It grants neither semantic plan approval nor acquisition authority.
Tests cover real namespaces,
same-length wrong expectations,
missing/changed/partial files,
identical-content symlinks,
private modes,
non-files,
whole-plan multi-chunk hashing and caller expectation snapshots.
R4 passes all verification phases;
`unit exit 0` was inspected at line 9869 of `preparation-attempt-unit-r4-20260912.out`.
Independent review identified unchecked metadata extents,
lossy numeric filesystem identities and relative-path drift across awaits.
Creation now records exact UTF-8 `rootPlanBytes` and returns an absolute namespace path.
Verification validates canonical UUID/digest metadata,
checks the independently supplied byte extent before reading the plan,
and uses BigInt inode/device/size and nanosecond timestamp observations.
This binds total work to the measured plan rather than an unmeasured global size ceiling.
It is observed stability,
not a lease against an adversarial concurrent writer or proof of original creation.

Deterministic stream observers now exercise append,
truncate,
same-size mutation and identical-byte pathname replacement during reading.
Controls also cover marker-first refusal,
one-byte-over-extent rejection before a plan stream,
canonical metadata and cwd changes.
R5 passes build,
types and focused tests including the controlled filesystem changes.
Its formatter findings concern explicit UUID constants,
ASCII iteration and a test disposer statement layout.
The identity scanner now uses a bounded code-unit cursor rather than string spreading,
with named UUID segment dimensions.
R6 passes build,
types and focused tests;
its formatter rejected the scanner's function-root cursor.
The cursor now belongs to the counted loop only.
R7 passes all phases on Node `26.8.2`;
`unit exit 0` was inspected at line 9880 of `preparation-attempt-unit-r7-20260912.out`.
The source checkpoint is `926bb930d`.
Follow-up `33d3b20ab` makes the successful cwd control assert the settled result explicitly,
so a removed path boundary produces an ordinary assertion rather than an uncaught filesystem error.
No semantic phase plan,
lease or paid acquisition authority is introduced by this checkpoint.

The combined removal proof exposed test gaps rather than production changes.
Its first raw-parser-cause mutation survived:
Node `26.8.2` rendered only `private-fi` from the long canary,
so absence of the complete input was not a privacy witness.
`b0824a634` adds a native short-input exposure control and a sanitized-boundary assertion.
The isolated second run detects that removal.
Source trace,
probe catalogs and the discarded filename-collision probe are recorded in
[Node JSON diagnostic privacy canaries](../troubleshooting/node-json-error-privacy-canary.md).

The second proof run also shows that the existing sync-await test can pass with the explicit await removed.
A `returned` flag sampled after one `setImmediate` does not establish that disposal has not already started.
The revised test observes the actual handle's asynchronous disposal boundary,
first proves the observer detects a disposable handle,
and requires no disposal before releasing the controlled sync promise.
R9 passes build,
types,
focused tests,
formatter,
lint and the full suite;
`unit exit 0` was inspected at line 9886 of `preparation-attempt-unit-r9-20260912.out`.
Only Markdown wrapping required correction.
The third isolated run uses base `33d3b20ab` plus the explicitly hashed committed test overlays from `a7dcdc526`;
it stops on the symbol-spy diagnostic formatting failure,
not a counted guard detection.

The first namespace container launch selected Node `26.7.0` and correctly failed its runtime assertion.
An explicit Node `26.8.2` launcher passed the same tool/cgroup/credential-mask checks.
R3 nevertheless shows Node `26.7.0` in the actual failing Mise task children.
This distinction was already recorded for the pairing identity proof;
newer harnesses incorrectly relied on the worker-only check.
The qualification,
receipt and request proof logs also show `26.7.0`.
Those old records remain unchanged as evidence of their actual runtime.

R4 pins `MISE_NODE_VERSION=26.8.2` for the experiment
and supplies a `NODE_OPTIONS` preload to inherited Node environments.
The preload records version,
executable,
entry file and phase,
and refuses another version.
The worker requires a matching observation for every designated test file.
A native Mise control succeeds on `26.8.2` and refuses `26.7.0` with the expected runtime diagnostic.
The source trace and verified override are recorded in
[Node command contexts](../troubleshooting/node-lookup-in-agent-command-context.md).
No lockfile or installed runtime was modified.

R3 also exposed Sinon diagnostic formatting rather than an ordinary failed assertion:
`Cannot convert a Symbol value to a string` arose when the symbol-named disposal spy's matcher failed.
`b197a67f9` compares its primitive `callCount` instead.
R10 passes every source and Markdown phase;
`unit exit 0` was inspected at line 9885 of `preparation-attempt-unit-r10-20260912.out`.
Its per-child runtime ledger is `preparation-attempt-runtime-r10-20260912.jsonl`.
R4 reached the intended ordinary assertions for 25 removals,
but its relative-path witness still passed after removing the pin.
The former relative path resolved to the same namespace from both working directories.
`2d62d0c61` constructs independently owned cwd locations at different depths
and explicitly proves that resolving the supplied relative path from the changed cwd gives a different location.
It also tests a self-consistent malformed UUID marker and matching expectation,
not only the early diagnostic for malformed input.

Independent review prompted fresh exclusive runtime ledgers and a per-invocation identity,
with each designated file's runtime observations retained in the final report.
The successful and deliberately older Mise runtime controls are themselves checked against their ledger rows.
Coverage is the observed task shells and every designated test-entry process,
not hypothetical descendants that clear the preload environment.

R11 passes all verification phases;
`unit exit 0` was inspected at line 9884 of `preparation-attempt-unit-r11-20260912.out`.
Its runtime ledger is `preparation-attempt-runtime-r11-20260912.jsonl`,
run identity `6d7c04be-110f-43fd-a078-9b2a6657fa14`.
R5's complete isolated proof passes all 26 registered successful mutant builds
and designated ordinary assertions,
then restores source hashes,
rebuilds and passes the namespace suite.
Every designated test entry and observed task shell is verified on Node `26.8.2`.
Peak memory is 794329088 bytes,
with no OOM or PID-limit event.

`namespace-guard-proof-20260912` retains 261 hash-checked files,
including failed revisions,
committed test overlays,
per-invocation runtime records and the final assertions,
plus cleanup audit and removal evidence.
The audit found only owned generated paths and the committed test overlays,
with no root Git sentinels.
Directory and registration absence are verified at `2026-09-12T23:39:16.154Z`.
The fixed namespace reader/initializer boundary is verified;
task 41's semantic phase plans,
leases,
source authority and acquisition journal remain unfinished.
Combined file-indirection and post-read metadata removals prove their joint boundaries,
not necessity of every redundant mechanism separately.

The harness-diagnostic follow-up is complete:
[called symbol-spy diagnostics](../troubleshooting/sinon-called-symbol-spy-diagnostics.md)
records the source path,
consumer workaround,
bounded disposable prototype and local additive-comment draft.
No upstream communication or installed dependency edit occurred.
