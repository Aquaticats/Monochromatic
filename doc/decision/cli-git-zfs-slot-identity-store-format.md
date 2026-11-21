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

## The design accepted

Accepted in the question tool on 2026-11-21,
recorded at 2026-11-21T17:23:49-05:00,
in three answers:

- "A:
  accept with its proposals (Recommended)",
  to whether the design is accepted with its first-ranked option for its choices 1,
  3,
  4,
  5,
  6 and 8;
- "A:
  O_EXCL staging,
  then link (Recommended)",
  to how a slot's first record is published;
- "A:
  no listing per sweep (Recommended)",
  to whether each sweep run lists the store folder.

The design is [`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`][design] at commit `3e0fe761a6`,
merged into `main` as `8649808092`.
Its gate review on Sonnet said ready after one fix round,
with eight low findings left for the build
(`~/temp/agent/gate-review-zfs-slot-identity-store-per-slot-files-design-2026-11-21/keep/review.md`,
section "Recheck of 3e0fe761a6").

What the answers settle:

- names:
  a flat root-owned folder held by descriptor,
  one file named by the slot,
  and `.parent` for the parent's record;
- publication:
  the record is created with `O_EXCL` under a hidden staging name,
  linked to its real name in one `linkat` (whose `EEXIST` decides the one winner),
  and the staging name is then removed;
  the unnamed-file route is not taken;
- the parent's record is replaced by one rename,
  and two adoptions at once leave the later one's record,
  as on `main`;
- a second recorder of one slot reads the record that won and carries on with it;
- each record file is bound at 262,538 bytes,
  derived from the kernel's access-list and name limits;
- the folder is root-owned `0755` and each record root-owned `0644`;
- the adoption lists the folder twice and refuses when the set changed;
- a sweep reads only the records of the slots it reaches and does not list the folder;
- the lock-hold document is rewritten and renamed as the publication's write cost,
  as the repository's rules settle.

What the project accepted as lost,
as the question stated it:
an attach no longer refuses on another slot's corrupt or doubled record,
which the adoption's listing and `doctor` catch instead;
and an adoption can miss a slot published after its second listing,
whose record is still correct and only unnamed in the adoption's report.

One commitment made in the question:
a writer killed during publication leaves a hidden staging file,
and the build adds the removal of known staging names to `doctor` or the sweep,
so that the adoption is not their only remover
(the review's low finding R1).

[design]: ../planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md
