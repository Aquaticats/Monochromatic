# Temporal and participant source context: experiments isolating the conflicting-diagnosis boundary

The experiments after the first repair-path check:
context-purpose rules for critics and panels,
claim extent and document evidence,
the editor responsibility treatment,
the accurate-claim control,
isolated and clustered panel review,
and the selector evidence checks;
the claim-extent and editor responsibility treatments were not integrated,
while the evidence roles,
cluster packets and selector evidence they measured were implemented later
in the cluster panel file.

Part of [Temporal and participant source context](translation-repair-temporal-context-2026-09-10.md),
which lists every file of this record.

## Next measured boundary

The critic's existing nearby-context rule permits only locating material carried across passages.
The panel's rule also concentrates on relocation.
A candidate's coverage obligations,
its editable target,
and the source evidence that can verify a current claim are different boundaries.
Treating "not required to translate" as "cannot support any current wording" defeats the supplied context.

The next hypothesis changes those existing context-purpose rules,
not the documents,
roster,
quorums,
response schema or production graph.
Nearby source may verify references,
time,
participants and details already asserted in the current target.
It adds no obligation to translate neighboring passages.
Nearby archive shows placement,
not factual authority.
Critic quotes remain anchored to the current documents;
when source evidence exists only nearby,
`sourceQuote` can be omitted while a current `targetQuote` anchors the claim.

The independent advisor supports this focused experiment and identifies a falsifier:
a compound claim may bundle a supported item with an unsupported one,
then continue authorizing deletion of the whole list.
Inspect propositions and actual tally outcomes,
not just whether one new temporal claim appears.
A fixed original panel packet tests the old claims only;
new critic claims need their own newly built panel packet before any end-to-end claim.
Task 19 remains in progress.
No new production prompt change has been made.

## Completed evidence-scope experiment

`proc_12b5` completed `translation-repair-evidence-scope-probe-20260910` in 476 seconds.
Script:
`~/temp/agent/probe-repair-evidence-scope-20260910.mjs`.
Plan:
`~/temp/agent/repair-evidence-scope-plan-20260910.out`.
Log:
`~/temp/agent/repair-evidence-scope-probe-20260910.log`.
Report:
`~/temp/agent/repair-evidence-scope-probe-20260910/report.json`.

The offline plan reconstructs real critic claims,
clusters and panel outcomes from twelve cached calls,
with zero misses and messages equal to the captured full-repair requests.
It checks the actual exchange field is 360000 ms.

The live experiment first applies the context-role treatment to the unchanged original panel packet.
It then runs the real critic stage under the critic treatment,
including exact quote anchoring,
and builds a fresh panel packet from surviving new claims.
The real panel stage and `tallyVotes` determine authorization in both panel runs.

Only existing user-message context-purpose rules change.
System instructions,
name policy,
source and target text,
neighbor text,
response schemas and stock role allocation remain fixed.
The first fixed-packet panel uses the same prompt-dependent original cohort;
the newly derived packet may select a different stock window,
which must not be misreported as a matched per-seat comparison.

The cap is fifty-four live JSON requests,
with a 1200000 ms global bound and shared abort on request exhaustion.
No editor calls or new production stages are bought by this experiment.
The run made twenty live requests.
On the fixed old packet,
both blanket list-addition claims moved from accepted to `needs-human`,
each with three supporting and three opposing votes.
That removes their authorization but is not unanimous rejection.

The new critic stage produced thirty-two anchored claims.
Several correctly identify the friend as the discloser to Mio,
and one explicitly places the reconnection disclosure in 2022.
They omit `sourceQuote` and retain exact target anchors,
so the suspected anchoring obstacle does not discard them.

The freshly derived panel accepts those precise claims,
but also accepts new blanket list-removal claims with five supporting votes.
It also accepts claims against the second friendship bullet,
whose supporting source section is outside the bounded window.
Thus the advisor's compound-claim/scope falsifier remains live;
the treatment alone is not sufficient.
No production prompt change is integrated from this result.

The run logged 0.00237755 USD on Bedrock,
two OpenRouter calls reporting zero,
and eight unpriced Hyper plus six unpriced Synthetic calls.
The daily helper ran afterward.

## Completed claim-extent and source-evidence experiment

`proc_9549` completed `translation-repair-claim-extent-probe-20260910` in 576 seconds.
Script:
`~/temp/agent/probe-repair-claim-extent-20260910.mjs`.
Plan:
`~/temp/agent/repair-claim-extent-plan-20260910.out`.
Log:
`~/temp/agent/repair-claim-extent-probe-20260910.log`.
Report:
`~/temp/agent/repair-claim-extent-probe-20260910/report.json`.

It keeps the new thirty-two-claim packet fixed,
reproducing the derived-panel baseline from seven cached requests with zero misses.
The baseline includes the unreadable DeepSeek-flash reply and its recovery attempt;
it is not filtered to usable responses.
An offline assertion initially mishandled the additional recovery user message.
The transformation now targets only the one original context-bearing user message,
preserves recovery instructions,
and separately asserts that no baseline client exception was swallowed.
The corrected plan passes before live calls.

The experiment factors two changes:

- Full current-entry Chinese source as additional factual evidence only,
  without expanding current editable or coverage scope.
  This source is 1755 characters;
  the experiment asserts its unchanged hash and an 8000-character input bound.
  No global whole-document context policy is adopted.
- A generic claim-extent rule:
  an unsupported item cannot prove that every item in its list or span is unsupported;
  narrow actual errors remain independently judgeable.

It measures each change alone and the combination,
with the completed scoped-window baseline retained as the comparator.
Name policy,
claim identities,
stock panel roster,
schemas,
nearby archive and actual tally rules stay unchanged.
The limit is fifty-four live requests,
360000 ms per exchange and 1200000 ms globally.
No critics,
editors or new production stages are added in this experiment.

The run made twenty-one live calls.
Claim extent alone still authorizes blanket removal.
Document evidence alone changes one blanket claim to `needs-human`,
but leaves false claims against the supported second-friendship bullet authorized.
The combined treatment again authorizes blanket removal.
The true participant correction remains accepted in all arms.
These results do not justify integrating either treatment as the remedy.

Per-claim participation differs from reported heard-panelist counts in two arms:
one reply uses severity words as votes,
and another omits verdicts for later claims.
The real resolver records `unknown-vote` and `missing-verdict` findings;
do not count these as valid support or opposition.
The combined arm has no such findings but still fails the intended authorization result.

The run logged 0.00470453 USD on Bedrock,
three OpenRouter calls reporting zero,
and ten unpriced Hyper plus two unpriced Synthetic calls.
The daily helper ran afterward.

## Completed editor responsibility experiment

Further voting-only prompt trials are not scheduled now.
The independent advisor recommends testing the existing editor's responsibility boundary
against actual assembled candidates.

`proc_b599` completed `translation-repair-editor-responsibility-probe-20260910` in 388 seconds.
Script:
`~/temp/agent/probe-editor-responsibility-20260910.mjs`.
Plan:
`~/temp/agent/editor-responsibility-plan-20260910.out`.
Log:
`~/temp/agent/editor-responsibility-probe-20260910.log`.
Report:
`~/temp/agent/editor-responsibility-probe-20260910/report.json`.

Both arms use the same newly derived accepted-issue packet,
including the true participant correction and false blanket claims.
This is not a comparison against old editor outputs that received different issues.
The production deduplication and envelope builders yield sixteen accepted issues and four edit regions.
The editor models are the unchanged stock trio.

The treatment replaces "reviewers confirmed" with a responsibility to verify fallible findings,
and rewrites the conflicting source-support rule rather than appending another exception.
The current source slice defines coverage;
the complete hash-checked original document supplies factual evidence;
only the current English edit regions may change.
It permits correcting an already-asserted fact's actor,
direction,
time or relationship without importing neighboring events or copying neighboring passages.
Name policy is unchanged and remains task 21.

The probe makes six calls maximum,
three concurrently,
with the existing response schema,
360000 ms exchange bound and 1200000 ms global bound.
It calls no panel or selector.
Every raw edit is resolved and applied through production `buildEditorCandidates`,
including the existing preservation gate.
It records complete assembled candidates,
rejected operations,
MDX compilation and the existing structure validator's result.
The validator is observed,
not added as a new publication gate.

The offline positive control applies a localized participant correction through the same assembler,
preserves all bytes outside its region,
and passes the existing structure check.
All three baseline editors remove the list and change the established group name.
All three treatment editors do the same;
one moves the administrator detail into the preceding region and deletes the list region entirely.
Every assembled candidate compiles,
but every candidate fails the existing archive block-shape check.
No candidate supplies the intended retained participant correction.
This falsifies the tested editor-responsibility treatment on this packet;
it is not integrated.

The run logged four unpriced Hyper and two unpriced Synthetic calls.
The per-provider spend and daily helpers ran afterward.
No reported dollar amount is treated as proof that unpriced service was free.

A further caller trace found that `editor-ensemble.ts` supplies no neighboring-source evidence
to either `selectPerEnvelope` or `selectChunkPatch`.
Per-envelope selection sees the current English region and a bounded English surroundings view;
whole-chunk selection sees only the current original as evidence.
Thus a successful new editor candidate will still require verification at those selection boundaries.
Do not claim the existing repair window already reaches every repair selector.

## Completed accurate-claim diagnostic control

`proc_ec6f` completed `translation-repair-editor-accurate-claim-probe-20260910` in 262 seconds.
Script:
`~/temp/agent/probe-editor-accurate-claim-20260910.mjs`.
Plan:
`~/temp/agent/editor-accurate-claim-plan-20260910.out`.
Log:
`~/temp/agent/editor-accurate-claim-probe-20260910.log`.
Report:
`~/temp/agent/editor-accurate-claim-probe-20260910/report.json`.

This is a diagnostic positive control,
not a proposal to silently filter production claims.
The stock editor receives only the source-verified participant claim,
while the current source,
translation,
neighboring context,
stock editor models,
four edit regions and original preservation licenses remain unchanged.
Keeping the original licenses prevents a stricter gate from creating an apparent improvement.
Only the visible issue packet changes.

Three calls are allowed,
with a 360000 ms exchange bound and 1200000 ms global bound.
The real assembler and existing structure validator process each response.
Compare raw operations and assembled candidates against the completed stock-editor arm,
not just whether the process exits successfully.
All three editors correct the disclosure direction and retain the list,
second friendship item,
administrator item and established `Harunome Hanbai` name.
Both GLM outputs explicitly place disclosure at reconnection.
DeepSeek's bullet says the friend came out to Mio without adding a childhood timing claim,
but its other prose also repeats the administrator detail.
All three assembled candidates pass the existing structure check.

The editors also change other still-editable regions despite those regions having no visible issue in this control.
Those changes do not make the complete candidates automatically publishable;
the diagnostic establishes that the editors can produce the intended list correction with a non-conflicting packet.
It does not justify a production gold-label filter or claim that every problem is solved.

The run logged two unpriced Hyper and one unpriced Synthetic call.
The spend and daily helpers ran afterward.
No production prompt or admission-rule change is made.

## Completed isolated-claim panel diagnostic

`proc_02d2` completed `translation-repair-panel-isolated-claims-probe-20260910` in 42 seconds.
Script:
`~/temp/agent/probe-panel-isolated-claims-20260910.mjs`.
Plan:
`~/temp/agent/panel-isolated-claims-plan-20260910.out`.
Log:
`~/temp/agent/panel-isolated-claims-probe-20260910.log`.
Report:
`~/temp/agent/panel-isolated-claims-probe-20260910/report.json`.

The next control isolates the accurate participant claim and the false second-friendship addition claim,
using the same source,
translation,
full original evidence,
system instructions and context-purpose rule as the completed document-evidence-only panel arm.
The production builder renumbers each isolated packet;
this is a packet-content/size diagnostic,
not a claim that numeric position has been independently eliminated as a factor.

The offline plan found eight distinct panelists had actually been asked in that full-packet arm,
although only six were heard.
The control includes all eight rather than selecting favorable or usable responses.
Compare each model's matched verdict separately from newly usable responses.
The real resolver and tally retain the original nine-seat configured basis.

The limit is sixteen calls,
six concurrent,
360000 ms per exchange and 1200000 ms globally.
No per-claim production calls or admission filter is introduced.
The true participant claim is accepted with seven supporting votes and one abstention.
DeepSeek-flash returned `supported, severity: major` in the vote field;
the real resolver records that as an abstention,
not a supported vote.

The false second-friendship addition claim is rejected with six opposing and two supporting votes.
Among the original six responding panelists,
DeepSeek Pro and Mercury switch from supported in the full packet to unsupported in isolation.
Gemma-e2b and Gemma-26b remain supportive;
Kimi and Qwen remain opposed.
Thus the matched responding set changes from four-to-two support to four-to-two opposition,
independent of the newly usable MiniMax and DeepSeek-flash replies.

This supports packet-content/size interference;
it does not prove that renumbering or every other presentation factor is irrelevant.
The run logged 0.00161704 USD on Bedrock,
two OpenRouter calls reporting zero,
and six unpriced Hyper plus four unpriced Synthetic calls.
The daily helper ran afterward.

## Completed existing-cluster review diagnostic

`proc_2d45` completed `translation-repair-panel-clusters-probe-20260910` in 430 seconds.
Script:
`~/temp/agent/probe-panel-clusters-20260910.mjs`.
Plan:
`~/temp/agent/panel-clusters-plan-20260910.out`.
Log:
`~/temp/agent/panel-clusters-probe-20260910.log`.
Report:
`~/temp/agent/panel-clusters-probe-20260910/report.json`.

The original aggregation contains five clusters with three,
six,
three,
five and fifteen claims.
The last cluster contains the list's distinct participant,
friendship,
name and administrator claims;
clustering proposes possible merges and does not already prove they are one defect.

The experiment uses those existing clusters as separate review units,
retaining every claim and the within-cluster merge question.
It holds the original eight asked panelists,
full source evidence,
current source/target,
context-purpose rule,
system instructions,
schemas and configured nine-seat tally basis fixed.
The production builder,
resolver and tally run independently for each cluster.

Forty calls maximum,
six concurrent,
360000 ms per exchange and 1200000 ms globally.
No production batching change or additional generation round has been implemented.
The mixed list cluster no longer authorizes blanket list deletion:
the broadest claim ties four-to-four and the two-item removal claim is rejected three-to-five.
Both false claims against the second friendship bullet are rejected two-to-six.
The actual participant correction remains accepted seven-to-one.
Name-change claims and administrator-placement claims remain accepted;
this does not complete task 21 or establish a good final candidate.
The existing resolver records malformed vote/group replies rather than silently counting them.

The non-list clusters retain supported corrections and reject the spurious clinical/tense claims.
The run logged 0.00473602 USD on Bedrock,
five OpenRouter calls reporting zero,
and fifteen unpriced Hyper plus ten unpriced Synthetic calls.
The daily helper ran afterward.

## Completed stock-window cluster-to-editor verification

`proc_ba3f` completed `translation-repair-cluster-repair-integration-20260910` in 155 seconds.
Script:
`~/temp/agent/probe-cluster-repair-integration-20260910.mjs`.
Plan:
`~/temp/agent/cluster-repair-integration-plan-20260910.out`.
Log:
`~/temp/agent/cluster-repair-integration-probe-20260910.log`.
Report:
`~/temp/agent/cluster-repair-integration-probe-20260910/report.json`.

This moves from the fixed eight-panelist diagnostic to actual prompt-dependent stock windows.
`gatherStageVoices` receives the final context-bearing messages before it selects seats from the nine-seat panel.
The resolver and tally run independently per existing cluster,
retaining each cluster's actual ballots and configured basis rather than reinterpreting a union of seats.
Completed cluster-probe payloads seed a copied disposable cache.

The resulting issues go through production deduplication,
envelope derivation,
the stock editor prompt and the existing patch/preservation gate.
No editor responsibility treatment is used.
The report retains raw editor operations,
complete assembled candidates,
compilation and the existing structure check.
It does not yet call the repair selectors that lack neighboring source.

The bound is eighty JSON requests,
360000 ms per exchange and 1200000 ms globally.
No production batching change has been made.
The actual windows made thirty panel calls followed by three editor calls.
The resulting packet has more narrowly scoped edit regions and preserves the second friendship item outside the edits.
One editor,
GLM-5.3,
produces the correct friend-to-Mio disclosure bullet and retains the second friendship bullet.
It moves the administrator fact into prose and changes the group name,
so it is not a complete publication-ready result.
GLM-flash removes the first bullet;
DeepSeek replaces it with another copy of the second friendship bullet.
All three pass the structural check,
which correctly does not establish semantic accuracy.

The run logged six unpriced Synthetic and two unpriced Hyper calls;
completed payloads supplied cached requests.
The spend and daily helpers ran afterward.
No production batching change is yet applied.

## Completed repair-selector evidence check

`proc_ae78` completed `translation-repair-selection-context-probe-20260910` in 273 seconds.
Script:
`~/temp/agent/probe-repair-selection-context-20260910.mjs`.
Plan:
`~/temp/agent/repair-selection-context-plan-20260910.out`.
Log:
`~/temp/agent/repair-selection-context-probe-20260910.log`.
Report:
`~/temp/agent/repair-selection-context-probe-20260910/report.json`.

It uses actual candidates from the completed cluster/editor run,
with their actual authorship and unchanged fallbacks.
The stock per-envelope selector judges only the affected disclosure region;
the stock whole-chunk selector separately judges the three assembled candidates.
No invented composite or new writing is introduced.

The contrast adds nearby and whole-document source evidence,
without changing system criteria,
candidate wording or available judge roster.
The interceptor preserves the original prompt-dependent windows for the paired comparison;
a later compiled change still needs production-window verification.
The offline plan exercises both selector APIs,
checks the exchange field,
and preserves any additional recovery user messages instead of mistaking them for the candidate sheet.

The cap is sixty-four requests,
360000 ms per exchange and 1200000 ms globally.
The baseline regional selector declines and keeps the incorrect current bullet.
With source evidence,
it selects the correct friend-to-Mio replacement at weight four over six ballots.
This is an actual selected regional operation,
not a fallback or a vote-count-only claim.

The whole-chunk baseline selects the deleting candidate.
Added source evidence moves the whole-chunk vote to a three-to-three tie,
which invokes the existing indecision fallback and still returns that deleting candidate.
Do not describe the whole-chunk treatment as a successful selection.
Several reasons still call the corrected first bullet newly added,
but the chunk selector has never been shown the existing English that already carries the item.

The run logged 0.00158577 USD on Bedrock,
0.00105248 USD on OpenRouter,
and six unpriced Hyper plus eight unpriced Synthetic calls.
The daily helper ran afterward.
The unresolved group-name mutation remains task 21.

## Completed existing-English comparison check

`proc_3b2b` completed `translation-repair-selection-baseline-probe-20260910` in 58 seconds.
Script:
`~/temp/agent/probe-repair-selection-baseline-20260910.mjs`.
Plan:
`~/temp/agent/repair-selection-baseline-plan-20260910.out`.
Log:
`~/temp/agent/repair-selection-baseline-probe-20260910.log`.
Report:
`~/temp/agent/repair-selection-baseline-probe-20260910/report.json`.

This adds only the existing English before repair to the completed source-evidence whole-chunk treatment.
Source evidence,
candidate wording,
authorship,
fallbacks,
system criteria and original judge window remain fixed.
The existing English is comparison context for what was already carried,
not independent factual authority.
Every offered candidate is an actual changed text,
not the unchanged baseline;
no model identity is disclosed through this comparison block.

The probe calls only the stock whole-chunk selector,
with at most twenty-four requests and the same 360000/1200000 ms bounds.
The offline plan checks both the candidate-sheet transformation and recovery-message preservation.
No new writing or invented composite is introduced.
The actual whole-chunk selector chooses GLM-5.3's correct-disclosure candidate at weight five over six ballots,
not by fallback.
Its group-name mutation remains the separate task 21 blocker.
The run logged 0.00079041 USD on Bedrock,
one OpenRouter call reporting zero,
and two unpriced Hyper plus one unpriced Synthetic call.
The daily helper ran afterward.
