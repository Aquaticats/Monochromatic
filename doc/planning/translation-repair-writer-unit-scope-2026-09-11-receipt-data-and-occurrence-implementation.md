# Writer calibration unit scope: the receipt data reader and fresh occurrence reconstruction

The versioned terminal receipt envelope,
the readers that rebuild an occurrence from complete documents,
the verification rounds under two Node versions,
and the guard-removal proof that closed the receipt boundary.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Receipt data and fresh occurrence implementation

`345d7f7d4` defines a versioned terminal receipt data envelope and fixed receipt diagnostics.
`52fdd006b` adds `readPreparationReceipt` and `readPreparationOccurrence`.
These compare unknown receipt data against independently supplied plan/configuration and current document expectations.
They retain only raw final seat outcomes,
reparse complete documents,
select the registered parent and replay current definition exemptions,
media ownership,
ordered findings and native preparation handoffs.
Returned `scope: 'receipt-bound-occurrence'` is not pairing qualification or writer admission.

Independent review identified missing explicit section correspondence and discarded alignment findings.
The native pre-block constructor is already `alignDocumentSections`;
`d786cd757` forwards its optional `sectionPairing` from the independent occurrence expectation
and retains its findings separately from block-round findings.
Sealing and final slicing options do not enter the native pre-block constructor;
final child construction remains separate.
That commit adds actual-stage/injected-HTTP parity,
receipt refusal,
source/target/parent binding,
ownership and actual footnote-transition reuse tests.

The first verification rebuilt successfully,
then TypeScript reported missing logger arguments at `assertPairingSeats` calls.
The receipt reader now requires and forwards the caller logger.
`preparation-receipt-verification-r2-20260912.out` passes build,
types and focused tests,
including the actual footnote transition and registered section-alignment controls.
Its formatter stops on destructuring,
shadowing,
statement-layout and fixture `Object.setPrototypeOf` return-type findings.
Those findings are corrected without changing receipt acceptance policy.
`preparation-receipt-verification-r3-20260912.out` passes build,
types,
focused tests,
formatter,
rebuild,
types,
zero-warning lint,
full unit and Markdown lint.
The full-suite terminal was inspected at line 9619 of `preparation-receipt-unit-r3-20260912.out`:
`unit exit 0`.
Independent re-review found no concrete remaining defect within this declared data/reconstruction scope.
Valid-wrong-document removal witnesses,
non-prefix sparse outcomes,
changed alignment interpretation,
later mapped parents and ownership of alignment metadata still need additional controls before isolated guard proof.
`5ae0f9a65` adds those boundary controls.
R4 passes build and types,
but the later-parent fixture fails before acquisition:
its reused fixture tries default section alignment before applying the explicit registered correspondence.
`qualification-transport.test-fixture.ts` now owns the shared mock client independently of parent selection;
the receipt fixture applies the intended native alignment directly.
This is a fixture correction,
not evidence of a production receipt defect.

After the owner ran `mise upgrade` in the worktree and restarted the terminal/session,
the agent's own Bash probe resolves `/home/user/.local/share/mise/installs/node/26.8.2/bin/node`
and returns `v26.8.2`.
The R5 runner's `/proc` executable link confirms Node 26.8.2.
R5 passes build,
types,
focused tests,
formatter,
rebuild,
types,
lint and full unit tests,
then stops on Markdown semantic wrapping.
R3 remains recorded under Node 26.7.0.

Guard-proof preparation identified test-evidence gaps rather than new production defects.
The receipt ownership test compared against the same potentially mutated receipt;
it now captures an independent pre-mutation snapshot in a local fixture.
Occurrence ownership tests likewise assert the input expectation is unchanged before attempting another read.
The caption-gap parity case did not exercise deterministic media ownership:
`claimMediaAdjacentTargets` requires a complete `details` run.
Its label is corrected and an actual transcript-container control now requires the added relations and findings.
R6 passes all verification phases under Node 26.8.2.
The actual full-suite `unit exit 0` was inspected at line 9661 of `preparation-receipt-unit-r6-20260912.out`.
The verified source boundary is `eb4223366`.
`cd2ac051a` adds a direct successful-call assertion before the explicit section-correspondence replay,
so removing that forwarding produces an ordinary assertion failure instead of an uncaught domain exception.
Its manual type check passes.

The owned guard worktree is `translation-repair-receipt-guard-20260912`,
created from the main worktree at `eb4223366`.
It carries only the committed `cd2ac051a` test overlay,
with base and overlay hashes recorded in `.guard-sandbox/test-overlay.json`.
The attempted detached switch was refused because the temporary repository's CLI configuration was untrusted;
no trust override or native write bypass was used.
The existing base plus explicit committed test overlay remains the recorded experiment.

`prepare-receipt-guard-20260912.mts` creates owned dependency links and outputs.
The launcher fixes no network,
2 GiB memory,
2 CPUs and 512 PIDs,
with read-only source mounts and owned writable worktree/log paths.
Known source-root `.env.local.json` files are masked with an owned empty JSON file;
the worker verifies those masks and absence of injected API credentials before running tools or tests.
The first tools probe selected retained Node 26.7.0;
`receipt-guard-tools-r2-20260912.out` verifies the explicit Node 26.8.2 launcher,
Mise 2026.9.5,
Git 2.52.0 and ripgrep 15.2.0 inside the bounded container.
That probe covers the launcher,
not the Node processes selected inside Mise tasks.
The retained receipt failure traces report Node `26.7.0`;
the successful guard detections must be attributed to that runtime rather than `26.8.2`.
`receipt-guard-mutations-20260912.out` records the completed baseline/mutation/restoration run.
All 22 registered mutations rebuild successfully,
fail their designated ordinary `AssertionError` and leave source hashes restored.
The restored build and receipt suite pass.
Peak container memory is 723116032 bytes;
no OOM or PID-limit event occurred.
Source and target hash removals each accept a valid wrong document with the unchanged local question,
so those failures demonstrate the full-document boundary rather than a later diagnostic difference.
Removing section correspondence instead wrongly refuses a valid input;
that case demonstrates required forwarding,
not unsafe acceptance.

`receipt-guard-proof-20260912` retains 72 hash-checked records/scripts/logs,
plus the subsequent cleanup audit,
test overlay patch and removal records.
The audit found only the committed test overlay and owned generated paths,
with no root Git sentinels.
Git removed the owned worktree from the main worktree.
`removal.json` verifies its directory and registration are absent at `2026-09-12T17:42:22.550Z`.
This completes the receipt-data/current-occurrence boundary proof,
not task 41's exclusive journal,
transport/configuration provenance,
source-channel/dependency closure or later native execution gate.
This data reader does not independently establish an exclusive attempt directory,
provider transmission,
actual configuration derivation,
allowed target lineage,
source channels or dependency authority.
Those owning-journal obligations remain open.
