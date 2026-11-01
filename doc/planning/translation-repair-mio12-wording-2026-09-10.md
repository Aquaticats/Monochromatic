# Mio12 wording follow-up

## Scope

The owner delegated the best preventive changes after rejecting guard-first treatment of verse flattening.
Mio12 now has published-page evidence for the verse remedy.
Its separate stored-review quorum failure is fixed through `b5da9866b`.
This follow-up distinguishes fidelity and policy failures from optional wording preferences.
No corpus edit,
new correction loop,
roster change or broader publication rejection is proposed.

Evidence files:

- `~/temp/agent/Mio12-wording-trace-20260910.out`.
- `~/temp/agent/Mio12-wording-summary-20260910.out`.
- `~/temp/agent/Mio12-archive-review-trace-20260910.out`.
- `~/temp/agent/Mio12-20260910/artifacts/Mio.json`.
- `~/temp/agent/Mio12-20260910/fixed/people/Mio/page.en.md`.

## Names as references versus names as the subject of a quotation

Source slice zero distinguishes the character `澪` from the chosen name `Mio`.
The original archive retains that distinction.
The translation lane replaces both with `Mio`,
so the question and denial can read as denying her own name.
The repair lane retains the character but adds a pronunciation gloss.
That gloss is unnecessary to preserve the contrast.

The initial translation slate already contained the archive's distinguishing wording:
GLM-flash and Mercury both reproduced it.
Qwen3.8 supplied the name-normalized version that won 3.5 weight against two.
Its self-vote explicitly justified using declared `Mio` for `澪`.
The remaining candidate copied the Chinese source after an author repair;
judges recognized that it was untranslated.
A quoted-form exception must not license copying the whole source-language quotation.

The contest chose translation:
three ballots for translation,
one for repair and two for neither.
The repair ballot explicitly identified the name-denial problem;
other ballots objected to the added reading gloss.
Consolidation endorsed the standing translation.
The final paragraph refiner did not touch the blockquote,
and the later naturalness findings were recorded without generating new work.

The exact identity context declares only Mio's name,
alias,
location and she pronoun.
It supplies no alternative interpretation of the character in the question.

The writer and editor rules do not distinguish referring to a person from discussing a word's spelling or form.
`translate-wire.ts` enforces declared and archived name spelling.
`edit-prompt.ts` additionally requires translating source-language quotations fully,
while preserving genuinely foreign phrases with a gloss.
Those general rules need an explicit scope boundary:
name authority does not normalize the different forms that a passage is comparing.
Preserving a quoted character as the subject of the sentence does not leave ordinary prose untranslated.
No speculative pronunciation or etymology is needed.

The trailing `aba` has a related issue:
it transcribes a source-language filler instead of communicating its conversational tone.
Preserve the tone without requiring a literal transliteration or inventing an apology.
The intended change needs a bounded writer and selection probe before a full pass.

## Archive names in the repair lane

Source slice four names the QQ group in mixed script.
The archived English calls it `Harunome Hanbai`.
The repair replaces that with Chinese,
a romanized name and an English gloss.
The translation alternative drops the archive's list,
and the repair wins.
The accepted terminology issue explicitly demanded preserving and glossing the Chinese group name instead of
using the archived English name;
its adjudication had four supporting and two opposing ballots.
The issue therefore enters before editing,
not only in the final writer.
`~/temp/agent/Mio12-name-origin-20260910.out` records the issue and initial translation slate.

The initial translator has an explicit archived-name authority rule;
the editor's rule list does not share it and instead emphasizes foreign-phrase preservation plus glossing.
Consolidating the authority rule across the writing and judging roles is preferable to a list of special group names.
This must preserve declared-name precedence and contributor identity rules.
It must also retain the names-as-mentioned distinction described in this document.

## Temporal and participant ambiguity

Slice four retains the archived bullet about coming out to a best friend "in primary school".
The surrounding source is about the last year of her life.
The following source section describes an April 2022 reconnection and the friend coming out to Mio.
The page's later paragraph preserves that event direction.

The bullet therefore warrants source-context checking,
not an automatic token replacement:
both the modifier's attachment and who disclosed to whom matter.
A general remedy must preserve event participants and temporal scope,
not merely replace `in` with `from` or assert an unsupported reciprocal disclosure.
The actual `neighbouringSource` call for slice four has been checked.
It supplies slice three's childhood SRS paragraph and slice five's heading,
not slice six's April 2022 reconnection and disclosure paragraph.
The deciding event context is outside that physical-slice window.
See `~/temp/agent/Mio12-temporal-context-20260910.out`.
This supports testing a meaningful source-context boundary before adding stronger temporal instructions;
it does not establish how the models will respond to that change.

## Translation-side apparatus and faithful but defective English

`Translation:` is recognized by `isVerifiableEditorialArchiveBlock`:
the actual invocation returns true.
Nevertheless,
archive revision selection deleted the first chat's label.
A recorded selector reason says it is translation-side apparatus rather than factual prose,
"therefore should be removed".
That contradicts the existing criterion to retain verifiable editorial apparatus.

`archive-block-review-stage.ts` calls the selection task an "unsupported archive-only block"
before the selection has established that it is unsupported.
The task should describe an unclaimed block neutrally.
No new deterministic retention whitelist is needed;
the label is already recognized.

The first chat is retained verbatim,
including `musculine` and other usage issues.
The source image `people/Mio/photos/photo7.webp` was inspected directly:
it is Chinese chat with a Chinese embedded post,
not an English-original passage whose spelling must be sealed.

The review classifications overlap:
`source-supported` means the block states source content,
while `revise` includes defective prose.
A faithful translation with a typo fits both descriptions.
If reviewers choose retention,
`recordArchiveBlockNaturalness` records defects but cannot change the text.
That audit-only behavior is deliberate under the no-loop design.

That initial proposed explanation is superseded for Mio12 by the cache-only replay recorded in this document.
The correct typo revision was already produced;
stronger generation instructions or another copy-edit round are not the primary remedy.
Do not add a rejection-triggered copy-edit loop or loosen source support.

## Optional wording preferences

The university phrase `a local 985 (a top-tier university)` is awkward,
but its meaning is explained.
The source acronym `SRS` gained an expansion in initial translation and consolidation.
These deserve terminology and readability checking,
but they do not independently justify another whole-entry pass solely for stylistic preference.
No unsupported terminology assertion or new glossary entry has been implemented.

## Next work

- Build a bounded prototype for shared name authority,
  quoted-form preservation and conversational filler handling.
- Prototype a source-context boundary that reaches the dated reconnection paragraph,
  then measure selection with unchanged candidates and instructions.
- Probe neutral archive-review framing and non-overlapping revision criteria in the existing graph.
- Keep passing verse and quorum behavior intact.
- Resume the entry queue only after the material findings have an evidenced remedy and the required page reading.

A focused advisor call about prioritization timed out without feedback.
It supplies no endorsement or objection.
## Completed name-form measurement

`proc_b0ce` completed `translation-repair-name-form-probe-20260910` on frozen `b5da9866b` in 139 seconds.
The script is `~/temp/agent/probe-name-form-20260910.mjs`;
log `~/temp/agent/name-form-probe-20260910.log`;
report `~/temp/agent/name-form-probe-20260910/report.json`.
The offline plan was exercised successfully.

The experiment keeps the recorded anonymous slate and all non-system messages fixed.
It asks the six original responding judges directly in both arms,
not through different prompt-rotated subsets,
and then asks Qwen3.8 and Mercury to write under both arms.
There are at most 16 model requests,
six concurrent requests,
a 360000 ms exchange bound and a 1200000 ms global bound.
Normal production completion caps remain unchanged.
Mio12's completed-prompt cache is copied into the disposable probe directory for reuse where identities match.
The original cache and corpus are untouched.

The treatment clarifies reference-name authority versus quoted spelling or character forms,
and distinguishes conversational tone from literal filler transliteration.
No production prompt has changed yet.
Read the actual judgments and writing,
including the untranslated-copy negative control,
before deciding whether to integrate the wording.
This experiment does not yet measure the group-name repair path or the separate source-context change.

Both treatment writers preserve `澪` and `Mio`.
Qwen's baseline conflated them;
its treatment retains the character and uses `uhh` instead of `aba`.
Mercury retains the archive wording in both arms.
All writing outputs compile.
Qwen's treatment changes quote typography,
so this is not a claim that every output detail is publication-ready.

Judging improved only partially:
two of six baseline judgments select the distinguishing archive wording,
versus three of six under the treatment.
DeepSeek-flash changes to that candidate and explicitly recognizes the erased contrast.
Gemma-e2b,
Mercury and Qwen still prefer the normalized version.
Qwen invokes the new filler warning to object to the archive's apology while still treating its character as
unnecessary mixed-script text.
No judge selects the untranslated-copy negative control.
These are per-seat results on a fixed slate,
not the production window's tally.

The eight baseline requests reused completed Mio12 payloads.
Eight new calls are logged:
two Bedrock calls costing 0.00059064 USD,
two OpenRouter calls reporting zero cost,
and four Hyper calls without per-call prices.
The daily helper ran after the probe.
Evidence is `~/temp/agent/name-form-probe-spend-20260910.out` and
`~/temp/agent/name-form-costs-after-20260910.out`.

The treatment is not ready to be copied verbatim into production.
Its appended guidance leaves the existing unqualified declared-name criterion in place,
and its filler warning can distract from the core contrast.
Next test the form exception inside the actual faithfulness and name-authority criteria,
without bundling it with a filler preference.
Preserve the existing official-English-work-title and contributor precedence rules when sharing name authority.
The group-name editor path and source-context treatment are still unmeasured.

## Completed inline-criterion measurement

`proc_75c1` completed `translation-repair-inline-name-scope-probe-20260910` on frozen `b5da9866b` in 40 seconds.
The offline plan passed.
The script is `~/temp/agent/probe-inline-name-scope-20260910.mjs`,
log `~/temp/agent/inline-name-scope-probe-20260910.log`,
and report `~/temp/agent/inline-name-scope-probe-20260910/report.json`.

Only the existing faithfulness exemption and name-authority criterion change.
The treatment distinguishes references to entities from mentions of language forms inside those rules,
instead of leaving their unqualified versions next to an appended exception.
It contains no new filler warning,
new candidate,
source annotation,
producer identity or changed candidate order.
Reversing the criterion replacements restores every original message byte.
The experiment makes six direct judge requests,
with a 360000 ms exchange deadline and a 720000 ms global deadline.
It reuses the recorded baseline rather than purchasing it again.

Read the choices and reasons before selecting an integration.
The original slate's English candidates also differ in framing and filler treatment,
so a split result does not alone prove inability to recognize a pure spelling contrast.
No requirement that every model agree is being introduced.

Four of six judgments now select the distinguishing archive wording,
versus two on the recorded baseline and three with the appended rule.
Mercury and DeepSeek-flash explicitly identify the erased contrast;
Gemma-e2b changes to the distinguishing candidate as well.
Qwen still prefers the normalized name.
DeepSeek-pro also picks it,
but its explanation incorrectly claims that this candidate contains `澪`.
The candidate data and numbering were verified unchanged,
so that explanation is not treated as evidence that it preserves the character.
No judge selects the untranslated copy.

The inline exception is the selected judging remedy,
without a requirement for unanimous agreement.
It changes the active criteria rather than leaving an unqualified name rule next to a competing appendix.
The writer-side evidence from the first experiment supports carrying the same reference-versus-mention distinction
into the existing name rule.
Do not carry its filler warning or broaden archived-name authority over the official-English-title rule.

Logged spend:
two Bedrock calls totaling 0.00056812 USD,
one OpenRouter call reporting zero cost,
two unpriced Hyper calls and one unpriced Synthetic call.
The daily helper ran after the measurement.
`~/temp/agent/inline-name-scope-spend-20260910.out` records the provider totals.

Guard `9e727def9` now specifies the shared scope in initial writing,
repair editing,
consolidation,
criticism and lane selection,
plus the actual translation criteria.
The group-name policy and temporal-context measurements remain separate pending work.
## Integration and group-name measurement

`514db9b1f` integrates the measured quoted-form scope in initial translation,
repair editing,
consolidation,
criticism,
shared contest policy and the actual translation criteria.
`9e727def9` failed six integration cases before that change.
`fd293e6d3` updates older literal-wording assertions,
and `b55a42a0a` explicitly names the declared-identity exemption so an `it` or `they` cannot be read as exempting
mentioned word forms from coverage.
Dedicated metadata criteria remain unchanged.
No filler warning or output-rejection rule was added.

The group-name measurement completed in 103 seconds.
It isolates the actual Mio terminology claim rather than replaying the original multi-claim packet.
Baseline judgments support that claim five to one;
the proper-name scope produces three supporting and three opposing judgments.
Replaying those actual votes through `tallyVotes` changes the claim from `accepted` to `needs-human`,
so it no longer authorizes an editor correction under the existing strict-majority rule.
This is not a claim that all judges now reject it.
The explicit-definition control receives five supporting baseline votes and six under treatment.
Ordinary definitions are not excused by the name exception.

Evidence:
`~/temp/agent/archive-name-gloss-probe-20260910/report.json`,
`~/temp/agent/archive-name-gloss-replay-20260910.out`
and `~/temp/agent/archive-name-gloss-spend-20260910.out`.
The measured source scope is identical under the prior frozen and current adjudication builders.
The probe logged 0.00219198 USD on Bedrock,
zero reported OpenRouter cost,
eight unpriced Hyper calls and four unpriced Synthetic calls.
The daily helper ran afterward.

`272db0e92` fails without the scoped first-use rule;
`8a59469f2` integrates it in the existing vocabulary policy.
It retains the owner's [inline-gloss rule](../decision/translation-repair-inline-gloss.md)
and adds no curated glossary entry.
The corpus writing principles and the owner's stronger translation rule were both read before editing the policy.
`9578ad0bc` supplies the composed export's required type annotation.
`85f7bac01` makes fidelity-fixture assertions identify judges instead of relying on prompt-rotated arrival order.

Build,
types,
oxlint and the full suite pass through `85f7bac01`.
The suite ends `unit exit 0` in `~/temp/agent/naming-final-unit-20260910.out`.

The current-build integration probe completed as `proc_87ab`
(`translation-repair-naming-integration-20260910`)
on frozen `85f7bac01` in 49 seconds.
It runs the real `runTranslateStage` over the opening with Qwen3.8 and Mercury writing and the six original
responding judges seated.
It allows at most 20 stage requests,
a 360000 ms exchange bound and a 720000 ms global bound.
Its report is `~/temp/agent/naming-integration-probe-20260910/report.json`
and log `~/temp/agent/naming-integration-probe-20260910.log`.
It used six stage requests:
two writer calls and four judge calls.
Qwen produced a fresh wording retaining the elliptical `澪?` and the separate name `Mio`,
without the archive's added question-about-meaning framing.
Mercury reproduced the archive.
All four responding judges selected Qwen's fresh rendering,
with the ordinary self-vote weight retained.
There was no structural repair,
follow-up generation or outage fallback.
The text compiles and the canonical source hash is unchanged.
This verifies the combined current-build instructions through the producing and selecting stage,
not only their presence in a prompt.

The integration logged two Bedrock calls costing 0.00056918 USD,
two OpenRouter calls reporting zero cost,
and two unpriced Synthetic calls.
The post-probe daily helper ran.
`~/temp/agent/naming-integration-spend-20260910.out` records the totals.

The name-policy work is verified locally and at the bounded live stage.
The remaining material follow-ups are archive apparatus/copy-edit treatment and temporal source context.
## Cache-only archive replay corrects the remaining diagnosis

`recover-Mio12-archive-context-20260910.mjs` reconstructs the actual two-picture support using twelve completed
payloads and no provider client.
Its first draft used receipt order for readers;
the overlap mismatch exposed that mistake.
The corrected replay derives the seated reader order from the frozen code and recorded dryness,
and reproduces the logged overlap values for both pictures.
It uses the real `archiveBlockSourceContexts` formatter and original archive block offsets.

`replay-Mio12-archive-review-20260910.mjs` then replays the complete old stage from completed prompt payloads.
The reconstructed selector messages match the recorded Mio12 messages byte for byte.
There are 21 cached outcomes and two expected cache misses for unavailable calls;
missing payloads throw locally rather than contacting a provider.

The label path is exact:
six reviewers classify `Translation:` as valid editorial context,
while GPT-OSS proposes removal.
All six deciding judges choose the sole removal candidate.
Unlike the initial reviewers,
those selectors do not receive the English archive showing the transcript that the label introduces.
They see the label,
source context and review findings,
and describe it as having no rendered content or purpose.
The missing archive context is therefore another decision-input gap,
not merely an unhelpful task title.

The retained-chat path is different from the initial hypothesis.
Seven schema-valid reviews meet the eleven-seat participation quorum.
Kimi already supplies a complete correction of `musculine` and `Wechat`;
Gemma supplies a bad eighty-character partial rewrite.
Three `source-supported` responses fail exact anchoring,
leaving four eligible replies.
The retention-anchor threshold returns the original before either actual revision reaches selection.
The all-retention naturalness-audit branch is not the branch this block took.

The revised remedy keeps the review and independent-selector quorums unchanged,
continues excluding unanchored retention proof,
and allows actual revision candidates to reach that independent selector.
Retention-only resolution still uses the existing anchor threshold and unresolved fallback.
The selector must be able to compare revisions with the unchanged block,
under a neutral task and with the already-available English archive as context,
not as factual source authority.
A focused independent review supports those boundaries.

Evidence:
`~/temp/agent/Mio12-archive-context-20260910.json`,
`~/temp/agent/Mio12-archive-context-replay-20260910.out`,
`~/temp/agent/Mio12-archive-review-replay-20260910.json`
and `~/temp/agent/Mio12-archive-decisions-20260910.out`.

## Completed archive decision measurement

`proc_5ae5` completed `translation-repair-archive-decisions-probe-20260910` in 44 seconds.
Its script is `~/temp/agent/probe-archive-decisions-20260910.mjs`;
log `~/temp/agent/archive-decisions-probe-20260910.log`;
report `~/temp/agent/archive-decisions-probe-20260910/report.json`.

The offline plan passed and the label baseline matches the actual cached selector messages exactly.
The experiment compares that baseline with archive-context-only input,
then the combined neutral task,
archive context and unchanged-block option.
It also sends the two already-produced chat revisions and original block to the independent judges.
No reviewer or writer is re-asked to generate a correction.
There are at most 24 model requests,
six concurrent requests,
a 360000 ms exchange bound and a 1200000 ms global bound.
The baseline reuses completed payloads.

The existing chat correction wins five of six judgments against the bad partial rewrite and unchanged original.
The remaining judge chooses the original while mistakenly describing it as the corrected candidate;
that reason is not treated as evidence of a correction.
This supports exposing the already-produced revision to independent selection.

The label results do not establish a remedy.
All six baseline judges remove it,
and all six still remove it with archive context added.
The combined context,
neutral task and original option retains it on only one of six judgments.
Context restoration and an unchanged-block option are necessary decision information,
but this measurement does not show them sufficient to fix the label outcome.

The next evidence boundary is visible in the code:
selection receives bare `finding` strings,
not the review `disposition` that explains them.
Six reasons for retaining editorial context therefore arrive under `LATEST REVIEW FINDINGS`
as if they were defect reports supporting removal.
Selectors repeatedly describe those findings as agreeing on removal,
although their actual decisions were six retentions and one revision.
The proposal-to-reason association is also absent for competing corrections.

The experiment logged 0.00464245 USD on Bedrock,
three OpenRouter calls reporting zero cost,
and nine unpriced Hyper calls.
The baseline was reused from completed payloads.
The daily helper ran afterward.

## Completed structured-assessment measurement

`proc_0425` completed `translation-repair-archive-assessments-probe-20260910` in 127 seconds.
The script is `~/temp/agent/probe-archive-assessments-20260910.mjs`;
log `~/temp/agent/archive-assessments-probe-20260910.log`;
report `~/temp/agent/archive-assessments-probe-20260910/report.json`.
The offline plan passed.

The same neutral comparison,
source,
archive context and candidate values now carry eligible review assessments as structured data:
disposition,
proposed retain/revise action,
candidate number and finding.
The evidence explicitly remains opinion to check against the documents,
not authority or votes deciding the answer.
Unanchored retention claims are not promoted into this evidence.
The deciding criteria are unchanged.

Actual label and chat cases are accompanied by invented orphan-label and contradicted-biography controls.
Those controls deliberately carry a misleading retention majority:
a selector must still inspect context and source instead of rubber-stamping earlier opinions.
There are at most 24 requests,
six concurrent requests,
a 360000 ms exchange bound and a 1200000 ms global bound.
No writer is asked to generate a replacement.

All six judges now retain the actual useful label.
Five of six choose the already-produced full typo correction;
the remaining judge chooses the uncorrected original.
All six remove the directly contradicted award claim despite the deliberately misleading retention majority.
The orphan-label control is not a success:
four retain it,
and one of the two numerical removal votes has a reason arguing for retention.
The result does not establish perfect placement reasoning or a blanket rule for orphan labels.
No automatic keep decision or source-proof relaxation is inferred from it.

The measured correction is to preserve the review decision's meaning in the handoff,
not to add another writing round.
`4170c5d2f` fails four boundary cases before implementation,
while its retention-only fallback and initial-quorum controls pass.
`bfcb5b568` routes admissible revision candidates to the existing independent selector after review quorum,
retains the anchor gate for retention-only resolution,
adds the unchanged block as an explicit option,
and supplies archive context plus anonymous typed assessments linked to their candidate.
Unanchored retention claims stay out of selector evidence but remain in audit findings.
Both stage quorums remain unchanged.
Selecting the exact original returns `retained`,
not a fictitious revision.

The implementation is verified through `6c24c82c1`:
build,
types,
oxlint and the full suite pass.
`~/temp/agent/archive-selection-final-unit-20260910.out` ends `unit exit 0`.
Coverage includes seven heard reviews over an eleven-seat bench,
insufficient retention anchors,
sole-revision selector failure,
original selection,
original echo deduplication,
prior findings,
retention-only fallback and initial-review failure.
Older fixture routing now identifies the actual archive evidence boundary rather than an outdated task sentence.

The typed-assessment experiment logged 0.00427526 USD on Bedrock,
four OpenRouter calls reporting zero cost,
and twelve unpriced Hyper calls.
The daily helper ran after the probe.

## Current-build archive integration exposes a remaining generation gap

`proc_3f3e` completed `translation-repair-archive-integration-20260910` on frozen `6c24c82c1` in 311 seconds.
The offline plan passed.
It invokes the real public archive-review stage for the actual label and chat,
using the recovered corroborated source support and original eleven-seat roster.
Both fresh review and independent selection are exercised,
with at most 48 forwarded requests,
a 360000 ms exchange bound and a 1200000 ms global bound.
The report is `~/temp/agent/archive-integration-probe-20260910/report.json`
and log `~/temp/agent/archive-integration-probe-20260910.log`.
This was not a successful end-to-end correction.
The label stayed through the unresolved-retention fallback:
five editorial assessments survived,
while two reviewers incorrectly treated the label as source-supported prose and failed anchoring.
No label selector ran in that case.

The chat reached independent selection,
so the routing fix was exercised.
But the fresh review produced only Gemma's broader revision,
not Kimi's earlier minimal typo correction.
That revision fixes the spellings while introducing two untranslated `唔` utterances and replacing readable
English terms with bare `mtf` and `GD`.
The selector split three to three and kept the original,
including its misspellings.
The program did not silently ship the flawed revision,
but the desired corrected English was not produced.

A cache-only replay confirms the actual proposer and values in
`~/temp/agent/archive-integration-replay-20260910.json` and
`~/temp/agent/archive-integration-review-evidence-20260910.out`.
`photo6.webp` was inspected directly as well as `photo7.webp`;
reader inferences about filler placement are not substituted for the image evidence.

The remaining producer-brief issue is now directly observed:
`source-supported` does not say no necessary English correction remains,
and `revise` does not clearly require a minimal complete English replacement rather than source-language copying.
This does not invalidate the earlier routing diagnosis:
the old good correction was blocked,
while this new cohort produced no equally good correction to select.

The integration logged 0.00465705 USD on Bedrock,
two OpenRouter calls reporting zero cost,
thirteen unpriced Hyper calls and four unpriced Synthetic calls.

## Completed initial-review brief measurement

`proc_4c65` completed `translation-repair-archive-review-brief-probe-20260910`
on frozen `6c24c82c1` in 547 seconds.
The script is `~/temp/agent/probe-archive-review-brief-20260910.mjs`,
log `~/temp/agent/archive-review-brief-probe-20260910.log`,
and report `~/temp/agent/archive-review-brief-probe-20260910/report.json`.
The offline plan passed through the public stage;
the private message builder is not a barrel export and is not called as one.

Only the existing first-review system brief changes.
The treatment separates useful editorial apparatus,
necessary revisions and already-adequate source-supported English;
requires complete minimal English corrections;
rejects mere literalness as a correction reason;
and treats parallel image readings as witnesses rather than additional utterances.
It does not alter source or archive data,
selector instructions,
quorums,
rosters or the fixed-depth graph.
It forwards at most 48 requests,
with the same 360000 ms exchange and 1200000 ms global bounds.
No new correction round or Chinese-character rejection rule is introduced.

The treatment used 39 requests.
It retained the label and selected a complete English chat correction,
with `musculine` and `Wechat` corrected and no untranslated `唔` introduced.
The selected rendering also uses a single dash and renders the source-supported trait as "sensitive yet determined".
It preserves the archive's message structure and readable terminology.
The selector chose that candidate at weight three over alternatives at weight two and one;
this was a real selected revision,
not a fallback.

The result is not unanimous proof of every dialogue interpretation.
Some judges prefer a more literal variant that removes disputed dialogue lines;
others consider the archive's contextual renderings defensible.
That disagreement is recorded rather than converted into an automatic deletion rule.

The probe also exposes a separate audit input gap:
both retained-label naturalness rounds evaluate `Translation:` without its neighboring archive content,
and report it as an empty translation.
Those fourteen findings are audit-only,
not a reason the label was removed or a successful quality assessment.
Task 20 tracks the actual context/role boundary for those audits,
without skipping required review or adding a loop.

`ede699627` fails before the brief change.
`ba01babda` integrates the exact measured first-review wording.
Build,
types,
oxlint and the full unit suite pass;
`~/temp/agent/archive-brief-unit-20260910.out` ends `unit exit 0`.
The probe logged 0.00518614 USD on Bedrock,
two OpenRouter calls reporting zero cost,
eighteen unpriced Hyper calls and eleven unpriced Synthetic calls.
The daily helper ran afterward.

## Completed compiled-brief integration

`proc_4eec` completed `translation-repair-archive-brief-integrated-20260910` in 783 seconds on frozen `ba01babda`,
without a message-intercepting treatment wrapper.
It seeds its disposable cache from the completed brief probe,
so identical requests reuse completed results.
This also exercises the prompt-dependent review window with the wording now in the actual builder.
Its report is `~/temp/agent/archive-brief-integrated-20260910/report.json`
and log `~/temp/agent/archive-brief-integrated-20260910.log`.
The public stage retains the same eleven-seat roster and request/deadline bounds.
It made 41 forwarded requests,
retained the label through the normal evidence-backed branch,
and selected the same corrected English chat.
Five revisions competed with the unchanged original;
the chosen Kimi revision won weight 3.5 over seven ballots,
with ordinary author self-vote weighting retained.
Both outputs compile.
This validates the actual compiled review/selection path,
not only the message-intercepted experiment.

The integration logged 0.00046112 USD on Bedrock,
two OpenRouter calls reporting zero cost,
three unpriced Hyper calls and seven unpriced Synthetic calls.
The daily helper ran afterward.
The log records 26 disk-payload reuses;
the spend records do not account for every forwarded request,
so unreported failed or cancelled usage is not claimed free.
No existing cache was modified.
Task 18 is complete at this boundary.
Task 19's temporal context remains before the next full Mio pass.
Task 20's audit accuracy is deferred and unresolved,
not a readiness blocker:
the caller trace and controlled public-wrapper check show detailed audit findings neither change text
nor reach the entry's persisted findings.
The calls remain an operational cost and cancellation/error path.
See [archive audit context measurement](translation-repair-archive-audit-context-2026-09-10.md).
No full-entry pass is active.
