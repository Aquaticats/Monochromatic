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
  Those paths carry every symbolic link resolved,
  so a store root spelled through `$HOME` is compared by both spellings as well;
  before this a root under a linked home (`/home` to `/var/home` on this machine) matched no holder at all.
  A process the kernel refuses to describe is matched by the name it calls itself,
  and its versions are kept with that name in the line.
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
The next install runs the pass because of that file and finishes the deletion,
printing one line saying it did.

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

## The other removals an install makes

Three removals are older than this pass and run beside it.
Each asks the same question this pass asks about a running process,
so a version in use is not taken away by one rule and kept by another.

- `install`,
  after it installs a version of a tool this repository publishes,
  deletes every older finished version:
  a release archive (`remove_older_archives` in `src/toolchain/published_archive.rs`)
  or a crate (`remove_older_than_newest` in `src/toolchain/cargo_installs.rs`).
  It keeps what this pass keeps,
  apart from the age floor and the size bound:
  a version any visible lock names,
  a version a running process of this user uses,
  and the version it just installed,
  so a concurrent newer install cannot undo this run's own work.
  Both lists come from `src/toolchain/store_keeps.rs`,
  and a lock or a process table that cannot be read keeps every version.
- `install`,
  at its end,
  deletes the older managed copies of this repository's tools
  in the root that does not hold the newest one (`src/toolchain/managed_losers.rs`).
  It keeps a version a running process of this user uses.
  It does not keep a version a lock names:
  the owner's rule is that these tools are one set across both roots,
  so a tree whose lock pins an older version runs the newer one and is warned
  (`doc/decision/file-enforcer-rust-rewrite.md`).
  A root that keeps a version because a process runs it ends with its `current` link naming that version,
  never one this run removed,
  and only a root that keeps no finished version forwards its link to the other root.
- `prune --toolchain` deletes the versions no known lock names (see the package README).
  It keeps a version a running process of this user uses,
  and a process table it cannot read refuses the command instead of deleting anything.

The name a process the kernel hides calls itself is not used by these three,
though this pass uses it:
this pass judges only while the stores are over their bound,
while they run whenever a newer version arrives,
and a hidden name that happened to match a tool would keep every version of that tool for good.

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
