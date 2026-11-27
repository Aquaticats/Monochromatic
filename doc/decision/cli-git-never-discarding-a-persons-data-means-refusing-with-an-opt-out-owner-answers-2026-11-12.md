# Cli git never discarding a persons data means refusing with an opt out owner answers 2026 11 12

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 27047 to 27143 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner on what "never discard user data" means, and on deleting after a merge (2026-11-12, about 10:33, typed while the answers of 10:32:10 were being recorded)

The owner's message,
whole:

"I do want to say "never discard user data" means to block potentially destructive commands with a --no-enforce-* opt-out,
not that we save everything myself.
And wthe merge + delete one - the delete should still be able to delete async because the wrapper has the data of what is there and what is not there and what is pushed and what is local."

What it settles:

- The rule's meaning:
  a command that could destroy a person's data is refused,
  and the person runs it anyway with the guard's `--no-enforce-*` option.
  It does not mean that the wrapper saves a copy of everything and lets the command run.
  This session had read it the other way since 2026-11-11
  (the recorded reason "a refusal only moves the same loss to `rm -rf`" was used to turn refusals into saved copies),
  and briefed agents that way:
  the discard guard,
  abort and skip,
  the exempt repository,
  the fast-forward carry.
- The removal of a landed branch's worktree stays asynchronous.
  Its safety comes from what the wrapper already knows:
  what was there,
  what was not,
  what is pushed and what exists only locally.
  The fix makes the deferred deletion check against that knowledge;
  it does not move the deletion back into the merge.

What it leaves to ask,
because two answers given five minutes earlier chose the saving behaviour as it was put to the owner
("Ship as built" for the discard guard at 10:27:45,
"Save and go on" for abort and skip at 10:32:10):
whether those two become refusals with an option,
and how far the meaning reaches into work in flight.
Asked through the question tool in the same turn.

#### The owner's answers of 2026-11-12T10:35:07-05:00 to those three questions

- The discard guard,
  when a command would overwrite or delete a file that exists nowhere else:
  "Refuse with opt-out (Recommended)".
  This replaces "Ship as built" of 10:27:45.
  The command is refused and names the file;
  `--no-enforce-<rule>` runs it as real Git would,
  with no copy.
  The finished branch (`2f6ac5f6f0`) is reworked before it ships.
- Abort and skip,
  when they would discard work that exists nowhere else:
  "Refuse with opt-out (Recommended)".
  This replaces "Save and go on" of 10:32:10.
- How far the meaning reaches:
  "Redirect everything".
  The option as it was put:
  the agents in flight are told to protect by refusing with a `--no-enforce-*` opt-out,
  to add no new saved copies and to keep existing refusals;
  "and agents are also started to turn shipped save-and-continue behaviour into refusals with an opt-out".
  The shipped behaviour named in the question:
  `clean` saving a nested repository it would delete,
  and a merge carrying a person's uncommitted edits.
  The cost named in the question:
  commands that work today start being refused,
  and it is a large change to make without looking at each case.

What stays as answered and is not touched by this:
`git clean -fX` and `-fx` of ignored files delete,
exit 0 and are logged ("Keep allow and log",
10:27:45);
a batch whose scratch reference cannot be proven is committed and says so (10:27:45);
a commit against a service that does not answer fails with a clear message (10:32:10);
a merge step hung in an LFS download is ended before the branch write (10:32:10).

The existing opt-outs the wrapper documents
(`package/git-policy/cli/README.md`):
`--no-enforce-worktree`,
`--no-enforce-worktree-branch`,
`--no-enforce-security`,
`--no-enforce-final-newline`,
`--no-enforce-bulk-add`,
and the generic `--no-enforce-<policy-id>` (`SPEC.md`,
"Generic `--no-enforce-<policy-id>` escapes").

#### What this session got wrong, and what it changes

- Since 2026-11-11 this session briefed agents to turn refusals into a saved copy and a command that runs,
  citing a sentence of the record ("a refusal only moves the same loss to `rm -rf`") as if it were the owner's rule.
  The rule is the owner's sentence in this section.
  Every brief and note from now on states it in these words:
  a command that could destroy a person's data is refused and names what it would destroy,
  and the guard's `--no-enforce-*` option runs it as real Git would;
  the wrapper keeps no copy.
- Proposed for `package/git-policy/cli/SPEC.md`,
  where a reader of the package meets it
  (a rule about one package goes in that package's documents):
  a section "What never discarding a person's data means" with the owner's sentence,
  the list of guards and their opt-outs,
  and the three recorded exceptions listed here.
  It is written by the agent that inventories the shipped behaviour (round 296).

From lines 27235 to 27285 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner's answers of 2026-11-12T10:32:10-05:00, through the question tool: four decisions taken under delegation, two confirmed and two overturned

The item numbers are those of `~/temp/agent/owner-questions-2026-11-12T1000/open-questions.md`.
(scratch,
deleted on 2026-11-27;
not kept).
Each had been decided by this session while the owner was away,
with the veto open.

- Item 1.7,
  abort and skip:
  "Save and go on (Recommended)".
  Confirmed:
  `--skip` saves the hand resolution it discards and goes on,
  content that stands only in the index is saved and the abort goes on,
  a refusal stays only for content that cannot be read or written,
  and the saved paths are compared again a bounded number of rounds before Git starts.
  The branches leave the hold-out list when the third pass is done
  (the read that hangs on a FIFO).
- Item 1.8,
  a merge step hung in an LFS download:
  "End it before the write (Recommended)".
  Confirmed:
  ended where the branch has not been written yet,
  otherwise owned until it ends;
  "Never cut a branch write" holds for the write itself.
- Item 1.9,
  what a push prints of an LFS URL:
  "Mask recognised secrets only".
  Overturned.
  This session's decision had been to hide user information,
  path,
  query and fragment.
  The owner's:
  a URL is shown,
  and only what is recognised as a secret in it is masked.
  The cost was stated in the question:
  a secret of a shape the mask does not know is printed.
  The branch `fix/git-policy-cli-review-findings-lfs-output-masks-every-secret` (`cda54f2d9e`) has to be changed before it can go anywhere.
- Item 1.10,
  a commit when the commit service has stopped reading or answering:
  "Fail with a clear message".
  Overturned.
  This session's decision had been a second way of making the commit that needs no service.
  The owner's:
  one way of making a commit;
  when the service does not answer,
  the commit fails after a wait and says so clearly.
  The branch `fix/git-policy-cli-commit-is-made-when-the-service-stops-reading-or-answering` (`3b33af6478`) is superseded as a whole;
  what an audit found still stands and still has to be fixed:
  one case fails only after 10.1 s,
  the other blocks in the request write with no bound.

Thirty items of that list are still to ask.

From lines 27286 to 27328 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner's answers of 2026-11-12T10:27:45-05:00, through the question tool: four of the parked questions

Each question as asked is in the session's transcript;
the item numbers are those of `~/temp/agent/owner-questions-2026-11-12T1000/open-questions.md`.
(scratch,
deleted on 2026-11-27;
not kept).

- Item 1.1,
  the hand-over of a landed worktree's removal to the commit service,
  which can delete what is written into the landed worktree in a short window:
  "Keep until alpha.14 (Recommended)".
  Nothing changes;
  the fix goes out with alpha.14,
  and no agent is restarted at a landed path without a check for a pending removal.
- Item 1.3,
  `git clean -fX` and `-fx` deleting the only copy of an ignored file:
  "Keep allow and log".
  The answer of 2026-11-10 stands against the audits:
  these two forms delete,
  exit 0 and are logged,
  and no copy is saved.
  The option "Save a copy first" was the recommended one and was not picked.
  Sent as a note to the agents working on clean and on the discard guard,
  so that none adds a copy for these forms.
- Item 1.5,
  a batch that fails when the commit service cannot prove the write of its scratch reference:
  "Commit and say so (Recommended)".
  The commit is made as real Git would make it,
  with a line that says the extra protection was not in place for it.
  The reading the recommendation rested on
  (the window is the one real Git has on every commit)
  was not checked against the code when the question was put;
  the agent that builds it checks it first and reports if it does not hold.
- Item 1.6,
  the discard guard:
  "Ship as built (Recommended)".
  The branch leaves the list of what is held out of releases
  once its review and its last hole (`git read-tree -m -u`) are done,
  with autocorrection switched off for mistyped commands and the `--f` forms refused,
  as the owner was shown.
  Where the branch would save a copy for `clean -fX` or `-fx` of ignored files,
  item 1.3's answer wins.

Thirty-four items of that list are still to ask.
