# Writer calibration unit scope: the current-evidence qualification checkpoint

Task 41's pairing-only milestone:
shared question construction,
the replay check that qualifies prepared block pairing,
the verification rounds,
and the guard-removal proof on a disposable worktree.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Current-evidence implementation checkpoint

Task 41 is active.
`b3ad51467` extracts `blockPairingQuestion` and routes `prepareBlockPairing` through it,
so numbering,
definition exemptions and the existing versioned cache key have one implementation.
`0af39bd18` adds exact-key,
changed-text/order,
definition and empty-side controls;
`16f9b6732` corrects their harness-required asynchronous callbacks and retains formatter output.
The first type-aware formatter invocation saw stale built exports and synchronous test callbacks,
not a pairing guard failure.
Fresh build,
types,
zero-warning lint and full-unit verification pass on `16f9b6732` source.
`pairing-question-unit-20260912.out` contains the new question controls at line 362
and the inspected `unit exit 0` at line 9310.
Scoped Markdown lint also passes.
These checks verify shared question construction,
not current-evidence qualification or writer admission.

Question construction alone does not qualify evidence.
`647f713d2` adds the separate `qualifyPreparedBlockPairing` replay check;
`14083dfd7` adds transport-backed tests.
It retallies actual queried outcomes,
requires the configured usable quorum,
recomputes existing media claims and definition separation,
and rejects historical cache or unresolved fallback as current qualification.
Source insertions and target declines retain the existing all-originals-placed condition.
Unendorsed archive gaps retained inside a split rendering also remain unqualified.

Independent review found unchecked preparation-level findings and incomplete fast-path accounting.
`790b26946` shares complete queried handoff findings with production,
rebuilds the final handoff through `finishPreparedBlockPairing`,
and records zero-question structural relations,
source insertions and archive blocks without a source.
`7998ce3db` adds complete-finding,
index,
extra-key and ownership controls.
All returned variants explicitly say `pairing-only`.
This implementation is awaiting complete verification;
it is not verified by the earlier question-construction suite.
The first build passed but TypeScript `TS2322` required an explicit positive questioned-variant branch,
fixed with an unknown-disposition control in `8af49c645`.
The next build and types passed;
formatter findings concerned the test transport's function expression/asynchronous signature
and an insufficient symbol description.
`86997ec10` uses a declared transport returning its already available reply through `Promise.resolve`
and retains the reviewed formatter output.
`qualified-pairing-verification-r3-20260912.out` records passing builds,
types,
formatter and zero-warning lint,
but the full suite failed on fixture assumptions:

- The rotated window asks four of five configured identities;
  `SyntheticModelNotServedError` loses `glm-5.3` before transport,
  leaving three heard replies but only two usable replies.
- One uncorroborated suggestion is removed before agreement findings are generated,
  so it does not make `cacheEligible` false.
  A separate crossed-definition control must exercise an actually dropped endorsed relation.
- An entirely empty document has no aligned parent.
  The empty-side helper contract needs an explicitly supplied empty side,
  not a claim that section alignment produced one.

`dcb478f1c` corrects these fixtures without changing production behavior or thresholds.
`qualified-pairing-verification-focused-20260912.out` records passing rebuild and type checks,
then the qualifier and existing parent-preparation tests through the package's file-filtered `test:unit` task.
The focused test log ends with `unit-focused exit 0` at line 229.
This includes the otherwise-qualifiable uncacheable outcome,
complete media findings,
all supplied empty-side accounting and unchanged cold/warm production preparation.
The fresh full run `qualified-pairing-verification-r4-20260912.out` passes every phase on `dcb478f1c` source:
build,
types,
formatter,
rebuild,
types,
zero-warning lint,
full unit and Markdown lint.
`qualified-pairing-unit-r4-20260912.out` ends with the inspected `unit exit 0` at line 9478.
Independent re-review found no concrete remaining pairing-only defect,
but requested one composed relabel/reprepare qualification test before guard-removal proof.
`c62c6b37c` adds the composed lifecycle test:
actual acquired definition relations drive relabeling and repreparation,
old evidence fails against changed current text,
and newly acquired evidence qualifies with current labels,
nodes,
ranges,
line flags and stage inputs.
Both direct relabel and initially unqualified crossed-definition cases pass,
alongside the actual pass/cache lifecycle test.
`qualified-pairing-unit-footnote-footnote-20260912.out` records `unit-footnote exit 0` at line 75;
its build and types also pass.
The subsequent full run `qualified-pairing-verification-r5-20260912.out` passes all phases,
including the inspected full-suite `unit exit 0` at line 9508.
`c95318ebe` additionally guards ownership of the zero-question result.
The owning journal is not complete.

Removal proof completed in the owned disposable worktree
`~/temp/agent/translation-repair-qualified-guard-20260912`,
created from the main worktree at `519e87092` to avoid copying ignored state.
Its only source overlay is the committed `c95318ebe` boundary test,
bound in `.guard-sandbox/test-overlay.json`.
The dependency preparation records thirty-two links and an absent credential file.
`worker.mts` and `mutations.mts` live under that worktree's `.guard-sandbox/`;
the launcher is `~/temp/agent/run-qualified-guard-container-20260912.mts`.
The no-network mutation run verified the 2 GiB,
2 CPU,
512 PID limits.
All twenty-two registered changes built successfully and failed their designated normal `AssertionError`.
Source hashes were restored and the rebuilt restored tests passed.
Peak memory was 667873280 bytes,
with no OOM or PID-limit events.
The target-key case changes refusal from `question` to `result`;
complete handoff validation still refuses stale evidence.
That case proves diagnostic binding,
not that stale evidence would otherwise be accepted.

Proof is retained in `~/temp/agent/qualified-guard-proof-20260912`,
with sixty-six copied records,
script files and logs,
plus the cleanup audit and test-overlay patch.
The audit rechecks source hashes,
permits only the committed test overlay and owned generated paths,
and finds no root Git sentinels.
The owned worktree was removed through `git worktree remove --force` from the main worktree.
`removal.json` verifies both filesystem and worktree-registration absence.
The retained proof and unrelated worktrees remain untouched.
Continue the owning journal's exact document,
roster,
attempt and source-channel bindings;
this pairing-only milestone does not complete task 41.
No paid preparation or writing was involved.
These harness and lint failures are not semantic guard-detection evidence.

The owning journal must bind the exact acquired roster,
full documents,
parser data,
containers and attempt identity before accepting this replay check.
Initial crossed-footnote evidence can remain recorded but unqualified while relabeling and repreparation occur;
only a final recipe must pass complete target accounting.
Source-channel manifests,
current full-document/range binding,
persisted recipes and native reviewed-input execution remain unimplemented.
