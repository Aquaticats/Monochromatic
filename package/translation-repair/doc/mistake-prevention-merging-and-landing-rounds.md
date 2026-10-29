# Preventing mistakes: merging and landing rounds

Rules for what the merge of a round keeps,
carries and breaks,
and for the tree a round lands from.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
The older rules for work written apart are in
[Work merged from parallel briefs](mistake-prevention-shell-edits-commits-and-agents.md#work-merged-from-parallel-briefs);
this file holds the rules added from the rounds of 2026-10-29 (UTC).

## Lines a merge keeps or drops

What happened:
the agent of round w367 merged onto a tree and dropped the index line of B996 from `audit-ledger-entries-by-id.md`,
on the ground that main had deleted it.
Main had never had the line,
because round w363,
which added it,
had not landed.
The lead caught it and restored the line in a commit on the round's tree (ledger M323).

The rule:
a line that one side of a conflict lacks is not a deletion
unless the merge base held the line and that side removed it.
`git show <merge base>:<path>` shows whether the base held it.
An index conflict is resolved to the union of both sides' ids in id order,
and a brief for a merge says so.

What enforces it:
habit,
and the source-scan unit tests,
which fail on an id that heads two entries and on an id that no heading names.

## Constants that tests print

What happened:
round w369 moved the artifact generation to 22
and missed three run test files that print `read-at-generation=21`,
23 literals in all;
its hand-back listed no run of the run package's tests.
Round w371 had added a case that expects `read-at-generation=21`,
and the merge of the two rounds failed the same step again (ledger M323).
The artifact package's `generation: 21` cases are older-generation fixtures on purpose and passed.

The rule:
a round that moves a number that tests print as text
(a generation,
a version,
a count inside a log line)
searches every package's tests and fixtures for the old number's text with `rg` before its hand-back,
reads each hit,
and lists the packages searched.
The merge of two rounds repeats the search on the merged head,
since a case added by one round for the same number is found there and not in either round.
The whole gate on the merged head stays the check before landing.

What enforces it:
the gate's unit step of the run package found both;
the search is habit.

## Behaviour a merge carries outside its conflicts

What happened:
the merge agent of round w376 kept a `parked` hook that `702d149a74` had made dead,
and `lint:types` of corpus-pass failed with 4 errors until `00c80c7d0d`.
It also dropped a check of `passBoundOf` and the start of the day in `run-seats-futile.ts`,
because the pass bound was gone;
that the behaviour stayed the same was the agent's reading and not a run (ledger M323).

The rule:
an agent that resolves a merge runs the type check and the unit tests of every package a conflicting file belongs to
before its hand-back.
A break the merge caused gets a commit of its own,
named as the merge's.
A statement that carried behaviour is unchanged is backed by a test run,
or is labelled a reading,
and the gate is what confirms it.
A tree made fresh has no `dist`,
so the build runs before the type check.

What enforces it:
the lead's step 0 in the brief of a merge agent,
and the gate on the merged head.

## Landing a tree

What happened:
the lead's landing of round w368 failed on its first attempt,
before any build,
because the landing script ran under `env --ignore-environment PATH HOME`,
which drops `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS`,
and its `systemd-run --user` could not reach the user bus.
The merge it had made stayed unpushed in the tree.
A worktree was added while a batch retire was removing others,
and the cli-git hook failed resolving a tree mid-removal.
A helper was called before it was written.
The agent of round w374 ran `mise run sync:files` in its tree,
which wrote a forbidden-strings rules file of 11,064 bytes where the full file is 14,950,
and rewrote the tracked `mise.toml` (ledger M317 and M321).

The rule:
a landing script that starts a user unit runs with `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS` kept
beside `PATH` and `HOME`.
They hold no key,
and without them the script stops with "Failed to connect to user scope bus" before it builds anything.
No worktree is added while a retire runs.
If an add exits with an error from the hook,
the tree is checked with `git status --short` and finished after the retire ends,
since the add itself can succeed.
A helper is written before the command that names it is sent.
The size of `.cache/forbidden-strings.rules.txt` in the landing tree is read before a commit there,
and compared with the file of a tree known to be complete
(14,950 bytes on 2026-10-29).
A short file is replaced by the full one and read again.
The pre-push pass of cli-git rescans every blob that the parent of a pushed commit lacks,
with the landing tree's rules
(the section "Content already in history is not scanned again" of `package/git-policy/cli/README.md`),
so the landing tree's file is the one that counts.
`git status --short` is read after `mise run sync:files`,
and a tracked file it rewrote is restored.

What enforces it:
habit,
and the lead's landing script,
which is scratch tooling outside the repository.
