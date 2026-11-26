# The wrapper's alpha.47 ZFS route: the owner's two answers on refusing a removal

Status: accepted by the owner on 2026-11-26, through the question tool (asked at about 12:55, answered by 16:10 -05:00).
Each question carried its finding, its options with their costs, and a ranking;
both were recorded first in `doc/handover/questions-for-the-owner.md`.

## A slot removal behind a global option the wrapper does not know

The finding:
the round that fixed the alpha.47 ZFS pass's High (merged as `6aeba4328d`)
made the slot removal and move refusals read every global option in the wrapper's copy of Git 2.56's option table
(`handle_options` in Git's `git.c`).
A global option a newer Git adds, typed before a relative slot path such as `../checkouts/slot-one`,
leaves the path unread, so the removal is forwarded and Git deletes the bound slot's registration while its record still says filled;
a slot named by its last name alone is still refused.

Answer:
"Refuse it (Recommended)".

So in a repository that holds slot records,
a `git worktree remove` or `git worktree move` behind a global option the wrapper does not know is refused,
and the refusal names that option and says which spellings the wrapper reads
(the command without it, or `git -C <repository> worktree ...`).
In a repository with no slot records the command is forwarded as before.
Either way a check fails once the installed Git is newer than the table
(branch `test/git-policy-cli-wrapper-global-option-table-fails-on-a-newer-git`).
Not chosen: "Keep forwarding".

## A slot whose binding record cannot be read

The finding (the alpha.47 ZFS pass, finding 12,
`~/temp/agent/wrapper-alpha-47-release-review-zfs-route-2026-11-26/keep/review.md`):
with a bound slot's binding record overwritten with garbage,
`git worktree move` refuses with the unreadable record's reason,
while `git worktree remove -f -f` exits 0 and Git deletes the registration and the slot folder.
That was the decided text of `package/git-policy/cli/SPEC.md` (line 5424 on `aba76005c3`:
"a slot nothing proves [is] left to Git"),
and with the clone mounted and the mount table unreadable Git would delete inside the clone.

Answer:
"Refuse the removal too (Recommended)".

So the removal refuses as the move does,
naming the record it cannot read and what to repair,
and `SPEC.md`'s passage changes to say so.
Not chosen: "Keep SPEC as written".
