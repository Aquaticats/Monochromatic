# Writer calibration unit scope: native request materialization without serving

The capture of native provider request bodies through a mandatory stopping transport,
the independent native-client controls,
and the guard-removal proof that closed the non-serving request boundary.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Native request materialization in progress

`301aec38a` and `4dca79bbb` introduce non-serving native request capture.
`captureBlockPairingRequests` constructs the shared block protocol,
then invokes the existing provider clients individually with capture-only credentials,
a mandatory stopping transport and a Bedrock accounting adapter that refuses every operation.
It records header-free body/endpoint/label/stream projections in configured model and provider order.
It does not execute the production router or seating operation.
The emitted self-digest is not independent configuration or provenance evidence.
The conservative POST envelope is initial gather plus `STAGE_RETRY_ROUNDS`,
permitted text routes and existing HTTP retries,
without a recovery nudge.

`188e052c5` tests native bodies,
current measured caps,
source escaping,
input ownership,
timeout/roster identity and refusal of structural dispatch.
Initial build,
types and focused tests pass;
formatter findings concern statement layout and sequential awaits in tests.
Independent review identified the need to honor provider-derived exchange cancellation,
keep the internal capture stop out of public failures,
wrap invalid native JSON and narrow the request/provenance descriptions.
Those changes include injected-adapter tests for request shape,
provider-derived aborts,
no-capture/multiple-capture flows,
unexpected errors and native Bedrock accounting refusal.
R2 passes build and types,
then its array-root request fixture demonstrates a validation gap:
`json-guard.ts` deliberately admits arrays for property probing.
The capture operation now rejects arrays explicitly before inspecting object controls.
This does not alter the already verified receipt reader's separate exact-key/prototype checks.

`preparation-request-bounds.unit.test.ts` and its routing fixture now drive native providers through
`promptUniqueClient` and `createRoutingClient` into the real `prepareBlockPairing` caller.
Separate controls exercise lost-stage rounds with native HTTP retries,
and payment-refusal fallback across each configured route.
They compare observed bodies with captured projections and check the recorded envelopes,
without adding a nudge or treating routes as independent votes.
R3 rebuilds successfully,
then TypeScript reports that `ChatMessage` is not exported by the built entry point.
The routing fixture now uses the exported `ChatTextRequest['messages']` type.
R4 passes build and types.
The native transient-round/HTTP control and the other fallback cases pass,
but Qwen's final route selects Anthropic stream grammar while the fixture emits OpenAI frames.
The client correctly reports `MalformedCompletionError` with `anthropic stream ended without message_stop`;
the fixture then observes retry calls instead of its expected completed stage.
The fixture now emits the grammar declared by the actual native exchange,
and compares complete header-free projections including stream metadata.
R5 passes build,
types and all focused interception/caller controls.
Its formatter findings are fixture function placement and the private stop class's required `Error` suffix.
Those are corrected without changing native payloads or the capture contract.
The non-serving positive control now asserts successful materialization through `Promise.allSettled`,
so a removed transport binding can produce a designated assertion rather than an uncaught network/capture exception.
R6 passes all verification phases under Node 26.8.2.
The actual full-suite `unit exit 0` was inspected at line 9735 of
`preparation-request-capture-unit-r6-20260912.out`.
Independent review found no remaining capture-adapter defect,
but requested stronger verification:
compare production capture against independently constructed native clients,
exercise every provider's HTTP retry ceiling,
and separate timeout-only from roster-order-only digest changes.
`preparation-native-clients.test-fixture.ts` now constructs every provider through its corresponding
native factory,
not the production capture factory being checked.
The native lost-round/HTTP test is parameterized across Synthetic,
Hyper,
Bedrock and OpenRouter.
Timeout-only and roster-order-only digest controls are separate.
R7 passes all verification phases under Node 26.8.2.
The actual full-suite `unit exit 0` was inspected at line 9833 of
`preparation-request-capture-unit-r7-20260912.out`.
The independent native-constructor,
per-provider retry,
route-fallback and separate digest-sensitivity controls pass.
Isolated removal checks remain before closing the request-materialization proof.
The owned request-capture guard worktree is `translation-repair-request-guard-20260912`,
created from the main worktree at `b19e50a16` with no test overlay.
`prepare-request-guard-20260912.mts` reuses the bounded,
no-network launcher and masks known source-root credential files.
The worker requires Node 26.8.2,
2 GiB memory,
2 CPUs and 512 PIDs before running any check.
This did not constrain Node selected by nested Mise tasks:
the retained request failure traces report Node `26.7.0`.
The completed mutation detections are evidence on that runtime,
not proof that every command used the worker's `26.8.2`.
`request-guard-mutations-20260912.mts` registers request/ledger/transport,
input ownership,
projection,
error identity and native-bound controls.
The mandatory-transport mutation runs only inside that no-network container with dummy client keys.
All 22 registered request-capture mutations rebuild successfully and fail their designated ordinary assertion.
Restored source hashes,
build and tests pass.
Peak memory is 851001344 bytes;
no OOM or PID-limit event occurred.
Every assertion was read in `request-guard-proof-20260912/assertion-reading.jsonl`.
The mandatory-transport case is counted by its ordinary assertion,
not the subsequent no-network `getaddrinfo EAI_AGAIN` failures in other cases.
No outside transmission or paid model response occurred.

`request-guard-proof-20260912` retains 69 hash-checked records/scripts/logs,
plus assertion-reading,
audit and removal records.
The cleanup audit finds no tracked changes,
only owned generated paths and no root Git sentinels.
Git removes only that owned worktree.
`removal.json` verifies directory and registration absence at `2026-09-12T19:28:05.972Z`.
This completes the non-serving request-materialization proof;
it does not complete the exclusive acquisition journal or authorize paid preparation.

The request-capture test's fetch probe needs explicit isolation.
This worktree resolves `@monochromatic-dev/test-harness-core/ts` to its own `package/test-harness/core/src/index.ts`;
`package/test-harness/core/src/sinon.ts` uses ordinary process-global Sinon.
`mise.toml`'s `runTestFiles` launches each test file in a separate Node process.
The capture file now sets `concurrency: 1` around the process-global positive-control stub and its readers.
The newer main-worktree skill's per-attempt method isolation does not describe this older resolved implementation.
