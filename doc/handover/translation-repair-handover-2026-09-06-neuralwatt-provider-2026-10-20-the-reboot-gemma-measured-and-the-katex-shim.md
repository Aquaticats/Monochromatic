# Translation repair handover 2026-09-06: the reboot, Gemma measured and the katex shim, 2026-10-21

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/shim/katex` is now `package/pnpm-override/shim-katex`.

The machine reboot with three agents halted,
the owner's rulings on the picture bench and on request parameters,
the Gemma measurement,
and the katex shim with the rounds behind it.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.
It continues [the NeuralWatt rotation reads and the first fix rounds](translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md).

The owner rebooted the machine on 2026-10-21 with three agents halted mid-task,
each told to write its state down first;
no pass,
gate or sweep unit was running.
Each resumed from a handover of its own kept outside the repository,
and all three have since finished:

- the w257 follow-ups
  (composite evidence,
  the source-context lookup,
  the launch-line note)
  landed as `0f80b593ef`;
- the picture bench investigation became the w258 fix,
  landed as `2c5f812492`;
- the tracing of the two reads' findings sent classes B and A1,
  with the owner's two rulings of 2026-10-21,
  to the w259 round.

The reads themselves are recorded in
[the rotation restart record](../planning/translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

After the reboot:

- The w257 follow-ups finished at `fc61e60462`
  (B716 composite evidence,
  B717 source-context lookup,
  B713 amended:
  two files carried the launch-line note,
  not five)
  and were merged with main as `e39e7b163a`;
  gate g139 runs on that tree,
  and it lands when green.
- The picture bench fix runs in w258 from main:
  a length-cut empty reading is asked again,
  NeuralWatt alone gets a documented lower reasoning effort for pictures,
  `google/gemma-4-26b-a4b-it` on OpenRouter is prepared as a reader behind its measurement probe,
  and the picture energy estimate stops counting base64 as text.
  Its ledger entries start at B718.
  The keyed measurement of Gemma is mine to run once the probe lands.
- The tracing of the two reads' findings resumed from its handover section.
- The reads are recorded in
  [the rotation restart record](../planning/translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md)
  (`90ffed222a`).

Next:
land g139;
gate the picture bench fix and any classes the tracing finds;
post corrective comments on `4e0cbc1b5f`,
`e0dbf84a38`,
`49194af623` and `96d9d0be32` once they are on GitHub;
rebuild the branch worktree,
freeze the new digest,
run the rotation sweep,
measure Gemma,
and restart the rotation at `zheermao101`.

Owner rulings of 2026-10-21,
given just before the reboot:

- On the picture bench:
  "Yes try gemma".
  So `google/gemma-4-26b-a4b-it` on OpenRouter
  (zero data retention)
  is measured as a second picture reader from another family,
  over a sample of pictures,
  and seated if it reads them well.
- On request parameters:
  "NeuralWatt is a special case of all the providers:
  It implements what it documents correctly,
  almost always - I'm lifting the ban on passing params,
  but only for NeuralWatt."
  So NeuralWatt requests may now carry documented parameters,
  such as a lower reasoning effort for `glm-5.3-flash` on pictures:
  in `XingZ6016` its reply streamed 40652 reasoning characters and no content before the 18316-token cap cut it;
  that the default effort `max` caused this is a guess not yet checked against NeuralWatt's documentation.
  Every other provider still gets no such parameter.

Also queued:
an empty reply cut at the length cap is asked again
(today only refusals are).
The pass log of 2026-10-21 no longer fails its two scans (`36159b960f`),
and the reads of the three restarted passes are recorded in
[the rotation restart record](../planning/translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md)
(`90ffed222a`).
An investigation of the picture bench runs
(why GLM replied empty,
which wet vision readers could replace the Bedrock seats);
its fix joins w257 before gate g139.
The rotation had restarted on the g138 build without waiting for them:
the day's free energy is spent either way,
and a read on that build still finds defects.

Gemma measured,
2026-10-21:

- The picture bench fix landed as `2c5f812492` after gate g141
  (2721 pass,
  0 fail,
  every step exit 0),
  and the branch worktree was rebuilt to it.
- The keyed `reader-measure` bench measured `gemma-4-26b-a4b-it` on OpenRouter
  (zero data retention)
  three times,
  nine pictures at most each.
  The first two runs
  (08:44 and 08:48 UTC)
  named the recent corpus-pass runs directories,
  the second adding the full-corpus pass of 2026-08-19;
  both found 3 trusted records,
  all for `zheermao101`,
  and ended "not measured":
  a verdict needs 6 readings.
  A count of every stored picture reading on disk showed why:
  the recent runs directories hold only those 3,
  while 107 older single-entry runs directories hold 192 distinct trusted readings across 10 entries.
- The third run
  (09:02 UTC)
  named those older directories:
  9 pictures matched,
  9 readings produced,
  and the result line said both clauses were met,
  against a seated floor of 1.00 corroborated by all and a ceiling of 0.00 corroborated by none,
  so every Gemma reading was corroborated by every seated reading of its picture.
  Limit:
  the 9 pictures come from two entries,
  `Mio` 7 and `Toka_ls` 2.
  Gemma calls spent: 15.
- The first two runs' NEXT line advised a larger `--sample`
  when the input that limited them was the number of trusted records;
  and no line printed Gemma's own rates.
  Both are bench report defects.
- The w260 round lifts the hold `openrouter-pictures-unmeasured`,
  bumps the picture-reading cache version,
  fixes the two report defects with the matched pictures' entries printed too,
  and records it all;
  its ledger numbers are B800 to B809.

Next,
in order:
review and gate w259 and w260
and land each;
rebuild the branch worktree,
freeze the new digest in the launcher,
run the rotation sweep,
and restart the rotation at `zheermao101`.

The sweep's salts covering a bench of at most nine seats is already recorded
(`package/translation-repair/doc/audit-ledger-output-decided-by-arrival-order.md`).

The katex shim and the rounds behind it,
2026-10-21:

- w259 landed as `6d525ca189` after gate g142
  (2732 pass,
  0 fail)
  and a clean rotation sweep.
- w260's gate g143 was green,
  but its rotation sweep failed at load under salts 4 to 9,
  every file with `ERR_MODULE_NOT_FOUND` for `katex`.
  The cause was outside the repository:
  scratch worktrees sat under a directory whose own `node_modules`,
  another session's npm install,
  had answered imports the workspace does not declare,
  and that install was rewritten without `katex` at 17:10:03 UTC
  (ledger B801 and B802).
- The w262 round replaced the override `'micromark-extension-math>katex': '-'`
  with a workspace shim,
  `package/shim/katex`,
  whose every member throws a named error;
  the corpus-pass build digest did not move.
  Its first gate in a worktree with no `node_modules` above it
  (`~/worktrees/tr-w262`)
  found a second undeclared resolution,
  in `consistency/src/consistency-shown-kinds.unit.test.ts`,
  fixed in `e32aed4101`.
  Gate g145 on that commit was green
  (2732 pass,
  0 fail)
  and its rotation sweep clean under salts 0 to 9.
- Main had moved its lockfile meanwhile,
  so w262 was merged with main as `b04b10b083`,
  and gate g146 runs on that merge before it lands.
- w260 was moved to `~/worktrees/tr-w260`
  and merged with `b04b10b083` as `1eabd8c9ff`
  (one index-list conflict,
  both sides kept).
- Gates and sweeps now run only in worktrees under `~/worktrees`,
  which has no `node_modules` above it;
  the gate script refuses any other worktree.
