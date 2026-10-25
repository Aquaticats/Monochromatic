# OpenRouter and the corpus passes: the defects of XingZ6021 on build bcff23b8ce (2026-10-25)

The ten defect candidates and observations that the read of `XingZ6021` found (N1 to N10),
with the second reader's verdicts,
the owner's rulings of 2026-10-25,
the corrections the check made to the read,
and the points no log settles.
None of the ten was fixed in the run's build.
The defects `D1` to `D7` of `XingZ6020` are accounted for in
[the stages of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md),
under the seven checks;
the same defects with the commit that fixed each are in
[the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md).
The log,
the providers and the meters are in
[the run of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).

Paths are at the run's tip `bcff23b8ce` unless a path is of the ledger,
which is read on main at `b975d68aaf`.
Times are UTC.
Line numbers are of the log unless a line is called a page line,
a source line or an archive line.
The check's verdict for each candidate is one of:
new,
known (with the ledger id),
or by design.

## N1: the Shinonome section ships text the source does not contain

Verdict of the check:
confirmed,
and extended by findings that the read missed.
It is the defect that matters most in this run.
A page shipped as `SETTLED` carries archive text that the source does not contain,
after the run's own critics said so,
and the code publishes such a page by omission,
not by a recorded decision.

### The page against the source and the archive

- Shipped page lines 293 to 306:
  the heading `### Part Five: Shinonome` (293),
  `<details>` (295),
  the summary (296,
  555 characters),
  three paragraphs (298,
  300 and 302),
  `</details>` (304) and the attribution (306).
  The check compared page lines 296,
  298,
  300 and 302 with archive lines 244,
  246,
  248 and 250 (`archive.page.en.md`):
  all four are byte-identical (`c4.ts`).
  The section is the archive's text unchanged;
  the run left it as it was.
- The source section is `source.page.md` lines 237 to 251:
  `### 其五：东云` (237),
  `<details>` (239),
  the summary (241,
  93 characters),
  four paragraphs (243,
  245,
  247 and 249;
  109,
  159,
  172 and 104 characters) and `</details>` (251).
  The first reader gave lines 238 to 250,
  one line short at each end;
  the check's lines are the ones filed.
- The *Metal Gear Solid 2* quotation is in the shipped summary (page line 296):
  "She lived much like the words spoken by Big Boss in *Metal Gear Solid 2*:
  "The loser will be liberated from the
  battlefield.
  And the winner... will remain.
  The survivor will live out every one of the rest of his days as a soldier."
  She was a warrior on this battlefield,
  and although she was unable to escape the cruelty of the fight,
  her spirit and
  determination will never be forgotten."
  A scan of the source for `Metal Gear`,
  `MGS`,
  `Big Boss`,
  `合金装备`,
  `潜龙`,
  `胜者`,
  `败者` and `Snake` finds nothing in that section
  (the only hit on `战士` or `战场` is source line 547,
  a different section).
- The two paragraphs built on it are page line 298
  ("Her passing is not a defeat,
  but a different kind of liberation ...")
  and page line 300
  ("However,
  as the quote reminds us,
  "The winner will remain."
  ...",
  which cites the quotation outright).
  Neither paragraph renders any of the source's four paragraphs.
  They paraphrase themes that the source has elsewhere
  (freedom from shackles,
  carrying on,
  a call for acceptance),
  but not one sentence of them translates a source sentence.
  The third paragraph (page line 302,
  "May her soul find peace in another world ...") is a loose echo of source paragraph four
  (source line 249,
  the wish to be free of shackles and to have peace).
- Two qualifications for fairness,
  from the check:
  the source does say her life was short (source line 245,
  `她的一生虽然短暂`) and calls the marginalized `勇士` (warriors,
  source line 243),
  so "her life was short" and "warrior" are not additions in the section as a whole;
  they are misplaced into the summary.
  The source's summary,
  which says she lived in a world not kind to her under social pressure and endless prejudice,
  is not rendered anywhere on the page.
- The translate lane's text for the same slices was faithful
  (artifact `comparison[67..70].translateText`):
  slice 67
  "...She lived in a world that was not kind to her,
  bearing the pressure of social discipline and endless prejudice,
  yet she always chose to move forward in her own way.";
  slice 68
  "She was the epitome of countless marginalized individuals ...";
  slice 69
  "Her life was short,
  yet like a candle flame it brought a little light to others ...";
  slice 70
  "May she be free,
  in another world,
  of every chain that bound her ...".
  All four lost.
- Effect:
  four slices (67 to 70,
  1,584 characters in page lines 296 to 302 by `wc --chars`) are the archive's text,
  not a translation of their source in the parts named.

### Why it shipped, link by link

The first reader traced six links;
the check confirmed them and found three more,
marked here as added by the check.

1. Critics.
   Slice 68 drew two critical `accuracy/non-translation` claims (lines 8933 and 8934);
   slice 67 nine claims,
   seven major and two minor (lines 8950 to 8958;
   DeepSeek filed 4 and Haiku 5,
   with `accuracy/addition` for the quotation,
   for "her life was short" and for "warrior on a battlefield",
   and `accuracy/omission` for the unkind world);
   slice 69 two critical `non-translation` claims (lines 9035 and 9036);
   slice 70 seven claims,
   six major and one minor (lines 9100 to 9106;
   added by the check).
   The critic stage heard 2 of 8 voices (lines 8932,
   8949,
   9034 and 9099),
   `deepseek-v4.1-flash` and `claude-haiku-5-5`.
2. Panel and electorate floor.
   Each panel packet heard 2 of 8 (`stage-short-bench (panel reachable 2 of 8, quorum 2)` is in chunk 67's findings),
   so the stage's own quorum was 2 and was met.
   The tally then needs weight 3:
   `adjudication/src/tally-votes.ts:78` to `:82`
   (`electorate = supported + unsupported + ambiguous + sourceDefect`;
   `if (electorate < config.minBallotWeight) return 'needs-human'`,
   the check at `:81`);
   `shared-contract/src/adjudicate-model.ts:311` to `:315` (`DEFAULT_ADJUDICATION_CONFIG`,
   `minBallotWeight: 3` at `:312`,
   no `weights`).
   The weight of a panelist defaults to 1
   (`adjudication/src/tally-claim.ts:33` to `:43`;
   `adjudicate-model.ts:296`,
   "absent panelists weigh one"),
   and the run passes no config
   (`adjudication/src/panel-stage.ts:430` to `:434`;
   `repair/src/repair-slice-key.ts:722` to `:726` says no run can replace it).
   Two panelists weigh 2,
   below 3,
   so every claim lands `needs-human`,
   even where both panelists support it:
   line 9058 (slice 68,
   `needs-human critical`,
   "panel claude-haiku-5-5 supported,
   deepseek-v4.1-flash supported"),
   line 9080 (slice 69,
   the same),
   line 9285 (slice 67's quotation claim,
   `needs-human major`,
   both supported) and lines 9238 to 9244 (slice 70,
   six `needs-human major`,
   one minor).
   Totals for the four slices:
   20 claims (9,
   2,
   2 and 7) and 17 issues in the artifact,
   all `needs-human` (12 major,
   2 critical and 3 minor;
   artifact `chunks[67..70].issues`).
3. Editor.
   Envelopes are cut from accepted issues only
   (`repair/src/repair-chunk.ts:316` to `:323`,
   with `:332` to `:334` filtering `issue.status === 'accepted'`),
   so with none accepted the chunk logs `nothing to edit, unchanged`
   (lines 9059,
   9081,
   9245 and 9286).
   Nothing in the pipeline reads a `needs-human` status except to record it:
   the only non-test mentions are the status type (`shared-contract/src/adjudicate-model.ts:140`),
   the tally,
   a comment in `artifact/src/artifact-v1-read.ts:25` (non-accepted issues are left out of graded precision)
   and the probe code.
   No severity,
   including critical,
   changes this.
4. The two guards that exist for exactly this case could not fire (added by the check).
   - The archive dispute.
     `archive/src/archive-dispute.ts` (owner classes 107,
     2026-09-24,
     "Not eligible;
     fall back to the repair text",
     and 176,
     2026-09-27)
     makes the archive ineligible when the repair lane's adjudicators accepted an addition or a major accuracy claim
     against it.
     It reads only `issue.status === 'accepted'` (`:306` to `:311`,
     `disputingIssuesOf`).
     With the panel unable to accept anything,
     the dispute could not arise;
     the log's `translate-archive-disputed` lines are for slices 2,
     3 and 4 only (lines 12402 to 12404).
     Had it fired,
     the archive would not have been "a candidate,
     a fallback,
     a standing or a lane offer anywhere downstream"
     (the header of `archive-dispute.ts`).
   - Non-translation votes.
     `archive/src/non-translation-evidence.ts:29` (`NON_TRANSLATION_BLOCK_VOTES = 3`) and `:267`
     (`votes >= NON_TRANSLATION_BLOCK_VOTES`):
     "three independent voices,
     not a bare pair".
     Slices 68 and 69 drew exactly two critical `non-translation` votes each
     (artifact `nonTranslationVotes: 2`,
     `nonTranslationStanding: false`),
     so they did not stand either.
     This is a second absolute floor of 3 beside the tally's floor of 3.
5. Silent-stage record (added by the check).
   `repair/src/repair-cache-gate.ts` and `ChunkRepairOutcome.silentStages` (ledger B820,
   `doc/audit-ledger-silent-stages-recorded-as-data.md:11` on) exist to mark a panel that heard fewer than quorum.
   The stage quorum shrinks to the reachable bench
   (`stage/src/stage-reachable-quorum.ts:1` to `:30`,
   floor `MIN_STAGE_VOICES = 2`),
   so 2 of 2 counts as quorum met:
   `silentStages` is empty on all 118 chunks.
   The stage believes it heard a quorum that its own tally cannot use.
6. Lane contest.
   For 71 slices in all,
   `lane contest: 1 of 7 seats reachable, below the floor of 2; no round is opened and the slice keeps its incumbent`
   (`lane-contest/src/lane-contest-stage.ts:444` to `:446`;
   first line 17107);
   the artifact records `verdict: quorum-not-met, usable: 0, benchBelowFloor: true` for slices 66 to 71,
   including 67 to 70.
   The seven late judges are all unreachable except `deepseek-v4.1-flash`:
   Haiku holds `late-judge-dropped`
   (`shared-contract/src/claude-card-sides.ts:264` to `:268`;
   the comment at `:37` to `:39` says "which the owner did not name"),
   and Qwen holds it by the owner's answer of 2026-10-22
   (`shared-contract/src/model-cards.ts:108` to `:115`).
7. Slate.
   The consolidation bought a slate for these slices
   (a lane text existed,
   so the skip at `consolidate/src/consolidate-slice-buy.ts:251` to `:257` could not apply).
   The lone judge,
   `deepseek-v4.1-flash`,
   said in its own words that the archive text was the fault,
   and was outvoted by the floor of two ballots
   (the log prints the weights):
   line 19632 (slice 67,
   weight 0.5)
   "Candidate 3 is disqualified:
   it invents a lengthy quotation attributed to Big Boss in *Metal Gear Solid 2*,
   adds the
   characterization that her life was short and filled with defiance,
   and drops the original's propositions ...";
   line 19689 (slice 68)
   "Candidate 1 is not a rendering of the ORIGINAL at all";
   line 19728 (slice 69,
   weight 1)
   "Candidate 1 is not a translation of the ORIGINAL and adds unrelated material about a quote,
   a warrior,
   and victory.
   Candidates 2,
   3,
   and 4 all render the passage completely and faithfully";
   line 19764 (slice 70)
   "Candidate 2 is unacceptable:
   it drops most of the passage".
   Then `bench short of quorum ... the winner needs weight N from at least 2 ballots` and
   `winner was named by 1 ballot against a floor of 2; keeping the fallback`
   (lines 19633 to 19635 for slice 67),
   and the terminal `slate-declined-standing`
   (artifact `consolidation.slices[67..70].terminal`;
   mapping at `consolidate/src/consolidate-settle.ts:128` to `:130`).
8. Ship.
   `consolidate/src/consolidate-slice-buy.ts:411` to `:415` warns
   `standing lacks contest endorsement and the single consolidation attempt kept it (slate-declined-standing);
   shipping with the finding recorded`
   (the text is at `:413`;
   log lines 19636,
   19693,
   19732 and 19768).
   Page checking records `final-selection-unendorsed (slice N): archive wording stands without contest endorsement,
   recorded as evidence` and returns it as a finding
   (`page-check/src/corpus-run/final-selection-completeness.ts:146` to `:147`;
   the lines for slices 67 to 70 are among the 35 at 21893 to 21927).
   The `TALLY` counts the slice in `standingUnendorsed=66`
   (`slice-cache/src/corpus-run/settled-tally.ts:72` to `:140`,
   `standingsKeptUnendorsed` at `:96`;
   slices that kept the archive for want of any valid wording are excluded there,
   and these did not).
   `repairIssues=341 repairAccepted=89` are printed,
   but no field says that 234 of the 341 issues were left for a human.
   `DEFECTS` did not print,
   and no `publish: shipping with defect` warn exists (`rg --count`:
   0),
   which is correct by the code:
   `PublishCheck` (`page-check/src/corpus-run/publish-defects.ts:30` to `:40`:
   archive-original,
   contributor-names,
   destinations,
   front-matter,
   headings,
   no-valid-wording,
   page-agreement,
   page-footnotes,
   page-source-unparseable;
   emitted at `corpus-pass/src/corpus-run/pass-entry-persist.ts:150` to `:160` and
   `page-check/src/corpus-run/publish-fixed.ts:369` to `:380`)
   has no check on `needs-human` or on unendorsed standing,
   and none of its checks measures whether the wording follows the source.
   So a page like this one ships clean.

### Does an owner rule say such a slice ships the archive?

No.
The check searched `package/translation-repair/doc/` on main for `keep the archive`,
`keeps the archive`,
`needs-human`,
`one judge`,
`always ships`,
`unendorsed`,
`lacks contest endorsement`,
`electorate` and `minBallotWeight`.
The rules that touch the case:

- "A run always ships" (owner,
  2026-09-27;
  `doc/design-commitments-shipping-and-the-archive.md:49`):
  a check reports and repairs,
  it never withholds pages.
  It covers the page shipping;
  it does not say what wording a slice carries.
- "A settled page ships with its defects reported" ("Ship with the defect",
  2026-09-27,
  same file):
  the `DEFECTS` list is the publish checks,
  which do not include this.
- "A slice with no valid wording keeps the archive's,
  and the page ships" ("Keep archive,
  ship",
  2026-09-27,
  same file,
  `:119`):
  applies where no wording passes the deterministic rule.
  Valid wording existed here (the translate lane's text),
  so the rule does not describe this slice.
- "One judge alone never decides" (the select floor of two ballots,
  `MIN_SELECTION_BALLOTS`;
  the same file `:163` calls it "the standing rule";
  the self-vote counting toward it was confirmed by the owner on 2026-10-20,
  `doc/audit-ledger-outcome-rules-and-stage-records.md:600` to `:604`).
  It is the rule that kept the lone judge's ballot from replacing the archive.
- Classes 107 and 176 (the archive dispute) say the opposite of "ship the archive" for an archive that adds material,
  but only when the adjudicators accept the claim.
- The owner's 2026-08-06 decision on the panel floor
  (`run/src/corpus-run/run-config.ts:496` to `:501`:
  `minBallotWeight` 3 is "3-of-6 (50 percent)",
  "50% is okay here") considered a panel of six to ten,
  not a reachable bench of two.
- The 2026-10-23 ruling on the no-archive case (same file) rejected "unbacked wording marked as unreviewed" because it
  "would break the standing rule that one judge alone never decides".
  It is the nearest statement of the owner's preference for this slice,
  and it concerns a slice with no archive text;
  the owner had not ruled the case where an archive exists and the run's own critics call it a non-translation.

So the archive shipped as the default of a chain of floors,
each built for a healthy bench and each reading an absolute number
(weight 3 in the tally,
3 votes for non-translation,
2 ballots in the slate) that a bench of two cannot meet.
No rule records that the combination was meant to ship an archive that three layers of the run's own evidence call a
non-translation.
The first reader's open question,
whether `SETTLED` with a `needs-human` major or critical is meant to be publishable,
is answered by the check:
it is publishable by the code,
by omission and not by a recorded decision.

### The owner's rulings of 2026-10-25 and the change in progress

The owner answered three questions about these floors on 2026-10-25.
The filing brief gives the answers;
the full text of the questions was not among the filer's inputs.

- On the floor of weight 3 in the tally:
  "floor of 3 - don't set a floor."
- On a lone judge:
  "Two ballots decide (Recommended)".
- On the scope of the change:
  "Every weight floor".

The change that follows from them is in progress and has no commit yet,
so this record names no commit for it.
It is the work that answers N1:
the eight links of this section leave the archive in place at three absolute floors
(the tally's weight 3,
the 3 non-translation votes and the 2-ballot slate),
and the owner's scope answer names every weight floor.
The filer did not see the change,
so this record does not describe what it will do beyond the three answers.
When it lands,
a rerun of `XingZ60` is the way to see whether the Shinonome section still ships the archive's text.

## N2 to N10, with the check's verdicts

### N2: a panel that cannot reach its electorate floor keeps running

Check:
confirmed,
new.

- 165 panel rounds closed on 2 of 8 voices (lines 5798 to 10650),
  7 on 3 of 8 and 42 on 4 of 8 (41 before the dry reading and 1 after it,
  line 5493);
  214 `panel: bench short of quorum` lines in all.
  With weights of 1 an electorate of 2 is below the floor of 3 for every claim,
  so rounds on 2 voices can only return `needs-human`
  (`tally-votes.ts:81` to `:82`;
  `adjudicate-model.ts:312`).
- Adjudications (the check's `c7.ts`,
  its own parse of the `chunk N: issue adjudicated/... <status> <severity>:` lines):
  341 in all.
  Before the dry reading (00:01:55.805 UTC) there are 108,
  of which 9 are `needs-human` (all minor).
  After it there are 233,
  of which 225 are `needs-human` (144 minor,
  78 major and 3 critical),
  7 `accepted` and 1 `rejected`.
  So 234 of the 341 issues were left for a human.
- The split behind the 225 is corrected.
  The first reader wrote that 167 name one or two panel voices,
  that 58 name four or five and were "not decided for a reason I did not trace",
  and that 41 more adjudications named one voice.
  The panel decision lines carry the weights
  (`adjudication/src/panel-stage.ts:142` to `:175`,
  `describePanelDecision`,
  logged at `:436`;
  for example line 9158 `... needs-human at major; issue/35bdfb...: supported 2, unsupported 0, ambiguous 0,
  source-defect 0, abstain 0`).
  Over all 225 `needs-human` after the reading,
  224 have a panel of exactly two voices and an electorate of weight 2,
  and 1 (line 5552,
  00:02:12 UTC) has four voices and a split of 1 supported and 3 ambiguous.
  None has one voice and none has five.
  The chunk line names the filer and the panel together,
  so counting every name on the line exceeds the panel;
  the check did not establish where the "four or five" came from.
  So the 58 unexplained adjudications do not exist,
  and the "41 more named one voice" is unsupported.
- Before the reading,
  all 9 `needs-human` had an electorate of 4 and were genuine splits
  (eight 2 to 2,
  and one with 1 supported,
  2 unsupported and 1 ambiguous).
  The 8 adjudications decided after the reading are all slice 17 at 00:02:12.277 UTC
  (lines 5545 to 5553 except 5552),
  a panel that still had four voices because Kimi and Qwen were heard just before the reading propagated.
  Of the 224 two-voice adjudications,
  none reached a decision;
  the earliest is at 00:03:31.9 UTC.
  So the electorate floor explains 224 of the 225.
- A short-bench branch exists
  (`stage/src/stage-windowed-rounds.ts` closes the round on the reachable bench;
  `closed on a short bench` is built at `:135`,
  `neededWith` at `:266`;
  the panel stage logs `panel: bench short of quorum, reachable 2 of 8; closing on 2 voices`,
  `stage/src/stage-quorum.ts:553`),
  but it closes the round on fewer voices and does not skip a round that cannot decide.
- Ledger:
  searched for `minBallotWeight`,
  `electorate`,
  `panel.*floor` and `short.*panel`:
  no entry.
  The nearest are B820
  (`audit-ledger-silent-stages-recorded-as-data.md:11`,
  a panel with zero voices cached as examined;
  its fix marks a panel that heard fewer than quorum,
  which is not triggered here)
  and E5 (panel logging).
  New as a finding.
  The change in progress (the section on the owner's rulings) covers it by its scope,
  "Every weight floor".

### N3: the late benches omit seats on purpose, and with Kimi dry they reach one voice

Check:
confirmed,
known.

- Haiku holds `late-judge-dropped`
  (`shared-contract/src/claude-card-sides.ts:264` to `:268`:
  the owner did not name the roster-wide rounds after the lanes),
  and Qwen was removed by the owner's answer of 2026-10-22
  (`shared-contract/src/model-cards.ts:108` to `:115`,
  `run/src/corpus-run/run-config.ts:282` (`LATE_JUDGE_DROPPED`) and `:320` to `:323` (`RUN_LATE_JUDGES`)),
  so with Synthetic dry the late bench reads 1 of 7:
  DeepSeek alone.
- The slate skip needs `laneTexts.length === 0` (`consolidate/src/consolidate-slice-buy.ts:251` to `:257`),
  so it skipped 10 slices only.
  Counts for lines 17281 to 21837 (the check's `c15.ts`):
  90 slate `select` rounds `1/7 no quorum`;
  `produceConsolidations` 86 rounds on 2 of 9,
  4 on 3 of 9 and 1 on 4 of 9
  (the first reader's "91 ... on 2 of 9" in its D3 section conflicts with its own split of 86 and 5 in N2);
  31 naturalness-review rounds `1/7 no quorum`;
  5 gate and 5 polish-gate rounds `1/7`;
  71 lane-contest skips and 10 slate skips (`rg --count`).
  The cost was 21.1 min and 69.6 Wh for 6 changed slices.
- Ledger:
  B835 (`doc/audit-ledger-short-benches-and-the-last-live-seat.md:362`,
  scope statement at `:390` to `:402`) is the fix of this build,
  and records that it skips only slices with an eligible standing and no lane text.
  This run is the first measurement after it.
  The lane contest is now skipped (71 slices,
  0.06 s),
  so D3's most expensive stage is fixed.

### N4: one gate ballot can ship a consolidation

Check:
corrected,
known.

- Slices 13,
  14,
  33,
  44,
  84 and 101 changed at the consolidate gate (lines 17852 to 20295).
  The first reader wrote that all six had `usable:1`.
  The check read the artifact's `consolidation.slices[].gate` for the six `consolidated` slices:
  slices 13,
  14,
  33,
  44 and 84 have one usable ballot;
  slice 101 has two (`usable=2`,
  gate round `2/7` at line 20793,
  inside the 2 percent window of Synthetic).
  So five of six rest on one ballot.
- Ledger:
  B835 (`:402`) records four slices of `XingZ6020` (13,
  44,
  67 and 69) that "shipped a lone judge's proposal as `consolidated`",
  kept as designed "because skipping would change the text that ships".
  Known.

### N5: the refinement pass shipped nothing

Check:
confirmed,
known in kind.

- Artifact:
  `refined: true` on 0 chunks and `changed` on 14.
  18 of 18 won refinements were rolled back because the probers or checkers heard one voice of three
  (`repair/src/refine-probe-verdict.ts:148`,
  `repair/src/refine-recheck.ts:226`);
  the rollbacks log and are correct by their rule.
  The 18 slices kept their pre-rewrite text.
  Warn shapes:
  `refine-probe-unheard` 15,
  `refine-recheck-unheard` 2,
  `refine-rolled-back-by-probe` 1.
- Ledger:
  the rules are `doc/audit-ledger-which-seats-a-quorum-counts.md:285` to `:290`
  (`refine-probe-unheard (<heard> of <configured> probers heard)`),
  `:204` to `:232` (`refine-recheck-unheard`) and `:352`;
  the silent rollback of `XingZ6020` is B829 (`doc/audit-ledger.md:114`).
  The consequence on a short bench is new only as a measurement.

### N6: a winner can rest on one independent ballot

Check:
confirmed,
known by owner ruling.

- `house-rule/src/candidate-select-model.ts:55` (`SELF_VOTE_WEIGHT: number = 1 / 2`).
  The check's count of `won weight` lines for translate slices 88 to 117:
  1.5 on 23 and 2 on 7.
  For 23 of the 30 tail slices and 42 of the 66 head slices the winner is DeepSeek's own candidate at 1.0 plus 0.5;
  this meets the select floor by design (`translate-stage/src/candidate-select.ts` header,
  weight 1.00 from at least 2 ballots).
- The owner confirmed on 2026-10-20 that a self-vote counts toward the floor of two ballots,
  rejecting "Exclude self-votes"
  (`doc/audit-ledger-outcome-rules-and-stage-records.md:600` to `:604`).

### N7: `neuralwattReckonedWh` stays at 0.324

Check:
by design,
not a defect;
minor and conservative.

- Log:
  the only change of `neuralwattReckonedWh` is 0.000 to 0.324 between the readings at line 6142 (00:04:07.552 UTC)
  and line 6847 (00:05:08.254 UTC);
  no `METERS` line lies between them.
  The overrun stream is line 6606 (00:04:47.175 UTC:
  `overrun, elapsed 24037ms ... 32016 content chars`;
  the warn at 6607 reads "ended a call that exceeded its content bound,
  32016 characters ... against a bound of
  32000,
  voice lost").
  It has no `SPEND` and no energy line.
  The step lies inside that interval.
- Code:
  `provider/src/neuralwatt-client.ts:503` passes `onAbandonedAttempt: held.noteAbandoned`.
  `provider/src/transient-retry.ts:366` to `:382` tells it of every attempt that delivered something and failed
  (the comment cites ledger P1,
  2026-09-28:
  every attempt that delivered something was billed).
  `provider/src/neuralwatt-ledger.ts:522` to `:529` (`noteAbandoned`) appends a ledger line at the reservation's
  estimate with `estimated: 'abandoned-estimate'`.
  `provider/src/neuralwatt-day-energy.ts:119` to `:127` sums those lines as `reckonedWh`;
  lines are filtered by the day they fall in (`entryIsInDay`,
  `:43` to `:55`),
  so the line stays until the day rolls over.
  `remainingWh = dailyWh - Math.max(spentWh, accountWh) - reservedWh` (`:136` to `:143`).
- Arithmetic at line 21775:
  ledger spend 280.882 plus 0.324 is 281.206,
  larger than the account's 280.882,
  so the maximum picks the ledger,
  and 550 less 281.206 less 0.913 held for calls in flight is 267.881,
  the logged `neuralwattWh`.
  So the estimate counts against the day's budget by 0.324 Wh (0.06 percent of 550).
  The account meter equals the measured sum exactly,
  so the cut stream added nothing the account shows;
  the estimate is a conservative over-count,
  not a missed release.
- The size 0.324 equals the model's estimate for a prompt of this size
  (`0.3237 Wh estimated for 16941 prompt tokens`,
  line 1505,
  and 17,094 and 17,023 tokens at lines 1512 and 1515).
  The overrun call's own prompt size is not logged,
  so the link remains an inference from timing and size,
  and the code path supplies the mechanism.
- Ledger:
  searches of `doc/` for `abandoned-estimate`,
  `noteAbandoned` and `reckonedWh` find only the field described as "the share resting on estimates"
  (`doc/configuration.md:112`,
  `doc/provider-availability.md:130`).
  No ledger entry records that an abandoned-attempt estimate stays on the day.
  New as an observation;
  the behaviour matches the P1 comment.

### N8: the DeepSeek cost is booked at peak though the run was off peak

Status:
the fix is in progress as ledger B920
(`package/translation-repair/doc/audit-ledger-providers-routing-and-seating.md`)
and has not landed:
`git log origin/main --grep B920` finds no commit and the ledger on `main` at `7a61c5e19e` has no B920.
As first filed on 2026-10-25 (UTC),
N8 was an open defect candidate filed for the owner;
the check explained the mechanism and did not call it a defect.
The update of 2026-10-25 (UTC) under this heading describes the fix;
the bullets filed on 2026-10-25 are kept as written.

Note of 2026-10-25 (UTC),
added with the third round of the fix,
which merges `main` into its branch:
the entry B920 now stands in `package/translation-repair/doc/audit-ledger-deepseek-off-peak-pricing.md`,
not in the providers file named in the status lines of this heading,
and those status lines are history.

- The meter fell by 0.07 USD while 15 calls booked 0.147093 USD.
  The mechanism is in the run file under "The DeepSeek balance against the booked cost":
  the build prices DeepSeek at peak always
  (`provider/src/direct-cost.ts:15` to `:20`;
  `shared-contract/src/direct-card-sides.ts:51` to `:56`),
  the run lay between Saturday 22:49 UTC and Sunday 00:55 UTC,
  so every call was off peak and the true rate was half.
  Half of the booked 0.145968 is 0.0730,
  against a meter fall of 0.07.
- The check's verdict:
  neither rounding nor a defect,
  because the bound is stated in the code as the upper bound;
  new only as a measured ratio (2.1 times).
- The open question,
  which the code does not answer:
  whether booking at peak when the call falls in off-peak hours is wanted.
  The cost of the rule is a booked spend up to twice the true spend on a weekend or an off-peak run
  (the filer's reading of the stated bound).
  This record keeps it as an open defect candidate until the owner rules.

Update of 2026-10-25 (UTC),
from the lead's brief for this filing:
ledger B920 fixes N8 once it lands,
and it has not landed.

- The rule:
  DeepSeek is booked at the card's peak rates only for a request that started in a weekday peak window in UTC,
  01:00 to 04:00 UTC and 06:00 to 10:00 UTC
  (half-open,
  so 01:00 UTC and 06:00 UTC are peak and 04:00 UTC and 10:00 UTC are not),
  and at half otherwise.
- The recomputation on this run:
  the 15 DeepSeek calls were all off peak (Saturday and Sunday in UTC),
  so the booked 0.147093 USD recomputes to 0.073547 USD,
  against a meter fall of 0.07 USD read to two decimals.
  The run file already named half of 0.147093 as 0.0735.
- The call path had no clock,
  so one was added.
- Every earlier run's DeepSeek figure is a peak-rate upper bound that over-reads its off-peak calls by up to twice.
  That includes the 0.278706 USD booked by `XingZ6020`.
- Limits kept:
  a call that spans a window edge is booked at its start;
  a Chinese make-up working Saturday counts as off peak,
  because the page says Monday through Friday.
- The question this section left to the owner,
  whether booking at peak when a call falls in off-peak hours is wanted,
  has no owner ruling recorded in this record;
  the change in progress books each call at the rate of its own hour.

Note of 2026-10-25 (UTC),
added with the third round of the fix,
which merges `main` into its branch:
the rule as built differs from the bullets "The rule" and "Limits kept" of the update under this heading,
which are kept as written.
A call is booked at the card's peak rates when its request started,
or its reply completed,
inside a weekday peak window in UTC,
and at half of them only when both instants lie outside every window;
a call that crosses a window edge is no longer booked at its start only.
Not bounded:
a make-up working Saturday,
and a call that spans a whole window with both instants outside it.
The current rule is in `package/translation-repair/provider/src/direct-cost.ts`
and in ledger B920 (`package/translation-repair/doc/audit-ledger-deepseek-off-peak-pricing.md`).

### N9: the page keeps the archive's Chinese translator comment and its invented sentence

Check:
confirmed,
by design.

- Shipped page line 52 and archive line 39 are the same line,
  carrying `<!-- 前面这句中文懂什么意思，先这么翻译，大家看一下 -->` and
  "Trans aid work had always been a part in her daily life.";
  the source holds two HTML comments,
  the shipped page one.
  `doc/design-commitments-shipping-and-the-archive.md` says "HTML comments stay:
  `entry-notes.ts` reads them as editor
  comments and a reader never sees them",
  and the log reads it as `ARCHIVE NOTE ... reading=advisory` (line 202).
  Not a defect against a recorded rule.
  The invented sentence is the archive's;
  no claim filed against it in the run was traced by the check.

### N10: page consistency heard one voice

Check:
confirmed,
known in kind.

- Lines 21838 to 21850:
  `page-consistency: bench short of quorum, reachable 1 of 7; closing on 2 voices`,
  `1/7 heard ... no quorum (1 of 2 needed)`;
  assembly rewrote 36 slices and withdrew none (line 21892).
  The name and pronoun differences on the page therefore survive.
- Ledger:
  the consistency ledgers discuss lost voices
  (`doc/audit-ledger-consistency-pass-voices-lost-and-places-left-unlocated.md:191`)
  but none records a one-voice round as intended or as a defect.
  Known in kind (short benches),
  new as a measurement.
  Whether a one-voice consistency round should still run was not traced.

### Also noted

Qwen cut streams persist
(34 of 196,
17 percent,
5,605 s summed,
none in `SPEND`),
five of them in the 2 percent regen window,
so the post-quorum wait of D5 is still a cost
(21 grace minutes after the dry reading against 151 before;
first reader only).
The 63 `PROMPT-REUSE` lines and the 19 `schema-mismatch on neuralwatt` re-asks are normal traffic,
not defects.

## What the second reader changed

The check's corrections to the read,
all applied in the three files of this record:

- Lines without a level or blank are 239 (228 plus 11 blank),
  not 240.
- The wire id of DeepSeek's own key is `deepseek-flash` (`provider/src/direct-client.ts:292`);
  the roster id is `deepseek-v4.1-flash`;
  the link to `deepseek-v4.1-flash-1017` is DeepSeek's documentation only,
  and nothing in the source maps to it.
- The split of the 225 `needs-human` adjudications:
  224 had exactly two panel voices,
  1 had four,
  none had one or five,
  and the log does print the weights.
  The 58 unexplained adjudications and the "41 more named one voice" do not exist.
- Panel rounds on 4 of 8 are 42,
  not 49 (the 7 rounds that heard 3 are counted separately).
- `produceConsolidations` rounds on 2 of 9 are 86,
  with 4 on 3 of 9 and 1 on 4 of 9,
  not 91 on 2 of 9.
- N4:
  slice 101 had two usable gate ballots;
  the other five had one.
- The Shinonome source section is lines 237 to 251,
  not 238 to 250,
  and the summary has 93 characters.
- N7 is by design;
  N3 and N4 are known (B835);
  N5 is known in kind (B829);
  N6 is known by owner ruling;
  N9 is by design;
  N10 is known in kind.
- The 5,773 against 5,770 "no provider can take" lines is explained by three `reaskElsewhereNudged` warns
  (lines 912,
  1235 and 1700).
- N8 is explained by the peak pricing.
- The consolidation changed 6 slices and five of them went through a single gate ballot,
  not all six.

What the check added that the read missed:
slice 70 as part of the Shinonome failure
(seven claims,
seven `needs-human` adjudications and the lone judge's line 19764);
the archive-dispute path that cannot fire when the panel cannot accept a claim;
the empty `silentStages` record;
the second absolute floor `NON_TRANSLATION_BLOCK_VOTES = 3`;
and the panel decision lines that print the weights.

## Points the check did not examine, and points no log settles

First reader only,
not recomputed by the check:

- The repair lane's "88 computed,
  median 35.5 s"
  (the check's median over all 118 closes is 21.8 s,
  and the computed-only median is unverified),
  the grace-minute figures (151 before and 21 after the dry reading) and the Qwen cut statistics
  other than the count of 34.
- The personal sections of the page (tense,
  pronouns,
  names) beyond the Shinonome section,
  the HTML tag balance and the footnote counts.
  The read's quotations of the page at page lines 188,
  276,
  208,
  360 to 368 and 387 were not compared with the source by the check.
- The 51,
  31 and 6 slice counts by lane relation,
  and the energy estimate ratios.

Not determined by either reader:

- What became of the five consolidation-polish refinements that won (lines 18923,
  20856,
  20911,
  21277 and 21432).
- Whether the low rate of repair edits (14 of 118) is the dry credit or the faster Claude bench:
  the repair chunk pass straddles 00:01:55 UTC,
  and no arm of this run had a wet Synthetic throughout.
- Which of the 22 head slices that have no `decideBestCandidate` line took the archive text unselected;
  the table shows 66 head winners.
- Why the estimator spread of `glm-5.3-flash` is smaller on this build than in `XingZ6020`.
- Whether the Synthetic weekly credit was drawn by other users of the same key;
  the log has only this run's calls.
- The bodies of the 15 zero-content completed streams (14 NeuralWatt,
  1 DeepSeek).
- The unit name `corpus-pass-corpus-pass-XingZ6021` and the unit's exit status:
  both come from the brief of the read,
  not from `systemctl`;
  the log's `Finished in 7560.86s` is what the read saw.
- The full text of the owner's questions of 2026-10-25,
  the content of the change in progress,
  and the text of the re-check finding R2:
  none was among the filer's inputs.
- The read of `XingZ6020`,
  whose figures this record quotes for comparison,
  was not filed when this record was made;
  it is now filed as checked in
  [the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md)
  and the three files beside it.

## Figures from the brief of the read that were checked

- Confirmed:
  `START` at line 196 with tip `bcff23b8ce4179b56b7ab4029ce35c2dd06cfd47` and pipeline `sha256-tree-v1:899b55a6...`;
  entry `XingZ60`,
  one entry pending;
  `DONE` at line 21931 `processed=1 of pending=1; artifacts=1/1 declined=0`;
  `elapsed=7544901ms`;
  118 slices.
- Confirmed:
  NeuralWatt's 550 Wh day started 19:15:08 UTC on 2026-10-24 (line 204).
- Confirmed:
  Synthetic dry at 00:01:55 UTC (line 5431).
