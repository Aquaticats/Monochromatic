# Status: pass-path rules by commit, 2026-09-02 to 2026-09-16

The validation of the front-matter path on 2026-08-29 that opens the file,
what stayed in the package after the provider-free layer was removed on 2026-09-16,
and the rules added to the pass path from 2026-09-02 on,
listed by the commit that made each:
the front-matter guard,
verse breaks,
archive revision and original rules,
footnote assembly classes,
quorum sizing,
roster seating and provider client changes.

Part of [Status](status.md),
which lists every file of this record.

## The b716eb99e validation and pass-path rules by commit, 2026-09-16 back to 2026-09-02

Fresh `b716eb99e` validation reached four slices and all nine active seats,
but correctly published no artifact or page.
The translate lane produced changed metadata whose visible name followed source identity,
then lane contest selected the unchanged repair metadata.
Consolidation produced another changed candidate,
but its final gate restored exact incumbent metadata.
The final guard refused that output as `incumbent-fallback` on three attempts;
subsequent attempt stalled because invalid contest and consolidation decisions had become cache-terminal.
This confirms final refusal but not live closure of the front-matter path.
Candidate validation now preserves source `name` and `info.alias` identity equality,
contest winners that fail publication invariants remain retryable,
and consolidation standing text must pass same syntax guard before unchanged result becomes resumable.
Final-page validation applies same source identity relation even when page differs bytewise from archive.
Since `cbedea357` (2026-09-16) the package again holds only what a pass or a probe reaches.
The sessions of 2026-09-10 to 2026-09-15 added a provider-free layer of 143 modules (archive naming,
a container runner for a "producer input" comparison,
preparation attempt,
receipt,
root and selection records,
a persisted-input DTO reader,
request capture) that no build entry reached and no pass used;
it is removed with its tests,
its three build entries and its bootstrap and sealed rolldown configs.
The pass-path additions of those sessions stay and are described in the paragraphs that follow.

Since `34e5c7ecd` (2026-09-02,
owner's decision in `doc/decision/translation-repair-front-matter-guard.md`) the final guard is structural only:
the metadata slice sits at slice zero over both sides' front-matter bytes,
the page parses,
the identity and attribution rules hold,
and the page's visible name is not the directory id where the source names the person differently (`directory-id-name`,
checked on the assembled page whether or not it equals the archive;
since `6d85b619a` a handle that is the name in both languages,
as for 8 of the pinned corpus's 92 entries,
passes).
The owner delegated the verse remedy after rejecting the guard-first approach.
Since `1a6ebbf62`,
writers receive parsed explicit-break facts before generation,
and selectors receive the same rendered-line contract with counts matched to their actual anonymous slate.
Author repair inherits that contract;
lane comparison and consolidation use it too.
It distinguishes visible breaks from soft newlines and recommends unambiguous `<br/>` spelling.
Canonical source,
archive-backed layouts and ordinary-passage prompts are unchanged.
Since `b3113164a`,
initial source-only writers also see actual Markdown break syntax displayed as `<br/>`.
Only parser-confirmed break spans change in that view;
canonical text and offsets remain untouched.
The caller must explicitly establish an absent incumbent,
and metadata and nonempty archive wording are excluded.
The measured source-spelling treatment gave both probed writers five breaks without a trailing break,
where Mercury had still flattened its initial output under instructions alone.
The deterministic floor remains a backstop,
not proof that the model behavior is repaired.
The bounded writer/selection comparison is recorded in
[the verse-remedy review](../../../doc/planning/translation-repair-verse-remedy-review-2026-09-10.md).
Since `7c0ce152f` (2026-09-10,
the twenty-ninth class) a source-only passage must preserve a minimum count of explicit line breaks
within each top-level block.
Mio10's missing-archive poem kept its words but lost all five Markdown hard breaks,
which the blank-separated-block verse heuristic and physical-line count did not detect.
The skeleton now counts parsed Markdown breaks and intrinsic lowercase `br` elements,
not soft newlines,
code or custom components.
Nonempty archive text stays outside this check;
expansion and archive-backed formatting choices are unchanged.
The guard failed before the fix,
and the publisher test preserves accepted breaks through wrapping and insertion into a disposable output tree.
Since `dc51b02d9` (2026-09-10,
the twenty-eighth class) archive revision proposals may carry a nonempty `sourceQuote` as supporting evidence,
matching the declared JSON schema.
The handwritten guard had rejected those otherwise readable MiniMax and Gemma revisions on the ninth `Mio`,
leaving archive review below quorum and stopping the pass before lanes.
A revision still faces independent selection;
only `source-supported` retention requires a nonempty anchored quote.
The real-stage guard failed `provider-unavailable` before the fix and passes after it.
Since `ad6d506a6` (2026-09-09,
the twenty-seventh class) an assembly with an unattributed structural regression tries each single replacement
withdrawal before withdrawing the whole lane.
Each trial splices and parses the whole document,
including slice joins,
and must leave no introduced structural or footnote defect against the archive.
The first successful trial preserves every other replacement;
when none succeeds,
the existing blanket fallback remains.
The eighth `Mio` had selected malformed JSX at slice 16,
then withdrawn all 15 translations,
including a valid linked poem at slice 17,
and stopped at the publisher for its missing destination.
The author-defense policy is unchanged;
the added proof is deterministic and buys no model calls.
The reproduction failed first,
then passed with guards for unrepaired multi-slice damage,
container joins and a counterfactual that still breaks footnotes.
Since `f6cd6e5e7` and `07a99e23a` (2026-09-09,
the twenty-fifth class) archive block review sees corroborated transcriptions of pictures referenced by its
aligned source section,
under `CORROBORATED PICTURE SOURCE SUPPORT` with each reader's wording labelled.
The optional `preparePassEntry.readPictures` seam runs before a review can remove an archive translation of
picture text;
`pass-entry.ts` binds one entry-scoped reader for preparation and the final picture phase.
Completed readings are retained in memory within that pinned entry,
not merely persisted to the disk cache whose open `resumed` map is a snapshot.
Changed slice boundaries or later reader reseating do not purchase completed pictures again;
newly exposed references still get read.
Target-only sections and unrelated,
textless or unavailable readings add no source support.
The preservation,
section-scope and no-repurchase guards were shown failing on the pre-fix build before passing.
Since `40aba2fdb` (2026-09-09) every stage gather sizes its quorum on the seats a wet provider serves,
not on the seats the phase seated:
`reachableQuorum` keeps the bench quorum while the reachable seats can meet it and otherwise needs half of
them rounded up,
never fewer than two (`MIN_STAGE_VOICES`),
a seat the router refused is dropped from the pending list rather than re-asked,
and a short gather carries `stage-short-bench (<stage> reachable r of n, quorum q)` into the artifact
(`stage-reachable-quorum.ts`;
the 2026-09-09 addendum of `doc/decision/translation-repair-short-bench-share.md`).
Since `a5e0efc7f` (2026-09-09) `deepseek-v4-pro-0813` writes no translate-lane candidate:
two 40-round producer calibrations read it below the pooled null,
the second across the Bonferroni threshold (z -2.31 on 2026-09-08,
z -3.18 on 2026-09-09),
so it joins `TRANSLATOR_DROPPED` and keeps every judge seat,
the consolidation seat and its place as the price anchor.
Since `028432713` (2026-09-09) `inception/mercury-2.5` writes in both lanes:
the 40-round calibration of that day read it at 18 of 101 disinterested ballots,
z -0.43 against a 19.5 percent pooled null,
not separated from it,
so `WRITER_UNMEASURED` is empty again and its completion cap is the pooled 90th percentile,
measured off its own 136 calls.
Since `19b6043a3` (2026-09-09) the owner's 2026-09-03 cost decision on Kimi-K3 is a fact of the reach,
not only of the seat reader:
`OPENROUTER_WITHHELD` lives in `openrouter-catalog.ts`,
`reachOf` and the picture reach say OpenRouter does not serve a withheld model,
and a seated Kimi-K3 whose provider dries mid-phase is an unreachable seat rather than 22 calls at 3 and 15 USD
per million (1.14 USD on the third `noname` pass,
61 percent of its OpenRouter spend);
the seat reader withholds the seat whose only wet provider would have been OpenRouter,
so the substitute checker sits as before.
Since `db5927630` (2026-09-09,
the owner's answer to the thin-bench question) a select round whose bench is short of quorum,
because the router refused seats for want of a wet provider,
seats a winner by a share of the reachable weight:
the minimum scales as `MIN_SELECTION_WEIGHT` times reachable over quorum,
two ballots naming the winner is the floor (`MIN_SELECTION_BALLOTS`),
a bench at or above quorum keeps the absolute 2,
and the round's findings carry `select-short-bench (reachable r of n, minimum m)`
(`candidate-select-minimum.ts`;
`doc/decision/translation-repair-short-bench-share.md`).
Since `1463cd359` (2026-09-09,
the owner's answer to the incumbent question) a consolidation whose standing failed the deterministic gate runs
against the incumbent where the incumbent passes it,
with `ineligible-standing-replaced-by-incumbent` recorded and no contest endorsement,
and the artifact's shipped kind `incumbent` has the page write the incumbent's text;
the entry stops only where the incumbent fails too,
or is the standing (the 2026-09-09 addendum of `doc/decision/translation-repair-ineligible-standing.md`).
Since `b7a0b4f5f` (2026-09-09,
the owner's answer to the glossary question) `community-glossary.ts` beside the corpus pin lists the community's
terms (自切 as "self-surgery",
超天酱 as "KAngel" of the game Needy Streamer Overload,
the archive's renderings),
the terms an entry's source carries ride in its identity context as `COMMUNITY TERMS`,
and the select,
lane-contest and consolidation-gate sheets name each candidate lacking every accepted rendering in a
`COMMUNITY RENDERINGS` block,
evidence to weigh rather than a bar;
the owner curates the file (`doc/decision/translation-repair-community-glossary.md`).
Since `3224ff347` and `fcc8ca197` (2026-09-09,
the owner's "Mercury 2.5 is out and approved") `inception/mercury-2.5` is the twelfth roster model and the one
only OpenRouter serves (its roster id is OpenRouter's spelling;
0.04 and 0.15 USD per million,
text only,
one endpoint),
seated in critic,
panel and judge in both lanes,
the late bench and the slate on the fidelity probe of that day
(14 of 14 distinct questions chose the complete text,
no damaged pick,
no decline;
recorded on its card in `src/model-cards.ts` (moved on 2026-10-18 to
`package/translation-repair/shared-contract/src/model-cards.ts`)),
and held out of every writing seat by `WRITER_UNMEASURED` until the 40-round producer calibration is read;
a model one provider alone serves holds no seat until a measurement seats it,
whichever provider it is.
Since `78ea8c8c7` (2026-09-09,
the owner's "do everything in our power to NOT bleed") every call on every provider carries `max_tokens`
at a measured ceiling per roster model (`completion-cap.ts`:
the highest 99th percentile of completion tokens any provider with at least 100 calls of the model recorded over
142,437 completed calls,
floored at the pooled 90th of 3,831;
each cap cuts under one percent of that model's completed calls),
a caller's own ceiling only ever lowers it,
Hyper takes the lower of the cap and its own per-model ceiling,
and a reply that meets the cap ends `finish_reason=length` and is read as a truncated voice;
the reasoning controls stay off the wire as the owner's 2026-08-25 instruction requires.
Since `b71a55385` and `33a023445` (2026-09-09) every round asks quorum plus one seat,
from a bench rotated deterministically by the prompt (`stage-fanout-window.ts`),
and the rest only when a voice is lost:
the sixteen `gatherStageVoices` stages through their retry rounds,
and the six stages that read their own round (`stage-windowed-rounds.ts`;
the lane contest,
section and block pairing,
the consolidation gate,
the naturalness review and the polish gate) through retry rounds they never had before,
each returning one outcome per seat asked in roster order;
quorum is unchanged and still computed over the whole bench.
With it:
the coverage verdict takes its majority over the seats the gather asked (`gather.asked`),
a judge bench whose window could not carry a unanimous self-written slate on self-votes alone asks the whole bench
(`candidate-select-fanout.ts`;
four seats),
the naturalness confirmation challenges exactly the seats the discovery asked at the discovery's quorum,
a fixture scripting every seat passes `fanOut: 'whole-bench'`,
and a fixture counting calls counts `firstRoundWindow({ benchSize, })`.

## Naturalness quorum basis, name authority, archive review and panel packets

Generation fourteen (`725a31b12`,
completed through `b5da9866b` on 2026-09-10) records each naturalness round's effective `quorumOver`.
Mio12 exposed why the asked-seat list is not that basis:
four acceptable voices among six asked seats failed the runtime's wider nine-seat quorum,
but the artifact reader recomputed a three-voice threshold and refused the correct runtime verdict.
The new reader derives from the stored wider basis,
checks confirmation uses that same basis,
and preserves the seat-count interpretation only for older generations.
Runtime rejects invalid bases before asking reviewers.
No missing reviewer identities are invented and the intended quorum is not reduced.

Since `514db9b1f`,
name authority distinguishes references to people from literal words,
characters or spellings under discussion.
Writers,
critics and deciding criteria preserve that discussed form instead of normalizing it to the declared name.
Ordinary surrounding prose still becomes English.
`8a59469f2` also separates established translation-side names from generic terms needing a first-use gloss;
ordinary terminology and explicit source definitions remain covered by the existing rule.
The real bounded translation stage on verified `85f7bac01` produced and selected a fresh wording retaining
Mio's character/name distinction,
with all four responding judges choosing it.
The measured group-name votes no longer authorize the false gloss correction under the existing tally rule.
These are preventive instructions,
not new output-rejection gates or a new glossary entry.

Archive review since `bfcb5b568` separates retention-anchor sufficiency from revision selection.
An actual revision may reach independent selection after the initial review quorum,
without enough anchored retention opinions.
The selector receives the unchanged original,
archive context and anonymous typed opinions linked to the actual candidates;
unanchored retention claims stay audit-only.
Neither initial nor selector quorum is reduced.
`ba01babda` clarifies the first review's complete minimal English correction responsibility.
The compiled-stage Mio chat probe selected a corrected full block while preserving its useful `Translation:` label.
Retained-label naturalness audits still produce inaccurate findings;
context trials have not established a remedy and further work is deferred.
The corpus wrapper discards their detailed findings and does not let their verdicts change text,
so this audit-quality issue is not a page-readiness blocker.
The awaited calls still consume resources and can propagate operational failures.
These stage results do not establish whole-page readiness.

Since `e086402a6`,
a following heading-only source slice brings its next body into the neighboring-context window.
This is a single forward extension,
not a search through distant sections;
metadata,
empty content or another heading stop it.
Source and archive views use the same positions.
The `134a2e20c` trial reproduced the measured Mio context but still removed the list and changed a group name.
Subsequent measurements led to clustered panel packets and source-evidence handoffs,
verified locally through `8adb77fb9`.
`runPanelStage` now preplans one packet per existing cluster,
uses bounded sequential packet execution and keeps every claim's configured electorate separate.
Complete same-entry source evidence participates in repair cache/twin identity and reaches the panel and both selectors.
Whole-chunk selection also receives the known existing English before repair,
so an already-carried detail is not mistaken for a new addition.
The compiled full-path check still removed the disclosure item,
and group-name authority remains unresolved.
`029c0a886` adds measured guidance distinguishing a distorted source-grounded event from independent added content,
and asks adjudicators to check the claimed category as well as the existence of an error.
Genuine addition and omission controls remain actionable.
It does not change the category enum or automatically filter claims.
`b9d3b2ea0` preserves each member's effective decision through issue merging:
accepted issues no longer carry rejected diagnoses as confirmed editor instructions.
Source-defect protection still blocks a whole panel-merged cluster.
Emission deduplication preserves known claim evidence.
The compiled repair path now generates and selects the correct friend-to-Mio disclosure,
retaining the list and avoiding the childhood-time attachment.
The group-name mutation remains open;
these checks do not establish whole-page readiness.
Initial translation writers remain unchanged because their separate context experiment did not correct the bullet.

## Pass-path rules by commit, 2026-09-09 back to 2026-09-04

Since `037d1f650`,
`4f87555fc` and `1fe7ca2fe` (2026-09-09) the OpenRouter client sends `provider.sort: 'price'` with the
zero-data-retention and `require_parameters` preferences,
prints `cached=N` on the `SPEND` line off `usage.prompt_tokens_details.cached_tokens`,
writes a `SPEND ... estimated=abandoned` line reckoned from the delivered characters for every stream a round
abandoned,
and no longer serves `hf:Qwen/Qwen3.8-27B` or `glm-5.3` (`OPENROUTER_DROPPED_SEATS`),
after the reckoning of 2026-09-09 found 58 USD of a 200 USD day in abandoned streams the endpoints billed to the end
and the anchor judge routed to endpoints at twice the listing price.
Since `efc9a4f3c` (2026-09-09,
the twenty-third class) pair agreement decides a target two sources claim without a corroborated merge by votes,
the way it already decides one source named against two targets:
the better-voted claim keeps the target and the other source is left unpaired,
a tie keeps neither,
and the finding reads `contested target (target N: source A outvotes source B, X to Y)`;
the first `noname` pass had paired the original's `## 简介` heading with the archive's opening paragraph by
source order and shipped the page without the heading.
Since `7bcea2dc4` (2026-09-09,
the twenty-second class) the archive-original recognizer reads a whole-page note written in English:
`(Original Language: English)`,
and the pinned corpus's own `(Original Language: Engish)` on `gqt`,
matched on the lowercased note,
decline the entry the way the Chinese wordings do;
four `gqt` passes had repaired an English original from its Chinese back-translation.
Since `3620004db` (2026-09-09,
the twenty-first class's fourth face) the composed page runs through the same footnote assembly guard each lane
runs,
and the artifact records the outcome as `pageAssembly` (generation thirteen):
the slices the guard trimmed with the text the page carries,
the slices it took back,
and its findings;
`wouldShipTextFor` applies the section ahead of the polish,
the consolidation and the contest,
so every reader composes the page the guard settled,
and an older artifact reads with an empty section.
The twenty-second `hakureico` pass had trimmed the orphan in the translate lane and shipped the consolidation's
fresh rendering of both notes at the same slice,
which the page guard refused.
Since `379122379` (2026-09-09,
the twenty-first class's third face) the orphan trim reads blocks the way GFM ends definitions:
a definition line opens a block whether or not a blank line precedes it,
a definition's indented continuation stays in its block,
and the gaps between blocks survive a cut as written;
the twenty-first `hakureico` pass had rendered its two notes one line apart,
the trim saw one block,
and the assembly guard withdrew both notes and the reference,
which the page guard then refused to ship.
Both assemblers log every assembly guard finding after the withdraw warning.
Since `9abcbee50` (2026-09-09,
the twenty-first class's second face) a delivery ledger's shipped row carries the text the document carries,
which the assembly guard may have trimmed:
`guardFootnoteAssembly` reports `trimmed`,
both lane results carry `trimmedReplacements`,
`buildSliceDelivery` reads a shipped row's text from them before the decision,
and `assertDeliveryCoherent` accepts a shipped row whose text is its decision with definition blocks cut and
nothing else (`isDefinitionTrim`);
the twentieth `hakureico` pass had stopped at the reassembly invariant with the trim on the page and the judges'
text in the row.
Since `aedee7414` (2026-09-09,
the twenty-first class) an orphan footnote definition takes only its own block,
and the composed page's footnote graph is read before the tally:
`guardFootnoteAssembly` had withdrawn a two-definition insertion whole for the one orphan among them
(the nineteenth `hakureico` page shipped `“Mayday”[^1]` with no note,
since the sealed letter carries no `[^2]`),
so `trimOrphanDefinitions` now cuts the orphan's block out of a definitions-only replacement first
(`assembly-footnote-trimmed`),
and `assertPageFootnotesIntact` reads the would-ship page against the archive and stops the entry as
`page-footnote-integrity`,
run beside the carried guard by `assertPageGuards`.
Since `d6db46519` (2026-09-09,
the twentieth class's second face) a carried insertion's evidence is the full coverage votes' regions alone:
`judgeCoverage` had recorded every anchored quote whatever its degree,
and a partial voter's quote of an unrelated sentence,
rewritten by a lane,
stopped the sixth `yuki418330012` pass at the guard;
an interrupted entry's findings now reach the log as `INTERRUPTED <id>: <finding>` lines before its tally.
Since `11143f681` (2026-09-09,
the twentieth class) the carried-insertion guard reads the page the way the evidence was anchored:
`assertCarriedInsertionsRemain` folds soft line breaks and normalizes punctuation on both sides before it looks
for a carried region,
since the semantic wrap breaks a changed slice at its clauses and the fifth `yuki418330012` pass had stopped as
`carried-evidence-lost` with every word of the credits line on the page
(`Contributors for this entry:\nZhenli,\nSansan,\nSuona` against the anchored one-line quote),
and its findings name each lost region.
Since `1ba94c27a`,
corrected by `ea07a1512`,
completed by `e5ff6c4f8` and closed by `2da6e7f22` (2026-09-08,
the nineteenth class) the archive's footnotes follow the original's before any lane reads a slice:
the block pairing treats footnote definitions as order-free blocks
(`readBlockPairing` takes their indices as `freeOrder` and reads its never-backwards rule over body pairs alone;
the third `yuki418330012` launch had lost six of eight voices for pairing two definitions by content where the
archive had renumbered them),
a crossing definition pair is kept out of the slicing and read for the relabel (`splitDefinitionPairs`),
`footnoteRelabelOfDefinitions` reads the label map off the definitions the roster paired
(the `yuki418330012` archive had renumbered `洲洲[^2]` and `真理[^1]` by first appearance,
and the page shipped the original's markers above the archive's definitions,
each pointing at the other's note),
the positional reading off the paired slices stays as the fallback
(a slice whose sides carry different counts of notes is left out,
named;
the `hakureico` archive carries no `[^2]` at all),
`applyFootnoteRelabel` rewrites references and definition openers in one pass,
`reorderFootnoteDefinitions` moves the archive's definitions into the original's order,
and the pass prepares again over the rewritten archive,
logging `FOOTNOTES entry=<id> relabelled [^2]->[^1], [^1]->[^2] off the definitions the roster paired` and
`FOOTNOTES entry=<id> definitions moved into the original's order: [^1], [^2]`.
The footnote extension of 2026-09-11 keeps supplied correspondence,
existing forced elimination and collision-avoidance displacement separate.
An unmatched archive note may receive a fresh positive-decimal label only after every source label is accounted for;
that move establishes no source correspondence.
Identity relations remain evidence even when they change no bytes.
`footnoteRelabelOf` now takes the complete preparation's source text,
archive text and slices,
projecting positioned document references into checked ranges rather than reparsing container halves.
`applyFootnoteRelabel` matches normalized identities but replaces exact raw spans,
refuses missing changing map domains and merging destinations,
and reparses the resulting marker graph.
Definition movement preserves each separator once and withholds changes across non-blank gaps or split containers.
Any rename or reorder touching declared English-original text or its owning note withholds the entire operation.
The pass records a structured `withheld` reason,
keeps the original preparation when withheld,
and prepares changed text again through its existing caches.
Cache eligibility is unchanged:
an incomplete crossed agreement can require fresh acquisition on replay,
while independently cached original and relabelled questions are reused under their distinct keys.
See `doc/planning/translation-repair-archive-footnote-collisions-2026-09-11.md` for verification status.
Since `439667ec3` (2026-09-08,
the owner's decision in `doc/decision/translation-repair-archive-original.md`) an archive note saying the English
is the original is authority:
a span note (`以下` with `原文` and `英文`) seals the archive from the note's end to the next heading or the end
of the page,
the sealed blocks and the originals paired with them reach no slice
(an original behind the seal becomes an insertion at its end),
the preparation records `archiveOriginalSpans` (artifact generation twelve),
`assertArchiveOriginalComplete` refuses a page that does not carry every sealed span byte for byte,
and the pass logs `ARCHIVE ORIGINAL entry=<id> span=<k> [start, end)`;
a whole-page note (`原文即英文` or `不要动本篇`) declines the entry before any purchase,
`declined/<id>.json` beside the artifacts,
`TALLY <id> status=DECLINED reason=archive-original`,
counted as done by every later pass and reported by `verify-published` as `declined=N`
(`cheonwoomaeng`,
the one such entry in the pinned corpus,
declines in 41 ms).
Since `bb04656ef` (2026-09-08,
the eighteenth class) a whole stream whose choice stopped on `finish_reason: "error"` with no error object
beside it
(the shape OpenRouter's Together and CoreWeave routes delivered seven times since 2026-09-03,
each after reasoning and with no content)
is read as the provider failure it is (`openrouter-error-finish.ts`,
asked by `openrouter-stream-error.ts` when no chunk carried an error object),
rides the retry ladder as `InStreamProviderError` with `code unnamed` and the upstream's own reason as its kind,
and no longer reaches the reply ladder as an empty answer that lost the voice as `schema-mismatch`.
Since `8bf9deec0` (2026-09-08,
the seventeenth class) a reply whose JSON opening was written twice,
an abandoned fragment and then the whole object
(`{"best": 1{"best": 1, ...}`,
the shape reasoning streams from Bedrock's `openai.gpt-oss-120b` and OpenRouter's Makora route deliver),
is read past the fragment (`json-false-start.ts`,
each brace inside the first 256 characters tried as the start until one parses),
the caller's guard still judges what was read,
and the log carries `json false start: read the object past an abandoned opening of N chars`
where before the voice was lost as `schema-mismatch`.
Since `20e5135a6`,
merged as `d11f36799` (2026-09-08,
the owner's rule "publish the archive's front matter as is;
render it only where the archive never translated it",
addendum in the same decision) the archive's front matter stands unless the archive shows the directory id as the
visible name while the source names the person and no clause of 2026-09-07 makes the id stand:
where it stands the preparation makes no metadata slice (`frontMatterAuthority: 'archive'`,
logged as `FRONT MATTER entry=<id> authority=archive`),
the page carries the archive's bytes,
and the final guard refuses any other front matter or any rendered slice as `archive-front-matter`;
artifact generation eleven records the authority in its preparation and the rebuild reads it off the file.
Measured over the pinned corpus,
all 92 archives stand.
Since `169a86173` (2026-09-08) `google.gemma-4-e2b` also writes,
as translator (eight,
quorum 4) and consolidation writer (ten),
seated by the 40-round producer calibration of the same day
(30 of 298 disinterested ballots,
z -1.40 against the pooled null of 12.8 percent,
94 of 94 asks usable;
the seating decision's addendum of 2026-09-08).
Since `6bfe6da56` (2026-09-04,
two owner decisions after the luxuanwen3 pass lost a full run to its front matter) the identity rule reads containment:
where the source declares `name` and `info.alias` the same,
the translated name must appear among the comma-separated renderings of the translated alias,
which may carry the original script beside it (7 of the 14 such archives at the pinned corpus do),
and consolidation withholds a standing text the deterministic gate refused from its slate,
failing the slice under `ConsolidationStandingIneligibleError` when nothing valid ships
rather than letting the page guard refuse the entry after the run has been paid for
(`doc/decision/translation-repair-ineligible-standing.md`),
until the owner ruled on 2026-09-27 that such a slice keeps the archive's wording
and the page ships with it reported (that record's eighteenth addendum).
Since `82888d43b` (2026-09-04,
owner's decision after the re-run stopped at its first paragraph)
a reference the archive rendered another way is owed once,
not twice:
per atom kind,
the atoms only the original carries and the atoms only the page carries form one pool
and a candidate owes the larger side's count from it,
drawn from either side (luxuanwen3 links `x.com` where its original links `twitter.com`;
8 of the 93 entries carry such a rewrite),
while a one-way divergence stays an addition or a drop owed as before
(`doc/decision/translation-repair-rewritten-destination.md`).
The run log names which verdict refused a standing (`fails the deterministic publication rule`,
with the findings,
or `lacks contest endorsement`) and a contest winner's findings (`558b46e11`),
and an ineligible standing stops the entry instead of queueing a reattempt of the same refusal (`ae1d2b55f`).
Whether the lanes kept the archive's metadata is not the guard's question:
Chinese and English metadata always differ,
so the 2026-08-28 byte comparison fired on every kept incumbent and discarded the Carena0442 pass,
and the night's reading of which panel chose the keep (`daaf0ffa0`,
`6f70a2085`,
`1160ebb4c`) was removed with it.
