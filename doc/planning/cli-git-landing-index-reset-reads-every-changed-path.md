# A landing's index reset reads every changed path: options for the owner

## Status

A proposal for the owner to decide from,
unapproved and non-final,
written by the Pi run for issue 6566
on 2026-11-27T06:14:06-05:00,
in the worktree of branch
`docs/planning-issue-6566-proposal-for-a-landing-that-reads-every-changed-path`.
No code,
control or `SPEC.md` was changed by the round that wrote it.

Each claim about behaviour below names what it rests on:
a quoted line with its path and line number,
a command run in a throwaway repository of the round's own scratch with its figure,
or the record that issue 6546 left,
where the text says so.
Every Git claim is taken from the source of the release installed here
(`git version 2.56.1`,
fetched on 2026-11-27T06:06:52-05:00),
never from memory.

What is not known,
and what the owner's answer needs:

- The 30 s a landing of 4,097 staged paths cost in the record for issue 6546
  did not reproduce in the two shapes this round could run:
  a staged change list of 4,097 paths read no file at all,
  and cost 3.06 s.
  The record's own instrument named the in-process reset as the 30 s,
  and the readings below name the entry set the reset writes.
  How both hold together is open,
  and it is the first question at the end.
- Which step of the wrapper opened 4,097 files without reading one of them
  is not known:
  the count is measured,
  the owner of those opens is not.
- No merge that lands a vendor drop,
  and no landing of an imported history,
  was run for this round:
  the shapes run are a staged change list and `git commit -a`.
- How one path's cost divides between the open,
  the read and the hash is only bounded:
  this host's spread between two runs of one arm
  (2.5 to 5.5 times)
  is wider than the hash of a few bytes can be.

## What happens today

### A landing rebuilds every entry of its change list from the commit

A commit that reaches the commit service ends with the service moving the branch
and then rewriting the entries of the commit's paths in the worktree's index.
The service never writes the live index in place:
it copies it to a private file beside it,
rewrites the copy,
and renames the copy over the live index while it holds Git's `index.lock`
(`package/git-policy/cli/SPEC.md`,
lines 18920 to 18932):

> The built index is written without a Git process (`index_reset_file.rs`,
> called from `build_index` in `service_land_follow.rs`;

and `package/git-policy/cli/src/native/landing/service_land_follow.rs`,
lines 237 to 243:

```text
    let copy: PrivateCopy = PrivateCopy::make(lock, seq)?;
    ...
        match reset_in_process(git, repository, new, paths, (lock, &copy)) {
            Some(names) => written = names,
            None => {
                reset_once(git, repository, new, paths, Some(built.as_path()))?;
```

So the copy starts as the live index,
and the entries the person staged are in it,
with the stat data the person's own `git add` wrote
(`package/git-policy/cli/SPEC.md`,
line 18977,
"filled from the live index while the lock is held").

The paths the reset names are either the list the command carried
or,
for a commit that real Git built in a scratch repository
(a merge,
an amend,
a signed commit,
a change list past `MAX_CHANGES`),
every path that differs between the old and the new commit
(`src/native/service/service_journal_intent_text_and_parse.rs`,
lines 86 to 92):

> Paths whose index entries the batch updates.
> Whether the index entries to update are the paths that differ between `old` and `new`,

### An entry the reset writes is compared with the copy before it is written

The reset walks the union of the commit's paths and the paths the copy holds,
and it writes an entry only where the copy's entry differs from the commit's tree
in mode or object name.
Where they already agree it leaves the entry,
and its stat data,
exactly as they stand
(`src/native/index/index_reset.rs`,
lines 376 to 383):

```text
                let same: bool = matches!(held.as_slice(), [only] if only.stage() == 0 && only.mode == index_mode(entry.mode) && only.id == entry.id);
                if same {
                    continue;
                }
```

That line is the reason a staged commit lands without reading its files,
and it is measured below.

### A written entry carries the file's stat data only through a proof that reads and hashes it

Every entry the reset does write asks the worktree for the stat data,
and the answer is either the file's own nine fields or all zeros
(`src/native/index/index_reset.rs`,
lines 247 to 250):

```text
            WorktreeStat::Recorded(source) => source
                .recorded(path, wanted.id.as_slice(), mode)
                .unwrap_or_default(),
```

`recorded` is `LandingStat::record` (`src/native/index/index_reset_worktree_stat_landing.rs`,
lines 362 to 401),
and after its stat checks and its attribute check it opens the file,
reads it and hashes it into a blob name,
refusing everything else.
The doc comment of `src/native/index/index_reset_worktree_stat.rs`,
line 22,
states the bar:

> So the stat data here is recorded only when all five hold:

and lines 24 to 47 name them:
a regular file reached below a held root,
raw bytes that hash to the entry's object name,
no attribute conversion and no carriage return,
one unchanged inode that is older than the index file being written,
and a change time that carries nanoseconds.

`package/git-policy/cli/SPEC.md`,
lines 19049 to 19052,
states the same rule for the landing:

> The entries the update makes from the tree record the worktree's stat data where the file provably holds the tree's bytes
> (`index_reset_worktree_stat.rs`),
> and the all-zero value `git reset --no-refresh` writes everywhere else.
> A path's file proves it when all of these hold:

The fifth condition is in the source and in `stamp_proves_one_version`
(`src/native/index/index_reset_worktree_stat_proofs.rs`,
lines 138 to 140),
and the `SPEC.md` list states four of the five;
that gap is worth its own small correction,
and none was made here.

The zero value is not a harmless placeholder:
`package/git-policy/cli/SPEC.md`,
lines 19119 to 19120,
prices it:

> The next commit's staleness question finds the recorded entries clean and starts no Git for them,
> where the zero value cost one Git start and one hash of each file the previous commit landed.

### Where a person meets the read

The read happens for every entry the reset writes,
so the shape of the cost is decided by whether the live index already held the commit's content:

- Everything staged,
  and the commit built from that index:
  the copy holds the commit's blobs,
  the reset writes nothing,
  and no file is read.
  Measured below:
  4,097 opened,
  none read.
- Nothing staged,
  and the commit made with `git commit -a`:
  the copy holds the previous commit's blobs,
  the reset writes every changed path,
  and each of them is read and hashed once.
  The staging that built the commit has already read the same file once
  for its own object name,
  so the bytes are read twice per changed path.
  Measured below:
  4,096 files read twice,
  7.00 s.
- A merge that lands a vendor drop,
  an amend,
  a rebase,
  or any commit real Git built:
  the copy holds whatever the person had staged,
  so the read lands on every path of the commit the person had not staged,
  which is the whole of a vendor drop or an import.

### What native Git does in the same place

#### `git commit` of a staged index

Git refreshes the whole index and writes it only where something changed
(`builtin/commit.c`,
lines 482 to 490,
[`git/git` at `v2.56.1`](https://github.com/git/git/blob/v2.56.1/builtin/commit.c#L482-L490)):

```c
	if (!only && !pathspec.nr) {
		repo_hold_locked_index(the_repository, &index_lock,
				       LOCK_DIE_ON_ERROR);
		refresh_cache_or_die(refresh_flags);
		if (the_repository->index->cache_changed
		    || !cache_tree_fully_valid(the_repository->index))
			cache_tree_update(the_repository->index, WRITE_TREE_SILENT);
		if (write_locked_index(the_repository->index, &index_lock,
				       COMMIT_LOCK | SKIP_IF_UNCHANGED))
```

`refresh_index` `lstat`s each entry and reads content only where the stat data moved
or the entry is racily clean
(`read-cache.c`,
lines 1509 to 1530 and 1417 to 1432,
[`read-cache.c`](https://github.com/git/git/blob/v2.56.1/read-cache.c#L1417-L1432)).
So Git's own commit of a staged change list costs one `lstat` per index entry and no hash of a changed path.
`SKIP_IF_UNCHANGED` means the index file is not even rewritten.

#### `git commit -a`

Git stages into the real index and then refreshes it
(`builtin/commit.c`,
lines 454 to 470,
[`builtin/commit.c`](https://github.com/git/git/blob/v2.56.1/builtin/commit.c#L454-L470)):

```c
	if (all || (also && pathspec.nr)) {
		...
		add_files_to_cache(the_repository, also ? prefix : NULL,
				   &pathspec, ps_matched, 0, 0, 0 );
		...
		refresh_cache_or_die(refresh_flags);
```

The entries that `add_files_to_cache` writes carry the stat data of the same `lstat`
that its own read hashed,
so `-a` reads each changed file once,
and the refresh behind it finds the entries clean.

#### `git reset`

A mixed reset writes the tree's entries with no stat data,
because `make_cache_entry` is called with no refresh option
(`builtin/reset.c`,
line 169,
[`builtin/reset.c`](https://github.com/git/git/blob/v2.56.1/builtin/reset.c#L151-L186)),
and `refresh_cache_entry` returns the entry untouched without `CE_MATCH_REFRESH`
(`read-cache.c`,
lines 867 to 891 and 1376 to 1377,
[`read-cache.c`](https://github.com/git/git/blob/v2.56.1/read-cache.c#L1366-L1377)).
Then,
unless `--no-refresh` was given,
`reset` refreshes the whole index and can advise against its own refresh
(`builtin/reset.c`,
lines 498 to 511,
[`builtin/reset.c`](https://github.com/git/git/blob/v2.56.1/builtin/reset.c#L498-L511)):

```c
			if (!no_refresh && repo_get_work_tree(the_repository)) {
				...
				refresh_index(the_repository->index, flags, NULL, NULL,
					      _("Unstaged changes after reset:"));
				...
					advise(_("It took %.2f seconds to refresh the index after reset.  You can use\n"
						 "'--no-refresh' to avoid this."), t_delta_in_ms / 1000.0);
```

Git therefore has the same choice the wrapper has,
and Git's default is the refresh:
one `lstat` per entry and one hash of every path the reset wrote,
because a written entry has no stat data to compare.
The wrapper took the other branch (`--no-refresh`) and wrote its own proof,
which reads and hashes a path only when the file can prove the bytes.

#### The racy rule, `core.trustctime` and `core.checkStat`

Git's comparison reads the entry's recorded fields and the file's,
and trusts them where they agree
(`read-cache.c`,
lines 318 to 336,
[`read-cache.c`](https://github.com/git/git/blob/v2.56.1/read-cache.c#L318-L336),
and `statinfo.c`,
`match_stat_data`,
[`statinfo.c`](https://github.com/git/git/blob/v2.56.1/statinfo.c#L61-L100)).
Two settings widen or narrow what that comparison reads:

- `core.trustctime` (true by default) makes the change time a field the comparison reads;
  false ignores it
  ([`Documentation/config/core.adoc`](https://github.com/git/git/blob/v2.56.1/Documentation/config/core.adoc#L114-L119)).
- `core.checkStat` (missing or `default`) reads the sub-second parts of both times,
  the owner,
  the group,
  the inode and the device;
  `minimal` reads only whole seconds and the size
  ([`Documentation/config/core.adoc`](https://github.com/git/git/blob/v2.56.1/Documentation/config/core.adoc#L136-L151)).

"Racily clean" is the one hole in that trust,
and Git states it in three places,
the manual
([`Documentation/technical/racy-git.adoc`](https://github.com/git/git/blob/v2.56.1/Documentation/technical/racy-git.adoc#L66-L78)):

```text
The first `update-index` computes the object name of the
contents of file `foo` and updates the index entry for `foo`
along with the `struct stat` information.  If the modification
that follows it happens very fast so that the file's `st_mtime`
timestamp does not change, after this sequence, the cached stat
information the index entry records still exactly match what you
would see in the filesystem, even though the file `foo` is now
different.
```

the guard (`read-cache.c`,
lines 362 to 385,
[`read-cache.c`](https://github.com/git/git/blob/v2.56.1/read-cache.c#L362-L385)):

```c
static int is_racy_stat(const struct index_state *istate,
			const struct stat_data *sd)
{
	return (istate->timestamp.sec &&
#ifdef USE_NSEC
		 /* nanosecond timestamped files can also be racy! */
		(istate->timestamp.sec < sd->sd_mtime.sec ||
		 (istate->timestamp.sec == sd->sd_mtime.sec &&
		  istate->timestamp.nsec <= sd->sd_mtime.nsec))
```

and the smudge that keeps the hole from moving
(`read-cache.c`,
lines 448 to 456,
[`read-cache.c`](https://github.com/git/git/blob/v2.56.1/read-cache.c#L448-L456)),
which is also why an entry written without stat data is read:

```c
	/* Racily smudged entry? */
	if (!ce->ce_stat_data.sd_size) {
		if (!is_empty_blob_oid(&ce->oid, the_repository->hash_algo))
			changed |= DATA_CHANGED;
	}
```

So an entry whose recorded size is zero is always "changed",
and the reader that wants to know reads the file.
That is the path by which the wrapper's zero value costs the next command a hash.

#### The index's own timestamp

The timestamp the racy rule compares with is the index file's own modification time,
set when the index is read and again when it is written
(`read-cache.c`,
lines 2326 to 2327 and 3115 to 3116,
[`read-cache.c`](https://github.com/git/git/blob/v2.56.1/read-cache.c#L2326-L2327)).
An index file touched later than every entry it holds moves the racy window past all of them,
so nothing is hashed on doubt;
an index file written a moment after the entries it describes puts every one of them inside the window,
and every read of them hashes.
That is why the landing puts the live index's times back onto the copy after each write of it
(`package/git-policy/cli/SPEC.md`,
lines 18979 to 18991),
and why a cheap form that writes stat data has to keep that discipline to help rather than to hide.

## What was measured

All runs are on this host on 2026-11-27,
load average between 42.3 and 52.3 (`uptime` before and after each run),
each in a `systemd-run --user --scope` of its own with `MemoryMax`,
`CPUQuota`,
`TasksMax` and `IOWeight=1`.
The fixture is one throwaway repository of the round's scratch with 4,098 paths,
4,097 of them small files under `many/`,
and the counts of opens,
reads and closes are the kernel's own inotify events
for those paths,
read by a probe of the round's own (`count_file_accesses.rs`),
whose positive control read one watched file and reported `opens=1 accesses=1`.

### The record for issue 6546, taken as given

`doc/troubleshooting/git-policy-cli-landing-index-reset-reads-every-changed-path.md`
holds the measurement this issue rests on,
taken 2026-11-27 between 04:00 and 05:00 -05:00 at a load average of 123 to 186,
from the twin harness `service_landing_state::awkward_change_lists_land_as_native_git_commits_them`:

- native Git,
  one byte copy:
  0.595 s and 0.607 s;
- the wrapper,
  through the service,
  in the other copy:
  33.5 s and 34.9 s;
- inside the wrapper:
  `build the index` 30.5 s and 32.1 s,
  of which the landing's in-process reset is 30.3 s and 29.8 s.

The record states that the reset "is `index_reset_file::reset_index_file`",
and its own account of the cost is one read and one hash a path at 7.3 ms a path.

### A staged change list of 4,097 paths

Everything staged with `git add`,
then `git commit --no-enforce-only -m "probe change list"`
(installed release 0.6.1-alpha.44,
load average 42 to 47):

- the whole command:
  3.06 s;
- inotify:
  `opens=4097 accesses=0 closes=4097`,
  so every changed path was opened once and not one was read;
- the installed index's entries kept the stat data `git add` had written:
  `ctime: 1795777301:712774702` and the same value as `mtime` for `many/f0.txt`,
  while the index file's own time stayed at `2026-11-27 06:01:46.947759205 -0500`;
- `git status --short` printed nothing afterwards.

That is `index_reset.rs` line 379 doing its work:
the copy already held the commit's tree,
so the reset wrote no entry and ran no proof.
The 4,097 opens were something else in the command,
and which step made them is not known (see the questions).

### A change list nothing staged

2 files modified and `git commit -a` (load average 42 to 47):

- the whole command:
  57 ms;
- inotify:
  `opens=6 accesses=4 closes=6 distinct_files_read=2`,
  two reads of each file;
- the entries took the files' new stat data afterwards.

So the proof does read the entries the reset writes,
and the probe sees those reads.

4,096 files rewritten and `git commit -a --no-enforce-only --no-enforce-bulk-add`
(load average 50.0 to 52.3):

- the whole command:
  7.00 s;
- inotify:
  `opens=12288 accesses=8192 closes=12288 distinct_files_read=4096`,
  three opens and two reads of each changed file,
  one read for the staging that made the commit and one for the landing's proof;
- the commit printed `4096 files changed, 4096 insertions(+), 4096 deletions(-)`.

### One path's work, three ways

The same 4,097 small files,
one pass of each arm,
the arms one after another in one scope so they share a load reading
(load average 42.3 to 47.3):

- one `lstat` a path:
  119.96 ms in all,
  29.3 µs a path in the first run and 9.3 µs a path in the second;
- one open and one read to the end a path:
  1835 ms and 2491 ms in all,
  448 µs and 608 µs a path;
- open,
  read and SHA-1 a path,
  by `sha1sum`:
  53.8 µs a path warm in the first run,
  297 µs a path warm in the second.

The spread between two runs of one arm,
2.5 to 5.5 times,
is wider than the hash of a few bytes can be,
so what these figures support is the ratio and not a split:
one `lstat` costs 0.009 to 0.029 ms a path,
one open and read costs 0.30 to 0.61 ms a path,
and a read is 10 to 65 times an `lstat` on this host under this load.
The landing's own shape agrees:
3.06 s for 4,097 paths at 0.75 ms a path where it reads nothing,
and 7.00 s for 4,096 paths at 1.7 ms a path where it reads and hashes each one
behind a staging that read the same bytes already.

### What could not be measured

- Where the record's 30 s went.
  The reset's own share of `build the index` was computed by subtraction in the record,
  and its scratch expired,
  so the only readings are the ones above,
  and neither of them reproduces a read of a staged path.
- Which step opened 4,097 files without reading them.
  The probe counts events,
  and inotify carries no process number.
  An attempt to trace the service was not made:
  it runs in a unit the round's client started,
  and the probe answers without attaching to it.
- A merge that lands a vendor drop,
  and a landing after an import.
  Both need a fixture of thousands of paths on two branches,
  and the round ran the two shapes it could afford.
- The hash's own share of one path's cost,
  for the reason the arms' spread gives.
- Whether a landing's read is what a person waits for on a quiet disk:
  this host was at a load average above 42 for every run,
  and no quiet reading was available.

## The options

Six forms a landing could take,
each designed until its problems show,
plus one form that was designed and rejected.

### Option A: keep the proof, read and hash every path the reset writes

#### What it does

Nothing changes.
The reset writes the commit's entries,
and each one carries the file's stat data only after the five-condition proof read and hashed that file.

#### Pros

- Nothing about a person's edit can be hidden:
  the stat data written is the stat data of the inode whose bytes were hashed,
  and an entry that cannot prove that keeps the zero value,
  which every reader must hash.
- The next command finds the entries clean and starts no Git for them
  (`package/git-policy/cli/SPEC.md`,
  line 19119).
- The differential controls compare the installed index against `git reset` byte for byte today,
  and this is the form they compare.

#### Cons

- The landing pays a full open and read a written path inside Git's `index.lock`:
  0.30 to 0.61 ms a path on this host at a load average of 42 to 52,
  and 7.3 ms a path at the record's load average of 123 to 186.
- The bytes are read twice per changed path when the wrapper staged them itself,
  once for the object name and once for the proof.
- The cost grows with the change list,
  so the one shape a person meets it in
  (a vendor drop,
  an import,
  an amend of a large tree) is the shape it hurts most.

#### At the terminal

The person runs the commit,
and the shell returns when the last file of the change list has been read and hashed.

```text
# the person's terminal, with the durations the measurements above give
$ git commit -a -m "import"
 4096 files changed, 4096 insertions(+), 4096 deletions(-)
# 7.00 s here for 4,096 written paths, and 33.5 s and 34.9 s for the record's 4,097 staged paths
```

### Option B: keep the live index's stat data where the blob already matches

This is the form the issue names,
and it is listed here because the owner will ask about it.

#### What it does

The landing would keep an entry's stat data where the copy's object name and mode already equal the commit's,
and refuse to read that file.

#### Pros

- It states the rule native Git applies everywhere,
  so a reader of the code can check it against Git's documented behaviour.
- It costs no new field on the wire and no change to the journal record.

#### Cons

- The code already does exactly this,
  one line before the write (`src/native/index/index_reset.rs`,
  lines 379 to 381):
  where the blob and mode agree the reset continues and never asks for stat data.
- Where the blob differs,
  which is the only case that reads today,
  the live index's stat data describes the *old* bytes,
  so keeping it would leave the entry stale and make the next command hash the file anyway.
  The form therefore changes no landing's cost.
- A control would show no difference at all,
  which is itself a reason to state this in the issue rather than build it.

#### At the terminal

Nothing a person could see:
the same commit,
the same duration,
the same index.

```text
# the person's terminal, with the measurement above
$ git commit -m "vendor drop"
# 3.06 s before, 3.06 s after: the entries it writes are the ones whose blob differs
```

### Option C: carry the stat data of the commit's own staging index

#### What it does

The index the commit was staged through already holds,
for every changed path,
the object name *and* the stat data of the file whose bytes it hashed.
For the wrapper's own route that is the private index it built during the preparation;
for a commit real Git built it is the scratch index that Git staged into.
The landing would take the stat data from that index instead of proving the file again,
and write it only where the file's `lstat` still reports exactly those fields,
keeping the zero value everywhere else.
That is Git's own rule (`match_stat_data`),
applied to a memory the wrapper already has.

The wrapper already does this for a different index:
a rebuild's private index is copied from the real index so that its scratch `git commit` keeps the cached stat data
(`package/git-policy/cli/README.md`,
lines 6193 to 6199):

> A rebuild's private index is made from a copy of the real index,
> so the scratch `git commit` keeps the cached stat data of every entry the new tree shares with it
> and does not hash the work tree again (measured:
> 233,618,538 bytes of LFS content cleaned and 3.6 s with an index built by a plain `read-tree`,
> against 0.1 s with the real index's stat data).

#### Pros

- A written path costs one `lstat` (0.009 to 0.029 ms measured) instead of one open and read
  (0.30 to 0.61 ms measured),
  and the whole landing of a staged-then-amended vendor drop would land in the time the staged shape already takes.
- The entries stay clean for the next command,
  which is the property `package/git-policy/cli/SPEC.md` line 19119 buys with the proof.
- Nothing can be hidden that Git's own rule would not also hide:
  the bytes were hashed once,
  at the `lstat` the stat data comes from,
  and any later write changes the change time the `lstat` check reads.
- It is the same shape of change the package made once already,
  with a measured 36 times gain on the same kind of work.

#### Cons

- The journal record carries no stat data and no name of the staging index
  (`src/native/service/service_journal_intent_text_and_parse.rs`,
  lines 47 to 103),
  so the form needs either a new field on the wire and in the journal,
  or the staging index left where a restart can read it.
  That is a protocol change,
  with the release-to-release story a protocol change always needs.
- Two shapes have no staging index to copy from:
  a commit an older release or a person wrote outside the wrapper,
  and the reconciliation's own index update.
  Both keep today's proof or the zero value,
  so two paths through the landing exist.
- The racy rule has to be applied twice,
  once against the staging index's own time and once against the copy's,
  and the copy's times are already the live index's for that reason.
- The differential controls against `git reset` would have to be stated again:
  the installed index already differs from Git's by carrying stat data,
  and this widens which entries differ.

#### At the terminal

The commit returns once every written path has been `lstat`ed,
and the person's next `git status` has nothing to hash.

```text
# the person's terminal, with the durations the measurements above give
$ git commit --amend -m "vendor drop"
 4096 files changed, 4096 insertions(+), 4096 deletions(-)
# the staged shape's 3.06 s plus one lstat a path, 0.04 to 0.12 s for 4,096 of them,
# against 7.00 s measured for the reading form
```

### Option D: write the zero value and let the next command pay

#### What it does

The landing writes the commit's entries with all-zero stat data,
exactly as `git reset --no-refresh` writes them,
and the proof is dropped.

#### Pros

- The landing holds the lock for the index write alone and reads nothing.
- Nothing can be hidden,
  not even by a proof that is wrong:
  an entry with a zero size is "changed" to every reader
  (`read-cache.c`,
  lines 448 to 452),
  so the next command hashes the file.
- It is one behaviour instead of two,
  and it is what Git's own `--no-refresh` does.

#### Cons

- The wrapper's own next command pays one hash a path,
  and native Git's next command pays a Git start and a hash a path,
  which is the cost `package/git-policy/cli/SPEC.md` line 19120 names.
- A person who lands a vendor drop and then runs `git status` waits twice:
  once for the landing,
  once for the status.
- The two index bytes the differential controls compare would change for every path a landing writes,
  so every control that asserts on stat data has to be restated.

#### At the terminal

The commit returns quickly and the person's next command is the slow one.

```text
# the person's terminal, with the figures above
$ git commit -a -m "import"
# returns after the index write, about the staged shape's 3.06 s at 0.75 ms a path
$ git status
# and this now hashes each imported file: 1.2 to 2.5 s of reading alone for 4,096 of them
```

### Option E: write the zero value, then refresh through Git after the lock is released

#### What it does

The landing writes zeros,
installs the index,
releases the lock,
and then asks `git update-index --refresh` for the whole index in the service,
after the person already has the answer.
Git's own `git reset` does this inside its own command (`builtin/reset.c`,
line 500),
and advises `--no-refresh` when it took long.

#### Pros

- The lock is held for the index write alone,
  so another client's commit does not wait behind the reads,
  which is the part of the record's 30 s that hurts other people.
- The refresh is Git's own code and Git's own rule,
  so nothing about which entry keeps what stat data is decided here.
- A crash after the installation leaves an index every reader can use.

#### Cons

- The work is not saved,
  only moved:
  the refresh hashes every path the reset wrote,
  because a written entry has no stat data to compare,
  so the total cost is at least Option A's.
- A second writer of the live index,
  after the lock is gone,
  races whatever the person stages next,
  and needs the lock again to write,
  which is a new ordering problem in a place the package has spent controls on
  (`doc/troubleshooting/git-policy-cli-landing-controls-wait-on-order-not-time.md` is the file for the last one).
- The service's timing record would carry a phase no client waits for,
  and the next command may still find zeros if it arrives before the refresh.

#### At the terminal

The commit returns before the files are hashed,
and the wait for those files moves to whichever command comes next.

```text
# the person's terminal, with the figures above
$ git commit -a -m "import"
# returns once the index is installed and the lock is released
$ git status
# clean, and fast, if the service has finished its refresh; the same hashes as Option A if it has not
```

### Option F: a flag that picks between the proof and the zero value for one commit

#### What it does

The landing's behaviour becomes a choice:
today's proof by default,
and one option (`--no-landing-refresh`,
or a `core.` setting) that lands with zeros for a commit a person knows is a vendor drop.

#### Pros

- The person who meets the cost can decide when to pay it,
  and the person who does not is unaffected.
- The default keeps every control and every differential comparison as they stand.

#### Cons

- It delivers today's behaviour plus a flag,
  so it removes no read for anyone who does not know the flag exists.
- The wrapper's own behaviour then depends on an option native Git has no equivalent of,
  and the flag needs its own documentation,
  controls and a story for a service of another release.
- A person who sets it and then loses their work to a `git status` of a zeroed index has a defect report
  the flag's own documentation caused.

#### At the terminal

The person who knows the flag saves the landing's read;
everyone else waits as today.

```text
# the person's terminal, with the figures above
$ git commit -a --no-landing-refresh -m "import"
# returns after the index write, about the staged shape's 3.06 s at 0.75 ms a path,
# and the next command hashes the 4,096 paths
```

### A form designed and rejected: record the stat data without proving the bytes

Record the file's `lstat` fields for a written entry without hashing the bytes,
guarded only by the racy rule and by the inode's own change time.
This is the cheapest form of all,
one `lstat` a path with no memory to carry,
and it is rejected:

- for an entry the reset writes,
  the bytes the entry records came from staging that happened earlier,
  and nothing in the landing knows the stat data those bytes were read at,
  so any rewrite between the staging and the landing that kept the size and the modification time
  leaves the entry looking clean,
  which is the one outcome the whole proof exists to prevent;
- the racy rule narrows that window to the index's own second,
  and `core.trustctime=false`
  or a file system with coarse times widens it again
  (`statinfo.c`,
  `match_stat_data`);
- the wrapper's own staleness question trusts the same fields,
  so a wrong record would hide the edit from the next commit as well as from `git status`.

## Ranking

Ranked by what each form delivers for a person whose edit must never be hidden from the next commit,
and for a landing of thousands of paths on a loaded host.
Effort to build is a cost to state,
not a reason to rank a form lower.

Ranking:
C > D > E > A > F > B.

- **C over D**:
  both hold a landing to one `lstat` a path,
  and C is the only one whose entries the *next* command finds clean.
  D is exactly as safe and moves the same read onto the person's next command,
  which on an imported tree of thousands of paths is the command they run to see what they landed.
- **D over E**:
  both keep the reads out of the lock,
  and D ends there,
  where E adds a second writer of the live index after the lock is released,
  racing what the person stages next,
  so E buys the same speed with a new ordering problem and a phase no one waits for.
- **E over A**:
  both hash every written path,
  and E does it outside the index lock,
  so a second client's commit is not held behind another person's vendor drop,
  which is the harm the record's 30 s names beyond the person's own wait.
- **A over F**:
  A is today's behaviour,
  complete and controlled,
  where F delivers A plus a switch.
  A person who knows the switch gains D's wait,
  and nobody else gains anything,
  so A states the same trade with less surface.
- **F over B**:
  F changes a landing's cost for the person who asks,
  where B changes nothing at all for anyone,
  because the entries it would spare are the ones the reset already leaves alone.

## Questions only the owner can answer

1. Which measurement stands as the cost to beat?
   The record's 30 s in `build the index` for a staged change list,
   whose own instrument named the in-process reset,
   or this round's 3.06 s for the same shape with no read of any changed path,
   and 7.00 s for 4,096 paths whose entries the copy did not already hold?
   The answer decides whether the target is one hash a path or none.
2. Which shapes must a cheap form cover?
   The staged change list,
   which already costs nothing,
   or the shapes that do read:
   `git commit -a`,
   an amend,
   a merge that lands a vendor drop,
   a rebase,
   an import,
   and every commit past `MAX_CHANGES` that real Git builds?
   Only the second set is worth a change.
3. May the landing take the stat data from the index the commit was staged through?
   That is Option C,
   and it needs a field on the wire and in the journal,
   or the staging index kept where a restart can read it.
   Without that answer Option C cannot be built,
   and the choice is between A,
   D,
   E and F.
4. Is the zero value acceptable as the landing's answer for a path it cannot prove,
   given that the person's next command then pays a hash a path,
   and native Git's own `git status` pays a Git start and a hash a path?
   `--no-refresh` is the branch Git offers the same person,
   with an advice line
   (`builtin/reset.c`,
   lines 502 to 511).
5. Should any of this be a flag?
   Option F is the only form where a person decides,
   and it is also the only one that adds a behaviour
   native Git has no equivalent of.
6. Are the differential controls still the standard?
   The installed index already differs from `git reset --no-refresh` by the stat data it carries,
   so the controls compare something narrower than the bytes.
   Which entries' stat data a control must pin,
   and against what,
   is a decision the owner's answer sets.
7. Does the fifth condition belong in `SPEC.md`?
   The source states five conditions and the specification lists four,
   the missing one being that the file's change time carries nanoseconds.
   That is a small correction to the specification,
   and it was not made here.
