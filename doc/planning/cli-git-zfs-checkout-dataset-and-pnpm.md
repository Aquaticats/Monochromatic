# What a ZFS dataset at `~/Monochromatic` changes for pnpm and other checkout tools

**Non-final draft by an agent, unapproved.** Measured on 2026-11-18 at local offset `-05:00`:
alternating offline installs of the repository lockfile on the host, syscall traces and file inventories in disposable
checkouts, and block sharing, snapshot retention and inotify in a throwaway VM running OpenZFS 2.4.4.
Read the wrapper and upstream source to distinguish shipped behavior from the proposed ZFS route.
No host dataset, pool property, checkout location, or package-manager configuration was changed.

## Conclusion

Keep the store in home if the checkout gets its own dataset at the same path. With the configured global virtual store,
**the boundary does not move the package-payload import**. pnpm imports from `<storeDir>/files` into
`<storeDir>/links`, both in home, and links from the checkout into `links`. In the cross-dataset install trace, pnpm put
**zero imported regular package files and zero package-payload bytes** in the checkout. Four paired offline installs
across the boundary took **0.984 to 1.101 times** their same-round home controls. This does not measure the cost of
copying dependencies for a project-local virtual store. That layout copied 971,230,775 logical payload bytes in a
separate counterexample.

A mounted ZFS clone and a clean install are different cases. The clone gets the original symlink texts, including
relative paths written for `~/Monochromatic`, not for a directory under `~/Monochromatic/.claude/worktrees/`.
Keep the wrapper's relink on the clone route. Keeping the main path and worktree locations does not require moving
the pnpm store. The dataset decision itself remains open; see [the worktree draft](cli-git-zfs-worktree.md),
“Owner's answers of 2026-11-18”, and `WTU` in `AGENTS.md`.

## Where pnpm writes

`pnpm-workspace.yaml:548,577,591` sets `packageImportMethod: clone-or-copy`,
`storeDir: ${HOME}/.local/share/pnpm/store` and `virtualStoreType: global`.
`pnpm store path` reported `~/.local/share/pnpm/store/v11` with pnpm 12.21.0.
The content-addressed files are under that versioned directory's `files`; the materialized package graphs are under
`links`; the checkout's `node_modules` directories contain links, shims and install metadata.
The settings comment at `pnpm-workspace.yaml:568-571` describes per-worktree package imports; it is not the path the
configured global virtual store uses for registry files.

- Each arm had `node_modules` at 229 workspace locations. Across those trees, without following symlinks, both arms
  had 1,664 symlinks: 327 into `store/v11/links` and 1,337 inside the checkout, with zero dangling. Their stored link
  texts totalled 91,900 bytes in home and 90,919 across the boundary. The 981-byte difference is one `../` per global
  link because the test locations differed by one path component. Link text is metadata, not a read or copy of the
  target file's data.
- Each arm had 117 regular files (including `.modules.yaml`, `.pnpm/lock.yaml` and executable shims) and zero regular
  files with more than one link. They held 1,504,261 logical bytes in home and 1,502,154 across the boundary in the
  timed runs. Comparing the last output trees with the input archive found no added paths outside `node_modules`.
- A separate `strace` run at a new cross-dataset checkout path found zero checkout `FICLONE`, hard-link,
  `copy_file_range` or `sendfile` calls for the package imports. It found 1,616 successful `FICLONE` calls **inside the
  shared home store**, at least 28,511,186 bytes referenced by those calls (11 destination paths were gone at inventory
  time). The inherited-state relink at another new path also made 1,616 store-local clone calls. A warm CAFS and a
  populated `links` tree do not guarantee zero work in the shared store for a new path. This repo has path-specific
  `link:` overrides (`pnpm-workspace.yaml:304,391,412,428,457`); pnpm's
  [`lockfileToDepGraph`](https://github.com/pnpm/pnpm/blob/v12.21.0/pnpm11/deps/graph-hasher/src/lockfileToDepGraph.ts#L48-L61)
  includes an absolute target for those graph nodes. The trace proves where the imports went, not which override
  requested each import. The store also records projects. No store entries were pruned to start a run.
- Positive control: changed **only the disposable checkout's** virtual-store setting to `project`, so CAFS-to-
  `node_modules/.pnpm` really crossed datasets. `strace` recorded 13 failed `FICLONE` calls (`EXDEV`, concurrent
  importer probes), then 43,930 positive `copy_file_range` results totalling 971,230,775 bytes and another 43,930 EOF
  results. No successful `FICLONE`, hard link or `sendfile` was recorded for those imports. The final trees had 44,186
  regular files, 973,243,351 logical bytes, and 2,937 symlinks, none to the global store. The difference between final
  inventory and transferred bytes includes generated metadata and shims. The tree showed 260,273,152 bytes in
  `du --block-size=1` during that run, not an estimate of exclusive pool allocation or evidence of shared blocks.

`clone-or-copy` does not try hard links. The [pnpm importer][pnpm-importer] probes a forced clone (`copyFileSync` with
`COPYFILE_FICLONE_FORCE` on Linux), selects a clone or copy importer, and uses plain `copyFileSync` on the copy path.
After the probe, the copy importer is reused; it does not need to try and fail a clone once per file.
The symlink to a package in the virtual store is a separate step from either import operation.

## Offline install cost on the host

Used plain installed pnpm 12.21.0, `pnpm install --offline --frozen-lockfile` with scripts left at repo defaults and
`CI=true`. Used fresh `git archive` extractions of `d507a55f0e64f276dc222e769c4a1a4162b96cce`, the full lockfile and all
244 workspace projects, not the main checkout or an already-installed `node_modules`. Both arms used the real store.
The scopes had `CPUQuota=800%`, `MemoryMax=8G`, `TasksMax=2000`, `IOWeight=1` and a deadline per install.
Measured elapsed wall time through the scope, excluding extraction, tracing and inventory. Used four pairs to get two
pairs in each order and observe run-to-run spread without repeatedly rebuilding the workspace. No cache was flushed.

- Control: scratch in `zpcachyos/ROOT/cos/home`; treatment: disposable folder under `~/.cache` in
  `zpcachyos/nosnap/user-cache`. Both use `aes-256-gcm` with `encryptionroot=zpcachyos`. `zfs get` returned 32 KiB
  records and `zstd-3` in home; 128 KiB and `compression=on` in cache. The treatment is a boundary stand-in, not a
  simulation of every tunable a new checkout dataset could inherit.
- Home then cache: 0.301 s and 0.307 s, **1.021×**. Cache then home: 0.279 s and 0.280 s respectively, **0.993×**.
  Home then cache: 0.276 s and 0.271 s, **0.984×**. Cache then home: 0.310 s and 0.282 s respectively, **1.101×**.
  Each home arm is **1.000×** by definition. The home control alone varied by a factor of **1.091** from minimum to
  maximum. The treatment went both ways and the largest deviation was comparable to that spread. These pairs do not
  establish a repeatable wall-time penalty for putting `node_modules` across the boundary with `global` enabled.
- Settled `du --summarize --block-size=1` over just the `node_modules` trees, including directory and symlink storage:
  9,558,528 bytes in home and 9,546,240 bytes across the boundary. Immediate whole-checkout `du` deltas sometimes
  changed as the archive extraction settled, so they were discarded. Logical payload and link bytes are reported
  under “Where pnpm writes”. Device write volume and exclusive physical space attributable to the installs were not
  isolated on the live host. The counters in “When a copy still shares blocks” isolate allocation on a separate pool.

## When a copy still shares blocks

The kernel's [`vfs_clone_file_range`][linux-ficlone] rejects differing superblocks with `EXDEV` before calling ZFS.
[`copy_file_range`][linux-copy-range] can dispatch across filesystems with the same implementation. In exact-version
OpenZFS, [`zpl_copy_file_range`][zfs-zpl] tries `zfs_clone_range` and falls back to generic/splice byte copy on
`EOPNOTSUPP`, `EINVAL`, `EXDEV` or `EAGAIN`. A successful return from the syscall alone does not distinguish the two.
[`zfs_clone_range`][zfs-clone] requires the same pool and, for two encrypted datasets, matching data keys.
[`dmu_objset_crypto_key_equal`][zfs-keys] compares `zk_guid` for the two dataset keys, not `encryptionroot`.
Sharing an encryption root is not proof that the datasets share a data key.

Node's [`CopyFile`][node-copy] passes the flags to `uv_fs_copyfile`. On Linux, [libuv's copy][uv-copy] calls the
`FICLONE` ioctl for the clone flags and returns the error for `FORCE`. The ordinary data path enters `uv_fs_sendfile`,
which [tries `copy_file_range` first][uv-range] and can fall back to `sendfile` or a read/write loop. The actual pnpm
project-virtual trace reached `copy_file_range`; the source trace is not a substitute for that observation.

The independent guest pool had one file-backed vdev, no other workloads, compression off, 128 KiB records and
`zfs_bclone_enabled=1`. Ran OpenZFS userspace **and** module 2.4.4 on kernel `6.8.0-147-generic`.
For each operation, synced before and after with guest `zpool sync`, recorded exact-byte properties, and compared
contents for successful copies. Each source was 8,388,608 bytes. The two counters in each pair are
`bcloneused` and `bclonesaved`; `allocated` is a separate pool-wide delta including metadata.

- Unencrypted read/write across datasets, negative control: `(0, 0)` remained `(0, 0)`; allocated grew 8,395,776.
  `FICLONE` inside the source dataset, positive control: `(0, 0)` became `(8,388,608, 8,388,608)`; allocated grew
  19,456. Unencrypted `FICLONE` across datasets returned `EXDEV` and changed neither counter.
- Unencrypted `copy_file_range` across datasets: `(8,388,608, 16,777,216)` became `(8,388,608, 25,165,824)`;
  allocated grew 8,192. The unchanged `bcloneused` is expected here: the same source blocks were already in the
  block pointer reference table after the controls. The new reference raises `bclonesaved` without registering new
  unique shared blocks.
- Independently created encrypted siblings with a common `encryptionroot`: `FICLONE` still gave `EXDEV`.
  `copy_file_range` returned the full length, but counters stayed `(8,388,608, 25,165,824)` and allocation grew
  8,399,360. An independently encrypted destination with another encryption root likewise gained no block savings.
- Encrypted dataset to its actual snapshot clone: `FICLONE` still gave `EXDEV`; `copy_file_range` changed counters
  from `(8,388,608, 25,165,824)` to `(16,777,216, 33,554,432)` and allocated grew 12,288. Thus this encrypted
  configuration does share across the boundary through `copy_file_range`.
- Node 26.13.1 `copyFileSync` with no flags returned the same 8,388,608 bytes through `copy_file_range` in all
  cases tested. To the unencrypted sibling it saved 8,388,608 bytes and allocated 8,192. To the encrypted clone it
  saved 8,388,608 bytes and allocated 41,984. To the independently created sibling sharing its encryption root it
  saved zero and allocated 8,430,592 bytes; to the unrelated encryption root it saved zero and allocated 8,399,360.

A host `strace` of `cp --reflink=auto` on a 1,048,576-byte fixture from home to `~/.cache` confirmed `FICLONE` returned
`EXDEV`, then `copy_file_range` returned all the bytes. No host pool counter was used. The earlier worktree draft's
no-savings observation on the host is consistent with the encrypted-sibling result. Its `FICLONE` observation holds;
“no block sharing from any cross-dataset copy” does not. For a new independently created encrypted checkout dataset,
expect the sibling behavior if data really has to cross from home. The checkout and a clone of the checkout can share
keys with each other without either sharing a key with home. The guest does not measure the future host layout.

## Store placement ranked for this project

### Keep the shared store in home (recommended)

- Cost: the measured four-pair result under “Offline install cost on the host”. Global package imports still take
  place in one dataset. Ordinary tracked and ignored checkout state moves to the checkout dataset, not the store.
- Fresh clone: the inherited workspace-internal relative symlinks still work. The inherited links to the home store
  do not know the new depth. In a full-lockfile relocation stand-in, 327 inherited store links were wrong and
  dangling; all 327 resolved after `pnpm install --offline --frozen-lockfile`. In the VM, a real nested ZFS clone
  preserved and broke the external relative link of a minimal fixture in the same way. This is a path-depth problem,
  not a restriction on following a symlink across a ZFS mount. A misplaced link can also make a resolver search a
  parent checkout instead of reporting that the child lacks the package.
- Wrapper: [`SPEC.md` “The repairs”](../../package/git-policy/cli/SPEC.md#L4745) and the
  [repair section of `README.md`](../../package/git-policy/cli/README.md#repairing-package-manager-state-in-a-new-worktree)
  run the first `pnpm` on `PATH` with `install --offline --frozen-lockfile` when there is a lockfile and a real
  `node_modules` directory. They queue that work after the add when the service answers and run it inline otherwise.
  Keep that contract for clones; the probe ran the repair command, not a shipped ZFS add. If the snapshot has no
  `node_modules`, this repair trigger does not install it. The wrapper does not run `mise run prepare:pnpm:store-links`
  as its pnpm repair. That separate task (`mise.toml:1204-1322`) repairs broken `link:` overrides **inside** global-store
  slots, not the inherited links **to** them. The host `realpath "$HOME"` returned `/home/user` in this run; the
  date-specific symlinked-home problem in `doc/troubleshooting/pnpm-symlinked-store-path.md` was not reproduced here.
- Snapshots: retains the owner's “worktree clones and the pnpm store stay snapshotted” and chosen shared store, and
  does not add the CAFS to checkout snapshots. The clone snapshot does not version the target of an external symlink.
  Restoring a checkout snapshot still needs compatible data in the shared store; if it was pruned, a frozen offline
  install cannot fetch what is missing. A new checkout dataset needs to be covered by the proposed recursive home
  snapshots; a mount at the desired path alone does not make a dataset a child of home's dataset in the ZFS tree.

### Put an ignored store inside the checkout dataset

- Cost: `.pnpm-store/` is already ignored (`.gitignore:38`). Seeded a disposable CAFS with a reflink copy of `files`
  and an SQLite backup of `index.db`; started with no `links`. The cold internal-store install did 43,939 successful
  `FICLONE` calls inside the store (at least 970,140,026 logical bytes from surviving destinations; 629 destination
  paths no longer existed at inventory). There were no payload copy or hard-link calls. `du --block-size=1` at that
  point reported 1,289,132,544 bytes for CAFS, 26,891,264 for `links` and 1,325,044,736 for the trial store. These are
  `du` readings, not new exclusive space; `du` does not subtract shared reflink blocks. One untraced relink sample
  of the relocated copy took 1.341 times its immediately paired in-tree origin reinstall, not an install benchmark.
- Fresh clone: links from `node_modules` into the relative `.pnpm-store` still work. A full-lockfile copy at new depth
  had zero broken links; `require.resolve('typescript')` found the copy's own store and `.bin/tsc --version` ran before
  relink. The minimal VM clone confirmed the relative in-tree link worked at mount time. That does not mean the
  package-manager state is all portable: the full-state copy's `.modules.yaml` still recorded the origin's absolute
  `storeDir`. Installing with the clone's own `--store-dir` succeeded, reprocessing 11 packages in the output. Keep
  the wrapper repair if using this option; the current mise config (`.config/mise/conf.d/20-env.toml:22`) injects the
  home store and must agree with the in-tree choice. Test that configuration explicitly before adopting it.
- Snapshots: a clone starts with references to existing store blocks, not an independent physical copy of all of them.
  In the guest the clone had `used=0`, `written=0`, `referenced=8,422,912` bytes at mount. Deleting an 8,388,608-byte
  in-tree blob from the origin left 8,412,160 bytes charged to its snapshots and the blob readable in the clone.
  Future additions, changed store state and outdated contents held by snapshots cost space; the fixture does not
  predict the repo's retention bill. Git ignore cannot exclude a folder from a dataset snapshot. This option keeps
  store contents snapshotted and supports local reflinks, but replaces the agreed shared home store with a changing
  store in each checkout and in each clone. A clone does not gain packages added to the origin after its snapshot.

### Put one shared global store on a separate cache dataset

- Cost: keep both CAFS and `links` on that dataset. A disposable copy in the `~/.cache` dataset, used by the same
  lockfile from a checkout in that dataset, still produced 327 working links to the configured store. The two
  post-seed swaps against the home store for checkouts in `~/.cache` gave **0.348× and 0.957×**; the home-store
  control moved by **2.383×** between them, so this does not establish a speed gain. The disposable seed copy used
  1,309,765,632 bytes by `du` before materializing slots and 2,202,859,520 after the last run. Those readings include
  the seed and describe the trial store, not per-worktree growth or a minimum cost to migrate the live store.
- Fresh clone: external relative links have the same relocation problem as the home-store case. The wrapper still
  needs to relink with the chosen store configured. The stand-in did not create a new host dataset or migrate the
  live store. Don't split `files` and `links` from each other to try to solve this; that would put the boundary on the
  actual import path rather than on the symlink path.
- Snapshots: a store in `zpcachyos/nosnap/user-cache` would be excluded rather than retained by home's recursive
  snapshots. An explicit separately snapshotted store could preserve retention but adds storage policy and migration
  work. A store dataset, even mounted as a folder under the checkout, is not part of the checkout dataset snapshot.
  This option changes the owner's statement against a separate store dataset. Its performance rationale needs
  revisiting if the project ever wants exclusion: with CAFS and `links` together, global imports do not lose reflinks.

**Ranking: shared home store > in-tree store > separate cache store.** Home wins over in-tree because there is no
observed recurring package-payload copy into the checkout to remove, and home keeps the agreed common store without
putting its churn into the checkout's origin and worktree snapshots. In-tree beats separate cache if fresh-clone
readiness is worth a store change: it preserves internal links at mount and includes their targets in the same snapshot,
whereas a separate cache leaves the relink need and changes the agreed snapshot coverage without a demonstrated
install win. None of these choices requires moving `~/Monochromatic` or its worktrees.

## Other writers and consumers

### Cargo

No new dataset crossing on the normal build path. `~/.cargo/config.toml:10` sets
`build-dir = "/home/user/.cargo/build/{workspace-path-hash}"`. `findmnt --target ~/.cargo/build` reports
`zpcachyos/nosnap/cargo-build`. Intermediate output is already separate from checkout `target/`;
[the project decision](../decision/cargo-build-dir-host-cache.md) distinguishes it from final artifacts.
In a disposable Cargo 1.102.0-nightly crate, external build output to an in-dataset `target/` used a successful
`linkat`; with `target/` across a boundary, `linkat` returned `EXDEV` and `copy_file_range` wrote 4,618,568 bytes.
Both executables printed their own fixture's `CARGO_MANIFEST_DIR`. On this host the real final-artifact boundary
already exists both before and after the proposed change. The real shared build directory was not walked or modified.

The Git wrapper's prewarm copies the source workspace's external build slot to the destination workspace's external
slot, both in the Cargo build dataset, and cleans workspace members so path-dependent artifacts are rebuilt
(`package/git-policy/cli/SPEC.md:4761-4796`). A snapshot of the checkout inherits `target/`, not the external
fingerprints. Do not treat the copy of `target/` as proof a new clone can reuse its workspace artifacts. Another
boundary matters for the repair's lockfile recovery: a true clone and the common Git administrative directory are on
different datasets, so the cross-filesystem displaced-lock fallback, not an atomic exchange, applies
(`package/git-policy/cli/SPEC.md:4843-4847`).

### Mise and toolchain launchers

`mise` is the task runner here. `mise.toml:573-583` delegates `toolchain:install` to `mpm-shadow`; it isn't a mise
backend copying every tool into the checkout. A disposable task writing `.cache/a` under mise 2026.11.9 worked on
both dataset device IDs (57 and 60), and wrote its own project path. This tests per-project file creation, not a
complete toolchain install. A boundary does not stop an ordinary write, or resolving a link to an installed tool.
The wrapper still checks `.cache/toolchain-launchers` for records naming another checkout's `toolchain.lock.json`
and reruns `mpm-shadow install --frozen-lockfile` where required (`package/git-policy/cli/SPEC.md:4888-4895`).
That path-bearing state survives a clone and remains the reason for repair, not the dataset boundary. `mise.toml`
and the repo mise configuration also participate in which pnpm store string the task and repair see.

### Git wrapper ignored-file copy

A planned ZFS clone already has the selected snapshot's ignored files, including `node_modules` if present. A plain
fallback add from the main checkout to an ordinary directory under `.claude/worktrees/` stays in the checkout dataset
and can still reflink. That is not true for a fallback copy from a mounted worktree clone. The wrapper stages ignored
files beside the destination (`worktree_copy_stage.rs:209-217` under `package/git-policy/cli/src/native/worktree_copying/`),
tries `FICLONE` for each regular file, then `std::io::copy` (`worktree_copy_stage_leaf_copying.rs:234-249`).
It publishes with a hard link from the stage when possible, else a copy (`worktree_copy_install_entry.rs:239-264`).
A destination that is itself a mount can separate its stage in the parent from the destination. Ordinary source and
staging directories on one dataset do not cost a cross-dataset deep copy just because `/home` is now separate.

The source copy path rebases an outgoing relative symlink to an absolute target
(`worktree_copy_symlink_target.rs:1-19` in the same folder). A ZFS snapshot and clone do not apply that code. Even
a rebased pnpm link is not a substitute for relink: the global package graph may need links to the new checkout.
The installed wrapper refused to commit the disposable fixture without a user commit service, so the ignored-copy
claim is source-backed, not a full wrapper syscall measurement. The host `cp` and guest clone traces verified the
filesystem operations separately; they did not test the wrapper's staging implementation or a shipped ZFS route.

### Editor file watchers

The new filesystem can be watched after it is mounted. A watch on the old directory inode is not a watch on the
mounted filesystem; [`inotify(7)`][inotify] documents both the mount-over case and the lack of recursive watches.
In the VM a watch on `.claude/worktrees` (descriptor 1) saw the parent control file. A watch on its mountpoint
placeholder installed before the clone mount (descriptor 2) saw no write inside the clone. A watch installed on the
mounted clone (descriptor 3) saw `IN_CREATE` for the same write. A watcher registered against a replaced checkout
must refresh or reopen it; one that already watches the live checkout can keep that watch while a new mount beneath
it needs a new watch or a rescan. Whether an individual editor recursively discovers the new mount or excludes
`.claude/worktrees` is its own policy. No editor GUI or watcher implementation was modified or tested.

## Evidence and remaining checks

Raw measurements and scripts: `~/temp/agent/checkout-zfs-pnpm-install-measurements-2026-11-19/keep/`.
The isolated pool, main guest and transient template guest were removed. This draft does not settle the mount
mechanism, benchmark clone creation or relink scheduling in the proposed wrapper, measure long-term snapshot churn,
or measure a cold registry download. Check the actual live checkout migration, the installed wrapper with its real
service and the editor in use before treating any of those as verified. No recommendation here changes an owner decision.

[pnpm-importer]: https://github.com/pnpm/pnpm/blob/v12.21.0/pnpm11/fs/indexed-pkg-importer/src/index.ts#L119-L140
[linux-ficlone]: https://github.com/torvalds/linux/blob/master/fs/remap_range.c#L371-L400
[linux-copy-range]: https://github.com/torvalds/linux/blob/master/fs/read_write.c#L1480-L1494
[zfs-zpl]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/os/linux/zfs/zpl_file_range.c#L87-L128
[zfs-clone]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/zfs/zfs_vnops.c#L1608-L1651
[zfs-keys]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/zfs/dsl_crypt.c#L270-L300
[node-copy]: https://github.com/nodejs/node/blob/v26.13.1/src/node_file.cc#L3023-L3039
[uv-copy]: https://github.com/nodejs/node/blob/v26.13.1/deps/uv/src/unix/fs.c#L1358-L1387
[uv-range]: https://github.com/nodejs/node/blob/v26.13.1/deps/uv/src/unix/fs.c#L981-L1074
[inotify]: https://man7.org/linux/man-pages/man7/inotify.7.html#:~:text=If%20a%20filesystem%20is%20mounted%20on%20top
