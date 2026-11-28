# Rounds of one read that change the same sentences merge with no textual conflict and leave a tree that does not build

Three rounds of one review read,
each started from the same findings and each changing sentences the others read,
merged into `main` on 2026-11-27 with no textual conflict.
The tree holding all three did not build,
and two controls failed on words another round had changed.
Git's merge said nothing about either,
because each side's changed lines stood in places of their own.

## What happened

The read is the Claude Sonnet second pass over `mpm-shadow` `0.2.7-alpha.15`.
Three of its rounds merged in this order:

- the recovery's order,
  `cda9f0c7b3`,
  merged 2026-11-27T15:28:54-05:00;
- a publish raced between its two renames,
  `2c6513069b`,
  merged 2026-11-27T15:32:24-05:00;
- a hidden name nobody can judge,
  `7a216deafc`,
  merged 2026-11-27T15:39:08-05:00.

The merged tree did not build.
Then two controls failed on sentences another of the rounds had reworded.

## Why the merge said nothing

One round made the list of removed leftovers carry whether each held a finished copy,
and another round's control compared an entry in that list with a path.
One round reworded the sentence a publish prints when a recovery finished it,
and gave the removal a clause for a finished copy that went,
while two controls of another round asserted the earlier words.
Each of those changes stood on lines of its own,
so the merge took both sides of every file and reported no conflict.

## The fix

`af35ea923d`
(2026-11-27T15:38:54-05:00,
`test(mpm-shadow/cli): three controls read what the other rounds of the same read changed`)
changed three controls' lines and no release code:
the control with the path compares the entry's path now,
and the two that asserted the earlier words read the words the merged code prints.

## What keeps it from happening again

Branches of one package that ended together share one integration worktree,
and one run of the package's checks,
so a tree holding all of them is built and tested before any of them merges.
Reading both sides of a merge in the files each side touched is what showed,
on this merge,
that nothing of either side was lost.

This record is unapproved and non-final.
What was done is recorded here and in the commit `af35ea923d`.
What is open is the owner's review,
and the question of whether a driver task should run the merge steps a session does by hand.
