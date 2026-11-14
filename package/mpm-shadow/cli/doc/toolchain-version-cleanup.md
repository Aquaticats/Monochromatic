# The cleanup of old toolchain versions

mpm-shadow installs one directory per tool version
(`<store>/<tool>/<version>`,
`<store>/cargo/<crate>/<version>`)
and keeps the archives it downloaded for each version
(`<store>/downloads/<tool>/<version>`).

`install` and `update` remove the versions nobody needs,
by themselves,
with no command to remember:
at the end of every sync that installed a version,
and at the start of any later sync while a removal an earlier run started is unfinished.

## The measurement it was sized from

Read-only,
  on this machine's stores,
2026-11-13.
The run kept it as `store-inventory.json`,
`store-inventory.md`,
`proc-scan.txt`,
`deletion-cost-home-volume.txt`
and `cost-of-a-pass-on-the-real-store.txt`.

- The two stores are `<data home>/monochromatic-dev/toolchain`,
  the repository scope,
  and `<data home>/monochromatic-dev/toolchain-global`,
  the machine scope.
- Together they held **55.4 GiB**,
  and **11.3 GiB** of that was versions some visible lock names:
  160 worktree locks,
  the main checkout's lock,
  and the machine scope's lock.
- **42 GiB** was versions no lock names,
  or downloads an earlier layout left.
  The largest parts are nine unnamed builds of `github-lemonade-sdk-llamacpp-rocm` at 3.1 GiB each,
  six unnamed `aws` versions at 250 MiB each,
  and 4.5 GiB of flat download files named `<tool>-<version>-<file>` under `downloads/`,
  which nothing reads.
- The process table held seven store paths,
  all inside versions the locks also named.
  Reading the `exe`,
  `cwd`,
  `maps`
  and open descriptors of 1102 processes cost about 0.18 s there.
- Deleting costs about 30 microseconds per file on this store volume:
  4743 files took 145 ms,
  8820 took 260 ms,
  and 1180 files totalling 3.2 GiB took 62 ms.

`findmnt` reports `noatime` for this machine's home volume,
so a version's access time is not maintained and was **not** used.
The age of a version is the write time of its own directory,
which is when the version arrived:
`install` unpacks into a hidden staging directory and renames it into place,
so the directory's write time is the install,
and it changes again when an entry is added to or removed from that directory.

## What a pass costs an install

Measured on this machine,
in the files named above.

- An install that installed nothing and has no removal to finish:
  two `stat` calls,
  one per store root for the unfinished-removal marker,
  and the pass is not run at all.
- A pass that removes,
  measured on the replay of this machine's listing
  (136 version directories and leftover downloads,
   59 of them removed,
  33 851 MiB):
  158 to 173 ms before the second look was added to the removals,
  and 300 to 349 ms with it,
  over runs that alternated the two on one machine
  (its process count moved between 1082 and 1103).
  A real store's deletion costs more than the replay's sparse files do,
  so the share the second look adds there is
  smaller than it looks here.
- A pass on a store it has never measured:
  9.3 s,
  for one `stat` and a size walk of every version directory.
- A pass afterwards:
  1 ms for the versions,
  because `.mpm-shadow-version-sizes.json` records each version's size beside the write time it had,
  and one `stat` per version says whether the record still holds.
  The rest of a pass is 30 ms for `git worktree list`,
  0.27 to 0.7 s to read and parse 164 lock files,
  and 0.15 s for the process table.
- The process table,
  once for the plan and once more after the versions of a pass are moved out of their names:
  145 to 192 ms per look,
  measured on the replay of the real listing below.
  The second look is per pass,
  not per version:
  the whole batch is moved aside first,
  so fifty-nine removals pay it once.
- The removals:
  at most two seconds,
  at the measured 30 microseconds per file.

## What the bound counts

The bound is compared with what the pass can judge:
the version directories,
the downloads kept for each of them,
and the download files an earlier layout left.
The files beside those are not versions,
are not removed,
and are not counted:
the programs the `bin` downloader writes,
the exposure and launcher link farms,
and `bin.config.json`.
Counting what it cannot remove would make the pass empty the store and stay over the bound.

## The rule

While the stores keep more than the size bound,
the versions written longest ago go first,
until they are inside it.
Every version that stays says why it stays.

A version is kept when any of these holds,
the most specific first:

- **a lock names it**:
  this scope's lock,
  the lock of every worktree `git worktree list` reports,
  and the machine scope's lock.
  A lock that cannot be read keeps every version,
  with the reason in the line.
- **this run installed it**.
- **the tool's `current` link names it**.
- **a running process of this user uses it**:
  its program (`/proc/<pid>/exe`),
  its working directory (`/proc/<pid>/cwd`),
  a file it has mapped (`/proc/<pid>/maps`),
  or a file it has open (the `/proc/<pid>/fd` links).
  A process the kernel refuses to describe is matched by the name it calls itself,
  and its versions are kept with that name in the line.
  The process table is asked twice for the versions this pass removes:
  once while the plan is made,
  and once more after every version of the batch has been moved out of its name,
  which is the moment the removals become final.
  A version something holds then is renamed back and kept,
  with the hold in its line.
- **it was written inside the age floor**.
- **another run wrote it after the scan**:
  the directory write time is read again before the rename,
  and a version that changed is kept with that reason.
- **the process table could not be read**:
  every version is kept,
  with the reason in the line.

## A removal cut short

A removal is two steps:

1. the version's downloads directory and then the version directory are renamed to
   `.mpm-shadow-removing.<name>` beside them,
   one atomic rename each;
2. those two are deleted.

A run killed between the two steps leaves a hidden `.mpm-shadow-removing.*` directory,
which no scan lists as a version,
and a `.mpm-shadow-removal-unfinished` file in the store root.
The next install runs the pass because of that file,
finds every `.mpm-shadow-removing.*` directory by walking the store's own directories,
and finishes the deletion,
printing one line saying it did.
That step reads no lock:
the version is condemned already,
so a lock that cannot be read must not hide it.
A `.mpm-shadow-removing.*` directory something holds is renamed back instead,
and a later pass removes it once its holder is gone.
The marker in the store root goes only once the store holds no `.mpm-shadow-removing.*` at all.

## The window that is left

Nothing can start using a version by its own name once it is renamed,
so the second look at the process table sees every process that holds one:
a program,
a working directory,
an open file,
or a mapping that entered the version before its rename is still there to be seen,
and one that tries afterwards meets a name that no longer exists.
No lock,
link,
launcher or exposure path names a removal marker,
so the marker's own name is the only way into a version after its rename.
Two cases remain,
and neither can be closed from this side:

- a process the kernel refuses to describe (`dumpable = 0`),
  which hides its program and its mappings;
- a process of another user,
  which this user cannot read at all.

Both are handled the same way as before:
the name such a process calls itself is compared with the tool's directory name,
and a version it may be running is kept only when that name matches.
A version such a process holds without such a match can still be removed while it runs.

One more case is left in principle:
a process handed a removal marker's own name by something outside this program could enter the marker between the
second look and the delete.
Nothing mpm-shadow writes names a marker,
and a marker stands for at most one pass,
so this needs a program that reads the store for markers and starts work inside one.

## What it says

One line per version removed and per version kept,
on standard error.
A pass that judges nothing prints nothing,
which covers the stores being inside the bound,
and a pass that installed nothing and has no removal to finish,
which is not run at all.

## The bounds

Each is set in the environment and has the shipped default when unset.

- `MPM_SHADOW_STORE_SIZE_BOUND`:
  the space both stores may keep,
  24 GiB.
  Sized from the 11.3 GiB of pinned versions the measurement found.
  `24GiB`,
  `25769803776`
  and `512MiB` all read.
- `MPM_SHADOW_STORE_AGE_FLOOR`:
  how recently a version may have been written and still be removed,
  one hour.
  **A proposal**,
  because no measurement settles it:
  an install writes its lock row before the version it installs,
  so this floor only covers a copy a person or another tool made by hand.
  `2h`,
  `90m`
  and `3600` all read as seconds.
- `MPM_SHADOW_STORE_WORK_LIMIT`:
  how long one pass may spend removing,
  two seconds.
  Sized from the deletion measurement:
  at 30 microseconds per file it removes about 66 000 files,
  more than any single version the stores hold,
  so an install never waits on a long deletion and later installs finish the rest.
  `5s` and `500ms` both read.

## What it does not touch

- Anything outside the stores,
  and anything under a build folder.
- `cargo`'s registry cache and this repository's own crates there,
  which the shadow's post-run cleanup still owns when it acts as `mpm`.
- The older copies of this repository's own tools in the other scope's root,
  which the install of an old copy already removes.
- A tool directory no visible lock names:
  the scan is driven by the locks' own tool ids,
  so a tool that left every plan is never judged.
- The versions that `pnpm`,
  `uv`,
  `rustup`
  and the `bin` downloader keep in their own homes,
  which mpm-shadow never wrote.
