# Design commitments: sheets, probes and audits

The commitments on the grading sheets,
the binding of a grade to its draw,
the introduced-defect probe,
and the audit of every stage that changes shipped text.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **Detection and repair are graded by separate instruments.**
  `formatGradingSheet` asks only whether an accepted issue is a real defect and shows no correction,
  because seeing one makes an alleged defect look more real and would move that answer.
  `formatRepairSheet` asks,
  on its own sheet and after the first is done,
  whether the returned wording fixes it.
  Keeping them apart is what lets one round's precision be compared with
  another's rather than with a changed instrument.
  Model and corpus text reaches those sheets fenced (`fenceForMarkdown`),
  since a replacement is arbitrary text crossing into
  Markdown grammar and can otherwise invent a heading or a grade box.
  A sheet is **read** before it is handed to anyone,
  item by item,
  including the reasoning it shows the grader.
  A sheet whose generator ran is not a sheet that asks a sensible question:
  the first introduced-defect verification sheet reached the user with all
  eight of its reviewer claims argued against the pre-edit translation rather than the original,
  one of them reporting a corrected mistranslation as damage,
  and nothing in the pipeline could have caught that because every stage had succeeded.
- **A grade is bound to the draw it was written on.**
  Sheets print no issue ids,
  deliberately,
  because a hash is noise a human has to read past,
  so grades are joined back to machine verdicts **by position**.
  Seed and corpus pin cannot carry that join alone:
  the draw is deterministic in its seed but not in its **pool**,
  and the pool grows with every entry that settles,
  so one seed at one commit names different items at different times.
  Two draws can then agree on seed,
  pin and item count while describing different issues,
  and a positional join would mislabel every verdict without erroring anywhere.
  `computeDrawDigest` fingerprints the ordered item identities,
  both sheets and the manifest carry it from one computation,
  and `parseSampleManifest` recomputes it rather than trusting the stored string,
  since a digest never checked against its own contents proves only that two files share characters.
  A draw taken before the binding existed is scoreable and says so;
  a pair where only one side carries a digest is refused,
  because one draw writes both in the same instant.
- **A new measurement records before it decides.**
  `runIntroducedDefectProbe` asks whether a repair broke something nobody raised,
  which `regressedKnownIssues` cannot see because it reads verdicts keyed by issues a critic already filed.
  It ships in shadow mode:
  the report reaches the outcome and the artifacts,
  and candidate selection does not read it.
  It was built expecting the opposite failure.
  Every region it inspects contains a defect by construction,
  that being why the region was edited,
  so a model asked whether anything is wrong was expected to find something,
  and gating on an over-eager probe would have discarded correct fixes.
  Measured on 2026-08-12 over all 857 probed regions of the settled artifacts,
  the probe was nearly silent instead:
  2438 of 2571 prober verdicts found nothing,
  and the raise rate barely moved with how much text the edit removed.
  **Read that as history,
  not as the probe's behaviour.**
  Those verdicts were produced under a question that made the pre-edit **translation** the standard of accuracy,
  asking whether the replacement introduced a defect the **before** text did not have.
  Read back,
  every claim it produced argued from that text,
  and one reported a corrected mistranslation as damage,
  so the figure measures whether an edit **changed** anything rather than whether it damaged anything.
  The question is now anchored on the **original**,
  and every probe figure taken before that change is withdrawn.
  Under the new question,
  on a twenty-region draw read against the Chinese,
  the probe flagged three of three damaged regions with one false positive and no misses,
  which is the first version of this instrument that discriminated at all.
  That reading is one agent's and one draw's,
  so it sizes nothing.
  A second reading on 2026-09-03,
  every region with a corroborated claim across the four probed landings,
  found six of ten true (three of them the house tense rule),
  three false (two misparse 我方才知道 as "our side") and one borderline.
  Shadow mode stands until a human grades a sample.
  What changed on that reading:
  the lane contest is shown the corroborated
  claims against the repair candidate as evidence lines after both candidates,
  with the four-in-ten error rate stated,
  and its cache key folds them in;
  nothing acts on a claim (`repair-damage-evidence.ts`).
  The claims come from both probes the lane runs,
  the accuracy repair's and the naturalness rewrite's,
  each line naming its edit:
  a keyword233 draw the same day had the rewrite move a paragraph into the present tense,
  three probers corroborated it,
  and the contest was shown nothing while only the accuracy probe was read.
  The contest driver logs how many claims a slice's judges were shown.
- **Every stage that changes shipped text is audited,
  including the last one.**
  The naturalness lane runs after the accuracy stage and rewrites whole slices,
  so the accuracy probe's verdict describes text the lane may have replaced.
  `retainsResolvedIssues` guards only the opposite direction,
  that a rewrite did not undo a confirmed repair,
  and a rewrite can leave every confirmed repair standing while damaging the wording around them.
  An accepted refinement therefore runs the same probe against its own pair,
  the repaired text against the refined text,
  recorded as `refinementDefects`
  and reported apart from the accuracy figures because the two audit different edits against different baselines.
  Its prompt gets a second framing:
  telling a prober that an editor was fixing defects,
  when it was rewriting already-correct text for fluency,
  invites reading every rephrasing as a failed repair.
  `probe-sensitivity` checks that framing against injected damage,
  and its control is the case that matters,
  since a probe that reads rephrasing as
  damage would flag every refinement the lane ships and would look identical to a clean run while doing it.
