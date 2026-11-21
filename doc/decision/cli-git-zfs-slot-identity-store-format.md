# cli-git ZFS slot identity store format decision

## Status

Accepted 2026-11-21T16:24:40-05:00 in the question tool,
under the answer "B:
folder,
one file per slot".
The question had been open in `doc/handover/questions-for-the-owner.md` since 2026-11-20
("approve ZFS step 4's slot identity store format?"),
asked by step 4's second integration pass,
because step 8 builds on the format and no record approved it.
This record was written by an agent.
It records the choice and what the question said;
the design that carries it out is still to be written and accepted.

## What was decided

The slot identity store,
which holds for each worktree slot the identities the attach checks compare against
(the folders' device,
inode,
owner,
permissions and access lists),
becomes a folder with one file per slot.
Each slot's file is made with `O_EXCL`,
and the store takes no lock.

## What the question put to the project

The question described the store as built:
one root-owned file,
`git-worktree-clone-slot-identities`,
one line per slot,
appended under a lock file only root can open,
given up after two seconds,
as `package/git-policy/cli/SPEC.md` describes under "The recorded identities store".

It offered two options and ranked the built one first:

- A,
  one file as built:
  one read sees the parent and every slot in one state,
  which step 8's checks were designed around;
  the cost is one lock and a cap of one read's size on the whole store.
- B,
  a folder with one file per slot,
  each made with `O_EXCL`,
  no lock:
  lock-free,
  which the repository's concurrency rule prefers,
  and with no whole-store size cap;
  the cost is that no single read sees every slot at once,
  and a rewrite of the format,
  every refusal that names a line,
  and about a thousand lines of controls before step 8.

It also named a rejected third shape:
rebuilding the whole file and renaming it into place without a lock,
which lets two recording runs have the later rename record identities taken from a folder that changed in between.

## Consequences the project accepted

- The single consistent read of every slot is given up.
  Step 8's checks must be designed to need only what a per-slot layout can give.
- The store's format,
  its refusals,
  its controls,
  the adoption's replacement of the parent record,
  and the lock with its measured wait (`STORE_LOCK_WAIT`,
  `package/git-policy/cli/doc/slot-identity-store-lock-hold.md`)
  are rewritten or removed before step 8 builds on the store.
- Nothing of the ZFS route is released,
  so the change is a design change with no migration of written stores.

## Left to the design

The answer settles the shape and leaves these to a design proposal under `doc/planning/`,
which goes to its own review before it is built:

- the file names inside the folder and their grammar,
  and the folder's and files' owner and modes;
- how the parent's record is replaced when step 5's adoption changes it,
  since an `O_EXCL` file cannot be overwritten in place;
- what a reader does with a slot file that is partly written or appears while it reads;
- the size bound for one file,
  and what replaces the whole-store bound;
- what step 8's sweep reads,
  and in which order,
  to decide without one read of every slot;
- what becomes of the size check merged for issue 6327 and of the lock-hold document.
