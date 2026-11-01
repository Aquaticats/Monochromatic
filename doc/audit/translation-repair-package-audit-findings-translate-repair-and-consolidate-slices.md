# Package audit: findings of the translate, repair and consolidate slices

The findings of the slices that hold the pipeline's three stages:
translate,
repair and consolidate.

Part of [Translation-repair package audit](translation-repair-package-audit.md),
which lists every file of this record.

## repair-1, MAJOR, verified: a slice settled while a whole stage heard nobody is cached as a decision

`src/repair-translation.ts:453-470`:
the only gate on the write is `outcome.heardCritics === 0`.
The quorum gather never throws on shortfall (`src/stage-quorum.ts:369-377` returns
`stage-quorum-unmet (...)` as a finding),
and every silent stage downstream lands on an ordinary
"unchanged" exit:
a silent panel makes every claim `needs-human` (`src/tally-votes.ts:76`,
electorate
below `minBallotWeight`),
so `src/repair-chunk.ts:274` returns "nothing to edit,
unchanged";
a silent editor returns `src/repair-editor-stage.ts:251` "no operation survived the gate";
a silent checker leaves every tally unresolved and `src/select-candidate.ts:193` ranks the unchanged
candidate first on the tie.
All three carry `heardCritics > 0`,
so all three are persisted and memoised.
`src/refine-phase.ts:336` persists unconditionally,
so `refine-candidates (0/N heard)` and a
rollback produced by a silent recheck are stored too.
Consequence:
a provider outage lasting one slice (the `#199` week of 429s,
or the `#235` shape where
every seat of one role sits on a dark provider) is frozen into the cache as "examined and found
nothing to change";
every later run resumes it,
reports success,
and never re-asks.
The findings travel with the record,
so the artifact says `stage-quorum-unmet` while the text ships settled.
Retroactive:
caches written during the `#199` outage may already hold such records.
Tracked as `#238`.

FIXED 2026-08-26 in `1a96979ad`,
together with slices-1.
The finding has one spelling,
built and read through `stage-silence.ts`;
the repair driver refuses to cache a settlement carrying it through `cacheRefusalsOf`,
which names its reasons,
the refine driver does the same,
both cache guards refuse to resume such a record,
and the namespace loader now says which record it refused instead of dropping it silently.
Guard:
a persisted slice carrying `stage-quorum-unmet (critic 0/6)` is not resumed while its neighbour is;
shown to fail with the guard's finding check removed (2 failing lines),
restored,
passed.
Retroactive scan:
this worktree's default runs directory holds 150 slice-cache files and none carries the finding;
the main worktree has no runs directory,
so nothing needed purging.
Not guarded:
the two persist-side refusals in the drivers have no driver-level test,
because the scripted client in the driver suites cannot make one stage fall short of quorum;
the existing no-critic refusal beside them has none either (repair-9),
and both are listed there.

## repair-2, MINOR (downgraded from the reviewer's MAJOR): the refine loop lacks the accuracy loop's abort check

`src/refine-phase.ts` has no `signal.throwIfAborted()` before its persist at `:336`;
`src/repair-translation.ts:437` checks before its write.
The reviewer's mechanism,
that an abort settles every remaining slice as silence,
does not hold:
`src/stage-round.ts:103-108` rethrows a caught error when the signal is aborted,
`src/stage-call.ts:244-246`
does the same,
and `src/stream-drain.ts:353-358` throws `StreamCutShortError` under a caller abort
rather than returning partial text,
so an aborted call propagates as a throw and nothing after it is
persisted.
The symmetric check is still owed as a guard against a future path that turns an abort into a non-ok
outcome,
and `refine-phase.unit.test.ts` has no abort case.
Folded into `#238`.

FIXED 2026-08-26 in `d7c707cc3`:
`refine-phase.ts` calls `signal.throwIfAborted()` before its persist,
the check
`repair-translation.ts` makes before its own write,
and `refine-phase.unit.test.ts` has an abort case:
a client that
aborts the run from inside its first exchange and still answers leaves the cache empty and the phase rejected.
GFP,
reported as measured:
with the new line removed the abort case STILL PASSES,
because the stage machinery already
throws on an aborted signal before the phase reaches its write,
exactly as the entry's mechanism paragraph says.
The line is the symmetric guard the entry asked for against a future path that settles an aborted call;
the case
pins the behaviour,
not the line,
and this paragraph says so rather than claiming a guard it cannot show failing.

## repair-3, MAJOR, verified: envelopes adopted without a vote lose their authors

`src/editor-ensemble.ts:232-243`:
the sole-proposal path pushes the winner and its contributors but
no round (`rounds.push` happens only on the judged path at `:282`).
`src/issue-authors.ts:311-318` builds a composite's `perIssue` from `envelopeAuthorsFromRounds({ rounds })`,
and `:227` reads `byEnvelope[envelopeId] ?? []`,
so every issue served by a sole-adopted envelope has no author.
Under `checkerSelfCertificationPermitted: true` (the live setting) a checker that wrote that text votes
on it at full weight and the artifact's `wroteTheText` is false for it.
Bounded today because the checker and editor rosters do not overlap (`#187`);
the record is wrong now
and the weighting becomes wrong on the first overlapping roster.
`issue-authors.unit.test.ts` has no composite built from a sole envelope.
Tracked as `#239`.

FIXED 2026-08-26 in `7103ae59c`:
a sole adoption is a round of its own kind,
`adopted`,
carrying the one slate
entry,
its index,
a reason,
and empty vote fields so every reader of a round keeps its shape;
`issue-authors` treats it like a selected round and the artifact rounds reader parses it.
Guards:
the authors case fails with the kind check reverted (2 lines),
the selection case fails with the push
removed (2 lines);
both restored and passing.

## repair-4, MINOR, verified: an outcome with no prepared slice is refined against an empty original

`src/refine-phase.ts:219` (`prepared?.source.text ?? ''`) and `:236` (`?? outcome.repairedText`);
`src/repair-refine-step.ts:150-157` throws for that state after the phase,
so the tolerance buys
model calls before a refusal that comes anyway.
Unreachable through `repairPreparedDocument` today.

FIXED 2026-08-26 in `d7c707cc3`:
the phase refuses an outcome naming a slice the preparation never produced before
any call,
as `UnpreparedSliceError` (`unprepared-slice.ts`,
marked,
one slice index in its message),
and
`repair-refine-step.ts` throws the same class for the same fault instead of a bare `Error`;
the `??` fallbacks are gone.
Guard:
with the refusal removed,
the case
`REFUSES an outcome naming a slice the preparation never produced before buying any call` fails;
restored,
passes.

## repair-5, MINOR, verified: six prompts fence document text with a fixed `=====`

`src/edit-prompt.ts:18`,
`src/critic-prompt.ts:18`,
`src/adjudicate-prompt.ts:17`,
`src/resolution-wire.ts:21`,
`src/restoration-judge-wire.ts:23`,
`src/derivability-wire.ts:22`,
against `src/refine-prompt.ts`,
whose note calls the fixed value forgeable (a setext underline) and
picks its fence against the content with `selectFence`.
A page carrying a `=====` line closes the instruction block early;
every gated stage bounds the damage.

FIXED 2026-08-26 in `9135037e8`:
all six builders choose their fence through `selectFence` against every text they
enclose,
the rendered claim,
issue,
region and reference blocks included;
the adjudication and editor prompts render
their blocks before the nearby block so the fence can see them;
the six constants are gone.
Guard:
each builder put back on a fixed `=====` fails its case
`FENCES the blocks with a delimiter the enclosed text cannot reproduce` (six suites,
one case each;
`edit-prompt.unit.test.ts` is new,
the builder having had no suite);
restored,
passes.

## repair-6, MINOR, verified: two unmarked classes whose messages hold positions only

`src/repair-unheard.ts` (`RepairUnheardError`) and `src/placement-layout.ts` (`PlacementLayoutError`)
carry no `messageNamesOnly` and quote nothing,
so the slice they name is muted at the boundary.
`repair-unheard.ts:160` spells `` archive`s `` with a backtick for an apostrophe.

FIXED 2026-08-26 in `9135037e8`:
`RepairUnheardError` takes a slice index and an `UnheardClaim`,
`PlacementLayoutError` a position and a `PlacementFault` of seven kinds,
and each writes its sentence from indices,
offsets and counts;
both carry the marker and the inventory names their parts.
The backtick apostrophe went with the
message it sat in.
Guard:
with either marker removed the inventory cases `KEEPS exactly the classes the inventory records` and
`KEEPS a reason for every class that writes its sentence and stays unmarked` fail;
restored,
passes.

## repair-7, MINOR, verified: a stale comment names an exit that no longer exists

`src/repair-translation.ts:536`:
"A blocked document already returned above" against the module
note at `:47-56` ("NOTHING BLOCKS THE DOCUMENT ANY MORE").

FIXED 2026-08-26 in `d7c707cc3`:
the comment says nothing blocks a document any more and points at the module note.
Prose only.

## repair-8, MINOR, verified: typography restoration runs after the structural gate

`src/apply-patch.ts:346` applies `restoreTypography` to `newText` after every check,
while
`src/refine-stage.ts:255` gated the raw `operation.newText`,
so a protected atom holding a straight
quote between word characters can be altered by text no gate saw.
Low frequency.

FIXED 2026-08-26 in `9135037e8`:
`applyPatchOperations` restores typography before its checks,
so the
unchanged-region and preservation gates read the text that ships and the applied record equals it;
`refine-stage.ts`
restores a rewrite before `gateParagraphRewrite` reads it and forwards the restored text.
Guard:
the apply gate reading the raw text again fails
`REJECTS as unchanged an edit whose only difference is a flattened apostrophe`;
restored,
passes.
NO GUARD for the refine half:
with its gate reading the raw text again the refine suite still passes,
because no case
there turns on quote style;
the register's scenario (a protected atom holding a straight quote between word characters)
is what such a case would need,
and it is owed.

## repair-9, MINOR, verified: the guards the three MAJORs need do not exist

No abort case in `refine-phase.unit.test.ts`;
the only silence-and-cache case in
`repair-translation.unit.test.ts` is the critic one;
no sole-envelope composite in
`issue-authors.unit.test.ts`;
`repair-not-applicable`,
`assertUnheardKeptArchive`,
`heardNobodyAbout`,
`settleChunkFromChecks`,
`collectEnvelopeProposals` are named by 0 test files;
`src/refine-slice-settle.ts:149` types the refiner roster as `RepairModels['checkerModelIds']`.

FIXED IN PART 2026-08-26 in `d7c707cc3`:
the abort case (repair-2),
the refiner roster typed as
`readonly RosterModelId[]` rather than as the checker seats (the borrowed type was the only non-optional one),
and suites for `repair-not-applicable` (the anchor never counts toward the non-translation block;
every count zero;
the finding's wording),
`repair-unheard` (`heardNobodyAbout` on all three inputs;
`assertUnheardKeptArchive` refuses
a foreign wording and a claimed change,
accepts the archive wording,
asks nothing of a slice somebody spoke about)
and `editor-proposals` (a duplicate merges its writer into the first proposal;
distinct texts stay apart in roster
order;
a silent model contributes nothing).
The sole-envelope composite landed with `#239` and the silence-and-cache cases with `#238`.
Guards:
the claimed-change refusal removed fails `REFUSES a silent slice that claims a change`;
the merge disabled
fails `MERGES a duplicate proposal into the first one`;
the anchor counted as standing fails
`REFUSES to count an anchor toward the non-translation block`;
each restored,
passes.
COMPLETED 2026-08-26 in `9135037e8`:
`settleChunkFromChecks` reaches the bundle and has a case.
Guard:
with the credit inverted the case `CREDITS exactly the creditable issues the checker majority confirmed fixed`
fails;
restored,
passes.

## translate-1, MAJOR, verified: pairing agreement is filtered from the first usable voice's reply only

`src/pair-blocks-stage.ts:330-333` and `src/pair-sections-stage.ts:368-372`:
`const agreed = (pairings[0] ?? []).filter(votes >= AGREEMENT_NEEDED)` with `AGREEMENT_NEEDED = 2`.
Votes are tallied over every usable reply,
but candidates come only from the first,
so a
correspondence two other voices named is dropped whenever the first voice omitted it,
and which pairs
survive depends on which seat answered usably first.
Both files' module notes say "AGREEMENT IS PER PAIR rather than per reply",
which the code contradicts.
Pairing is the input every later stage reasons from:
a dropped block pair leaves its original
`source-only` (the `#157` shape returns),
a dropped section pair sends a translated section through as absent (`#159`).
Both suites seat only two voices,
a shape that cannot show it.
Tracked as `#245`.

FIXED 2026-08-26 in `4eaa89ede`:
agreement is per pair through `agreePairs` (`pair-agreement.ts`),
shared by both
stages:
every distinct pair across every usable voice is counted,
a contested source keeps its better-voted target
or is dropped with a finding on a tie,
and the result stays strictly increasing on both sides with a finding for an
agreed pair that would run backwards.
Guards:
six helper cases plus a three-voice case per stage;
the blocks stage reverted to the first reply's pairs
fails its three-voice case,
the tie rule removed fails the contested case;
restored,
both pass.

## translate-2, MAJOR, verified: ledger file names restart at zero per process and overwrite on relaunch

`src/candidate-ledger.ts:78` (`const state = { recorded: 0 }`),
`:208-209` (ordinal from that counter),
`:252-255` (`writeFile` to `${padded}.json`).
Relaunching into the same runs directory is the documented resume path,
and `ledger-report.ts`
reads the whole directory with no way to tell an overwritten file from an original,
so the `#212`
evidence (producer standings,
self-preference counts) is silently lost for every relaunched pass.
Tracked as `#246`.

FIXED 2026-08-26 in `b00f9d3b2`:
every ledger file is named by a per-process launch stamp (file-safe ISO time plus
process id) and then the ordinal,
so names never collide across launches and still sort as text into contest order.
Guard:
the same ordinal under two launches names two files and names sort by launch then ordinal;
with the launch dropped from the name two cases fail;
restored,
passes.
The reader-side check slices-2 proposed is not added:
with unique names the collision cannot recur,
and files written before this fix already collided in a way no reader can undo.

## translate-3, MINOR, verified: `sentinel-probe` prints any error's message to stdout

`src/corpus-run/sentinel-probe.ts:166-173`,
the same shape as calibrate-2.
Folded into `#237`.

CLOSED by `#237` (translate-3 was folded into it):
`sentinel-probe.ts` prints its error line through `refusalText`
(line 168 at `8e10cfa19`),
so a marked refusal prints whole and anything else prints its class name.
Nothing further to land.

## translate-4, MINOR, verified: a stage note says a guard is missing that `translate-slice.ts` now runs

`src/translate-stage.ts:31-33` against `src/translate-slice.ts` (`findDroppedDeclaredNames`,
two sites).

FIXED 2026-08-26 in `732d7fb41`:
the stage note says where declared names are checked,
in `translate-slice.ts`
through `findDroppedDeclaredNames`,
and keeps the cross-slice gap as the one thing the stage still does not do.
Prose only.

## translate-5, MINOR, verified: the translate slice cache is at version 6 with a history ending at 5

`src/translate-document-contract.ts:57` against the paragraphs at `:31-50`;
nobody can tell what the bump to 6 discarded or why.

FIXED 2026-08-26 in `732d7fb41`:
the history has its version 6 paragraph,
the bump from `973f7b47d` on 2026-08-22
that carried the verse rule to the judges' sheet in both lanes,
so every slice judged without the rule is re-judged;
neither the record's shape nor its key changed.
Prose only.

## translate-6, MINOR, verified: twenty-one files in the slice have no sibling unit test

Named in the reviewer's report (`~/temp/agent/audit-translate.md`);
the branches that deserve a
direct case are `translate-retry.ts:200-228`,
`translate-slice-attempt.ts:182-183`,
the shipped and
withdrawn index sets in `translate-assemble.ts`,
every `blankAgainst` branch in `translate-absence.ts`,
and the three-voice pairing case translate-1 needs.

FIXED 2026-08-26 in `8e10cfa19`,
on the branches the entry named rather than on file names:
`translate-retry.ts`
(a first decline followed by a decision keeps the decision and both rounds' findings with the retry marker;
a first
decision asks once;
two declines settle as `no-candidate-backed` with one marker),
`translate-slice-attempt.ts` (now
exported:
an insertion nobody could fill is reported `unfilled` with its reason and findings;
a content slice nobody
answered settles on its incumbent) and `translate-assemble.ts` (a replacement dropping a footnote marker is withdrawn
at assembly,
listed as withdrawn and not as changed,
and the warning names the count).
`blankAgainst`'s four branches
were already held by `translate-absence-blank.unit.test.ts`,
and the three-voice pairing case landed with `#245`.
Guards:
the retried-decline list emptied fails the first and third retry cases;
the insertion backstop made to rethrow
fails the unfilled case;
the withdrawn set emptied fails the assembly case;
each restored,
passes.
The slice-attempt fixture builds its insertion by hand the way the document suite does,
because the deterministic
aligner yields no slices at all for a heading-count mismatch (measured:
0 slices for a two-against-one pair).

## consolidate-1, MAJOR, verified: a slice with no standing text buys a producer round it then discards

`src/consolidate-driver.ts:333-338` computes `standingText`,
which `standingTextFor`
(`src/consolidate-standing.ts:40-44`) answers as `''` for a `settled-neither` or `quorum-not-met`
contest;
`settleFresh` then builds the producer input (`:378-405`) and calls `produceConsolidations`
unconditionally,
and `settleConsolidation`'s first exit (`src/consolidate-settle.ts:407-412`) refuses
with `no-standing-text` before judging anything.
One roster of producer calls plus up to one roster of repair calls per deadlocked slice,
with no
possible effect on output;
on a night the contest loses quorum,
every contested slice pays it.
Whether such a slice should instead be judged against both lane renderings is a design question the
reviewer raised separately;
the spend is a defect regardless.
Tracked as `#251`.

FIXED 2026-08-26 in `a8bc69508`:
`settleFresh` in `src/consolidate-driver.ts` hands the settle half an empty slate
when the standing text is empty and asks no producer,
so the `no-standing-text` terminal,
its floor and its
findings still come from `settleConsolidation` and the settlement still resumes.
Guard:
a `settled-neither` and a `quorum-not-met` contest each settle to `no-standing-text` under the driver
test's refusing client,
which throws on any call;
restoring the purchase fails 2 cases with that throw.
The design question (judge such a slice against both lane renderings instead) stays open and is not decided here.

## consolidate-2 to consolidate-10, MINOR, verified where cited

Bare `Error` for operator refusals in `src/grade-agreement.ts:170-184,266-269`;
the agreement guard
checks count but not index presence (`:238-268`);
sheet items indexed by position,
not by printed
number (`src/grade-sheet-read.ts:252-265`);
the scorecard prints 0 for an empty denominator
(`src/scorecard.ts:397-399,447-452`);
an internal invariant thrown as bare `Error`
(`src/consolidate-driver.ts:322-326`);
`describeAbandon` falling back to `String(error)` into a warn
line (`src/abandon-kind.ts:76-77`);
four prose drifts;
`consolidate-produce.ts` and `tally-claim.ts`
reached only through callers;
a gate ballot lost whole when one list field is not an array
(`src/consolidate-gate-wire.ts:132-146`).
The reviewer also answered calibrate's open question:
`candidate-select.ts:159` sets `judges` to the
SEATED roster,
so a half-dark panel is correctly excluded by the window trial's full-panel filter.
Open question it raised for the main session:
the consolidation gate's bar is absolute
(`CONSOLIDATE_GATE_QUORUM = 2`,
`HEARD_NEEDED = 2`),
so two of ten ballots against one can replace a
memorial page's wording and be cached as settled;
whether that bar is intended is recorded here,
undecided.

FIXED 2026-08-26 in `f83e9b449`,
with the guards in `c27dfa297`.
consolidate-2:
the pre-grade parser and the agreement guard refuse as `StatedRefusalError`,
naming positions and
counts;
the unknown verdict is no longer quoted,
since the rule that keeps text out of a refusal goes by class.
consolidate-3:
the agreement guard refuses a pre-grade file that misses any sheet position,
count aside.
consolidate-4:
the sheet reader parses the printed item number and refuses when it disagrees with the position.
Guards:
the presence loop removed fails `REFUSES a pre-grade file that misses a sheet position`;
the heading check
disabled fails `REFUSES a sheet whose printed item number disagrees with its position`;
a non-array pre-grades file
is refused as a stated refusal (case added);
each restored,
passes.
consolidate-5:
`recall-benchmark` refuses,
after keeping its record,
to print rates over no dispatched entries or no
planted seeds,
so a zero denominator never prints as a zero rate;
the scorecard's fields are unchanged.
No unit guard
(command-level);
the refusal names both counts and the kept path.
consolidate-6:
the contested-slice invariant is `ConsolidationLedgerGapError`,
marked and in the inventory.
Guard:
the marker removed fails the two inventory cases;
restored,
passes.
consolidate-7:
`describeAbandon` names an unfamiliar class through `refusalText` instead of rendering its text.
Guard:
`String(error)` restored fails `NAMES THE CLASS of anything it does not recognise`;
restored,
passes.
consolidate-8:
the driver's TSDoc says the producers are asked at every slice the contest was asked about,
the
selection header lists total resolution in the order `compareCandidates` reads,
`consolidateSliceKey` documents both
neighbouring texts,
and `quoteLine` and `renderCandidate` carry examples.
Prose only.
consolidate-9:
`consolidate-produce.unit.test.ts` drives the producing half on its own with a scripted transport:
a proposal refused for shape is sent back to its author and both verdicts are reported,
a valid slate asks nothing
more and reports the same verdict twice,
and the repair's findings follow the gather's.
Guard:
the repair round bypassed fails `SENDS a proposal refused for shape back to its author`;
restored,
passes.
consolidate-10:
a gate ballot whose `unsupported` or `dropped` is not a list keeps its choice and reads the field as
empty;
the wire type carries the fields as `unknown` and the reader narrows.
Guard:
the list checks restored in the guard fail `KEEPS the choice and reads the field as an empty list`;
restored,
passes.
The open question the reviewer raised (whether the gate's absolute bar of two ballots is intended) stays recorded above
and undecided;
it is a design question,
not a defect this pass lands.
