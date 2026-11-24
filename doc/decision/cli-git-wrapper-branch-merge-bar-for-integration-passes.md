# Cli git wrapper branch merge bar for integration passes

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 46132 to 46132 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Seven further things,
  none of them a new write:
  by the diff of `a-session-file-this-run-may-rewrite.ts`,
  `main` rewrites at rest any session file Pi names,
  and the branch only its own copy,
  the file the caller named,
  or a file born after the run started.
  So the branch narrows,
  which is what the owner's "Merge when shown to narrow" asks.

From lines 46503 to 46507 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


Seven answers of the owner,
asked through the question tool when the owner wrote "What other questions do you need to ask me now?"
(answered between 2026-11-16T11:22 and 11:25,
read from the clock at 11:25:32;
each quoted as chosen):

- **When a wrapper branch may go into a pass:
  "Narrowing,
  in general (Recommended)".**
  A branch goes into a pass once a review that built it shows it fixes something `main` has and its remaining findings are all things `main` has too or does worse.
  Open findings go back to the author as follow-up on `main`.
  A regression is caught by the pass's two whole suites and the gate.
  What follows for the work:
  each review and each author states,
  per finding,
  whether `main` has it;
  a finding the branch introduces still stops it.

From lines 46566 to 46581 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### Answers of the evening (2026-11-15, answered between 19:09 and 19:14, written here at 19:16:23-05:00)

Three answers through the question tool,
each quoted as chosen.

- **"Merge when shown to narrow (Recommended)"**:
  a fix that narrows a data-loss window `main` already has goes into a pass once
  a test fails on `main` and passes on the branch,
  and the newest review that built and ran it reproduced nothing `main` does not also have.
  What that review still finds stays with the branch's author and follows in later passes.
  Asked about the commit that lands with the empty tree;
  the option's words cover any such fix.
  The session's reading,
  narrow:
  a thing a review reproduces at the tip that `main` does not have is new with the branch and holds the merge;
  a review that built and ran nothing satisfies neither half;
  the masking of credentials is not under this rule,
  because "Keep waiting for masking" is the newer answer on that branch alone.

From lines 46594 to 46610 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### Answers of the afternoon (2026-11-15, asked 14:01, answered about 16:15, written here at 16:22:30-05:00)

Two answers through the question tool,
each quoted as chosen.

- **A finding in test support that can signal a process the harness no longer holds**:
  "Merge now,
  fix after".
  Asked of the shared wait of the binary controls (`e12b36fd81`),
  whose review read that the kill at a test's deadline can go to a number after its process was reaped.
  The coordinating session had held the branch back and recommended that;
  the owner chose the other way.
  So the rule of the morning stands without that exception:
  a wrapper branch whose only open findings are in test code or test support goes into a pass,
  this one at `e12b36fd81`,
  and the author's fix follows in commits of its own.
  The session's adoption recorded at 14:00:27 is withdrawn.

From lines 46649 to 46659 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A branch with findings still open**:
  "Yes,
  tests too".
  A branch of the wrapper goes into an integration pass while the only findings its reviews left open are in doc comments or in test code;
  those are finished and reviewed afterwards,
  in commits of their own.
  Read narrowly by the coordinating session:
  the code that is not a test has had a review that found no defect in it,
  and new code that is not a test still has its review before a pass.
  Asked about the wrapper only;
  the driver,
  mpm and cargo keep the earlier rule.
