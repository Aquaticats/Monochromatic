# Scoped archive evidence

Task 26 designs the generic implementation after the successful
[name-repair prototype](translation-repair-name-authority-2026-09-10.md).
Tasks 27,
29 and 28 cover qualified revision acquisition,
initial-use corroboration and repair-lifecycle integration.
Task 21 remains incomplete until that integration is verified.
No production revision feature has been added.

## Verified starting point

The `b9d3b2ea0` evidence-injection prototype completes the real repair path
with correct disclosure direction,
administrator wording and `Harunome Hanbai` retained.
All three editors retain the group name.
The selected GLM-flash chunk wins six votes,
not a fallback.
Regional ties leave a weaker composite,
but whole-chunk selection chooses the complete editor draft.

`~/temp/agent/temporal-revision-path-replay-20260910.json`
replays all 58 requests with zero cache misses and identical final text.
`~/temp/agent/temporal-revision-boundary-20260910.out`
records valid strict grammar and paragraph/list structure.
Some selector reasons incorrectly reject source-supported date or name expansions;
those reasons are not credited as factual evidence.
The selected summary itself does not carry the wrong childhood timing.

This verifies an evidence treatment,
not generic acquisition or a complete published page.
The prototype still selects one known historical revision manually.

## Required interface properties

- Read from the pinned pre-repair archive,
  not generated candidates or the mutable working tree.
- Derive revision provenance rather than maintaining a name list.
- Keep exact before/after fragments,
  origin revision/path and current occurrence scope.
- Never reuse an inferred correspondence across entries.
- Distinguish a recorded wording change from factual truth,
  an official English name or an official work title.
- Do not label arbitrary revisions as naming choices.
- Keep source facts,
  explicit naming declarations,
  work-title precedence and mentioned-form preservation intact.
- Missing or incomplete history must be represented honestly,
  not manufactured as an empty authoritative record.
- Cache and twin identities must include the normalized evidence actually supplied.
- Stored artifacts must retain enough provenance to audit and rebuild that input.
- No additional model-generation round is introduced.

## Existing module facilities

`package/module/translation-repair/src/corpus-source.ts` (moved on 2026-10-18 to
`package/translation-repair/corpus-source/src/corpus-source.ts`)
already reads blobs at `CorpusPin.commitSha`,
rather than reading the working tree.
It folds CRLF to LF through `foldCarriageReturns`.
Lone carriage returns are not folded.
`src/corpus-run/git-command.ts` supplies the existing `resolveGit` helper,
which prefers the real binary over the repository's mutation-policy shim.
Reuse these choices rather than creating a second Git-resolution convention.

The proposed seam is a module that accepts the pin and archive path
and returns scoped revision records plus honest coverage/availability findings.
It owns Git reading and parsing.
Consumers should not need to reconstruct blame origins or join diff hunks.
Tests should cross that interface using disposable Git repositories.
The exact record and failure contracts remain to be finalized from the probes.

## Git source and initial acquisition probe

The installed command reports Git 2.55.0.
Read-only source reference:
`~/temp/agent/git-reference-20260910`,
tag `v2.55.0`,
commit `e9019fcafe0040228b8631c30f97ae1adb61bcdc`.
The inspected paths are `Documentation/git-blame.adoc`,
`Documentation/blame-options.adoc`,
`Documentation/diff-options.adoc` and `builtin/blame.c`.
No third-party source is edited.

`~/temp/agent/prototype-archive-revision-acquisition-20260910.mjs`
uses pinned line origins and ordinary parent-relative diffs.
It does not name Mio's origin revision in its acquisition algorithm.
The first read-only invocation on `people/Mio/page.en.md` produces:

- 234 current lines and nine origin revisions considered.
- Eleven Git calls and 131374 bytes of captured output.
- Forty-two single-line replacement records.
- The desired `春の芽工作室` to `Harunome Hanbai` replacement,
  with the correct `33a9d3d9...` origin.
- Other records consisting of punctuation,
  reordered wording and partial-word changes such as `dmit` to `ccep`.

Results:
`~/temp/agent/archive-revision-acquisition-20260910.json`.
The result demonstrates why indiscriminately labeling all records as names would be wrong.
The recorded elapsed time is one invocation,
not a performance comparison or a corpus-wide estimate.

Current prototype limits are deliberate investigation scope,
not the final implementation contract:

- Only unambiguous single-line replacement hunks are paired.
- Boundary,
  ignored and foreign-path origins are not used.
- Initial insertions and deletion-only changes provide no pair.
- A minimal contiguous changed span is derived with surrogate-pair boundaries preserved.
  This is not a general semantic diff.
- Current-range checks reject a normalization mismatch.
- Author names and email addresses are not part of emitted evidence.

Disposable-fixture verification is still required for roots,
merges,
renames,
shallow history,
multiple edits,
reversions,
path quoting and line-ending behavior.
The collector must not infer naming semantics from a hunk's shape.
The verified Git mechanics and limitations are recorded in
[Git blame archive revisions](../troubleshooting/git-blame-archive-revisions.md).

## Generic evidence wording

The successful probe's sentence calls the edit a local naming choice.
That statement is accurate for the inspected Mio revision,
but cannot be copied blindly over arbitrary acquired records.

`proc_d506` tested a generic conditional explanation.
Its factual-error,
archive-only work-title,
mentioned-form and explicit-English-name controls remained actionable.
The actual reference-name allegation nevertheless became accepted,
with three supports,
two oppositions and one malformed abstention.
The probe stopped before selector comparisons.
No corresponding formatter is integrated.

## Completed generic-wording probe

`proc_b1a6` stopped after eighteen calls in 54 seconds.
It stays closer to the successful fragment description,
but calls the revision a local English wording choice,
not a naming choice,
and excludes independent factual authority.

- Script:
  `~/temp/agent/probe-generic-revision-context-r2-20260910.mjs`.
- Plan:
  `~/temp/agent/generic-revision-context-plan-20260910-r2.out`.
- Log:
  `~/temp/agent/generic-revision-context-probe-20260910-r2.log`.
- Report:
  `~/temp/agent/generic-revision-context-probe-20260910-r2/report.json`.

The same preplanned safety controls run before the actual name allegation
and the previously constructed name-only selector comparison in both orders.
System rules,
schemas,
roles and tally thresholds remain unchanged.
The cap is sixty requests and a twenty-minute global bound.
No production code or corpus content changes.

The factual-error and archive-only work-title corrections remain authorized.
The genuine mentioned-form correction ties three-to-three and is not authorized,
so the probe stops without relaxing that positive control.
The generic description is not integrated.
The first generic probe logged 0.00292189 USD on Bedrock,
with four OpenRouter calls reporting zero and eleven Hyper plus seven Synthetic calls unpriced.
The second logged 0.00157165 USD on Bedrock,
with two OpenRouter calls reporting zero and eight Hyper plus three Synthetic calls unpriced.
The daily helper ran after both.

## Reconsider the evidence source

An independent advisor identified the missing semantic variable:
the successful history payload explicitly described a naming choice,
while arbitrary automatically acquired edits have no such classification.
Before building history machinery,
test whether typed current-archive use evidence can supply that fact directly.

### Typed current-archive occurrence use

- Pros:
  directly represents whether a span is a person,
  group or place reference,
  a creative-work title,
  a mentioned form or ordinary prose.
  It does not require Git history or a source-to-English name mapping.
- Cons:
  requires corroborated semantic classification,
  exact immutable anchors,
  and verification that an added response field does not degrade the existing alignment task.

### Narrow historical evidence

- Pros:
  the inspected naming revision has already improved actual repair and selection.
  The acquisition prototype finds its origin without a name-specific lookup.
- Cons:
  history availability and provenance need handling,
  ordinary edits are not naming declarations,
  and the generic descriptions tested so far do not preserve all required behavior.

### Generic history prose

- Pros:
  uniform representation of raw before/after facts.
- Cons:
  the current probes either lose name preservation or suppress a genuine mentioned-form correction.

Ranking:
typed current-archive evidence,
then narrowly scoped historical evidence,
then generic history prose.
Typed use states the policy-relevant fact directly without the historical-availability problem.
Narrow history has a measured successful case,
whereas generic prose has not retained the needed behavior.
This is the investigation ranking,
not an implementation decision yet.

## Candidate typed-record contract

A record observes the use of one exact span in the initial English archive.
It is not an official-name declaration,
a source-name alias,
a canonical replacement,
a global glossary entry or a finding that the surrounding facts are correct.
No generated candidate may establish the record.

The proposed closed use kinds are person-reference name,
group-reference name,
place-reference name,
creative-work title,
mentioned form,
ordinary prose and unresolved.
Unresolved or disputed classifications yield no naming evidence;
omission is not converted into an ordinary-prose judgment.
Only the exact occurrence is in scope.

The existing block-pairing call is a candidate seam.
It already reads both block lists before downstream criticism.
An auxiliary classification field could share that call without adding a production generation round.
Independent respondents must classify the same exact candidate;
no respondent sees another respondent's classification.
Primary pairings must remain usable when auxiliary classification is malformed.

## Completed classification prototype

`proc_a1a0` completed `translation-repair-archive-use-classification-probe-20260910`
in 115 seconds with fourteen requests.

- Script:
  `~/temp/agent/probe-archive-use-classification-20260910.mjs`.
- Plan:
  `~/temp/agent/archive-use-classification-plan-20260910.out`.
- Log:
  `~/temp/agent/archive-use-classification-probe-20260910.log`.
- Report:
  `~/temp/agent/archive-use-classification-probe-20260910/report.json`.

Candidate spans are discovered from parsed emphasis nodes,
not from the literal group name.
This is a bounded discovery prototype,
not a claim to find every plain-text name.
The real Mio slice is accompanied by invented person,
group,
place,
work-title,
mentioned-form and ordinary-prose cases,
plus an unrelated target block that should not be paired.
Expected kinds are not sent to models.

Baseline and augmented calls use the same full preparation roster
through the existing public block-pairing stage.
The augmented schema adds a use kind for each supplied candidate index.
The original pairing validator still decides pairing usability;
missing or malformed auxiliary fields are recorded separately.
Two independent exact-kind agreements are required,
and conflicting corroborated kinds yield no annotation.

The cap is 48 requests and a twenty-minute global bound.
No production schema or prompt is changed.
Both baseline and augmented stages hear seven usable voices,
return the same nine block pairs,
leave the unrelated target unpaired,
and report no pairing findings.
All eight candidate uses receive the expected corroborated kind.
Mio's group reference,
ordinary `owned` and the wrong-person reference each have six matching classifications and one dissent;
work titles,
mentioned form,
explicit group name and place have seven matching classifications.
No auxiliary wire findings are reported.

The initial scratch harness supplied `fanOut: 'all'`,
which is not a valid `FanOutMode` spelling.
`stage-fanout-window.ts` accepts `window` or `whole-bench`;
the unsupported value followed the window branch.
This was not an eleven-voice exhaustive run.
`~/temp/agent/verify-archive-use-window-20260910.mjs`
replays the same fourteen calls with valid `fanOut: 'window'`,
zero misses,
identical pairings and identical corroborated classifications.
The corrected replay is the configuration verification.

Next compare typed current-use evidence with neutral history evidence
at the same claim and selector seams,
including all genuine-correction controls.
Neither a runtime Git dependency nor a typed-use implementation is selected yet.
The classification probe logged 0.0001324 USD on Bedrock
and 0.00048586 USD on OpenRouter,
with four Hyper and six Synthetic calls unpriced.
The daily helper ran afterward.

## Completed old-packet evidence comparison

`proc_b67a` stopped after eighteen requests in 102 seconds.

- Script:
  `~/temp/agent/probe-typed-archive-evidence-20260910.mjs`.
- Plan:
  `~/temp/agent/typed-archive-evidence-plan-20260910.out`.
- Log:
  `~/temp/agent/typed-archive-evidence-probe-20260910.log`.
- Report:
  `~/temp/agent/typed-archive-evidence-probe-20260910/report.json`.

The real record is selected by corroborated group-reference kind within the original archive range,
not by matching a literal name.
Contested claims join by target-span overlap.
The neutral historical record joins to the same range through the automatic acquisition prototype.
It is not described as a naming choice in the history arm.

The typed arm supplies only the observed occurrence use and its exact archive text.
It states no source-name equivalence,
official name,
canonical replacement or correctness of entity participation.
Classifier provenance stays in the report,
not as reputation evidence on the judge's sheet.

The actual name packet is compared under typed current-use evidence and neutral history evidence.
It retains its compound administrator/name allegation.
Typed-use controls cover explicit English naming,
ordinary factual role errors,
work-title translation,
mentioned forms,
wrong named participants,
place-reference conventions and official-title retention.
The pure administrator issue also remains in the control set.

If admission remains valid,
the existing name-only role-repair discriminator is selected under both evidence arms
and both candidate orders.
It retains the real/composite contributor identities and adverse fallback controls.
The cap is 108 requests,
360000 ms per exchange and 1200000 ms globally.
System policies,
schemas,
configured rosters and tally thresholds stay unchanged for these downstream calls.
No production feature is added.

Both typed-use and neutral-history evidence still authorize false renames in the old five-claim packet.
The explicit-English-name control remains authorized.
The run stops before later controls and selectors.
This comparison is not matched to the successful isolated-history benchmark,
so it does not yet decide between the evidence designs.
Bedrock logged 0.00236314 USD;
two OpenRouter calls reported zero,
with nine Hyper and two Synthetic calls unpriced.
The daily helper ran afterward.

## Matched isolated-claim comparison

An independent advisor required the exact successful history benchmark
before interpreting another typed-use result.
The same pure claim,
source,
archive,
full-source context,
schema,
numbering and actual model identities are used for every arm.
Evidence remains in the same message position,
with outer labels identifying the data kind truthfully.

`proc_deb7` first reproduced the semantic result from six cached calls with zero misses,
but stopped because its assertion compared the provider's optional `total_tokens` field.
`prompt-payload-store.ts` reconstructs `prompt_tokens` and `completion_tokens`,
not `total_tokens`.
The corrected assertion still requires identical kind,
raw model text,
parsed value and both stored token counts.
This is cache-metadata normalization,
not changed model evidence.

`proc_15ab` completed the corrected comparison in 67 seconds with eighteen requests,
including the six cached benchmark calls.

- Exact history benchmark:
  rejected four-to-two,
  matching the stored result.
- Generic typed-use evidence:
  rejected four-to-two.
- Kind-specific explanatory prose:
  accepted with three supports,
  two oppositions and one malformed abstention.

Only the generic typed-use description proceeds.
The more interpretive description is not integrated.
Results:
`~/temp/agent/matched-archive-evidence-probe-20260910-r2/report.json`.
Bedrock logged 0.00172278 USD;
two OpenRouter calls reported zero,
with four Hyper and two Synthetic calls unpriced.
The daily helper ran afterward.

## Preparation seam and corroboration check

`~/temp/agent/verify-archive-use-preparation-seam-20260910.mjs`
reads the pinned blobs,
applies the actual `passArchiveText` normalization
and runs the actual section aligner.
The group reference is in section zero,
with six source blocks and seven target blocks.
`prepare-with-pairing.ts` therefore reaches its existing block-pairing call for this section.
No extra preparation model round is needed for this measured Mio occurrence.
One-to-one or empty-side fast paths remain a separate coverage consideration.

`~/temp/agent/archive-use-quorum-check-20260910.out`
also checks the classified records against the configured preparation quorum:
six of eleven.
Every expected kind already has six or seven supporters,
so these observed records meet that stronger basis as well as the prototype's two-agreement floor.
No additional model call is made for this check.
The final metadata quorum contract still belongs to the design decision;
the existing pair-agreement threshold is not silently changed.

## Completed typed-use follow-up

`proc_9854` completed `translation-repair-typed-archive-evidence-followup-20260910`
in 466 seconds with 85 requests.

- Script:
  `~/temp/agent/probe-typed-archive-evidence-r2-20260910.mjs`.
- Plan:
  `~/temp/agent/typed-archive-evidence-plan-20260910-r2.out`.
- Log:
  `~/temp/agent/typed-archive-evidence-probe-20260910-r2.log`.
- Report:
  `~/temp/agent/typed-archive-evidence-probe-20260910-r2/report.json`.

The pure and compound claims are now tested separately,
matching the successful benchmark's scope.
The explicit-English,
wrong-entity,
mentioned-form,
work-title,
ordinary-term,
place-reference and administrator controls remain.
Fresh critics and their name-touching panel follow,
then the typed-only selector discriminator in both orders.
The cap remains 108 requests and a twenty-minute global bound.
No production implementation is selected yet.

The pure name and compound claims are each rejected four-to-two.
The genuine English-name,
role,
work-title,
mentioned-form and wrong-person corrections remain authorized.
Place-name retention and official-title retention controls are rejected as false complaints.
The pure administrator issue receives six supports.
Malformed votes remain abstentions;
one contains a long repeated-character sequence and is not counted as a valid support.

The typed-only selector chooses the preserving repair at weight five,
and at weight six with candidate order reversed.
Neither result is fallback retention.
Fresh criticism leaves no actionable rename claim and retains an accepted administrator diagnosis.

However,
no participant-correction claim survives in this fresh critic packet.
The raw replies confirm that the discarded long target quotes concern other issues,
not an otherwise-valid participant correction:
`~/temp/agent/typed-archive-critic-raw-20260910-r2.out`.
One critic is lost and the stage hears five of its initial six asks.
Do not attribute the nomination gap to the typed evidence alone;
participation and generation differ from the successful history run.
The complete repair must still satisfy the earlier temporal goal.

Bedrock logged 0.00660591 USD;
six OpenRouter calls reported zero,
with twenty-eight Hyper and twenty-one Synthetic calls unpriced.
The daily helper ran afterward.

## Completed full typed-evidence repair

`proc_0e80` completed `translation-repair-temporal-typed-path-probe-20260910`
in 542 seconds on frozen `.frozen-dist-b9d3b2ea0`.

- Script:
  `~/temp/agent/probe-temporal-typed-path-20260910.mjs`.
- Plan:
  `~/temp/agent/temporal-typed-path-plan-20260910.out`.
- Log:
  `~/temp/agent/temporal-typed-path-probe-20260910.log`.
- Report:
  `~/temp/agent/temporal-typed-path-probe-20260910/report.json`.

This executes actual `repairChunk`
with generic corroborated occurrence-use evidence on existing document sheets,
not Git-history evidence.
Policies,
schemas,
roles,
source,
archive,
thresholds and production generation depth remain unchanged.
The bound is one slice,
120 requests and a twenty-minute global limit.
No production feature is added.
Inspect name,
administrator and disclosure wording together,
including the actual candidate set and fallback decisions.

The run makes seventy requests and compiles,
but fails the combined goal:
it keeps `Harunome Hanbai` and repairs the administrator wording,
while retaining `She came out to her best friend in primary school.`
The addition complaints are rejected;
no participant-correction claim exists to create an editable region for that sentence.
`~/temp/agent/temporal-typed-path-results-20260910.out`
and `~/temp/agent/temporal-typed-actor-selection-20260910.out`
record the actual result,
claims and candidate set.

This does not establish typed evidence as the cause.
The critic cohorts and recovery differed,
and all evidence-injection prototypes applied their context after the existing seat-window calculation.
A future compiled prompt can select different seats.
Nevertheless,
this result does not validate typed-only evidence for the complete goal.

Bedrock logged 0.00533967 USD;
four OpenRouter calls reported zero,
with nineteen Hyper and twenty-two Synthetic calls unpriced.
The daily helper ran afterward.

## Selected implementation

The [qualified archive naming revision decision][qualified-decision]
selects the conjunction of corroborated initial reference use
and exact pinned revision provenance.
An independent advisor reviewed this choice after the typed full-path result.
The selected design has the stronger positive end-to-end evidence;
causal attribution of the typed nomination gap is not required to prefer it.

The automatic collector and corroborated use record already join on the same exact current occurrence.
That join reconstructs the successful naming-specific history message byte-for-byte,
without selecting a name or origin revision by literal lookup.
The classification supplies the fact that this occurrence is a reference name;
the revision supplies the recorded change.
Neither alone is promoted into stronger authority.

Task 27 implements structured qualification and pinned acquisition.
Task 29 implements the auxiliary initial-use observations in existing pairing calls.
Task 28 wires only the selected closed evidence type into final prompt builders before seat calculation,
then verifies real acquisition and invoked-model windows through the compiled repair path.
No further wording search or typed-versus-history comparison precedes implementation.
The fresh full Mio pass remains blocked on that implementation and verification.

## Acquisition implementation checkpoint

Task 27 is in progress,
not complete.
The new public API is `readQualifiedArchiveNamingRevisions`.
It currently joins caller-supplied initial-use observations to pinned history;
task 29 has not yet acquired those observations through production pairing calls.
No renderer,
repair handoff,
cache or artifact integration exists yet.

The initial acquisition fixtures pass after build and type checking:
reference kinds,
non-reference exclusions,
six-of-eleven support rather than two-reader pairing agreement,
duplicate readers,
dissent,
conflicting corroborated kinds,
whole-span equality,
role/partial/multi-line exclusions and surrogate boundaries.
The initial new-API red was only a missing export,
not proof of the individual guards.

Independent implementation review found object-context and physical EOF gaps.
`archive-provenance-red-r2-20260910.out` demonstrates replacement-object,
EOF-addition,
EOF-removal and grafted-ancestry failures.
The graft test includes a working positive control using ordinary native blame.
The first attempt used unsupported `git commit-tree --message` and is not credited;
the corrected fixture uses documented `-m`.

`781291222` supplies the shared intrinsic corpus Git context:
no replacement refs,
no lazy fetching,
removed inherited repository routing,
and a platform-null graft file assigned after inherited environment.
Parents come from the raw commit object.
`c34ab893e` preserves physical line termination and validates diff EOF marker placement.
Build,
types,
initial naming tests,
provenance tests and existing corpus-read tests pass in
`~/temp/agent/archive-provenance-green-20260910.out`.

Remaining work includes:

- Complete lint remediation.
  `archive-naming-format-20260910.out` is a failing intermediate checkpoint,
  not a clean result.
- Replace normalized-line text lookup with coordinate provenance through shared normalization.
  Duplicate lines must retain their distinct origins;
  current code conservatively withholds them.
- Finish roots,
  merges,
  renames,
  shallow,
  reversion,
  unusual-path,
  CRLF/bare-CR,
  normalization/stub,
  duplicate-scope,
  stale-anchor,
  malformed-output,
  cancellation,
  routing and lazy-promisor fixtures.
- Run guard mutations against committed code and show each relevant test fails.
- Verify the real pinned Mio lookup without a name or origin-revision constant.
- Update package documentation and run the complete package suite,
  reading its final `unit exit 0`.

The reviewer questioned the word “choice,”
then explicitly reconciled it with the design review:
neutral history stays beneath the qualification boundary;
the renderer may use the successful phrase as an operational name for the recorded current form.
It does not assert intent or that the predecessor was a name.
The decision records this definition.
No fallback to the already-failed generic wording treatment was adopted.

[qualified-decision]: ../decision/translation-repair-qualified-archive-naming-revisions.md
