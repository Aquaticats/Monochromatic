# Design commitments: votes, repairs and run-offs

The commitments on how votes are sized and read,
what a repair patch keeps,
when a rewrite is rechecked,
and when a contest or a tie is run off,
under the owner's standing directive that the quality of the end result decides.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **A confirmation sizes its quorum on the seats it could ask and reach.**
  Owner,
  2026-09-27,
  asked whether the bench seats a naturalness confirmation may not ask
  (it asks only its discovery's seats) count as out of reach when its quorum is sized,
  as a seat the router refuses does under the 2026-09-09 short-bench rule:
  "Count as out of reach".
  With nothing refused,
  or with asked seats able to meet the bench quorum,
  nothing changes;
  in XingZ624's shape (5 of 8 asked,
  2 refused) the confirmation decides on 2 of its 3 reachable voices
  rather than closing `quorum-not-met` (ledger E3).
- **A panel verdict gives its reason before its vote.**
  Owner,
  2026-09-27,
  asked whether each issue-panel verdict carries a reason,
  and if so before or after the vote,
  told that no panel bench exists
  and that the last panel-sheet change moved support from 72% to 65% on one run per arm:
  "Reason before vote".
  Measured before the ruling over 265 stored artifacts:
  about 783 panel verdicts per entry,
  and 167 characters on average for the reasons other stages' ballots carry.
  The reason is stored with the ballot and logged beside the issue's decision (ledger E5).
  Its effect on votes is measured at the next run,
  TianqiChen666,
  as the share of panel ballots voting supported against TianqiChen66620's,
  one run per arm (`package/translation-repair/doc/status.md`,
  the 2026-09-27 section).
- **Every ballot and verdict gives its reason before its choice.**
  Owner,
  2026-10-21:
  "Reason first everywhere",
  extending the 2026-09-27 ruling from the panel verdict to the other ballots
  (full record in [the rulings of 2026-10-21](design-commitments-rulings-of-2026-10-21.md)).
- **A winning patch sheds the edits a checker voted worse,
  and the rest is rechecked.**
  Owner,
  2026-09-28,
  asked what happens to edits inside a selected repair patch whose issues the checkers did not confirm
  (345 of 2,148 patches over every run,
  18 of 51 on TianqiChen666;
  691 such issues:
  651 not fixed,
  39 on a fixed and worse tie,
  1 on a worse majority):
  "Revert worse-voted,
  recheck".
  An edit whose issue drew at least one worse ballot is stripped from the patch,
  and the reduced patch faces one more checker round before it ships;
  a not-fixed edit with no worse ballot stays (ledger L3).
  Corrected after the ruling:
  the question put the stripped share at 40 of the 691 issues,
  counting only the ties and the worse majority;
  at least one worse ballot is 58 issues in 31 patches over every run,
  and 15 issues in 6 of 51 patches on TianqiChen666 (`ride-along-split.mjs`,
  recounted 2026-09-28).
- **A rewrite after a lost patch is rechecked before it ships.**
  Owner,
  2026-09-28,
  asked what happens when the naturalness stage rewrites the archive
  after the accuracy patch lost,
  which shipped with no checker round
  (1,218 of 2,144 refined slices over every run;
  106 of 125 on TianqiChen666,
  75 over accepted issues):
  "Recheck the rewrite".
  The rewrite faces a checker round against the slice's accepted issues and the regression probe,
  and a rewrite the checkers find worse keeps the text before it (ledger L11).
  Decided for quality,
  2026-09-28,
  under the owner's standing directive that the quality of the end result decides:
  a rewrite the regression probe admits a claim against keeps the text before it too;
  175 of 2,144 kept rewrites carried one,
  and a graded reading of flagged regions found six of ten true.
- **Inside a licensed quote,
  an edit keeps the markup atoms its issue does not license it to remove.**
  Owner,
  2026-09-28,
  asked what the editor's preservation gate should protect inside a quote an edit may change,
  where it could never reject anything:
  "Markup atoms".
  Footnote references,
  link destinations,
  MDX expressions,
  inline code and tags survive every edit
  except a removal an addition issue names;
  prose damage inside the quote stays with the checkers (ledger L4).
  Refined for quality,
  2026-09-28,
  under the owner's standing directive that the quality of the end result decides:
  a replay of the ruling's wording refused 24 recorded edits,
  17 of which re-marked markup or moved it rather than lost it.
  Markup the source carries survives;
  an atom one edit drops and another edit of the same patch writes has survived;
  an MDX expression,
  inline code or tag the translation authored may be re-marked (same kind,
  or into markup the source carries)
  but never dropped;
  a footnote reference or link destination has no such excuse,
  since a label or address is an identifier
  no checker judges.
  The addition removal stays the one licensed loss.
- **A claim the panel settles at neutral asks for no edit;
  the sheets define every severity.**
  Decided for quality,
  2026-09-28,
  under the owner's standing directive that the quality of the end result decides (ledger L5):
  neutral asserts no defect,
  yet 28 accepted neutral issues shipped an edit,
  8 of them claims calling the rendering correct.
  The tally holds such an acceptance for a human;
  the critic and panel sheets define minor,
  major and critical after MQM
  and neutral as a finding a human should see that names no defect in the translation;
  the panel votes down a claim naming nothing wrong and lifts a real defect filed neutral to minor.
  This refines the 2026-09-27 ruling "Major+ accuracy" without contradicting it:
  that ruling concerned accuracy claims other than additions,
  and kept additions disputing "at any severity"
  as the option's wording said;
  an accepted addition can no longer settle at neutral,
  so in effect an addition disputes the archive from minor up,
  and one no supporter finds a defect in disputes nothing.
- **The quality of the end result decides;
  a choice made for it is not a design decision.**
  Owner,
  2026-09-28,
  standing directive,
  answering two questions put as design choices
  (which fix-shaped refusals the L4 markup gate should let through,
  and whether the L11 probe stays in shadow):
  "Do not try to save effort and just do the option that would result in best quality of the end result",
  and "I want to make 'prefer quality of the end result' a standing directive
  and decisions that are made for quality and don't conflict with other decisions aren't design decisions."
  A choice whose options differ in the quality of what ships is decided for quality,
  recorded,
  and built,
  never asked;
  effort,
  code size and wall clock are not reasons to take the lesser option.
  A question goes to the owner only when a quality choice conflicts with an earlier ruling
  or the options differ in something other than quality.
- **A slate over an archive most of the contest condemned is run off.**
  Owner,
  2026-10-11,
  answering what should trigger the run-off after TianqiChen66622 slice 4 shipped the archive's present-tense
  sentence over a contest whose ballots called it flawed 4 to 1:
  "Majority flawed yes".
  This replaces the 2026-09-24 rule ("Run-off only when every contest ballot called the archive flawed")
  for the run-off only;
  the forfeit of an undecided gate
  (2026-09-27,
  "Slate's choice")
  moves to majority too
  (owner,
  2026-10-11:
  "Majority,
  like the run-off").
- **A tie the standing drew no ballot in is run off among the tied.**
  Owner,
  2026-10-11,
  same slice,
  where the slate tied three ways at weight 1 and no judge chose the archive:
  "Run off the tied finalists".
  The standing still ships if that run-off ties again.
- **A polish gate that ties ships the polished text.**
  Owner,
  2026-10-21:
  "Polished text wins ties"
  (full record in [the rulings of 2026-10-21](design-commitments-rulings-of-2026-10-21.md)).
- **Two answering judges decide;
  no weight floor stands in their way;
  one judge alone never decides.**
  Owner,
  2026-10-25,
  after the corpus pass XingZ6021 shipped slices 67 to 69 with a quotation from Metal Gear Solid 2
  and two paragraphs the source does not hold,
  although the two panelists heard supported every critic claim against them:
  the panel tally asked for a summed weight of 3 (`minBallotWeight`),
  two panelists at the default weight of one summed to 2,
  and a unanimous panel left every claim for a human.
  Asked whether to keep that floor:
  "floor of 3 - don't set a floor."
  Asked what decides a claim that only one judge answers
  (and,
  by the same stroke,
  a claim two judges answer):
  "Two ballots decide (Recommended)".
  Asked how far this goes:
  "Every weight floor".
  So a claim or a contest is decided once two or more judges have answered it,
  by the shares of the weight that answered (the decision,
  source-defect and run-off ratios,
  which stay),
  and a summed weight is never compared with a number again.
  One judge alone still never decides,
  whatever it weighs:
  the older standing rule of the owner,
  restated here,
  which also keeps the lane contest at two seats and the slate at two judges.
  A self-vote counts as one answering judge,
  as the owner ruled on 2026-10-20 ("Count self-votes"),
  and its half weight only ranks candidates.
  What fell:
  the panel tally's `minBallotWeight` of 3,
  the candidate selection's minimum weight of 2 with its scaled short-bench minimum,
  the run-off and condemned-fallback exceptions that existed to get around that weight,
  the roster guard's weight arithmetic,
  and the count of three non-translation votes
  (now two critics heard,
  which cannot come from fewer than two critics;
  the contradiction check and the good-translation anchor remain the guards against a false block).
  A consolidation gate that heard fewer than two judges ships nothing past its neither on its own authority.
  The owner answered the open questions the same day:
  "Allow two checkers" (the checker count is two),
  "Keep waiting for half" (a gather above two voices and the bench waits stay),
  "Two judges must name it" (a declined round ships a candidate only when two or more judges named it,
  and otherwise the wording in place stays,
  or the source text under the note where none exists),
  "Ship the slate's wording" (a gate that hears fewer than two usable answers over an archive the publication rule
  refused ships the slate's pick when two or more slate ballots chose it),
  and "Ship the slate's pick (Recommended)" (the same for an eligible standing a majority of the contest judges
  called flawed).
  The lead ruled that a page title settles only on a rendering two or more voices gave
  (a title one voice alone rendered,
  or a tie of one voice apiece,
  stays unsettled)
  and that a consistency round decides on two or more answering ballots,
  and that the derivability probe and the restoration judge,
  which feed the benchmark scorecard only,
  stay as they are.
  The lead ruled,
  settled by "Two judges must name it",
  that a challenge round tied across every offered candidate ships by the fixed preference
  only among the candidates two or more judges named,
  and otherwise the wording in place stays by the path of the declined round.
  On the third re-check the lead ruled,
  by the same rules,
  that an issue counts as regressed only when two or more checker ballots name `worse`
  (the count ranks third in the candidate selection, so it can choose which candidate ships),
  that the sheet sentence about a declined round says declining does not by itself keep the wording
  and that a judge's own decline names no candidate,
  and that a page title round is stored only when every title it asked is settled.
  The stage quorum that sizes a round (a majority of the seated bench,
  two on a short bench) was not changed (ledger B900 to B919).
