# The cleanup of old toolchain versions

mpm-shadow installs one directory per tool version
(`<store>/<tool>/<version>`,
`<store>/cargo/<crate>/<version>`)
and keeps the archives it downloaded for each version
(`<store>/downloads/<tool>/<version>`).

`install` and `update` now remove the versions nobody needs,
by themselves,
with no command to remember:
at the end of every sync that installed a version,
and at the start of any later sync while a removal an earlier run started is unfinished.

## The measurement it was sized from

Read-only, on this machine's stores, 2026-11-13
(the run's `keep/store-inventory.json`,
`keep/store-inventory.md`,
`keep/proc-scan.txt`,
`keep/deletion-cost-home-volume.txt`):

- The two stores are `<data home>/monochromatic-dev/toolchain` (the repository scope)
  and `<data home>/monochromatic-dev/toolchain-global` (the machine scope).
- Together they held **55.4 GiB**, and **11.3 GiB** of that was versions some visible lock names
  (160 worktree locks, the main checkout's lock and the global lock).
- **42 GiB** was versions no lock names, or downloads an earlier layout left:
  - `github-lemonade-sdk-llamacpp-rocm`: ten versions at 3.1 GiB each, nine of them unnamed (28.4 GiB);
  - `aws`: seven versions at 272 MiB, six unnamed (1.6 GiB);
  - `cargo-monochromatic-*` and `node`, `java`, `cmake`, `go`, `zig`, `uv`: unnamed older versions (a further few GiB);
  - flat download files named `<tool>-<version>-<file>` under `downloads/`, which nothing reads (4.5 GiB).
- The process table held 7 store paths, all inside versions the locks also named;
  the read of all 1102 processes' `exe`, `cwd`, `maps` and open descriptors costs about 0.18 s there.
- Deleting costs about 30 microseconds per file on this store volume
  (4743 files in 145 ms, 8820 in 260 ms, 1180 files of 3.2 GiB in 62 ms),
  so the 2 s work limit removes about 66 000 files.

`findmnt` on this machine's home volume reports `noatime`,
so a version's access time is not maintained and was **not** used:
the age of a version is the write time of its own directory,
which is when the version was installed.

## What a pass costs an install

Measured on this machine, 2026-11-13
(`keep/cost-of-a-pass-on-the-real-store.txt`, `keep/deletion-cost-home-volume.txt`):

- an install that installed nothing and has no removal to finish:
  two `stat` calls (the unfinished-removal markers of the two roots), and the pass is not run at all;
- a pass on a store it has never measured: 9.3 s (one `stat` and a size walk per version directory);
- a pass afterwards: 1 ms for the versions (`.mpm-shadow-version-sizes.json` records each version's size beside
  the write time it had, and one `stat` per version says whether the record still holds),
  plus 30 ms for `git worktree list`, 0.27 to 0.7 s to read and parse 164 lock files,
  and 0.15 s for the process table;
- the removals: at most 2 s, at the measured 30 microseconds per file.

## The rule

While the stores keep more than the size bound,
the versions written longest ago go first,
until they are inside it.
Every version that stays says why it stays.

A version is kept when any of these holds, cheapest check first:

- **a lock names it**:
  this scope's lock,
  the lock of every worktree `git worktree list` reports,
  and the machine scope's lock.
  A lock that cannot be read keeps every version, with the reason in the line.
- **this run installed it**.
- **the tool's `current` link names it**.
- **a running process of this user uses it**:
  its program (`/proc/<pid>/exe`),
  its working directory (`/proc/<pid>/cwd`),
  a file it has mapped (`/proc/<pid>/maps`),
  or a file it has open (`/proc/<pid>/fd`).
  A process the kernel refuses to describe (`dumpable = 0`, which wrapper programs that hold tokens set) is
  matched by the name it calls itself,
  and its versions are kept with that name in the line.
- **it was written inside the age floor**.

A version directory written by another run after the scan is kept as well:
its own write time is read again before the removal starts.

## What the bound counts

The bound is compared with what the pass can judge:
the version directories,
the downloads kept for each of them,
and the download files an earlier layout left.
The files beside those (the `bin` downloader's binaries,
the exposure and launcher link farms,
`bin.config.json`)
are not versions,
are not removed,
and are not counted:
counting what it cannot remove would make the pass empty the store and stay over the bound.

## The bounds

Each is set in the environment and has the shipped default when unset.

- `MPM_SHADOW_STORE_SIZE_BOUND`: the space both stores may keep,
  24 GiB (`24GiB`, `25769803776` and `512MiB` all read).
  Sized from the 11.3 GiB of pinned versions the measurement found.
- `MPM_SHADOW_STORE_AGE_FLOOR`: how recently a version may have been written and still be removed,
  one hour (`2h`, `90m`, `3600` seconds).
  **A proposal**: no measurement settles it, because an install writes its lock row before the version it
  installs, so this floor only covers a copy a person or another tool made by hand.
- `MPM_SHADOW_STORE_WORK_LIMIT`: how long one pass may spend removing,
  2 s (`5s`, `500ms`).
  Sized from the 30 microseconds per file the deletion measurement found,
  so that an install never waits on a long deletion;
  whatever is left is finished by the next install.

## A removal cut short

A removal is two steps:

1. the version's downloads directory and then the version directory are renamed to
   `.mpm-shadow-removing.<name>` beside them (one atomic rename each);
2. those two are deleted.

A run killed between the two steps leaves a hidden `.mpm-shadow-removing.*` directory,
which no scan lists as a version,
and a `.mpm-shadow-removal-unfinished` file in the store root.
The next install runs the pass because of that file and finishes the deletion,
printing one line saying it did.

## What it says

One line per version removed and per version kept, on standard error.
A pass that judges nothing (the stores are inside the bound) prints nothing;
so does a pass that installed nothing and has no removal to finish, which is why it is not even run.

## What it does not touch

- Anything outside the stores, and anything under a build folder.
- `cargo`'s registry cache and this repository's own crates there:
  the shadow's post-run cleanup (`mpm-shadow` acting as `mpm`) still owns that.
- The older copies of this repository's own tools in the other scope's root,
  which the install of an old copy already removes (`managed_losers.rs`).
- A tool directory no visible lock names:
  the scan is driven by the locks' own tool ids,
  so a tool that left every plan is never judged.
- The versions of tools that `pnpm`, `uv`, `rustup` and the `bin` downloader keep in their own homes:
  mpm-shadow never wrote those directories.
