# The start probe for the ZFS worktree route

**Non-final, unapproved proposal by an agent.** This document designs one open item of the accepted ZFS worktree
design, "The start probe a kernel-release answer asks for", so that step 6 of the implementation plan can be
written. A Pi session wrote it on 2026-11-19 from the sources listed under
[Sources read](#sources-read), without running a VM, a probe or any pool command, and it is not accepted. What it
checked: the accepted design, the plan and the measurements record, OpenZFS `zfs-2.4.4` sources, the running
kernel's exported system calls, and the wrapper's own code. What stays open: the four owner questions under
[Questions for the owner](#questions-for-the-owner), and every measurement under
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

- The start-time module gate stays exactly as decided, in the [proposal][proposal], lines 238 to 244:

  > At start, before the first ioctl, both binaries must prove compatibility with the loaded module and target
  > ABI, including module release, architecture, endianness, request contract and kernel/module BTF layout. A
  > missing device, missing required BTF or mismatch disables raw requests, including reads; no force flag. ...
  > On gate refusal the wrapper uses the applicable same-dataset reflink or ordinary Git route for an unused
  > slot. ... Report what failed, including when an update makes existing clones temporarily unavailable. A
  > failed gate does not authorize an unrelated mount or cleanup; keep registrations and durable recovery state
  > intact.

  The probe adds a functional check beside that gate and replaces none of it; the quoted decision covers the ZFS
  ioctl device and layout, not a kernel-release list (the [proposal][proposal], lines 617 to 620).

- The unselected allowlist's four qualifying arms name what the answer was chosen against: "built handler
  attach, staging unmount, refusal and namespace controls on that release" (the [proposal][proposal], lines 620
  to 623). They are the shape of the functional evidence the probe is expected to seek, though the answer does
  not bind the probe to all four.

- The probe admits untested releases: "The probe admits untested releases and leaves unmeasured behaviour to
  appear on the host" (the [proposal][proposal], lines 629 to 630). A partial pass is therefore not a defect of
  this design, provided the missing piece is reported.

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
  [proposal][proposal], lines 152 to 153). A probe cannot be a start-time requirement for every command.

- Each add requires a current readiness report: "Require a current handler readiness report for the actual pool,
  checkout and boot, not just a present executable" (the [proposal][proposal], lines 352 to 354). The probe's
  result can live in that record, which the wrapper already must read.

- The build owes one acceptance test for it: "Run the built probe against an untested release and against one it
  must refuse, and record what it changes at start, what it leaves behind and what it prints" (the
  [proposal][proposal], lines 1841 to 1843). Step 6 also owes two VM arms of its own: "an untested release, and a
  module whose field layout differs from the approved one, each asserting that raw requests are disabled" (the
  [plan][plan], lines 739 to 745).

- No second request channel is added for mounting: the design keeps "the project's ZED handler for in-session
  requests **plus** a systemd unit for boot and shutdown, not a second request channel for mounting" (the
  [proposal][proposal], lines 478 to 480), and fixes the intent vocabulary to `v1:<nonce>:mount` and
  `v1:<nonce>:unmount` (lines 536 to 539). A probe cannot be triggered by a third intent or a new channel, which
  is why this design places it in the existing boot unit.
- The handler's mount and unmount modes "Run both modes in the host mount namespace" (the [proposal][proposal],
  line 496), and this proposal keeps that. The probe is not one of those modes: it attaches nothing for a
  worktree, it runs in a child mount namespace of its own, and the limit that it therefore cannot prove the
  host's propagation is stated under [Limits](#limits).
- The plan files this item as "step 6, blocking, writer design work" (the [plan][plan], lines 2200 to 2203), and
  lists the start probe among "The three blocking designs, as documents ... the critical path for steps 6 and 9"
  (the [plan][plan], lines 2102 to 2107).

## Facts

### The design's own statements

- The start-time gate runs in both binaries "At start, before the first ioctl" (the [proposal][proposal], line
  239). This proposal reads "start" as the start of a process that will make raw requests, and the boot unit's
  `ExecStart` is one such process (the [proposal][proposal], lines 500 to 505), which is where this design places
  the probe; the wrapper's own start keeps the module gate alone.
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
- The wrapper has no ZFS code today: `package/zfs` does not exist in this checkout, and no file under
  `package/git-policy/cli/src` calls `fsopen`, `fsconfig`, `fsmount`, `move_mount`, `unshare` or `setns`
  (repository search, 2026-11-19). The probe is new code, not a change to existing wrapper behaviour.

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
  Only a process in the init user namespace can attach a dataset there.
- The fs_context path takes `fc->source` as the dataset name, holds that objset, and calls `sget_fc` keyed on it;
  it calls `zfs_domount` only when the returned superblock has no root yet
  (`module/os/linux/zfs/zpl_super.c:850` to `:920`). A second attach of an already mounted dataset therefore
  reuses the existing superblock and skips the dataset mount work, including the key check.
- A read-only request against an existing read-write superblock refuses with `EBUSY`, except for a snapshot
  (`module/os/linux/zfs/zpl_super.c:931` to `:937`).
- The fs_context accepts `source` and the negatable `exec`, `suid` and `dev` options
  (`module/os/linux/zfs/zpl_super.c:579` to `:583`, applied at `:669` to `:676`).
- The fs_context operations table wires `parse_param`, `get_tree`, `reconfigure` and `free`
  (`module/os/linux/zfs/zpl_super.c:1022` to `:1032`). `zpl_get_tree` sets the context's root dentry
  (`:937` to `:945`), which is what `fsmount` turns into a mount and `move_mount` then places.
- The dataset mount path in the kernel makes no ZFS ioctl request: `zpl_get_tree` calls `zfs_domount`, which calls
  `zfsvfs_create` (`module/os/linux/zfs/zfs_vfsops.c:1347` to `:1370`). A probe on this path therefore needs no
  `/dev/zfs`, and it proves nothing about the device or the ioctl contract.
- Delegation is checked for clone creation, not for the kernel's attach: the clone policy requires `clone`,
  `create` and `mount` on the parent (`module/zfs/zfs_ioctl.c:1140` to `:1149`), while `zpl_get_tree` and
  `zfs_domount` check no delegation. A root-run attach therefore proves the kernel path, not the account's
  grants.
- An attach of an encrypted dataset with an unavailable key refuses at `fsconfig(create)` with `EACCES`
  ([measurements][measurements], item 5, line 231), which is the observable that shows `zfs_domount` ran.
- A mounted dataset's `.zfs/snapshot` directory lists its snapshot names through `dmu_snapshot_list_next`, with
  no ioctl and no delegation check (`module/os/linux/zfs/zpl_ctldir.c:281` to `:320`). A root process can
  therefore name a snapshot for the probe by reading a directory, without `/dev/zfs`.
- The CLI's "cannot mount: legacy mountpoint" refusal is the userspace branch for an explicit `zfs mount`
  (`cmd/zfs/zfs_main.c:7239` to `:7247`), not the fs_context path; item 10 measured that branch and the
  descriptor attach beside it ([measurements][measurements], item 10, lines 359 to 375).

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
- Unprivileged user namespaces are enabled here: `/proc/sys/user/max_user_namespaces` reads `252369` and
  `/proc/sys/kernel/unprivileged_userns_clone` reads `1` (read-only, 2026-11-19).
- The design's checkout dataset does not exist yet on this host: `findmnt --target /home/user/Monochromatic`
  resolves to `/home` ([measurements][measurements], line 31). A probe on this host today would have no managed
  ZFS source to attach.

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
  (lines 243 to 245).
- Item 7 measured `nosuid` stopping setuid elevation against an unrestricted positive control
  ([measurements][measurements], item 7, lines 283 to 300).
- Item 15 measured a root scan grouping tasks by mount namespace, finding both post-attach holders, and finding
  none after they stopped ([measurements][measurements], item 15, lines 612 to 660). It recorded that a
  process-visible scan is not proof of complete absence.
- Item 16 recorded that no built-in or local image booted `7.3.5-1-cachyos` with OpenZFS `2.4.4`, and that the
  remote catalog was not queried ([measurements][measurements], item 16, lines 668 to 706).
- The guest every measured item ran on is `6.8.0-147-generic` with OpenZFS userspace and module `2.4.4-1`
  ([measurements][measurements], lines 37 to 39, 669 to 673).

### The wrapper's code

- The no-crossing walk already exists: `RESOLVE_NO_XDEV`, `RESOLVE_NO_MAGICLINKS`, `RESOLVE_NO_SYMLINKS` and
  `RESOLVE_BENEATH` are defined at `package/git-policy/cli/src/native/subvolume/fs_beneath.rs:44` to `:54`,
  `ObjectId` at `:80`, the resolve mask at `:163`, `open_root` at `:184` and `open_directory_at` at `:203`. The
  wrapper already calls `libc::syscall` for `openat2` (`:167`), so the probe's other syscalls need no new
  dependency.
- Mount facts are already read from `/proc/self/mountinfo`: `MOUNTINFO_PATH` at
  `package/git-policy/cli/src/native/subvolume/fs_mount_options.rs:37`, `MountFacts` at `:52` and
  `read_mount_facts` at `:339`.
- A boundary walk that refuses a nested mount and names it already exists: `find_boundary_below` at
  `package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs:465` and its `_except` form at `:483`,
  with `device_of` at `fs_beneath.rs:244` and `handle_path` at `:307`. The probe's mount-crossing refusal control
  can reuse them.
- The `doctor` filesystem report is the section to extend: `SECTION_MARKER` at
  `package/git-policy/cli/src/native/health/doctor_filesystem.rs:51`, `gather_facts` at `:227`,
  `render_section` at `:366` and `checkout_section_with` at `:455`.
- The report's read ceiling is `REPORT_SCAN_CEILING`, 5 s, at
  `package/git-policy/cli/src/native/service/wait_ceilings.rs:1280`, measured on this host on 2026-11-09
  (`:1250` to `:1279`). The design calls it a guest trial budget, not a shipped mount limit (the
  [proposal][proposal], lines 750 to 752).
- A cached start check already exists: `binary_identity`, `read_version_text` and `store_version_text` in
  `package/git-policy/cli/src/native/invocation/git_version_cache.rs:1` to `:23` (module doc), `binary_identity` at
  `:91`, `read_version_text` at `:238` and `store_version_text` at `:283` key a per-user cache on the
  executable's canonical path, size, times, device and inode.
- The Git check is the precedent for a cheap start check: `MINIMUM_GIT_VERSION` at
  `package/git-policy/cli/src/native/invocation/git_version.rs:52`, one version question per invocation, a
  warning and continued command on an older Git, and a stop only when the version cannot be read
  (`package/git-policy/cli/SPEC.md:18651` to `:18680`).
- The wrapper's tests already run an unprivileged private mount namespace and skip with a stated reason where
  user namespaces are refused: `a_private_mount_namespace_is_available` and
  `inside_a_private_mount_namespace` in
  `package/git-policy/cli/src/native/binary/binary_worktree_removal_around_a_volume_tests.rs:159` to `:200`.

## Assumptions

Each assumption is stated so a measurement can break it, and the measurement that would is named.

- A private mount namespace made by root from the init user namespace can attach a ZFS dataset, because
  `zfs_domount` sees the init user namespace and `zone_dataset_visible` returns visible and writable there
  (`module/os/linux/spl/spl-zone.c:297` to `:307`). The measurement that breaks or confirms it is
  [M2](#m2-the-host-mount-table-is-unchanged).
- Once the probe's namespace has been made private, its attaches and unmounts do not reach the host mount table.
  This is reasoning from mount propagation, not a measurement in this design's shape.
  [M2](#m2-the-host-mount-table-is-unchanged) measures it, and item 3 shows a shared parent does
  propagate and refuses the staging move ([measurements][measurements], item 3).
- Attaching a snapshot of the checkout exercises `zfs_domount` and therefore the key check, while attaching the
  already mounted checkout dataset skips both. The measurement is
  [M1](#m1-which-source-exercises-the-dataset-mount-path), whose control is the unloaded key from item 5.
- A snapshot attach adds no pool history entry and changes no dataset property. This is an inference from the
  kernel's mount path having no history call (`module/os/linux/zfs/zfs_vfsops.c:1347` to `:1490`), and
  [M2](#m2-the-host-mount-table-is-unchanged) reads pool history before and after.
- A killed probe leaves nothing in the host mount table and nothing on disk, because every mount it makes lives
  in a namespace that dies with it and every file it writes lives on a tmpfs in that namespace. The measurement
  is [M3](#m3-interruption-leaves-nothing).
- The account cannot attach any ZFS source, even inside a user namespace, so every ZFS arm of the probe runs as
  root. This rests on the `EPERM` check, the init-namespace test and the uid 0 attach check named under
  OpenZFS at `zfs-2.4.4`, together with
  the design's own H1 gate 7 expectation (the [proposal][proposal], lines 1940 to 1942); the measurement is
  [M6](#m6-user-namespace-limits), and it also states the confined case rather than only asserting it.
- The probe's cost is small enough for the boot unit to pay on every boot of this host. No measurement exists,
  and this design refuses to state a duration; [M4](#m4-what-the-probe-costs) measures it against a control arm
  of the same binary.
- The four allowlist arms can be exercised without creating a clone, using a read-only snapshot as the attach
  source. The clone's exact shape (its encryption root and its slot properties) is then not exercised, which is
  the clone-shape limit under [Limits](#limits); [M1](#m1-which-source-exercises-the-dataset-mount-path) and
  [M8](#m8-the-real-clone-attach-as-evidence) measure what the snapshot arm does and does not cover.

## Limits

- The probe cannot prove host-namespace propagation, a clone's real slot, or ZED ordering. Only the handler's
  actual attach at a real slot does that, which is why [Way 4](#way-4-the-products-first-real-attach-as-the-probe)
  keeps its evidence and why [M8](#m8-the-real-clone-attach-as-evidence) measures it.
- The probe's verdict is a property of the running kernel and module build. A module reload, a livepatch, or a
  container runtime with a different namespace setup can change the answer after the record was written.
- The namespace arm proves only what a process-visible scan can see. Pinned namespaces without a live member and
  races stay unproven, as item 15 records ([measurements][measurements], lines 645 to 652).
- The probe proves the syscall surface on the build it ran on, not on every newer build than the
  stated syscall floor.
- A probe refusal names an arm and an errno. It cannot say whether one particular clone would attach, and it is
  not a substitute for the module gate's device and layout checks.
- The probe reads no file content from the attached source; it proves the mount through `fstatfs`, a bounded
  directory read and mount facts. A source whose root directory is unreadable to root is therefore not detected
  by content.
- A probe that holds a snapshot for its duration can make a concurrent `zfs destroy` of that exact snapshot
  refuse `EBUSY`; [M2](#m2-the-host-mount-table-is-unchanged) records whether that window is observable.
- If the project chooses a pool-writing arm ([Way 2](#way-2-a-root-probe-on-a-disposable-dataset)), the probe
  inherits `ENOSPC` at boot, cleanup after interruption, and the handler's sweep having to ignore a probe clone.

## Ways to meet the open item

Four distinct ways follow. Each states who runs it and where, what it exercises, what it changes at start, what
it leaves behind, what a refusal prints, and its costs. Cost is stated as the work to write and to test, never as
a duration, because no measurement of this work exists. None of them changes a decision of the accepted design.

### Way 1: a root probe on a read-only live ZFS source in a private mount namespace

It runs as root from the handler binary, as a new mode beside `boot-reconcile`, inside a fresh private mount
namespace it creates and destroys itself. It makes the namespace's mounts private, mounts a tmpfs for scratch,
walks from the checkout root to a probe slot with the wrapper's `openat2` walk, attaches a read-only checkout
snapshot by descriptor (`fsopen`, `fsconfig` with the snapshot name and `nosuid,nodev`, `fsconfig(create)`,
`fsmount` with `nosuid,nodev,rdonly`, `move_mount` onto the held slot descriptor), reads a bounded directory
listing and `fstatfs` there, moves the mount to a probe staging directory and unmounts it from a second process,
then exercises the refusal walk (a planted link, an escape) and the namespace scan (a forked holder in a sibling
namespace, then its exit). It prints one line naming the arm and errno on refusal, and writes the result into the
readiness record.

What it changes at start: one private mount namespace, one tmpfs, one transient superblock for the snapshot, one
transient mount, one forked child per arm that needs a second process, a bounded read, and one root-owned record.
It creates no dataset, snapshot, bookmark, property or file outside its tmpfs, and it makes no ZFS ioctl call.

What it leaves when it fails or is interrupted: nothing on disk and nothing in the host mount table, because the
namespace and the tmpfs die with the process; a killed probe leaves either no record or a started marker with the
fingerprint, and the next boot reruns it.

For it: it exercises the kernel's real dataset mount, key check, descriptor placement, flag enforcement, staging
move, unmount from a fresh process, refusal and namespace scan on the running module and kernel, which are the
allowlist's four arms with no clone and no pool write. It runs before the first real attach, on a host with no
clones yet, and it reuses the handler's own attach and unmount code, so it fails where the product would fail.

Against it: it needs root, so the wrapper cannot run it per invocation; it cannot use the host namespace, so it
cannot prove the host's propagation; its source is a snapshot, so a clone's slot properties and encryption root
are not exercised; and it must find a snapshot, which today's host does not have in the checkout dataset.

Build cost: a new handler mode, the namespace and scratch setup, four arms, a record block and a `doctor`
section, all on `libc::syscall` with the existing walk and mount-fact helpers. Test cost: device-free tests for
the arm decisions, the record and the wording, and every live arm only in the disposable VM; the owed
measurements under [Owed measurements](#owed-measurements).

### Way 2: a root probe on a disposable dataset

It runs as root from the handler binary and does everything Way 1 does, plus it creates a probe snapshot and
clone under a reserved parent on the live pool, attaches the clone read-write at a probe slot, fills nothing,
then unmounts and destroys both. It proves the clone shape itself, including its inherited properties and its
encryption root.

What it changes at start: the pool gains a snapshot, a clone, a mount and a destruction, every boot, plus the
record. What it leaves when it fails or is interrupted: a probe snapshot or clone that survives until the next
start or until a person removes it, and a pool that may be out of space at the moment the probe needs to write.

For it: it is the only way that exercises a real writable clone, independent of live worktree state and of
whether any snapshot exists.

Against it: it writes the pool at start, needs grants on a second parent outside the handler's pinned clone
parent (or a reserved name the handler's sweep must learn to skip), competes with sanoid and the space guard, can
strand a probe clone, and puts a write on the pool precisely when the pool's health is what the start check is
for. `doctor` is forbidden to create a test dataset (the [proposal][proposal], lines 1153 to 1155), so this arm
could never serve a `doctor`-triggered check.

Build cost: Way 1 plus dataset lifecycle, a reserved parent, a sweep exclusion and cleanup after interruption.
Test cost: the highest of the four, because teardown, `ENOSPC` and stranded-state controls all have to run in
the VM.

### Way 3: an account probe of the mount surface without ZFS

It runs as the account, per invocation or on a first managed use, in a private user and mount namespace made the
way the wrapper's tests already make one (`unshare --user --map-root-user --mount`, or the same syscalls in
process). It mounts a tmpfs, walks to a probe slot, `fsmount`s the tmpfs, moves it onto the held descriptor,
moves it to staging and unmounts it from a second process, exercises the refusal walk, and exercises a holder in
its own child namespace.

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

### Way 4: the product's first real attach as the probe

It runs no separate check. The handler's first real attach after boot is already the functional evidence, and its
outcome is recorded in the readiness record; later managed attaches read that record.

What it changes at start: nothing extra. What it leaves when it fails: exactly what a failed first attach leaves,
a claimed and registered slot that needs the design's recovery.

For it: zero extra work, the exact topology (a real clone, its slot, the host namespace, ZED ordering) and no
synthetic source.

Against it: it runs after the first attach, which is the attach the answer wants qualified, so it cannot refuse
it; it needs a clone to exist, so a fresh host never probes; and it reports an operation's failure as a
compatibility verdict, which mixes two different questions.

Build cost: a record block only. Test cost: one VM arm on the built handler, plus the existing add controls.

## Ranking

Ranking: Way 1 > Way 3 > Way 4 > Way 2.

Way 1 ranks first because it is the only way that exercises the real dataset mount path at start, on a fresh
host, with no pool write and no change to a live worktree, so it can refuse before the first managed attach.

Way 3 ranks second because it is the only way a process that is not root can run, and it catches a regression in
the syscall surface and in the wrapper's own walk before any root act; it ranks after Way 1 because it never
exercises `zfs_domount` or the key path, which is the part of the answer's question that a kernel release most
often changes.

Way 4 ranks third because its evidence is the strongest per unit of work (a real clone in the host namespace), and
it is nearly free; it ranks after Way 3 because it arrives after the attach it would have qualified and cannot
run on a host without clones.

Way 2 ranks last because it is the only way that writes the live pool at every start; it buys the clone shape,
which Way 1 plus Way 4's recorded real attach cover without a write.

The recommended shape is Way 1, with Way 4's record folded in free (the first real attach's outcome joins the
readiness record, and no new mechanism), and Way 3 considered only if the owner wants a per-invocation companion.

## The proposed probe

This section proposes Way 1 in the detail the open item asks for. Every part of it is a writer's proposal inside
an undesigned item, and the questions under [Questions for the owner](#questions-for-the-owner) can change parts of it.

### The arms

1.  Namespace and scratch. As root, `unshare(CLONE_NEWNS)` into a new mount namespace, make every mount in it
    private (`mount_setattr` with `AT_RECURSIVE` and `MOUNT_ATTR_PRIVATE`, or `MS_REC|MS_PRIVATE` on `/`), and
    mount a fresh tmpfs for scratch. This step runs before any other mount, so a SIGKILL before it
    completes leaves nothing that could have propagated.
2.  Walk. Open the checkout root, walk to a probe slot under it with the wrapper's no-crossing `openat2` flags,
    and record the slot's `ObjectId`. Refusal controls run in a scratch tree the probe owns: a symbolic link at a
    component must refuse `ELOOP`, a `..` escape must refuse `EXDEV`, a mount crossing must refuse `EXDEV`, and a
    recorded identity that changed must refuse. Each refusal must name the component or field, the way item 2's
    refusals did ([measurements][measurements], lines 80 to 92).
3.  Attach. `fsopen("zfs")`, `fsconfig` the snapshot name as `source` and the `nosuid` and `nodev` flags, ask
    for read-only through the generic `ro` flag, run `FSCONFIG_CMD_CREATE`, then `fsmount` with `MOUNT_ATTR_NOSUID`,
    `MOUNT_ATTR_NODEV` and `MOUNT_ATTR_RDONLY`, and `move_mount` onto the held slot descriptor. The source is
    one snapshot name read from the mounted checkout's `.zfs/snapshot` directory, written as
    `<checkout-dataset>@<snapshot>`; the probe reads a directory and creates no snapshot. If that directory cannot
    be read or lists nothing, the arm is recorded as not exercised, and the route stays enabled with that gap
    reported, which is the posture the answer's "admits untested releases" sentence sets.
4.  Verify. Read `/proc/self/mountinfo`, require the expected mount ID, filesystem `zfs`, source dataset, and
    `nosuid,nodev,ro` flags; `fstatfs` the mount and read a bounded number of directory entries. No file content
    is read.
5.  Staging and unmount. From a forked child process, reopen the slot through the recorded parent, move the mount
    to the probe staging directory in the same namespace, verify the mount there, release descriptors, and
    unmount acting on the held descriptor, never on a path. This is the design's staging contract without the
    account-owned staging folder, and the child proves the unmount comes from a process that did not attach
    ([measurements][measurements], item 4).
6.  Namespace scan. Fork a holder that unshares its own mount namespace and keeps a mount of the same snapshot,
    run the root scan of `/proc/<pid>/ns/mnt` with each namespace's representative `mountinfo`, require the
    holder's namespace to be named, stop the holder, and require the scan to be clear. This is item 15's
    mechanism on a known holder, and its limit stands: a process-visible scan is not proof of absence.
7.  Teardown. Unmount the probe staging mount if it remains, unmount the tmpfs, drop the namespace, and write the
    record only after every arm finished.

If an arm fails, the probe stops at that arm, records its name and errno, tears the namespace down, and reports.
The refusal controls in the walk arm are expected refusals: an arm that attaches instead of refusing fails the
probe, so the harness has a positive control for its own check.

### Who runs it, and where

Root, from the handler binary, inside the private mount namespace the run creates. The probe never runs in the
host mount namespace, never runs as the account, and never runs inside `doctor`.

Placement: the existing boot unit. `reconcile-git-worktree-clone-mounts.service` already runs as root at boot,
already orders after the key and checkout providers, and already publishes readiness (the
[proposal][proposal], lines 492 to 505). The probe runs as part of its start when the managed route is
configured, before it reconciles intent, so the readiness record it publishes carries the result. No new request
channel is added, which matters because the design forbids a second mount request channel and fixes the intent
vocabulary to `mount` and `unmount` (the [proposal][proposal], lines 478 to 480 and 536 to 539). An on-demand
probe would need a third intent kind or a new channel, and this proposal does not propose one.

The wrapper's own per-invocation start check stays the module gate. The wrapper reads the probe's result from the
readiness record it already must read before an add (the [proposal][proposal], lines 352 to 354) and reports it.

### What it changes at start

- One new private mount namespace, made private before any mount is made.
- One tmpfs for scratch, no larger than the probe needs to hold a few directories and a link.
- One mount of a checkout snapshot, and the superblock it creates or reuses, for the duration of the run.
- One forked child for the unmount arm and one for the namespace arm, both short-lived, both the probe's own.
- One bounded directory read and one `fstatfs`.
- One block in the readiness record, written only after every arm finished.
- Journal lines from the boot unit.

It creates no dataset, snapshot, bookmark or property, makes no ZFS ioctl request, starts no service, and writes
no file outside its tmpfs and the root-owned record.

### What it leaves when it fails or is interrupted

A failed arm tears the namespace down and leaves the record naming the arm, the errno, the fingerprint and the
fallback. A SIGKILL leaves nothing in the host mount table and nothing on disk, which
[Assumptions](#assumptions) states and [M3](#m3-interruption-leaves-nothing) measures, because the namespace and
the tmpfs die with the process and the record is written last. If the record is absent, the wrapper's add preflight
already treats the readiness as not current and refuses the managed attach, which is the decided behaviour for a
missing readiness report (the [proposal][proposal], lines 352 to 354), not a new rule.

The one residue a killed probe can leave is a "started" marker, if the implementation writes one before the
arms. This proposal suggests it does, with the fingerprint and no result, so `doctor` can say the probe began
and did not finish, and the next boot overwrites it.

### What a refusal prints

The wrapper prints one line to standard error on the managed operation it refuses, naming the probe arm, the
running kernel release, the module version, the failed call and its errno, the route the slot will use instead,
and the `doctor` command that prints the last result. It never suggests a force flag, because the gate has none
and this probe adds none.

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
and source, whether the source existed, and the resulting verdict. It lives with the other root-owned state the
design already names, and the wrapper reads it without writing it.

The record is not authority to act. It is a cache of an observation, and the wrapper rebuilds nothing from it
except the verdict and its wording, the way the design treats its other runtime files (the [proposal][proposal],
lines 555 to 556).

### What the result gates

A result with every arm passed enables the managed attach for that boot and module identity.

A result naming a failed attach, staging or unmount arm disables new managed attaches, sends an unused slot to
the reflink or ordinary Git route, and keeps existing bindings, registrations and intent intact. Whether removal
of an existing binding stays available is the owner's answer to
[Q1](#q1-what-a-probe-failure-disables); this proposal's ranked
option keeps it available, because a kernel that cannot attach is exactly when a person needs the removal path
that does not attach.

A result naming an unexercised arm, because no snapshot or no source existed, enables the route with that gap
reported, which the answer's "admits untested releases" sentence allows.

A module or kernel whose fingerprint changed since the record makes the record stale; the next boot's probe
replaces it, and an add in between reads stale readiness, which the preflight must report as not current rather
than as a pass.

### What the probe never does

It never mounts by path on an account-controlled tree, never runs the `zfs` program, never opens `/dev/zfs`,
never creates or destroys a dataset or snapshot, never promotes, never sets a property, never writes a
registration or a receipt, never runs in the host mount namespace, never runs from `doctor`, and never offers a
force flag. The mounts it makes live only in its own namespace.

## Owed measurements

Every item here is owed before the probe can be trusted. None ran for this proposal, and each runs on a
disposable file-backed pool inside a throwaway VM, never on this host's pool and never with its `/dev/zfs`.

Way 1 needs M1, M2, M3, M4, M5, M7 and M9. Way 2 needs M10 plus M1 to M5. Way 3 needs M2, M4, M5 and M6. Way 4
needs M8 and M9.

### M1: which source exercises the dataset mount path

Attach a checkout snapshot, which the design's arm uses, and the already mounted checkout dataset beside it, and
record which one runs `zfs_domount` and the key check. Control: unload the key and repeat both; item 5 measured
`fsconfig(create)` refusing `EACCES` in that state ([measurements][measurements], line 231), so the arm that
refuses with the key unloaded is the arm that exercises the mount path, and the arm that still attaches is the
one that reused a superblock. Also record whether the already mounted dataset's read-write superblock refuses
the read-only request with `EBUSY` (`module/os/linux/zfs/zpl_super.c:931` to `:937`), and adjust the arm's flags
or source if it does.

Gates: Way 1, and the claim in [Assumptions](#assumptions) that a snapshot is the arm that certifies the key
path.

### M2: the host mount table is unchanged

Read `/proc/self/mountinfo` in the host namespace, and a representative `mountinfo` for every other visible
namespace, before and after a complete probe. Read the dataset list, the pool history and the checkout's
properties before and after as well. Control: an arm that deliberately mounts in the host namespace (fixture
only, never shipped) must show the difference the harness looks for, so a null result counts.

Gates: Way 1 and Way 3; also the claim that a probe leaves no pool history entry, and the snapshot-hold window.

### M3: interruption leaves nothing

Kill the probe with SIGKILL after each step: after `unshare`, after the tmpfs, after `fsconfig(create)`, after
`fsmount`, after `move_mount`, and before the unmount. Record the host mount table, every namespace's
representative `mountinfo`, the probe slot, the staging directory, the record, and whether a second run succeeds.
Control: the same probe allowed to finish, and a run killed before `unshare` returns.

Gates: Way 1's "leaves nothing" claims and the started-marker suggestion.

### M4: what the probe costs

Measure each arm and the whole probe as a ratio to a control arm of the same binary with the probe disabled,
interleaved in the same round, and report the range between rounds. Measure at boot, with the boot unit, and
measure any other placement an owner answer later asks for. Control: items 4 and 15's timings are not a control,
because they measure other work.

Gates: Way 1 and Way 3 placement, and any deadline the implementation ships; no deadline is stated here before
this measurement.

### M5: the refusal arm refuses for the right reason

Run the walk arm's four refusal controls and the success control, and require the success case to attach where
the refusal cases refuse. Control: the unmodified slot must attach, so a harness that refuses everything fails
its own control.

Gates: Way 1 and Way 3.

### M6: user namespace limits

As the account, attempt the probe's attach of a ZFS source inside a user namespace and require `EPERM`, with a
tmpfs mount in the same namespace succeeding as the control. Then disable unprivileged namespaces in the guest
(`user.max_user_namespaces` set to 0) and record that the account arm is skipped with a stated reason rather than
failing the boot. Control: root's attach in the init namespace succeeds.

Gates: Way 3's skip path, and the assumption that every ZFS arm needs root.

### M7: the namespace arm sees a known holder

Start the probe's forked holder, require the scan to name its namespace with the matching source, stop it, and
require the scan to be clear. Repeat with a holder that has no live member visible to the scan and record the
scan's report as unknown rather than clear.

Gates: Way 1's namespace arm and the limit that a process-visible scan is not proof of absence.

### M8: the real clone attach as evidence

Record the handler's first real attach after boot in the same record, and compare its outcome with the probe's
snapshot arm on the same build. Control: an unloaded-key clone attach must fail with `EACCES`, as item 5
measured, on the same boot as a passing probe arm.

Gates: Way 4's folded-in evidence, and the limit that a snapshot arm is not a clone arm.

### M9: the untested release and the refused release

Run the built probe on a release it must accept and on one it must refuse, and record what it changes at start,
what it leaves behind and what it prints, as the design's acceptance item requires (the
[proposal][proposal], lines 1841 to 1843), together with step 6's two arms (the [plan][plan], lines 739 to 745).
Control: the accepted release must pass, so a probe that refuses everything fails its own control.

Gates: every way, and the acceptance item.

### M10: a pool-writing arm, only if the owner picks it

If the owner chooses Way 2, measure the create and destroy of the probe dataset and clone, the space it takes,
its behaviour at `ENOSPC`, what an interrupted creation leaves, and whether the handler's sweep ignores it.
Control: way 1's read-only arms on the same build, so the write cost is a measured difference.

Gates: Way 2 only.

## Questions for the owner

Each question is separable, each has options with their costs, and the ranking gives the reason for every
adjacent pair. A later turn can put these into the question tool as written.

### Q1: what a probe failure disables

The probe fails only when an arm it ran actually failed, which is a real observation, not a missing piece. What
that observation disables is not decided.

- Block new managed attaches and keep removal (Recommended). Pros: an unused slot takes the reflink or Git
  route, an existing unmounted clone can still be destroyed through the module-gated ioctls, and a kernel that
  cannot attach is exactly when a person needs the removal that does not attach. Cons: the managed route is half
  available, and a reader must understand that attach and removal are gated separately.
- Mirror the module gate: pros, one rule for every start check, and the gate's wording already exists; cons, raw
  requests are disabled including destroy, so an existing unmounted clone cannot be removed until the kernel
  changes, which strands the very bindings the design says it preserves.
- Warn and continue: pros, the closest match to the Git-version check, and no availability loss; cons, an add
  proceeds into an attach the probe already saw fail, and its failure lands mid-operation on a claimed slot.

Ranking: block new managed attaches and keep removal, then mirror the module gate, then warn and continue,
because removal is the recovery path a broken attach needs, and a warning leaves the failure to appear inside
the operation the probe exists to protect.

### Q2: whether the probe may write to the live pool

- Read only (Recommended). Pros: no dataset, snapshot, property or space change at boot, no stranded probe
  object, and no interaction with sanoid or the space guard. Cons: the clone's own shape and a writable attach
  are exercised only by the real product attach that Way 4 records.
- A disposable probe dataset: pros, it exercises the clone shape exactly, independent of live state; cons, it
  writes the pool at every boot, needs a second parent and a sweep exclusion, can strand an object or hit
  `ENOSPC`, and puts a write where the pool's health is the question.

Ranking: read only, then a disposable dataset, because the read-only arms cover the kernel path that a release
changes, and the clone shape is covered by the real attach's recorded outcome without a boot-time write.

### Q3: whether the account runs a companion probe

- Root only (Recommended). Pros: one implementation, no user-namespace dependency, and the arm that matters
  needs root anyway. Cons: a syscall or namespace regression is seen only at the next boot, never before a given
  invocation.
- Root plus an account companion: pros, a per-invocation check of the walk, the mount surface and the wrapper's
  helpers before any root act, and the wrapper's tests already have the skip control; cons, a second
  implementation, a check that never touches ZFS, and a skip exactly on hosts where user namespaces are refused.

Ranking: root only, then the companion, because the account arm cannot observe the dataset mount path, which is
the failure the answer asks about, and its per-invocation timing is not yet measured.

### Q4: whether the namespace arm runs at every start

- Every start, with a short-lived forked holder (Recommended). Pros: the scan is exercised against a known
  holder on every boot, and a broken scan is caught before it can mis-report a removal. Cons: one extra process
  and mount at every boot, and a holder the rest of the system can see for its short life.
- Boot scan only, with the holder arm in acceptance tests: pros, fewer moving parts at boot; cons, the scan's
  ability to find a holder is then never exercised on the host, and a scan that always reports clear passes.

Ranking: every start, then boot scan only, because a scan that cannot be shown to find a holder is the failure
item 15 warns about, and the holder is the probe's own short-lived child.

## What a writer settles without asking

- The arm order, the private-namespace setup before any mount, and every syscall the probe uses.
- The scratch location (a tmpfs inside the probe's namespace), the staging directory inside it, and that the
  probe never reuses the account-owned staging folder.
- The source choice: a snapshot name read from the mounted checkout's `.zfs/snapshot` directory, taking the
  first name the read returns, and "not exercised" when the directory is unreadable or empty.
- That the probe reads no file content, only `fstatfs` and a bounded directory listing.
- The mount flags, the read-only request and the `EBUSY` handling M1 measures.
- The record's home and fields, the fingerprint's inputs, and that the record is written last.
- The wording of the wrapper line, the `doctor` section and the journal line, under the design's own reporting
  rules (the [proposal][proposal], lines 755 to 762).
- That `doctor` reads the result and never runs a probe, and that no force flag exists.
- That the probe runs in the boot unit before reconciliation, and that no on-demand channel is added.
- The deadline's value, set only after M4 measures the arms, and fail-closed behavior when it is exceeded.
- The fallback for an unused slot and the preservation of existing bindings, which the accepted design already
  decides.

## What this proposal does not change

It does not introduce an allowlist, does not withdraw or loosen the module gate, does not authorize a path mount
or a new request channel, does not add a force flag, does not make the probe a `doctor` action, does not create
or destroy a dataset at start, does not change the intent vocabulary, does not move the handler's mount and
unmount modes out of the host mount namespace, does not change any removal, promote, staging or delegation
decision, and does not decide any of the three remaining blocking designs.

## Sources read

- `doc/decision/cli-git-zfs-worktree-design.md`, whole.
- [The proposal][proposal], whole, including its "Open items the decisions leave", "ABI gate and tests",
  "Project decisions", "Decided defaults", "Descriptor mount and unmount", "Boot and shutdown unit", "Waiting
  and refusal", "What doctor checks and prints" and "Acceptance tests the build still owes".
- [The plan][plan], including "Wrapper code the design reuses", step 6, "What can start now", "Open items and
  the step that needs each", and "Where the design is silent".
- [The measurements record][measurements], whole.
- OpenZFS `zfs-2.4.4` at commit `71a9f9578616`, cloned on 2026-11-19: `module/os/linux/zfs/zpl_super.c`,
  `module/os/linux/zfs/zfs_vfsops.c`, `module/os/linux/zfs/zpl_ctldir.c`,
  `module/os/linux/spl/spl-zone.c`, `module/zfs/zfs_ioctl.c`, `module/zfs/dmu_objset.c`,
  `module/zfs/dsl_crypt.c` and `cmd/zfs/zfs_main.c`.
- This host, read only, 2026-11-19: `uname --kernel-release`, `/sys/module/zfs/version`, `/proc/kallsyms`,
  `/proc/filesystems`, `/sys/kernel/btf/`, `/dev/zfs` mode, `/proc/sys/user/max_user_namespaces` and
  `/proc/sys/kernel/unprivileged_userns_clone`.
- The wrapper's code: `package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  `.../subvolume/fs_mount_options.rs`, `.../subvolume/subvolume_remove_walk.rs`,
  `.../health/doctor_filesystem.rs`, `.../service/wait_ceilings.rs`,
  `.../invocation/git_version.rs`, `.../invocation/git_version_cache.rs`,
  `.../binary/binary_worktree_removal_around_a_volume_tests.rs`, and `package/git-policy/cli/SPEC.md`
  "Compatibility and degradation".

[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[owed-tests]: cli-git-zfs-worktree-design-proposal.md#acceptance-tests-the-build-still-owes
[plan]: cli-git-zfs-worktree-implementation-plan.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
