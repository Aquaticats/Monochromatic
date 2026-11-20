# The start probe for the ZFS worktree route

**Non-final, unapproved proposal by an agent.** This document designs one open item of the accepted ZFS worktree
design, "The start probe a kernel-release answer asks for", so that step 6 of the implementation plan can be
written. A Pi session wrote it on 2026-11-19, revised it twice the same day to answer a gate review of commit
`4167f50724` and then a gate re-check of `fa52e2e948`; it is not accepted. It was written without running a VM, a
probe or any pool command, from the sources listed under [Sources read](#sources-read). What it checked: the
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
  [Q5](#q5-whether-an-unexercised-arm-may-enable-the-route).

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

- The handler's mount and unmount modes "Run both modes in the host mount namespace" (the [proposal][proposal],
  line 496), and this proposal keeps that. The probe is not one of those modes: it attaches nothing for a
  worktree, it runs in a child mount namespace of its own, and the limit that it therefore cannot prove the
  host's propagation is stated under [Limits](#limits).

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
  wrapper invocation; which start it must cover is put to the owner as
  [Q2](#q2-which-start-the-probe-covers), and the recommendation places it in the boot unit.
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
  dataset, or a missing snapshot of it, only as the `ENOENT` from `fsconfig(create)`.
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
  `/proc/modules` lists `zfs` with use count ten (read-only, 2026-11-19).
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
- The probe reads no file content from the attached source; it proves the mount through `fstatfs`, a bounded
  directory read and mount facts. A source whose root directory is unreadable to root is therefore not detected
  by content.
- A probe that holds a snapshot for its duration can make a concurrent `zfs destroy` of that exact snapshot
  refuse `EBUSY`; [M2](#m2-the-mount-table-and-the-pool-are-unchanged) records whether that window is
  observable. The holder and the unmount child end with the process that forked them, so neither outlives a
  probe killed at its deadline ([M3](#m3-interruption-leaves-nothing)). The residue is the worker: a `SIGKILL`
  cannot interrupt a kernel wait, so a worker inside one holds the snapshot until the wait returns.
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

The ways that remain follow. Each states who runs it and where, what it exercises, what it changes at start, what
it leaves behind, what a refusal prints, and its costs. Cost is stated as the work to write and to test, never as
a duration, because no measurement of this work exists.

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
outcome is written into the `first-attach` file beside the probe's record; later managed attaches read that record.

What it changes at start: nothing extra. What it leaves when it fails: exactly what a failed first attach leaves,
a claimed and registered slot that needs the design's recovery.

For it: zero extra work, the exact topology (a real clone, its slot, the host namespace, ZED ordering) and no
synthetic source.

Against it: it runs after the first attach, which is the attach the answer wants qualified, so it cannot refuse
it; it needs a clone to exist, so a fresh host never probes; and it reports an operation's failure as a
compatibility verdict, which mixes two different questions.

Build cost: a record block only. Test cost: one VM arm on the built handler, plus the existing add controls.

### Way 5: a setup-time probe dataset

It runs the same root probe as [Way 1](#way-1-a-root-probe-on-a-live-snapshot), from the same place, with one
difference: the source is a dataset the root-once setup creates, not a snapshot the probe finds. The setup
command that `doctor` prints (administrative commands printed by `doctor` do not run inside the wrapper, the
[proposal][proposal], line 153) creates the dataset and one empty snapshot, and records the name in the root-owned
configuration the handler reads.

The example this design names, for the owner to accept or replace at
[Q4](#q4-whether-a-setup-time-probe-dataset-exists), is the dataset `zpcachyos/nosnap/git-worktree-probe` with
the snapshot `ready`. It sits under the unmounted container `zpcachyos/nosnap`, which the sanoid configuration
already carries as `autosnap = no` and whose children are the pool's unmounted exclusion datasets (the
[proposal][proposal], lines 1099 to 1124). Its properties are `mountpoint=legacy`, `canmount=off`,
`com.sun:auto-snapshot=false` and one user property marking it as the probe's source; the snapshot is empty.
Nothing mounts a legacy dataset automatically and `canmount=off` stops `zfs mount -a` and the mount generator too
(the [proposal][proposal], lines 262 to 270), so nothing but the probe attaches it. Its key is the pool root's,
which the boot unit already waits for before `ExecStart`; an unavailable key is then an `EACCES` refusal from the
attach arm, reported rather than hidden.

What the account can do to it: nothing. Delegation is scoped to the checkout dataset and the clone parent (the
[proposal][proposal], lines 319 to 327), and that same passage says not to put unrelated ZFS descendants there.
This dataset is under neither, so the account holds no `create`, `mount`, `destroy` or `userprop` on it and can
neither create, rename, mount, snapshot nor destroy it. That is the property
[Way 1](#way-1-a-root-probe-on-a-live-snapshot) cannot have, since its source name is account-influenced.

How its existence is detected: the probe asks the kernel, not the pool. It attaches `<dataset>@ready` with
`FSCONFIG_CMD_CREATE_EXCL`, and a missing dataset or snapshot is the `ENOENT` from `fsconfig(create)`, as the
OpenZFS facts state ([Facts](#openzfs-at-zfs-244)). A missing name is reported as an unexercised attach arm and
never created.

What it changes at start: the same list as [Way 1](#way-1-a-root-probe-on-a-live-snapshot) minus the directory
read of `.zfs/snapshot`, so no account-influenced name and no access-time question.

What it leaves when it fails or is interrupted: the same as [Way 1](#way-1-a-root-probe-on-a-live-snapshot). The
dataset itself outlives every run by design.

For it: the source always exists and no other mount can hold its superblock, so the attach arm runs on a fresh
host, and the exclusive create refuses rather than silently reusing a superblock, which is the reuse hazard
[Way 1](#way-1-a-root-probe-on-a-live-snapshot) has to skip past. It is not account-influenced, needs no
directory read, and no other process pins or destroys it, so it cannot collide with the space guard, the sweep or
a removal.

Against it: it adds an object to the pool's layout, which is why it is the owner's question
([Q4](#q4-whether-a-setup-time-probe-dataset-exists)) and not this proposal's decision. It needs the setup
command, a sanoid exclusion section and the user property, preflight acceptance, and it is one more thing that
can go missing or drift. Making it a clone of the checkout snapshot instead of an independent dataset would
exercise a real clone shape, but a clone pins its origin snapshot until the clone is destroyed, so this design
keeps it an independent dataset and leaves the clone shape to
[Way 4](#way-4-the-products-first-attach-as-the-probe).

What it changes in the plan, all of it named here rather than decided:

- Step 18, `doctor`: the root-once and setup commands it prints gain the dataset creation, its snapshot and the
  configuration field, and its filesystem section gains the probe result (the [plan][plan], lines 1490 to 1530).
- Step 19, the migration and the units: the setup gains the commands, and the boot unit gains `ExecStartPre=`
  running the probe (the [plan][plan], lines 1572 to 1593).
- Step 20, sanoid coverage: the dataset gets its own `autosnap = no` section beside the named exclusions, with
  the `com.sun:auto-snapshot=false` marker, and the guard's report names it (the [plan][plan], lines 1677 to
  1700).
- No step of the sweep, the destroy-anyway path or preflight changes, because the dataset is outside the pinned
  clone parent whose immediate children the sweep enumerates (the [proposal][proposal], lines 540 to 541) and no
  account grant reaches it. A draft of this design assumed a place under that parent, which would have changed
  all three.

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
the owner's to accept at [Q4](#q4-whether-a-setup-time-probe-dataset-exists): with a yes the recommendation's
source is Way 5, and without that answer this design uses Way 1.

Way 1 ranks second because it reaches `zfs_domount` and the key path at start on a host whose checkout already
has an unheld snapshot, without a layout change, and it refuses before the first managed attach. It ranks above
Way 3 because Way 3 never reaches the dataset mount path, which is the failure the answer exists to catch.

Way 3 ranks third because it catches a syscall or namespace regression before any root act and can run per
invocation, but it cannot refuse a bad ZFS attach, and it is skipped exactly where confinement matters. It ranks
above Way 4 because Way 4 cannot refuse anything: its evidence arrives after the attach it would have qualified,
and it never runs on a host without clones.

Way 4 ranks fourth and is still part of the recommendation, folded in free: the first real attach's outcome lands
in the `first-attach` file beside the probe's record with no new mechanism, which is the only evidence about a
real clone in the host namespace that any way gives.

Way 1 and Way 3 are complements, not alternatives: Way 3's per-invocation check is
[Q7](#q7-whether-the-account-runs-a-companion-probe), and [the ranking](#ranking) orders the ways that could serve
as the start check's own evidence.

The recommended shape is the probe's arms under [The proposed probe](#the-proposed-probe), sourced from
[Way 5](#way-5-a-setup-time-probe-dataset) if the owner answers [Q4](#q4-whether-a-setup-time-probe-dataset-exists)
yes and from [Way 1](#way-1-a-root-probe-on-a-live-snapshot) if not, with
[Way 4](#way-4-the-products-first-attach-as-the-probe)'s record folded in and
[Way 3](#way-3-an-account-probe-with-no-zfs) only if the owner wants a per-invocation companion.

## The proposed probe

This section proposes the recommended shape in the detail the open item asks for. The arms are the same whichever
answer [Q4](#q4-whether-a-setup-time-probe-dataset-exists) gives; only the source differs. Every part of it is a
writer's proposal inside an undesigned item, and the questions under
[Questions for the owner](#questions-for-the-owner) can change parts of it.

### The arms

1.  Namespace and scratch. The mode process forks a worker, and the worker calls `setpgid(0, 0)` so the mode can
    kill the whole group, then `unshare(CLONE_NEWNS)`. It makes every mount in the new namespace private
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
    descriptor on the worker's mount and no mount it did not make; it closes every descriptor numbered higher
    than standard error before it runs. It sets `PR_SET_PDEATHSIG` to `SIGKILL` and holds a control pipe whose
    end of file ends it. It unshares its own mount namespace, makes it private, and attaches the same source
    read-only at the directory `holder`, which the mode created beside `scratch`: its mount lives in its own
    namespace, and the worker's tmpfs is not visible to it. It uses the plain `FSCONFIG_CMD_CREATE`, because
    reusing the worker's superblock is what this arm wants: it needs a mount in a second namespace, not a second
    superblock. It starts after arm 5 and before the move, because a probe that has already unmounted has no
    mount for a holder to hold, and it waits for the mode's signal.
7.  The move target and the unmount, from a second process. The worker forks a child, which sets
    `PR_SET_PDEATHSIG` to `SIGKILL` and holds the same control pipe. The child reopens the slot through the
    recorded parent, moves the mount from the held mount descriptor onto the move-target directory descriptor,
    verifies it there with `mountinfo`, closes every descriptor that would hold it busy, and unmounts it by path
    inside the probe's own tmpfs. The worker waits for the child, bounded by the mode's deadline. It never unmounts
    a path in the account's tree, which is the only tree the accepted rule forbids (the [plan][plan], line 275).
    This is probe-private code: it proves that a second process can move and unmount what another process
    attached, and it does not exercise the product's unmount from a held descriptor, which step 9 owns.
8.  Namespace scan. Run the probe's own scan: group live tasks by `/proc/<pid>/ns/mnt`, read a representative
    `mountinfo` per namespace, and match the ZFS filesystem and exact source. Require every namespace that mentions
    the source to be one of the probe's own, the worker's or the holder's, since arm 4 proved no superblock of it
    existed before the probe and nothing else can attach it; a foreign mount of the same source is reported as a
    failure of this arm rather than hidden. Require the holder's namespace to be named while it lives, stop the
    holder, and require only the worker's namespace to remain. A process-visible scan is not proof of absence, and
    a pinned namespace with no live member stays invisible (the [proposal][proposal], lines 708 to 710), so a clear
    report is not absence. This scan is probe-private; step 9 builds the product's (the [plan][plan], lines 909 to
    916), and this design proposes that step 9 reuse this grouping and source match rather than write a second
    scanner of the same mechanism.
9.  Teardown. Unmount the probe's mount if it remains, unmount the tmpfs, stop the holder and wait for it, and
    exit. The mode process writes the record only after the worker exits or its deadline passes.

If an arm fails, the worker stops at that arm, records its name and errno, tears the namespace down, and reports.
A name held by another namespace is not a failure: arm 3 tries the next one. The refusal controls in the walk arm
are expected refusals: an arm that attaches instead of refusing fails the probe, so the harness has a positive
control for its own check.

### Who runs it, and where

Root, from the handler binary, inside the private mount namespace the run creates. The probe never runs in the
host mount namespace, never runs as the account, and never runs inside `doctor`.

Placement: a new mode of the handler binary, `start-probe`, beside `boot-reconcile`, `shutdown-unmount` and
`adopt-worktree-parent` (the [plan][plan], lines 640 to 641 list those three; step 6 adds this fourth name to that
list), run as `ExecStartPre=` of the existing boot unit.
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

The mode process enforces the deadline rather than the worker: it forks the worker into a process group of its
own, waits for it with a monotonic deadline, and on expiry sends `SIGKILL` to that whole group and to the holder,
reaps the worker only with `WNOHANG`, records a timed-out result, writes the record and exits successfully without
waiting further. It exits successfully in every case because the unit's start must continue: a failed or
timed-out probe is a verdict in the record, never a failed unit. The worker's mounts die with its namespace, so a
worker that survives an uninterruptible wait, which `SIGKILL` cannot interrupt, holds nothing in the host mount
table; it can hold the snapshot busy until the wait returns, which is under [Limits](#limits). The holder and the
unmount child carry `PR_SET_PDEATHSIG` set to `SIGKILL` and a control pipe whose end of file ends them, so neither
outlives the process that forked it; the unit's own `KillMode=` default of `control-group` is a second bound
rather than the only one (`systemd.kill(5)`, `KillMode=`). What a missed deadline does to the route is
[Q6](#q6-whether-a-missed-deadline-fails-closed); the deadline's value comes from
[M4](#m4-what-the-probe-costs).

The unit sets no `TimeoutStartSec=`. A start timeout bounds a oneshot's `ExecStart` as well as its
`ExecStartPre=`: the timer is armed per command with `s->timeout_start_usec` (systemd v262 `service.c:2991`,
armed at `:2178`), and a command that times out sends the unit to its stop action, which for this unit is
`shutdown-unmount`. The reconcile's duration is unsized until plan step 22, so any value that fits the probe could
kill the reconcile mid-attach and then unmount what it had attached. The oneshot default is no start timeout at
all (systemd v262 `service.c:1225` to `:1227`; `systemd.service(5)`, `TimeoutStartSec=`), the probe's own deadline
bounds the probe, and step 22 sizes the reconcile's own waits. The accepted unit is ordered before
`systemd-user-sessions.service` (the [proposal][proposal], lines 496 to 498), so the probe delays a login by its
own run time; [M4](#m4-what-the-probe-costs) measures that delay and [Limits](#limits) records that the probe does
not run in the host namespace.

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
`ExecStartPre=` running the probe; step 7's readiness record references the probe's record rather than owning it
(the [plan][plan], lines 762 to 779 and 825 to 829). Until step 19 lands, the probe runs from the guest's
transient test unit named under [M9](#m9-the-untested-release-and-the-refused-release), so the design does not
wait on the unit.

Every process that makes a raw ZFS request reads it before doing so: the wrapper's add preflight, where the
proposal already requires a readiness report (the [proposal][proposal], lines 351 to 352), the handler's
`boot-reconcile` mode, the handler's mount mode, and the handler's sweep mode that a ZED event starts (the
[plan][plan], lines 645 to 647). The wrapper's other invocations make no raw request and read nothing. The sweep
and the boot unit are independent triggers (the [proposal][proposal], lines 478 to 480 name both), so a sweep can
reach the record before the unit's probe has written it; an absent or stale record then refuses the attach and
keeps the intent, which is the decided behaviour for a missing readiness report (the [proposal][proposal], lines
351 to 352) and needs no new rule. The module gate binds both binaries (the [proposal][proposal], lines 235 to
237); the probe's verdict binds at least every attach path, and what else a failure disables is
[Q3](#q3-what-a-failed-arm-disables).

### What it changes at start

- One worker process, the mode process that forks it and enforces its deadline, a holder the mode forks, and an
  unmount child the worker forks, each short-lived.
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
finish. The next boot starts with an empty directory.

### What a refusal prints

The wrapper prints one line to standard error on the managed operation it refuses, naming the probe arm, the
running kernel release, the module version, the failed call and its errno, the route the slot will use instead,
and the `doctor` command that prints the last result. It never suggests a force flag, because the gate has none
and this probe adds none. Which operations a failed arm refuses is [Q3](#q3-what-a-failed-arm-disables).

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
the resulting verdict. The probe's own fields are written only by the mode process. The first real attach's
outcome, which [Way 4](#way-4-the-products-first-attach-as-the-probe) folds in, goes into a second file beside it,
`first-attach`, written by the handler process that made that attach, under the same atomic rename; each file has
one writer, so no lock is needed. The wrapper and `doctor` read both and write neither.

The record is not authority to act. It is a cache of an observation, and the wrapper rebuilds nothing from it
except the verdict and its wording, the way the design treats its other runtime files (the [proposal][proposal],
lines 555 to 556).

### What the result gates

A result with every arm passed enables managed attaches for that boot and module identity.

A result naming a failed attach, walk, move or unmount arm, or a failed namespace arm, is a real observation.
What it disables is the owner's answer to [Q3](#q3-what-a-failed-arm-disables); the options there state the effect
on unused slots, on boot reconcile, and on removal of mounted and unmounted clones.

A result naming an unexercised arm, because no source existed, every candidate was held, or the kernel refused the
exclusive create, is reported. Whether it enables the route is the owner's answer to
[Q5](#q5-whether-an-unexercised-arm-may-enable-the-route).

A result naming a missed deadline is written as timed out. Whether the route then refuses or continues is the
owner's answer to [Q6](#q6-whether-a-missed-deadline-fails-closed).

A module or kernel whose fingerprint changed since the record makes the record stale; the next boot's probe
replaces it, and an add in between reads stale readiness, which the preflight must report as not current rather
than as a pass.

### What the probe never does

It never mounts by path on an account-controlled tree, never runs the `zfs` program, never opens `/dev/zfs`,
never creates or destroys a dataset or snapshot, never promotes, never sets a property, never writes a
registration or a receipt, never runs in the host mount namespace, never runs from `doctor`, and never offers a
force flag. It never looks up a name inside `.zfs/snapshot`: opening that directory resolves the two components
`.zfs` and `snapshot` by name, and beyond them the probe reads the directory's entries with `getdents` and never
opens, stats or resolves an entry it finds, because such a lookup starts the automounter and can mount into a
namespace the probe does not control (`module/os/linux/zfs/zpl_ctldir.c:255` to `:277`,
`module/os/linux/zfs/zfs_ctldir.c:1190` to `:1198`). The mounts it makes live only in its own namespace.

## Owed measurements

Every item here is owed before the probe can be trusted. None ran for this proposal, and each runs on a
disposable file-backed pool inside a throwaway VM, never on this host's pool and never with its `/dev/zfs`.

The recommended shape needs M1, M2, M3, M4, M5, M7, M8 and M9, plus M10 where the owner answers
[Q4](#q4-whether-a-setup-time-probe-dataset-exists) yes. [Way 3](#way-3-an-account-probe-with-no-zfs) needs M2,
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
`reads`, `writes` and four times (`module/zfs/spa_stats.c:206` to `:214`).

The txg comparison is by row, not by the highest txg: an idle pool commits a txg on its own timer, every
`zfs_txg_timeout` seconds (5 by default, `module/zfs/txg.c:532` to `:553`), so the highest txg always moves and a
before-and-after comparison of counters proves nothing. Instead, take the first reading's highest `txg` and require
`ndirty`, `nwritten` and `writes` to be zero on every committed row whose `txg` is greater than that value; a row
in the open state has not committed yet, so its counters are not evidence either way. Read the second time soon
enough that the first reading's highest `txg` is still in the list, because the list keeps `zfs_txg_history` rows,
100 on this host (read-only `/sys/module/zfs/parameters/zfs_txg_history`); with the parameter at 0 the reading is
empty, which the design reports as unmeasured rather than as zero writes. The positive control is the same rule with a
deliberate write in a scratch dataset, which must leave a committed row with a `txg` greater than the cut and a
nonzero `ndirty`, `nwritten` or `writes`, so a harness that never reads the file fails its own control.

Controls, each fixture-only and never shipped: a deliberate mount in the guest's initial namespace, which must
show the mount-table difference the harness looks for; a deliberate write in a scratch dataset, which must leave
the nonzero committed row with a `txg` greater than the cut; and a deliberate `open` of an entry name inside
`.zfs/snapshot`, which must change the guest's initial mount table, because the automounter mounts through a
helper (`module/zfs/zfs_ctldir.c:1190` to `:1198`). The call is an `open`: `stat`, `lstat` and `fstatat` do not
automount (`statx(2)`).

Gates: [Way 1](#way-1-a-root-probe-on-a-live-snapshot), [Way 3](#way-3-an-account-probe-with-no-zfs) and
[Way 5](#way-5-a-setup-time-probe-dataset); also the claim that a probe leaves no pool write, the access-time
question on `.zfs/snapshot`, the snapshot-hold window, and the prohibition on lookups inside `.zfs/snapshot`.

### M3: interruption leaves nothing

Kill the probe with `SIGKILL` after each step: after `unshare`, after the tmpfs, after `fsconfig(create)`, after
`fsmount`, after `move_mount`, after the holder starts, after the move, and before the unmount. Kill each process
separately as well as the whole run: the mode process alone (the worker must then survive or die cleanly, and the
record must say what happened), the worker alone while the mode lives, the holder alone, and the unmount child
alone. Record the host mount table, every namespace's representative `mountinfo`, the slot, the move-target
directory, the record and its directory, whether any namespace still mentions the source after the kill, and
whether a second run succeeds. Control: the same probe allowed to finish, and a run killed before `unshare`
returns.

Gates: the claims that it leaves nothing in the host mount table and nothing outside the record's directory, the
started record, the parent's deadline path, and each child's `PR_SET_PDEATHSIG` and control pipe.

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
[Q7](#q7-whether-the-account-runs-a-companion-probe).

### M7: the namespace arm sees a known holder and reports only the probe's own namespaces

Start the probe's holder, require the scan to name its namespace with the matching source, stop the holder, and
require only the worker's namespace to remain. Repeat with the source already mounted in a second namespace
before the probe runs, and require the attach arm to answer `EBUSY` and the probe to record that name as held
instead of refusing spuriously. Repeat with a namespace pinned and no live member in it, and record that the scan
does not name it and reports the source clear: a clear report is not absence, which the document says wherever it
relies on the scan.

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

Gates: every way, and the acceptance item.

### M10: the setup-time dataset, if the owner adds one

If [Q4](#q4-whether-a-setup-time-probe-dataset-exists) is answered yes, measure the setup command's creation and
its idempotent second run, the space it takes, that nothing else mounts it, that the sweep and the destroy-anyway
path never see it, that the sanoid configuration excludes it, and what the probe records when the dataset or its
snapshot is missing (`ENOENT` from `fsconfig(create)`). Control:
[Way 1](#way-1-a-root-probe-on-a-live-snapshot)'s source on the same build, so the difference in exercised arms is
a measured one.

Gates: [Way 5](#way-5-a-setup-time-probe-dataset) only.

## Questions for the owner

Each question is separable from the others, each option states its costs, and the ranking gives the reason for
every adjacent pair. The order is the order of dependence, so an answer never needs a later question's answer:
the plan reading, then the start, then what a failure disables, then the source, then the two questions that
inherit the source and the failure, then the companion. Each question opens with a concrete case and describes
every option in words rather than by a section name, so it can be read and answered in the question tool without
this document beside it. Where an answer changes the plan or a decision, the question says so.

### Q1: whether the plan bars the probe's unmount arm

The plan records that "No production kernel request runs before the first two, and no unmount runs before the
third" (the [plan][plan], lines 82 to 88), and the third is the staging design that step 9 builds. The case: at
boot, before any clone exists, the probe mounts a read-only snapshot inside a mount namespace of its own, moves
that mount to a directory inside its own tmpfs and unmounts it there. Nothing of the account's tree is touched
and the product's staging folder is not used. Does the plan's sentence forbid that arm until step 9?

- Production-only (Recommended). The probe's unmount arm is private to the probe and runs from step 6, because
  the sentence is about the product's mounts and the product's unmount does not exist yet. Pros: the evidence that
  a second process can move and unmount a mount arrives with the rest of the probe, and step 9 is not on the
  critical path twice. Cons: a reader of the plan's sentence has to accept this reading, and the arm can never
  prove what the product's own unmount must do.
- Every unmount, including the probe's. The probe's move and unmount arms, and the namespace arm that judges them,
  move from step 6 to step 9. Pros: the sentence keeps its literal scope, and the probe's unmount would exercise
  the product's own unmount code. Cons: step 6 ships a start gate that never exercises the unmount half of the
  route the answer names, and step 9 grows a second consumer of its staging design.

Ranking: production-only, then every unmount, because the probe mounts nothing the account or the checkout owns
and the product's unmount is a different subject; the cost of the second option is a step 6 gate whose unmount
evidence waits for a later step, with no other evidence gained.

### Q2: which start the probe covers

The answer's words are "Probe at start", and the module gate reads that as the start of each process that will
make raw requests (the [proposal][proposal], line 235). A functional probe needs root and mount calls, so it
cannot be that. The case: a person reboots, logs in and runs a managed add. Before the login is admitted, the boot
unit's `ExecStartPre=` has run the probe, and the add reads the record and finds the kernel, module and boot it
names are the running ones.

- Boot only (Recommended). One root process per boot, one implementation, no new trigger path, and no
  per-invocation cost. A record whose fingerprint no longer matches the running kernel, module or boot is stale,
  and the managed route then refuses the attach and keeps the intent (the [proposal][proposal], lines 351 to 352).
  Cons: on a host where the module could be reloaded without a reboot, the record would stay stale until the next
  boot.

Ranking: boot only, and no rerun trigger is offered as an alternative, because a module cannot be reloaded on this
host without a reboot: the root filesystem is on ZFS and the module is in use, so a module change arrives with a
reboot that runs the probe again ([The kernel the design names](#the-kernel-the-design-names)). A trigger keyed to
a module load is a mechanism this host cannot exercise; if the owner wants one anyway, it is a new question and a
new mechanism, because it would add a second writer of the record and a probe that can race an attach. The probe
does run again when the checkout provider is restarted, since this unit requires that provider
([Who runs it, and where](#who-runs-it-and-where)).

### Q3: what a failed arm disables

An arm that ran and failed is a real observation, and what it disables is open. The arms are: the walk to the
slot, the attach, the move and the unmount, and the namespace scan. The case: after a kernel package update the
probe's attach arm fails with `EPERM`. A person then runs an add for a worktree whose slot has no clone yet, and
later removes a clone whose worktree they no longer need. Under the recommended option the add takes the reflink
or ordinary Git route and prints one line, and the removal succeeds for a clone that is not mounted. Under the
second option the add takes the same fallback but both kinds of removal are refused until the kernel changes.
Under the third the add proceeds into the attach the probe just saw fail.

Every option that refuses attaches leaves existing unmounted clones unmounted until the route returns, which is
the availability cost the allowlist answer was chosen against: the project kept a functional probe instead of an
exact release allowlist because an allowlist would disable the route after every kernel update (the
[proposal][proposal], lines 1294 to 1306). The option that refuses nothing loses that protection instead.

- Refuse new managed attaches, keep every other operation (Recommended). Unused slots take the same-dataset
  reflink or ordinary Git route, which needs no ZFS attach. Boot reconcile attaches nothing new and keeps every
  intent and registration. Removal of a clone that is not mounted stays available through the module-gated
  ioctls. Removal of a mounted clone needs an unmount and the scan of every task's namespace, so when the failed
  arm is the move, the unmount or the scan, that removal is refused and reported rather than silently degraded.
  Pros: the recovery path that needs no attach stays available, which is exactly when a person needs it. Cons: the
  managed route is half available, and a reader must know that attach and removal are gated separately.
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

### Q4: whether a setup-time probe dataset exists

The probe needs a ZFS source to attach, and today the only candidate is a snapshot of the checkout, whose name the
account can create (the [proposal][proposal], lines 319 to 321) and which may not exist yet or may already be
held. The alternative is one small dataset created once by root: `zpcachyos/nosnap/git-worktree-probe`, with one
empty snapshot named `ready`, sitting under the unmounted container `zpcachyos/nosnap` that the sanoid
configuration already marks `autosnap = no`. Its properties are `mountpoint=legacy`, `canmount=off`,
`com.sun:auto-snapshot=false` and a user property naming it as the probe's source, so nothing mounts it
automatically and no recursive sanoid ancestor reaches it. The account holds no grant there, so it can neither
create, mount, rename, snapshot nor destroy it, and the sweep that lists the clone parent's children never sees
it. The probe notices its absence by trying to attach it: a missing dataset or a missing `ready` is the `ENOENT`
from `fsconfig(create)`, reported as an unexercised arm and never created.

The case: a fresh host, migrated and rebooted, before any clone exists. Without the dataset the probe finds no
unheld checkout snapshot, records the attach arm as not exercised, and the first add depends on the answer to
[Q5](#q5-whether-an-unexercised-arm-may-enable-the-route). With the dataset the probe attaches `ready` on every
boot, and the attach arm always runs.

- Yes, the setup-time probe dataset (Recommended). It changes what the setup command that `doctor` prints creates,
  `doctor`'s own filesystem section, the boot unit's `ExecStartPre=`, and the sanoid sections with their guard
  report: steps 18, 19 and 20 of the plan. Pros: the source always exists, nothing else mounts it, every arm runs
  on every boot, the attach's exclusive create proves a fresh superblock, no account-influenced name is read, and
  no directory read happens at start. Cons: a new object in the layout, with a sanoid section, a configuration
  field, preflight acceptance and a way to go missing or drift, which the probe reports as an unexercised arm.
- No, a snapshot of the checkout (the design's Way 1). Pros: no layout change, nothing new in the sanoid
  configuration, the sweep or preflight. Cons: the source can be absent or already held, so the arms can be
  unexercised on a fresh host and while any other namespace holds that snapshot, the name is account-influenced,
  and the one directory read may update its access time.

Ranking: yes, then no, because a source that always exists and that nothing else mounts is what makes every arm
run and removes the reuse and influence questions at their root; the layout cost is one object with a named
place and its exclusion, while the cost of no is arms that silently do not run. This design ships Way 1's source
until the owner answers yes, so the answer decides the source and nothing else.

### Q5: whether an unexercised arm may enable the route

An arm is unexercised when it could not run, not when it ran and failed. The case: a fresh host whose probe found
no unheld source, so its attach arm is recorded as not exercised. A person runs their first managed add. Under
the first option the add proceeds and prints a line saying which arm did not run, and `doctor` repeats the gap.
Under the second the add is refused with that same line and nothing else can attach until a source exists.

The causes differ by source. With a checkout snapshot the attach arm can be unexercised because no snapshot
exists or because other namespaces hold every candidate. With the setup-time dataset of
[Q4](#q4-whether-a-setup-time-probe-dataset-exists) it is unexercised only when the dataset or its `ready`
snapshot is missing, and on a kernel older than 6.6, which refuses the command that proves the source is fresh.

- Reported, and the route stays enabled (Recommended today). Pros: a host with no snapshot keeps a working route,
  the answer's "admits untested releases" posture holds, and `doctor` names the gap. Cons: the arms the answer
  exists to exercise may never run, so a broken kernel passes with the gap visible only in `doctor`.
- An unexercised arm blocks the managed route, with the same per-operation effects
  [Q3](#q3-what-a-failed-arm-disables) states. Pros: the route is only ever enabled by evidence. Cons: with a
  checkout snapshot a fresh host has no source until the first add, and the first add needs the route, so the
  route could never start; with the setup-time dataset that objection goes away, which makes this the stronger
  answer if [Q4](#q4-whether-a-setup-time-probe-dataset-exists) is answered yes.

Ranking: reported and enabled, then blocked, because a source that can be absent on a fresh host turns blocking
into a boot-time lockout; if [Q4](#q4-whether-a-setup-time-probe-dataset-exists) is answered yes, this ranking
reverses, because the source then always exists and the route can be gated on evidence without a lockout. The
recommendation marked in the first option is therefore the one for today's source, and the owner who answers
[Q4](#q4-whether-a-setup-time-probe-dataset-exists) yes should expect to prefer the second option here.

### Q6: whether a missed deadline fails closed

The mode process enforces a deadline over the worker and records a timeout either way. The case: a kernel whose
attach blocks in the kernel, so the probe never finishes. The login the boot unit orders before is admitted late,
the record says timed out, and the next add either refuses and prints why, or proceeds on the strength of a record
that says the probe could not finish.

- Fail closed (Recommended). A timeout leaves no pass, so the record is not current, adds are refused, existing
  bindings and intent are kept, and the per-operation effects are [Q3](#q3-what-a-failed-arm-disables)'s
  recommended ones. Pros: a kernel that hangs an attach neither blocks a login indefinitely nor silently enables
  the route, and it matches the gate's posture of no force flag. Cons: a slow but working kernel loses the route
  until a timing measurement sizes the deadline ([M4](#m4-what-the-probe-costs)).
- Fail open, with a warning. Pros: availability is never lost to a slow boot. Cons: an attach that hangs is
  attempted on the strength of a record that says the probe could not finish, which is the failure the answer
  exists to catch.

Ranking: fail closed, then fail open, because the probe's whole value is that a missing pass is not a pass, and
the cost is a deadline that measurement can size.

### Q7: whether the account runs a companion probe

The companion is a second, smaller check that runs as the account, in a user and mount namespace, and exercises
the walk, a tmpfs mount and unmount, and the refusal controls, without touching ZFS (the design's Way 3). The
case: the account runs an add on a boot where the probe passed. Under the first option nothing more runs and the
add reads the record the boot wrote. Under the second the add first runs the companion and fails early, with a
line naming the walk or the mount call, before any root work starts.

- Root only (Recommended). Pros: one implementation, no user-namespace dependency, and the arm that matters needs
  root anyway. Cons: a syscall or namespace regression is seen only at the next boot, never before a given
  invocation.
- Root plus an account companion. Pros: a per-invocation check of the walk, the mount surface and the wrapper's
  helpers before any root act, and the wrapper's tests already carry the skip control. Cons: a second
  implementation, a check that never touches ZFS, and a skip exactly on hosts where user namespaces are refused.

Ranking: root only, then the companion, because the account arm cannot observe the dataset mount path, which is
the failure the answer asks about, and its per-invocation timing is not yet measured.

## What a writer settles without asking

- The arm order, the private-namespace setup before any mount, and every syscall the probe uses.
- The scratch location: the record's directory under `/run`, the tmpfs mounted at its scratch mount point, the
  slot, the move-target directory and the refusal tree inside that tmpfs, and that the probe never touches the
  account-owned staging folder or the checkout's own directories.
- The source selection rule: the first accepted candidate in the order the read returns, a valid component name,
  the dataset-name bound, trying the next name when the exclusive create answers `EBUSY`, and the not-exercised
  record when no candidate qualifies.
- That the holder is forked by the mode process and the unmount child by the worker, that each child carries
  `PR_SET_PDEATHSIG` set to `SIGKILL` and a control pipe, and that the mode kills the worker's process group at
  the deadline.
- That the holder starts before the move and attaches its own mount of the same source.
- That the namespace arm's holder runs at every start, with its cost under [M4](#m4-what-the-probe-costs), and
  that the scan is judged against the probe's own namespaces rather than against an empty set.
- That the unmount is by path inside the probe's own tmpfs, and that the probe therefore does not validate the
  product's held-descriptor unmount. Whether the plan's sentence bars that arm at all is
  [Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm), not a reading this design settles.
- That the probe reads no file content, only `fstatfs`, a bounded directory listing on its own mount, and, under
  [Way 1](#way-1-a-root-probe-on-a-live-snapshot) only, `getdents` on `.zfs/snapshot`.
- The mount flags, the read-only request, the exclusive create as the freshness oracle, and the `EBUSY` handling
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures.
- The record's directory, mode, fields and atomic-rename write, the `first-attach` file beside it, and the started
  record before the fork.
- The wording of the wrapper line, the `doctor` section and the journal line, under the design's own reporting
  rules (the [proposal][proposal], lines 755 to 762).
- That `doctor` reads the result and never runs a probe, and that no force flag exists.
- That the probe is a handler mode named `start-probe`, run as `ExecStartPre=` of the existing unit, that it runs
  the module gate first and skips itself when the gate refuses, that the mode process enforces the deadline and
  always exits successfully, and that no on-demand channel is added.
- The deadline's value, set only after [M4](#m4-what-the-probe-costs) measures the arms; the unit carries no
  start timeout, which keeps its `ExecStart` unbounded until step 22 sizes the reconcile.
- The fallback for an unused slot and the preservation of existing bindings, which the accepted design already
  decides.

## What this proposal does not change

It does not introduce an allowlist, does not withdraw or loosen the module gate, does not authorize a path mount
or a new request channel, does not add a force flag, does not make the probe a `doctor` action, does not create
or destroy a dataset at start, does not change the intent vocabulary, does not move the handler's mount and
unmount modes out of the host mount namespace, does not add a pool object of its own (the setup-time probe dataset
is the owner's act, printed by `doctor` and put as
[Q4](#q4-whether-a-setup-time-probe-dataset-exists)), does not change any removal, promote, staging or delegation
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
  `module/os/linux/zfs/zpl_ctldir.c`, `module/os/linux/zfs/zfs_ioctl_os.c`,
  `module/os/linux/spl/spl-zone.c`, `module/zfs/zfs_ioctl.c`, `module/zfs/zfs_ctldir.c`,
  `module/zfs/dmu_objset.c`, `module/zfs/dsl_crypt.c`, `module/zfs/dsl_dataset.c`,
  `module/zfs/spa_stats.c`, `module/zfs/txg.c` and `cmd/zfs/zfs_main.c`.
- The kernel header and source named under [The kernel headers and source](#the-kernel-headers-and-source): this
  host's `/usr/include/linux/mount.h` (linux-api-headers 1:7.1-1), the 7.3.5 tree's
  `/usr/src/linux-cachyos/include/uapi/linux/mount.h`, Linux v6.8 and v7.3 `fs/super.c`, and Linux v7.3
  `fs/fsopen.c`, with `fs/fsopen.c` fetched at v5.2, v5.11, v5.12, v5.15 and v6.0 to v6.7 as well.
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
