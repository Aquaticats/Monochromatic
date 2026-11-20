# The start probe for the ZFS worktree route

**Non-final, unapproved proposal by an agent.** This document designs one open item of the accepted ZFS worktree
design, "The start probe a kernel-release answer asks for", so that step 6 of the implementation plan can be
written. A Pi session wrote it on 2026-11-19 and revised it in the rounds listed under [Revisions](#revisions); it
is not accepted. It was written without running a VM, a probe or any pool command, from the sources listed under
[Sources read](#sources-read). What it checked: the
accepted design, the plan (every section list and step it names) and the measurements record, OpenZFS `zfs-2.4.4`
sources, the running kernel's exported system calls, this host's `include/uapi/linux/mount.h`, systemd v262
`service.c`, `unit.c` and `exec-invoke.c`, Linux v6.8 and v7.3 `fs/super.c`, `fs/fsopen.c` at v5.2, v6.5, v6.6 and
v7.3, and the wrapper's own code (fetched, because this host's 7.3.5 tree ships no `fs` sources). No kernel source
was read for the first draft, which is how a non-existent mount attribute and a wrong `errno` survived it; the
kernel statements under [The kernel the design names](#the-kernel-the-design-names) name the version each was read
from. What stays open:
the owner questions under [Questions for the owner](#questions-for-the-owner), and every measurement under
[Owed measurements](#owed-measurements), none of which ran. The design is accepted with its open items still
open, and this file changes no decision of it. Nothing here authorizes work on this host's pool.

## Revisions

Each revision is labelled with the commit's own timestamp, read with `git log --format=%cI`, and names the work it
carried.

- `2026-11-19T18:02:41-05:00` (commit `4167f50724`): the first draft.
- `2026-11-19T18:40:12-05:00` (`eee32e31fd`): the answer to a gate review of that draft, by the commit that says
  so.
- `2026-11-19T18:41:09-05:00` (`fa52e2e948`): the mount header fields cited at their exact lines.
- `2026-11-19T19:27:20-05:00` (`45aca9ab4a`): the answer to a first gate re-check.
- `2026-11-19T19:33:27-05:00` (`370c6cb9e2`): the start-probe arms, the txg window and the owner questions
  tightened.
- `2026-11-19T20:26:05-05:00` (`10f2435d34` and this entry's commit): the answer to a second gate re-check. Its
  findings were weighed one by one, and every one earned a fix, with none refused: the duplication in the source
  question,
  which listed one option twice; the start question, which was a confirmation with one option and moved to the
  writer's list; the move and unmount arm, which could not unmount while the worker held references on the mount;
  the holder protocol, now stating every pipe and its endpoints, the parent-death race and the process-group call;
  the txg window, which now carries a start fence, an end fence and an idle control; and `ExecStartPre=-`, which
  keeps a probe crash from skipping the reconcile. Also fixed: the marked options and the costs charged to one
  way, the sanoid facts the source question left out, the fold-in that cost nothing, the scope of the "never opens
  `/dev/zfs`" statement, the Way 5 layout facts, the teardown arm's own unmount, two option sentences, the
  control-directory path, the unnamed handler modes and the prose the checkers do not count. The measurements this
  round added are under [Owed measurements](#owed-measurements).

## The open item, the answer, and what bounds them

### The open item

The accepted design leaves this item undesigned, in the words of the [proposal][proposal], lines 1630 to 1635:

> The start probe a kernel-release answer asks for. The answer's words: "Probe at start". What the probe exercises
> (descriptor attach, staging unmount, the namespace refusal), in which namespace and as which user it runs, what
> it changes on the machine at start, what it leaves behind when it fails or is interrupted, and what a refusal
> prints for the caller and `doctor` are all undesigned. Until that exists, the runtime module gate stays the only
> start check this design describes ([ABI gate and tests](cli-git-zfs-worktree-design-proposal.md#abi-gate-and-tests)).

### The answer it carries

The item's answer is recorded under "Kernel-release readiness" in the [proposal][proposal], lines 1294 to 1306:

> Kernel-release readiness. Decided: "Probe at start", a functional probe at start, against the ranking the
> question round carried, which put an exact allowlist first. The probe's own steps, its side effects at start and
> the way it refuses have no design yet;
> [Open items the decisions leave](cli-git-zfs-worktree-design-proposal.md#open-items-the-decisions-leave)
> names them. The runtime module gate of "A: Test plus runtime gate" stays, and the unselected allowlist's facts stay:
> it would qualify a release only by built-product runs on that release and would disable the managed route after
> every CachyOS kernel package update until a run passes, which makes existing unmounted clones unavailable in the
> meantime.

The item continues with the state of `7.3.5-1-cachyos` qualification.

The same [proposal][proposal], lines 617 to 631, states what the project weighed and what the answer admits:

> The project decided a functional probe at start, answering "Probe at start", against the ranking the question
> round carried, which put an exact allowlist first. The probe's steps, its side effects at start and the way it
> refuses have no design yet ...; the project's answer, "test plus runtime gate", covers the ZFS module check
> rather than a kernel-release list, and that gate stays. The unselected allowlist would qualify a release only by
> passing built handler attach, staging unmount, refusal and namespace controls on that release, compared
> independently of the ZFS ioctl ABI gate; fixture results alone do not qualify it, and under that allowlist
> `doctor` and preflight would report the running release and any missing qualification. ... The probe admits
> untested releases and leaves unmeasured behaviour to appear on the host, where a failure shows as a refused
> mount or as one the probe did not detect (reasoning); its own design is not described here, and neither is the
> allowlist's qualification run, which is named under
> [Acceptance tests the build still owes][owed-tests].

### The decisions that bound it

Each of these is decided, and this proposal keeps every one of them.

- The start-time module gate stays exactly as decided, in the [proposal][proposal], lines 235 to 244:

  > At start, before the first ioctl, both binaries must prove compatibility with the loaded module and target
  > ABI, including module release, architecture, endianness, request contract and kernel/module BTF layout. A
  > missing device, missing required BTF or mismatch disables raw requests, including reads; no force flag. ...
  > On gate refusal the wrapper uses the applicable same-dataset reflink or ordinary Git route for an unused
  > slot. ... Report what failed, including when an update makes existing clones temporarily unavailable. A
  > failed gate does not authorize an unrelated mount or cleanup; keep registrations and durable recovery state
  > intact.

  The probe adds a functional check beside that gate and replaces none of it; the quoted decision covers the ZFS
  ioctl device and layout, not a kernel-release list (the [proposal][proposal], lines 617 to 620).

- The unselected allowlist's qualifying arms name what the answer was chosen against: "built handler attach,
  staging unmount, refusal and namespace controls on that release" (the [proposal][proposal], lines 620 to 623).
  They are the shape of the functional evidence the probe is expected to seek, though the answer does not bind
  the probe to every one of them.

- The probe admits untested releases: "The probe admits untested releases and leaves unmeasured behaviour to
  appear on the host" (the [proposal][proposal], lines 629 to 630). A partial pass is therefore not a defect of
  this design, provided the missing piece is reported; whether a missing piece may enable the route is
  [Q4](#q4-whether-an-unexercised-arm-may-enable-the-route).

- Existing bindings and intent survive an unavailable route: "it preserves existing bindings and intent while
  the route is unavailable" (the [proposal][proposal], line 1306), and an unused slot takes "the applicable
  same-dataset reflink or ordinary Git path" (line 626).

- A failed start check is not permission to touch anything else: "A failed gate does not authorize an unrelated
  mount or cleanup; keep registrations and durable recovery state intact" (the [proposal][proposal], lines 243
  to 244).

- `doctor` is read-only: "Setup findings go to standard error and do not change `doctor`'s exit status. `doctor`
  does not start a service, create a test dataset, repair registrations or change configuration" (the
  [proposal][proposal], lines 1153 to 1155). The probe's result reaches `doctor` by being read, never by
  `doctor` running a probe.

- The wrapper must start without OpenZFS: "the wrapper must start even when OpenZFS is absent" (the
  [proposal][proposal], lines 152 to 153).

- Each add requires a current readiness report: "Require a current handler readiness report for the actual pool,
  checkout and boot, not just a present executable" (the [proposal][proposal], lines 351 to 352). The probe's
  result can live beside that record, which the wrapper already must read.

- The wrapper owns snapshot and clone creation, and the handler does not destroy anything: the decision record
  gives the wrapper "unprivileged snapshot and clone creation" and the unit "clone mount and unmount, not Git or
  automatic deletion" ([decision][decision], lines 22 to 26). A root probe that created or destroyed a snapshot
  or clone at start would need a request the accepted design does not have (the [proposal][proposal], lines 211
  to 212).

- The build owes one acceptance test for it: "Run the built probe against an untested release and against one it
  must refuse, and record what it changes at start, what it leaves behind and what it prints" (the
  [proposal][proposal], lines 1841 to 1843). Step 6 also owes its own VM arms: "an untested release, and a
  module whose field layout differs from the approved one, each asserting that raw requests are disabled" (the
  [plan][plan], lines 739 to 745).

- No second request channel is added for mounting: the design keeps "the project's ZED handler for in-session
  requests **plus** a systemd unit for boot and shutdown, not a second request channel for mounting" (the
  [proposal][proposal], lines 478 to 480), and fixes the intent vocabulary to `v1:<nonce>:mount` and
  `v1:<nonce>:unmount` (lines 537 to 539). A probe cannot be triggered by a third intent or a new channel, which
  is why this design places it in the existing boot unit.

- The handler's `boot-reconcile` and `shutdown-unmount` modes "Run both modes in the host mount namespace" (the
  [proposal][proposal], line 496), and this proposal keeps that. The probe is not one of those modes: it attaches
  nothing for a worktree, it runs in a child mount namespace of its own, and the limit that it therefore cannot
  prove the host's propagation is stated under [Limits](#limits).

- No production kernel request and no unmount runs early: the [plan][plan], lines 82 to 88 record that "No
  production kernel request runs before the first two, and no unmount runs before the third", and the third is
  the staging design. Step 6 is the start probe's own step, and step 9, which builds the product's unmount and
  holder scan, depends on steps 2, 4, 6 and 8 (the [plan][plan], lines 967 to 971). Whether that sentence bars
  the probe's own unmount arm is the owner's reading, put in one line as
  [Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm); the arm is probe-private code either way, because the
  product's held-descriptor unmount does not exist when step 6 lands.

- The plan files this item as "step 6, blocking, writer design work" (the [plan][plan], lines 2200 to 2203), and
  lists the start probe among "The three blocking designs, as documents ... the critical path for steps 6 and 9"
  (the [plan][plan], lines 2102 to 2107).

## Facts

### The design's own statements

- The start-time gate runs in both binaries "At start, before the first ioctl" (the [proposal][proposal], line
  235). That gate is a set of reads. This proposal reads "start" in its sentence as the start of a process that
  will make raw requests, which covers every wrapper invocation and the boot unit's `ExecStart` (the
  [proposal][proposal], lines 500 to 505). A functional probe needs root and mounts, so it cannot run in every
  wrapper invocation; this design places it in the boot unit and settles that as a writer's item under
  [What a writer settles without asking](#what-a-writer-settles-without-asking).
- The probe is a separate check from the module gate, and the gate's failure path is decided while the probe's is
  not (the [proposal][proposal], lines 617 to 620 and 1294 to 1306).
- The syscall floor is stated as Linux 5.6 for `openat2` and 5.2 for the mount APIs, and the demonstrated guest
  baseline is `6.8.0-147-generic` with OpenZFS 2.4.4 (the [proposal][proposal], lines 612 to 615).
- The descriptor route needs a verified private immediate parent and verified private staging, and a shared
  parent refused the staging move (the [proposal][proposal], lines 738 to 739; [measurements][measurements], item
  3).
- The mount handler runs in the host mount namespace, `ExecStart` verifies ABI, keys, checkout and the private
  propagation the boot unit sets, then reconciles intent and publishes readiness (the [proposal][proposal],
  lines 492 to 500).
- `doctor` extends the read-only filesystem report and never starts a service or creates a test dataset (the
  [proposal][proposal], lines 1149 to 1155).
- The wrapper has no ZFS code today: `package/zfs` does not exist in this checkout, and no source file under
  `package/git-policy/cli/src` calls `fsopen`, `fsconfig`, `fsmount`, `move_mount` or `setns` (repository
  search, 2026-11-19). Tests do start the `unshare` program to enter a namespace, which is how the existing
  private-namespace tests run (for example
  `package/git-policy/cli/src/native/binary/binary_worktree_removal_around_a_volume_tests.rs:166` and `:194`);
  no wrapper code path enters one. The probe is new code, not a change to existing wrapper behaviour.

### OpenZFS at `zfs-2.4.4`

These lines are from the tag's commit `71a9f9578616`, cloned on 2026-11-19 from `openzfs/zfs`.

- The ZFS filesystem type declares `FS_USERNS_MOUNT` (`module/os/linux/zfs/zpl_super.c:1096` and `:1098`), so a
  user namespace may ask for the filesystem, which is the declaration the design's H1 gate 7 already expects to
  be refused in a confined namespace (the [proposal][proposal], lines 1940 to 1942).
- The kernel's own check refuses that request: `zfs_domount` returns `EPERM` when the caller is not in the global
  zone and the dataset is not visible there (`module/os/linux/zfs/zfs_vfsops.c:1357` to `:1366`).
- `inglobalzone` is true only for a credential whose user namespace is `init_user_ns`
  (`module/os/linux/spl/spl-zone.c:396` to `:403`), and a dataset can be made visible in another user namespace
  only by `zone_dataset_attach`, whose credential check requires uid 0 (`:157` to `:164`, called at `:192`).
  Only a process in the init user namespace can attach a dataset there, and the ioctl that asks for it is
  registered with `zfs_secpolicy_config` (`module/os/linux/zfs/zfs_ioctl_os.c:215` to `:216`), the pool
  administration policy (`module/zfs/zfs_ioctl.c:1155` to `:1160`).
- A name that is not a dataset or a snapshot of one has no objset to hold, and the mount path reports that as
  `ENOENT`: `zpl_get_tree` returns the error from `dmu_objset_hold`
  (`module/os/linux/zfs/zpl_super.c:857` to `:859`), and the hold sets `ENOENT` for a missing head dataset and
  for a missing snapshot (`module/zfs/dsl_dataset.c:825` to `:847`). The probe therefore detects a missing probe
  dataset, or a missing snapshot of it, only as the `ENOENT` from `fsconfig(create)`, and an unimported pool answers
  the same `ENOENT` from `spa_open` (`module/zfs/spa.c:6478` to `:6482`), so the record names the arm and not the
  cause.
- The fs_context path takes `fc->source` as the dataset name, holds that objset, and calls `sget_fc` keyed on it;
  it calls `zfs_domount` only when the returned superblock has no root yet
  (`module/os/linux/zfs/zpl_super.c:850` to `:920`). A second attach of an already mounted dataset therefore
  reuses the existing superblock and skips the dataset mount work, including the key check
  (`zpl_test_super` at `:345` to `:357`, the `s_root` test at `:905` to `:921`).
- A read-only request against an existing read-write superblock refuses with `EBUSY`, except for a snapshot
  (`module/os/linux/zfs/zpl_super.c:936` to `:943`).
- The fs_context accepts `source` and the negatable `exec`, `suid` and `dev` options
  (`module/os/linux/zfs/zpl_super.c:579` to `:583`, applied at `:669` to `:676`).
- The fs_context operations table wires `parse_param`, `get_tree`, `reconfigure` and `free`
  (`module/os/linux/zfs/zpl_super.c:1022` to `:1032`). `zpl_get_tree` sets the context's root dentry
  (`:945` to `:949`), which is what `fsmount` turns into a mount and `move_mount` then places.
- The dataset mount path in the kernel makes no ZFS ioctl request: `zpl_get_tree` calls `zfs_domount`, which calls
  `zfsvfs_create` (`module/os/linux/zfs/zfs_vfsops.c:1347` to `:1375`; the call is at `:1375`). A probe on this
  path therefore needs no `/dev/zfs`, and it proves nothing about the device or the ioctl contract.
- The snapshot branch of `zfs_domount` is not the dataset branch: for a snapshot it sets `z_issnap` and skips
  `zfsvfs_setup` (`module/os/linux/zfs/zfs_vfsops.c:1414` to `:1437`), and `zfsctl_create` is skipped too
  (`:1456` to `:1457`). So the probe's snapshot mount runs neither the callback registration, the ZIL open and
  replay and the unlinked-set drain that `zfsvfs_setup` does (`:725` to `:823`), nor the `.zfs` control
  directory's creation. Attaching an existing snapshot checks the key on the way: `zfsvfs_create` calls
  `dmu_objset_own` with decryption asked for (`module/os/linux/zfs/zfs_vfsops.c:650` to `:660`, the `B_TRUE`
  argument at `:660`; the function is defined at `module/zfs/dmu_objset.c:816`), which is the path item 5
  measured refusing with `EACCES` while the key was unavailable.
- Delegation is checked for clone creation, not for the kernel's attach: the clone policy requires `clone` on the
  origin (`module/zfs/zfs_ioctl.c:1138` to `:1141`) and `create` and `mount` on the parent (`:1143` to `:1148`),
  while `zpl_get_tree` and `zfs_domount` check no delegation. A root-run attach therefore proves the kernel path,
  not the account's grants.
- An attach of an encrypted dataset with an unavailable key refuses at `fsconfig(create)` with `EACCES`
  ([measurements][measurements], item 5, line 231), which is the observable that shows `zfs_domount` ran.
- A mounted dataset's `.zfs/snapshot` directory lists its snapshot names through `dmu_snapshot_list_next`, with
  no ioctl and no delegation check (`module/os/linux/zfs/zpl_ctldir.c:281` to `:320`). A root process can
  therefore name a snapshot for the probe by reading a directory, without `/dev/zfs`.
- Reading that directory is not the same as naming a snapshot entry in it. `zpl_snapdir_lookup` flags each entry
  of `.zfs/snapshot` for automount (`module/os/linux/zfs/zpl_ctldir.c:255` to `:277`), and the automount runs
  `mount` through `call_usermodehelper` (`module/os/linux/zfs/zfs_ctldir.c:1190` to `:1198` and `:1344`). The
  helper takes a mount point even from another mount namespace (`module/os/linux/zfs/zpl_super.c:490` to `:493`),
  so a lookup by name inside `.zfs/snapshot` can put a mount in a namespace the probe does not control. That the
  helper mounts in the init mount namespace specifically is an inference, because the source read shows the call
  and the path, not the helper's own namespace; this design does not rely on it, because it never names an entry
  there.
- The CLI's legacy-mountpoint refusal is the userspace branch for an explicit `zfs mount`: it prints
  `cannot %s '%s': legacy mountpoint` (`cmd/zfs/zfs_main.c:7240` to `:7248`), not the fs_context path; item 10
  measured that branch and the descriptor attach beside it ([measurements][measurements], item 10, lines 359 to
  375).
- No pool-history call exists on the mount path: the only history write in the VFS file is the upgrade
  (`module/os/linux/zfs/zfs_vfsops.c:1911` to `:1964`). Pool history records administrative commands, so it
  cannot show a mount or an atime write either way. The per-pool txg kstat can:
  `/proc/spl/kstat/zfs/<pool>/txgs` is installed by `module/zfs/spa_stats.c:288` to `:304`.
- A dataset that is mounted keeps its wrapping key: a key mapping holds the wrapping key
  (`module/zfs/dsl_crypt.c:624` to `:626`), and `spa_keystore_unload_wkey_impl` refuses with `EBUSY` while that
  refcount is nonzero (`:902` to `:903`). Item 5 unmounted the checkout and home before unloading the key
  ([measurements][measurements], item 5, lines 221 to 222).

### The kernel the design names

- This host boots `7.3.5-1-cachyos` with module version `2.4.4-1` ([measurements][measurements], lines 22 and
  671 to 681; read-only).
- On that running kernel, `/proc/kallsyms` exports `__x64_sys_fsopen`, `__x64_sys_fsconfig`,
  `__x64_sys_fsmount`, `__x64_sys_move_mount`, `__x64_sys_openat2`, `__x64_sys_mount_setattr` and
  `__x64_sys_unshare` (read-only grep of the symbol names, 2026-11-19). The syscalls the design names exist
  there; the listing proves presence on this boot, not behaviour.
- `/sys/kernel/btf/zfs` and `/sys/kernel/btf/spl` are readable, `/proc/filesystems` lists `nodev zfs`, and
  `/dev/zfs` is mode `0666` on this host (read-only, 2026-11-19). Device presence alone is weak evidence, which
  is why the design's own gate checks access and layout.
- The module's `srcversion` reads `A3553C308C29FD4B6595A72` and its `version` reads `2.4.4-1`, while
  `/sys/module/zfs/notes/` is empty, so a build ID is not exported there and `srcversion` is the readable build
  identity on this kernel (read-only, 2026-11-19).
- The root filesystem is on ZFS and the module is in use, so the module cannot be unloaded or reloaded without a
  reboot: `findmnt --target /` names `zpcachyos/ROOT/cos/root` ([measurements][measurements], lines 26 to 29) and
  `/proc/modules` lists `zfs` with a use count of 10 (read-only, 2026-11-19).
- Unprivileged user namespaces are enabled here: `/proc/sys/user/max_user_namespaces` reads `252369` and
  `/proc/sys/kernel/unprivileged_userns_clone` reads `1` (read-only, 2026-11-19).
- The design's checkout dataset does not exist yet on this host: `findmnt --target /home/user/Monochromatic`
  resolves to `/home` ([measurements][measurements], line 31). A probe on this host today would have no managed
  ZFS source to attach.

### The kernel headers and source

- This host's `/usr/include/linux/mount.h` (package `linux-api-headers` 1:7.1-1) lists every `MOUNT_ATTR_*`
  value (`:118` to `:128`) and `struct mount_attr` with separate `attr_set`, `attr_clr`, `propagation` and
  `userns_fd` fields (`:133` to `:138`). There is no `MOUNT_ATTR_PRIVATE`; propagation is its own field of that
  struct (`:136`), and it takes the `MS_*` propagation values (`MS_PRIVATE` at `:32`, the value list beside
  `mnt_propagation` at `:171`).
- That copy is byte-identical to this host's 7.3.5 tree, `/usr/src/linux-cachyos/include/uapi/linux/mount.h`,
  apart from the include guard name (diff, 2026-11-19), so the mount-header facts this design cites are the
  running kernel's own `UAPI`, not another release's.
- Linux v6.8 `fs/super.c` (fetched; the 7.3.5 tree on this host ships no `fs` sources) states the matched
  superblock rule in the `sget_fc` doc comment: an extant superblock is returned unless the caller's user
  namespace differs or the context asked that reuse is not allowed, and both cases return `EBUSY`
  (`fs/super.c:715` to `:718` for the rule, the check at `:779`, the return at `:786`). Linux v7.3 has the same
  check at `fs/super.c:906` to `:915`. `zfs_domount`'s `EPERM` is therefore reached only for a new superblock.
- Exclusive reuse is the freshness oracle this design uses. `FSCONFIG_CMD_CREATE_EXCL` is the command (value 8:
  this host's `/usr/include/linux/mount.h:106`; Linux v7.3 `fs/fsopen.c:294`). It sets `fc->exclusive`
  (`fs/fsopen.c:217` to `:229`), which `sget_fc` turns into `EBUSY` for any matched extant superblock. ZFS
  reaches that check and is not a legacy context, so the command is accepted: `zpl_get_tree` calls
  `sget_fc(fc, zpl_test_super, set_anon_super_fc)` (`module/os/linux/zfs/zpl_super.c:872`).
- The command is newer than this design's stated syscall floor. Linux v6.6 carries it (`fs/fsopen.c:293`) and
  v6.5 does not: there an unknown command returns `EOPNOTSUPP` (`fs/fsopen.c:360` to `:361`). A kernel that
  refuses the command has no freshness oracle, which [Limits](#limits) states and
  [M1](#m1-which-source-exercises-the-dataset-mount-path) records.

### The measurements record

- Item 2 measured the descriptor walk refusing a symbolic link, an escape, a procfs magic link, bind and ZFS
  mount crossings, and changed device, inode, uid, mode and ACLs, with a valid attach as the success control
  ([measurements][measurements], item 2, lines 74 to 116).
- Item 3 measured a shared parent refusing the staging move with `EINVAL` and only a private checkout allowing
  the move and unmount ([measurements][measurements], item 3, lines 118 to 168).
- Item 4 measured a fresh process reopening and moving an attached clone, a busy unmount leaving it staged, and
  a wrong-source control that reused mount ID, device and inode and was still refused
  ([measurements][measurements], item 4, lines 172 to 213).
- Item 5 measured `fsconfig(create)` refusing with `EACCES` while the key was unavailable, before `fsmount`
  ([measurements][measurements], item 5, line 231), and a later descriptor attach passing after key load
  (lines 243 to 245). It unloaded the key only after unmounting the checkout and home (lines 221 to 222).
- Item 7 measured `nosuid` stopping setuid elevation against an unrestricted positive control
  ([measurements][measurements], item 7, lines 283 to 300).
- Item 15 measured a root scan grouping tasks by mount namespace, finding both post-attach holders, and finding
  none after they stopped ([measurements][measurements], item 15, lines 612 to 660). It recorded that a
  process-visible scan is not proof of complete absence (lines 662 to 666).
- Item 16 recorded that no built-in or local image booted `7.3.5-1-cachyos` with OpenZFS `2.4.4`, and that the
  remote catalog was not queried ([measurements][measurements], item 16, lines 668 to 706).
- The guest every measured item ran on is `6.8.0-147-generic` with OpenZFS userspace and module `2.4.4-1`
  ([measurements][measurements], lines 37 to 39, 669 to 673).

### The wrapper's code

- The no-crossing walk already exists: `RESOLVE_NO_XDEV`, `RESOLVE_NO_MAGICLINKS`, `RESOLVE_NO_SYMLINKS` and
  `RESOLVE_BENEATH` are defined at `package/git-policy/cli/src/native/subvolume/fs_beneath.rs:44` to `:54`,
  `ObjectId` at `:80`, the resolve mask at `:163`, `open_root` at `:184` and `open_directory_at` at `:203`. The
  wrapper already calls `libc::syscall` for `openat2` (`:167`), so the probe's other syscalls need no new
  dependency. `open_root` and `open_directory_at` open with `O_RDONLY` and `O_DIRECTORY` and read no directory
  entry (`:190`, `:204`), so a walk changes no access time.
- Mount facts are already read from `/proc/self/mountinfo`: `MOUNTINFO_PATH` at
  `package/git-policy/cli/src/native/subvolume/fs_mount_options.rs:37`, `MountFacts` at `:52` and
  `read_mount_facts` at `:339`.
- A boundary walk that refuses a nested mount and names it already exists: `find_boundary_below` at
  `package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs:465` and its `_except` form at `:483`,
  with `device_of` at `fs_beneath.rs:244` and `handle_path` at `:307`. The probe's mount-crossing refusal control
  can reuse them.
- The `doctor` filesystem report is the section to extend: `SECTION_MARKER` at
  `package/git-policy/cli/src/native/health/doctor_filesystem.rs:51`, `gather_facts` at `:227`, `render_section`
  at `:366` and `checkout_section_with` at `:455`.
- The report's read ceiling is `REPORT_SCAN_CEILING`, 5 s, at
  `package/git-policy/cli/src/native/service/wait_ceilings.rs:1280`, measured on this host on 2026-11-09
  (`:1261` to `:1279`). The design calls it a guest trial budget, not a shipped mount limit (the
  [proposal][proposal], lines 750 to 752), and the plan records that the design's citation of it is its owner's
  to correct (the [plan][plan], lines 250 to 259).
- A cached start check already exists: `binary_identity`, `read_version_text` and `store_version_text` in
  `package/git-policy/cli/src/native/invocation/git_version_cache.rs:1` to `:23` (module doc),
  `binary_identity` at `:91`, `read_version_text` at `:238` and `store_version_text` at `:283` key a per-user
  cache on the executable's canonical path, size, times, device and inode.
- The Git check is the precedent for a cheap start check: `MINIMUM_GIT_VERSION` at
  `package/git-policy/cli/src/native/invocation/git_version.rs:52`, one version question per invocation, a
  warning and continued command on an older Git, and a stop only when the version cannot be read
  (`package/git-policy/cli/SPEC.md:18651` to `:18680`).
- The wrapper's tests already run an unprivileged private mount namespace and skip with a stated reason where
  user namespaces are refused: `a_private_mount_namespace_is_available` and `inside_a_private_mount_namespace` in
  `package/git-policy/cli/src/native/binary/binary_worktree_removal_around_a_volume_tests.rs:159` to `:200`.

## Assumptions

Each assumption is stated so a measurement can break it, and the measurement that would is named.

- A private mount namespace made by root from the init user namespace can attach a ZFS dataset, because
  `zfs_domount` sees the init user namespace and `zone_dataset_visible` returns visible and writable there
  (`module/os/linux/spl/spl-zone.c:297` to `:307`). The measurement that breaks or confirms it is
  [M1](#m1-which-source-exercises-the-dataset-mount-path), which shows the attach happening at all, and
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged), which shows it staying inside the namespace.
- Once the probe's namespace has been made private, its attaches and unmounts do not reach the host mount table.
  This is reasoning from mount propagation, not a measurement in this design's shape.
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged) measures it, and item 3 shows a shared parent does
  propagate and refuses the staging move ([measurements][measurements], item 3).
- A source that no namespace mounts yet reaches `zfs_domount` and the key check, while a source with an extant
  superblock skips both. The oracle is `FSCONFIG_CMD_CREATE_EXCL`, which answers `EBUSY` instead of attaching when
  any extant superblock matches ([The kernel headers and source](#the-kernel-headers-and-source)), and
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures it. With the checkout mounted, its key is
  loaded, so the refusal itself is a regression arm only, as
  [M1](#m1-which-source-exercises-the-dataset-mount-path) states.
- A snapshot attach records no pool history entry and changes no dataset property. This is an inference from the
  kernel's mount path having no history call (`module/os/linux/zfs/zfs_vfsops.c:1911` to `:1964`), and
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged) reads the txg kstat, which can show a write, beside it.
- A killed probe leaves nothing in the host mount table and nothing outside the record's directory, because
  every mount it makes lives in a namespace that dies with it and every file it writes lives in that directory or
  on the tmpfs mounted inside it. The measurement is [M3](#m3-interruption-leaves-nothing).
- The account cannot attach any ZFS source, even inside a user namespace, so every ZFS arm of the probe runs as
  root. This rests on the `EPERM` check, the init-namespace test and the uid 0 attach check named under OpenZFS
  at `zfs-2.4.4`, together with the design's own H1 gate 7 expectation (the [proposal][proposal], lines 1940 to
  1942); the measurement is [M6](#m6-user-namespace-limits), which expects `EPERM` on a source no namespace
  mounts and `EBUSY` on one that is mounted.
- The probe's cost is small enough for the boot unit to pay on every boot of this host. No measurement exists,
  and this design refuses to state a duration; [M4](#m4-what-the-probe-costs) measures it against a control arm
  of the same binary, and measures the delay it adds to the first login.
- Every allowlist arm can be exercised without creating a clone, using a read-only snapshot as the attach
  source. The clone's exact shape (its encryption root and its slot properties) is then not exercised, which is
  the clone-shape limit under [Limits](#limits);
  [M1](#m1-which-source-exercises-the-dataset-mount-path) and
  [M8](#m8-the-real-clone-attach-as-evidence) measure what the snapshot arm does and does not cover.

## Limits

- The probe cannot prove host-namespace propagation, a clone's real slot, or ZED ordering. Only the handler's
  actual attach at a real slot does that, which is why [Way 4](#way-4-the-products-first-attach-as-the-probe)
  keeps its evidence and why [M8](#m8-the-real-clone-attach-as-evidence) measures it.
- The probe's mount lands in a directory of its own tmpfs, not in the checkout's slot, so the probe does not
  show the real slot, the account-owned staging folder, or the checkout's propagation into a shared parent.
  Those belong to step 9's staging design and to [Way 4](#way-4-the-products-first-attach-as-the-probe)'s
  record.
- The probe's unmount arm is probe-private. It proves that a second process can move the mount to a move-target
  directory in the probe's own root-only tmpfs and unmount it there by path. It does not exercise the product's
  unmount from a held descriptor, which has no design until step 9 (the [proposal][proposal], lines 681 to 683;
  the [plan][plan], line 265).
- The probe's namespace scan is probe-private. It does not exercise step 9's holder scan (the [plan][plan], lines
  909 to 916), and a process-visible scan is not proof of absence (the [proposal][proposal], lines 708 to 710).
- The snapshot branch of `zfs_domount` skips `zfsvfs_setup` and `zfsctl_create`
  (`module/os/linux/zfs/zfs_vfsops.c:1414` to `:1457`), so the probe's mount runs neither the callback
  registration, the ZIL open and replay and the unlinked-set drain, nor the control directory's creation. The
  dataset shape those cover is only exercised by a real dataset attach.
- The probe reports the flags it asked for and reads them back from `mountinfo`. It does not prove that a later
  account mount is refused the same flags.
- The key check is regression coverage only in the product: the probe reads `.zfs/snapshot` from a mounted
  checkout, so the checkout's key is loaded and no arm can produce the `EACCES` refusal. Item 5's unloaded-key
  procedure runs in the VM fixture, not at start ([M1](#m1-which-source-exercises-the-dataset-mount-path)).
- The probe's verdict is a property of the running kernel and module build. A livepatch, or a container runtime
  with a different namespace setup, can change the answer after the record was written. The ZFS module cannot be
  unloaded and reloaded on this host without a reboot, so a module change arrives with a reboot that runs the
  probe again ([The kernel the design names](#the-kernel-the-design-names)); the probe also runs again when the
  checkout provider is explicitly stopped or restarted, because `Requires=` on that provider stops or restarts
  this unit and every start re-runs its probe ([Who runs it, and where](#who-runs-it-and-where)). Nothing else
  re-runs it between boots.
- The probe proves the syscall surface on the build it ran on, not on every newer build than the stated syscall
  floor.
- A probe refusal names an arm and an errno. It cannot say whether one particular clone would attach, and it is
  not a substitute for the module gate's device and layout checks.
- The module gate the mode runs first may open `/dev/zfs` for the access check the accepted gate asks for (the
  [proposal][proposal], lines 235 to 237); the probe's own arms never do. A device that fails the gate's check is
  therefore a gate verdict, not an arm's, and the scope of "never opens `/dev/zfs`" is the arms.
- The probe reads no file content from the attached source; it proves the mount through `fstatfs`, a bounded
  directory read and mount facts. A source whose root directory is unreadable to root is therefore not detected
  by content.
- A probe that holds a snapshot for its duration can make a concurrent `zfs destroy` of that exact snapshot
  refuse `EBUSY`; [M2](#m2-the-mount-table-and-the-pool-are-unchanged) records whether that window is
  observable. The holder and the unmount child each carry a parent-death signal set after their own fork and the
  pipes their forking process owns, so an interruptible child does not outlive that thread unless it died between
  the fork and the call that sets the signal, which the child's `getppid()` recheck covers
  ([M3](#m3-interruption-leaves-nothing)). A child inside an uninterruptible kernel wait survives its `SIGKILL`
  until the wait returns, exactly as the worker does, so those bounds cover an interruptible child rather than
  every child. The residue is the worker: it sets no parent-death signal, so a mode killed alone leaves it
  running, and a `SIGKILL` cannot interrupt a kernel wait, so a worker inside one holds the snapshot until the wait
  returns.
- The snapshot name is account-influenced under [Way 1](#way-1-a-root-probe-on-a-live-snapshot): the account
  holds `snapshot` on the checkout (the [proposal][proposal], lines 319 to 321). The probe mounts the first name
  whose exclusive create succeeds, read-only, with `nosuid` and `nodev`, and reads no content from it.
  [Way 5](#way-5-a-setup-time-probe-dataset) removes the influence.
- Under [Way 1](#way-1-a-root-probe-on-a-live-snapshot) the probe reads `.zfs/snapshot` with `getdents`, which
  may update that control directory's access time. Whether such an update reaches the pool is unmeasured;
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged) records it, and
  [Way 5](#way-5-a-setup-time-probe-dataset) reads no directory at start.
- The freshness oracle needs Linux 6.6 or newer. `FSCONFIG_CMD_CREATE_EXCL` is refused with `EOPNOTSUPP` by an
  older kernel ([The kernel headers and source](#the-kernel-headers-and-source)), so on such a kernel the attach
  arm is unexercised and the probe never claims a fresh superblock. Where the command exists, it tells a fresh
  attach from a held superblock even when no mount of that superblock is visible in any readable namespace,
  because `sget_fc` matches on the objset rather than on a mount; the cost is that such a source is skipped
  rather than attached.

## Ways to meet the open item

The ways that remain each state who runs it and where, what it exercises, what it changes at start, what it
leaves behind, what a refusal prints, and its costs. Cost is stated as the work to write and to test, never as a
duration, because no measurement of this work exists.

[Way 2](#way-2-ruled-out-by-the-accepted-design) is not one of them: the accepted design rules it out, so it is
recorded and dropped rather than ranked.

### Way 1: a root probe on a live snapshot

It runs as root from the handler binary, as a new mode beside `boot-reconcile`, inside a fresh private mount
namespace it creates and destroys itself. It makes the namespace's mounts private, mounts a tmpfs for scratch,
walks to a slot in that tmpfs with the wrapper's `openat2` walk, attaches a read-only checkout snapshot by
descriptor (`fsopen` on the `zfs` type, `fsconfig` with the snapshot name and `nosuid,nodev`,
`FSCONFIG_CMD_CREATE_EXCL`, `fsmount` with `nosuid,nodev,rdonly`, `move_mount` onto the held slot descriptor),
reads a bounded directory listing and `fstatfs` there, moves the mount to a move-target directory in the same
tmpfs and unmounts it from a second process, then exercises the refusal walk and the namespace scan. It prints
one line naming the arm and errno on refusal, and the mode process writes the result into its own record.

What it changes at start: one private mount namespace, one tmpfs, one transient superblock for the snapshot, one
transient mount, a forked child per arm that needs a second process, a bounded read, one directory read of
`.zfs/snapshot`, and one root-owned record. It creates no dataset, snapshot, bookmark, property or file outside
the record's directory and the tmpfs inside it, and it makes no ZFS ioctl call.

What it leaves when it fails or is interrupted: nothing in the host mount table and nothing outside the record's
directory on `/run`, because the namespace and the tmpfs die with the process; a killed probe leaves either no
record or a started marker with the fingerprint, in that directory, and the record outlives a failed or stopped
unit, as [the record's own section](#where-the-verdict-is-enforced-and-where-the-record-lives) states.

For it: it exercises the kernel's real dataset mount, key path, descriptor placement, flag reporting, the move
and a second process's unmount, a refusal walk and a namespace scan on the running module and kernel, before
the first real attach, with no pool write and no clone. It calls the same kernel calls the product's attach
calls, so a kernel that cannot attach a snapshot cannot pass it.

Against it: it needs root, so the wrapper cannot run it per invocation; it cannot use the host namespace, so it
cannot prove the host's propagation; its source is a snapshot, so a clone's slot properties and encryption root
are not exercised; a snapshot another namespace already holds is refused by the exclusive create and has to be
skipped, and today's host has no checkout dataset at all; and the name it reads is account-influenced.

Build cost: a new handler mode, the namespace and scratch setup, the arms, a record and a `doctor` section, all
on `libc::syscall` with the existing walk and mount-fact helpers. Test cost: device-free tests for the arm
decisions, the record and the wording, and every live arm only in the disposable VM; the owed measurements under
[Owed measurements](#owed-measurements).

### Way 2 ruled out by the accepted design

The discarded alternative was a root probe that created a snapshot and clone on the live pool at every start,
attached the clone read-write, then destroyed both. It is recorded here only because a reader of the ranking
would otherwise ask why the clone shape is not measured at start.

It is ruled out, not merely ranked low. The wrapper owns "unprivileged snapshot and clone creation" and the unit
owns mount and unmount, "not Git or automatic deletion" ([decision][decision], lines 22 to 26); the handler "has
no Git, pool administration, or dataset destruction operation" (the [proposal][proposal], lines 160 to 161), and
"There are no pool management, dataset rename, or generic dataset creation requests" (lines 211 to 212). A boot
probe that created and destroyed a clone would need exactly such a request, a reserved parent, a sweep exclusion
and `/dev/zfs`, so it would contradict the accepted design and this proposal's own statement that the probe never
opens `/dev/zfs`. Its cost was also the highest: a boot-time write on the pool whose health is the question, a
stranded object after interruption, and `ENOSPC` at boot. No measurement is owed for it and no question about it
remains.

### Way 3: an account probe with no ZFS

It runs as the account, per invocation or on a first managed use, in a private user and mount namespace made the
way the wrapper's tests already make one (`unshare --user --map-root-user --mount`, or the same syscalls in
process). It mounts a tmpfs, walks to a slot in it, `fsmount`s the tmpfs, moves it onto the held descriptor,
moves it to the move target and unmounts it from a second process, exercises the refusal walk, and exercises a
holder in its own child namespace.

What it changes at start: nothing on disk if the scratch is a tmpfs, plus a per-user cached result. No root, no
`/dev/zfs`, no pool.

What it leaves when it fails or is interrupted: nothing but the cache entry, and nothing at all if the process is
killed before the entry is written.

For it: it is cheap enough to run per invocation, needs no root, catches a kernel or namespace regression before
the first root act, and reuses the wrapper's walk and mount-fact code.

Against it: it never touches `zfs_domount` or the key path, so a kernel whose ZFS mount is broken passes it; a
user-namespace mount differs from the init-namespace attach; and it is skipped exactly where user namespaces are
refused, which is a host whose confinement would matter most. It cannot stand alone as the answer's probe.

Build cost: one probe module, a cached record and a `doctor` line. Test cost: device-free tests plus the existing
user-namespace skip control.

### Way 4: the product's first attach as the probe

It runs no separate check. The handler's first real attach after boot is already the functional evidence, and its
outcome can be written into the `first-attach` file beside the probe's record; later managed attaches read that
record. Whether the file exists is [Q7](#q7-whether-the-first-real-attach-records-its-outcome).

What it changes at start: nothing extra. What it leaves when it fails: exactly what a failed first attach leaves,
a claimed and registered slot that needs the design's recovery.

For it: no separate check, the exact topology (a real clone, its slot, the host namespace, ZED ordering) and no
synthetic source.

Against it: it runs after the first attach, which is the attach the answer wants qualified, so it cannot refuse
it; it needs a clone to exist, so a fresh host never probes; and it reports an operation's failure as a
compatibility verdict, which mixes two different questions.

Build cost: a record block, one write in the product's attach path, a reader in `doctor` and a new measurement.
Test cost: one VM arm on the built handler, plus the existing add controls. The write is a duty plan steps 4 and
8 carry, since they build the attach, and no record decides it, which is why it is the owner's question rather
than this design's.

### Way 5: a setup-time probe dataset

It runs the same root probe as [Way 1](#way-1-a-root-probe-on-a-live-snapshot), from the same place, with one
difference: the source is a dataset the root-once setup creates, not a snapshot the probe finds. The setup
command that `doctor` prints (administrative commands printed by `doctor` do not run inside the wrapper, the
[proposal][proposal], line 153) creates the dataset and one empty snapshot, and records its name in the root-owned
configuration the handler reads, beside the pool and dataset names and GUIDs that configuration already pins (the
[proposal][proposal], line 319). The probe reads that name and checks it as a name, since it makes no ioctl;
`doctor` is where the GUIDs are compared.

The example this design names, for the owner to accept or replace at
[Q3](#q3-whether-a-setup-time-probe-dataset-exists), is the dataset `zpcachyos/nosnap/git-worktree-probe` with
the snapshot `ready`. It sits under the container `zpcachyos/nosnap`, which is unmounted (`mountpoint=none`,
`canmount=off`) and carries `autosnap = no` with each of its children (the [proposal][proposal], lines 1098 to
1124); those children are mounted caches, and the container is the one dataset there that nothing mounts. Its
properties are `mountpoint=legacy`, `canmount=off`, `com.sun:auto-snapshot=false` and the user property
`org.monochromatic:git-worktree-probe-source=v1`, which `doctor` reads through an ioctl to name the source; the
probe reads no property, because that is a raw request it does not make. The snapshot is empty.
Nothing mounts a legacy dataset automatically and `canmount=off` stops `zfs mount -a` and the mount generator too
(the [proposal][proposal], lines 262 to 270), so no automatic mount job reaches it and only a root process that
asks for it attaches it. Its key is the pool root's, which the boot unit already waits for before `ExecStart`; an
unavailable key is then an `EACCES` refusal from the attach arm, reported rather than hidden. No sanoid section is
needed to keep it out of snapshots: the only recursive section is the home template,
`zpcachyos/ROOT/cos/home`, and `zpcachyos/nosnap` is outside that subtree (the [proposal][proposal], lines 1081
to 1092). The section this design names in step 20 follows the configuration's existing convention of naming
every exclusion, so a later recursive ancestor cannot silently include it.

What the account can do to it: nothing. Delegation is scoped to the checkout dataset and the clone parent (the
[proposal][proposal], lines 319 to 327), and that same passage says not to put unrelated ZFS descendants there.
This dataset is under neither, so the account holds no `create`, `mount`, `destroy` or `userprop` on it and can
neither create, rename, mount, snapshot nor destroy it. That is the property
[Way 1](#way-1-a-root-probe-on-a-live-snapshot) cannot have, since its source name is account-influenced.

How its existence is detected: the probe asks the kernel, not the pool. It attaches `<dataset>@ready` with
`FSCONFIG_CMD_CREATE_EXCL`, and a missing dataset or snapshot is the `ENOENT` from `fsconfig(create)`, as the
OpenZFS facts state ([Facts](#openzfs-at-zfs-244)). `ENOENT` alone does not name the cause: `spa_open` returns
it for a pool that is not imported (`module/zfs/spa.c:6478` to `:6482`), so the record names the arm and the call
rather than the cause: an existing dataset whose attach answered `ENOENT` is a failure `doctor` reports as such,
after it checks that the configured dataset exists. A missing name is reported as an unexercised attach arm and
never created.

What it changes at start: the same list as [Way 1](#way-1-a-root-probe-on-a-live-snapshot) minus the directory
read of `.zfs/snapshot`, so no account-influenced name and no access-time question.

What it leaves when it fails or is interrupted: the same as [Way 1](#way-1-a-root-probe-on-a-live-snapshot). The
dataset itself outlives every run by design.

For it: the source always exists and no mount outside the probe holds its superblock, so the attach arm runs on a
fresh host, and the exclusive create refuses rather than silently reusing a superblock, which is the reuse hazard
[Way 1](#way-1-a-root-probe-on-a-live-snapshot) has to skip past. It is not account-influenced, needs no
directory read, and no account-run process and no automatic job pins or destroys it, so it cannot collide with
the space guard or the sweep.

Against it: it adds an object to the pool's layout, which is why it is the owner's question
([Q3](#q3-whether-a-setup-time-probe-dataset-exists)) and not this proposal's decision. It needs the setup
command, a sanoid exclusion section and the user property, and it is one more thing that can go missing or drift.
A further root process could still attach it, which arm 7's scan reports as a foreign mount rather than hides.
Making it a clone of the checkout snapshot instead of an independent dataset would exercise a real clone shape,
but a clone pins its origin snapshot until the clone is destroyed, so this design keeps it an independent dataset
and leaves the clone shape to [Way 4](#way-4-the-products-first-attach-as-the-probe).

What it changes in the plan, all of it named here rather than decided:

- Step 18, `doctor`: the root-once and setup commands it prints gain the dataset creation, its snapshot and the
  configuration field (the [plan][plan], lines 1490 to 1530). The probe result in the filesystem section is not
  this way's cost, because every answer needs it.
- Step 19, the migration and the units: the setup gains the commands (the [plan][plan], lines 1572 to 1593). The
  boot unit's `ExecStartPre=-` is not this way's cost either, because every answer needs the unit to run the
  probe.
- Step 20, sanoid coverage: the dataset gets its own `autosnap = no` section beside the named exclusions, with
  the `com.sun:auto-snapshot=false` marker, and the guard's report names it (the [plan][plan], lines 1677 to
  1700).
- No step of the sweep, the destroy-anyway path or preflight changes. The dataset is outside the pinned clone
  parent whose immediate children those steps enumerate (the [proposal][proposal], lines 540 to 541), and no
  account grant reaches it, so none of them can meet it. A draft of this design assumed a place under that parent,
  which would have changed every one of those steps.

Build cost: [Way 1](#way-1-a-root-probe-on-a-live-snapshot) plus the setup command's dataset creation, the
configuration field, the exclusion and the missing-dataset report. Test cost: the same as
[Way 1](#way-1-a-root-probe-on-a-live-snapshot), plus a fixture that starts without the dataset and one that
starts without its snapshot.

## Ranking

Ranking: Way 5 > Way 1 > Way 3 > Way 4. [Way 2](#way-2-ruled-out-by-the-accepted-design) is not ranked, being
ruled out by the accepted design.

Way 5 ranks first because it is [Way 1](#way-1-a-root-probe-on-a-live-snapshot) with a source that always exists
and that nothing else mounts, so every arm runs on every boot, the exclusive create proves the superblock is new,
and no account-influenced name is read. It ranks above Way 1 because of that, and the layout change it needs is
the owner's to accept at [Q3](#q3-whether-a-setup-time-probe-dataset-exists): with a yes the recommendation's
source is Way 5, and without that answer this design uses Way 1.

Way 1 ranks second because it reaches `zfs_domount` and the key path at start on a host whose checkout already
has an unheld snapshot, without a layout change, and it refuses before the first managed attach. It ranks above
Way 3 because Way 3 never reaches the dataset mount path, which is the failure the answer exists to catch.

Way 3 ranks third because it catches a syscall or namespace regression before any root act and can run per
invocation, but it cannot refuse a bad ZFS attach, and it is skipped exactly where confinement matters. It ranks
above Way 4 because Way 4 cannot refuse anything: its evidence arrives after the attach it would have qualified,
and it never runs on a host without clones.

Way 4 ranks fourth. Its record is the owner's to accept at
[Q7](#q7-whether-the-first-real-attach-records-its-outcome), because it adds a write to the product's attach path,
a second file, a reader in `doctor` and a new measurement, and no record decides that write. This design therefore
does not call it free and does not settle it. It is the only evidence about a real clone in the host namespace
that any way gives.

Way 1 and Way 3 are complements, not alternatives: Way 3's per-invocation check is
[Q6](#q6-whether-the-account-runs-a-companion-probe), and [the ranking](#ranking) orders the ways that could serve
as the start check's own evidence.

The recommended shape is the probe's arms under [The proposed probe](#the-proposed-probe), sourced from
[Way 5](#way-5-a-setup-time-probe-dataset) if the owner answers [Q3](#q3-whether-a-setup-time-probe-dataset-exists)
yes and from [Way 1](#way-1-a-root-probe-on-a-live-snapshot) if not, with
[Way 4](#way-4-the-products-first-attach-as-the-probe)'s record only if the owner answers
[Q7](#q7-whether-the-first-real-attach-records-its-outcome) yes, and
[Way 3](#way-3-an-account-probe-with-no-zfs) only if the owner wants a per-invocation companion.

## The proposed probe

This section proposes the recommended shape in the detail the open item asks for. The arms are the same whichever
answer [Q3](#q3-whether-a-setup-time-probe-dataset-exists) gives; only the source differs. Every part of it is a
writer's proposal inside an undesigned item, and the questions under
[Questions for the owner](#questions-for-the-owner) can change parts of it.

### The arms

1.  Namespace and scratch. The mode process forks a worker and calls `setpgid(worker, worker)` at once, so the
    group exists even if the worker dies before its own call, and the worker calls `setpgid(0, 0)` as well. The
    worker then calls `unshare(CLONE_NEWNS)`. It makes every mount in the new namespace private
    (`mount_setattr` with `AT_RECURSIVE` and `attr.propagation = MS_PRIVATE`, or
    `mount(NULL, "/", NULL, MS_REC | MS_PRIVATE, NULL)`), and mounts a fresh tmpfs on the scratch mount point the
    mode created. It then creates the slot, the move-target directory and a refusal tree inside the tmpfs. Until
    the `unshare` returns, no mount exists that could propagate, so a `SIGKILL` before it leaves nothing.
2.  Walk and refusal controls, over trees the probe owns. The worker opens the tmpfs root, walks to the slot with
    the wrapper's no-crossing `openat2` flags, and records the slot's `ObjectId`. The refusal controls run in the
    refusal tree: a symbolic link at a component must refuse `ELOOP`, a `..` escape must refuse `EXDEV`, a bind
    mount planted across a component must refuse `EXDEV`, and a recorded identity that changed must refuse. Each
    refusal must name the component or field, the way item 2's refusals did ([measurements][measurements], lines
    80 to 92). Separately, and read-only, it opens the checkout root through the wrapper's `open_root`, which is a
    plain `open` with a canonical-path check rather than the `openat2` mask, and records its `ObjectId`: that
    proves the root itself opens and reads no entry there.
3.  Source choice. Under [Way 5](#way-5-a-setup-time-probe-dataset) the source is the dataset and snapshot the
    root-owned configuration names, and a missing name is the `ENOENT` arm 4 reports. Under
    [Way 1](#way-1-a-root-probe-on-a-live-snapshot) the worker reads `.zfs/snapshot` with `getdents` only, never a
    lookup by name, and takes the names in the order the read returns them. It accepts a name only if it is a
    valid ZFS component name and the assembled `<checkout-dataset>@<name>` fits the dataset-name bound. It tries
    the first accepted name and, when arm 4 answers `EBUSY`, records that name as held and tries the next. If the
    directory is unreadable or every candidate is held, the attach arm and every arm after it are recorded not
    exercised, with the reason.
4.  Attach, with the freshness oracle. `fsopen` on the `zfs` filesystem type, `fsconfig` the source name,
    `fsconfig` the `nosuid` and `nodev` options, read-only through the generic `ro` flag, then
    `FSCONFIG_CMD_CREATE_EXCL`, then `fsmount` with `MOUNT_ATTR_NOSUID`, `MOUNT_ATTR_NODEV` and
    `MOUNT_ATTR_RDONLY`, and `move_mount` from the mount descriptor onto the held slot descriptor. The exclusive
    create is what proves the superblock is new: a matched extant superblock answers `EBUSY` before `zfs_domount`
    runs, whether or not any mount of it is visible. A kernel older than 6.6 refuses the command with
    `EOPNOTSUPP`, which records the attach arm as not exercised and stops the probe there rather than falling
    back to a plain create that could reuse a superblock.
5.  Verify. Read `/proc/self/mountinfo`, require the expected mount ID, filesystem `zfs`, exact source, and
    `nosuid,nodev,ro` flags; `fstatfs` the mount and read a bounded number of directory entries; no file content
    is read. The arm reports the flags it asked for; it does not prove that a later mount is refused them.
6.  A holder for the namespace arm. The mode process forks the holder, not the worker, so the holder inherits no
    descriptor on the worker's mount and no mount it did not make. It unshares its own mount namespace, makes it
    private, and attaches the same source read-only at the directory `holder`, which the mode created beside
    `scratch`: its mount lives in its own namespace, and the worker's tmpfs is not visible to it. It uses the plain
    `FSCONFIG_CMD_CREATE`, because reusing the worker's superblock is what this arm wants: it needs a mount in a
    second namespace, not a second mount record. It starts after arm 5 and before the move, because a probe that
    has already unmounted has no mount for a holder to hold, and it waits on its control pipe for the mode's
    signal. The mode stops and reaps it between arm 7's two scan passes, and reads the outcome line it wrote to
    its status pipe. Its pipes and its parent-death signal are the protocol described after the arms.
7.  Namespace scan, in two passes, before anything is unmounted. Run the probe's own scan: group live tasks by
    `/proc/<pid>/ns/mnt`, read a representative `mountinfo` per namespace, and match the ZFS filesystem and exact
    source. In the first pass, while the worker's mount and the holder's both exist, require the set of
    namespaces that mention the source to be exactly the worker's and the holder's: the exclusive create proved
    no superblock existed when the probe attached, so a further namespace mentioning it is either a mount the
    probe leaked or a mount another root process made during the run, and both are recorded as a failure of this
    arm rather than hidden. In the second pass, after the mode stops the holder and reaps it, require the worker's
    namespace to be the only one that mentions the source. A process-visible scan is not proof of absence, and a
    pinned namespace with no live member stays invisible (the [proposal][proposal], lines 708 to 710), so a clear
    report is not absence. This scan is probe-private; step 9 builds the product's (the [plan][plan], lines 909 to
    916), and this design proposes that step 9 reuse this grouping and source match rather than write a second
    scanner of the same mechanism.
8.  The move target and the unmount, from a second process. Before it forks, the worker closes the mount
    descriptor `fsmount` returned and every descriptor whose path is on the attached mount, including the ones the
    verify arm opened there. Each of them holds a reference on that mount, and a non-lazy unmount refuses while the
    mount's count exceeds the one its parent holds plus the one the unmounting path walk takes
    (`fs/namespace.c:2010` calls `propagate_mount_busy(mnt, 2)`; the count test is `fs/pnode.c:371` to `:374`,
    reached from `:501` to `:513`; the `fsmount` file holds such a reference because `dentry_open` puts the path in
    it, `fs/namespace.c:4705`, and marks it `FMODE_NEED_UNMOUNT`, `:4715`). A child's close cannot do that work for
    the worker: `fork` copies descriptors, and the open file description lives while any copy does. The worker then
    forks a child with its own control pipe and status pipe, and the child rechecks `getppid()`, resolves the slot
    and the move-target directories inside the tmpfs by path, and moves the mount with `move_mount` between those
    two paths. `move_mount` looks both paths up with `filename_lookup` and drops those references when it returns
    (`fs/namespace.c:4747`, `:4789` to `:4812`), so the child holds nothing when it unmounts. It verifies the mount
    point from `/proc/self/mountinfo`, which is not on the mounted tree, and unmounts by path with `umount2`,
    which must not answer `EBUSY`: an `EBUSY` there fails this arm. The worker waits for the child,
    bounded by the mode's deadline, and then runs the scan once more: no namespace may mention the source. It never
    unmounts a path in the account's tree, which is the only tree the accepted rule forbids (the [plan][plan], line
    275). This is probe-private code: it proves that a second process can move and unmount what another process
    attached, and it does not exercise the product's unmount from a held descriptor, which step 9 owns.
9.  Teardown. The worker writes its last outcome line, closes its status pipe and exits; the death of the mount
    namespace removes the tmpfs and everything mounted inside it, so the probe unmounts no path of its own. The
    holder is already gone, because the mode stopped it between the scan's passes. The mode process writes the
    record only after the worker exits or its deadline passes.

If an arm fails, the worker stops at that arm, records its name and errno, tears the namespace down, and reports.
A name held by another namespace is not a failure: arm 3 tries the next one. The refusal controls in the walk arm
are expected refusals: an arm that attaches instead of refusing fails the probe, so the harness has a positive
control for its own check.

The mode, the worker and each child speak over pipes that belong to one child each, so no end of file can arrive
from another process's copy and no child inherits a channel meant for a sibling. Every pipe is created
immediately before the fork that needs it, and the forking process keeps one end and closes the others as soon as
the fork returns: the mode keeps one request pipe and one reply pipe to the worker, and one control pipe and one
status pipe to the holder, and the worker keeps one control pipe and one status pipe to the unmount child. The
mode cannot own that child's ends, since the child is its worker's, so the worker relays its outcome line to the
mode and the mode's bound on that child is the process group's `SIGKILL`. The holder and the unmount child each
move their control read end to descriptor 3 and their status write end to descriptor 4, then close every
descriptor numbered 5 or higher with `close_range(5, ~0U, 0)`, so those two pipes survive the sweep that clears
everything else. A child sets `PR_SET_PDEATHSIG` to `SIGKILL` and then rechecks `getppid()`, because the kernel
sends the signal from `forget_original_parent`, which runs for the thread that forked the child rather than for
the process (`kernel/exit.c:729` to `:732`, `PR_SET_PDEATHSIG(2const)`, CAVEATS), and because a forking thread
that died before the call leaves `getppid()` as the only signal (`kernel/sys.c:2552` to `:2561`). The mode is
single-threaded and forks from the thread that lives until it exits, so every child's parent-death signal has a
live thread behind it.

Requests are single bytes. The worker asks for the holder to start before the move and to stop between the scan's
passes; the mode forks, stops and reaps the holder on those requests and writes the outcome back on its reply
pipe. The mode does no work between reading a request and answering it, so neither process can wait on the other
for a pipe the other is waiting to fill. The holder and the worker write one line per arm, naming the arm and its
errno, to their own status pipes, which the mode reads until end of file; the unmount child writes to the pipe its
worker owns and the worker relays that line. The mode therefore writes every field of the record from values it
read itself. The worker sets no parent-death signal of its own, because it must finish its unmount and report even
when the mode dies; [M3](#m3-interruption-leaves-nothing) records that kill, and [Limits](#limits) names the
residue.

### Who runs it, and where

Root, from the handler binary, inside the private mount namespace the run creates. The probe never runs in the
host mount namespace, never runs as the account, and never runs inside `doctor`.

Placement: a new mode of the handler binary, `start-probe`, beside `boot-reconcile`, `shutdown-unmount` and
`adopt-worktree-parent` (the [plan][plan], lines 640 to 641 list those; step 6 adds `start-probe` to the list),
run as `ExecStartPre=-` of the existing boot unit. The dash prefix is load-bearing: without it a probe that exits
non-zero, panics or dies of a signal makes the unit fail before `ExecStart`, so the reconcile is skipped and the
unit's stop action runs at boot, which is the outcome the placement exists to avoid. With the prefix such an exit
is recorded and has no further effect, and `ExecStart` still runs (`systemd.service(5)`, the special executable
prefixes). The prefix covers a non-zero or signal exit and not a timeout, which is one reason the unit sets none:
an expiry in `ExecStartPre=` sends the unit to its stop action whatever the prefix says (`systemd.service(5)`,
`ExecStartPre=`; systemd v262 `service.c:3047` to `:3057` spawns that command with the start timeout, and `:5156`
to `:5166` turns an expiry there into the stop path).
`reconcile-git-worktree-clone-mounts.service` already runs as root at boot, already orders after the key and
checkout providers, and already publishes readiness (the [proposal][proposal], lines 492 to 505). `ExecStartPre=`
is the service manager's own child process, so the `unshare` inside the probe cannot change the namespace the
unit's `ExecStart` reconciles in, which the accepted design requires to be the host namespace (the
[proposal][proposal], line 496). No new unit and no new request channel are added, which matters because the
design forbids a second mount request channel and fixes the intent vocabulary to `mount` and `unmount` (the
[proposal][proposal], lines 478 to 480 and 537 to 539). An on-demand probe would need a third intent kind or a
new channel, and this proposal does not propose one.

The mode runs the module gate first, in the same process, before it makes any mount. A gate refusal records the
probe as skipped with the gate's own reason and enables nothing, because the gate's refusal "does not authorize
an unrelated mount" (the [proposal][proposal], lines 243 to 244); a skipped probe is not a probe verdict either
way.

The mode process enforces the deadline rather than the worker: it forks the worker, puts it in a process group of
its own, waits for it with a monotonic deadline, and on expiry sends `SIGKILL` to that whole group and to the
holder, reaps the worker only with `WNOHANG`, records a timed-out result, writes the record and exits without
waiting further. It exits successfully whenever it reaches its own end, because the unit's start must continue: a
failed or timed-out probe is a verdict in the record, never a failed unit. A mode that panics, aborts, takes a
signal or is killed for memory never reaches that end, and the dash prefix is what keeps the unit's start going
then; what the next boot reads in that case is the started record that
[the interruption section](#what-it-leaves-when-it-fails-or-is-interrupted) describes. The worker's mounts die
with its namespace, so a worker that survives an uninterruptible wait, which `SIGKILL` cannot interrupt, holds
nothing in the host mount table; it can hold the snapshot busy until the wait returns, which is under
[Limits](#limits). The holder and the unmount child each carry a parent-death signal and the pipes their forking
process owns, described after the arms, so an interruptible child does not outlive the thread that forked it, and
a child inside an uninterruptible kernel wait can outlive it until that wait returns; the unit's own `KillMode=`
default of `control-group` sends nothing until the unit stops, so it bounds a leaked process only after a failed
or finished run (`systemd.kill(5)`, `KillMode=`). What a missed deadline does to the route is
[Q5](#q5-whether-a-missed-deadline-fails-closed); the deadline's value comes from
[M4](#m4-what-the-probe-costs).

The unit sets no `TimeoutStartSec=`. A start timeout bounds a oneshot's `ExecStart` as well as its
`ExecStartPre=`: the timer is armed per command with `s->timeout_start_usec` (systemd v262 `service.c:2991`,
armed at `:2178`), and a command that times out sends the unit to its stop action, which for this unit is
`shutdown-unmount`. The reconcile's duration is unsized until plan step 22, so any value that fits the probe could
kill the reconcile mid-attach and then unmount what it had attached. The oneshot default is no start timeout at
all (systemd v262 `service.c:1225` to `:1227`; `systemd.service(5)`, `TimeoutStartSec=`), the probe's own deadline
bounds the worker and the holder, and the mode's own phases are short and non-blocking rather than timed. No step
sizes a unit start timeout: plan step 22 sizes the mount-request deadline of step 7 and the unmount deadline of
step 9, and asks for workloads to size any shipped deadline (the [plan][plan], lines 1778 to 1779 and 1811 to
1812). A mode wedged in one of its own phases would therefore hold up `systemd-user-sessions.service` until it was
killed by hand, which [M4](#m4-what-the-probe-costs) reports beside the login delay it measures. The accepted unit
is ordered before `systemd-user-sessions.service` (the [proposal][proposal], lines 496 to 498), so the probe delays
a login by its own run time; [M4](#m4-what-the-probe-costs) measures that delay and [Limits](#limits) records that
the probe does not run in the host namespace.

Nothing re-runs the probe between boots except a restart of the checkout provider: the unit requires that
provider and orders after it (the [proposal][proposal], lines 483 to 486), and `Requires=` stops or restarts the
requiring unit when the required one is explicitly stopped or restarted (`systemd.unit(5)`, `Requires=`, systemd
262). Every start of this unit runs its `ExecStartPre=` again, so the probe and its record are refreshed then. A
module reload cannot happen here without a reboot, so a module change arrives with a reboot that runs the probe
again ([The kernel the design names](#the-kernel-the-design-names)). A record whose fingerprint no longer matches
the running kernel, module or boot is stale, and the wrapper reads stale readiness as not current rather than
running a probe.

The wrapper's own per-invocation start check stays the module gate. The wrapper reads the probe's result from the
record it already must read before an add (the [proposal][proposal], lines 351 to 352) and reports it.

### Where the verdict is enforced, and where the record lives

The record is `/run/git-worktree-clone-mount-probe/result`: a root-owned file, mode `0644`, in a root-owned
directory of mode `0755` that the mode process creates itself if it is absent, the same shape as the receipts
directory `/run/git-worktree-clone-mount-results` (the [proposal][proposal], lines 550 to 553). The mode creates
that directory rather than relying on the unit's `RuntimeDirectory=`, because systemd removes a runtime directory
when the unit reaches its dead state, failed included (systemd v262 `service.c:2591` to `:2599`, `unit.c:6452` to
`:6459`; `systemd.exec(5)`, `RuntimeDirectory=`), and that would delete the record exactly when a failed arm's
verdict is what `doctor` and the add preflight must read. Because the directory is on `/run`, the record is
per-boot state that no disk holds. The mode also creates `scratch` and `holder` inside that directory, as the
mount point for the worker's tmpfs and the directory the holder attaches at; both mounts live only in a child
namespace, so the host sees empty directories there after a run and the record file beside them.

Step 6 builds the mode and its writer. The unit that carries the `Type=oneshot`, `RemainAfterExit=yes` and
`ExecStart` running `boot-reconcile` is step 19's (the [plan][plan], lines 1591 to 1593), and it gains
`ExecStartPre=-` running the probe; step 7's readiness record references the probe's record rather than owning it
(the [plan][plan], lines 762 to 779 and 825 to 829). Until step 19 lands, the probe runs from the guest's
transient test unit named under [M9](#m9-the-untested-release-and-the-refused-release), so the design does not
wait on the unit.

Every process that makes a raw ZFS request reads it before doing so: the wrapper's add preflight, where the
proposal already requires a readiness report (the [proposal][proposal], lines 351 to 352), the handler's
`boot-reconcile` mode, its `shutdown-unmount` mode, and the sweep the no-argument invocation runs, which a ZED
event starts (the [plan][plan], lines 640 to 647). The wrapper's other invocations make no raw request and read
nothing. The sweep and the boot unit are independent triggers (the [proposal][proposal], lines 478 to 480 name
both), so a sweep can reach the record before the unit's probe has written it; an absent or stale record then
refuses the attach and keeps the intent, which is the decided behaviour for a missing readiness report (the
[proposal][proposal], lines 351 to 352) and needs no new rule. The module gate binds both binaries (the
[proposal][proposal], lines 235 to 237); the probe's verdict binds at least every attach path, and what else a
failure disables is [Q2](#q2-what-a-failed-arm-disables).

### What it changes at start

- One worker process, the mode process that forks it and enforces its deadline, a holder the mode forks, and an
  unmount child the worker forks, each short-lived, with one control pipe and one status pipe per child.
- One new private mount namespace for the worker and one for the holder, made private before any mount is made.
- One tmpfs for scratch, no larger than the probe needs to hold a few directories and a link.
- One mount of a read-only source, and the superblock it creates, for the duration of the run.
- One bounded directory read on the probe's own read-only mount and one `fstatfs`.
- Under [Way 1](#way-1-a-root-probe-on-a-live-snapshot) only, one `getdents` read of `.zfs/snapshot`.
- One root-owned directory under `/run` and its record, rewritten by atomic rename.
- Journal lines from the boot unit.

It creates no dataset, snapshot, bookmark or property, makes no ZFS ioctl request, starts no service, and writes
no file outside the record's directory and the tmpfs inside it.

### What it leaves when it fails or is interrupted

A failed arm tears the namespace down and leaves the record naming the arm, the errno, the fingerprint and the
fallback. The record's directory is not a runtime directory, as
[the record's own section](#where-the-verdict-is-enforced-and-where-the-record-lives) states, so the record
survives the unit's failure and a manual stop. A `SIGKILL` leaves nothing in the host mount table and nothing
outside the record's directory, which [Assumptions](#assumptions) states and
[M3](#m3-interruption-leaves-nothing) measures, because the namespace and the tmpfs die with the process. If the
record is absent, the wrapper's add preflight already treats the readiness as not current and refuses the managed
attach, which is the decided behaviour for a missing readiness report (the [proposal][proposal], lines 351 to
352), not a new rule.

The one residue a killed probe can leave is a started record: the mode process writes one, with the fingerprint
and no result, by atomic rename before it forks the worker, and the final record replaces it the same way. Both
live in the record's directory on `/run`, so neither is on disk, and `doctor` can say the probe began and did not
finish. A reader that finds a started record with no result reads it as not current rather than as a pass, which is
the same rule as an absent record, so a mode that panicked or was killed leaves a boot whose adds refuse the
managed attach. The next boot's probe replaces it, and the next boot starts with an empty directory.

### What a refusal prints

The wrapper prints one line to standard error on the managed operation it refuses, naming the probe arm, the
running kernel release, the module version, the failed call and its errno, the route the slot will use instead,
and the `doctor` command that prints the last result. It never suggests a force flag, because the gate has none
and this probe adds none. Which operations a failed arm refuses is [Q2](#q2-what-a-failed-arm-disables).

`doctor` prints a new section under the existing filesystem report, headed by its section marker pattern, with
the fingerprint, when the last probe ran, each arm and its outcome, any difference between the recorded
fingerprint and the live kernel, module or boot, the sources and mounts the probe used, and which route remains
available. The section goes to standard error and changes no exit status, as the design requires for setup
findings (the [proposal][proposal], lines 1153 to 1155).

The handler's journal line names the same arm and errno under `reconcile-git-worktree-clone-mounts.service`, so a
boot failure is visible without a caller.

### The result record and its fingerprint

The record holds the fingerprint: kernel release from `uname`, module version from `/sys/module/zfs/version`,
the module's `srcversion`, the boot ID from `/proc/sys/kernel/random/boot_id`, and the handler binary's identity
in the shape `binary_identity` records. A module's build ID where the kernel exports one, under
`/sys/module/zfs/notes/`, joins the fingerprint too; that directory is empty on this host, so `srcversion` is the
build input that is always readable there. The record also holds the time of the run, each arm's outcome, errno
and source, the holder's and the worker's namespace identities, the chosen source, whether the source existed, and
the resulting verdict. The probe's own fields are written only by the mode process, from the lines it reads on the
status pipes. The first real attach's outcome, which
[Way 4](#way-4-the-products-first-attach-as-the-probe) describes, goes into a second file beside it,
`first-attach`, only if the owner answers
[Q7](#q7-whether-the-first-real-attach-records-its-outcome) yes; the handler process that made that attach writes
it, under the same atomic rename, and each file then has one writer, so no lock is needed. The wrapper and `doctor`
read both and write neither.

The record is not authority to act. It is a cache of an observation, and the wrapper rebuilds nothing from it
except the verdict and its wording, the way the design treats its other runtime files (the [proposal][proposal],
lines 555 to 556).

### What the result gates

A result with every arm passed enables managed attaches for that boot and module identity.

A result naming a failed attach, walk, move or unmount arm, or a failed namespace arm, is a real observation.
What it disables is the owner's answer to [Q2](#q2-what-a-failed-arm-disables); the options there state the effect
on unused slots, on boot reconcile, and on removal of mounted and unmounted clones.

A result naming an unexercised arm, because no source existed, every candidate was held, or the kernel refused the
exclusive create, is reported. Whether it enables the route is the owner's answer to
[Q4](#q4-whether-an-unexercised-arm-may-enable-the-route).

A result naming a missed deadline is written as timed out. Whether the route then refuses or continues is the
owner's answer to [Q5](#q5-whether-a-missed-deadline-fails-closed).

A module or kernel whose fingerprint changed since the record makes the record stale; the next boot's probe
replaces it, and an add in between reads stale readiness, which the preflight must report as not current rather
than as a pass.

### What the probe never does

Its arms never open `/dev/zfs`, because the dataset mount path in the kernel makes no ZFS ioctl request:
`zpl_get_tree` holds the objset and reaches `zfs_domount`, which calls `zfsvfs_create`
(`module/os/linux/zfs/zpl_super.c:850` to `:920`, `module/os/linux/zfs/zfs_vfsops.c:1347` to `:1375`). The module
gate the mode runs first may open it, because the accepted gate checks device access (the [proposal][proposal],
lines 235 to 237) and step 6 designs that check; `stat`, `access(2)` and a plain `open` all answer "missing
device" without a raw request, so this design does not choose between them. Beyond that, it never mounts by path on
an account-controlled tree, never runs the `zfs` program, never creates or destroys a dataset or snapshot, never
promotes, never sets a property, never writes a registration or a receipt, never runs in the host mount namespace,
never runs from `doctor`, and never offers a force flag. It never looks up a name inside `.zfs/snapshot`: opening
that directory resolves the two components `.zfs` and `snapshot` by name, and beyond them the probe reads the
directory's entries with `getdents` and never opens, stats or resolves an entry it finds, because such a lookup
starts the automounter and can mount into a namespace the probe does not control
(`module/os/linux/zfs/zpl_ctldir.c:255` to `:277`, `module/os/linux/zfs/zfs_ctldir.c:1190` to `:1198`). The mounts
it makes live only in its own namespace.

## Owed measurements

Every item here is owed before the probe can be trusted. None ran for this proposal, and each runs on a
disposable file-backed pool inside a throwaway VM, never on this host's pool and never with its `/dev/zfs`.

The recommended shape needs M1, M2, M3, M4, M5, M7, M8 and M9, plus M10 where the owner answers
[Q3](#q3-whether-a-setup-time-probe-dataset-exists) yes. [Way 3](#way-3-an-account-probe-with-no-zfs) needs M2,
M4, M5, M6 and M9. [Way 4](#way-4-the-products-first-attach-as-the-probe) needs M8 and M9.

### M1: which source exercises the dataset mount path

Attach a snapshot that no namespace mounts, and the already mounted checkout dataset beside it, and record which
one runs `zfs_domount` and the key path. The oracle for a fresh superblock is `FSCONFIG_CMD_CREATE_EXCL`: the
fresh source must attach, and the already mounted one must answer `EBUSY` from `fsconfig(create)` rather than
succeed while reusing a superblock. A successful attach is no evidence either way, because with the checkout
mounted its key is loaded. Control: item 5's unloaded-key procedure, which unmounts the checkout and home first
([measurements][measurements], lines 221 to 222), unloads the key, and attempts the fixture's snapshot source,
whose name the fixture supplies because `.zfs/snapshot` is unreadable while the checkout is unmounted; require
`EACCES` from `fsconfig(create)` (item 5, line 231). Also record whether a read-only request against a read-write
dataset superblock refuses `EBUSY` (`module/os/linux/zfs/zpl_super.c:936` to `:943`), adjust the arm's flags if it
does, and record what a kernel older than 6.6 returns for the exclusive create, so the unexercised arm's wording
is measured rather than assumed.

Gates: [Way 1](#way-1-a-root-probe-on-a-live-snapshot) and
[Way 5](#way-5-a-setup-time-probe-dataset), the exclusive create as the freshness oracle, the claim that a source
no namespace mounts is the one that reaches the mount path, and the limit that the key refusal is regression
coverage only.

### M2: the mount table and the pool are unchanged

Read the guest's initial mount namespace's `/proc/self/mountinfo`, and a representative `mountinfo` for every
other visible namespace, before and after a complete probe. Read `/proc/spl/kstat/zfs/<pool>/txgs` before and
after, plus the dataset list, the checkout's `atime` property and the pool history as secondary readings. The txg
kstat is the one that can show a write: pool history records administrative commands only, since the mount path
has no history call (`module/os/linux/zfs/zfs_vfsops.c:1911` to `:1964`) and the kstat is installed at
`module/zfs/spa_stats.c:288` to `:304`. Its columns are `txg`, `birth`, `state`, `ndirty`, `nread`, `nwritten`,
`reads` and `writes`, followed by the completion times (`module/zfs/spa_stats.c:206` to `:214`).

The txg comparison is by row, not by the highest txg: an idle pool commits a txg on its own timer, every
`zfs_txg_timeout` seconds (5 by default, `module/zfs/txg.c:532` to `:553`), so the highest txg always moves and a
before-and-after comparison of counters proves nothing. The window carries a start fence and an end fence, both a
sync. Run `zpool sync` in the guest, then read the kstat and call the then-open row's txg `T0`: the sync committed
everything written before it, so `T0` starts clean and the probe's writes can land only in `T0` or later. Run the
probe, run `zpool sync` again, and read the kstat again: that sync commits everything the probe wrote, so the
highest committed row is the window's end, and a late write cannot hide in the row that is still open. Require,
for every committed row whose `txg` is no lower than `T0`, `ndirty`, `nwritten` and `writes` to be zero, and
require such a row to exist: the end sync commits `T0` itself, so that requirement is met unless the history list
has already dropped the row, which the `zfs_txg_history` bound turns into an unmeasured report rather than a pass.

An idle txg writes nothing: with nothing changed in it, an empty config dirty list and no multihost protection,
the uberblock write is skipped (`module/zfs/vdev_label.c:2064` to `:2079`), so a committed row with zero `ndirty`,
`nwritten` and `writes` is the expected reading for a probe that wrote nothing. The measurement therefore needs a
pool the harness keeps otherwise idle, because a write from anything else lands in the same window and cannot be
told from the probe's; the harness records the window's rows with their counters so a nonzero row can be
attributed. Read the second time soon enough that `T0` is still in the list, because the list keeps
`zfs_txg_history` rows, 100 on this host (read-only `/sys/module/zfs/parameters/zfs_txg_history`); with the
parameter at 0 the reading is empty, which the design reports as unmeasured rather than as zero writes.

Controls, each fixture-only and never shipped: the idle window, which runs the same syncs and readings with
`sleep` standing in for the probe and must read zero on every row of the window, so a harness that reports zero
for a pool something else is writing fails its own control; a deliberate mount in the guest's initial namespace,
which must show the mount-table difference the harness looks for; a deliberate write in a scratch dataset, which
must leave a nonzero committed row in the window, including one made inside `T0` itself, so a harness that never
reads the file, or that only looks past the window, fails its own control; and a deliberate `open` of an entry name
inside `.zfs/snapshot`, which must change the guest's initial mount table, because the automounter mounts through a
helper (`module/os/linux/zfs/zfs_ctldir.c:1190` to `:1198`). The call is an `open`: `stat`, `lstat` and `fstatat`
do not automount (`statx(2)`).

Gates: [Way 1](#way-1-a-root-probe-on-a-live-snapshot), [Way 3](#way-3-an-account-probe-with-no-zfs) and
[Way 5](#way-5-a-setup-time-probe-dataset); also the claim that a probe leaves no pool write, the access-time
question on `.zfs/snapshot`, the snapshot-hold window, and the prohibition on lookups inside `.zfs/snapshot`.

### M3: interruption leaves nothing

Kill the probe with `SIGKILL` after each step: after `unshare`, after the tmpfs, after `fsconfig(create)`, after
`fsmount`, after `move_mount`, after the holder starts, between the scan's two passes, after the move, and before
the unmount. Kill each process separately as well as the whole run: the mode process alone (the worker must then
survive or die cleanly, and the record must say what happened), the worker alone while the mode lives, the holder
alone, and the unmount child alone. Record the host mount table, every namespace's representative `mountinfo`,
the slot, the move-target directory, the record and its directory, whether any namespace still mentions the
source after the kill, and whether a second run succeeds. Control: the same probe allowed to finish, and a run
killed before `unshare` returns.

Gates: the claims that it leaves nothing in the host mount table and nothing outside the record's directory, the
started record, the parent's deadline path, and each child's parent-death signal, `getppid()` recheck and pipes.

### M4: what the probe costs

Measure each arm and the whole probe as a ratio to a control arm of the same binary with the probe disabled,
interleaved in the same round, and report the range between rounds. Measure at boot, with the boot unit, the
delay it adds to the first login that `systemd-user-sessions.service` admits, and measure any other placement an
owner answer later asks for. Control: items 4 and 15's timings are not a control, because they measure other
work.

Gates: [Way 1](#way-1-a-root-probe-on-a-live-snapshot), [Way 3](#way-3-an-account-probe-with-no-zfs) and
[Way 5](#way-5-a-setup-time-probe-dataset) placement, the probe's own deadline, and the time-to-login cost of a
boot ordered before user sessions. No value is stated before this measurement.

### M5: the refusal arm refuses for the right reason

Run every refusal control the walk arm lists and the success control, and require the success case to attach
where the refusal cases refuse. Control: the unmodified slot must attach, so a harness that refuses everything
fails its own control.

Gates: [Way 1](#way-1-a-root-probe-on-a-live-snapshot) and [Way 3](#way-3-an-account-probe-with-no-zfs).

### M6: user namespace limits

As the account, attempt the probe's attach of a ZFS source that no namespace mounts, inside a user namespace, and
require `EPERM` from `zfs_domount`. Then repeat with the source mounted in the guest's initial namespace and
require `EBUSY` from the matched extant superblock instead, because `sget_fc` refuses a superblock from another
user namespace before `zfs_domount` runs (Linux v6.8 `fs/super.c:779` to `:786`). Control: a tmpfs mount in the
same user namespace succeeds, and root's attach in the init namespace succeeds. Then disable unprivileged
namespaces in the guest (`user.max_user_namespaces` set to 0) and record that the account arm is skipped with a
stated reason rather than failing the boot.

Gates: [Way 3](#way-3-an-account-probe-with-no-zfs)'s skip path, the assumption that every ZFS arm needs root, and
[Q6](#q6-whether-the-account-runs-a-companion-probe).

### M7: the namespace arm sees a known holder and reports only the probe's own namespaces

Record the scan's two passes and what each must show: the worker's and the holder's namespaces while both mounts
exist, the worker's namespace alone after the holder is stopped, and no namespace after the unmount. Repeat with
the source already mounted in a second namespace before the probe runs, and require the attach arm to answer
`EBUSY` and the probe to record that name as held instead of refusing spuriously. Repeat with a root mount of the
same source made in a further namespace during the run, and require the first pass to name it and the arm to fail
rather than hide it. Repeat with a namespace pinned and no live member in it, and record that the scan does not
name it and reports the source clear: a clear report is not absence, which the document says wherever it relies on
the scan.

Gates: [Way 1](#way-1-a-root-probe-on-a-live-snapshot)'s namespace arm, the exclusive create as the freshness
oracle, and the limit that a process-visible scan is not proof of absence.

### M8: the real clone attach as evidence

Record the handler's first real attach after boot in the `first-attach` file beside the probe's record, written
by the process that made the attach, and compare its outcome with the probe's snapshot arm on the same build.
Control: an unloaded-key clone attach must fail with `EACCES`, as item 5 measured, on the same boot as a passing
probe arm.

Gates: [Way 4](#way-4-the-products-first-attach-as-the-probe)'s folded-in evidence, and the limit that a snapshot
arm is not a clone arm.

### M9: the untested release and the refused release

Run the built probe on a release it must accept and on one it must refuse, and record what it changes at start,
what it leaves behind and what it prints, as the design's acceptance item requires (the [proposal][proposal],
lines 1841 to 1843), together with step 6's own VM arms (the [plan][plan], lines 739 to 745). Control: the
accepted release must pass, so a probe that refuses everything fails its own control.

The refused-release arm needs a release, or a fault, that makes an arm fail. A fault is buildable in the guest
today: a transient unit named `git-worktree-probe-test.service`, created with `systemd-run` and carrying
`SystemCallFilter=` for `move_mount` or `fsmount` together with `SystemCallErrorNumber=EPERM`, runs the
`start-probe` mode as its `ExecStart=`, which makes the move or the attach arm fail with `EPERM` and the probe
refuse. Without the error number a system call filter kills the process with `SIGSYS` rather than failing the call
(`systemd.exec(5)`, `SystemCallFilter=`), which would test the signal path and not the arm. The same transient
unit carries the passing control arm, and it is also what runs the probe before step 19's unit exists. The
untested-release arm needs the custom CachyOS template that no built-in or local image provides (the
[proposal][proposal], lines 1826 to 1834; [measurements][measurements], item 16, lines 668 to 706); until it
exists, run this arm on the Ubuntu guest with a labelled limit that says it is not the host release.

Two more arms use deliberate faults rather than releases. A test-only build that keeps one extra descriptor open
on the attached mount must make the move and unmount arm fail with `EBUSY`, and the same build closing it must
pass, so that arm's unmount is shown to check the mount rather than pass vacuously (`fs/namespace.c:2010`). A
`SIGKILL` to the mode process must leave the unit's `ExecStart` to run, which is what `ExecStartPre=-` buys, and
must leave a started record that the add preflight and `doctor` read as not current rather than as a pass.

Gates: every way, and the acceptance item.

### M10: the setup-time dataset, if the owner adds one

If [Q3](#q3-whether-a-setup-time-probe-dataset-exists) is answered yes, measure the setup command's creation and
its idempotent second run, the space it takes, that nothing else mounts it, that the sweep and the destroy-anyway
path never see it, that the sanoid configuration excludes it, and what the probe records when the dataset or its
snapshot is missing (`ENOENT` from `fsconfig(create)`). Control:
[Way 1](#way-1-a-root-probe-on-a-live-snapshot)'s source on the same build, so the difference in exercised arms is
a measured one.

Gates: [Way 5](#way-5-a-setup-time-probe-dataset) only.

## Questions for the owner

Each question is separable from the others, each option states its costs, and the ranking gives the reason for
every adjacent pair. The order is the order of dependence, so an answer never needs a later question's answer:
the plan reading, then what a failure disables, then the source, then the questions that inherit those answers,
then the companion, then the second record file, which nothing else needs. Each question opens with a concrete
case and describes every option in words rather than by a section name, so it can be read and answered in the
question tool without this document beside it. Where an answer changes the plan or a decision, the question says
so.

### Q1: whether the plan bars the probe's unmount arm

The plan names the designs that block the route early, the start probe, the runtime layout checker and the
staging design, and says of them that "No production kernel request runs before the first two, and no unmount
runs before the third" (the [plan][plan], lines 82 to 88). Step 9 builds the staging design. The reading in one
line: this design takes the sentence as production-only, so the probe's own unmount arm runs from step 6, because
that arm mounts and unmounts inside a namespace and a tmpfs of its own and touches nothing the product owns. The
case: a boot on a fresh host, before any clone exists; the probe mounts a read-only snapshot in its own namespace,
moves it to a directory inside its own tmpfs and unmounts it there, and the product's staging folder is never
used.

- That reading holds (Recommended). The arm runs from step 6. Pros: the evidence that a second process can move
  and unmount a mount arrives with the rest of the probe. Cons: a reader has to accept that the sentence is about
  the product's mounts.
- The sentence covers every unmount, including the probe's. The probe's move and unmount arms, and the scan that
  judges them, are built in step 9 instead, still inside the probe's own namespace, and the rest of the probe,
  including the holder, stays in step 6. Pros: the sentence keeps its literal scope and the arm is written beside
  the product's unmount code. Cons: step 6 ships a gate with an unexercised unmount half, and step 9 grows a
  second consumer of its staging design.

Ranking: that reading holds, then the literal reading, because the probe unmounts nothing the account or the
product owns and the evidence is worth having with the rest of the gate; the literal reading costs a gate that
never exercises half of what the answer names.

### Q2: what a failed arm disables

An arm that ran and failed is a real observation, and what it disables is open. The arms are: the walk to the
slot, the attach, the move, the unmount and the namespace scan. The case: after a kernel package update the
probe's attach arm fails with `EPERM`. A person then runs an add for a worktree whose slot has no clone yet, and
later removes a clone whose worktree they no longer need. The option that refuses new managed attaches and keeps
every other operation lets the add take the reflink or ordinary Git route and print one line, and lets the removal
succeed for a clone that is not mounted. The option that mirrors the module gate takes the same fallback but
refuses removal for a mounted clone and an unmounted one alike until the kernel changes. The option that warns and
continues lets the add proceed into the attach the probe just saw fail.

Every option that refuses attaches leaves existing unmounted clones unmounted until the route returns, which is
the availability cost the allowlist answer was chosen against: the project kept a functional probe instead of an
exact release allowlist because an allowlist would disable the route after every kernel update (the
[proposal][proposal], lines 1294 to 1306). The option that refuses nothing loses that protection instead.

- Refuse new managed attaches, keep every other operation (Recommended). Unused slots take the same-dataset
  reflink (a cheap copy-on-write copy of the dataset's blocks) or ordinary Git route, which needs no ZFS attach.
  Boot reconcile attaches nothing at all and keeps every intent and registration. Removal of a clone that is not
  mounted stays available through the module-gated ioctls, the raw ZFS requests the module gate guards and the
  mount calls it does not. Removal of a mounted clone needs an unmount and the scan of every task's namespace, so
  when the failed arm is the move, the unmount or the scan, that removal is refused and reported rather than
  silently degraded. Pros: the recovery path that needs no attach stays available, which is exactly when a person
  needs it. Cons: the managed route is half available, and a reader must know that attach and removal are gated
  separately.
- Mirror the module gate, the start check that disables raw requests when the loaded module does not match the
  binary: refuse every ZFS operation. Unused slots take the reflink or Git route. Boot reconcile attaches nothing
  and unmounts nothing, keeping intent. Removal refuses for a mounted clone and an unmounted one alike, because
  destroying a clone is a raw request. Pros: one rule for every start check, with wording that already exists.
  Cons: an existing unmounted clone cannot be removed until the kernel changes, which strands the bindings the
  design preserves, and it is the cost the allowlist answer was chosen against.
- Warn and continue: the add proceeds into the attach the probe saw fail. Unused slots, reconcile and removal are
  unchanged. Pros: no availability loss, and it matches the Git-version check's warn-and-continue precedent.
  Cons: the failure appears inside the operation the probe exists to protect, on a claimed slot.

Ranking: refuse new attaches and keep the rest, then mirror the module gate, then warn and continue, because
removal without an attach is the recovery path a broken attach needs, and a warning leaves the failure to land
mid-operation.

### Q3: whether a setup-time probe dataset exists

The probe needs a ZFS source to attach. With no new dataset the only candidate is a snapshot of the checkout,
whose name the account can create (the [proposal][proposal], lines 319 to 321). The accepted configuration keeps
the home template recursive, and that template reaches the checkout dataset and each clone by descent and
snapshots home hourly (the [proposal][proposal], lines 1081 to 1092), so such a snapshot usually exists; the same
passage lets the account destroy any checkout snapshot (the [proposal][proposal], lines 324 to 326), the space
guard prunes home's hourlies, and a namespace that already holds a snapshot makes the probe skip it. The
alternative is one small dataset created once by root: `zpcachyos/nosnap/git-worktree-probe`, with one empty
snapshot named `ready`, sitting under the container `zpcachyos/nosnap`, which is unmounted and carries
`autosnap = no` (the [proposal][proposal], lines 1098 to 1124). Its properties are `mountpoint=legacy`,
`canmount=off`, `com.sun:auto-snapshot=false` and a user property naming it as the probe's source, so nothing
mounts it automatically and no recursive sanoid ancestor reaches it. The account holds no grant there, so it can
neither create, mount, rename, snapshot nor destroy it, and the sweep that lists the clone parent's children never
sees it. The probe notices its absence by trying to attach it: a missing dataset or a missing `ready` is the
`ENOENT` from `fsconfig(create)`, reported as an unexercised arm (a step that could not run) and never created.

The case: a fresh host, migrated and rebooted, before any clone exists and before sanoid's first hourly snapshot
of the checkout. Without the dataset the probe finds no unheld checkout snapshot and records the attach arm as not
exercised. With the dataset the probe attaches `ready` on every boot, and the attach arm always runs.

- Yes, the setup-time probe dataset (Recommended). It changes what the setup command that `doctor` prints creates,
  the plan's step 19 setup commands, and the sanoid sections with their guard report: steps 18 to 20 of the plan.
  The boot unit's `ExecStartPre=-` and the `doctor` filesystem section are not part of this cost, because every
  answer needs them. Pros: the source always exists, nothing outside root mounts it, every arm runs on every boot,
  the attach's exclusive create (a command that makes the kernel refuse a source whose superblock, its record of
  that mount, already exists) proves a fresh mount, no account-influenced name is read, and no directory read
  happens at start. Cons: a new object in the layout, with a sanoid section, a configuration field and a way to go
  missing or drift, which the probe reports as an unexercised arm.
- No, a snapshot of the checkout (the design's Way 1). Pros: no layout change, nothing new in the sanoid
  configuration, the sweep or preflight, and the hourly template usually leaves a snapshot to take. Cons: the
  source can be absent right after migration, can be destroyed by the account or a prune between the directory
  read and the attach (an `ENOENT` the attach arm reports), or be held by another namespace, so the arms can be
  unexercised while any of that lasts; the name is account-influenced; the one directory read may update its
  access time; and the probe's own hold can make a prune of that exact snapshot fail `EBUSY`.

Ranking: yes, then no, because a source that always exists and that nothing else mounts is what makes every arm
run and removes the reuse and influence questions at their root; the layout cost is one object with a named place
and its exclusion, while the cost of no is steps that silently do not run and a source the account can remove.
This design ships Way 1's source until the owner answers yes, so the answer decides the source and the layout
change.

### Q4: whether an unexercised arm may enable the route

An arm is unexercised when it could not run, not when it ran and failed. The case: a fresh host whose probe found
no unheld source, so its attach arm is recorded as not exercised. A person runs their first managed add. The
option that reports the gap and keeps the route enabled lets the add proceed and print a line naming the arm that
did not run, which `doctor` repeats. The option that blocks the managed route sends the add to the same-dataset
reflink or ordinary Git route instead of a managed attach, prints that line, and holds every managed attach until
a source exists.

The causes differ by source. With a checkout snapshot the attach arm can be unexercised because no snapshot
exists, because a prune or the account destroyed the one the probe found before it attached, or because other
namespaces hold every candidate. With the setup-time dataset of
[Q3](#q3-whether-a-setup-time-probe-dataset-exists) it is unexercised only when the dataset or its `ready`
snapshot is missing, which `doctor` then reports as a failure rather than a gap, and on a kernel older than 6.6,
which refuses the command that proves the source is fresh.

- Reported, and the route stays enabled. Pros: a host with no source keeps a working route, the answer's "admits
  untested releases" posture holds, and `doctor` names the gap. Cons: the steps the answer exists to run may never
  run, so a broken kernel passes with the gap visible only in `doctor`.
- The unexercised arm blocks the managed route, with the same per-operation effects
  [Q2](#q2-what-a-failed-arm-disables) states: an unused slot takes the reflink or ordinary Git route, removal of
  a clone that is not mounted stays available unless that answer refused every ZFS operation, and removal of a
  mounted clone follows the failed-arm answer when the arm that did not run is the move, the unmount or the scan.
  Pros: the route is only ever enabled by evidence. Cons: with a checkout snapshot the route stays blocked from
  the first boot until the next boot after a snapshot exists, because the probe runs at boot only apart from a
  checkout-provider restart and the first add does not create one; with the setup-time dataset that objection goes
  away, which makes this the stronger answer if
  [Q3](#q3-whether-a-setup-time-probe-dataset-exists) is answered yes.

Ranking, by the answer to [Q3](#q3-whether-a-setup-time-probe-dataset-exists). If the setup-time dataset exists,
the unexercised arm blocks first and reporting comes second, because the source then always exists and evidence
can gate the route without a lockout, so an owner who answers yes there should expect to prefer blocking here. If
the source is a checkout snapshot, reporting comes first and blocking second, because a source that can be absent
turns blocking into a lockout until the next boot.

### Q5: whether a missed deadline fails closed

The probe runs as a parent process, the mode, that forks the work into a worker and enforces a deadline over it,
and it records a timeout either way. The case: a kernel whose attach blocks in the kernel, so the probe never
finishes. The login the boot unit orders before is admitted late, the record says timed out, and the next add
either refuses and prints why, or proceeds on the strength of a record that says the probe could not finish.

- Fail closed (Recommended). A timeout leaves no pass, so the record is not current and adds are refused, with
  their reason printed: an unused slot takes the reflink or ordinary Git route, removal of a clone that is not
  mounted stays available, and removal of a mounted clone follows the answer
  [Q2](#q2-what-a-failed-arm-disables) gives for a failed-arm case. Existing bindings and intent are kept. Pros: a
  kernel that hangs an attach neither blocks a login indefinitely nor silently enables the route, and it matches
  the gate's posture of no force flag. Cons: a slow but working kernel loses the route until a timing measurement
  sizes the deadline ([M4](#m4-what-the-probe-costs)).
- Fail open, with a warning. Pros: availability is never lost to a slow boot. Cons: an attach that hangs is
  attempted on the strength of a record that says the probe could not finish, which is the failure the answer
  exists to catch.

Ranking: fail closed, then fail open, because the probe's whole value is that a missing pass is not a pass, and
the cost is a deadline that measurement can size.

### Q6: whether the account runs a companion probe

The companion is a second, smaller check that runs as the account, in a user and mount namespace, and exercises
the walk, a tmpfs mount and unmount, and the refusal controls, without touching ZFS (the design's Way 3). The
case: a kernel update changes what a user namespace may pass to `mount_setattr`, so a mount made as the account
refuses with `EPERM` while the root probe, in the init user namespace, passes. The product's own attach runs as
root, so the companion cannot stand in for the probe and its failure does not show that the attach would fail. The
root-only option runs nothing more: the add reads the record the boot wrote, and the account-side regression is
seen only where something makes an account-side mount. The root-plus-companion option runs the companion first and
stops with a line naming the mount call, before any root work starts.

- Root only (Recommended). Pros: one implementation, no user-namespace dependency, and the arm that matters needs
  root anyway. Cons: an account-side regression of the walk or the mount surface is seen only where an
  account-side mount is attempted, which the managed attach path does not do.
- Root plus an account companion. Pros: a per-invocation check of the walk, the mount surface and the wrapper's
  helpers before any root act, and the wrapper's tests already carry the skip control. Cons: a second
  implementation, a check that never touches ZFS and so never predicts the root attach's outcome, and a skip
  exactly on hosts where user namespaces are refused.

Ranking: root only, then the companion, because the account arm cannot observe the dataset mount path, which is
the failure the answer asks about, so a companion that refuses would stop a managed attach without evidence that
the attach would fail; its per-invocation timing is unmeasured either way.

### Q7: whether the first real attach records its outcome

The product's first managed attach after a boot either succeeds or fails, and nothing keeps its outcome today: the
handler reports it in the journal, and the probe's record holds only the probe's arms. The design can have the
handler process that made that attach write the outcome to a second file, `first-attach`, beside the probe's
record, so `doctor` shows it next to the probe's result. No part of the decision record, the proposal or the plan
decides that write.

The case: a fresh host whose probe passed on a snapshot, and whose first managed add attaches a real clone
read-write at a real slot in the host namespace. With the file, `doctor` can show that attach's outcome beside the
probe's. Without it, the only evidence about a real clone that boot is a journal line.

- No, the probe record only (Recommended). Pros: no new write in the attach path, no second file, and an
  operation's failure cannot be read as a compatibility verdict. Cons: the one outcome from a real clone, a real
  slot and the host namespace is not kept, so `doctor` can show the probe and not the product.
- Yes, write `first-attach`. Pros: the outcome from a real clone, a real slot and the host namespace is kept, and
  `doctor` compares it with the probe's result on the same build. Cons: a new duty in the attach path, which plan
  steps 4 and 8 build, a second file, a `doctor` reader and a new measurement
  ([M8](#m8-the-real-clone-attach-as-evidence)), and an operation's failure read as a compatibility verdict.

Ranking: the probe record only, then writing `first-attach`, because that write is a duty in the product's attach
path that no record decides, and because it mixes an operation's failure with a compatibility verdict; the file
earns its place only if evidence from a real clone is worth that duty and that ambiguity.

## What a writer settles without asking

- The scratch location: the record's directory under `/run`, the tmpfs mounted at its scratch mount point, the
  slot, the move-target directory and the refusal tree inside that tmpfs, and that the probe never touches the
  account-owned staging folder or the checkout's own directories.
- The source selection rule: the first accepted candidate in the order the read returns, a valid component name,
  the dataset-name bound, trying the next name when the exclusive create answers `EBUSY`, and the not-exercised
  record when no candidate qualifies.
- The arm order, the private-namespace setup before any mount, and every syscall the probe uses.
- That the probe runs at boot only: one root process per boot, from `ExecStartPre=-` of the existing unit, with
  no on-demand trigger and no second writer of the record. A module cannot be reloaded on this host without a
  reboot, so a module change arrives with a reboot that runs the probe again
  ([The kernel the design names](#the-kernel-the-design-names)); the probe also runs again when the checkout
  provider is restarted, because the unit requires that provider
  ([Who runs it, and where](#who-runs-it-and-where)). A trigger keyed to a module load would add a second writer
  of the record and a probe that can race an attach, and this host cannot exercise it, so this design does not
  propose one. A record whose fingerprint no longer matches the running kernel, module or boot is stale, and the
  managed route then refuses the attach and keeps the intent (the [proposal][proposal], lines 351 to 352).
- That the holder is forked by the mode process and the unmount child by the worker, that each child carries a
  parent-death signal and pipes of its own as the protocol after the arms states, and that the mode kills the
  worker's process group at the deadline.
- That the holder starts before the move and attaches its own mount of the same source.
- That the protocol after the arms settles who closes which pipe end, when a child moves its ends to low
  descriptors, and that the mode writes the record from the outcome lines it reads.
- That the namespace arm's holder runs at every start, with its cost under [M4](#m4-what-the-probe-costs), and
  that the scan is judged against the probe's own namespaces rather than against an empty set.
- That the move and unmount arm closes every descriptor the worker holds on the attached mount before it forks,
  and that its unmount must not answer `EBUSY`.
- That the probe reads no file content, only `fstatfs`, a bounded directory listing on its own mount, and, under
  [Way 1](#way-1-a-root-probe-on-a-live-snapshot) only, `getdents` on `.zfs/snapshot`.
- The mount flags, the read-only request, the exclusive create as the freshness oracle, and the `EBUSY` handling
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures.
- That the unmount is by path inside the probe's own tmpfs, and that the probe therefore does not validate the
  product's held-descriptor unmount. Whether the plan's sentence bars that arm at all is
  [Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm), not a reading this design settles.
- That the probe tears down without unmounting its own tmpfs, whose removal is the namespace's death.
- The wording of the wrapper line, the `doctor` section and the journal line, under the design's own reporting
  rules (the [proposal][proposal], lines 755 to 762).
- That `doctor` reads the result and never runs a probe, and that no force flag exists.
- The record's directory, mode, fields and atomic-rename write, and the started record before the fork. The
  `first-attach` file is not settled here; it is [Q7](#q7-whether-the-first-real-attach-records-its-outcome).
- That the probe is a handler mode named `start-probe`, run as `ExecStartPre=-` of the existing unit, that it runs
  the module gate first and skips itself when the gate refuses, that the mode process enforces the deadline and
  exits successfully whenever it reaches its own end, and that no on-demand channel is added.
- The deadline's value, set only after [M4](#m4-what-the-probe-costs) measures the arms; the unit carries no
  start timeout, and no plan step sizes one: step 22 sizes the mount-request deadline of step 7, the unmount
  deadline of step 9 and any shipped deadline those waits would need (the [plan][plan], lines 1778 to 1779 and
  1811 to 1812).
- The fallback for an unused slot and the preservation of existing bindings, which the accepted design already
  decides.

## What this proposal does not change

It does not introduce an allowlist, does not withdraw or loosen the module gate, does not authorize a path mount
or a new request channel, does not add a force flag, does not make the probe a `doctor` action, does not create
or destroy a dataset at start, does not change the intent vocabulary, does not move the handler's `boot-reconcile`
and `shutdown-unmount` modes out of the host mount namespace, does not add a pool object of its own (the setup-time
probe dataset is the owner's act, printed by `doctor` and put as
[Q3](#q3-whether-a-setup-time-probe-dataset-exists)), does not change any removal, promote, staging or delegation
decision, and does not decide any of the remaining blocking designs. It also does not bring the discarded
pool-writing probe back: that one contradicts the accepted design, as
[its note](#way-2-ruled-out-by-the-accepted-design) states.

## Sources read

- [The decision record][decision], whole.
- [The proposal][proposal], the sections this document cites, and the section lists it names: "Open items the
  decisions leave", "ABI gate and tests", "Project decisions", "Decided defaults", "Descriptor mount and
  unmount", "Boot and shutdown unit", "Waiting and refusal", "What doctor checks and prints" and "Acceptance
  tests the build still owes".
- [The plan][plan], the sections this document cites, and the section lists it names: "Dependencies and their
  state", "Wrapper code the design reuses" (read whole), "What the design forbids", steps 6, 7 and 9, steps 18
  to 20, "What can start now", "Open items and the step that needs each", and "Where the design is silent: this
  plan's suggestions".
- [The measurements record][measurements], the items this document cites, each with the measured output beside
  it.
- OpenZFS `zfs-2.4.4` at commit `71a9f9578616`, cloned on 2026-11-19:
  `module/os/linux/zfs/zpl_super.c`, `module/os/linux/zfs/zfs_vfsops.c`,
  `module/os/linux/zfs/zpl_ctldir.c`, `module/os/linux/zfs/zfs_ctldir.c`,
  `module/os/linux/zfs/zfs_ioctl_os.c`, `module/os/linux/spl/spl-zone.c`, `module/zfs/zfs_ioctl.c`,
  `module/zfs/dmu_objset.c`, `module/zfs/dsl_crypt.c`, `module/zfs/dsl_dataset.c`, `module/zfs/spa.c`,
  `module/zfs/spa_stats.c`, `module/zfs/txg.c`, `module/zfs/vdev_label.c` and `cmd/zfs/zfs_main.c`.
- The kernel header and source named under [The kernel headers and source](#the-kernel-headers-and-source): this
  host's `/usr/include/linux/mount.h` (linux-api-headers 1:7.1-1), the 7.3.5 tree's
  `/usr/src/linux-cachyos/include/uapi/linux/mount.h`, Linux v6.8 and v7.3 `fs/super.c`, Linux v7.3
  `fs/fsopen.c`, `fs/namespace.c`, `fs/pnode.c`, `kernel/exit.c` and `kernel/sys.c`, with `fs/fsopen.c` fetched at
  v5.2, v5.11, v5.12, v5.15 and v6.0 to v6.7 as well. The round that added the parent-death and unmount citations
  fetched `fs/namespace.c`, `fs/pnode.c`, `kernel/exit.c` and `kernel/sys.c` at v7.3 itself.
- systemd v262 `src/core/service.c`, `src/core/unit.c` and `src/core/exec-invoke.c` (fetched), and
  `systemd.service(5)`, `systemd.exec(5)`, `systemd.unit(5)`, `systemd.kill(5)` and `statx(2)` read on this host.
- `PR_SET_PDEATHSIG(2const)` on this host, for the parent-death signal and the cases that clear it.
- This host, read only, 2026-11-19: `uname --kernel-release`, `/sys/module/zfs/version`, `/proc/kallsyms`,
  `/proc/filesystems`, `/sys/kernel/btf/`, `/dev/zfs` mode, `/proc/modules` (for the `zfs` use count),
  `/proc/spl/kstat/zfs/zpcachyos/txgs`, `/sys/module/zfs/parameters/zfs_txg_history` and `zfs_txg_timeout`,
  `findmnt --target /`, `/proc/sys/user/max_user_namespaces` and `/proc/sys/kernel/unprivileged_userns_clone`.
- The wrapper's code: `package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  `.../subvolume/fs_mount_options.rs`, `.../subvolume/subvolume_remove_walk.rs`,
  `.../health/doctor_filesystem.rs`, `.../service/wait_ceilings.rs`,
  `.../invocation/git_version.rs`, `.../invocation/git_version_cache.rs`,
  `.../binary/binary_worktree_removal_around_a_volume_tests.rs`, and `package/git-policy/cli/SPEC.md`
  "Compatibility and degradation".

[decision]: ../decision/cli-git-zfs-worktree-design.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[owed-tests]: cli-git-zfs-worktree-design-proposal.md#acceptance-tests-the-build-still-owes
[plan]: cli-git-zfs-worktree-implementation-plan.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
