# Preventing mistakes: commits on a branch and figures told to others

Rules for checks that are made once and then relied on:
which commits a branch holds,
the hashes given to commands,
and the figures,
causes and tool behaviours that one session tells another.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Which commits a branch holds

What happened:
on 2026-10-29 (UTC) the lead checked that the commits of round 8 were on `origin/main`,
before posting corrective comments on them,
and the first check was `git branch --contains A --contains B`.
That command lists a branch that holds any one of the commits,
and not only one that holds all of them (ledger M302).

The rule:
the presence of each commit on a branch is checked one commit at a time,
by `git merge-base --is-ancestor <sha> origin/main`,
which exits 0 when the commit is on the branch and 1 when it is not.
A positive control comes first:
a commit known to be on the branch exits 0.
A listing from `--contains` with two or more commits is never read as "the branch holds them all".
Measured on 2026-10-29 (UTC):
`git branch --remotes --contains 4cb480344f --contains bdee68d1d0` listed `origin/main`,
and `git merge-base --is-ancestor bdee68d1d0 origin/main` exited 1,
so `bdee68d1d0` was not on the branch the listing named.

What enforces it:
habit.

## Hashes given to a command

What happened:
on 2026-10-29 (UTC) the lead set up an integration tree for round w374
and gave the setup script a full hash for the round's head,
`1a6c9296b2bda8a6...`,
which the lead had made up from the ten characters it knew.
Git refused the merge with "not something we can merge",
and nothing changed.
The lead merged by hand with the short hash that git had printed (ledger M319).

The rule:
a hash given to a command,
a script or a brief is copied from the output of a git command run in that turn,
and never completed,
extended or typed from memory.
`git rev-parse <short hash>` expands a short hash to the full one.
A refused merge is read as a hash that git does not hold,
and the next try uses a hash read from git,
not another typed one.

What enforces it:
git refuses an object it does not hold,
which is how this slip was found;
a short hash typed from memory that happened to name another commit would not be refused,
so the rule is habit.

## Figures told to others

What happened:
the lead told a peer session that a gate takes "about an hour" and had not measured it.
The measured times were 24 m 47 s for gate g223 and 19 m 44 s for gate g224,
and the lead sent a correction (ledger M303).
On 2026-10-29 (UTC) the lead's summary said that 19 old worktrees remained.
A count by command gave 18 clean worktrees and `tr-w321`,
which held an install artifact in `pnpm-lock.yaml` (ledger M319).

The rule:
a figure of time,
size or count that is told to another session,
an agent or the owner,
or written into the handover,
is read off a log or a command first,
and the message says which run it came from.
A count of worktrees,
files or commits is made by a command in the turn that states it.
Where nothing was measured,
the message says that the figure is a guess or that it is not known.

What enforces it:
habit.

## Causes and tool behaviours told to others

What happened:
on 2026-10-29 (UTC) the lead told the owner or wrote into the notes five statements
before the reading that settles them,
and each was corrected (ledger M318).
Deleting another session's task output would "likely stop" an `EDQUOT` error,
and `quota` then showed the file far from the limit.
`toolchain:sync` was said not to compare an exposed binary's version with the lock,
and a peer then read its source and said it compares the shared cargo record with the lock.
A file's modify time was taken for its install time,
and its birth time was the install.
A count of 15 was called typography cases,
and it was 15 failing lines across the gate's suites.
A red step was read as a merge interaction,
and the round's own missing change caused it.

The rule:
a cause,
or a statement of what another tool does,
is told only after the command that measures it has run
or the source that decides it has been read,
and the message names which.
Where neither has happened the message says "not investigated" or "a guess",
and says what would settle it.
A quota is read with `quota`,
and the time a file was made on this machine is its birth time (`stat --format=%w`),
not its modify time.
A cause of a red step is read from the first failing line of the step's log,
and from the commit that wrote the text that line reads.
A retraction goes to the people who were told,
and says what the first statement rested on.

What enforces it:
habit.

## Evidence a ledger cites

What happened:
a probe script of the lead removed `sigprobe/signals.log` before it wrote new evidence,
and the file removed was the one ledger B1295 cites as its measurement (ledger B1341).

The rule:
a probe writes new evidence under new names and never removes a file a ledger may cite;
a rerun gets a new file and the entry that cites the old one is corrected in place if the old one must go,
with the original reading copied into the entry.

What enforces it:
habit and review only.
A scan cannot check a path outside the repository,
so the entry that cites such a file also states what it read,
which is what B1295 now does.
A script that a ledger cites by name lives in the folder of the round whose logs the entry cites,
or the entry names the folder it is in (`count-cases.ts` of B1327 is in `w405-fix`,
and the entry first said the folder of its own round's logs,
ledger B1359).

## A limit of a tool is measured on the file

What happened:
a ledger said that no lint rule caps a file at 300 lines and that the 300 of the lint is for `index.ts` files,
because the file in question passed at its length.
`eslint/max-lines` is 300 for code files,
counting neither blank lines nor comments,
and the file passed at 277
(ledger B1352).
The control for the measurement,
a file written twice,
was no control:
its errors were redeclarations and never reached the rule.

The rule:
a claim that a tool does or does not limit something is measured on the file,
with the tool and the configuration
the repository runs:
the count the rule would report (the rule at a limit of one line,
with the same skips),
and a positive control that the rule fires in that folder under that configuration (a file that must break it,
with distinct declarations so that no other error hides it).
The log of each is kept,
and a log is never overwritten by the next run.

What enforces it:
habit and review.

## What was covered is stated with what was not

What happened:
a round switched off 56 branches of a reader,
one at a time,
and said that every branch was switched off or covered.
36 failed a case at once,
5 of the 20 that survived got a case,
15 got none and were argued by reading,
and the
termination guard of a loop was not switched off (ledger B1327,
corrected in B1355).
The heading,
a note,
a test header and a commit message each said "every branch".

The rule:
a statement of what a check covered gives the number tried,
the number that failed the check,
the number that
survived with what was done for each,
and what was not tried and why.
"Every" is written only where the list it ranges over is the list tried.

What enforces it:
habit and review.

## A commit message that names an item or a count

What happened:
messages of round w406 named an item number two commits both claimed (item 14),
a count that a later read of the log
changed (two TSDoc descriptions where there were four),
"the previous commit" with no hash,
and a fix of over-width
lines that left some (ledger B1360).
A message is not amended,
so the error stays in the history.

The rule:
a message names a commit by its hash,
an item by the entry that holds it,
and a count after `git show --stat` or the
log has been read;
"the previous commit" is never written.
A message found wrong is corrected by a sentence with the full hash in the ledger of the round that found it,
and the lead posts the comment.

What enforces it:
the commit hook rejects nothing of this kind;
review and the ledger carry it.
