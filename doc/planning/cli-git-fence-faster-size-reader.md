# A faster reader of the fence's sizes

**Unapproved and non-final.**
This round designs and measures the readers that could replace the conversion fence's one `git ls-tree`;
it picks nothing,
merges nothing,
and only the owner declares it done.
The numbers are this round's runs on this machine on 2026-11-21 and 2026-11-22,
and the records the tables come from are committed beside this proposal (`package/git-policy/cli/perf/size-reader-*`).

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
and this round is the reader's design and its measurements,
revised against a gate review of 2026-11-21.

The 338 ms is not what the same step costs on every source,
and the gate review found the variable this round never varied:
whether the source's objects are loose or packed.

- The earlier round's timing source was all loose.
  Its own script (`~/temp/agent/zfs-step-3-fence-integration-findings-fixed-2026-11-21/scratch/guest-run.ts`,
  its `git init` to `git update-ref` lines) builds it with `git init`,
  `git add --all`,
  `git write-tree`,
  `git commit-tree` and no `git gc`,
  so all 24,971 objects stayed loose.
- On that layout,
  in this round's guest,
  the `git ls-tree` step is **340.9 ms** of an **868.5 ms** snapshot add:
  **39 percent** of the add.
- On the same tree packed in place by one `git gc`,
  the same step is **24.0 ms** of a **517.1 ms** add:
  **4.6 percent**.
  The step moves by a factor of fourteen while every step both layouts pay stays put (`checkSource`,
  `walkSnapshot`,
  `git check-attr`,
  `prune`,
  all within about a third of each other;
  the tables are below).
- The layout a person meets is the packed one.
  On this host's main checkout,
  `HEAD`'s tree has 23,163 distinct blobs,
  of which 446 (1.9 percent) are loose and 22,717 (98.1 percent) are packed,
  counted from `git ls-tree -r -z HEAD` and an `is_file` per id
  (the gate review's probe,
  re-run on this repository's main checkout).
  An all-loose source is a bulk `git add` that no `git gc` has packed yet;
  whether a commit through the wrapper triggers `gc.auto` was not checked here.
- So the earlier round's packed fixture saw 33 to 36 ms,
  because its own `make-fixture.ts` ends with `git gc` ("The real repository's objects are packed,
  so the fixture's are too"),
  and the two rounds differed in exactly the thing that moves the read ten times.

Two things follow before any option is ranked,
and both are measured below,
not inferred:

- on the common,
  packed layout the `git ls-tree` that ships is the fastest of the three readers that need no crate;
- on the rare,
  all-loose layout the in-process reader is the fastest,
  and `gix` 0.90.0 is slower than the shipped read.

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
  so every one of its 24,572 paths needs a size,
  which is the fence's worst case.
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

The four the owner is asked about,
A to D:

- **A**:
  keep the shipped read,
  `git ls-tree -r -l -z <commit>`.
  One Git process walks the commit's trees,
  prints every path with its mode,
  kind,
  object name and size,
  and the fence parses the 24,572 records and keeps the ones it needs.
- **B**:
  `gix`,
  at the release that would actually be added:
  `gix` 0.90.0,
  whose twenty plumbing crates are in this crate's lock at identical versions.
  Measured in scratch crates only,
  never in this package,
  as the facade (`gix` 0.90.0) and,
  separately,
  as **`gix-odb` 0.87.0 alone**,
  the narrower dependency that would need the fence's own ref reading for replacements.
- **C**:
  the object database in process,
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
- **D**:
  one `git cat-file --batch-check`,
  fed the distinct object names of the paths that need a size,
  read from the index in process.
  No tree walk and no path output;
  one Git process,
  one line per distinct blob,
  formatted `%(objectname) %(objectsize)`.

The prototype of option C is committed on this branch,
with `PROTOTYPE FOR MEASUREMENT, NOT FOR MERGING` in its subject,
and it is reachable only from the `measurement-routes` feature,
so no shipped build compiles it.

## Correctness first

Every option has to answer the map the shipped read answers,
and the add has to write the bytes Git's own checkout writes.

- **The crate reader's map**:
  on the shapes where Git answers from somewhere else (`worktree_clone_size_reader_tests.rs`,
  eight controls,
  all passing in this round):
  loose objects and their packed copies,
  a subdirectory tree,
  a symlink,
  an empty blob,
  a file the index marks executable,
  an **alternate object store**,
  a **replaced blob**,
  a **shallow history**,
  a **partial clone's promisor**,
  a **reftable repository's replace ref**,
  and a repository whose paths need no size at all,
  which must read nothing.
- **The class harness**:
  twelve attribute and content classes,
  each compared with Git's own checkout by `git checkout-index --prefix`,
  four arms (shipped,
  `ls-tree`,
  `cat-file`,
  object database).
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
  **48 of 48 EQUAL**.
- **The 150-combination sample**,
  seed 11:
  nine attribute rules times six configuration states times five content shapes,
  times the rule-present and the rule-removed state,
  sampled deterministically to 150 cases,
  each added by the same four arms and compared with Git's own checkout.
  **600 of 600 EQUAL**.

What each option does in those shapes:

- **`git ls-tree` and `git cat-file`** are Git.
  A partial clone's missing blob is fetched from the promisor remote by the run itself,
  a replaced blob answers the replacement's size,
  a shallow or grafted history answers from the objects that are present,
  and an alternate is followed by Git.
  A **corrupt or missing object does not fail the run**,
  which this round corrected:
  with a loose blob deleted from a fixture,
  `git ls-tree -r -l -z` exits 0 and prints the size field as `BAD`,
  and `git cat-file --batch-check` exits 0 and prints `<id> missing`.
  The shipped parser and the prototype's skip a field that is not a number,
  so the path is absent from the map,
  which the decision turns into `WriteThroughGit("the tree lists no blob for it")`.
  The outcome is safe,
  because Git writes that path and fails itself if it must,
  but it is not the failed run an earlier draft of this proposal claimed.
- **The in-process reader** answers from the files.
  It follows `info/alternates` to depth 5 and refuses a quoted or escaped alternate path.
  It refuses the whole database when a repository keeps its refs where the reader's scans never look:
  `refs/replace` holds a ref,
  `shallow` exists,
  `info/grafts` exists,
  or `reftable/` exists,
  because the reftable backend keeps a replace ref there and not in `refs/replace` or `packed-refs`.
  One `git cat-file` run then answers every id.
  An id it cannot size goes to that same run for that id alone:
  a promisor's missing blob,
  a loose file over the inflate bound,
  a pack entry it does not understand.
  The fence's layout read already refuses a ref store other than `files` (`repo_read_layout.rs`,
  the `extensions.refstorage` arm),
  so no fence read reaches a reftable source today;
  the reader refuses one anyway,
  and the shipped clean check that opens the same database now refuses it too.
- **The partial-clone control now fails when a promisor fetch is not handled**,
  which the gate review showed the old one could never do.
  The crate's origin names no `uploadpack.allowFilter`,
  so `git clone --filter=blob:none` printed "filtering not recognized by server,
  ignoring",
  fetched every object,
  and left the clone holding every blob while still carrying the `.promisor` file the control asserted.
  The control now turns the filter on at the origin,
  gives each reader a clone of its own,
  and asserts with `GIT_NO_LAZY_FETCH` that every id the index names is absent before that reader runs.
  A reader that answers only what it finds in the files leaves those paths out of the map,
  and the control fails;
  it passes because the in-process reader asks Git for the ids it cannot size.
- **`gix`** answers the shipped read's map on a complete repository:
  every one of `gix` 0.90.0's 24,572 sizes equals `git ls-tree -r -l`'s after sorting,
  with no differing entry and the same byte sum,
  518,458,993 (`size-reader-gix-map-2026-11-21.json`).
  Where Git answers from somewhere else,
  measured with Git as the reference (`size-reader-gix-shapes-2026-11-21.txt`):
  a **replaced blob** (9 bytes replaced by 300,000):
  Git 600,000,
  `gix` 0.90.0 600,000,
  `gix` 0.77.0 **300,009**,
  `gix-odb` 0.87.0 alone **300,009**;
  a **reftable repository's replace ref**:
  Git 600,000,
  both `gix` releases **300,009**,
  `gix-odb` alone **300,009**;
  an **alternate object store**:
  all three agree with Git;
  a **partial clone whose blobs are truly absent** (origin with `uploadpack.allowFilter`,
  two objects in the clone):
  all three fail on the first absent id with "could not be found" and none fetches.
  A reader built on `gix` therefore needs the same fallback to one Git run as the in-process reader,
  for absent blobs and for reftable replace refs,
  and `gix-odb` alone needs the caller to pass replacements and to read them from the ref store itself.

## The measurements

### The fixture

This repository's tree at `5d2a6c58b3` as a standalone repository,
made by `git archive` of that commit:
**24,572 tracked paths** and 23,103 distinct blobs,
all 24,971 objects loose (350 MB in loose files) or one pack after `git gc` (190 MB),
a worktree of 1.2 GB,
and the `filter=lfs` lines removed from the copied `.gitattributes` and committed,
so every path takes the size proof instead of Git's smudge.
The fixture's own policy file lists no policy,
so a commit never pays a scanner over the tree.

### The orders

Every arm is one wrapper binary plus the environment naming its reader;
the shipped read is `main`'s binary at `02fd8540f9`,
the branch's arms are one measurement build,
and the identical-binary control is the branch's `ls-tree` arm run twice,
`branch-ls-tree` against `branch-ls-tree-again`,
so the control shares the binary,
the feature set and the environment.

- **Rung 1**:
  one `libvirt` guest of this round,
  `fence-faster-size-reader-guest`,
  Ubuntu 24.04,
  kernel 6.8.0-147,
  four cores and 7.9 GiB,
  the fixture on a 6 GiB btrfs image as a subvolume,
  so every add takes the snapshot rung.
  Five arms,
  ten rounds,
  one uncounted warm-up add per arm,
  `sync --file-system` before every add,
  and one Python process inside the guest drives every add,
  so no host round trip sits between arms.
  The order rotates one place per round and every pass over the arms runs the reversed list,
  so each arm takes each of the five slots twice and follows each other arm once.
- **Rung 2**:
  the same five arms and the same order on this host's ZFS home dataset,
  ten rounds,
  one uncounted warm-up per arm,
  the clone rung,
  so the read runs beside eight copying workers and no `sync` is taken.
- **The readers in a process of their own**:
  eight position-balanced rounds per reader and layout,
  the three no-crate readers through the fence's own measurement route in a fresh process,
  and the two `gix` candidates as their own binaries.
  On the host the `gix` binaries are the same ones the guest ran.
- **Medians** are the upper middle element for an even count.
  A timing is only read as a ratio to a control arm in the same rounds;
  the totals are read through the identical pair's own range.

### Rung 1, the btrfs snapshot rung, loose and then packed

Loose,
all 24,971 objects loose,
add totals in round order and the read step's median (the read is the `git ls-tree` process,
plus the parsing,
for the first two arms;
the `git cat-file` process plus the join for `cat-file`;
the in-process open and read for the object database):

- **`main`**:
  868.5 ms median,
  read **340.9 ms** (327.3 to 400.3),
  **39.3 percent** of the add.
- **`branch-ls-tree`**:
  882.1,
  read **348.1** (334.9 to 371.6),
  39.5 percent.
- **`branch-ls-tree-again`** (the control):
  882.5,
  read **349.7** (329.9 to 373.8),
  39.6 percent.
- **`branch-cat-file`**:
  896.5,
  read **358.4** (334.1 to 380.3),
  40.0 percent.
- **`branch-object-database`**:
  **748.9**,
  read **211.0** (199.7 to 219.9),
  28.2 percent.

The identical pair's per-round ratio is **0.94 to 1.04**,
so this harness resolves an add difference of about 5 percent.
The object-database arm is 0.77 to 0.89 of `main` in each of the ten rounds,
below the pair's whole range in every round:
the in-process read really does make the loose add about 14 percent faster.
`cat-file` at 1.00 to 1.06 of `main` buys nothing.

The same tree,
packed in place by one `git gc` (one pack,
24,971 objects):

- **`main`**:
  517.1 ms median,
  read **24.0 ms** (22.7 to 25.7),
  **4.6 percent** of the add.
- **`branch-ls-tree`**:
  527.7,
  read **28.2** (27.5 to 32.5),
  5.4 percent.
- **`branch-ls-tree-again`** (the control):
  529.7,
  read **27.9** (27.3 to 29.5),
  5.3 percent.
- **`branch-cat-file`**:
  541.1,
  read **41.7** (38.3 to 53.7),
  7.7 percent.
- **`branch-object-database`**:
  540.4,
  read **41.6** (40.2 to 43.1),
  7.7 percent.

The identical pair's per-round ratio here is **0.98 to 1.02**,
and both `cat-file` and the object database sit at 1.03 to 1.14 of `main` round by round,
above it in most rounds:
on the common layout both are about 4.5 percent slower adds than the shipped read,
and the branch's own `ls-tree` arm is indistinguishable from `main`.

### Rung 1, the readers in a process of their own

Whole-process wall time per read,
eight balanced rounds,
then the split the `gix` binaries print (open,
index,
header loop):

- **Loose**:
  `git ls-tree -r -l -z` **351.6 ms** (one 583.0 first round,
  338.3 to 352.5 after it);
  `git cat-file --batch-check` **347.6** (340.7 to 395.8);
  `gix` 0.77.0 **221.9** (loop **206.9**);
  `gix` 0.90.0 **397.5** (loop **385.4**);
  `gix-odb` 0.87.0 alone **398.4** (loop **394.2**).
- **Packed**:
  `git ls-tree` **23.0** (22.4 to 27.6);
  `git cat-file` **32.7** (31.9 to 36.5);
  `gix` 0.77.0 **23.0** (loop **9.9**);
  `gix` 0.90.0 **19.1** (loop **9.8**);
  `gix-odb` alone **13.5** (loop **9.3**).

So on loose objects `gix` 0.90.0 is slower than the spawned `git ls-tree`,
and its 0.77.0 predecessor is 1.9 times faster there;
on packed objects the `gix` header loop,
about 9.5 ms,
is what a facade would add to an add,
against 24 to 28 ms for the spawned read.

### Rung 1, cold, guest-cold only

Two repetitions per reader and layout,
`drop_caches` before each,
a fixed order inside a repetition:
indicative,
not resolved.

- **Packed readers**:
  `git ls-tree` 491.9 then 120.9 ms;
  `git cat-file` 288.0 then 140.7;
  `gix` 0.90.0 167.6 then 119.5;
  `gix-odb` alone 157.6 then 113.0.
  **Packed add read step**:
  `git ls-tree` 168.5 then 141.1;
  `git cat-file` 163.7 then 122.9;
  in-process 107.8 then 93.5.
- **Loose readers**:
  `git ls-tree` 970.9 then 935.3;
  `git cat-file` 1,155.7 then 1,574.4;
  `gix` 0.90.0 905.6 then 978.4;
  `gix-odb` alone 840.2 then 899.1.
  **Loose add read step**:
  `git ls-tree` 964.2 then 847.2;
  `git cat-file` 1,084.4 then 1,014.6;
  in-process 664.1 then 611.1.

A cold disk costs every reader hundreds of milliseconds and the first read after a drop far more;
the guest's image stays cached on the host,
so a cold disk would be slower than this.

### Rung 2, the clone rung, on this host's ZFS

Ten rounds,
warm-up per arm,
walls in the first line and the read step's medians below them.
The totals are noise at this size,
with the host loaded by other sessions and several rounds above 2 s;
the read steps are what can be read,
and they run beside the copies.

- **Loose**:
  walls `main` 947,
  `ls-tree` 826,
  `ls-tree-again` 910,
  `cat-file` 948,
  object database 938 ms;
  the identical pair ranges **0.90 to 1.24**.
  Read steps:
  `main` **270.2**;
  `ls-tree` **228.3**;
  `ls-tree-again` **239.0**;
  `cat-file` **243.2**;
  object database **209.6**.
- **Packed**:
  walls `main` 795,
  `ls-tree` 773,
  `ls-tree-again` 912,
  `cat-file` 797,
  object database 818 ms;
  the identical pair ranges **0.87 to 3.06**,
  because two rounds stalled.
  Read steps:
  `main` **24.8**;
  `ls-tree` **31.6**;
  `ls-tree-again` **27.9**;
  `cat-file` **39.0**;
  object database **65.0**.

On this host too the in-process read is the fastest on loose,
and the slowest of the three no-crate readers on packed.
The fresh-process readers on ZFS,
eight balanced rounds:

- **Loose**:
  `git ls-tree` **292.5** (206.5 to 10,368.9);
  `git cat-file` **208.4** (194.5 to 4,145.9);
  in-process **175.4** (158.3 to 11,575.8);
  `gix` 0.90.0's loop **227.3** (218.8 to 269.8);
  `gix-odb` alone's loop **234.5** (216.5 to 265.5).
- **Packed**:
  `git ls-tree` **36.7** (32.6 to 63.4);
  `git cat-file` **42.3** (37.8 to 49.4);
  in-process **61.2** (56.7 to 110.1);
  `gix` 0.90.0's loop **9.7** (8.9 to 12.5);
  `gix-odb` alone's loop **9.4** (8.8 to 10.8).

### What the numbers do and do not resolve

- On the guest,
  the identical pair's range is 0.94 to 1.04 loose and 0.98 to 1.02 packed,
  so the 14 percent loose difference and the 4.5 percent packed differences are above it in every round.
- On this host's ZFS,
  the walls are not resolvable at all;
  only the read steps are quoted.
- The cold phase has n of 2 and a fixed order;
  it indicates magnitude,
  nothing finer.
- No pre-fence `main` arm was re-run this round,
  so the "about twice `main`'s time" of the earlier btrfs finding is carried by that round's step records,
  not by a measurement here.
- A source's tree does not decide the read's cost;
  its object layout,
  its page cache and the disk do.
  Every table above states which layout it measured.

## The ranking, per layout

Measured cost of the fence's read,
by layout,
fastest first,
with the whole option each number belongs to.

**Loose objects** (the rare layout,
a bulk add never packed):

1. **C**,
   in-process:
   211.0 ms in the guest add and 175.4 ms on this host,
   0.60 to 0.62 of the shipped read,
   and fastest on both.
2. **A**,
   `git ls-tree`:
   340.9 ms in the guest,
   292.5 ms on this host.
3. **D**,
   `git cat-file`:
   358.4 ms in the guest,
   208.4 ms on this host.
4. **B**,
   `gix`:
   0.90.0's loop is **385.4** ms in the guest,
   slower than the shipped read,
   and 227.3 ms on this host,
   between C and A;
   0.77.0's is 206.9 ms in the guest,
   but that release shares none of the lock's twenty plumbing crates.

**Packed objects** (the common layout,
98.1 percent of this repository's `HEAD` blobs):

1. **B**,
   `gix`:
   the header loop is 9.8 ms in the guest and 9.7 ms on this host;
   `gix-odb` alone 9.3 and 9.4.
2. **A**,
   `git ls-tree`:
   24.0 to 28.2 ms in the guest add,
   24.8 to 31.6 ms on this host.
3. **D**,
   `git cat-file`:
   41.7 ms in the guest,
   39.0 to 42.3 ms on this host.
4. **C**,
   in-process:
   41.6 ms in the guest,
   61.2 to 65.0 ms on this host.

So no option wins both layouts,
and the two layouts differ by more than any option does:
the shipped read costs 24 ms on the layout people meet and 341 ms on the one step 10 measured.

## What a `gix` dependency would need vetted

The gate review found this section's facts wrong or partial for the release that was measured;
these are read from the lock files,
and the registry sources of the versions that would be added (`size-reader-gix-vetting-2026-11-21.json`).

- **Which release**:
  `gix` 0.90.0 is the release that unifies with this crate's lock:
  all twenty of its `gix-*` plumbing crates are in `package/git-policy/cli/Cargo.lock` at the same versions.
  `gix` 0.77.0,
  the release an earlier draft measured,
  shares **none** of them by version (18 by name,
  all at other versions),
  so adding it would add a second copy of each.
- **Feature names**:
  `max-performance-safe` does not exist in 0.90.0;
  the equivalent is `max-performance` plus an explicit `sha1`.
  Between 0.77.0 and 0.90.0 the facade also changed its replaced-blob answer and its loose-object cost,
  the pre-1.0 churn the lock would have to follow.
- **Size**:
  `gix` 0.90.0 adds **47** name and version pairs to this crate's lock (81 stay shared);
  `gix-odb` 0.87.0 alone adds **15** (64 shared),
  of which 13 of the 18 `gix-*` crates are shared;
  `gix` 0.77.0 would add **68**.
- **Licence**:
  the tree is not uniform.
  Of the 47 added by 0.90.0:
  32 are `MIT OR Apache-2.0`,
  3 `Unlicense OR MIT`,
  2 `MIT`,
  2 `MIT/Apache-2.0`,
  1 each of `(Apache-2.0 OR MIT) AND BSD-3-Clause` (`encoding_rs`),
  `Apache-2.0 OR MIT` (`multiversion_no_op`),
  `MPL-2.0` (`uluru`),
  `Apache-2.0` (`unicode-bom`),
  `(MIT OR Apache-2.0) AND Unicode-3.0` (`unicode-ident`),
  `Unlicense/MIT` (`walkdir`),
  `CC0-1.0 OR MIT-0 OR Apache-2.0` (`dunce`),
  and `Zlib` (`zlib-rs`).
  MPL-2.0 is file-level copyleft;
  the project analysed it for another product (`doc/decision/music-player-third-party-licenses.md`),
  so it can be decided,
  but nothing here decides it.
  `gix-odb` alone adds no MPL-2.0 and no BSD-3-Clause term:
  its 15 are 9 `MIT OR Apache-2.0`,
  3 `Unlicense OR MIT`,
  1 `Zlib`,
  1 `Unlicense/MIT`,
  1 `(MIT OR Apache-2.0) AND Unicode-3.0`.
- **`unsafe`**:
  the earlier draft's counts were wrong.
  Counted over the registry sources of the 0.90.0 set,
  real `unsafe` items and blocks (`unsafe fn`,
  `unsafe impl`,
  `unsafe trait`,
  `unsafe {`) are:
  `gix` 4,
  `gix-odb` 1,
  **`gix-pack` 11**,
  `gix-index` 1,
  and beside them `gix-path` 9,
  `gix-sec` 5,
  `gix-hash` 3,
  `gix-parallel` 3,
  `gix-ref` 3,
  `gix-tempfile` 2,
  `gix-config-value` 2,
  `gix-packetline` 2,
  `gix-trace` 2,
  `gix-actor` 1,
  `gix-commitgraph` 1,
  `gix-filter` 1,
  `gix-revision` 1.
  `gix-pack`'s include the pack mapping (`gix-pack/src/lib.rs`,
  the mmap whose comment says "we have to take the risk of somebody changing the file underneath"),
  most of the rest is parallel delta traversal and `Send`/`Sync` impls.
  A header read does not call them,
  but the build compiles them.
  `gix-odb` alone's set is 30 such items across 7 crates.
- **Build time**:
  an empty build directory,
  the release profile,
  a four-core quota:
  `gix` 0.90.0's tree is **100 compiled units in 18.45 s**;
  `gix-odb` 0.87.0 alone is **56 in 9.71 s**
  (`size-reader-gix-build-2026-11-21.txt`).
  (An earlier draft said unmeasured because the machine's shared `build.build-dir` reported "Fresh";
  the numbers here set `CARGO_BUILD_BUILD_DIR` to an empty folder.)
- **Process**:
  the `choosing-technology` skill requires a vet report (`doc/audit/<subject>-vet-<date>.md`),
  a candidate ledger with the repository's own incumbent and alternatives,
  equal-depth validation and a scored ranking before a recommendation.
  This proposal recommends nothing,
  so none is required now;
  if the owner picks B,
  that work is the next step,
  and it should be done for the release that unifies with the lock.

## What this round does not do

- It does not add a dependency,
  flip the reader,
  or change shipped behaviour beyond one refusal:
  the in-process object reader now refuses a repository whose refs are kept in `reftable/`,
  where its scans never look.
  The fence's shipped read is untouched,
  and the prototype is behind `measurement-routes`.
- It does not measure `gix` inside the fence.
  The crate has no such dependency this round,
  so `gix`'s numbers are scratch crates' on the same fixture and the same paths.
- It does not measure a promisor's fetch cost per option,
  nor what `git ls-tree -l` does with many absent blobs.
- It does not measure a cold disk beyond the guest's own cold state (n of 2,
  fixed order),
  and it does not measure rung 1 on ZFS (this host has no btrfs) or rung 2 in a guest.
- It does not overlap the read with `checkSource` (about 60 to 130 ms),
  or the attribute read (about 35 to 60 ms) on rung 1.
  A source-only read has no need for the snapshot,
  so the read might run beside them;
  nobody has measured it,
  and it is not a reader change at all.
- It does not move rung 2's read onto the waiting path,
  which that rung's design puts beside the copies.

## Where the numbers are

Committed beside the package that ships the read,
under `package/git-policy/cli/perf/`:

- `size-reader-guest-loose-adds-2026-11-21.jsonl` and `size-reader-guest-packed-adds-2026-11-21.jsonl`:
  every arm's every add,
  the step record it appended,
  and the slot it took.
- `size-reader-guest-loose-readers-2026-11-21.jsonl` and `size-reader-guest-packed-readers-2026-11-21.jsonl`:
  the fresh-process readers in the guest,
  with the `gix` splits.
- `size-reader-guest-loose-cold-2026-11-21.jsonl` and `size-reader-guest-packed-cold-2026-11-21.jsonl`:
  the cold phase.
- `size-reader-rung2-loose-2026-11-21.json` and `size-reader-rung2-packed-2026-11-21.json`:
  the host clone rung's five arms over ten rounds.
- `size-reader-host-readers-loose-2026-11-21.json` and `size-reader-host-readers-packed-2026-11-21.json`:
  the fresh-process readers on this host's ZFS.
- `size-reader-numbers-2026-11-21.json`:
  every table above,
  computed from those files.
- `size-reader-gix-vetting-2026-11-21.json`,
  `size-reader-gix-shapes-2026-11-21.txt`,
  `size-reader-gix-promisor-2026-11-21.txt`,
  `size-reader-gix-map-2026-11-21.json`:
  the vetting facts and the shape probes.
- `size-reader-classes-2026-11-21.json` and `size-reader-sample-11-2026-11-21.json`:
  the class harness and the 150-combination sample.

## The gate review's findings, each answered

- **High 1**:
  the headline misattributes the missing 338 ms:
  fixed above.
  The all-loose fixture is named with the earlier round's own script and lines,
  both layouts are measured on one tree,
  and the cold-image guess is gone;
  the cold state is measured,
  not guessed.
- **High 2**:
  the ranking holds only for a packed,
  warm repository:
  fixed.
  The ranking is stated per layout and per cache state,
  and the layout people meet is measured on this repository's own `HEAD`.
- **High 3**:
  the `gix` that was measured is not the one that could be added:
  fixed.
  `gix` 0.90.0 is measured for cost,
  split,
  replaced blobs,
  reftable replace refs and promisor absence,
  its feature names are corrected,
  its replaced-blob answer is recorded (600,000,
  where 0.77.0 answers 300,009),
  its reftable answer is recorded (300,009,
  wrong as 0.77.0 is),
  and `gix-odb` 0.87.0 alone is weighed separately.
- **Medium 1**:
  the partial-clone control is vacuous:
  fixed,
  and described in "Correctness first".
  The control now fails when a reader does not ask Git for an absent blob.
- **Medium 2**:
  the vetting facts are wrong or partial:
  fixed in "What a `gix` dependency would need vetted":
  the `unsafe` counts,
  every licence in the added tree,
  the added package count,
  the build time,
  and `gix-odb` alone as a candidate.
- **Medium 3**:
  "a corrupt or missing object fails the run" is wrong:
  fixed.
  `ls-tree` prints `BAD` and `cat-file` prints `missing`,
  both exit 0,
  and the path is handed to Git.
- **Medium 4**:
  the in-process reader's refusal does not cover reftable replace refs:
  fixed.
  `objects_are_replaced` refuses a repository with `reftable/`,
  a control builds one with a replace ref and holds every reader to the shipped map,
  and the fence's layout read that already refuses a non-`files` ref store is named.
- **Medium 5**:
  no owner question and a ranking that contradicts the text:
  fixed.
  The question and the four options are the last section,
  and the ranking now follows the measured costs per layout.
- **Medium 6**:
  the evidence is outside the repository:
  fixed.
  Every record and the computed tables are committed under `package/git-policy/cli/perf/`.
- **Low 1**:
  the `cat-file` slowness explanation is not what the code does:
  fixed.
  The ids reach `git cat-file` through one private input file,
  not a pipe (`transaction_git.rs`,
  `input_file`),
  so "23,106 round trips through a pipe" is gone,
  and no mechanism is asserted for `cat-file`'s cost in either direction.
  The `gix` "never asks the kernel per object" sentence is gone with it.
- **Low 2**:
  the rung-1 identical pair was `cat-file` twice while `main` and `ls-tree` were different binaries:
  fixed.
  The control is now the same binary with the same environment run twice,
  and the tables say the claim rests on the read-step records,
  with the pair's range stated.
- **Low 3**:
  no pre-fence arm:
  stated in "What the numbers do and do not resolve":
  the "about twice `main`'s time" is the earlier round's step record,
  not a measurement here.
- **Low 4**:
  the branch tip:
  dismissed as a fact about the brief and not the proposal.
  This document names no branch tip;
  the review's `456902fb69` and this branch's merged tip are both later than the `3dbc2819f3` the brief named,
  and neither changes a number above.

## The owner question

Put through the question tool,
with the context restated:

"Step 10's follow-up asked for a faster reader of the fence's blob sizes:
one `git ls-tree` cost 338 ms of an 838 ms btrfs snapshot add.
That number came from a test source whose objects were all loose.
On a packed source the same read costs 24 ms of a 517 ms add (4.6 percent),
and 98.1 percent of this repository's own `HEAD` blobs are packed.
On the all-loose layout it costs 341 ms of an 868 ms add (39 percent).
What should the project do about the read?"

- **A**,
  keep `git ls-tree`,
  close step 10's follow-up and record the layout dependence (Recommended).
  Pros:
  no new code or dependency;
  every control already runs against it;
  it is the fastest of the options that need no crate on the common packed layout (24 to 28 ms in the guest add,
  24.8 to 31.6 ms on the host,
  against 41.7 and 39 to 42.3 for `cat-file`);
  its map is Git's own answer for every shape measured.
  Cons:
  an all-loose source still pays 341 ms of an 868 ms rung-1 add (39 percent),
  and a cold read costs 0.6 to 1.6 s whatever the reader.
- **B**,
  take `gix` after a vet,
  at `gix` 0.90.0 or as `gix-odb` 0.87.0 alone.
  Pros:
  on the packed layout the header loop is 9.3 to 9.8 ms in the guest and 9.4 to 9.7 ms on the host,
  so about 15 ms of a 517 ms rung-1 add (3 percent) and about 15 ms of a 795 ms rung-2 add,
  where the read runs beside the copies;
  no process,
  no output to parse,
  and it answers by object name.
  Cons:
  47 new packages (15 for `gix-odb` alone) including MPL-2.0,
  Zlib and BSD-3-Clause terms;
  `unsafe` in the pack mapping;
  a pre-1.0 facade whose feature names and replaced-blob answer changed between 0.77 and 0.90;
  it ties or loses on loose objects (0.90.0's loop is 385 ms against the shipped 341 ms in the guest);
  it needs a fallback to Git for promisor blobs and for reftable replace refs,
  and `gix-odb` alone also needs the caller's own replacements;
  and the vet report the `choosing-technology` skill requires.
- **C**,
  build the in-process object-database reader properly (the prototype).
  Pros:
  no dependency;
  it is the fastest read on the all-loose layout (211 ms in the guest,
  175 ms on the host,
  0.60 to 0.62 of the shipped read) and it makes that add 14 percent faster;
  it reuses the repository's own pack and loose readers and refuses what it cannot answer.
  Cons:
  slower than the shipped read on the common packed layout (41.6 ms against 24.0 to 28.2 in the guest,
  61.2 to 65.0 against 24.8 to 31.6 on the host),
  so the packed add is about 4.5 percent slower;
  the most new code;
  and a reftable replace ref is refused rather than answered.
- **D**,
  one `git cat-file --batch-check`.
  Pros:
  no dependency,
  and Git decides every shape (promisor,
  replace,
  alternates).
  Cons:
  slower than the shipped read in every cell measured (41.7 against 24 to 28 ms packed in the guest,
  39.0 against 24.8 on the host,
  358 against 341 ms loose in the guest);
  new code for a loss.

**Ranking**:
A > B > C > D,
because:

- A ranks over B:
  the common layout gains about 15 ms of a 517 ms add from `gix`,
  nothing on rung 2 beyond that,
  and nothing at all on loose objects at 0.90.0,
  at the price of 47 packages,
  three licence families this project has not decided here,
  and a pre-1.0 facade whose feature set and replace-ref answer changed,
  across the 13 minor versions since the release an earlier draft measured.
- B ranks over C:
  on the layout people have,
  `gix`'s header loop is 9.5 ms against the in-process read's 42 to 65 ms,
  and the in-process reader makes the common add slower;
  C wins the loose layout by about 60 to 140 ms, depending on the rung,
  which is the rarer layout,
  and it wins it with more code than any other option.
- C ranks over D:
  C is the fastest read on the loose layout and ties `cat-file` on packed,
  while `cat-file` is slower than the shipped read in every cell and buys nothing.

Two things the rankings do not decide,
and the owner should say:

- whether the project's value is "faster in the common case" (A or B) or "no slow case" (C,
  or the unmeasured overlap);
- if B,
  whether the facade or `gix-odb` alone is worth its vetting,
  since the narrower candidate adds 15 packages instead of 47 and gives up the crate's own ref reading.
  If the owner wants the all-loose and cold cases fixed rather than recorded,
  the option to measure next is overlapping the read with `checkSource` or the attribute read on rung 1,
  before any reader is swapped.

**Unapproved and non-final.**
