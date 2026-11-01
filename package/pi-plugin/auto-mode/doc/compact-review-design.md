# Compact prepared-group review

## Implemented architecture

The 2026-10-17 refactor shares actual Pi preparation, execution, and finalization phases
across top-level and Codemode child groups.
`prepared-dispatch.mjs` retains the actual prepared records,
reviews the complete group, and releases those records into native execution.
The qualified SDK supplies the phase bodies through `createOwnedToolPhases`.

`integration/native-batch/prepared-group.mjs` owns membership, review disposition,
source checks, confirmation, and execution maps.
Automatic uncertainty and assessment failure use that group's scoped selector.
Established stale decisions remain withheld.
The separate manual group, original-source approval router, child dispatcher, whole-call runner API,
and suspended-before-hook protocols are removed.

The caller constructs one native session preset and awaits cleanup.
The native slot factory reserves a call before actual preparation.
Terminal retention remains independent of later parent eligibility.
Script-value use still requires current original ownership and a returned member.

## Retained boundaries

### Completed producer judgments

Pi retains historical assistant responses as conversation history.
The producer owner's weak-key response map therefore cannot rely on garbage collection to retire its values.
Each value contains an original judgment whose source-reader closure retains the original request body.
Completed group review storage can be released while this separate producer reference remains live.

An actual native-session reproduction completes twelve approved groups,
retains all 24 historical assistant responses and preserves the latest request/diagnostic.
Three unchanged runs retain 51,189,592 to 51,582,984 additional post-GC heap bytes.
A disposable causal experiment releases superseded producer values,
with observed retained heap of 17,779,424 and 17,789,088 bytes.
The final original request remains 1,632,936 bytes and every original execution completes.
That experiment establishes this reproduction's retention path;
it does not qualify its retirement timing for active/concurrent operations or prove the older worker OOM is resolved.

`test:native:producer-retirement` exercises this actual session/runtime boundary with a 32-MiB retained-heap ceiling.
Its original-request size,
historical response count,
completed executions and latest diagnostic assertions prevent a missing-work or cleared-history false pass.
The production owner keeps each consumer's current judgment and retires only completed, superseded judgments.
The handler completes its producer after group and selector disposal, including exceptional paths.
The frozen production qualification passes its fifteen build/type/consumer tasks;
its actual-session guard retains 16,688,184 additional heap bytes with every historical response still live.
New lifetime cases and built mutation controls extend this evidence separately.
The completed mutation set rebuilds removal of completion, removal of payload release,
premature active retirement, cross-consumer shared state and ambiguity checks during cleanup.
Every mutation fails its named regression; the restored native build, lifetime cases and memory guard pass.
The earlier group-only review-retirement test covers a different owner and cannot establish producer retirement.
The first maintained producer guard fails on an incorrect whole-outcome comparison:
the actual native outcome also includes its diagnostic reason.
That failure is not memory-retirement evidence.
The corrected fixture checks the required state and verdict independently and preserves the existing diagnostic.

Late response reuse invalidates authority without removing the original operation's cleanup obligation.
The committed cleanup regression fails the built pre-correction artifact with Node's `SuppressedError`:
the disposer adds a second ownership failure over the original operation failure.
Cleanup therefore checks exact original consumer ownership without authorizing a policy read.
New reservations and policy reads still reject ambiguous responses; response/stream tombstones remain.
The corrected build passes every nine lifetime cases, constructor and execution catalogs,
and the actual-session memory guard at 17,742,936 retained heap bytes.
The older MiMo OOM still requires full-history replay on the corrected artifact.

### Native fixture declarations

Typed fixture callers consume the immutable native artifact, whose runtime modules retain source-owner identity.
Copying source declarations into that artifact leaves their imports pointing at nonexistent source paths,
or makes the caller's composite TypeScript project traverse raw built implementations outside its file list.
The existing Rolldown/Oxc declaration generator now closes those types in a type-only fixture entry.
Native generation writes release-relative facades referencing that emitted type graph before activation.
No fixture runtime implementation is bundled into the policy entry.

The fixture environment forwards the qualified SDK's original `defineTool` function.
Its installed declaration explicitly supports concrete tool definitions passed through generic `customTools` arrays;
its installed implementation returns its argument unchanged.
This preserves the original Bash tool and renderer identities without weakening the consumer's compiler settings.
The complete native phase catalog and fresh emitted declaration type check remain the verification boundary.

An independent SDK consumer keeps the wrapper's preparation return as `unknown`
and retains the SDK erasure in its declared definition type before placing it in `customTools`.
A deliberately invalid registry name proves the compiler checks that fixture.
The valid fixture has no own type diagnostics and confirms the original SDK definition and renderer identities at runtime.
The separate catalog still needs that actual boundary in its declared types;
calling `defineTool` before discarding its erasure does not fix the registration contract.
No predicate can claim schema-valid input merely to satisfy that contract.
The SDK's later argument validation remains responsible for malformed model arguments.

The source compiler retains clause, heading, incorporation, authorization, admission,
and complete-operation fallback semantics.
Model replies supply estimates; selected readable instructions and observed authorization supply authority.
Question binding, source attribution, currentness, and callback reentrancy checks protect these boundaries.
Consolidation removes repeated execution ownership without discarding those checks.

[Decision review](decision-review.md), [instruction collection](instruction-collection.md),
and [operation preparation](operation-preparation.md) identify callers and alternatives.
Finite program profiles retain the real VM, recorder, queues, conversion, and issued-work drain.

## Artifact layout

Qualified upstream TypeScript stays in its verified compiler checkout.
Native output contains emitted SDK runtime, maintained contracts, generated helpers,
policy output, metadata, and licensing.
Identical files share storage across immutable releases.

The earlier proposal to prune everything outside a literal graph is not implemented.
Native verification loads contract fixtures from the generated artifact,
and generation uses imports absent from the literal source graph.
Retaining this development output preserves real consumers without another staging protocol.
The emitted graph explicitly records computed imports; an unreached file is not automatically dead.
The active artifact contains 1,030 regular files totaling 8,211,396 bytes.
Its conservatively reachable graph contains 794 files totaling 6,099,319 bytes.
Shared external dependency targets are excluded.

Maintenance removes 43 inactive build copies after live-process and repository-consumer checks,
retaining active output and verified rollback.
Builds preserve old paths for running sessions; automatic release eviction is not implemented.

## Executable evidence

All 22 real native lifecycle modes pass, including child preparation, hook blocking,
unsealed arguments, and cancellation.
The 24-case fallback matrix, native execution catalogs, full package units, and type check pass.

The final configured MiMo replay completes all 16 original calls,
with zero selectors and 17 actual live-Luna requests.
Positive-controlled payload inspection excludes raw structured commands, thinking, and signatures.
A prohibited process/Bash pair is withheld without a selector.
Removing both independent denial barriers through a process-local load hook makes the real test
execute three harmless fixture commands and fail.
Production artifact hashes remain unchanged.

The earlier comparison matches nine three-member traces against immutable baseline `native-release-1voz8z`.
The first prototype changed cancellation finalization and validation-result order;
the corrected traces exercise both differences.
Captured traces remain in private evidence after inactive build cleanup.

## Source inventory and limits

The parser inventory counts 813 tracked source files, 103,885 physical lines,
65,692 nonblank code lines, and 29,136 runtime statements.
It removes comments, interfaces, type aliases, and type-only declarations.
Remaining annotations and string data count as code lines.
Positive controls preserve a comment-looking string and detect an added statement.

The literal source graph reaches 296 files and 20,357 code lines.
Filename-classified verification accounts for 28,596 code lines in 290 files.
Build inputs and other files outside the graph are reported separately, not declared unused.
The intermediate compact-design baseline was 68,040 code lines and 30,301 statements;
the retired extension deletion predates that baseline.
There is no accepted numeric line budget.

Full maintained Oxlint covers 813 files with 486 rules:
83 errors and 13,686 warnings remain for MiMo's lint pass.
Behavioral verification does not establish lint completion.
Inventories and receipts remain private under
`${HOME}/temp/agent/pi-mimo-lint-approval-20261017`.

## Checkpoint maintenance lessons

An RPC tool-completion event does not establish an idle agent.
The supervisor monitor reports an approval dialog independently of its completed-tool threshold.
That threshold may distinguish an old idle checkpoint from new work,
but must not hide a newly blocked operation.
Private proof files are checked through the worker's actual mounted home;
a host path through an unmounted symlink is not an accessible handoff.
The user keeps root `MWK` unchanged and records these concrete monitor lessons here.
Pi accepts steering during a model request and delivers it after the current turn's tools;
follow-ups wait until the agent would otherwise stop.
The package's input records are owned instrumentation, not additional SDK contracts.
Their origin category cannot turn ordinary queued input into a manual approval request.
The category gate and its later whitelist repair are removed after actual configured-host reproduction.
Genuine changes to governing policy or executable tools still invalidate the corresponding approval inputs.
The unmodified SDK probe distinguishes queued text from the separate active-run abort operation.
An assertion that queued text saying stop guarantees immediate cancellation invents behavior Pi does not provide.
Native regression tests check both automatic execution and subsequent actual serialized delivery.

Correct use by reasonable consumers is the UX requirement.
A behavior being documented does not excuse a surprising result.
The RPC launcher must qualify its actual Git executable before allowing repository writes;
a successful generic `git --version` does not establish active policy protection.
See the [launcher PATH incident](../../../../doc/troubleshooting/git-policy-wrapper-launcher-path.md).

Capture a nonempty list of explicit file paths before staging,
including new files.
Use that same list for staging and committing,
then compare the committed paths with it.
Do not rebuild the commit path list from the unstaged diff after staging:
the list can become empty and cause Git to commit concurrent index entries.
Commit `8b34f75c03` demonstrates that failure with five unrelated troubleshooting documents.
The concurrent content is preserved and a corrective commit comment records the scope error.
The user explicitly keeps root `CPN` unchanged and places this lesson here.

A formatter's safe-fix label does not establish behavioral equivalence.
The complete source checkpoint comparison retains runtime literal values,
tagged-template raw text,
names and branch order before classifying changes.
Its 35 identical stripped ASTs and 11 differing files require different review evidence.
Real consumer checks still cover differences in imports,
return handling and error checks;
AST equality alone does not prove `Function.prototype.toString` compatibility.
The private native formatter also introduces TypeScript-only syntax into JavaScript,
documented in the
[readonly autofix investigation](../../../../doc/troubleshooting/oxlint-prefer-readonly-javascript.md).

## Typed-port semantic checks

The policy-error port initially adds a TypeScript member declaration that emits an own runtime field.
The inherited-setter consumer detects the changed property behavior;
the corrected implementation retains the original assignment.
The deciding source,
configuration probes and rebuilt mutants are retained in [the error-name contract](error-name-contract.md).

The fallback-catalog port initially validates material entries and fragment text before the original code needs them.
That extra validation changes a retryable transient into a fixture error.
The corrected port preserves the original first-entry check,
keeps uninspected tail values as `unknown` and narrows fragment text inside the original short-circuit predicate.
Original and port transient counts agree,
and the complete fallback/startup catalogs pass again on the newly compiled frozen runtime.
An assertion function's type describes exactly what its executed checks establish.

Review of the proposed policy-input port catches a separate syntax change before the write is applied:
`#state in input` becomes `'#state' in input`.
The original expression checks the class's private brand;
the proposal checks a public string property.
The supervisor stops that write and requests the original syntax and an independently compiled brand regression.
The maintained `test:native:policy-input-identity` consumer obtains a real loaded capability
from the built native environment.
It checks original binding identity,
every exported capability reader,
public-property lookalikes,
inherited copies,
proxies and borrowed getters/methods.
The original class artifact passes these cases.
Fresh types and the new consumer's scoped lint pass with zero warnings and errors.
Two separate private native builds replace the brand with the rejected string-property check
or remove the entry guard;
both fail the maintained consumer.
Restored code rebuilds and passes.
This establishes the new guard's sensitivity before the factory redesign;
the replacement still requires its own built ownership and real-host checks.
The first rebuilt factory passes the original identity cases,
but separate enumeration and serialization checks expose another observable change.
The original class passes both;
the factory publishes enumerable getters and serializes 2,425 code units of loader state
instead of the original `{}`.
The correction keeps the issued operations nonenumerable during staged construction,
before the object enters its ownership map or is frozen.
The maintained consumer checks this behavior without printing the serialized loader state.
Absence of an in-repository serialization caller does not justify surprising implicit data exposure.
Mutation testing also exposes a false positive in the borrowed `assertCurrent` case:
an invalid binding still throws when receiver authentication is removed.
Each borrowed operation must receive valid arguments,
so its observed rejection isolates receiver ownership.
The binding comes from the actual current publication rather than a fabricated object.
All readers and the reload test share one sequential suite because reload changes that publication;
the module-test harness otherwise dispatches concurrent children.
The corrected tests must reject the rebuilt authentication-removal variant and pass restored code.
The temporary loader declaration is an interop boundary;
it does not establish that the underlying JavaScript implementation is type-checked.
The root `TCV` rule remains unchanged;
these concrete porting lessons remain package-local.

## Type interfaces belong with their owners

The user keeps root `QF1` and the existing lint policy unchanged.
The package-size audit measures Unicode code points,
non-whitespace characters and executable structure.
Formatter wrapping is formatting,
not a regression or evidence of added implementation complexity.
Modules exceeding `max-lines` still require meaningful responsibility splits.

The formatter's producer owner changes from 11,829 to 11,958 characters,
with 9,432 non-whitespace characters in both versions.
The decision core changes from 12,290 to 12,307 characters,
with 10,273 non-whitespace characters in both versions.
Those changes add 146 whitespace characters in total.
The initial rejection based on changed line-count diagnostics is withdrawn.
The retained formatting has identical parser-normalized syntax trees and ordered comments;
lint completion still requires splitting over-limit modules and fixing the remaining diagnostics.

The proposed phase-catalog port exposes another design failure before acceptance.
The original consumer has 432 code lines,
126 AST statements and 25 functions.
Its ten-file replacement has 1,216 code lines,
357 statements and 71 functions.
The same positive-controlled parser excludes comments and type-only declarations from both code-line counts.
Its physical source grows from 444 lines and 12,322 bytes to 2,655 lines and 71,116 bytes.
The replacement's first complete catalog fails despite clean types and scoped lint.
That checkpoint is not accepted.

Much of the added interface reconstructs an owned session through generic unknown-field readers
and a forwarding operation for each session method.
The original session factory already has a typed implementation;
the runtime and maintained policy modules already have source-backed interop declarations.
The next consolidation puts the missing environment and assessment fixture contracts with those owners,
uses the actual built-module contracts,
and removes the duplicate consumer interface.
Untrusted external data still requires runtime narrowing.
A trusted source-backed declaration and a runtime callable check are different evidence;
neither needs to masquerade as the other.

Lint success must accompany a simpler ownership model and preserved consumer behavior.
Adding field readers around every call transfers the implementation's missing interface into each caller.
Review the resulting interface,
executable statement count and real consumer before accepting a typed port.
