# Object-store forms the in-process reader reads

Note 2026-11-16T12:00:00-05:00:
every measurement and every command in this record ran with Git 2.56.0 on this host.

## What and why

The wrapper's cleanliness proof reads the commit `HEAD` names and its whole tree without starting Git
(`src/native/repo_read/repo_read_odb.rs`,
`repo_read_pack.rs`,
`repo_read_tree.rs`).
A form of store it declines is safe,
because the caller then asks Git or refuses,
but it costs a removal:
a worktree that holds a volume this program did not make refuses where it is
meant to be removed around that volume,
and every fallback pays the file-by-file path.

This record is the answer to which stores it reads,
measured by making each one,
and which it declines on purpose,
with the words a person meets.

## The forms Git writes, and the reader reads each one

- Loose objects,
  the plain store.
- A pack `git gc --quiet --prune=now` makes,
  with a reverse index and a commit graph beside it.
- A pack a fetch makes:
  `git clone --no-local file://<source>` from a repository whose objects are packed.
- A tree stored as a delta,
  which `git repack -a -d -f --window=250 --depth=50` makes.
- A delta chain deeper than a hundred.
  `git gc --aggressive` runs
  `git repack -f --depth=250 --window=250`,
  and a tree below `HEAD` then sits at the end of
  more than a hundred deltas.
  Before 2026-11-16 the reader refused such a tree as unreadable while
  Git read it.
- A pack whose index is version 1:
  `git -c pack.indexVersion=1 repack -a -d -f` writes it and Git
  reads it.
  Before 2026-11-16 the reader refused the whole object database over it.
- A multi-pack index beside the packs (`git multi-pack-index write`),
  and reverse indexes
  (`.rev`,
  written by `git repack` by default).
  The reader ignores both and reads the plain indexes.
- A repository that borrows every object through `objects/info/alternates`
  (`git clone --shared`),
  and an alternate that borrows further.
- A commit graph (`git commit-graph write --reachable`,
  and the one `git gc` writes),
  which the
  reader never opens;
  it reads the commit object instead.
- A bitmap (`git repack --write-bitmap-index`),
  which the reader never opens.
- A SHA-256 repository:
  the pack index,
  the names and the object framing all carry 32 bytes.
- The empty tree,
  whose pack entry states a size of zero.

The controls are `src/native/repo_read/repo_read_tree_store_forms_tests.rs` with its fixtures file,
and each row compares the reader's flat list with `git ls-tree -r --full-tree HEAD` byte for byte.

## Forms Git does not write, though a pack can hold them

- A delta whose base is outside its own pack.
  Git appends such a base into the pack when it finishes a
  thin one (`fix_unresolved_deltas` calls `append_obj_to_pack` in `builtin/index-pack.c`),
  so a finished
  pack that Git wrote never holds this form.
  The reader still resolves a base by name through every
  pack and the loose files,
  and
  `repo_read_written_packs_tests.rs` proves both places with packs it writes itself.
- A pack with a multi-pack index and no plain index.
  Git refuses the pack
  (`packfile <path>.pack index unavailable`) even with a multi-pack index present,
  so there is nothing
  for the reader to add;
  it declines the same objects,
  and the removal falls back to Git.

## Forms the reader declines on purpose, and the words a person meets

Each of these makes Git answer something the files alone do not spell,
so the reader gives up and the
caller asks Git.
`ObjectDatabase::open_or_reason` answers the reason:

- A replace ref:
  names the ref under `refs/replace` and asks for `git replace --delete <name>`.
- A shallow repository:
  names `.git/shallow` and asks for `git fetch --unshallow`.
- Grafts:
  names `.git/info/grafts` and says `git filter-repo` replaces a graft.
- An alternates list written in a quoting or escaping the reader does not follow:
  names the file and
  says to write the path plainly.
- A pack index that is neither version 1 nor version 2:
  names the index and says
  `git index-pack <the pack>` writes the index again.
- An object whose bytes do not hash to the name asked for,
  in a pack or loose:
  answered as unreadable,
  never as the object.
- A delta that comes back to an entry it already walked,
  a walk longer than the number of entries the
  pack states it holds,
  an offset outside the pack,
  and a stated size the bytes do not fill:
  each ends
  the read,
  and every read stays inside the pack file's own size.

## What the change cost

Three rounds of the unforced removal of a clean worktree,
alternating the build before and after the
change,
in a repository whose objects are all in one pack (10,173 objects,
910,996 bytes under
`objects/pack`,
10,001 tracked files;
`git count-objects -v` reports `count: 0`,
`in-pack: 10173`,
`packs: 1`,
`size-pack: 849`):

- before:
  458.8,
  476.8,
  532.6 ms
- after:
  459.0,
  419.0,
  466.2 ms
- real Git as the control arm:
  360.1,
  365.8,
  381.3 ms

The mean ratio of after to before is 0.916,
and the run-to-run spread of one arm is wider than the
difference,
so hashing each object against its name costs nothing measurable at this size.

## What reads this store

`repo_read_query_quiet.rs` (the answer to `diff-index --cached --quiet`),
`repo_read_query.rs` (tree pairs,
`show`),
`worktree/worktree_clean.rs` (the cleanliness proof and the snapshot source),
`worktree/worktree_ladder_target.rs`,
`index/index_reset_tree.rs`,
`lfs/lfs_object_plan.rs`,
`landing/service_pruned.rs` and `landing/service_built_checks.rs`,
and `repo_read/object_abbreviation.rs` for the alternates list.
