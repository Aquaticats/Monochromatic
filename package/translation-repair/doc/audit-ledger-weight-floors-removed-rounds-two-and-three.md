# Audit ledger: the weight floors removed, rounds 2 and 3

Rounds 2 and 3 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
which holds B900 to B906:
the owner's answers of 2026-10-25 ("Allow two checkers",
"Keep waiting for half",
"Two judges must name it",
"Ship the slate's wording" and
"Ship the slate's pick (Recommended)"),
the lead's rulings on the findings of the independent re-checks,
the cache versions and the replays of `XingZ6021`.
The rulings are quoted in
[votes, repairs and run-offs](design-commitments-votes-repairs-and-run-offs.md)
and referenced from
[shipping and the archive](design-commitments-shipping-and-the-archive.md).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B907: the checker count of three became two (round 2)

Status:
fixed on 2026-10-25 (UTC),
red `e0f9ef0b42`,
fix `d902da3ffe`.

The owner's answer of 2026-10-25 to the checker count that B906 left open:
"Allow two checkers".
`MINIMUM_CHECKER_COUNT` in `assembly/src/repair-contract.ts` is 2,
and `assertCheckerQuorumReachable`,
`CheckerQuorumError` (its message names "a floor of 2" and why:
resolution needs two answering checkers),
the TSDoc and the comments in `run/src/corpus-run/run-seats.ts` and `run/src/corpus-run/run-config.ts` changed together.
The red cases (`assembly/src/checker-roster.unit.test.ts`) accept two checkers,
refuse one and an empty roster,
and refuse one checker that may also self-certify.
The count is a run guard and not an input of any cache key.
The historic text of `audit-ledger-which-seats-a-quorum-counts.md` still says three,
as it was true then.

## B908: a declined round ships a candidate only when two judges named it (round 2)

Status:
fixed on 2026-10-25 (UTC),
red `d8e239c806`,
fix `9622024d5d`.

The owner's answer of 2026-10-25,
"Two judges must name it",
to the preference that B903 and B906 left open.
Before:
a challenge round that declined with nothing to ship past (a withheld standing,
no run-off left)
shipped a candidate by a fixed order,
repair lane,
then translate lane,
then slate order,
whatever any judge named
(`shipPreferredPastDecline` in `translate-stage/src/translate-runoff-tie.ts`).
In `XingZ6021` this fired five times,
all in consolidation slates that heard one judge of seven
(slices 13,
14,
33,
44 and 84,
log lines 17830,
17911,
18793,
19401 and 20274),
each after `winner was named by 1 ballot against a floor of 2`.

The rule now:
the preference ranks only the candidates that two or more judges named,
and ships the best of them.
When none qualifies,
the wording in place stays,
with the finding `translate-slate-declined-archive-kept` and a warning in the log,
and where no wording exists the source text ships under the note
`This passage has not been translated yet. The original text follows.`
by the existing path.
The sheet text the judges read (`SHIPS_WHAT_TWO_JUDGES_NAMED` in `adjudication/src/select-decline-consequence.ts`,
formerly `SHIPS_BY_PREFERENCE`) says so.
The qualifier branch is nearly unreachable from a real tally
(a leader with two ballots is normally selected before a decline),
so direct tests of `settleAbsentDecline` pin it,
beside five judging-level cases.

Not changed in this entry,
a path of the same class that the owner's answer did not name:
a challenge round whose top weight is tied across every offered candidate shipped one by the fixed preference
(`breakRunoffTie`,
the tenth addendum of `doc/decision/translation-repair-ineligible-standing.md`),
and a control test pinned it.
The lead ruled afterwards that it waits for two judges too,
and B914 records the change.

A test slip,
recorded:
the first version of the "several qualify" direct case gave all candidates equal weight,
so `breakRunoffTie` fired and the case failed in the red commit for the wrong reason;
the fix commit gave one candidate a lower weight.
The red commit message also says "Seven cases ... five fail" where the file holds ten cases and five fail.

## B909: an unheard gate over a refused or flawed standing ships the pick two slate ballots chose (round 2)

Status:
fixed on 2026-10-25 (UTC),
red `371977693b`,
fix `b2dd131f80`;
widened to a flawed-majority standing in the addendum,
red `728a164502`,
fix `d8e1674fbf`
(the cache bump of that series,
`784efa1591`,
stands on the tie rule alone,
B912).

The owner's answer of 2026-10-25,
"Ship the slate's wording" (finding F5):
when the gate hears fewer than two usable answers over an archive the deterministic rule refused,
the slate's pick ships if two or more slate ballots chose it,
rather than falling to the archive that failed the rule.
`shipPastForfeitStanding` takes `slatePickBallots`
(`slatePickBallotsOf` reads the slate round's row for the shipped position),
and ships `consolidated` with the finding `unheard-gate-ships-slate-pick`.
A pick one slate ballot chose keeps the standing.
A gate that heard two usable answers is unchanged.

The replay does not support one expectation,
for the lead (B913):
the five slices of `XingZ6021` (13,
14,
33,
44 and 84) did not have a slate pick chosen by two ballots.
Each slate round heard one judge of seven and recorded
`winner was named by 1 ballot against a floor of 2`,
then the preference shipped a candidate the lone judge had not even named in slice 13.
Under both of the owner's answers a pick that one judge alone chose cannot ship,
so these slices keep the archive text (`no-valid-wording`) as in round 1.
Slice 101,
which drew two usable gate ballots for `consolidated`,
still ships.
The artifact does not record the slate's ballots,
so the log is the evidence.
The brief for finding F5 rested on a wrong premise taken from the re-check,
"the slate's two or more ballots":
the five slices had one slate ballot each,
and the lead confirmed that under "one judge alone never decides" they keep the archive text.

The flawed-majority reading,
resolved in the addendum.
Round 2 had adopted and flagged a narrower reading:
a standing that passes the rule but was called flawed by a contest majority kept the standing
when the gate heard fewer than two,
since the owner's answer spoke of the archive that fails the publication rule.
The owner answered on 2026-10-25,
"Ship the slate's pick (Recommended)":
when a majority of the contest judges found the eligible standing flawed,
the slate's pick had two or more slate ballots,
and the gate got fewer than two usable answers,
the slate's pick ships.
`shipPastForfeitStanding` now reads `slatePickBallots` for either kind of forfeit standing,
and the log line names which kind it was.
The finding `unheard-gate-ships-slate-pick` and its text cover both.
A pick of one slate ballot keeps the standing in both kinds,
and an eligible standing no contest majority called flawed never reaches the rule.
The red case that pinned the narrower reading was replaced
(pick of two and three slate ballots with one and no usable answers ship `consolidated`),
with controls for a pick of one ballot and for a standing no majority called flawed.
`CONSOLIDATE_CACHE_VERSION` moved to 37 in the same series,
on the tie rule alone (B912).

What the owner should see in this entry (findings F3 and F4 of re-check 2):
- The five slices of `XingZ6021` (13,
  14,
  33,
  44 and 84) keep a rule-failing archive,
  and the page reports it (finding `no-valid-wording`,
  terminal `slate-declined-standing`).
  That is what the owner's two answers together require,
  since the picks drew one slate ballot,
  and it differs from the brief's expectation that those slices ship the slate's proposal again.
- The branch "unheard gate keeps the standing" (`UNHEARD_GATE_KEEPS_STANDING_FINDING`,
  a pick of one slate ballot) is a guard that the stage cannot reach on a fresh decision:
  a fresh slate decision carries at least two ballots now
  (selected,
  tie-broken and shipped-past-decline all need two),
  so a pick of fewer than two never arrives at `shipPastForfeitStanding`.
  The function's tests pin it in isolation,
  and the gate replay row that shows it holds a hard-coded one ballot taken from the log.
- A slate pick shipped past an unheard gate (`UNHEARD_GATE_SHIPS_SLATE_PICK_FINDING`) is never persisted,
  because `consolidate/src/consolidate-persistence.ts` stores a gated settlement only when the gate heard
  `CONSOLIDATE_GATE_QUORUM` usable ballots and refuses an archive-kept one.
  The pick ships on the page of that run and the slice is asked again on the next run,
  which costs one re-ask per run
  and is consistent with the comment of that file,
  that a settlement made on a thin bench is the hour and not the slice,
  so a later pass asks the slice again.
  For the same reason no stored record stands behind the cache bumps for the unheard gate (B912).

## B910: a page title settles only on two or more heard voices (round 2)

Status:
fixed on 2026-10-25 (UTC),
red `fd07e7fc49`,
fix `cf2f03c5d2`,
cache version `c52409ad5a`.

The lead's ruling of 2026-10-25 on finding F3 of the independent re-check.
`settlePageTitles` (`artifact/src/page-title-lexicon-stage.ts`) took the leader of any voice count,
so a bench of seven that lost six seats for the title round settled,
and `passPageTitles` (`corpus-pass/src/corpus-run/pass-page-titles.ts`) stored,
the one voice's rendering as a line the translators and judges read.
Now a round that heard fewer than `MIN_STAGE_VOICES` (2) voices settles nothing,
logs why,
adds a finding saying every repeated title keeps its current rendering,
and stores nothing,
so a later run asks again.
`PAGE_TITLE_CACHE_VERSION` was 7 here.
This entry left a tie among three or more heard voices to the earliest seat on the roster,
and a title one voice alone rendered still settled when two or more were heard.
Re-check 2 (finding F1) showed that this met the lead's words and missed the owner's rule,
and B915 closes it.

## B911: the consistency round decides only on two or more answering ballots (round 2)

Status:
fixed on 2026-10-25 (UTC),
red `7faf9a705a`,
fix `f69a6bab89`,
cache version `9445a2c28f`.

The lead's ruling of 2026-10-25 on finding F4.
A gather on a seated bench of one or two seats closes on a quorum of one by design
(`stage/src/roster-quorum-size.ts`),
and `winnerOf`,
`countConsistency`,
`carryChoiceOf` and `runOffWinnerOf` then decided on one usable ballot.
`decidingQuorumOf` (`consistency/src/consistency-deciding-quorum.ts`) raises the quorum a count applies to two,
and a count below it changes nothing.
`reachableQuorum` and every wait are unchanged.
`CONSISTENCY_CACHE_VERSION` is 9.

Left alone,
by the lead's ruling:
the derivability probe (`repair/src/derivability-probe.ts`)
and the restoration judge (`repair/src/restoration-judge.ts`)
decide a seed on one cast verdict once their quorum is met,
and feed the benchmark scorecard only (`repair/src/repair-scorecard.ts`),
so no page changes with them.
The archive block review and the coverage verdict were checked in the re-check and are not lone-voice paths.

## B912: the cache versions of round 2

Status:
fixed on 2026-10-25 (UTC),
one commit per stage.

The same account as B904:
the key hashes the version,
the shape of the run and the texts,
not the rule that reads the ballots.
- `TRANSLATE_SLICE_CACHE_VERSION` 28 to 29 in `1c72668739`:
  a declined round ships a candidate only when two judges named it.
- `CONSOLIDATE_CACHE_VERSION` 35 to 36 in `5828f9efe6`:
  the same rule on the consolidation slate.
  The commit message and the first text of this entry named the unheard gate over a refused archive as a second reason;
  that part is withdrawn (finding F4 of re-check 2),
  since no settlement made over an unheard gate is stored (B909).
- `PAGE_TITLE_CACHE_VERSION` 6 to 7 in `c52409ad5a`:
  a title settles on two or more heard voices.
- `CONSISTENCY_CACHE_VERSION` 8 to 9 in `9445a2c28f`:
  a count decides on two or more usable ballots
  (a stored answer holds the heard voices and the quorum and the count is read from them,
  so the bump is the lead's ruling and the namespace discard is the other brace).

Moved again in the addendum,
one commit per stage:
- `TRANSLATE_SLICE_CACHE_VERSION` 29 to 30 in `21cda9419e`:
  a tie across every candidate ships by preference only among candidates two judges named (B914).
- `CONSOLIDATE_CACHE_VERSION` 36 to 37 in `784efa1591`:
  the same tie rule on the consolidation slate (B914).
  The flawed-majority gate answer (B909) is not a reason:
  a settlement made over an unheard gate is never stored,
  because `consolidate/src/consolidate-persistence.ts` refuses an archive-kept settlement
  and stores a gated one only when the gate heard `CONSOLIDATE_GATE_QUORUM` usable ballots.
  The bumps stand on the decline and tie rules:
  a stored settlement with a gate of two or more usable ballots can hold a candidate the fixed order shipped.
  The key needs it because a stored settlement holds the wording that shipped,
  and the same texts now ship another;
  the numbers 29 and 36 were set a few commits before and whether a record stands at either is not known,
  so the bump is the safe side and its cost is one re-run of the affected slices.
  The doc comments of versions 36 and 37 in `artifact/src/consolidate-key.ts` were corrected in `7cb19250db`.

Moved again in round 3,
one commit per stage:
- `PAGE_TITLE_CACHE_VERSION` 7 to 8 in `073ad8bfc0`:
  a title settles only on a rendering two or more voices gave (B915).
- `TRANSLATE_SLICE_CACHE_VERSION` 30 to 31 in `8c8331da79`
  and `CONSOLIDATE_CACHE_VERSION` 37 to 38 in `7cb19250db`:
  the reworded sheet text about a declined round over wording that cannot ship (B916).
  By the repository's convention a change to a sheet's wording moves the version of every key whose records hold
  ballots cast under it.
  The repair slice (47) and refine (17) versions stay,
  because neither reads this sentence (`declineConsequenceFor` is read by the translate and consolidation slates only).

Stay,
with the reason:
the repair slice (47) and refine (17) versions
(neither stores a slate shipped by preference or a title or consistency outcome),
the lane contest (unchanged rule),
the pairing version and the artifact generation.
`MINIMUM_CHECKER_COUNT` is a run guard and not a key input.

Checked by `git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 53b55cf5c3..HEAD`,
which lists exactly those nine commits
(the six of round 2 and its addendum,
and the three of round 3),
and by the positive control over `c265e8df95~1..c265e8df95`,
which lists `c265e8df95`.

## B913: the replay of XingZ6021 after round 2

Status:
measured on 2026-10-25 (UTC),
keyless,
no provider called,
scripts and output under `fix2/replay/` of the scratch directory.

The panel tally and the downstream replays are byte-identical to round 1 (`diff` of `replay-tally.out`
and `replay-downstream.out` finds no difference):
295 `needs-human` claims at the base,
55 now;
234 issues at the base,
49 now;
slice 67 nine claims;
slices with a dispute 3 at the base,
26 now;
envelopes 18 at the base,
78 now.
Round 2 touches no panel,
editor or archive-dispute rule.

The gate replay changed in what it reads and not in what it settles:
- Slices 13,
  14,
  33,
  44 and 84 (one usable gate ballot,
  slate pick of 1 ballot from the log):
  the base shipped `consolidated`,
  round 1 kept the standing (`unheard-gate-keeps-standing`),
  round 2 keeps the standing.
  Terminal `consolidated` on the gate path:
  6 at the base,
  1 at round 1,
  1 now.
- Slice 101 (two usable ballots for `consolidated`) ships at the base,
  at round 1 and now.
- Positive control for the new branch:
  the same gate with a slate pick of 2 ballots ships `consolidated` in all six slices,
  so the replay can show a ship when the slate's pick is two-ballot.
- The five translate-stage preferences that shipped a declined slate in the whole run
  (`rg` for `challenge round declined .* shipping candidate` finds five lines,
  all in the five slices named in B908)
  ship nothing now,
  because each pick was named by one ballot.
  The archive text stays in those five slices,
  as it did after the gate in round 1,
  and the gate is no longer asked for them.

The evidence that the five slices had one slate ballot each,
from `corpus-pass-XingZ6021.log` in the scratch directory:
- Slice 13:
  line 17690 (`select round: 1/7 heard ... no quorum (1 of 2 needed)`),
  line 17828 (`winner was named by 1 ballot against a floor of 2`),
  and the preference shipping at line 17830.
- Slice 14:
  lines 17774,
  17909 and 17911.
- Slice 33:
  lines 18557,
  18791 and 18793.
- Slice 44:
  lines 18991,
  19399 and 19401.
- Slice 84:
  lines 20190 and 20269 (both rounds heard one judge of seven),
  with the decline of the rejection kind and the preference shipping at line 20274.
- The five `challenge round declined .* shipping candidate` lines are the only ones in the run.

The premise of the F5 brief,
taken from the re-check,
was "the slate's two or more ballots".
The log shows one slate ballot in each of the five slices,
so under "one judge alone never decides" they keep the archive text,
and nothing changes there.
The lead confirmed it.

The addendum reran the three replay scripts on the final tree.
The panel tally and the downstream replays are byte-identical to the figures of this entry (`diff` finds no difference),
and so is the gate replay for every one of those figures:
slices 13,
14,
33,
44 and 84 keep the standing and slice 101 ships,
so no figure moved.
It could not move:
the five standings were ineligible and not flawed by a majority,
the pick had one slate ballot,
and the tie path has no recorded instance in the run
(`rg --count "challenge round tied at weight"` finds none in the log,
where the same log yields five `challenge round declined` lines for the positive control).
The replay script gained a what-if for the flawed-majority rule:
the same five gates over an eligible standing a majority called flawed keep the standing with the pick from the log
(one ballot) and ship `consolidated` with a pick of two slate ballots,
and slice 101 ships either way.

## B914: a tie across every offered candidate ships only among the candidates two judges named (addendum)

Status:
fixed on 2026-10-25 (UTC),
red `b6e39d3148`,
fix `17b86eee0e`,
cache versions `21cda9419e` and `784efa1591` (B912).

The lead's ruling of 2026-10-25,
settled by the owner's "Two judges must name it".
The tie of a challenge round across every offered candidate
(`breakRunoffTie` in `translate-stage/src/translate-runoff-tie.ts`,
class 174,
TianqiChen6669 slice 4)
shipped one by the fixed preference of repair lane,
then translate lane,
then slate order,
whatever any judge named.
B906 and B908 left it outside the owner's answer.
That tie is a round in which no judge pair backed any candidate whenever each tied candidate drew one ballot,
which is the shape the owner's rule refuses.

The rule now:
the preference ranks only the tied candidates that drew two or more ballots,
and ships the best of them
(basis `repair lane`,
`translate lane` or `slate order`,
as before,
with the finding `translate-runoff-tie-broken`).
If no tied candidate drew two ballots,
`breakRunoffTie` returns `unbroken`,
and `settleAbsentDecline` takes the path of `shipPreferredPastDecline`:
the wording in place stays with the finding `translate-slate-declined-archive-kept` and a warning in the log,
and where the caller ships nothing past a decline or the passage has no wording the absence is raised as before,
which leaves the passage to the source-text path under the untranslated note.
A tie that some candidates lost and others drew equally is still narrowed by the further run-off,
and a first round is still challenged.

The tests:
the control of `translate-ship-past-decline-two-judges.unit.test.ts` that pinned the old behaviour
("STILL BREAKS A TIE ACROSS EVERY FINALIST BY PREFERENCE",
two finalists one ballot each shipping the repair lane's text)
was replaced by one that expects the archive wording to stay,
beside direct cases of `settleAbsentDecline`:
a three-way tie of one ballot each keeps the archive,
the same tie raises the absence where the caller ships nothing past a decline,
a tie of two ballots each still ships the repair lane's text (a positive control that passes on the old rule too),
and a tie where only the translate lane's text drew two ballots ships it,
not the repair lane's text.
Four cases failed on the red commit.
Three judging-level cases of `translate-retry.unit.test.ts` and one of `translate-runoff.unit.test.ts`
pinned a one-ballot tie shipping by order over an absent incumbent
(the repair lane's text,
the translate lane's text and slate order);
they now expect the absence,
and the case with an incumbent (keeping it) is unchanged.
No tie of two or more ballots each can arise at the judging level on a bench of three,
so the direct cases carry that branch.

What the run showed:
the log of `XingZ6021` holds no tie-path line (B913),
so no figure of the replay moves.
The five preferences it does hold came from declined rounds and were handled in B908.

## B915: a page title settles only on a rendering two or more voices gave (round 3)

Status:
fixed on 2026-10-25 (UTC),
red `71a11223cc`,
fix `ae68de2303`,
cache version `073ad8bfc0`.

The lead's ruling of 2026-10-25 on finding F1 of re-check 2,
settled by the owner's rule that one judge alone never decides,
so no question went to the owner.
B910 stopped a round that heard fewer than two voices,
but `mostGiven` still returned the leader of any count,
so a title settled on one voice's rendering when two or more were heard:
two of seven seats answered and one of them gave an empty or zero-width rendering,
or the two answered with a different rendering each and the earliest seat won the tie.
The title was cached and read by translators and judges as an identity-context line.

The rule now:
`settlePageTitles` keeps a title only when `chosen.voices >= MIN_STAGE_VOICES`.
A title that one voice alone rendered,
or that tied at one voice apiece,
stays unsettled,
and a finding says how many titles were left
(`page title lexicon left N repeated titles unsettled, since no rendering was given by 2 voices`).
A tie of two voices apiece or more is still broken by the earliest seat,
which is a tie of two judges or more.
`corpus-pass/src/corpus-run/pass-page-titles.ts` needed no change:
it stores the titles the stage returns.
`PAGE_TITLE_CACHE_VERSION` is 8 (B912).

The tests:
the pins of `page-title-lexicon.unit.test.ts` at the lines of "BREAKS A TIE BY THE EARLIEST SEAT"
and of the zero-width case,
which settled on one voice,
were replaced by cases expecting the title unsettled with the finding.
New cases:
a title one voice of three rendered,
a tie of two voices apiece on four seats that the earliest seat breaks (the positive control),
and a round where one seat shows nothing and two seats give words (the title settles on two voices).
The red commit fails three cases (the tie of one voice apiece,
the one voice of three and the padded round) and its two-voice controls pass.

The same shape elsewhere,
checked and found absent:
the lead asked whether the consistency carry and the run-off decide on one named winner where several were heard.
They do not.
`winnerOf`,
`carryChoiceOf` and `runOffWinnerOf` name a winner only on a strict majority of the usable ballots
(`majorityText` in `consistency/src/consistency-majority.ts` needs more than half of the usable ballots),
and the quorum is raised to two (`decidingQuorumOf`).
A strict majority of two or more usable ballots is two ballots or more,
so no winner rests on one ballot,
and nothing needed fixing.
The finalists of a run-off are named from the same usable ballots and decide nothing alone.

## B916: the sheet text about a declined round over wording that cannot ship contradicted itself (round 3)

Status:
fixed on 2026-10-25 (UTC),
red `acb4c4b413`,
fix `beae57ce8a`,
cache versions `8c8331da79` and `7cb19250db`.

Finding F2 of re-check 2.
`SHIPS_WHAT_TWO_JUDGES_NAMED` (`adjudication/src/select-decline-consequence.ts`) is the sentence the judges of the
translate lane's and the consolidation's slates read about what declining does over a withheld standing.
It said that declining does not keep the wording as the shipped text,
and,
in its last clause,
that the archive's wording stays.
A judge told two things about the cost of declining is asked for caution by a promise that contradicts itself.

The sentence now says what happens in order:
the wording in place cannot ship unreported,
so declining is not a quiet keep;
a slate declined in every round ships a candidate only when two or more judges named it,
chosen among those by the fixed order;
and when no candidate was named by two judges that wording stays and the page reports it.
The module had no unit test of its own.
`select-decline-consequence.unit.test.ts` pins the order of the three statements,
the absence of the old clause and the three consequences by caller.
The rendered-sheets fixtures of `repair` import the constant and follow it without an edit.
By the repository's convention a change to a sheet's wording moves the cache versions of the keys whose records
hold ballots cast under it,
so the translate slice version is 31 and the consolidate version is 38 (B912).
