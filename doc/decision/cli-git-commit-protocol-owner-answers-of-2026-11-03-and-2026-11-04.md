# Cli git commit protocol owner answers of 2026 11 03 and 2026 11 04

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 46753 to 46754 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Commit design level 2 on the path where its eligibility cannot be shown:
  "Keep level 2 off (Recommended)".

From lines 46837 to 46841 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  - The commit design's R-never (the lane never removes an `index.lock` its own dead process left):
    "Reclaim it (Recommended)":
    the new protocol reclaims a proven-stale lock of its own dead incarnation through the shipped mechanism once that passes review;
    revision 19 is to be amended (coordinator),
    and the models agent was told.

From lines 47054 to 47061 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  - Commit notices (commit protocol plan question 4):
    "Line plus what was kept"
    (option B,
    not the ranked-first one-line option:
    every outcome prints its first line,
    then the indented `kept:` lines and their siblings from the drafts;
    command names stay placeholders).
    Sent to the models agent `a9c775c9196f1d836` as decision 24.

From lines 47522 to 47530 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Owner answer (question tool,
  2026-11-04):
  "Keep built,
  state limit (Recommended)"
  (keep release,
  recheck and exit;
  the helper count is bounded only by the commits that arrive during those milliseconds,
  overriding the reviewer's limit;
  the plain launch bugs still get fixed).

From lines 47611 to 47618 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Owner answers (question tool,
  2026-11-04):
  state document "Checked format (Recommended)";
  launch route "Systemd unit + fallback (Recommended)" (lifts the "manager-started route,
  until built" exclusion);
  cutover fence "New lock name (Recommended)".
  The notice-text question waits for drafted example texts;
  step 1 (the real state document) is the next build.

From lines 47732 to 47738 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Merge order:
  "Branch first,
  files after (Recommended)"
  (the branch moves first and the files follow;
  at the bound the folder and staging are untouched;
  the owner was told a direct `/usr/bin/git commit -a` in the lag window would commit the merge's reverse).
  This replaces the coordinator's delegated pick M-A.

From lines 47739 to 47744 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The person's own programs before the commit exists (editor,
  hooks,
  signer,
  forwarded `gc`):
  "No limit;
  Ctrl-C ends it (Recommended)".

From lines 47756 to 47771 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Commit protocol question 8 answer

Asked at 2026-11-04T05:07-05:00 through the question tool,
after the review of revision 19 (`commits-never-fail-sol-19a.txt`) found it "accurate after one small precision edit",
which was applied (`b98b3cad18`).
The owner picked "Retry it,
leave a notice (Recommended)":
when recovery after a power loss finds no decisive positive authority for a request that had returned success,
it makes a fresh attempt and the next wrapper command in that worktree prints an outcome-specific stored notice
(published with the new commit's id and the command to drop it,
or why it stopped).
The rare case where a commit removed on purpose comes back,
not promised to happen only once,
is accepted.
Recorded as decision 19 in the design.

From lines 47772 to 47798 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Commit protocol revision 15 answers

The owner,
through the question tool,
before the commit that records this entry,
answered the four questions of revision 15 (`4a6ad9d34a`) as the coordinator worded them after applying the fifteenth review's named edits
(`commits-never-fail-sol-15.txt` section 3;
the edited wording was not re-reviewed):
- Question 5:
  "Carry the file,
  save staged (Recommended)".
- Question 7a:
  "Clean up automatically (Recommended)"
  (by value tests within 7b's window under `index.lock`,
  current version saved first;
  "possible leftover" when any test fails).
- Question 7b:
  "A measured span (Recommended)"
  (starts when the merge's files are written;
  at its end no new cleanup starts;
  the value comes back to the owner before shipping).
- Question 7c:
  "Keep a reminder (Recommended)"
  (dismissing deletes no saved copy).
These supersede the provisional decisions 12 and 13.
Sent to the revision 16 agent.

From lines 47825 to 47859 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Commit protocol revision 13 answers

The thirteenth review (`commits-never-fail-sol-13.txt`) found questions 6 and 7 framed inaccurately:
question 6 omitted that a departure cut short by its limit can lose the record that forbids further attempts (so a withdrawn request could still land after a power cut),
and question 7's option A claimed it never undoes a later intentional change,
which a `reset --soft` plus switch disproves.
Answers 6 and 7 are therefore provisional until revision 14 asks them again;
answer 5 stands,
with its missing option to be shown.


The owner,
through the question tool,
answered before the commit that records this entry
(design revision 13,
`cc5e0822dc`,
section "Owner questions for revision 13"):
- Staged content differing from the file at a merge:
  "Carry the file,
  save staged (Recommended)"
  (the file as seen is carried;
  the staged-only version is saved to `saved/<path>.staged` and named in a line).
- A slow force at leaving:
  "Short wait,
  then say unconfirmed (Recommended)"
  (wait up to a named,
  measured limit,
  then exit saying the saved copy is not yet confirmed on disk while the wrapper keeps forcing it).
- How long to watch for a late `git switch` after a merge:
  "A measured span,
  then stop (Recommended)"
  (after the span the merge's files stay as changes,
  nothing lost,
  and the merge's message names how to undo them).

From lines 47966 to 47970 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

"Save on the side ref (Recommended)" for a branch a foreign program keeps moving;
"Leave out wrapper refs (Recommended)" for `git push --mirror` (everything except `refs/cli-git/*`,
with a line saying so);
"Amend the pending commit (Recommended)" for an amend while commits are pending.

From lines 36232 to 36234 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

Owner,
2026-11-03:
"Why did you cap the commit service to the magic number of 1GB of memory w/o evidence?"

From lines 36700 to 36702 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

with no PID file,
may be removed by the service (today never:
the owner picked "kept" for locks without a PID file);

From lines 46719 to 46724 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Mutation runs:
  "Pause all (Recommended)",
  after his own question,
  "We're not close to doing mutation runs though,
  right?
  Don't we still have a lot of things to fix before doing them?":

From lines 46772 to 46774 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Commit-speed picks (no `fast-import` pack flush unless `core.fsync` names loose objects;
  `sync_ms` renamed `index_update_ms`):
  "Keep both (Recommended)".

From lines 46955 to 46957 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Owner answer (question tool,
  2026-11-04) on the open pick:
  "Own format only (Recommended)"

From lines 48994 to 48994 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

and the widening "I'm widening the pnpm spawning exemption during worktree copy ops to all pkg managers."

From lines 49143 to 49145 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

QWD,
2026-11-04:
"Tighten QWD (Recommended)" (`3abcbf585e`).

From lines 49160 to 49163 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

Issue 6261,
written in the issue at 2026-11-04T21:30:23Z
(<https://github.com/Aquaticats/Monochromatic/issues/6261#issuecomment-6642606105>):
"The current layout is still completely unacceptable and far from enough to consider this issue complete."
