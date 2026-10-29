# Consolidation refuses to ship a standing the deterministic gate rejected

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the fifteenth addendum ("Preference + polish") no longer ships a declined slate's preferred candidate;
a candidate ships only when two or more judges named it,
and otherwise the archive stays.
The tenth addendum's tie across every finalist (class 174) follows the same rule by the lead's ruling of 2026-10-25:
the fixed order ranks only the tied candidates two or more judges named,
and otherwise the wording in place stays.

Decided by the owner on 2026-09-04 ("Yes:
prefer the best valid proposal,
else fail the slice at once"),
asked with two options after the luxuanwen3 pass of that day.

## What happened

- luxuanwen3's archive front matter broke the identity rule in `validateFrontMatterTranslation` (see
    `translation-repair-front-matter-guard.md`,
    addendum of the same day).
    The translate lane kept the
    archive-shaped text,
    so the consolidation's standing text was ineligible from the start.
- `consolidate-driver.ts` logged `slice 0: consolidation standing text fails publication eligibility and
    remains retryable`,
    then bought its single attempt:
    the slate carried two valid proposals,
    the judges
    endorsed the standing over them (three ballots calling the archive's shape "the declared translated
    identity"),
    and `consolidate-slice-buy.ts` shipped it "with the finding recorded".
- `assertFrontMatterComplete` refused the assembled page (`invalid-page`) 63 minutes and 2.61 USD later:
    `TALLY luxuanwen3 status=INCOMPLETE`,
    no page (`~/temp/agent/luxuanwen3-shapes-20260904.log`).
- The same shape awaits every entry whose archive metadata breaks a structural rule the page guard
    enforces,
    the 15 archives naming the directory id among them.

## The rule

- The driver's `standingValid` (the deterministic `validateTranslatedSlice` verdict on the standing text)
    reaches the settlement as `standingEligible`,
    separately from `standingMayShip`,
    which is contest
    endorsement.
- When the standing is ineligible,
`settleConsolidation` withholds it from the slate:
the judges are asked
    with `incumbentKind: 'absent'` and no incumbent candidate,
    so they choose among the valid proposals or
    decline.
    The settlement records `ineligible-standing-withheld` among its findings.
- Every exit that would keep the standing throws `ConsolidationStandingIneligibleError` naming the slice
    and the terminal:
    an empty floor (`incumbent-only`),
    a judges' decline (`slate-declined-standing`,
    re-raised from the judge's absence error with that error as the cause),
    and a gate that keeps the
    standing (`gate-kept-standing`).
    A `consolidated` terminal ships as before.
- The entry stops at that slice,
before the remaining slices are bought.

## What this is not

- Not the no-loop decision reopened.
The single attempt of `consolidate-slice-buy.ts` stays for a standing
    that merely lacks contest endorsement:
    that standing has passed the deterministic gate,
    and quality
    machinery may not withhold the entry over it.
    An ineligible standing was never going to ship.
- Not a second judged round.
Preferring the best valid proposal is the judges' existing choice over a slate
    without the incumbent,
    bought once.

## What landed

- `consolidate-ineligible-standing.ts` (error class,
`slateIncumbentFor`,
`requireShippableTerminal`),
    threaded through `consolidate-slice-buy.ts` and `consolidate-driver.ts`;
    `consolidate-settle.ts` split
    its context and its gate-and-ship halves into `consolidate-settle-context.ts` and
    `consolidate-settle-gate.ts` at the line cap.
- Guards in `consolidate-settle.unit.test.ts` (withheld standing ships the proposal;
    gate-kept,
    declined and empty-floor exits throw) and `consolidate-driver.unit.test.ts` (a repair-lane standing that dropped a
    source destination now fails the slice,
    since the publisher refuses such an entry through `DroppedDestinationError`),
    shown to fail with the withholding and the refusal neutralised.

## Option rejected

- Keep the late refusal at the page guard:
    no code,
    but every such entry pays a full run before failing,
    which is what luxuanwen3 did.

## Addendum 2026-09-09: a valid incumbent is kept where the standing is ineligible

Decided by the owner on 2026-09-09 ("Keep the incumbent"),
asked with two options after the sixth `Mio` on Bedrock alone:
at slice 3 the translate lane's standing failed the deterministic gate,
the archive's paragraph and list for that slice were valid and were never offered,
and the entry stopped under the rule in "The rule".

- The consolidation reads the gate's verdict on the incumbent
    (`row.incumbentText`,
    the page text the slice replaces)
    beside the standing's.
- When the standing is ineligible and the incumbent is eligible,
    the slate offers the incumbent in the standing's place (`incumbentKind: 'present'`),
    the settlement records `ineligible-standing-replaced-by-incumbent`,
    and every exit that kept the standing keeps the incumbent instead:
    the empty floor,
    the judges' decline and the gate's refusal.
- When both are ineligible,
    or the standing is the archive's own text (luxuanwen3,
    where the two are one),
    the rule of 2026-09-04 stands and the entry stops.
- The reason of 2026-09-04,
    that an ineligible standing was never going to ship,
    does not reach an incumbent that can.
    The option rejected,
    stop as now,
    costs the page over one slice whose safe text exists.
- What landed:
    `consolidate-standing-verdict.ts` reads the incumbent's verdict and returns the wording the settlement runs
    against (`settlementText`),
    the replacement finding and `incumbentStandsIn`;
    `consolidate-driver.ts` runs the settlement and its cache key against that wording and carries the finding
    through `consolidate-slice-buy.ts` into the judged round;
    the artifact's shipped kind `incumbent` (`artifact-two-lane-consolidate.ts`,
    read by `artifact-two-lane-read-consolidate-parts.ts`) says a kept standing is the incumbent's text,
    and `would-ship-text.ts` writes it,
    since a bare "unchanged" would have the page assembled from the lane's wording the gate refused.
- Guards shown to fail with the replacement neutralised:
    `consolidate-standing-verdict.unit.test.ts` (stands in;
    both fail;
    the standing is the incumbent),
    `consolidate-driver.unit.test.ts` ("KEEPS A VALID INCUMBENT",
    beside the 2026-09-04 case whose incumbent fails too and still stops),
    `artifact-two-lane-read-consolidate.unit.test.ts` and `would-ship-text.unit.test.ts` for the shipped kind.

## Addendum 2026-09-18: a gate that settles on neither over an ineligible standing ships the slate's choice

Taken under the rule of 2026-09-04 itself
("prefer the best valid proposal,
else fail the slice at once"),
after XingZ604 stopped INCOMPLETE at 4h53m on its slice 13.

- The archive's funeral paragraph and the contest winner both carried the original's neutral pronoun
    untranslated,
    so the standing was ineligible and no incumbent could stand in
    (the 2026-09-09 addendum's "both are ineligible" case).
- The slate,
    with the standing withheld,
    chose a valid proposal 3 of 4.
    The consolidate gate went 5 of 7 usable with neither rendering at quorum over the other,
    and the gate's tie rule kept the standing;
    `requireShippableTerminal` refused `gate-kept-standing` and the entry stopped.
- The gate's indecision is not a refusal.
    Its fallback to the standing is a conservative default,
    and an ineligible standing offers nothing conservative to keep;
    stopping there discards a valid text the judges endorsed,
    which is the opposite of the rule's first clause.
- `shipPastUndecidedGate` (`consolidate-ineligible-standing.ts`) now resolves a `neither` verdict toward
    the consolidation when the standing is ineligible,
    recording `undecided-gate-ships-proposal` on the settlement,
    and `gateAndShip` wraps and ships that outcome.
    A gate that refuses the consolidation at quorum still keeps the standing and still stops the slice
    (superseded by the thirteenth addendum);
    an eligible standing keeps the slice on indecision exactly as before.
- Guard shown to fail first (`b8b32e28d`):
    `consolidate-settle.unit.test.ts`
    ("SHIPS THE PROPOSAL THE SLATE CHOSE when the standing is ineligible and the gate settles on neither",
    with the eligible-standing contrast in the same case).
    Fixed in `0d0747200`.

## Addendum 2026-09-18, second: a tied slate over an ineligible standing is challenged once

Taken under the same rule after XingZ605 stopped INCOMPLETE at 4h08m on the same slice 13,
this time at the slate:
four valid proposals,
the judges 2 to 2,
and the decline named `slate-declined-standing`.

- The translate lane has challenged a declined slate once under `decline-challenge` since class fifty-three,
    narrowing a tie to the candidates that drew a ballot (`judgeSlateWithRetry`).
    The consolidation called the judge directly and gave the slice up on the first decline.
- `settleConsolidation` now routes the slate through the same challenge when the standing is ineligible,
    because there a decline stops the entry;
    an eligible standing keeps its single round,
    because there a decline keeps text the contest endorsed.
- A slate declined twice still ends the slice as before.
- Guard shown to fail first (`c6ec06788`),
    fixed in `adca69d4e`.

## Addendum 2026-09-28, nineteenth: no accepted claim settles at neutral

Decided for quality under the owner's standing directive of 2026-09-28
("prefer quality of the end result"),
by the whole-package audit
(`package/translation-repair/doc/audit-ledger.md`,
L5).

- Neutral is the severity that asserts no defect.
    The tally now holds an acceptance the panel settled at neutral as needs-human,
    so it disputes no archive.
- The seventeenth addendum's "an accepted `accuracy/addition` claim still disputes at any severity" stands as worded;
    since no accepted claim settles at neutral any more,
  in effect an addition disputes from minor up.
    Five accepted neutral additions over every run could dispute an archive before.
- The critic and panel sheets now define the severity scale
    (minor,
  major and critical after MQM),
    which the eleventh addendum's major-or-worse rule reads;
    the effect on how many slices it disputes is measured on the next run.

## Addendum 2026-09-27, eighteenth: a slice with no valid wording keeps the archive, and the page ships

Owner's answer of 2026-09-27 ("Keep archive,
ship"),
asked by the whole-package audit
(`package/translation-repair/doc/audit-ledger.md`,
E4)
after the ruling that a run always ships.
It replaces the "else fail the slice at once" half of the rule of 2026-09-04;
the first half stands,
so a valid proposal is still preferred wherever one exists.

- A stopped entry left the archive's whole page live,
    that slice included,
    so stopping protected nothing the page did not already show
    and discarded every other repair the entry had bought.
- Measured before the answer:
    18 real entries stopped on `ConsolidationStandingIneligibleError` from 2026-09-04 to 2026-09-27,
    about 34.3 hours in all,
    most on terminals later addenda had already turned into shipping the slate's choice;
    on the build of the answer,
    one real stop was of the kind still reachable
    (yulianNyanner,
  `incumbent-only`,
  0.48 hours).
- Every exit that stopped the entry now keeps the archive:
    the empty floor,
    the missing standing text with no lane text (ledger E4),
    a slate that reached the judges with nothing on it,
    and a gate leaving the refused standing in place.
    The settlement keeps its terminal,
    is marked `archiveKept`,
    records `no-valid-wording`,
    runs no polish,
    and is never cached.
- The artifact's shipped kind `archive` carries no text:
    the page reads the archive from the comparison row,
    so a silent archive stays silent rather than an empty string written at an anchor.
    It takes precedence over an incumbent that stood in,
    since on a disputed slice the stand-in is the repair text the rule refused too.
- The page ships with the slice on its `DEFECTS` line (`no-valid-wording`).
- `ConsolidationStandingIneligibleError` and `requireShippableTerminal` are gone;
    `nothingValidShips` and `keepTheArchive` (`consolidate-ineligible-standing.ts`) take their place.
- Guards shown to fail first (`4acd8bd40`),
    fixed in `7f79ada48` (settlement),
    `002f21f43` (artifact and reading)
    and `e7d409fdd` (report).
- Checked at the user boundary the same day,
  spending nothing:
    a real `corpus-pass --only mikaela_khara` over a scratch copy of its artifact,
    with slices 11 and 13 set to the archive-kept record,
    republished the page with the archive's wording at both
    (the unedited page carries it at neither),
    and `verify-published` agreed 1 of 1 at the expected length.
    The agreement census over the 214 stored pages reads identically before and after the change
    (137 agree,
  77 disagree,
  none refused).
    The `DEFECTS` line itself is unit-verified only;
    the first live `no-valid-wording` line is to be read when a run prints one.

## Addendum 2026-09-27, seventeenth: the major-or-worse rule reads the panel's severity

Owner's answer of 2026-09-27 ("Adjudicated"),
asked by the whole-package audit
(`package/translation-repair/doc/audit-ledger.md`,
L9)
on the eleventh addendum's rule.

- `disputesArchive` in `archive-dispute.ts` read each member claim's severity as its critic filed it.
    The panel settles an issue's severity
    (`AdjudicatedIssue.severity`:
  the upper median of the member claims and the supported ballots' re-grades),
    and that is the severity the rule reads now.
- Measured on XingZ6014:
    slice 66 was disputed on a claim filed major that the panel settled minor,
    and slice 19 was not disputed though the panel settled its issue major.
- An accepted `accuracy/addition` claim still disputes at any severity.
- The ARCHIVE RENDERING DISPUTED note names each claim with the panel's severity,
    so the severity printed beside a claim is the one the rule read.
- Guard shown to fail first (`b15ba5464`,
  `archive-dispute-severity.unit.test.ts`),
    fixed in `6a0f68cec`.

## Addendum 2026-09-27, sixteenth: a disputed reading the repair did not fix leaves the slice no eligible standing

Owner's answer of 2026-09-27 ("No eligible standing"),
asked by the whole-package audit (ledger L2)
on the eighth addendum's stand-in.
The eighth addendum made the repair lane's winning text the stand-in for a disputed archive rendering.
Where the repair lane's accuracy patch lost at the checkers,
that text is the archive's own wording,
or a refinement of it that keeps the reading (the class one hundred eight shape);
the translate lane then kept it as its incumbent,
both lanes agreed on it,
and no contest ran.

- Measured over TianqiChen66616,
  TianqiChen66620,
  XingZ6014,
  shihai4h2,
  CuspariaKLSY11 and hulicaijia31
    (probe `~/temp/agent/audit-repair/dispute-buckets.mjs`):
    87 disputed slices.
    The stand-in was the archive's own wording on 59,
    a refinement of the archive after the patch lost on 23,
    and a selected patch on 5;
    the checkers confirmed a disputing issue resolved on 1,
    and that on one ballot (ledger L1).
- Reading confirmed by the owner on 2026-09-27 ("Issue by issue"):
    the question named the archive's own wording,
    and its reason
    (a reading the panel found wrong never ships because the fix lost)
    holds issue by issue.
    The stand-in therefore stands only where the checkers confirmed every disputing issue resolved in the repair lane's winning text
    and the lane did not withdraw the slice;
    everywhere else the slice has no eligible standing.
    On the measured runs that is 86 of 87.
- Translate lane:
    the archive's own wording is shown with the dispute note as evidence,
    never as a candidate or a fallback,
    the path an archive the deterministic floor refuses already takes.
    The refusals that keep the archive over a replacement
    (alignment,
  quote loss,
  declared names)
    have nothing to keep on such a slice,
    so the stage's text is the lane's text and the consolidation's floors judge it.
    A proposal identical to the disputed wording is withheld from the slate.
- Consolidation:
    a standing that is the disputed wording
    (the archive's own,
  or the repair lane's text)
    is refused with a refusal naming the dispute;
    the incumbent stand-in of 2026-09-09 does not apply,
    since the incumbent is that wording;
    neither is offered on the slate,
    so the tenth addendum's preference cannot reach it.
- A slate rejected twice there ships by the fifteenth addendum,
    on the translate lane as in the consolidation.
    Before this change a content slice whose archive could not stand stopped the entry on such a rejection;
    extending the fifteenth addendum to the translate lane was confirmed by the owner on 2026-09-27
    ("Ship by preference").
- Cost:
  more slices reach the consolidation.
- Landed:
    `archiveDisputesOf` reads `resolvedIssueIds` and the withdrawn slices into `ArchiveDispute.standInEligible`;
    `disputedWordingsOf` names the refused wordings
    (the archive's own always,
  the repair lane's text where it may not stand);
    `validateTranslatedSlice` refuses a text that is one of them in all but whitespace
    (`disputed-wording.ts`),
    so every existing floor path applies:
    the translate stage excludes the archive as incumbent,
    the repair turn sends a copy back instead of collapsing it into the incumbent,
    the translate floor withholds it,
    and the consolidation refuses it as standing,
  incumbent stand-in,
  lane offer and proposal.
    The translate lane's "keep the archive" refusals (alignment,
  quote loss,
  declared names)
    do not apply where the archive is refused.
- Both slice keys carry the dispute note,
    which neither did:
    a disputed slice whose incumbent is the archive would have shared its key with the same slice undisputed.
    `CONSOLIDATE_CACHE_VERSION` 19 and `TRANSLATE_SLICE_CACHE_VERSION` 14.
- Found on the way:
    the consolidation producers' repair turn re-checked a revision without the declared names its first check read;
    it now reads both.
- Guard shown to fail first (`23cbbdaaa`,
  `archive-dispute-standing.unit.test.ts`;
    the translate stage shipped the disputed wording the judges preferred).

## Addendum 2026-09-27, fifteenth: a slate rejected twice over an ineligible standing ships by preference, its reasons to the polish

Owner's answer of 2026-09-27 ("Preference + polish"),
asked by the whole-package audit:
over an ineligible standing a slate the judges declined twice
(the first round and its `decline-challenge`)
raised in `translate-runoff-tie.ts`,
and `consolidate-settle.ts` turned that into `ConsolidationStandingIneligibleError`,
stopping the entry with valid proposals on the slate
(the hulicaijia14 shape).

- The slice ships one valid proposal by the tenth addendum's order:
    the repair lane's text,
    then the translate lane's text,
    then slate order;
    the rejection is recorded as a finding.
- The judges' reasons become required corrections for the final polish,
    as the fourteenth addendum does for the gate's objections.
- The sheet tells the slate judges what a decline now costs.
- Landed:
    `settleAbsentDecline` in `translate-runoff-tie.ts` ships the preferred candidate
    where the caller ships past a decline,
    the round is the challenge,
    and no run-off is left to narrow;
    a tie among some candidates still narrows first.
    The record is `judged` with the finding
    `translate-slate-declined-shipped-by-preference (<basis>)`
    and `shippedPastDecline`,
    which carries the reasons of every ballot that did not back the shipped candidate.
- `judgeTranslateSlate` takes `withheldStanding`
    (the judges are told `SHIPS_BY_PREFERENCE`,
  `select-decline-consequence.ts`,
    in place of a sentence saying the passage has no translation)
    and `shipPastDecline`,
  which defaults to it.
    The consolidation passes `withheldStanding` wherever the standing is ineligible.
- The polish's objection correction carries groups
    (`ObjectionGroup`,
  `refine-selection-context.ts`),
    each under its own heading:
    the slate's reasons and the gate's objections reach the same polish,
    and the finding counts each
    (`polish-objection-correction (consolidation slate: N objection(s), consolidation gate: M objection(s), ...)`).
- The translate lane (sixteenth addendum's extension):
    `runTranslateStage` passes `incumbentWithheld` where the archive's own wording fails the floor;
    the first production round still defers to the follow-up round,
    which carries the judges' rejection evidence,
    and only the follow-up round ships past a decline.
    Before this,
    such a content slice rethrew the absence and stopped the entry.
- `CONSOLIDATE_CACHE_VERSION` 18 and `TRANSLATE_SLICE_CACHE_VERSION` 13,
    key literals repinned in the same commit.
- Guard shown to fail first (`37128bf6c`,
  `slate-decline-ships-by-preference.unit.test.ts`);
    `consolidate-settle.unit.test.ts` pinned the stop and now pins the ship.

## Addendum 2026-09-27, fourteenth: the gate's objections over an ineligible standing go to the polish

Owner's answer of 2026-09-27 ("Objections to polish"),
asked on the thirteenth addendum's open question.
Over a standing the deterministic rule refused every gate verdict ships the slate's choice,
so the gate's ballots changed nothing on the page.

- The consolidation still ships;
    the thirteenth addendum stands.
- What the gate ballots held against the consolidation
    (their unsupported and dropped findings and their reasons)
    becomes required corrections for the final polish that already runs on the slice,
    so the round's evidence improves the text instead of only being recorded.
- The gate sheet tells the judge so.

## Addendum 2026-09-27, thirteenth: a gate preferring an ineligible standing ships the slate's choice

Taken under the rule of 2026-09-04
("prefer the best valid proposal,
else fail the slice at once")
after TianqiChen66619 stopped INCOMPLETE at 755,525 ms on slice 9:
the address floor refused the contest's repair text,
so the standing was ineligible,
the slate chose a valid proposal,
and the gate went 2 standing,
1 neither,
1 consolidated of 4 usable.
`requireShippableTerminal` refused `gate-kept-standing` and the entry stopped.

- The first addendum and the third kept that stop,
    reading "else fail the slice at once" as covering a gate's preference.
    The rule's failure is for a slice with no valid proposal;
    here the slate had chosen one,
    and a standing the deterministic rule refused cannot ship whatever the gate thinks of it.
- `shipPastForfeitStanding` (renamed from `shipPastUndecidedGate`,
    `consolidate-ineligible-standing.ts`) ships the slate's choice when the gate prefers an ineligible standing,
    recording `gate-preferred-ineligible-standing`
    (`GATE_PREFERRED_INELIGIBLE_STANDING_FINDING`);
    the gate ballots name what they held against the consolidation.
- Over an ineligible standing every gate verdict therefore ships the consolidation.
    The gate sheet said choosing the standing stops the entry;
    it now says the consolidation ships whichever the judge chooses
    and the ballot records what the judge holds against it.
- Unchanged:
  over an eligible standing a gate that prefers it keeps it.
- Open for the owner:
    whether the gate should still run over an ineligible standing,
    whose ballots now change nothing that ships,
    or whether a gate objection at quorum there should send the consolidation back for a repair round.
- Guards shown to fail first:
    `e7e6f706a` (`consolidate-settle.unit.test.ts`),
    `44e27ca0e` (`consolidate-driver.unit.test.ts`),
    and the sheet wording in `consolidate-gate-wire.unit.test.ts`.
    Fixed in `419605ff4` and the sheet fix `0b8788dae`.

## Addendum 2026-09-27, twelfth: an undecided gate over a standing every contest ballot condemned ships the slate's choice

Owner's answer of 2026-09-27:
"Slate's choice (Recommended)".
TianqiChen66610 slice 13 shipped the archive's gloss of the performer's "high-performance robot" image
as "a cute character she cosplayed as":
every contest ballot called the archive flawed,
the class one hundred six run-off ran,
the slate chose a valid proposal,
and the gate tied 2 to 2,
which settles on neither and so kept the standing.

- `shipPastUndecidedGate` in `consolidate-ineligible-standing.ts` takes `standingFlawedByAll`,
    passed from `settleConsolidation` as `runoffOverStanding` through `gateAndShip`.
    A gate settled on neither ships the slate's valid choice
    where the standing is ineligible (class fifty-four,
  unchanged)
    or where every contest ballot called it flawed (new),
    with the finding `undecided-gate-ships-proposal (standing flawed by every contest ballot)`
    (`FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING`).
- Unchanged:
  a gate that chose the standing keeps it (`gate-kept-standing`);
    a gate over an eligible standing that no contest condemned keeps it on neither.
- Guard shown to fail first (`6243c68d0`,
  `consolidate-settle.unit.test.ts`),
    fixed in `325c448ba`.

## Addendum 2026-09-27, eleventh: an accepted accuracy claim at major severity disputes the archive

Owner's answer of 2026-09-27:
"Major+ accuracy".
On TianqiChen66610 slice 13 the repair lane's adjudicators accepted major mistranslation claims
against the archive's gloss,
but only `accuracy/addition` made a class one hundred seven dispute,
so the archive stayed an eligible standing and shipped on the tied gate.

- `disputesArchive` in `archive-dispute.ts`:
    an accepted `accuracy/addition` claim disputes the archive at any severity (as before),
    and any other accepted `accuracy/` claim disputes it at `major` or `critical`.
    Minor mistranslations and claims outside accuracy do not.
    The severity read is the panel's since the seventeenth addendum.
- `ArchiveDispute.acceptedClaims` carries every disputing claim;
    the separate addition list is gone.
- The ARCHIVE RENDERING DISPUTED note names the rule,
    brings the addition rule only where an addition was accepted,
    and brings a reading rule where another accuracy claim was:
    a reading those claims name is not the page's authority,
    and a candidate keeping it carries an accepted error.
- The finding names the rule:
    `accuracy/addition at any severity, any other accuracy claim at major or worse`
    (since the seventeenth addendum,
  "any other accuracy claim the panel settled at major or worse").
- Guard shown to fail first (`bd6ad35e8`,
  `archive-dispute.unit.test.ts`),
    fixed in `25b9eaec7`.

## Addendum 2026-09-26, tenth: a challenge round tied across every valid candidate ships one by preference

Taken under the rule of 2026-09-04 ("prefer the best valid proposal,
else fail the slice at once")
and the owner's answer of 2026-09-24 ("Not eligible;
fall back to the repair text"),
after TianqiChen6669 stopped INCOMPLETE at 24 min on slice 4:
the class one hundred seventy-two floor refused the archive's "So,
she’s a girl of incredible perseverance",
the contest split 2 repair to 2 translate,
the slate of three valid renderings tied 1.5 to 1.5 between "bore the greatest pressure on her frailest body"
and "with the frailest body",
the run-off over those two tied 1.5 to 1.5 again,
and a run-off of two has nothing left to narrow.

- The rule's failure is for a slice with no valid proposal.
    Here every finalist passed the rule and the judges could not rank them,
    a failure to rank rather than a rejection,
    so the entry stopping on it served neither half of the rule.
- `settleAbsentDecline` in `translate-runoff-tie.ts`,
    called from `judgeTranslateSlate` wherever the slice has no incumbent,
    ships one candidate when a challenge round (`decline-challenge`) ties across every candidate it asked about:
    the repair lane's text first
    (the owner's fallback where the archive cannot stand,
  the minimal edit of the human translation),
    then the translate lane's text,
    then slate order.
    The record is `judged` with the tie's weight and the finding `translate-runoff-tie-broken (<basis>)`,
    and the gate still reads it as a fresh consolidation.
- Unchanged:
  a tie among some of the candidates narrows through the class eighty-two loop;
    a first round is still challenged;
    a rejection,
    a leader one judge alone named,
    or a round with no ballot still raises;
    a slice with an incumbent keeps it as before.
- Guard shown to fail first (`8666d2480`,
  `translate-retry.unit.test.ts`:
  repair lane,
  translate lane,
    slate order,
  and the incumbent kept),
    fixed in `8db508b0a`.

## Addendum 2026-09-24, ninth: the dispute note travels with the stand-in

Taken after CuspariaKLSY11,
the first pass on the eighth addendum,
shipped its slice 3 with the stand-in reading "She took medication that night":
the repair lane had softened the accepted addition (a named overdose) into a detail the original still never states,
the contest split 2 to 2 with one ballot calling the detail "from the original",
the consolidated proposal dropped it,
and the gate split 2 to 2 and kept the stand-in on "dropped page content ... which the Chinese does not contradict",
the apparatus clause of 2026-09-22 (class eighty-five) read over a text that stands in precisely because that detail was an accepted addition.

- The stand-in is judged as the incumbent on every sheet (eighth addendum),
    so every sheet's protection of page content and page apparatus covers it as well,
    and a softened accepted addition survives as page content the Chinese does not contradict.
    Nothing on any sheet said the slice was disputed or which claims had been accepted.
- `archiveDisputeNote` (`archive-dispute.ts`) writes one block per disputed slice,
    headed ARCHIVE RENDERING DISPUTED,
    naming how many `accuracy/addition` claims the repair lane's adjudicators accepted and each claim's category,
  severity and summary,
    and saying that a detail those claims name is not page content and not the page's apparatus,
    in the archive's wording or any softer one:
    a candidate leaving it out has dropped nothing,
    a candidate keeping it carries an accepted addition,
    and such a detail is judged against the ORIGINAL alone.
    `ArchiveDispute` carries the accepted claims (`acceptedClaims`) for it.
- The block is carried wherever the stand-in or the archive is judged or written against:
    the consolidation writer,
  slate judge and gate sheets (`consolidate-wire.ts`,
  `consolidate-settle-context.ts`,
  `consolidate-gate-wire.ts`),
    the translate writer and slate judge sheets (`translate-wire.ts`,
  `translate-judge.ts`),
    and the lane contest sheet (`lane-contest-wire.ts`,
  in the slice key too,
  the notes read off the repair chunks in `corpus-run/pass-contest.ts`).
    The wording names no sheet's own incumbent,
    because the lane contest shows the real archive while the consolidation and translate sheets show the stand-in.
- Not built:
  the repair lane's own sheets are unchanged,
    so the lane may still soften an accepted addition rather than remove it;
    the note reaches the contest and everything after it,
    where the softened detail is weighed as an addition rather than as page content.
- Guards shown to fail first (`65d159d2a`):
    `archive-dispute.unit.test.ts` (the note's wording and the accepted claims),
    `consolidate-wire.unit.test.ts`,
    `consolidate-gate-wire.unit.test.ts`,
    `translate-wire.unit.test.ts`,
    `lane-contest-wire.unit.test.ts`,
    `translate-slice-input.unit.test.ts`,
    `translate-judge.unit.test.ts`.
    Fixed in `85ed2881f`.

## Addendum 2026-09-24, eighth: a disputed archive rendering stands aside for the repair text

Taken on the owner's second answer of 2026-09-24
("Not eligible;
fall back to the repair text")
after CuspariaKLSY10 shipped its slice 3,
the death paragraph,
as the repair lane's minimal edit only because the contest chose that lane:
the archive rendering named a suicide method the original never states,
the repair lane's adjudicators accepted four `accuracy/addition` claims against it and removed the detail,
the translate slate backed nobody,
and the archive stood as the translate lane's text because an eligible standing keeps its single round.

- The deterministic floor reads the archive's shape,
    not its truth;
    a rendering the adjudicators found to add what the original never states passes it.
    The rule of 2026-09-09 (keep the incumbent where the standing is ineligible) would have kept that rendering as well,
    since it is the incumbent.
    A rendering carrying an accepted addition claim is therefore not an incumbent at all on that slice.
- `archiveDisputesOf` (`archive-dispute.ts`) reads the repair lane's chunks once the lane returns:
    a chunk with an accepted issue carrying an `accuracy/addition` claim is a dispute,
    and its repair text (whatever the repair lane decided,
    the archive's own where unchanged won) is the stand-in.
    A rejected or needs-human claim,
    or any other category,
    leaves the archive as it was;
    the mistranslation claims beside the addition on CuspariaKLSY10 do not dispute on their own.
    Narrowed by the sixteenth addendum:
    the stand-in stands only where the checkers confirmed every disputing issue resolved in it.
- On the translate lane (`document-lanes.ts` to `translate-slice-input.ts`) the stand-in is judged as the incumbent:
    a candidate on the slate and the fallback where the judges keep it,
    the archive rendering neither.
    The slice key names the stand-in,
    the record ships against the archive's own bytes (`changed` where the stand-in differs),
    and the finding `translate-archive-disputed` names the slice and the accepted claims.
    The lane ledger still reports the archive as the incumbent,
    so the contest compares the two lanes as before.
- On the consolidation (`pass-entry.ts`,
    `consolidate-archive-stand-in.ts`) the same reading makes the stand-in the incumbent of every sheet and key,
    and where the contest chose neither lane the stand-in is the standing;
    a kept standing there ships as the `incumbent` record so the page is written with it rather than left as the contest found it.
- Not built:
  a repair chunk the assembly withdrew still supplies its stand-in,
    and the repair lane's own floor is unchanged;
    the second answer's run-off runs through the first answer's path (seventh addendum) where the contest ballots call the archive flawed,
    and through the single round elsewhere.
- Guards shown to fail first (`fba338dc1`):
    `archive-dispute.unit.test.ts`,
    `translate-slice-input.unit.test.ts`,
    `translate-document.unit.test.ts` (the translator sheets carry the stand-in and never the archive phrase,
    the record ships changed with the finding),
    `consolidate-driver.unit.test.ts` (a settled-neither contest ships the stand-in as the standing).
    Fixed in `a452db225`.

## Addendum 2026-09-24, seventh: a tied slate over an eligible standing is run off when every contest ballot called the archive flawed

Taken on the owner's answer of 2026-09-24
("Run-off only when every contest ballot called the archive flawed")
after zheermao8 shipped its slice 9,
the reply label,
in the archive's wording "one member ... immediately" for 那些秋叶的成员…连夜:
the contest 2 to 2 with every ballot calling the archive flawed,
the slate 1 to 1 to 1 with both lane texts on offer,
and the eligible standing keeping its single round under the second addendum.

- The second addendum kept the single round over an eligible standing because a decline there keeps text the contest endorsed.
    On zheermao8 the contest had endorsed nothing:
    every usable ballot called the archive flawed and the lanes split,
    so the slate's decline kept a text every judge had refused.
- `archiveFlawedByAll` (`consolidate-archive-flawed.ts`) reads the contest:
    settled on neither,
    at least one ballot,
    every ballot's `archive` set to `flawed`.
    A ballot that did not say,
    a ballot that would publish the archive,
    a contest that chose a lane or missed quorum all leave the single round.
- `consolidate-driver.ts` raises `runoffOverStanding` from that reading;
    `settleConsolidation` then routes the slate through `judgeSlateWithRetry` as it does over an ineligible standing,
    the same panel on the narrowed finalists,
    each run-off strictly narrowing (fourth and sixth addenda).
- The judge stage now names the run-off finalists on a declined round even over an incumbent
    (`runoffFinalists` on `TranslateStageResult`),
    so the retry can narrow without the absence error the ineligible path raises.
- A run-off that ends undecided still ships the standing:
    the exit stays `slate-declined-standing`,
    never a stop,
    because the standing is eligible.
    The owner's second answer of the same day,
    an archive rendering carrying an accepted fabrication claim,
    is class one hundred seven and its own addendum.
- Guards shown to fail first (`200bf7a53`):
    `consolidate-archive-flawed.unit.test.ts`
    and `consolidate-settle.unit.test.ts`
    (the run-off chooses over an eligible standing;
    a run-off tied throughout ships the standing).
    Fixed in `2a872bbed`.

## Addendum 2026-09-19, sixth: a run-off is decided on the ballot floor

Taken under the same rule after mikaela4 stopped INCOMPLETE at 24 min on slice 28,
the Epilogue,
whose contest standing merged two of nine lines:
the slate 1.5 to 1 to 1 to 1,
the run-off over the two leaders 1.5 to 1 with two abstentions,
both calling the finalists ineligible on grounds the rule had answered,
`slate-declined-standing`.

- The fourth addendum left a run-off short of the minimum to end the slice.
    That is the case where the judges who chose agreed
    and the judges who abstained answered a question the run-off did not ask:
    the finalists were valid by the rule and by the slate's own verdicts,
    so the question was which,
    and an abstention answers neither.
- In a run-off (`runoff` passed from the retry with the finalists),
    the leader wins on the two-ballot floor under the weight minimum,
    carrying `select-runoff-under-minimum`;
    the minimum stands everywhere else.
- A run-off whose leader one judge alone named still ends the slice;
    one that ties across every finalist ends it no longer (tenth addendum).
- Guard shown to fail first (`49b9dccca`),
    fixed in `6a9a7ce27`.

## Addendum 2026-09-19, fifth: the translate slate carries only what the rule accepts

Taken under the same rule after XingZ615 stopped INCOMPLETE at 2h41m on slice 91,
the closing poem,
`slate-declined-standing` with one floor-valid candidate,
the Chinese original respaced.

- The standing was ineligible because the translate lane's judges chose a candidate the deterministic rule had refused
    (the attribution moved out of the quote)
    over two valid renderings;
    the lane keeps a refused candidate on its slate after its repair turn
    (the decision of 2026-08-15),
    and the judges do not apply the rule.
- `translate-floor.ts` now withholds from the translate slate every candidate the rule still refuses after that turn,
    recorded as `translate-candidate-refused`;
    the repair turn and the model's defence are unchanged,
    and a rule that cannot say keeps the candidate.
    This is "prefer the best valid proposal" applied one stage earlier,
    and it narrows the 2026-08-15 reading;
    the owner may veto it.
- The untranslated check compares in all but whitespace,
    so a copy of the original with respaced quote lines is refused at every floor.
- Guards shown to fail first (`636d56a8c`),
    fixed in `6eada935a`.

## Addendum 2026-09-19, fourth: the run-off is over the leaders

Taken under the same rule after XingZ613 stopped INCOMPLETE at 3h02m on slice 84,
the `其九：空白` heading,
whose archive rendering drops a line:
four valid proposals,
the judges at 1.5 to 1 to 1 against the minimum of 2,
the challenge round the same three-way question,
the same split,
`slate-declined-standing`.

- The class fifty-three run-off offered every candidate that drew a ballot,
    which on a plurality is the slate less the unnamed;
    the candidates below the leaders have lost the ranking as surely as the unnamed ones.
- `runoffFinalists` now offers the leaders alone:
    the candidates tied at the top on a tie,
    the leader and its runner-up on a plurality under the minimum,
    the runner-up by weight,
    then ballots,
    then slate position.
    Two candidates is a question a bench of whole and half ballots settles unless it abstains.
- The minimum and the two-ballot floor are unchanged;
    a run-off short of them still ends the slice as before.
- Guard shown to fail first (`9176c9819`),
    fixed in `4193495fa`.

## Addendum 2026-09-18, third: the gate is told when the standing cannot ship

Taken under the same rule after XingZ606 stopped INCOMPLETE at 3h04m on its slice 33,
this time at the gate:
the slate chose a valid proposal,
the gate kept the ineligible standing 4 of 5,
and the log said nothing of why.

- The gate sheet showed the ineligible standing as a rendering to keep and asked which of the two the
    original supports better,
    a comparison against a text that was never going to ship.
- `readStandingVerdict` now returns the refusal it logs (`standingRefusal`),
    threaded through `consolidate-driver.ts`,
    `consolidate-slice-buy.ts`,
    `consolidate-settle.ts` and `consolidate-settle-gate.ts` to the gate subject,
    and `buildConsolidateGateMessages` says the standing cannot ship,
    why,
    and that choosing it stops the entry.
- `gateConsolidatedSlice` logs every ballot with its model,
    choice and reason.
- A gate that refuses at quorum still stops the slice:
    the rule of 2026-09-04 is unchanged,
    the judges are informed
    (superseded by the thirteenth addendum,
    and the sheet no longer says choosing the standing stops the entry).
- Guards shown to fail first (`90e5e2a19`):
    `consolidate-gate-wire.unit.test.ts` (the block on the sheet,
    absent over an eligible standing)
    and `consolidate-settle.unit.test.ts` (the refusal reaches every gate sheet).
    Fixed in `139a7f99a`.

## Addendum 2026-10-29: the translate lane's two-ballot winner ships where the final round cannot seat two judges

Taken after the corpus pass `XingZ6022` shipped the archive at sixteen slices
(14,
44,
54,
60,
67 to 70,
72,
73,
75,
78,
79,
81,
82 and 85)
although the translate lane held a text two judges had chosen that passed the publication rule.
The consolidation offered it on the slate,
one slate judge was reachable,
"one judge alone never decides" declined it,
and `keepTheArchive` shipped the archive with the defect text "no wording passed the deterministic publication rule",
which was false there.
At slices 67 to 70 the archive carries an invented Metal Gear Solid 2 quotation.

The owner's answer (question tool),
choice "Ship the two-ballot text (Recommended)":
"When the final round can't seat two judges,
the translate lane's winner ships if two judges chose it and it passes the publication rule."

- A refused standing whose slate round heard fewer than two judges,
    or whose reachable slate judges are fewer than two,
    ships the translate lane's text where two or more judges chose it in the lane's own round
    and the lane offer admitted it (it passes the rule).
    The settlement keeps its terminal and carries the mark `translateWinnerShipped`
    and the finding `translate-winner-ships`.
    It polishes nothing and is never cached.
- With fewer reachable slate judges than a winner needs and such a text on the offer,
    the purchase asks neither writers nor slate
    (the check measured 65 calls and 10.679 Wh at these slices).
- The rules that stand:
    one judge alone never decides,
    two ballots decide,
    "Keep archive,
    ship" where no wording passes,
    and a disputed archive is never a candidate or a fallback.
    A text with one ballot,
    a text that fails the rule and a slate that seated two or more judges decide as before.
- Every kept archive says why:
    `no-wording-passed`,
    `final-round-unseated` or `slate-declined`,
    in its finding,
    its log line and (artifact generation 22) its record,
    and the publish defect `no-valid-wording` names the slices by reason.
- The ballots are read off the translate lane's records,
    not out of the artifact rows,
    which carry none.
- Ledger:
    B1020 to B1025 in `package/translation-repair/doc/audit-ledger-the-two-ballot-translate-text.md`.
