# A faster reader of the fence's sizes

**Unapproved and non-final.**
This round designs and measures the readers that could replace the conversion fence's one `git ls-tree`;
it picks nothing,
merges nothing,
and only the owner declares it done.
The numbers are one round's runs on this machine on 2026-11-21,
kept under `~/temp/agent/fence-faster-size-reader-design-2026-11-21/keep/`.

## What the project decided, and what this round answers

`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes,
records the question and the answer of 2026-11-21:
a btrfs snapshot add of this repository took about twice `main`'s time in the guest,
838 ms against 431 ms over five rounds,
with the identical-binary control;
338 ms of it was one `git ls-tree` reading every path's recorded size,
which the snapshot's constant-time copy leaves nothing to overlap with.
Asked whether to merge and then build a faster reader,
to merge and accept the cost,
or to build the reader first,
the project answered "A:
merge,
then faster reader (Recommended)".
Step 3 merged at `5d2a6c58b3` carrying that cost,
and this round is the reader's design and its measurements.

Two findings came before any ranking:

- the 338 ms is not what the same step costs today.
  In this round's own guest run on this repository's tree the `git ls-tree` step is 33 to 36 ms,
  and the snapshot add's medians are 654 to 713 ms;
  the read is about a twentieth of the add and not half of it (`keep/rung1-guest-btrfs.json`).
- the two readers that need no new dependency are **slower** than the `git ls-tree` that ships,
  and only the crate-backed reader is faster (`keep/reader-costs-fresh-process-host-zfs.json`).

## What the fence needs from the read

The fence proves each tracked path twice (`package/git-policy/cli/SPEC.md`,
rung 2;
`doc/planning/cli-git-clone-rung-conversion-fence.md`,
"The fence"):

- the checkout of the path's blob must be the blob,
  which the attributes decide with no blob read at all;
- the source's file must be the blob,
  which the size proof decides.
  A rule can only shrink a file on the way into the index,
  so a recorded size equal to the blob's size proves nothing was shrunk,
  and a path with no rule today can still hold a form an earlier rule wrote.

What that asks of a reader:

- **which paths**:
  every tracked path whose checkout writes the blob unchanged,
  which is every path the fence may carry.
  That is:
  no `filter`,
  no `ident`,
  no `working-tree-encoding`,
  no `text` value Git does not name,
  an output ending that is not the carriage return,
  and not a link.
  In this repository that is all but its 1,578 `filter=lfs` paths and its converting share.
  The fixture below drops the `filter=lfs` lines,
  so every one of its 24,575 paths needs a size,
  which is the fence's worst case and the one the earlier round measured.
- **which fields**:
  the **blob's size**,
  for the blob **the source's index records for that path**.
  The shipped read joins by path from the tree of `source.clean.head_commit`;
  a reader keyed by object name joins by the index's id.
  Both answer the same map,
  because `worktree_clean`'s `index_differs_from_tree` refuses a source whose index is not the tree of `HEAD`.
  The object name is not an answer the fence needs;
  it is the key a reader can join on.
- **the shape of the answer**:
  one map from path to size.
  A path the reader has no size for is absent from it,
  which the fence reads as "the tree lists no blob for it" and hands to Git.

What the add already has in hand,
with no read of its own:

- the source's **index bytes**,
  already parsed in process for the clean check (`stored_stats`,
  `parse_index`):
  every path with its mode,
  its recorded size and its object name;
- the **attributes**,
  from one `git check-attr --all -z --stdin`;
- the up-front skip's `git diff-tree --name-only` between the source's commit and the one the add names,
  which carries paths and no size (`src/native/worktree/worktree_ladder_target_tree_skip.rs`).

So the read is one question per wanted path:
how large is the blob this index entry names.
Nothing else about the object database is wanted,
and no content is needed at all.

## The options

1. **The shipped read**,
   `git ls-tree -r -l -z <commit>`.
   One Git process walks the commit's trees,
   prints every path with its mode,
   kind,
   object name and size,
   and the fence parses the 24,575 records and keeps the ones it needs.
2. **One `git cat-file --batch-check` **,
   fed the distinct object names of the paths that need a size,
   read from the index in process.
   No tree walk and no path output;
   one Git process,
   one line per distinct blob,
   formatted `%(objectname) %(objectsize)`
   .
3. **The object database in process**,
   from the prototype this branch carries,
   `package/git-policy/cli/src/native/worktree/worktree_clone_size_reader.rs`.
   `Pack::size_at` reads a pack entry's own header and,
   for a delta entry,
   the delta's header for its result size,
   without inflating the object.
   `ObjectDatabase::size` reads a loose object's header with `inflate_prefix` and `loose_header`.
   Ids it cannot size,
   and a database it may not read whole,
   go to one `git cat-file` run.
4. **`gix`**,
   the crate-backed in-process reader,
   measured in a scratch crate only (`~/temp/agent/fence-faster-size-reader-design-2026-11-21/scratch/gix-reader/`):
   `gix::open`,
   the index's entries,
   and `objects.header(id).size()`
   for each distinct name.

The prototype of option 3 is committed on this branch,
with `PROTOTYPE FOR MEASUREMENT, NOT FOR MERGING` in its subject,
and it is reachable only from the `measurement-routes` feature,
so no shipped build compiles it.

## Correctness first

Every option has to answer the map the shipped read answers,
and the add has to write the bytes Git's own checkout writes.

- **The map,
  on the shapes where Git answers from somewhere else** (`worktree_clone_size_reader_tests.rs`,
  seven controls,
  all passing):
  loose objects and their packed copies,
  a subdirectory tree,
  a symlink,
  an empty blob,
  a file the index marks executable,
  an **alternate object store** (`objects/info/alternates`),
  a **replaced blob** (`git replace`),
  a **shallow history** (`git clone --depth=1`),
  a **partial clone's promisor** (`git clone --filter=blob:none` with an index and no blobs),
  and a repository whose paths need no size at all,
  which must read nothing.
- **The class harness**:
  twelve attribute and content classes,
  each compared with Git's own checkout by `git checkout-index --prefix`,
  five arms.
  The classes are:
  no rule over a CRLF file;
  `* text=auto eol=lf` over a CRLF file;
  `*.txt text eol=lf` over CRLF;
  `*.txt text eol=crlf` over LF;
  `*.txt -text` over binary;
  `*.txt text=foo`;
  `*.txt ident`;
  `core.autocrlf=true`;
  `core.autocrlf=input`;
  `core.eol=crlf` with `text`;
  and a rule removed by a later commit,
  twice.
  **60 of 60 EQUAL** (`keep/check-classes.json`).
- **The 150-combination sample**,
  seed 11:
  nine attribute rules times six configuration states times five content shapes,
  times the rule-present and the rule-removed state,
  sampled deterministically to 150 cases,
  each added by four arms and compared with Git's own checkout.
  **600 of 600 EQUAL** (`keep/check-sample-11.json`).
- **The crate reader's map**,
  entry for entry:
  `gix`'s 24,575 sizes and `git ls-tree -r -l`'s are the same lines after sorting,
  with `cmp` reporting no difference,
  and the fixture's `ls-tree` sum of 762,846,945 bytes is `gix`'s sum as well.

What each option does in those shapes:

- **`git ls-tree` and `git cat-file` ** are Git.
  A partial clone's missing blob is fetched from the promisor remote by the run itself,
  a replaced blob answers the replacement's size,
  a shallow or grafted history answers from the objects that are present,
  an alternate is followed by Git,
  and a corrupt or missing object fails the run,
  which ends the rung and lets Git's own checkout write the worktree.
- **The in-process reader** answers from the files.
  It follows `info/alternates` to depth 5 and refuses a quoted or escaped alternate path.
  It refuses the whole database when `refs/replace` holds a ref,
  when `shallow` exists,
  or when `info/grafts` exists,
  and one `git cat-file` run then answers every id.
  An id it cannot size goes to that same run for that id alone:
  a promisor's missing blob,
  a loose file over the inflate bound,
  a pack entry it does not understand.
  A partial clone therefore fetches exactly the blobs a size read needs and no more,
  and the controls above hold it to each of those answers.
- **`gix`** was measured against a complete repository only.
  What it does with a promisor,
  a replace ref or a graft is **not measured here**,
  and a reader built on it would have to answer that before the fence may use it.

## The measurements

### The fixture

This repository's tree at `5d2a6c58b3` as a standalone repository,
made by `keep/make-fixture.ts`:
**24,575 tracked paths** and 23,106 distinct blobs,
`.git` of 376 MB in two packs,
a worktree of 1.2 GB,
and the `filter=lfs` lines removed from the copied `.gitattributes` and committed,
so every path takes the size proof instead of Git's smudge.
The wrapper's own policy file in the fixture lists no policy,
so a commit never pays a scanner over the tree.
The fixture's own commits name their paths in chunks of twelve hundred,
because the always-run final-newline check starts `git diff-index` with the whole candidate list
in one argument vector,
and a single commit of this tree fails with "Argument list too long (os error 7)".

### What one read costs, in a process of its own

The fence takes exactly one read per add,
in the add's process,
so a loop inside one process would price a warmed reader and not the one an add pays.
Each reader ran in a fresh process,
seven rounds,
with the order rotated by the round.
The medians on this fixture on this host's ZFS:

- `git ls-tree -r -l -z`:
  **34.5 ms**,
  with 757.4 in the run's first round,
  which was a cold page cache,
  and 30.9 to 35.7 after it;
- one `git cat-file --batch-check` over 23,106 distinct ids:
  **40.9 ms**,
  with 39.0 to 42.4;
- the object database in process:
  **59.8 ms**,
  with 57.6 to 76.5;
- `gix`:
  **10.7 ms**,
  with 10.3 to 12.0.

The source of those numbers is `keep/reader-costs-fresh-process-host-zfs.json`.

The two no-dependency options are slower than the shipped read.
`cat-file` looks each of 23,106 names up in the pack indexes,
and it pays 23,106 round trips through a pipe;
the in-process reader pays two or three `pread` calls per name,
where Git's own lookup reads through an mmap of the index.
`gix` is 3.2 times the shipped read,
because it reads the pack through its own memory map and never asks the kernel per object.

### Rung 1, the btrfs snapshot rung, in a throwaway guest

One `libvirt` guest through `mvm`,
`fence-faster-size-reader-guest`,
Ubuntu 24.04 with kernel 6.8.0-147 on four cores and 7 GiB,
made by this round and destroyed at its end.
The fixture sits on a 6 GiB btrfs image as a subvolume,
so the source is a snapshot rung's source.
Five arms,
five rounds,
the order rotated by the round,
`sync --file-system` before every add.
Each arm's totals in round order,
then its median:

- **`main`** (`5d2a6c58b3`'s binary,
  which already carries the fence and its `git ls-tree`):
  778 776 556 601 654 ms,
  median **653.9**;
- **the branch's `ls-tree` arm**,
  the same read through the prototype binary:
  798 684 571 745 629 ms,
  median **684.4**;
- **the branch's `cat-file` arm**:
  667 717 939 619 701 ms,
  median **700.6**;
- **the branch's object-database arm**:
  713 793 817 645 628 ms,
  median **712.8**;
- **the `cat-file` arm again**,
  the identical-binary control:
  737 812 588 582 661 ms,
  median **660.7**.

Each arm's ratio to `main` in round order,
the control last:

- `ls-tree`:
  1.02 0.88 1.03 1.24 0.96;
- `cat-file`:
  0.86 0.92 1.69 1.03 1.07;
- object database:
  0.92 1.02 1.47 1.07 0.96;
- `cat-file` again:
  0.95 1.05 1.06 0.97 1.01.

The medians of each add's own record,
`main` against the branch's `ls-tree` arm:
`git ls-tree` 36.4 against 32.9 plus 6.0 of parsing;
`git check-attr` 55.8 against 58.3;
the decision 45.2 against 45.9;
`checkSource` 97.6 against 104.6;
`walkSnapshot` 239.3 against 212.9;
`prune` 54.3 against 51.9;
`checkIgnore` 47.9 against 51.4;
`installIndexFile` 25.2 against 21.7.
The `cat-file` arm's read is `git cat-file` 53.0 plus 5.4 of joining,
and the object-database arm's is 48.4 in process.

So on this rung,
today:

- the fence's whole read is 33 to 53 ms of an add of about 650 to 700 ms,
  against the 338 ms the earlier guest run recorded;
- swapping the reader moves the read by tens of milliseconds,
  and every arm's ratio to `main` (0.86 to 1.69 across rounds)
  is no wider than the two `cat-file` arms' own disagreement,
  which is 0.88 to 1.60 round by round;
- no arm is faster than `main` in a way these runs can resolve at this size.

The source of those numbers is `keep/rung1-guest-btrfs.json`.

### Rung 2, the clone rung, on this host's ZFS

The same fixture on this host's ZFS home dataset,
five arms,
five rounds,
and no `sync`,
because the rung copies the files and the read runs beside the copies.
Each arm's totals in round order,
then its median:

- **`main`**:
  3953.0 1258.9 1712.0 1315.7 1217.1 ms,
  median **1315.7**;
- **`ls-tree`**:
  1755.6 1447.6 1687.2 3751.5 1345.2 ms,
  median **1687.2**;
- **`cat-file`**:
  1399.2 1295.5 1393.6 1296.5 1348.9 ms,
  median **1348.9**;
- **object database**:
  1968.9 1634.5 1394.6 1881.0 1264.8 ms,
  median **1634.5**;
- **`cat-file` again**:
  1553.6 1374.7 1655.7 1557.7 1262.3 ms,
  median **1553.6**.

The step medians of the same records:

- `ls-tree`:
  26.2 in `main` and 23.7 in the branch,
  plus 5.8 of parsing;
- `cat-file`:
  37.0 plus 5.6 of joining;
- object database:
  80.4 in process;
- `cloneEntries`:
  828.6 to 1000.4 ms beside them,
  which is what the add's wall time is made of.

The read is not this rung's cost.
It runs on the adding thread while eight clone workers copy the tracked files,
4.1 s of copy work over eight slices,
and the arms' totals are inside the identical pair's disagreement,
the pair ranging 0.83 to 1.07 of each other round by round.
This is the rung's design already (`doc/planning/cli-git-clone-rung-conversion-fence.md`,
"Limitations").

The source of those numbers is `keep/measure-rung2-host-zfs.json`.

## The ranking

Measured cost of the read,
fastest first,
with what taking it costs:

1. **`gix`**,
   a new dependency:
   10.7 ms.
   Pros:
   3.2 times the shipped read,
   no process,
   no pipe,
   no output to parse,
   and it answers by object name so the tree is never walked.
   Cons:
   a third-party dependency with a large tree,
   an `unsafe` surface to audit,
   and its promisor,
   replace-ref and graft answers are unmeasured here.
2. **`git ls-tree`**,
   shipped and needing no work:
   34.5 ms.
   Pros:
   no new code,
   no dependency,
   and every control already runs against it;
   a reader that Git changes is Git's change.
   Cons:
   one process per read,
   and 24,575 path records printed and parsed for 23,106 answers.
3. **One `git cat-file --batch-check` **,
   no dependency but new code:
   40.9 ms.
   Pros:
   no tree walk and no path output,
   and Git's own object lookup decides the shapes,
   a promisor fetch,
   a replace ref and an alternate included,
   exactly as `ls-tree` does.
   Cons:
   6.4 ms slower than the read it would replace,
   one process,
   and 23,106 round trips through a pipe;
   more code for a loss.
4. **The object database in process**,
   no dependency and a prototype already built:
   59.8 ms.
   Pros:
   no Git process at all in the common case,
   and a fallback that keeps the answered map identical for the shapes it refuses.
   Cons:
   25 ms slower than the shipped read,
   the most code of the four,
   with a size-only path through the pack and loose readers,
   a fallback union,
   and controls of its own for object databases Git answers differently.

Why each adjacent pair ranks as it does:

- `gix` above `git ls-tree`:
  3.2 times faster on the same 24,575 paths,
  at the price of a new dependency whose shapes are not yet controlled.
- `git ls-tree` above `cat-file`:
  `cat-file` is 6.4 ms slower here,
  and it would replace working code with new code,
  so nothing is bought.
- `cat-file` above the in-process reader:
  the prototype reader is 19 ms slower again,
  and it costs the most code,
  so the simpler of the two no-dependency options wins on both counts.

What follows for the project,
unapproved and non-final,
for the owner to decide:

- **Keep the shipped `git ls-tree`,
  or take `gix` after vetting.**
  The read today is about a twentieth of a rung-1 add,
  and it runs beside the copies on rung 2,
  so keeping the shipped read costs little,
  and `gix` would buy about 23 ms of a 654 ms add.
- `cat-file` and the in-process reader are measured and not recommended,
  because both are slower than the shipped read at this size.

## What a `gix` dependency would need vetted

`gix` 0.77.0,
the facade the scratch reader used.

- **Licence**:
  `MIT OR Apache-2.0`,
  from the crate's own `Cargo.toml` in the local registry copy,
  which is compatible with this crate's `LGPL-3.0-or-later`.
- **Maintenance**:
  actively released.
  Its plumbing crates are already in this crate's lock through the forbidden-strings scanner:
  `gix-actor`,
  `gix-bitmap`,
  `gix-chunk`,
  `gix-commitgraph`,
  `gix-date`,
  `gix-error`,
  `gix-fs`,
  `gix-hash`,
  `gix-hashtable`,
  `gix-index`,
  `gix-lock`,
  `gix-object`,
  `gix-parallel`,
  `gix-path`,
  `gix-revwalk`,
  `gix-tempfile`,
  `gix-trace`,
  `gix-traverse`,
  `gix-utils` and `gix-validate`,
  twenty of them in `package/git-policy/cli/Cargo.lock`.
- **Size**:
  the facade's normal dependency tree in the scratch crate is 104 crates.
  The lock already holds 20 of them,
  and what a size read adds is the object database and its pack machinery:
  `gix-odb`,
  `gix-pack`,
  `gix-features`,
  `gix-discover`,
  `gix-config`,
  `gix-ref`,
  `gix-attributes` and their own support.
- **`unsafe`**:
  small and all of it in the file and memory-mapped I/O layer,
  with 8 mentions in `gix`'s `src`,
  1 in `gix-odb`'s,
  and none in `gix-pack`'s,
  `gix-features`' or `gix-index`'s.
  Vetting would read those rather than count them,
  and the mmap and file I/O they serve is the surface this crate already uses.
- **Build time**:
  not measured.
  Every attempt at a clean build of the 104-crate tree reused the machine's shared cargo build directory,
  which `~/.cargo/config.toml` sets through `build.build-dir`,
  so the runs reported "Fresh" and took under a second.
  The scratch reader's own incremental rebuild is 0.2 to 0.7 s,
  and the 20 crates the lock already holds are compiled for the scanner already,
  so the increment inside this crate would be the missing crates and not the whole tree.
  A person buying this dependency should time a build with `build.build-dir` empty.

## What this round does not do

- It does not add a dependency,
  flip the reader,
  or change shipped behaviour.
  The one reader the fence ships with is untouched,
  and the prototype is behind `measurement-routes`.
- It does not reproduce the earlier run's 338 ms for `git ls-tree`.
  This round's guest run puts the same step at 33 to 36 ms and the same add at 556 to 939 ms.
  It does not know which of the two runs' cache states is the one a person meets,
  and a cold guest image,
  whose pack does not fit its page cache,
  is the state worth measuring before the 338 ms is treated as the cost.
- It does not measure `gix` inside the fence.
  The crate has no such dependency this round,
  so `gix`'s number is the scratch crate's,
  on the same fixture and the same paths.
- It does not measure `gix` against a promisor remote,
  a replace ref,
  a graft or an alternate.
- It does not measure a repository with no conversion rules at all,
  nor a partial clone's fetch cost per option.
- It does not run rung 1 on this host,
  which has no btrfs,
  and it does not move rung 2's reads onto the waiting path,
  which that rung's design puts beside the copies.
- It does not time a cold guest image.
  The guest's fixture was made once and the adds followed,
  so its packs were in the guest's page cache by the first counted round.

## Where the numbers are

- `keep/reader-costs-fresh-process-host-zfs.json`:
  the four readers over seven fresh-process rounds.
- `keep/rung1-guest-btrfs.json`:
  the guest's five arms over five rounds and their step splits.
- `keep/measure-rung2-host-zfs.json`:
  the host ZFS clone rung's five arms.
- `keep/check-classes.json`:
  the twelve classes,
  five arms,
  60 verdicts.
- `keep/check-sample-11.json`:
  the 150-combination sample,
  four arms,
  600 verdicts.
- `keep/make-fixture.ts`,
  `keep/measure-add.ts`,
  `keep/guest-adds.ts`,
  `keep/check-readers.ts` and `keep/reader-costs.ts`:
  the harnesses that made the fixture and the runs.
