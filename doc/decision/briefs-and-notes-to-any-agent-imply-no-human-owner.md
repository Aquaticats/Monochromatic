# Briefs and notes to any agent imply no human owner

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 30266 to 30298 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Prompts to Pi imply no human owner (2026-11-11T18:12:12-05:00)

The owner,
at about 18:07:
"From now on,
in prompts to Pi,
don't imply there is a human owner."
It covers everything the coordinator sends a Pi run:
briefs,
inbox notes,
wake notes and the shared rules a brief points at.
A requirement is stated as the project's own,
with the path of its record when one helps,
never as a person's words,
wishes or approval ("the owner said",
"its owner requires",
"I will review").
What changed:

- The shared rules a brief points at are now `package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`,
  in the workspace and without an owner in them;
  new briefs name that file,
  by its path in the main checkout,
  in place of `~/temp/agent/pi-runs/shared-rules-for-pi-briefs.md` (left as it is for the runs that already read it).
- The round 231 review briefs are written one by one by a starter that was already running,
  each with "Its owner requires:"
  and "the owner's session bus".
  A one-time job rewrites each as it appears,
  before its review reads it
  (`~/temp/agent/reword-round-231-review-briefs/`);
  the seven reviews that had already started read the old wording.
- Runs briefed before 18:07 keep the briefs they have.
- Not covered by this change,
  and Pi still reads them:
  `AGENTS.md`,
  this handover and other repository docs,
  which speak of the owner throughout.
  Asked how far the instruction reaches,
  the owner picked "Prompts I send only (Recommended)",
  so those stay as they are.
- Asked whether to write it into `AGENTS.md` as a new rule plus an amendment,
  the owner answered "Amend BRQ only."
  The first amendment said "to Pi,";
  the owner,
  on reading it:
  "\"to Pi,\" - too specific."
  So rule `BRQ` now says it of every brief and note,
  whichever agent it goes to:
  a requirement is stated as the project's own,
  with its record's path,
  implying no human owner.
  That covers Claude Code native subagents and reviewers of other vendors as well as Pi.
  The owner then:
  "You may change DQB."
  Rule `DQB` told a brief to "quote every owner decision and amendment";
  it now says to state every decision and amendment that bears on the task as the project's own,
  with path and lines.
- The scratch brief writers (`write-kimi-findings-briefs-round-159.ts`,
  `write-kimi-review.ts` and the others) still put the old wording into what they write.
  They may no longer be edited where they are,
  so their output is reworded as data before a run starts,
  until the brief writers live in the workspace package.
