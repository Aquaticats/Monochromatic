# Audit ledger: rule text, the consolidation gate and the floor replay

Contradictions and gaps in the rule text every sheet carries,
the consolidation gate finding,
and the replay of every floor over the archives and settled pages.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Consolidation gate

### G1: the gate sheet says choosing an ineligible standing stops the entry

Status:
fixed in `0b8788dae`;
decision addenda thirteen to fifteen record the owner's answers.
Since class one hundred eighty-five (`419605ff4`) a gate preferring an ineligible standing ships the slate's choice,
but `buildConsolidateGateMessages` still says 'Choosing "standing" stops this entry with no page',
and the `standingEligible` TSDoc in `src/consolidate-settle-gate.ts` (moved on 2026-10-18 to
`package/translation-repair/consolidate/src/consolidate-settle-gate.ts`) says a gate keeping it ends the slice.
Over an ineligible standing every gate verdict now ships the consolidation;
whether the gate should still run there is an owner question.

## Rule text and sheet consistency

Probes:
`~/temp/agent/audit-policy-text/` and `~/temp/agent/audit-sheets/`
(rendered sheets under `sheets/` and `rendered/`,
a sheet-by-block matrix in `report.txt`).
Paths are under `src/`.

### S1: the translate writer keeps the existing translation's tense, against the house tense rule on the same sheet

Status:
fixed in `00eed316e` (guard `b16350d39`),
with the translate writer's no-addition rule.
`translate-wire.ts` says "KEEP THE TENSE OF THE EXISTING TRANSLATION where one is shown";
`house-policy.ts` says a life told in the present is brought to the past.
Class eighty-five fixed the same wording on the consolidation writer only,
and `tense-authority-reaches-every-sheet.unit.test.ts` pins the wrong wording.
Fix:
the consolidation writer's ordered tense rule.

### S2: production translate writers never see ARCHIVE RENDERING DISPUTED

Status:
fixed in `9858a7acb` (guard `translate-produce-dispute.unit.test.ts`).
`translate-stage-repair.ts` spreads `archiveDisputeNote` into `produceTranslateSlate`,
which never declares or forwards it;
a capturing client shows 0 of 2 translator sheets carry the note.
Tests exercise only the builder.
Fix:
declare and forward it;
guard through `produceTranslateSlate`.

### S3: the typed decision seat (Jev) votes without the house rules or community renderings

Status:
fixed in `ce824d933` (guard `decision-seat-policy.unit.test.ts`).
`selectDecision` in `candidate-select-decision.ts` builds criteria,
evidence and candidates only;
the chat seat carries `JUDGE_POLICY_BLOCK` and `communityRenderingsBlock`.
Comments in `stage-decision-call.ts` and `polish-gate-house-rules.ts` claim otherwise.
Commit `bc6fdc9b9` kept the sheet from the seat on purpose;
the fix carries the policy in the decision state (about 4k tokens against a 32k context).

### S4: the adjudication panel has no identity context

Status:
fixed;
the panel sheet carries the fenced DECLARED NAMES block and the shared declared-identity rules
(`declared-identity-rule.ts`,
moved out of the critic),
threaded from `repairChunk` through `runPanelStage`;
guarded end to end in `repair-translation.unit.test.ts`,
shown red with the thread cut.
`buildAdjudicationMessages` takes no `identityContext`,
so a claim against a declared name can be supported and dispute the archive.

### S5: the apparatus list differs across six sheets

Status:
fixed (`e7e3f9c17`,
`2329042f4`,
`914ccb21d`).
`APPARATUS_KINDS` in `page-apparatus-clause.ts` is the one list,
the union of every kind any sheet named;
the critic,
the panel,
the contest,
the writers' clause,
the archive block review,
its selector (`ARCHIVE_BLOCK_SELECTION_CRITERIA`)
and the dispute note read it,
and the critic,
the panel and the selector gained `NARRATIVE_DETAIL_IS_NOT_APPARATUS`.
Guards:
`apparatus-kinds.unit.test.ts`,
the selection-prompt assertion in `archive-block-review-stage.unit.test.ts`.
Left as measured:
the block review's deterministic `editorial-context` check still counts only fixed-prefix,
contributor,
image and comment blocks,
so a reviewer calling a gloss block `editorial-context` is not counted;
the block then ships as the archive wrote it unless a revise vote wins selection,
and the selector now reads the apparatus kinds.
The unclaimed-block census found no footnote or gloss block in the corpus.
Original finding:
`PAGE_APPARATUS_IS_KEPT`,
`CONTEST_POLICY`,
the critic,
the panel,
archive block review
and its selector each list different kinds;
critic and panel keep "only when WRONG".
Fix:
one shared kinds constant with the narrative bound on every sheet that files,
votes on or writes against the archive.

### S6: `CONTEST_POLICY` still says keep what the Chinese is silent about

Status:
fixed (`e7e3f9c17`,
`fa892a7da`).
The silent-keeps sentence covers page apparatus and whole regions
and says a narrative detail the archive adds to a passage is not kept on the archive's word;
the judge tail's precedence sentence names "a criterion or any other rule you have been given".
Guards:
`dropped-covers-the-page.unit.test.ts`,
`contest-ballot-wire.unit.test.ts`,
`decision-seat-policy.unit.test.ts`.
Original finding:
"Where the Chinese is SILENT rather than contradicting,
dropping it is a fault of this kind,
and keeping it is correct" sits beside `NARRATIVE_DETAIL_IS_NOT_APPARATUS`;
the CuspariaKLSY11 gate ballot kept "took medication that night" on that wording.
The JUDGE tail's precedence line names criteria this sheet lacks.

### S7: `SIZE_NOTE_POLICY` calls silent surplus in any far-longer rendering page content

Status:
fixed (`d94f6787b`).
Silent surplus is page content only where the archive rendering also carries it;
surplus the archive does not carry,
a passage repeated or looping included,
is unsupported.
Guard:
`contest-size-note.unit.test.ts`.
Original finding:
The note exists for a looping candidate,
and its sentence tells the judge the loop's surplus is page content.

### S8: the narrative bound and the cited-reference rule have no precedence

Status:
fixed (`914ccb21d`).
Both reference rules say an event or a characterization a cited reference states is covered,
since the narrative bound is about detail no source states,
and that reader protection outranks the references;
`TRANSLATE_ATTESTED_RULE` says reader protection outranks the attested details;
the dispute note says the adjudicators weighed the references before accepting the claim.
Guard:
`reference-precedence.unit.test.ts`.
Original finding:
A characterization a cited reference states is ACCURATE on one rule and an addition on the other;
neither limits reader protection
(`cited-reference-rule.ts`,
`TRANSLATE_ATTESTED_RULE`,
`misreadingRule`).

### S9: reader-protection bullet contradicts itself on "took medication"

Status:
resolved by the owner,
2026-09-27:
"took medication" is not replicable and may stay (`9eba04abb`).
`house-policy.ts` keeps "took medication" vague as a method
and allows "at most that she had taken medication" for any medication tied to a death.
Proposed condition:
the allowance holds only where medication was not the means.

### S10: the refiner keeps "any word left in the original language" and every date unchanged

Status:
fixed.
The block-level precedence sentence landed in `e8f0b0369`;
`81962c75a` rewrites the refiner's survival sentence in both branches to name the house form
and adds to the translate writer that a name the archive left in Han is not a rendering to keep.
Guards:
`house-policy-wording.unit.test.ts`,
`consolidate-objection-polish.unit.test.ts`,
`translate-wire.unit.test.ts`.
Original finding:
Contradicts the house block on Han,
Ta,
titles,
handles,
shorthand
and month-first dates.
Only `edit-prompt.ts` says the house rules outrank the sheet;
seven other sheets carry the block with no precedence sentence.

### S11: the polish gate's preface was read as licensing the base's tense

Status:
fixed (`091307d14`).
The preface calls a present-tense line of the life in the base a tense the house rules correct,
never a reason to keep the base;
`HOUSE_CORRECTION_IS_AN_IMPROVEMENT` in `polish-gate-house-rules.ts` tells the comparative and objection gates
to prefer a polish that only applies a house rule,
and the comparative refiner is told such a paragraph is a rewrite to make.
Guards:
`consolidation-polish-gate.unit.test.ts`,
`house-policy-wording.unit.test.ts`.
Original finding:
"a present-tense line about their life is the base's choice" was quoted by a TianqiChen66616 ballot;
the comparative policy never prefers a polish whose only change is a house correction.

### S12: the polish gate lacks blocks the consolidate gate carries

Status:
fixed (`1a004389a`).
`candidate-judge-rules.ts` holds the contest's declared-names lines (moved out of `CONTEST_POLICY`,
text byte-identical),
an apparatus rule in candidate vocabulary,
and a judge line clause;
the polish gate reads all three,
the community renderings block and the dispute note,
threaded `applyFinalPolish` to `polishConsolidation` to `runConsolidationPolishRound` to the gate subject.
The bilingual clause arrives with the line clause (S15).
Guards:
`polish-gate-page-rules.unit.test.ts`,
the dispute assertion in `consolidation-polish-apply.unit.test.ts`.
Original finding:
Apparatus,
name scope,
community renderings,
dispute note (not forwarded by `consolidation-polish-apply.ts`),
bilingual clause.

### S13: the absolute naturalness review has no house rules yet its findings are required fixes

Status:
fixed (`3e4cf0441`).
The review splices `MEASUREMENT_POLICY_BLOCK` with its own verdict line,
drops "deliberate source-language kinship terms" for "a term the house rules keep in English letters with its gloss",
and counts a departure from a house rule of form as a material defect.
Guard:
`naturalness-review-sheet.unit.test.ts`.
Original finding:
It tells the reviewer to preserve source-language kinship terms,
against the house rendering of 姐姐.

### S14: the editor sheet has no identity context or cited references

Status:
fixed (`a8f5cb490` export prep,
`fa949b78f` criteria,
`f16d5e8bf` evidence).
The editor sheet shows the declared names with `DECLARED_IDENTITY_RULES`,
the cited references with their rule,
and the community renderings the translation lacks;
both editor selections read the declared names and references through `repairSelectionSourceEvidence`.
The envelope criterion's tense now follows the house tense rule,
not the surrounding English;
both editor faithfulness criteria carry the declared-name exemption and `PAGE_APPARATUS_IS_KEPT`;
every refine selection mode reads `HOUSE_FORM_CORRECTION_KEEPS_MEANING`.
`house-form-corrections.ts` holds the one house-form list the polish gate,
refiner,
review and refine selection share.
Guards:
`selection-criteria-house.unit.test.ts`,
`editor-page-evidence.unit.test.ts`.
Original finding:
Its selection judges see community departures the editors were never told of.
The editor and refine selection criteria lag the slate criteria,
and "Fits the surrounding text in register and tense" makes the surrounding English the tense authority.

### S15: the bilingual and line-structure rules are missing on the contest and both gates

Status:
fixed (`1a004389a` for the polish gate,
`070db03dd` for the lane contest and the consolidate gate).
`JUDGE_LINE_STRUCTURE_CLAUSE` carries the bilingual clause;
both subjects take a required `lineStructured`,
and the lane contest key appends a mark when the rule governs.
Guard:
`judge-line-structure.unit.test.ts`.

### S16: the picture scope rule never reached the consolidation writer or the slate

Status:
fixed (`efa502d21`).
The consolidation writer reads `TRANSLATE_PICTURE_SCOPE_RULE` when it shows pictures;
the translate judge,
which also judges the consolidation slate,
reads `JUDGE_PICTURE_SCOPE_RULE` in the pictures label.
Guards:
`consolidate-wire.unit.test.ts`,
`document-pictures-reach-the-wire.unit.test.ts`.

### S17: the consolidation slate is told a decline leaves the passage untranslated where it stops the entry

Status:
fixed with the fifteenth addendum
(`select-decline-consequence.ts`:
a withheld standing tells the judges a declined slate ships by preference).
`candidate-select-wire.ts` via `translate-judge.ts`.
The translate lane told the same falsehood over an archive the floor refuses,
fixed in the same change.

### S18: measurement sheets drift

Status:
fixed.
The rendering audit's tense line and the measurement tail's attempts and places landed in `e8f0b0369`;
`b6df6d5ee` gives the introduced-defect probe's drop rule the apparatus exception.
Guard:
`probe-page-rules.unit.test.ts`.
Original finding:
The rendering audit counts a tense English supplies as `altered-time`;
the introduced-defect probe says dropping wording the ORIGINAL never had is a correct repair,
against the apparatus clause;
the measurement tail's reader-protection line names deaths only,
not attempts or a place that was the means.

### S19: house-policy wording defects

Status:
fixed.
Most bullets were already fixed in the current text (the corner-bracket example,
italic titles,
positional words,
first-person contributors,
"trans girl",
English letters for terms,
Canadian spellings in rule text);
the OD rule now keeps OD unnamed where it is the means of a death or an attempt,
and `KEPT_SUBJECT_RULE` makes I (or that person) the one who does it.

- "OD is overdose" is prescribed where OD is a suicide method (Susiethegamer),
    which reader protection keeps vague.
- The corner-bracket rule contradicts its own 「盐田姐姐」 example.
- Titles:
  the rule says quotation marks,
    the archive and `archive-italic-title-restore.ts` use italics (owner call).
- The positional-word rule reads as a ban on "earlier",
    "below" and "above" in any sense.
- "Entries are written in the third person" against first-person contributors.
- "MtF is trans woman,
  as 药娘 is" omits the owner's "trans girl".
- "kept and glossed" does not forbid Han in English prose;
    runs shipped "大证 (…)".
- Non-Canadian spellings in rule text:
    "capitalised",
    "penalises",
    "characterisation",
    "characterise",
    "humor",
    "judgement".
- `KEPT_SUBJECT_RULE` grammar:
  "says that I (or that person) does it".

### S20: smaller sheet defects

Status:
fixed,
item by item.
The attested-details rule licenses only the attested details and the apparatus from the existing translation;
the critic and editor foreign-phrase rules carry one shared sentence taking names and titles out of their scope,
and the Han residue floor passes a Japanese phrase with its English gloss after it (would-ship refusals 68 to 66);
the restoration judge is told references come from archives written before the house rules;
the dispute note names non-translation and is fenced on the contest and gate;
the identity block is labelled DECLARED NAMES and fenced on every sheet,
and the house policy names it so.
The translate writer's "does not already carry" reopens archive narrative;
the critic's foreign-phrase rule conflicts with the title rule;
the restoration judge states a false reason;
the dispute wording omits non-translation;
declared names and the dispute note are unfenced on some sheets;
the identity block goes by four headings.

## Floor replay over archives and settled pages

Probes:
`~/temp/agent/audit-floor-replay/`
(`replay.mjs`,
`fires.json`,
`classification.json`,
`controls.json`,
`page-checks.json`,
`han-residue.json`).
Archive against itself:
1277 slices,
75 refused;
every settled would-ship slice:
5520 slices,
233 refused,
every true fire from a run built before its floor.

### F-1: the address floor refuses correct English whenever any third-person pronoun appears anywhere in the slice

Status:
fixed;
the floor reads block by block and refuses only a surplus third-person pronoun.
Replayed old against new over 1,264 archive slices and 3,975 would-ship slices
(`~/temp/agent/audit-floor-replay/address-replay.mjs`):
refusals 25 to 13,
none added.
Cleared:
every false refusal named here but lintong s1,
plus Huasheng s3 (an address rendered by name,
no pronoun in its place),
MTF_0615 s8,
Rentable_A s4 and windward0032 s4,
s15 and s18 (omissions the judges read).
Kept:
the person switches (Huasheng s7,
Mizuki_Yuuki s5,
yingying s2 twice,
XingZ60 s110 and s112),
the indirect-speech conversions this finding counted as switches (Xu_Yushu s12,
shihai4h s15),
Xu_Yushu s28 (an omitted quote whose refusal asks for it back),
lintong s1 (the count's limit:
a generic 你 beside a subject-dropped description),
and mikaela_khara s17,
whose would-ship text belongs to another slice (the F-6 carving).
The `thirdPerson` filter in `droppedAddressFindings` scans the whole slice;
the header names 干干你的 as left to the judges and the built floor refuses it.
Also counted as addresses:
迷你,
你们好,
你追我赶,
你我,
generic 你.
False archive refusals:
BI4PBV s3,
Zha_Ke s3 (a vocative),
Y1Ran s18,
Xu_Yushu s15,
lintong s1;
hulicaijia8 and hulicaijia13 had "“Sis!
What's wrong!”
I kept calling out to her" refused.
Fix:
refuse only a surplus of third-person pronouns over the original's own,
and drop the non-address patterns.

### F-2: publication checks cannot re-verify pages an earlier build published

Status:
closed by A16b and A16c,
by the owner's choice rather than by the fix first proposed.
`inArchiveTypography` re-applies today's typography to old artifacts,
so 77 of 209 pages built before class one hundred eighty-one no longer reproduce by splice.
Fix first proposed:
record the per-slice text shipped at publish and verify against it.
Re-measured on 2026-09-27 as 77 of 214 (A16c),
and put to the owner,
who chose that a pass rewrites such pages to the running build's reading.
`verify-published` now names such a disagreement `READ BY ANOTHER BUILD` with both digests,
and a pass resumed in the directory republishes the page,
after which it verifies;
a record proving an old page equals what its own build shipped would serve no reader,
since the page is rewritten rather than kept.

### F-3: no floor refuses a Han name or line left in English prose

Status:
fixed;
the floor in `translate-han-residue.ts`,
the identity flag in `identity-han-items.ts`.
shihai4h2 shipped "Wrong,\n小柿子."
(the archive has "Wrong.");
the identity context shows the archive's untranslated Han alias as the English declaration.
Fix:
a Han-residue floor outside comments,
code,
destinations and attributes,
excusing a parenthesized gloss;
mark a Han-only TRANSLATION value as untranslated.

The floor refuses a run of Han (with any kana and 々 inside it) in the candidate's prose,
read outside `protectedRanges` with comments and title-floor-accepted titles cut.
It excuses a parenthesized gloss,
a kana line with no Latin letter (a Japanese quotation kept beside its English),
and a run the original and the page both carry.
**The floor is relative to the page**,
chosen so the archive's deliberate keeps (the 澪 a name is written with,
Japanese lyric lines,
a hidden line inside an element) do not fail their own slices;
it refuses Han a candidate adds,
not Han the archive already carries from the original.
It runs with the untranslated floor in both the readable and unparseable branches (the F-5 lesson),
and a copied original is named once,
by the untranslated floor.

Measured with a prototype over the census (`~/temp/agent/audit-floor-replay/han-residue-census.mjs`,
`han-residue-prototype.mjs`),
then with the built floor (`han-residue-port.mjs`),
no per-slice delta:
of 1,264 archive slices 14 carry Han in prose and 4 are refused;
of 3,975 would-ship slices 98 carry Han and 68 are refused.
The four archive refusals are one entry's quotations the archive left untranslated
and respelt in traditional characters where the original writes simplified (一抹陽光 against 一抹阳光),
so on those slices the incumbent cannot stand in and a lane must translate them:
chosen,
not overlooked.
Every would-ship refusal matches a house rule:
handles left in Han (小柿子,
锦心,
雨狸,
洁澄天奏 and others),
terms (大证,
贴贴,
药娘),
titles,
a corner-bracketed word,
and the Chinese half of a pair the original gives in both languages.

Two mistakes surfaced on the way,
both fixed.
A bracketed title carrying both Han and Latin letters (《舞萌DX》) fell between the Han title floor,
which reads Han-only titles,
and the Latin title floor,
which reads Latin-only ones,
each header naming the other;
a pin asserted a bare 《Nyan物语》 in English prose valid.
`withoutGlossedTitles` now cuts every glossed title carrying Han,
so a glossed mixed title passes as a glossed Han-only title does and a bare one is refused by the residue floor.
And the first port of the floor iterated a line by code point against flags indexed by UTF-16 unit,
which would have shifted every flag after a character outside the basic plane;
lint caught the spread,
and a guard now pins the alignment.

The identity line now names each TRANSLATION item written in Han alone as still in Han and no English rendering,
item by item,
since the shihai4h value mixes Han items with a Latin one (`小柿子, 猫小泪, u3`).

### F-4: the suicide floor refuses ordinary English and its census counted entries, not slices

Status:
fixed;
an attempt on a life,
a death by one's own hand after a death word,
and an attributed quotation of a published work pass,
and the finding no longer says "she".
Replayed old against new over the same 5,239 slices
(`~/temp/agent/audit-floor-replay/floor-replay.mjs droppedSuicideFindings`):
refusals 11 to 8,
none added;
the three cleared are the correct English this finding names (one phrasing,
one quotation in two builds),
and each of the eight kept drops or blurs the word.
The header's entry census is corrected by the slice replay.
"Died by her own hand" had no replay case;
it is guarded by an invented one.
Refuses "attempts on her own life",
"died by her own hand",
and a canonical English quotation whose Chinese added 自杀;
the message hardcodes "she".

### F-5: a source the strict grammar refuses turns off floors that need no grammar

Status:
fixed for the floors;
the disagreement over `unknown` moves to X10.
The untranslated,
line-count and neutral-pronoun floors run before `unknown` is returned.
`validateTranslatedSlice` returns `unknown` before the untranslated,
line-count and neutral-pronoun checks;
stages disagree on what `unknown` means.

### F-6: `carveSettled` does not carve as the pipeline did

Status:
fixed in `c7f534353` (red guard `83f35d955`),
with A18.
It omits `includeFrontMatter`,
`frontMatterAuthority` and `sealArchiveOriginal`,
shifting slice indices by one on archive-authority entries.
`carveSettled` now carves through `rebuildPreparation`,
which reads every flag off the artifact,
so the probes,
the rendering audit and republishing share one carve,
and it reports whether the carve reproduces the run's rows (the displacement probe warns when it does not).
`includeFrontMatter` defaults to true,
so only the authority and the seal moved a carve;
the guard carves an archive-authority entry,
which the old code sliced with an extra metadata slice.

### F-7: the line-structure floor counts HTML comment lines

Status:
fixed;
content lines are read outside comments and one line reads "1 line".
Replayed (`floor-replay.mjs compareLineCounts`):
refusals 23 to 21,
the two cleared being these slices,
none added.
yulianNyanner s8 and s12 archives refused;
the message prints "1 lines".

### F-8: the sheet-leak floor misses six labels its own sheets print

Status:
fixed with E7;
every fenced header line is refused whatever its label,
and the list gains the missing heads for unfenced copies.
The guard calls the floor directly:
through the composed verdict the added block is refused by the block comparison first,
which hid the gap.
Replayed (`floor-replay.mjs sheetLeakFindings`):
no refusal before or after,
so the fence rule refuses no page text.

### F-9: the neutral-pronoun floor never reads the original

Status:
fixed;
the floor takes the original and asks only where it writes TA,
Ta or ta.
Replayed (`floor-replay.mjs neutralPronounFindings`):
the same four refusals before and after,
each on an original that writes the pronoun;
the false ones were constructed controls,
now guarded.
"The TA graded the cat's homework."
(助教) and "Ta!"
are refused.

### F-10: `assertHeadingsStayDistinct` goes silent when heading counts differ

Status:
fixed;
a page heading repeated more often than the original repeats any heading refuses whatever the counts.
Measured (`~/temp/agent/audit-floor-replay/heading-replay.mjs`) over 92 archive pages and 213 fixed run pages:
only yuliannyanner3 refused,
as before.
A first version counted one heading as a repeat and would have refused yingying's archive and 12 runs
(an added heading on an original with none);
caught by the measurement before commit.

### F-11: the glossary floors refuse a glossed Han title the title floor allows

Status:
fixed;
the glossary floors read the candidate with the accepted title occurrences cut.
Latent:
no replayed slice carries such a title,
and the community floor adds no refusal.

### F-12: low items

Status:
the first two fixed;
the plausible pair measured and closed,
no gap in the current build.
`ArchiveOriginalCompletenessError` stores neither entry nor span;
the declared-link floor refuses Zhihu @-mentions the archive rendered as the user's slug;
owner,
2026-09-27,
"Account handle":
an @-mention of a declared person may carry the account handle the page writes under the same link.

The error now carries `entryId`,
`spanIndex`,
`startOffset` and `endOffset` as fields,
and its message the offsets (guard shown red with one field unset).
The declared-link floor passes a rendering whose link text,
under the href of an original @-mention of the declared person,
carries an @-handle the page writes under that href,
and its finding offers the handle;
a link naming the person without a mention still owes the declared form.
Measured through the composed verdict:
the GLaDOSister s8 and Kotori s8 archives went from refused to passing;
zhangyubaka s18 is refused first by the suicide floor,
one of its eight kept refusals.

The plausible pair,
read from the code:
a lanes-agreed text is the translate lane's delivered text,
which is a candidate `translate-floor.ts` passed
through `validateTranslatedSlice` with `lineStructured`,
`declared` and `disputedWordings`,
and then only the semantic wrap,
which splits prose lines,
never joins them,
and skips governed slices;
a repair-lane text ships otherwise only through the contest verdict (line structure read since H2)
or the consolidation's `readStandingVerdict`,
both of which call the floor.
So the repair lane not calling `validateTranslatedSlice` leaves no unfloored path to the page.
Re-flooring every would-ship wording of the settled artifacts with today's floors
(`~/temp/agent/audit-floor-replay/refloor-by-decider.mjs`) refuses 163 of 1,961 contest winners,
51 of 853 consolidations,
11 of 269 lanes-agreed texts,
82 of 909 polishes and 7 of 163 page-assembly rows;
those artifacts were written by builds before the floors this audit added or tightened,
so the numbers measure how far the floors moved,
not a live gap.
Plausible,
unproven:
nothing re-floors a lanes-agreed would-ship text,
and the repair lane never calls `validateTranslatedSlice`.

## B699: the declared-name floor read a swap of declared names as a loss, and its first fix passed undeclared blends

Status:
two fixes of 2026-10-20 (UTC),
integrated in the tree at `191a8bc38d`.
The swap rule:
red `8f5129e0dd`,
fix `0adb8d12be`.
The whole-name arrival rule:
red `f9686ef927`,
fix `022f33254a`,
tests and scan repair `6766f25dec`,
README `1513587e43`.
The translate slice,
repair chunk,
refine slice,
consolidation polish and consistency carry settlements can differ for a slice they refused before.
No version moved for this work alone:
the version commit `28f4b8fc75` says the declared-name change lands after it and before any run,
so it rides inside the numbers that commit moved.

### The swap

Found in the read of XingZ6015,
slices 15 and 80 of XingZ60.
The front matter declares the name XingZ60 and the aliases Lili and Z60;
the original's 璃璃 renders as Lili.
The archive wrote "XingZ60" where the original writes the alias.
At slice 80 the winning repair patch (12 of 12 served accepted issues resolved,
a major tense error among them) and the translate replacement both wrote the alias,
and both were refused at 2026-10-19T21:58:36Z and 22:56:50Z with
"archive text carries "XingZ60" and the replacement does not",
so the archive paragraph shipped with its errors.
Slice 15 drew the same refusal in the translate lane.

The floor read every form the base carried and the candidate did not as a loss,
and checked each form alone,
so a person named by another of her own declared forms read as a lost name.

`findDroppedDeclaredNames` takes the optional `subjectForms`,
the forms the front matter declares for the entry's own subject.
A lost subject form is met by a subject form the candidate carries as a name and the base did not carry,
one new form for each lost form.
Contributor handles are not in the group and stay a plain loss,
and so does every caller that passes none.
`PreparedDocumentPair` carries `declaredSubjectForms`,
and the translate slice,
the repair chunk verdict,
the refine stage and the consistency floors of the corpus pass pass it.
Mittens (Blossom) becoming Mittens still loses Blossom with nothing in its place.

Replay over the 331 stored artifacts,
98 stored refusals,
85 reproduced by the old rule from stored texts
(43 translate and 42 repair;
the other 13 are repair patches whose text is not stored):
43 of the 85 passed under the swap rule,
every one a lost subject form met by a new subject form
(XingZ60 34,
hulicaijia 4,
Toka_ls 3,
GLaDOSister 2),
and 42 stayed refused with no subject form arriving.
The new answer is a subset of the old one on all 98.

### The blends

The lead's review of the swap rule read the 43 passes of that replay.
19 of them wrote an undeclared name:
"Xing Lili" (11),
"XingLili" (7) and "Xing Li Li" (1),
where the front matter declares XingZ60,
Lili and Z60.
A piece of the old name had been fused onto the alias,
a person renamed.
The carriage rule reads a name at a word edge,
a camel edge and a split spelling included,
so the new form met a lost one inside a blend nobody declared.
The swap rule's cases held no blend,
so none failed,
and the work went back to the agent.

`whole-name-arrival.ts` counts a new form only where it stands as a whole name,
on at least one occurrence:
it does not run on from or into a Latin letter or digit,
or across a hyphen or an underscore;
it is spelled as declared,
Markdown escapes,
case,
white-space runs and punctuation at its ends aside,
so "Li Li" is not "Lili";
and no capitalised Latin word stands directly beside it,
unless that word belongs to a declared form of anyone,
is an ordinary sentence opener,
title,
kin word or month (`ordinary-capitalised-words.ts`),
or stood beside the lost form in the base.
Only an arrival is held to this;
`nameOccurrences` joined the name projection,
and `carriesName` reads it,
so carriage is unchanged.

The fix commit shipped the two new modules without tests of their own,
wrote a no-break space literally,
and read blankness with `trim`,
so five of the package scans failed.
`6766f25dec` gave each exported function its tests,
folded a run of white space with `isTagWhitespace`,
and wrote the narrow no-break space (U+202F,
in the pinned page MTF_0615) as an escape;
the false-refusal count went from 75 to 67 with it.
Each guard of the arrival rule (joint,
spelling,
neighbour on either side,
ordinary words,
base-beside words,
declared words) was removed in turn and failed a named test.

Replay of the whole-name rule over the 85 controlled rows:
the 19 blends are refused again,
the other 24 still pass
(XingZ60 15 writing Lili alone,
hulicaijia 4,
Toka_ls 3,
GLaDOSister 2),
and the answer is a subset of the old one everywhere.
False refusals,
every whole-name occurrence of a declared subject form in the 91 pinned archives and 281 settled pages
tried as an arrival:
67 of 7596 held back
(48 written with other spacing than declared,
12 joined to a handle by an underscore,
7 beside a capitalised word of a longer name),
and none a sentence opener.
The caveat:
the list of ordinary capitalised words was drawn from the same corpus,
so the false-refusal count measures the list against the pages it came from.

Recurrence:
none of an earlier entry.
A rule that widens what a floor passes is replayed over the refusals it now passes,
and each pass is read,
before it lands.
