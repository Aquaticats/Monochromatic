# Preventing mistakes: commits on a branch and figures told to others

Rules for two checks that are made once and then relied on:
which commits a branch holds,
and the figures one session tells another.

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

## Figures told to others

What happened:
the lead told a peer session that a gate takes "about an hour" and had not measured it.
The measured times were 24 m 47 s for gate g223 and 19 m 44 s for gate g224,
and the lead sent a correction (ledger M303).

The rule:
a figure of time,
size or count that is told to another session or agent is read off a log or a command first,
and the message says which run it came from.
Where nothing was measured,
the message says that the figure is a guess or that it is not known.

What enforces it:
habit.
