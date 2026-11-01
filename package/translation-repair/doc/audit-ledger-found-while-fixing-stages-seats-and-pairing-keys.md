# Audit ledger: found while fixing, stages, seats and pairing keys

Defects met while fixing another finding,
in how the stages stop,
seat and re-seat,
and in what the slice and pairing keys carry.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## X1: the translate lane stopped the entry on a rejected slate over an archive the floor refuses

Status:
fixed with the fifteenth addendum's translate-lane extension (sixteenth addendum).
`translate-stage.ts` treats an archive failing the floor as an absent incumbent;
two rejected production rounds rethrew `TranslateAbsenceError`,
and `translate-slice-attempt.ts` rethrows it for a content slice,
so the entry stopped.
Guard:
`slate-decline-ships-by-preference.unit.test.ts` (the translate stage case).

## X4: the consolidation producers' repair turn re-checked revisions without the declared names

Status:
fixed with the sixteenth addendum.
`consolidate-produce.ts` validated each proposal with `subject.declared`
and passed no `declared` to `repairInvalidCandidates`,
so a revision that dropped a declared name passed the re-check.

## X5: neither slice key named the archive dispute

Status:
fixed with the sixteenth addendum.
`translateSliceKey` and `consolidateSliceKey` hashed the texts but not the dispute note,
so a slice judged under accepted claims could resume a record settled for the same texts undisputed,
and with the archive as incumbent on an unresolved dispute the texts would be identical.

## X6: the translate lane's refusals keep an archive the floor refuses

Status:
fixed in `6445a2e35` (guard `ee3a551e3`;
mutation checked with a control,
three mutants caught).
`translate-slice.ts` gates its alignment,
quote-loss and declared-name refusals on the slice having archive wording,
not on that wording passing the floor,
so a replacement refused there ships the floor-refused archive as the lane's text
(shihai4h2 slice 14 kept a 1665-code-point archive against a 102-character source);
the consolidation then refuses it as a standing.
The disputed case no longer does this;
the floor-refused case needs the stage's eligibility on the record.
Measured 2026-09-28 over 975 run logs (`~/temp/agent/audit-glossary-fix/x6-census.mjs`):
of 1,771 translate refusals,
up to 47 were followed by the consolidation refusing that slice's standing,
46 for a link the original carries and 1 for the untranslated pronoun;
"up to" because the census cannot tell which lane the refused standing came from.
The slice now asks `validateTranslatedSlice` the consolidation's standing question from the same inputs
(`translate-archive-floor.ts`),
and a refused archive is kept by no refusal:
the judges' replacement goes on,
the record carries `translate-archive-ineligible`,
and a log line gives the rule's reason.
A first fixture put a link in the original,
which re-paired the section as an insertion and failed for the wrong reason;
the guard uses the pronoun floor,
which reads only the text.
Rides inside the translate cache version 15 with a written account.
The 47 measures the symptom,
not the fix's reach,
which is unmeasured on the agreement path:
where the repair lane also left the archive,
a kept archive made the lanes agree and shipped with no floor,
leaving no consolidation line,
and such slices now go through the contest and the consolidation.
An original the grammar cannot read (`unknown`) lifts the refusals too,
as the consolidation refuses that standing alike.
Not a launch factor:
no TianqiChen666 run refused anything in the translate lane.

## X7: windowed stages re-ask a seat the router refused

Status:
fixed with guard `9466786dc`,
fix `84a6caa02`.
`stage-windowed-rounds.ts` put every seat that never answered back on its pending list,
a refused one included,
so the lane contest,
pairing,
the gate,
the naturalness review and the polish gate
re-asked a seat no wet provider served in every retry round
(a four-seat fixture asked it four times).
`stage-quorum.ts` fixed the same defect on 2026-09-09 (`hulicaijia`);
this path never got it.

## X8: windowed stages size quorum over the seated bench

Status:
fixed with E3 in `29baade8f` (red guards `a6cbd8fc5`):
`runWindowedRounds` re-sizes its quorum with `reachableQuorum` before every round,
and the five stages beside the naturalness review carry `stage-short-bench` in their findings.

Found as:
`runWindowedRounds` takes `heardNeeded` from `rosterQuorumSize` over the seated bench
and never applies `reachableQuorum`,
though the 2026-09-09 addendum of `doc/decision/translation-repair-short-bench-share.md`
says the reachable share sizes every gather.
A windowed stage whose reachable seats cannot meet the bench quorum spends its retry rounds and closes short
without the `stage-short-bench` finding the gathers carry.

## X10: a slice carved through an element can never settle, and the stages disagree on `unknown`

Status:
fixed in `a499a2fc2` (red guard `f4736eed4`,
whose fixture was corrected in the fix;
mutation checked).
The carve was not the cause:
NIGHT81473140 slice 22 holds the whole element,
an opener and a self-closing component with an expression attribute on one line and the closer on the next,
and the document and the slice both parse it whole.
The lone-container masker (`mask-container-tags.ts`) paired whole tag lines only,
so it blanked the closer as unpartnered and left the opener with no end;
it now pairs against openers and closers of the same names anywhere in the slice (`inline-container-tags.ts`)
and masks only a tag line left unpaired.
The deterministic carve of all 92 entries now has 0 of 1,259 slices whose source or translation the slice grammar refuses.
The stages still disagree on `unknown`,
which after this fix means an original the grammar refuses whole,
where the page would fail the upstream compile as well;
none occurs in the pinned corpus,
and no change is made there.
The first red guard used a fixture that is not valid MDX even whole (a `span` on the opener line),
so it failed for a reason other than the defect;
the fix commit replaced it with the entry's shape
and the mutation check (pre-fix masker,
rebuilt) showed the corrected case failing.

Found as (this first diagnosis blamed the carve and was wrong):
NIGHT81473140 slice 22 of the deterministic carve opens a `<blockquote>` that closes in a later slice,
so the strict grammar refuses the original (`end-tag-mismatch` at 1:1) and the verdict is `unknown`.
The producers read `unknown` as a pass (`translate-floor.ts` keeps any voice not `invalid`;
`translate-repair.ts` lets it stand with a `translate-unvalidated` finding),
while every gate reads it as a refusal (`translate-stage.ts` keeps the incumbent off the slate;
`lane-contest-eligibility.ts`,
`consolidate-lane-offer.ts`,
`consolidate-standing-verdict.ts`
and `consolidation-polish-round.ts` accept only `valid`),
and a candidate that mirrors the cut fails the strict grammar itself and is refused as unparseable.
The slice can therefore never settle,
and the entry stopped;
since the owner's ruling of 2026-09-27 ("Keep archive,
ship") no wording passing the rule keeps the archive there,
so the entry ships with the slice on its `DEFECTS` line,
but the slice is never improved.
The root is the carve cutting an element;
once carving keeps elements whole,
`unknown` means an original the upstream MDX compile would also refuse,
where refusing is right.
One slice of 1,277 in the replay;
no run has reached it (no settled artifact exists for the entry).

## X11: the consolidation run shape omitted the slate judges

Status:
fixed in `c5ac38345` (red guard `01c8bbad8`;
mutation checked with a control,
the dropped-judges mutant caught).
`9a7d48354` split the slate judges and the gate from the writers on 2026-09-02
and left them out of `consolidateRunShape`,
so a settlement another judging bench reached would resume under a bench that never judged it.
The run shape now carries the judges beside the writers.
Every consolidation key moved inside version 20,
under which no slice-cache file had been written
(checked 2026-09-28:
the newest is the TianqiChen66620 consolidation entry at 04:26 UTC on 2026-09-27).
The other run shapes carry every bench they ask:
the translate lane's its writers and judges,
refine's its refiners,
judges and checkers,
the lane contest's its one bench.
The same commit also folded `checkerSelfCertificationPermitted` into the repair run shape where refused;
that half was wrong and `6d9b361b4` took it out again (M37).

## X12: the lane contest, the preparation and the picture readings never re-seat under a hold

Status:
fixed in every phase:
lane contest,
picture readings,
insertion admission and preparation.
Contest:
prep `4bd530922`,
red guards `5acdd36c6`,
fix `2a0ee0272`
(mutation checked with a control,
seven mutants caught).
`contestHooksFor` (`corpus-run/pass-contest-reseat.ts`) re-reads the late judges while a hold runs
and keeps them once it ends;
`contestDocumentLanes` seats and keys each slice on them (`shapeFor`),
so a slice nobody re-seated keys as before and no cache version moves.
The three copies of the hold-and-memo re-seat are now one,
`reseatHookFor` (`corpus-run/pass-reseat-hook.ts`):
the consolidation hook moved onto it in `09839c622` and the lanes hook in `df601d839`,
after `f60abcb15` pinned that each lane keeps its own seating.
The migrations were mutation checked with a control,
followed by the full suite (0 FAIL,
1264 PASS):
eight mutants caught,
one of them only after `87c3d7aaf`.
The translate hook reading under the lanes phase survived the first run,
since no case told the phases apart;
`87c3d7aaf` does,
under the one view of all sixteen that separates them
(Hyper and OpenRouter dry:
the lanes phase finds the editors and refiners short,
the translate lane nothing).
`awaitBenchQuorum` lost its last caller with them and was removed in `c2e74caeb`,
with the `run-seats-read.ts` comment that said every driver asks it.
Pictures:
prep `3ef2e210d`,
red guards `fcd16ce66`,
fix `d69db363a`.
`picturesHooksFor` (`corpus-run/pass-pictures-reseat.ts`) re-reads the readers while a hold runs;
`readDocumentPictures` reads and keys each picture by them,
and `readSeatedPictures` builds one hook per call,
since each call reads the seats afresh.
The picture key has no version and already folded the readers.
Mutation checked with a control,
then the full suite (0 FAIL,
1266 PASS):
four mutants caught,
two survived.
The hook reading under another phase survived because no view tells the pictures,
lane contest
and consolidation phases apart;
`1fa856e0d` asserts each hook's logged re-seat line with its exact phase,
which caught all four phase mutants and a helper logging no phase.
The seam dropping the hook survived because no test drives a pass seam:
X14.
The census counts holds after the pictures reading under the pictures,
so its 11 in 3 logs is an upper bound:
the sighted pairing and the archive review run after the readings on the preparation's reading.
Insertion admission:
prep `c087e6e33`,
red guards `2fbb7ec24`,
fix `105e81220`.
`decidePassInsertionAdmission` asked every source-only candidate of the roster the lanes read.
`insertionHooksFor` (`corpus-run/pass-insertion-reseat.ts`) re-reads the roster under a hold
under its own `insertion admission` phase,
which leans on the wide bench as the preparation does,
and `admitPassInsertions`,
the seam `runPassEntry` now calls,
wires it (split out to keep `pass-entry.ts`
under its line cap,
and so a test drives the wiring:
X14).
The coverage stage caches nothing.
Mutation checked with a control,
then the full suite (0 FAIL,
1271 PASS):
four mutants caught,
one survived,
the new phase leaning on another bench,
since only some phases' benches were pinned;
`c2adcfdbf` pins the whole table,
which caught it and a lane contest bench mutant.
Preparation:
prep `aa635056a`,
red guards `f891b358e`,
fix `92556ba44`,
guards completed in `d00123df3`.
`preparePassEntry` asked its attestation,
every section and block pairing round
and every archive block review of the roster read before it.
`preparationHooksFor` (`corpus-run/pass-prepare-reseat.ts`) re-reads the roster under a hold
under the `preparation` phase,
and `runPassPreparation`,
the seam `runPassEntry` now calls,
wires it.
One `beforeItem` hook reaches every site that asks:
`prepareDocumentPairWithRoster` reads it before the section round
and before each section's block round,
`repairArchiveBlocks` before each block,
and `attestPassReferences` (`corpus-run/pass-attest-references.ts`) before the attestation,
only when the original links somewhere.
The pairing keys fold the roster that answers (X13),
so a re-seated round is keyed by it.
The first mutation run (control survived) caught six mutants and left four more alive besides the attestation site.
The pairing fixture never bought a section round,
its hook handed one constant roster
so a block round reusing the section round's seating passed,
and the seam fixture left no archive block unclaimed.
The phase table mutant survived only because the spec left out `corpus-run/run-seats-wait.unit.test.ts` (M41).
`d00123df3` rebuilt the pairing and review guards on a hook alternating two disjoint rosters,
requiring every call to ask the roster last handed over and each round to be reached,
and added a seam case whose archive carries an unclaimed block.
The attestation had no guard a fixture could reach,
since `preparePassEntry` read the linked pages off the web.
`attestPassReferences` now takes the round out of it with a required hook
(`keepBench` in `bench-seating.ts` for a caller with none,
since the repo models absence without nullish unions),
and `preparePassEntry` takes an injected reference reader,
as it takes `readPictures`,
so a case drives the whole preparation and requires every call to ask the roster the hook hands over.
The second run,
with a control,
caught all fourteen mutants,
including the attestation wiring,
an unlinked original still asking,
and the injected reader ignored;
the full suite then passed (0 FAIL,
1278 PASS).
All four call sites are guarded.
The one wiring a type guards rather than a test is omitting `beforeItem` at `attestPassReferences`,
which fails the type check;
replacing it with `keepBench` is the tested mutant.

Found as:
The fifth stage of the H5 family.
`runPassContest` reads its judges once and `contestDocumentLanes` has no per-slice hook,
so a dry-out inside the contest leaves every later slice on the judges read before it,
and nothing waits out the hold either:
the per-slice wait `e17d0c487` added on 2026-09-07 reached the lanes and the consolidation only.
The preparation (pairing,
archive review) and the picture readings read their benches once too.
Measured 2026-09-28 over 4,122 run logs (`~/temp/agent/audit-glossary-fix/phase-hold-census.mjs`),
holds that began inside each phase,
counted from its seat reading to the next phase's:
lane contest 222 in 12 logs,
preparation 25 in 4,
pictures 11 in 3
(consolidation 983 in 25,
lanes 353 in 42,
translate lane 115 in 14,
all three now re-seating).
With the consolidation hook re-seating,
`awaitBenchQuorum` has no production caller left,
and `corpus-run/run-seats-read.ts` still says every driver asks it before each chunk.

## X13: the pairing keys carry no roster

Status:
fixed in `fea6e4688` (red guards `e9caf3f27`;
mutation checked with a control,
three mutants caught).
Both keys fold the roster that answers,
last,
after a separator and the word `roster`;
the block round passes its roster through `blockPairingQuestion`,
and the block key goldens were recaptured
under a fixture roster.
Every pairing key moved inside version 3,
under which nothing was written.
The full suite after the mutants found one more test deriving pairing keys itself
(`corpus-run/pass-footnote-lifecycle.unit.test.ts`),
fixed in `b84c2384c` (M39).

Found as:
The same defect X11 fixed in the consolidation key.
`blockPairingQuestionKey` (`block-pairing-question-key.ts`) and the section round's `roundKey`
(`prepare-section-round.ts`) hash the version and the texts only.
The pipeline digest the pairing caches are opened under covers the static roster,
since any change to `corpus-run/run-config.ts` rebuilds `dist`,
but not a dry reading's subset:
`pass-entry.ts` seats `preparationSeats.roster`,
and a section paired by a subset resumes on the full bench under the same generation and key.
Both rounds persist once any voice was usable (`usable > 0`),
so a one-voice answer is kept as well.
Neither the keys' TSDoc nor `doc/decision/llm-assisted-block-pairing.md` rules that pairing is roster-independent.
Pairing answers live beside the slice caches (`slice-cache/<entry>/pairing.*.json`);
version 3 was set in `d614a0c1d` at 04:30 UTC on 2026-09-28,
after the newest slice-cache file (04:26 UTC on 2026-09-27),
so none was written under 3 and the roster can ride inside it.
It must land before the preparation half of X12,
or a re-seated section resumes the old bench's answer.

## X15: the pairing keys are not injective

Status:
fixed in `692ebe3c0` (red guard `aee557392`,
section kind pinned in `c89bbda4f`),
as hardening:
measured,
no pinned input reaches the aliasing.
`pairingQuestionKey` (`pairing-question-key.ts`) now builds both keys from one JSON value of fixed shape
(version,
question kind,
both sides' texts,
pictures,
roster),
which JSON escapes injectively and which keeps a section key and a block key apart over the same texts.
The block goldens were recaptured,
and the two embedded NUL cases they had pinned to one key now differ;
three tests that joined the key by hand name its material and call the encoder instead (the M39 lesson).
Rides inside version 3:
checked again on 2026-09-28,
the newest slice-cache file is still from 04:26 UTC on 2026-09-27.
Mutation checked with a control:
nine mutants caught (the question kind,
roster,
pictures,
version,
sides swapped,
texts flattened,
and each key passing the other's kind or dropping its pictures),
after `c89bbda4f` pinned the section round's kind,
which no test had named;
the full suite then passed (0 FAIL,
1279 PASS).
The draw keys hash joined fields as well and need nothing:
`sample-draw.ts` fixes its seed and kind within each sort,
so two keys meet only when their ids do,
and `damage-sample.ts` separates with NUL,
which neither its fixed domain,
its seed (`DAMAGE_SAMPLE_SEED`,
an environment value,
or a literal) nor an id can carry.

Found as:
Both pairing keys,
`roundKey` (`prepare-section-round.ts`)
and `blockPairingQuestionKey` (`block-pairing-question-key.ts`),
joined the texts of both sides with NUL and marked the side boundary with one more NUL element,
so an empty text beside the boundary aliased across it:
original `[a, '']` against translation `[c]` and original `[a]` against translation `['', c]` hashed the same bytes,
as did a NUL inside a text across a block boundary,
and UTF-8 folded every lone surrogate into U+FFFD.
The block key's TSDoc named only an embedded NUL as its aliasing path,
and kept its layout so that every key without pictures kept its historical bytes;
X13 had moved every pairing key inside version 3,
so no historical bytes remained to keep.
Measured 2026-09-28 (`~/temp/agent/audit-glossary-fix/x15-empty-text-probe.mjs`):
over all 92 pinned pairs,
570 sections and 5,063 blocks,
the archive side normalized as the preparation reads it,
no section or block text is empty and none holds a NUL,
and no parser edge case tried (front matter alone,
a bare heading,
a thematic break,
an empty code block,
quote,
list item or footnote,
a comment,
a picture,
a component) yields an empty text,
since a node's text is its own non-empty span.

Found as:
Reading `prepare-section-round.ts` whole after its raw NUL bytes had hidden it from every line search (M40).
The golden file also cited a scratch baseline (`question-baseline-QWeptI`,
from `62cedf6fa`) that no longer exists;
the recapture removed that line.

## X18: the preparation dropped the attestation's findings when the archive was corrected

Status:
fixed in `d80f56866` (red guard `5f4863f32`),
2026-09-28.
`preparePassEntry` (`corpus-run/pass-prepare.ts`) has three ways out:
no unclaimed block,
an archive the review leaves standing,
and an archive corrected and prepared again.
The first two carried the reference attestation's findings;
the third rebuilt its findings
from the re-preparation,
the relabel and the review,
and dropped them.
The attestation's and the page title lexicon's findings now form one `evidenceFindings` list on every way out.
The guard drives all three returns and asserts each is reached (the M42 prevention):
its first fixture,
on a two-seat roster,
never reached the third,
since the correction slate kept the archive,
and the reach assertion said so before any verdict was read;
both seats wrote the one revision,
each vote for it weighs `SELF_VOTE_WEIGHT` (1/2),
and 1 falls short of `MIN_SELECTION_WEIGHT` (2) (`candidate-select-model.ts`);
on four seats,
as the older correction test uses,
the slate selects the revision.

Found as:
Reading the returns while wiring H16's lexicon findings into them.
