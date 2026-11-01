# Archive footnote collisions in writer preparation

## Task and boundary

Task 39's implementation and verification boundary is complete at `086fd9f6e`.
Task 38's prepared writer inputs still require their separate population,
acquisition and reviewed-plan gates.
The corpus remains pinned at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379` and read-only.
No paid preparation or writer generation has run for the new plan.
No corpus passages or edits belong in this repository.

The existing policy is recorded in the translation package README's nineteenth-class account:
archive labels follow the original before lanes run,
references and definition openers move together,
and changed target text is prepared again.
The [archive-original decision][original] additionally protects sealed English-original spans byte for byte.

## Measured collision

The selected `Y1Ran` input has source labels `1` and `2`,
and archive labels `1`,
`2` and `3`.
A manually source-reviewed definition correspondence maps archive `2` to source `1`
and archive `3` to source `2`.
The additional archive definition at `1` remains unpaired.

Frozen `97599f638` returns:

```text
kind: open
the map lands on [^1] which the archive carries and the map does not move
(1 archive labels and 0 original labels unaccounted for)
```

The two-note permutation positive control closes successfully.
Evidence is `footnote-pool-closure.json` in the owned writer workspace
and `~/temp/agent/writer-pool-footnote-closure-20260911.out`.
These manually reviewed correspondences are mechanism controls,
not acquired model evidence or calibration approval.

## Direction

Use deterministic displacement of colliding unmapped archive labels into unused labels.
This is an operational rename,
not a new source correspondence.
For the measured input,
the intended composition is:

- Correspondence `2` to `1`.
- Correspondence `3` to `2`.
- Collision avoidance `1` to fresh label `4`.

Fresh labels must be absent from both normalized identifier universes.
Preserve every reference,
definition and note body.
No archive prose repair,
extra writer round or per-entry exception is authorized by this operation.

Displacement is allowed only after all source labels have correspondence evidence
or the existing forced elimination.
If source correspondence is incomplete,
keep a colliding map open rather than closing it with displacement.
Existing collision-free partial rewrites remain operationally valid without claiming complete correspondence.
Identity correspondences must not be guessed from equal label spelling:
the old `mapLabels` filtered identity moves out.
Complete correspondence evidence now remains distinguishable from the rewrite map.

Independent review supports this ownership split:
closure owns the validated composed map and typed provenance;
the application/pass boundary owns syntactic rewriting,
definition order,
graph preservation and protected-byte checks.

## Syntax safety findings

A provider-free probe on frozen `895843508` found additional deciding behavior:

- `applyFootnoteRelabel` scans the complete raw document.
  It rewrites matching strings inside front matter,
  inline code,
  fenced code,
  escaped literals,
  HTML comments,
  ordinary link destinations and JSX attributes.
- Case-equivalent references `Note` and `note` remain unchanged when a map only names definition spelling `NOTE`.
- An escaped closing bracket in a valid footnote identifier is truncated by the raw scanner.
- The document graph counts an escaped literal opening as a reference.
- Space-containing examples in this probe are ordinary text,
  not GFM footnote nodes.
- The parsed tree distinguishes normalized identifiers,
  decoded labels and exact source offsets.
  Rewriting must not derive a raw span length from a normalized or decoded label.

Evidence:
`footnote-syntax-probe.json` in the owned workspace and
`~/temp/agent/writer-footnote-syntax-probe-20260911.out`.
The fixtures are invented.
These observations are about this package's scanner/application composition,
not an upstream parser defect.

## Required implementation proof

- Keep supplied correspondence,
  elimination-derived correspondence,
  collision-avoidance moves and the composed rewrite map separate.
- Validate normalized membership,
  injectivity and final closure.
- Choose unused positive decimal labels deterministically without numeric-overflow assumptions.
- Rewrite simultaneously so swaps and displacement cannot cascade.
- Use syntax-positioned markers,
  not a whole-document substring replacement.
- Keep escaped literals,
  code,
  comments,
  front matter,
  link destinations and JSX attribute strings untouched.
- Retain unmatched definitions once in their existing relative order after source-ordered definitions.
- Compare reference/definition structure under the rename;
  never merge labels or introduce lost/unresolved links.
- If any relabel or reorder would change a protected English-original span,
  withhold the whole change and return the original archive with an explicit finding.
- Reparse after changes before using offsets,
  containers,
  hashes,
  pairings or a reviewed-plan digest.
- Verify the actual pinned `Y1Ran` consumer composition,
  existing permutations and no-op behavior,
  multiple colliders,
  incomplete source evidence,
  normalization-equivalent and escaped identifiers,
  invalid destinations,
  protected references/definitions,
  and guard removal.

## Implementation and intermediate verification

The mathematical closure layer is implemented through `81da9c055`.
It validates normalized membership and injectivity,
keeps supplied relations,
forced non-identity elimination and retained-label displacement separate,
and refuses displacement while any original remains unaccounted for.
The first eight retention regressions failed on the old implementation and pass on the new core.
Existing no-op closures do not invent identity correspondence.
Through `b3b6ee29c`,
the readers retain complete correspondence evidence,
and active marker inventories use strict MDX grammar with offset-preserving comment and invisible-line masks.
The rewrite validates destination grammar and whole-namespace injectivity,
copies non-marker bytes unchanged,
and checks the reparsed marker-role and identifier sequence.
The original-English guard anchors both sealed text and its declaring comment by position.
A rename or reorder touching either withholds the entire operation.
After an allowed rename,
protected offsets are derived again from the changed text.

Definition movement now preserves each distinct separator once,
compares normalized label ranks,
withholds changes across non-blank gaps or shared container delimiters,
and checks actual reparsed definition-block text against every copied block.
The public pass helper records supplied correspondence,
forced elimination and archive-only retention separately,
and records structured refusal instead of reporting withheld operations as applied.

The additional movement regressions exposed the old first-gap duplication,
comment relocation,
case-sensitive ordering and split-container behavior.
The malformed-JSX control also showed that `parseSliceBody`'s lone-tag mask was unsuitable for whole-document edits.
The marker inventory now requires document grammar;
the attempted `parseSliceBody.parsedText` API extension was removed.
Strictly parsed structural-fragment compatibility remains part of the full-suite and consumer review.

Build and type checks pass through `b3b6ee29c`.
Scoped retention,
correspondence,
syntax,
protection,
definition-order and existing relabel tests pass.
Logs are `footnote-{correspondence,syntax,protection}-{build,types,green}-20260911.out`
in `~/temp/agent`.
The source tree recorded as `75b769026` also passed the full unit process,
with `footnote-full-unit-20260911.out` ending at line 9253.
That direct invocation did not append a separate `unit exit 0` marker;
final wrapper-based verification remains required.
The second formatter run reported zero warnings and errors before the latest boundary test additions.
Final lint and individual guard-removal proof were still pending at `75b769026`.
The first implementation-review Advisor call timed out.
A later review identified prepared-fragment parsing,
stale rewrite domains and real pass/cache integration as remaining boundaries.
These findings supersede any inference that the initial guard catalog covered the complete operation.
The `75b769026` checkpoint did not complete task 39 or approve calibration.

The source trace currently covers:

- `package/module/translation-repair/src/archive-footnote-closure.ts` (moved on 2026-10-18 to
  `package/translation-repair/archive/src/archive-footnote-closure.ts`).
- `package/module/translation-repair/src/archive-footnote-relabel.ts` (moved on 2026-10-18 to
  `package/translation-repair/corpus-source/src/archive-footnote-relabel.ts`).
- `package/module/translation-repair/src/archive-footnote-order.ts` (moved on 2026-10-18 to
  `package/translation-repair/archive/src/archive-footnote-order.ts`).
- `package/module/translation-repair/src/corpus-run/pass-footnote-relabel.ts` (moved on 2026-10-18 to
  `package/translation-repair/corpus-pass/src/corpus-run/pass-footnote-relabel.ts`).
- `package/module/translation-repair/src/footnote-graph.ts` (moved on 2026-10-18 to
  `package/translation-repair/document-structure/src/footnote-graph.ts`).
- `package/module/translation-repair/src/footnote-identifier.ts` (moved on 2026-10-18 to
  `package/translation-repair/document-structure/src/footnote-identifier.ts`).

The actual pinned `Y1Ran` consumer control passed on frozen `.frozen-dist-b3b6ee29c`,
digest `sha256-tree-v1:7612f144d20cc33ccd9a0263801a50725401bca1ebb2e493785d9edc52cfa96f`.
It retained all three references and definitions,
with final definition labels `1`,
`2` and `4`,
and no footnote findings before or after.
Front matter remained identical;
all 45 target nodes appeared once across 23 fresh prepared slices with current ranges.
It exercised `translateSliceInput`,
compiled the masked MDX body without execution,
and proved a repeat operation is a no-op.
The configured no-network container used 2 GiB RAM,
2 CPUs and 512 PIDs.
The driver recorded zero fetch calls and 194 ms elapsed.
Evidence is `footnote-consumer-control.json` in the owned workspace
and `~/temp/agent/footnote-Y1-consumer-r3-20260911.out`.
Earlier harness attempts had a compiler import-resolution error and an incorrect assumption about
`PreparedDocumentPair` fields;
`targetText` and parsed nodes replaced the nonexistent `target` property.
No corpus bytes were modified.

At `2ce03f9a1`,
individual removal of 22 registered guards rebuilt successfully and failed at the intended normal assertion.
Restored source rebuilt and all registered tests passed.
The measured memory peak was 726011904 bytes,
with no OOM or PID-limit events.
The complete intermediate proof is retained in `~/temp/agent/footnote-guard-proof-20260911`.
The intermediate proof used owned worktree `~/temp/agent/translation-repair-footnote-guard-20260911`,
created from main at `f78921f15` with the single committed `2ce03f9a1` test overlay.
The updated proof and removal are recorded in `Verified boundary on 2026-09-12`.
No source from the development worktree was mutated by the experiment.

The independent review correctly identified another real input boundary:
a default-budget prepared container can yield a marker-bearing fragment with an unmatched container half.
The actual forty-paragraph fixture produced a marker slice at offsets 0 to 450 and the old reader threw `syntax`.
The hypothesis that the lexical length bound diverged on astral characters was not supported.
Actual parser controls accept 999 UTF-16 units,
including mixed astral/ASCII and escaped-bracket cases,
and reject 1000;
NFC and NFD identifiers remain distinct.
Evidence is `footnote-review-probe-r2-20260911.out` and the committed Unicode tests.

Red tests in `e305de648` demonstrated the prepared-fragment failure,
stale source/target slice inputs and missing map domains.
Through `9196cb780`,
changing map keys must occur in the current active namespace,
and `footnoteRelabelOf` takes complete `sourceText`,
`targetText` and slices.
It projects whole-document reference markers into checked current ranges instead of reparsing fragments.
`sliceFootnoteLabels` checks integer bounds,
exact text and whole-marker containment;
it uses a binary search followed by the intersecting marker run.
The pass caller and existing tests now pass complete document context.
Build,
types and the review regressions pass in `footnote-review-{build,types,green}-20260911.out`.

The actual `preparePassEntry` lifecycle tests now cover complete relabeling,
protected withholding and crossed-definition elimination with disposable on-disk caches.
The three cases passed in `footnote-lifecycle-unit-r3-20260912.out`.
They verify changed target text,
node text/hash/range metadata,
line flags,
final definition relations,
distinct persisted question keys and cold/warm transport counts.
The fully claimed fixture reaches no archive prose repair,
and the title-free source returns before lookup I/O.

The first lifecycle harness attempted to import the private `openPairingCache` helper;
it now reads actual cache envelopes and generation files directly.
A shared global-fetch stub collided across concurrent attempts;
it was removed in favor of the injected model transport plus the actual empty-title early-return check.
No harness or pairing policy was changed.

The crossed fixture exposed an important existing distinction:
`readBlockPairing` accepts definition-order exemptions,
but `agreePairs` still applies its existing monotone agreement.
The initial crossing slate retains two of three relations,
uses the existing forced elimination for the remaining label,
and is not cache-eligible.
Replay therefore reacquires its initial two voices but reuses the relabelled question.
The fully claimed monotone and protected cases reuse their complete cached questions without new calls.
The test now checks those actual branches rather than asserting all crossings are cacheable.

The companion compound suite passed bare-CR,
multiline-body,
separate-run and multiple-surplus operation cases.
Its initial indented-code expectation was wrong for MDX:
`micromark-extension-mdx-md@2.0.0` disables CommonMark indented code.
The fixture now contrasts MDX paragraph parsing with plain-Markdown code parsing.
It still needs its corrected verification run.

## Verified boundary on 2026-09-12

At `086fd9f6e`,
with source code recorded by `2cc8cfec1`,
build,
types,
zero-warning lint and the complete unit suite pass.
The actual final unit log ends with `unit exit 0` at line 9308 of
`~/temp/agent/footnote-final-unit-20260912.out`.
The corrected MDX/Markdown indentation contrast and all real pass/cache lifecycle cases pass in that run.

The current frozen `Y1Ran` rerun passes on `.frozen-dist-086fd9f6e`,
not merely the intermediate build.
It reproduces the expected three-note graph,
front matter,
45-node coverage,
23 current prepared slices,
shared projection,
MDX compilation and repeat no-op.
The output hash remains `4ad32024f12499157b11fcc06e099020deb34c8f1d0c87ecb16157e679942664`.
It records zero fetch calls and 307 ms elapsed,
without treating that duration as a performance comparison.
Evidence is `footnote-Y1-current-consumer-20260912.out` and `footnote-consumer-current.json`.

The updated proof detects 30 registered guard removals and binding mutations.
Every mutant rebuild succeeds,
every designated test fails with its intended normal `AssertionError`,
all source hashes are restored,
and restored build/tests pass.
The configured limits remain 2 GiB RAM,
2 CPUs,
512 PIDs and no network.
Measured peak memory is 775876608 bytes;
OOM and PID-limit counters remain zero.
Proof is retained at `~/temp/agent/footnote-guard-proof-r2-20260912`.
The original proof remains separately retained.

Independent re-review found no remaining concrete footnote correctness blocker.
Its remaining evidence gates were fulfilled by the current consumer run,
the updated mutation proof and normal owned-worktree cleanup.
The guard worktree was removed through `git worktree remove --force` from main
only after source-restoration and ignored-root audits.
No root sentinel artifacts existed;
the ignore rules themselves still matched their names.
Filesystem absence and absence from `git worktree list` are recorded in the retained proof's `removal.json`.

The next action is task 38's source-authority-qualified preparation population and review gate.
Neither a footnote mechanism control nor warm production cache reuse is current model correspondence evidence.
No paid preparation,
writer generation,
image calibration or newer full-entry pass occurred.
The first formatter run (`proc_d6ff`) completed with `no-nullish-union`
on the lexical no-marker return and `max-statements-per-line` on a lookup callback.
The scanner now uses a domain-specific `NO_GFM_MARKER` sentinel,
and the callback is split onto statement lines.
The package diff contains only task-owned footnote paths;
unrelated `mise.lock` drift remains untouched.
The second formatter run passed.
The final zero-warning lint result is recorded in `Verified boundary on 2026-09-12`.
Only mocked or provider-free work has run.
The latest fully checked translation runtime is `.frozen-dist-086fd9f6e`,
digest `sha256-tree-v1:6ec56fc725bc08c481d040f16a9d358d4826384c1a820879e2c73611472b83be`.

[original]: ../decision/translation-repair-archive-original.md
