# The start probe for the ZFS worktree route

**Non-final, unapproved proposal by an agent.** This document designs one open item of the accepted ZFS worktree
design, "The start probe a kernel-release answer asks for", so that step 6 of the implementation plan can be
written. A Pi session wrote it on 2026-11-19 and revised it in the rounds listed under [Revisions](#revisions); it
is not accepted. It was written without running a VM, a probe or any pool command, from the sources listed under
[Sources read](#sources-read). What it checked: the
accepted design, the plan (every section list and step it names) and the measurements record, the seven answers the
question round returned and the threat-model sentence recorded beside them, OpenZFS `zfs-2.4.4` sources, the
running kernel's exported system calls, this host's `include/uapi/linux/mount.h`, systemd v262 `service.c`, `unit.c`
and `exec-invoke.c`, Linux v6.8 and v7.3 `fs/super.c`, `fs/fsopen.c` at v5.2, v6.5, v6.6 and v7.3, Linux v7.3
`kernel/user_namespace.c`, `kernel/ucount.c` and `mm/shmem.c`, and the wrapper's own code (fetched, because this
host's 7.3.5 tree ships no `fs` sources). No kernel source
was read for the first draft, which is how a non-existent mount attribute and a wrong `errno` survived it; the
kernel statements under [The kernel the design names](#the-kernel-the-design-names) name the version each was read
from. What stays open: every measurement under [Owed measurements](#owed-measurements), none of which ran. The
questions this design put to the owner were answered on 2026-11-19, and [Answers of 2026-11-19](#answers-of-2026-11-19)
records each one with the option the project chose; the body beside it follows those answers. The design is
accepted with its open items still open, and this file changes no decision of it. Nothing here authorizes work on
this host's pool.

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
- `2026-11-19T20:31:12-05:00` (`10f2435d34`, `e205be87f7` and this entry's commit): the answer to a second gate
  re-check. Its findings were weighed one by one, and every one earned a fix, with none refused: the duplication in
  the source question, which listed one option twice; the start question, which was a confirmation with one option and
  moved to the writer's list; the move and unmount arm, which could not unmount while the worker held references on
  the mount; the holder protocol, now stating every pipe and its endpoints, the parent-death race and the
  process-group call; the txg window, which now carries a start fence, an end fence and an idle control; and
  `ExecStartPre=-`, which keeps a probe crash from skipping the reconcile. Also fixed: the marked options and the
  costs charged to one way, the sanoid facts the source question left out, the probe dataset's layout facts, the
  fold-in that cost nothing, the scope of the "never opens `/dev/zfs`" statement, the teardown arm's own unmount,
  two option sentences, the control-directory path, the unnamed handler modes and the prose the checkers do not
  count. The measurements this round added are under [Owed measurements](#owed-measurements).
- `2026-11-19T20:58:00-05:00` (commits `daa7a93f67`, `01e51e24c1`, `e850455f55` and this entry's commit): the
  owner's answers of 2026-11-19 applied. A new "Answers of 2026-11-19" section states each question, the option
  the project chose with the round's timestamp range, the option it did not take and why, and the threat model the
  answers assume. The question sections and the ways this design weighed are gone, leaving a pointer to the answers
  and one shape. The body follows the answers: the source is the dedicated dataset
  `zpcachyos/nosnap/git-worktree-probe` with its `ready` snapshot, so the account-influenced name, the
  `.zfs/snapshot` read and the reuse-skip go; the move and
  unmount arms stay in step 6; a failed arm refuses new attaches only; an unexercised arm blocks the route; a
  missed deadline fails closed. New design work: the account companion, with its namespace setup, its arms, its
  refusal, its skip, its printed lines, its per-add cost and the measurement that sizes it, and the `first-attach`
  record, with its writer, its claim, its staged rename, its keys, its reader and the steps that carry it. A new
  "Notes for the plan" section names what each answer adds to steps 4, 6, 7, 8, 10, 18, 19, 20 and 22. The
  measurements gained M11 and a control for M8; M1, M2, M6 and M10 follow the dedicated source, and the
  access-time and account-influenced-name work they gated is deleted. The three checkers ran on the design and on
  a broken copy, each failing there and passing here.

## Answers of 2026-11-19

Every question this design put to the owner was answered in the question tool between 2026-11-19T20:35 and
2026-11-19T20:43:51-05:00. The record is `doc/handover/cli-git-rust-implementation.md` in the main checkout, in the
paragraph beginning "THE ROUND WAS ASKED". This design now follows the answers: where its own ranking differed, the
answer wins, and the body, the record, the unit, the measurements and the plan-step notes take the chosen shape.
Each entry gives the question, the option the project chose in the question tool's words, the round's timestamp
range, and one sentence naming the option it did not take and why, in the project's terms.

### Q1: whether the plan bars the probe's unmount arm

Chosen: "Product mounts only (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The
plan's sentence that no unmount runs before the third design covers the product's mounts, so the probe's own move
and unmount arms are built in step 6, inside the namespace and the tmpfs the probe makes for itself. Not taken: the
literal reading, that the sentence covers the probe's unmount too, which would build the arm in step 9 and leave
step 6 shipping a gate whose unmount half never ran.

### Q2: what a failed arm disables

Chosen: "Refuse new attaches only (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. An arm
that ran and failed refuses new managed attaches: an unused slot takes the same-dataset reflink or the ordinary Git
route, `boot-reconcile` attaches nothing and keeps every intent and registration, removal of a clone that is not
mounted stays available, and removal of a mounted clone is refused only when the failed arm is the move, the unmount or
the scan. Not taken: mirroring the module gate, which would refuse every ZFS operation and strand the removal of an
unmounted clone until the kernel changed, the availability cost the allowlist answer was chosen against; and
warn-and-continue, which would proceed into the attach the probe had just seen fail.

### Q3: whether a setup-time probe dataset exists

Chosen: "Dedicated dataset (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The source is
the dataset `zpcachyos/nosnap/git-worktree-probe` with the empty snapshot `ready`, and the plan's steps 18 to 20 carry
its creation, its configuration field and its exclusion. Not taken: a snapshot of the checkout, which needs no layout
change, and which this design did not keep because the source can be absent right after a migration, the account holds
`snapshot` on the checkout and `destroy` on its descendants, and another namespace can hold every candidate, so arms
can silently not run.

### Q4: whether an unexercised arm may enable the route

Chosen: "Block until it has run (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. An arm
that could not run keeps the managed route blocked, with the per-operation effects the failed-arm answer states. Not
taken: reporting the gap and keeping the route enabled, which would let a broken kernel pass with the gap visible only
in `doctor`, now that the source always exists.

### Q5: whether a missed deadline fails closed

Chosen: "Fail closed (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. A probe that misses
its deadline writes a timed-out result, the record is not current, an add refuses and names the reason, an unused slot
takes the reflink or ordinary Git route, and existing bindings and intent stay. Not taken: failing open with a warning,
which would attempt an attach on the strength of a record that says the probe could not finish.

### Q6: whether the account runs a companion probe

Chosen: "Root plus account companion", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The account also
runs the small user-namespace companion before each managed add, and the root probe stays the only evidence about the
dataset mount path. Not taken: root only, this design's own first rank, which would leave an account-side regression of
the walk or the mount surface seen only where something mounts as the account; the project took the companion knowing
that it never touches ZFS and skips on a host that refuses user namespaces.

### Q7: whether the first real attach records its outcome

Chosen: "Write first-attach", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The handler process that
made the first real attach after boot writes its outcome to `first-attach` beside the probe's record, and `doctor`
shows it beside the probe's result. Not taken: the probe record only, this design's own first rank, which keeps no
outcome from a real clone, a real slot and the host namespace; the project took the write, with its duty on plan steps
4 and 8.

### The threat model these answers assume

The project stated, answering a question of the staging design the same evening: "We have to assume my machine
isn't compromised. sudo is passwordless here anyway." This design therefore treats the account as trusted and does
not treat root and the account as a security boundary. Nothing else in it moves because of that sentence: the
descriptor walk, the identity compare and the refusals stay as written, as checks that a path and an identity are
the ones the design pinned, not as a boundary between root and the account.

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
  this design, provided the missing piece is reported; the project's answer keeps the route blocked until every
  arm has run, and the record names the arm that did not run
  ([Q4](#q4-whether-an-unexercised-arm-may-enable-the-route)).

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
  holder scan, depends on steps 2, 4, 6 and 8 (the [plan][plan], lines 967 to 971). The project read that
  sentence as covering the product's mounts ([Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm)), so the
  probe's own move and unmount arms are built in step 6; they are probe-private code either way, because the
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
  no ioctl and no delegation check (`module/os/linux/zfs/zpl_ctldir.c:281` to `:320`). Nothing in this design
  reads that directory: the probe's source is the configured name, so no name is discovered at start and no
  lookup by name can start the automounter (`module/os/linux/zfs/zpl_ctldir.c:255` to `:277`).
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
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures it. The probe's source is the dedicated
  dataset's `ready` snapshot, whose key is the pool root's and which the boot unit already waits for before
  `ExecStart`, so the refusal itself is a regression arm only, as
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
- Every arm can be exercised without creating a clone, using the dedicated dataset's empty `ready` snapshot as
  the attach source. The clone's exact shape (its encryption root and its slot properties) is then not exercised,
  which is the clone-shape limit under [Limits](#limits);
  [M1](#m1-which-source-exercises-the-dataset-mount-path) and
  [M8](#m8-the-real-clone-attach-as-evidence) measure what the snapshot arm does and does not cover.
- The account can make a user namespace and a mount namespace on this host, map its own ID to 0 there, and mount a
  tmpfs inside them, so the companion runs. This rests on `user_namespaces(7)` and
  `/proc/sys/user/max_user_namespaces` reading `252369` and `/proc/sys/kernel/unprivileged_userns_clone` reading
  `1` here (read-only, 2026-11-19); [M11](#m11-the-account-companion-per-add) measures the arms and the skip.
- tmpfs accepts the descriptor calls the companion makes, so its attach, move and unmount arms reach a real mount
  without ZFS: it declares `FS_USERNS_MOUNT` and carries fs_context operations (`mm/shmem.c:5103`, `:5284`,
  `:5311` and `:5316`, read at v7.3). [M11](#m11-the-account-companion-per-add)'s success control breaks or
  confirms it.

## Limits

- The probe cannot prove host-namespace propagation, a clone's real slot, or ZED ordering. Only the handler's
  actual attach at a real slot does that, which is why
  [the first real attach's record](#the-first-real-attachs-record) keeps its evidence and why
  [M8](#m8-the-real-clone-attach-as-evidence) measures it.
- The probe's mount lands in a directory of its own tmpfs, not in the checkout's slot, so the probe does not
  show the real slot, the account-owned staging folder, or the checkout's propagation into a shared parent.
  Those belong to step 9's staging design and to [the first real attach's record](#the-first-real-attachs-record).
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
- The key check is regression coverage only in the product: the probe attaches the dedicated dataset's `ready`
  snapshot, whose key is the pool root's and which the boot unit waits for, so no arm produces the `EACCES`
  refusal on a healthy boot. Item 5's unloaded-key procedure runs in the VM fixture, not at start
  ([M1](#m1-which-source-exercises-the-dataset-mount-path)).
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
- A probe that holds the `ready` snapshot for its duration makes a concurrent `zfs destroy` of that exact
  snapshot refuse `EBUSY`; nothing else snapshots or destroys that dataset, so the window is not expected to be
  met, and [M2](#m2-the-mount-table-and-the-pool-are-unchanged) records what the hold shows. The holder and the
  unmount child each carry a parent-death signal set after their own fork and the pipes their forking process
  owns, so an interruptible child does not outlive that thread unless it died between
  the fork and the call that sets the signal, which the child's `getppid()` recheck covers
  ([M3](#m3-interruption-leaves-nothing)). A child inside an uninterruptible kernel wait survives its `SIGKILL`
  until the wait returns, exactly as the worker does, so those bounds cover an interruptible child rather than
  every child. The residue is the worker: it sets no parent-death signal, so a mode killed alone leaves it
  running, and a `SIGKILL` cannot interrupt a kernel wait, so a worker inside one holds the snapshot until the wait
  returns.
- The probe exercises one source, the configured dataset's `ready` snapshot, so an attach that succeeds there says
  nothing about another dataset's mountpoint or flags. The clone's shape and its encryption root are covered only
  by the first attach's record ([M8](#m8-the-real-clone-attach-as-evidence)).
- The freshness oracle needs Linux 6.6 or newer. `FSCONFIG_CMD_CREATE_EXCL` is refused with `EOPNOTSUPP` by an
  older kernel ([The kernel headers and source](#the-kernel-headers-and-source)), so on such a kernel the attach
  arm is unexercised and the probe never claims a fresh superblock. Where the command exists, it tells a fresh
  attach from a held superblock even when no mount of that superblock is visible in any readable namespace,
  because `sget_fc` matches on the objset rather than on a mount; the cost is that such a source is skipped
  rather than attached.
- The companion never reaches the dataset mount path, the key path or `/dev/zfs`, so its pass cannot predict the
  probe's or the product's attach, and its failure refuses an add without evidence that the root attach would
  fail. It is skipped on a host that refuses user namespaces or has none left, and its skip is printed rather
  than kept silent.
- The companion's cost on every managed add, and the deadline that bounds it, are unmeasured until
  [M11](#m11-the-account-companion-per-add); no value is stated here.
- The `first-attach` file records one operation and is not a compatibility verdict. It is written only by the
  process that made that attach, it is read by `doctor` alone, and no route turns on it.

## The shape the answers chose

The shape has three parts, and the answers fix all three: a root probe at boot on the dataset the source answer
names, the account companion before each add, and the first real attach's record. Every arm of the probe runs on
every boot, because the source exists and nothing else mounts it.

The probe is a new mode of the handler binary, in step 6, root-run, inside a private mount namespace of its own, and
its source is the dedicated dataset's `ready` snapshot. The companion is account-side code the add preflight runs,
in a user and mount namespace of its own, touching no ZFS. The record is a write in the handler's attach path, in
the process that made the attach.

Two shapes are not taken. A probe on a snapshot of the checkout needs no layout change, and the project did not
choose it for the reasons [Q3](#q3-whether-a-setup-time-probe-dataset-exists) records. A probe that created and
destroyed a clone at every start is ruled out rather than ranked. The accepted design gives the wrapper
"unprivileged snapshot and clone creation" and the unit mount and unmount, "not Git or automatic deletion" (the
[decision][decision], lines 22 to 26). The handler "has no Git, pool administration, or dataset destruction
operation" (the [proposal][proposal], lines 160 to 161), and "There are no pool management, dataset rename, or
generic dataset creation requests" (lines 211 to 212). Such a probe would also need `/dev/zfs` at boot, on the pool
whose health is the question, so it would add a boot-time write on that pool, a stranded object after an
interruption and `ENOSPC` at boot.

## The proposed probe

This section proposes the shape the answers settle, in the detail the open item asks for: one root mode at boot on
the dedicated source, one account companion in the add preflight, and one outcome file the first real attach
writes. Every part of it is a writer's proposal inside an undesigned item, and the answers under
[Answers of 2026-11-19](#answers-of-2026-11-19) are the only decisions it takes from the owner.

### The source: the probe dataset

The source is the dataset `zpcachyos/nosnap/git-worktree-probe` with the empty snapshot `ready`, which the source
answer chose. It sits under the container `zpcachyos/nosnap`, which is unmounted (`mountpoint=none`,
`canmount=off`) and carries `autosnap = no` with each of its children (the [proposal][proposal], lines 1098 to
1124); those children are mounted data directories, and the container is the one dataset there that nothing
mounts. Its properties are `mountpoint=legacy`, `canmount=off`, `com.sun:auto-snapshot=false` and the user property
`org.monochromatic:git-worktree-probe-source=v1`. Nothing mounts a legacy dataset automatically, and `canmount=off`
stops `zfs mount -a` and the mount generator too (the [proposal][proposal], lines 262 to 270), so only a root
process that asks for it attaches it.

Root-owned configuration pins its name and GUID beside the pool and dataset names that configuration already pins
(the [proposal][proposal], line 319). The probe reads that name and checks it as a name, because it makes no ioctl;
`doctor` is where the GUID and the user property are compared, through the ioctl the probe does not make. The
snapshot is empty. Its key is the pool root's, which the boot unit already waits for before `ExecStart`, so an
unavailable key is an `EACCES` refusal from the attach arm rather than a hidden gap. No sanoid section is needed to
keep it out of snapshots: the only recursive section is the home template, `zpcachyos/ROOT/cos/home`, and
`zpcachyos/nosnap` is outside that subtree (the [proposal][proposal], lines 1081 to 1092). The section this design
adds in step 20 follows the configuration's convention of naming every exclusion, so a later recursive ancestor
cannot silently include it.

What the account can do to it: nothing. Delegation is scoped to the checkout dataset and the clone parent (the
[proposal][proposal], lines 319 to 327), and that same passage says not to put unrelated ZFS descendants there.
This dataset is under neither, so the account holds no `create`, `mount`, `destroy` or `userprop` on it and can
neither create, rename, mount, snapshot nor destroy it. No account-run process and no automatic job pins or
destroys it, so it cannot collide with the space guard or the sweep, which enumerate the pinned clone parent's
children and never this dataset (the [proposal][proposal], lines 540 to 541).

How its existence is detected: the probe asks the kernel, not the pool. It attaches `<dataset>@ready` with
`FSCONFIG_CMD_CREATE_EXCL`, and a missing dataset or snapshot is the `ENOENT` from `fsconfig(create)`, as the
OpenZFS facts state. `ENOENT` alone does not name the cause: `spa_open` returns it for a pool that is not imported
(`module/zfs/spa.c:6478` to `:6482`), so the record names the arm and the call rather than the cause. An existing
dataset whose attach answered `ENOENT` is a failure `doctor` reports as such, after it checks that the configured
dataset exists; a dataset the configuration names and the pool does not hold is reported as an unexercised attach
arm and is never created. Until the setup steps run, that is the reading every boot gets: the configuration names a
dataset the pool does not hold, so the attach arm is unexercised and the managed route stays blocked
([Q4](#q4-whether-an-unexercised-arm-may-enable-the-route)), with removal of an unmounted clone still available.

What it changes at start: nothing beyond the attach the arms describe. There is no directory read of
`.zfs/snapshot`, no account-influenced name and no access-time question. The dataset itself outlives every run by
design, and the plan's steps 18 to 20 build the commands and configuration that create it, which the owner runs, as
[Notes for the plan](#notes-for-the-plan) records.

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
3.  Source, from the root-owned configuration. The worker reads the dataset name from the configuration
    `boot-reconcile` reads, accepts it only as a name (each `/`-separated component a valid ZFS component name,
    within the dataset-name bound, with no `@` and no `#`), and assembles `<dataset>@ready` from the snapshot name
    the design fixes. A configuration that is absent, unreadable or names nothing the parse accepts records the
    attach arm and every arm after it as not exercised, with that reason. The probe reads no property and no
    directory at start: the configuration is the only place the name comes from, and the user property that
    names the source is read by `doctor`, through an ioctl the probe does not make.
4.  Attach, with the freshness oracle. `fsopen` on the `zfs` filesystem type, `fsconfig` the source name,
    `fsconfig` the `nosuid` and `nodev` options, read-only through the generic `ro` flag, then
    `FSCONFIG_CMD_CREATE_EXCL`, then `fsmount` with `MOUNT_ATTR_NOSUID`, `MOUNT_ATTR_NODEV` and
    `MOUNT_ATTR_RDONLY`, and `move_mount` from the mount descriptor onto the held slot descriptor. The exclusive
    create is what proves the superblock is new: a matched extant superblock answers `EBUSY` before `zfs_domount`
    runs, whether or not any mount of it is visible. A missing dataset or a missing `ready` answers `ENOENT` from
    the same call, which the record attributes to the arm and the call, never to the missing object. A kernel
    older than 6.6 refuses the command with `EOPNOTSUPP`, which records the attach arm as not exercised and stops
    the probe there rather than falling back to a plain create that could reuse a superblock.
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
A source another namespace holds is not a probe failure: arm 4 records `EBUSY` with the source's name, and records
the attach arm and every arm after it as not exercised, because the design has no second source to try. The refusal
controls in the walk arm are expected refusals: an arm that attaches instead of refusing fails the probe, so the
harness has a positive control for its own check.

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

### The account companion

This is the second check the companion answer chose. It runs as the account, from the wrapper's own process, before
each managed add, and it never touches ZFS, the key path or `/dev/zfs`.

What it runs. The add preflight forks one child. The child calls `unshare(CLONE_NEWUSER)`, writes `deny` to
`/proc/self/setgroups`, writes one line each to `uid_map` and `gid_map` mapping the account's own effective user and
group to 0 in the new namespace, then calls `unshare(CLONE_NEWNS)`, makes every mount private, and mounts a tmpfs at
a scratch directory the parent made for the run. Those writes are the ones an unprivileged caller may make: one
line, its own effective ID, and `setgroups` denied before `gid_map` (`user_namespaces(7)`, the sections on defining
the two mappings and on `/proc/pid/setgroups`). tmpfs declares `FS_USERNS_MOUNT` and carries fs_context
operations, so the descriptor calls reach it without a path mount (`mm/shmem.c:5103`, `:5284`, `:5311` and `:5316`,
read at v7.3).

Then it runs the probe's own code on that tmpfs: the wrapper's no-crossing walk from the tmpfs root to the slot with
the `ObjectId` compare, `fsopen` on `tmpfs` with `fsconfig`, `fsmount` and `move_mount` onto the held slot
descriptor, the mount facts read back from `/proc/self/mountinfo`, the four refusal controls, and the move and the
unmount from a second child it forks. It makes no ZFS call, reads no file content, and mounts nothing outside its
own namespaces. Its unmount is its own tmpfs mount in its own namespace, and the unmount answer leaves it unbarred
for the same reason it leaves the probe's arm unbarred: neither is a product mount.

When, and as whom. In the add preflight, after the wrapper's own start gate and before the add's claim, so that a
refusal leaves no managed object, which is the preflight order the proposal already requires (the
[proposal][proposal], lines 348 to 355). It runs as the account and never as root; the managed attach itself still
runs as root in the handler, so the companion is never evidence about that attach.

What it refuses. A companion arm that ran and failed refuses that add's managed attach, prints one line, and leaves
an unused slot to the same-dataset reflink or ordinary Git route; it keeps every existing binding and intent. It
refuses no removal, no reconcile and no other wrapper command, and it never writes the probe's record: `doctor`
shows the probe's arms and the first attach's outcome, never a companion verdict.

Its deadline. The parent bounds the child with a monotonic deadline and, on expiry, kills the child's process group
and refuses the managed attach as unproven, which is the same posture the deadline answer gives the probe. No value
is stated; [M11](#m11-the-account-companion-per-add) measures the arms and sizes it.

Its skip. A host that refuses user namespaces, or has none left, answers `unshare(CLONE_NEWUSER)` with `EPERM`,
`ENOSPC` or `EUSERS` (`unshare(2)`, ERRORS); `kernel/user_namespace.c:91` to `:96` answers `ENOSPC` when the count
passes `user.max_user_namespaces`, which `kernel/ucount.c:77` installs. The companion then prints one line naming
the call, the errno and the two settings it read (`/proc/sys/user/max_user_namespaces` and
`/proc/sys/kernel/unprivileged_userns_clone`), records its arms as not exercised, and does not refuse the add. That
is the project's choice: it took a companion whose skip is printed rather than silent, and the unexercised-arm
answer governs the probe record's arms, not this per-add check. `doctor` prints both settings and says that the
companion skips while they answer so, so a host cannot lose the check without a printed reason.

What it prints. On a failed arm:
`managed add refused: the account companion's <arm> failed: <call> (errno <n>); this slot takes <route>.`
On a skip: `account companion skipped: <call> answered <errno>; user namespaces are unavailable here`, followed by
the values it read, `user.max_user_namespaces=<value>` and `kernel.unprivileged_userns_clone=<value>`. On a pass it
prints nothing. The lines go to standard error under the design's own reporting rules (the [proposal][proposal],
lines 755 to 762).

What it costs per add, and what sizes that. One fork, two `unshare` calls, two mapping writes, one tmpfs mount and
its unmount, the walk and its identity compare, the refusal controls, and one more child for the move and the
unmount, on every managed add. No duration is stated: [M11](#m11-the-account-companion-per-add) measures the cost as
a ratio to the same add with the companion disabled, interleaved in the same round, with the range between rounds,
and carries the controls that make that ratio able to fail.

### The first real attach's record

The write the record answer chose keeps the outcome no probe can produce: a real clone, at a real slot, in the host
mount namespace. The file is `first-attach`, beside `result` in `/run/git-worktree-clone-mount-probe/`, root-owned
mode `0644` in that root-owned `0755` directory, read-only to the wrapper and `doctor`.

Which process writes it, and when. The handler process that makes the first real clone attach after the current boot
writes it: `boot-reconcile`, the attaching sweep a ZED event starts, or the no-argument sweep, whichever attaches
first. It is that process, not the mode that supervises a probe and not the wrapper. It writes after the attach's
outcome is known, either the verified mount at the slot or the failing step and its errno, and before it publishes
this boot's receipt and readiness, so a reader that sees the readiness sees the file too, and a crash between the
two leaves an outcome without a receipt rather than a receipt without an outcome.

How one writer keeps it. The writer claims the name with an exclusive creation, the shape `write_exclusive_synced`
has (`package/git-policy/cli/src/native/foundation/private_storage.rs:500`), writing the started line; a process that
finds the claim answers `EEXIST` and writes nothing. The winner then stages the outcome in a sibling in the same
directory and renames it over the claimed name in one step, the shape `replace_private_file` has
(`.../foundation/private_storage_replacing_a_record.rs:54`), written with the record's own owner and mode rather than
the private `0600` the wrapper's helpers give the account's own files. A reader therefore sees the started line or
the whole outcome, and a winner that dies between the claim and the rename leaves the started line, which every
reader reports as begun and not finished, as it reports the probe's started record. The claim is per boot because
the directory is on `/run`; the outcome also carries the boot ID, and a reader whose boot differs reads the file as
stale.

Its format. One `key value` line per field, fixed keys and no free text, in the shape the probe's record uses:
`v1` first, then `kind operation-outcome`, `state started` or the outcome, `boot <id>`, `kernel <release>`,
`module <version>`, `srcversion <value>`, `handler <binary identity>`, `time <ISO 8601 with offset>`,
`operation attach`, and then either `outcome attached` with `mount <mount ID>`, `flags <the flags read back>`,
`slot <the slot name>`, `clone <the clone's dataset name>` and `guid <the clone's GUID>`, or
`outcome refused` with `slot <the slot name>`, `clone <the clone's dataset name>`, `guid <the clone's GUID>`,
`step <the failing step>` and `errno <name>`. A reader refuses a set of lines with an unknown key, a repeated key,
a missing required key or a value it cannot parse, and reports the file as unreadable rather than guessing.

How `doctor` reads it. In the same section as the probe's result, opened relative to the root-owned directory with
`open_regular_file` and bounded by `read_regular_file`'s limit, so a link or a named pipe at the name is refused
rather than waited on (`.../foundation/private_storage.rs:785` and `:817`). It prints the outcome and, for a
refusal, the step and the errno; an absent file as no real attach this boot; a started line as begun and not
finished; a boot ID that differs from this boot as stale; and an unreadable file as unreadable.

How it stays an operation outcome. Its own `kind` line says `operation-outcome` and it has no verdict field, and no
consumer gates on it: the add preflight reads the probe's record and the module gate, never this file, and a
first-attach refusal neither enables nor disables any route. `doctor` prints it under its own label and keeps the
two questions apart: the probe says whether this kernel and module can attach at all, and this file says what one
attach did on this boot, so a clone whose key was unloaded, a slot whose identity had drifted or a device that
failed reads as an operation failure and the probe's arms are unchanged. Where the two differ, `doctor` prints the
difference as a difference and names both, without calling either a verdict on the other.

Which plan steps carry the duty. Step 4, the attach module, builds the outcome line and the writer, because it is
where the failing step and the errno are known (the [plan][plan], lines 549 to 634). Step 8, the attaching sweep,
calls it, and its sweep is the path that makes the first attach after boot (lines 831 to 886). Step 7's readiness
record references the file rather than owning it, as it does the probe's record (lines 762 to 779). Step 18's
`doctor` gains the reader beside the probe's result (lines 1490 to 1571). [M8](#m8-the-real-clone-attach-as-evidence)
measures the file against the probe's snapshot arm on one build.

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
or finished run (`systemd.kill(5)`, `KillMode=`). A missed deadline writes a timed-out result and the route fails
closed ([Q5](#q5-whether-a-missed-deadline-fails-closed)); the deadline's value comes from
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

The account companion is not part of this placement: it is the wrapper's per-add check, described under
[The account companion](#the-account-companion), and no boot unit runs it.

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
namespace, so the host sees empty directories there after a run and the record file beside them. `first-attach`
lives beside `result` in that directory, with the same owner and mode, and the handler process that made the first
real attach after boot writes it ([The first real attach's record](#the-first-real-attachs-record)); the probe
mode neither writes nor reads it.

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
[proposal][proposal], lines 235 to 237); the probe's verdict binds at least every attach path, and a failed arm
refuses new managed attaches and keeps every other operation ([Q2](#q2-what-a-failed-arm-disables)).

### What it changes at start

- One worker process, the mode process that forks it and enforces its deadline, a holder the mode forks, and an
  unmount child the worker forks, each short-lived, with one control pipe and one status pipe per child.
- One new private mount namespace for the worker and one for the holder, made private before any mount is made.
- One tmpfs for scratch, no larger than the probe needs to hold a few directories and a link.
- One mount of a read-only source, and the superblock it creates, for the duration of the run.
- One bounded directory read on the probe's own read-only mount and one `fstatfs`.
- One root-owned directory under `/run` and its record, rewritten by atomic rename.
- Journal lines from the boot unit.

It creates no dataset, snapshot, bookmark or property, makes no ZFS ioctl request, starts no service, and writes
no file outside the record's directory and the tmpfs inside it.

The account companion adds nothing here: it runs in the add's own process, in namespaces of its own, and its tmpfs
lives and dies with that child. Neither does the first real attach, beyond the one file it writes at the attach's
end.

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

The `first-attach` file has the same started state: a claim carrying the started line and no outcome is read as
begun and not finished, never as a pass and never as a verdict.

### What a refusal prints

The wrapper prints one line to standard error on the managed operation it refuses, naming the probe arm, the
running kernel release, the module version, the failed call and its errno, the route the slot will use instead,
and the `doctor` command that prints the last result. It never suggests a force flag, because the gate has none
and this probe adds none. A failed arm refuses new managed attaches and keeps every other operation, with the
per-operation effects [Q2](#q2-what-a-failed-arm-disables) states.

`doctor` prints a new section under the existing filesystem report, headed by its section marker pattern, with
the fingerprint, when the last probe ran, each arm and its outcome, any difference between the recorded
fingerprint and the live kernel, module or boot, the sources and mounts the probe used, and which route remains
available. It prints the first real attach's outcome beside the probe's result, under its own label
([The first real attach's record](#the-first-real-attachs-record)), and the two user-namespace settings with the
companion's skip ([The account companion](#the-account-companion)). The section goes to standard error and changes
no exit status, as the design requires for setup findings (the [proposal][proposal], lines 1153 to 1155).

The companion prints its own line on the add it refuses or skips, as [The account companion](#the-account-companion)
states, and the wrapper's refusal line names the companion's arm and errno beside the probe's when both apply.

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
status pipes. The first real attach's outcome goes into a second file beside it, `first-attach`, written by the
handler process that made that attach, as [The first real attach's record](#the-first-real-attachs-record) states:
that file has its own writer per boot and its own claim, so each file has one writer and no lock is needed. The
wrapper and `doctor` read both and write neither.

The record is not authority to act. It is a cache of an observation, and the wrapper rebuilds nothing from it
except the verdict and its wording, the way the design treats its other runtime files (the [proposal][proposal],
lines 555 to 556).

### What the result gates

A result with every arm passed enables managed attaches for that boot and module identity.

A result naming a failed attach, walk, move or unmount arm, or a failed namespace arm, is a real observation, and
it refuses new managed attaches while it keeps every other operation ([Q2](#q2-what-a-failed-arm-disables)): an
unused slot takes the same-dataset reflink or ordinary Git route, `boot-reconcile` attaches nothing and keeps
every intent and registration, removal of a clone that is not mounted stays available, and removal of a mounted
clone is refused and reported when the failed arm is the move, the unmount or the scan.

A result naming an unexercised arm, because the configured source is absent, another namespace holds it, the
configuration names nothing the parse accepts, or the kernel refused the exclusive create, blocks the managed
route ([Q4](#q4-whether-an-unexercised-arm-may-enable-the-route)) with those same per-operation effects, and
`doctor` reports which arm did not run and why.

A result naming a missed deadline is written as timed out, and the route fails closed
([Q5](#q5-whether-a-missed-deadline-fails-closed)): the record is not current and the same effects follow.

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
never runs from `doctor`, and never offers a force flag. It never reads `.zfs/snapshot` at all: its source is the
configured name, so the `.zfs` control directory is never opened, and no entry name in it is ever looked up.

The account companion never touches ZFS, the key path or `/dev/zfs`, never runs as root, and mounts only inside the
user and mount namespaces it makes for itself; it writes no record. The mounts the probe and the companion make
live only in their own namespaces.

## Notes for the plan

These are notes for the plan's own text, and this design edits no plan step.

- Step 4 gains the `first-attach` writer, its format and its exclusive claim, because the attach module is where a
  failing step and its errno are known (the [plan][plan], lines 549 to 634).
- Step 6 gains the `start-probe` mode, run as `ExecStartPre=-` of the boot unit, with its attach, move, unmount and
  namespace arms, which the unmount answer places in this step, and the account companion's own check code beside
  them (lines 698 to 759).
- Step 7's readiness record references the probe's record and `first-attach` rather than owning either, and its
  preflight treats an absent, stale, started, timed-out or blocked record as not current (lines 762 to 779).
- Step 8's sweep calls the `first-attach` writer, and its sweep is the path that makes the first attach after boot
  (lines 831 to 886).
- Step 10's add preflight runs the account companion before the claim and applies the failed-arm answer's refusal
  rule and the unexercised-arm answer's block (lines 973 to 1068).
- Steps 18, 19 and 20 create the probe dataset, its empty `ready` snapshot, its configuration field, its user
  property and its sanoid section, as the source answer states (lines 1490 to 1571, 1572 to 1676 and 1677 to 1727).
  Step 18's `doctor` also prints the first attach, the two user-namespace settings and the companion's skip.
- Step 22 sizes the probe's deadline and the companion's per-add deadline beside the waits it already sizes (lines
  1774 to 1847).

## Owed measurements

Every item here is owed before the probe can be trusted. None ran for this proposal, and each runs on a
disposable file-backed pool inside a throwaway VM, never on this host's pool and never with its `/dev/zfs`.

The chosen shape needs every item here. The root probe needs M1, M2, M3, M4, M5, M7, M9 and M10, and M6 for the
`EPERM` it expects the account to meet. The account companion needs M6 and M11. The first real attach's record
needs M8 and M9.

### M1: which source exercises the dataset mount path

Attach the dedicated dataset's `ready` snapshot, which no namespace mounts, and the already mounted checkout
dataset beside it, and record which one runs `zfs_domount` and the key path. The oracle for a fresh superblock is
`FSCONFIG_CMD_CREATE_EXCL`: the fresh source must attach, and the already mounted one must answer `EBUSY` from
`fsconfig(create)` rather than succeed while reusing a superblock. A successful attach alone proves nothing about
the key check, because the source's key is the pool root's and is loaded; the two `EBUSY` and `EACCES` controls
carry that evidence. Control: item 5's unloaded-key procedure, which
unmounts the checkout and home first ([measurements][measurements], lines 221 to 222), unloads the key, and
attempts the fixture's `ready` source; require `EACCES` from `fsconfig(create)` (item 5, line 231). A second
control: the same attach of a name that the pool does not hold, which must answer `ENOENT`, so that a harness
reporting every refusal as a missing dataset fails its own control. Also record whether a read-only request
against a read-write dataset superblock refuses `EBUSY` (`module/os/linux/zfs/zpl_super.c:936` to `:943`), adjust
the arm's flags if it does, and record what a kernel older than 6.6 returns for the exclusive create, so the
unexercised arm's wording is measured rather than assumed.

Gates: the probe's attach arm and its source, the exclusive create as the freshness oracle, the claim that a
source no namespace mounts is the one that reaches the mount path, and the limit that the key refusal is regression
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
which must show the mount-table difference the harness looks for; and a deliberate write in a scratch dataset,
which must leave a nonzero committed row in the window, including one made inside `T0` itself, so a harness that
never reads the file, or that only looks past the window, fails its own control. A deliberate read of the attached
mount's own directory belongs beside them, since the probe makes one and the harness must show it changing no pool
row.

Gates: the probe and the account companion; also the claim that a probe leaves no pool write, and the
snapshot-hold window.

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
delay it adds to the first login that `systemd-user-sessions.service` admits, and measure any other placement a
later answer asks for. The companion is not measured here; [M11](#m11-the-account-companion-per-add) measures it.
Control: items 4 and 15's timings are not a control, because they measure other work.

Gates: the probe's placement, its own deadline, and the time-to-login cost of a boot ordered before user sessions.
No value is stated before this measurement.

### M5: the refusal arm refuses for the right reason

Run every refusal control the walk arm lists and the success control, and require the success case to attach
where the refusal cases refuse. Control: the unmodified slot must attach, so a harness that refuses everything
fails its own control.

Gates: the probe's walk, its refusal controls and the wrapper's `openat2` flags.

### M6: user namespace limits

As the account, attempt the probe's attach of a ZFS source that no namespace mounts, inside a user namespace, and
require `EPERM` from `zfs_domount`. Then repeat with the source mounted in the guest's initial namespace and
require `EBUSY` from the matched extant superblock instead, because `sget_fc` refuses a superblock from another
user namespace before `zfs_domount` runs (Linux v6.8 `fs/super.c:779` to `:786`). Control: a tmpfs mount in the
same user namespace succeeds, and root's attach in the init namespace succeeds. Then disable unprivileged
namespaces in the guest (`user.max_user_namespaces` set to 0) and record that the account arm is skipped with a
stated reason rather than failing the boot.

Gates: the assumption that every ZFS arm needs root, the companion's `EPERM` reading, and the skip path
[M11](#m11-the-account-companion-per-add) measures.

### M7: the namespace arm sees a known holder and reports only the probe's own namespaces

Record the scan's two passes and what each must show: the worker's and the holder's namespaces while both mounts
exist, the worker's namespace alone after the holder is stopped, and no namespace after the unmount. Repeat with
the source already mounted in a second namespace before the probe runs, and require the attach arm to answer
`EBUSY` and the probe to record that source as held and the later arms as not exercised instead of refusing
spuriously. Repeat with a root mount of the same source made in a further namespace during the run, and require
it to name that mount in the first pass and fail the arm. Repeat with a namespace pinned and no live member in it,
and record that the scan does not name it and reports the source clear: a clear report is not absence, which the
document says wherever it relies on the scan.

Gates: the probe's namespace arm, the exclusive create as the freshness oracle, and the limit that a
process-visible scan is not proof of absence.

### M8: the real clone attach as evidence

Record the handler's first real attach after boot as the `first-attach` file writes it, written by the process
that made the attach, and compare its outcome with the probe's snapshot arm on the same build. Controls, each of
which must fail its arm when it is broken: an unloaded-key clone attach must write `outcome refused` with the
`EACCES` that item 5 measured, on the same boot as a passing probe arm; a second attach in the same boot must write
nothing, because its claim answers `EEXIST`; and a writer killed after the claim must leave the started line, which
`doctor` reports as begun and not finished. Compare the file against the probe's arm both ways: a passing probe
with a refused first attach must leave the probe's record unchanged, and a failing probe arm with an attached
first attach must not turn that probe arm into a pass.

Gates: the first real attach's record, its one-writer claim, and the limit that a snapshot arm is not a clone arm.

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

Gates: the built probe, the built companion and the acceptance item.

### M10: the probe dataset

Measure the setup command's creation of `zpcachyos/nosnap/git-worktree-probe`, its `ready` snapshot, its user
property and its configuration field, and its idempotent second run. Measure the space it takes, that nothing else
mounts it, that the sweep and the destroy-anyway path never see it, and that the sanoid configuration excludes it.
Record what the probe writes when the dataset is absent, when its `ready` snapshot is absent, and when the
configuration names a dataset the pool does not hold: the first two answer `ENOENT` from `fsconfig(create)` and
the record names the arm, and the third is the same arm reported from an absent name. Controls: the same attach
of a name that is absent while the configuration names the real dataset, which must record the arm as not
exercised rather than as a pass, and the healthy dataset beside it, which must attach.

Gates: the probe's source and the setup steps.

### M11: the account companion, per add

Run the built companion in the add preflight on the guest, and record each arm: the user namespace and its mapping
writes, the mount namespace and its private propagation, the tmpfs scratch mount, the walk and its `ObjectId`
compare, the descriptor attach of the second tmpfs, the flags read back, the four refusal controls, and the move
and the unmount from the second child. Every refusal control needs its success control beside it, the shape
[M5](#m5-the-refusal-arm-refuses-for-the-right-reason) gives the probe's own, so a companion that refuses
everything fails its own control.

Measure what the companion costs one add as a ratio to the same add with the companion disabled, interleaved in the
same round, and report the range between rounds; size the per-add deadline from that measurement and from no other.
Controls that make the number and the verdict able to fail: the interleaved disabled arm, which must show the
difference the ratio reports; a test-only build whose child answers `EPERM` from `fsmount`, which must refuse the
add, name the call and take the fallback route, with a second build that passes; and a guest booted with
`user.max_user_namespaces` set to 0, which must print the skip line naming `ENOSPC` and the setting, must not
refuse the add, and must leave `doctor` naming the setting and the skip. A transient unit carrying
`SystemCallFilter=` for `unshare` with `SystemCallErrorNumber=EPERM` is the other refusal a host can show, and it
must print the same skip line with that errno.

Gates: the account companion's arms, its skip path, its per-add deadline, and the assumption that tmpfs accepts the
descriptor calls it makes.

## Questions the owner answered

The question round closed on 2026-11-19, and every answer is under
[Answers of 2026-11-19](#answers-of-2026-11-19), which states each question, the option the project chose, the
timestamp range and the option it did not take. No question of this design stays open. What stays open is
measurement: every item under [Owed measurements](#owed-measurements) is unrun, and nothing here authorizes work on
this host's pool.

## What a writer settles without asking

- The scratch location: the record's directory under `/run`, the tmpfs mounted at its scratch mount point, the
  slot, the move-target directory and the refusal tree inside that tmpfs, and that the probe never touches the
  account-owned staging folder or the checkout's own directories.
- The name rule: the source is the configured dataset's `ready` snapshot, the root-owned configuration is the
  only place the name comes from, an absent or unparseable configuration records the later arms as not exercised,
  and a missing dataset or snapshot is never created.
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
- That the probe reads no file content, only `fstatfs`, a bounded directory listing on its own mount, and mount
  facts.
- The mount flags, the read-only request, the exclusive create as the freshness oracle, and the `EBUSY` handling
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures.
- That the unmount and the move are by path inside the probe's own tmpfs, which the unmount answer places in
  step 6 ([Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm)), and that the probe therefore does not validate
  the product's held-descriptor unmount. The companion's tmpfs unmount is its own and is built with it.
- That the probe tears down without unmounting its own tmpfs, whose removal is the namespace's death.
- The wording of the wrapper line, the `doctor` section and the journal line, under the design's own reporting
  rules (the [proposal][proposal], lines 755 to 762).
- The record's directory, mode, fields and atomic-rename write, and the started record before the fork.
- The `first-attach` file's keys, its claim, its staged rename and the reader's refusals, as
  [The first real attach's record](#the-first-real-attachs-record) states, and that the handler process that made
  the attach is the only writer.
- That the account companion runs in the add preflight before the claim, in a child of the wrapper's process, that
  its failure refuses only that add's managed attach, and that its skip is printed rather than turned into a
  refusal.
- That `doctor` reads the result and never runs a probe, and that no force flag exists.
- That the probe is a handler mode named `start-probe`, run as `ExecStartPre=-` of the existing unit, that it runs
  the module gate first and skips itself when the gate refuses, that the mode process enforces the deadline and
  exits successfully whenever it reaches its own end, and that no on-demand channel is added.
- The deadline's value, set only after [M4](#m4-what-the-probe-costs) measures the arms, and the companion's
  per-add deadline, set only after [M11](#m11-the-account-companion-per-add); the unit carries no start timeout, and no
  plan step sizes one: step 22 sizes the mount-request deadline of step 7, the unmount deadline of step 9 and any
  shipped deadline those waits would need (the [plan][plan], lines 1778 to 1779 and 1811 to 1812).
- The fallback for an unused slot and the preservation of existing bindings, which the accepted design already
  decides.

## What this proposal does not change

It does not introduce an allowlist, does not withdraw or loosen the module gate, does not authorize a path mount
or a new request channel, does not add a force flag, does not make the probe a `doctor` action, does not create or
destroy a dataset at start (the probe dataset is the owner's act through the setup steps), does not change the
intent vocabulary, does not move the handler's `boot-reconcile` and `shutdown-unmount` modes out of the host mount
namespace, does not change any removal, promote, staging or delegation decision, and does not decide any of the
remaining blocking designs. It does add the two duties the answers chose: the account companion in the add
preflight, and the `first-attach` write in the handler's attach path, which the plan's steps 4, 6, 8 and 10 carry
as [Notes for the plan](#notes-for-the-plan) records. It also does not bring the discarded pool-writing probe
back: that one contradicts the accepted design, as [The shape the answers chose](#the-shape-the-answers-chose)
states.

## Sources read

- [The decision record][decision], whole.
- The answer record: `doc/handover/cli-git-rust-implementation.md` in the main checkout, the paragraph beginning
  `THE ROUND WAS ASKED`, which holds the seven answers [Answers of 2026-11-19](#answers-of-2026-11-19) records and
  the threat-model sentence quoted there.
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
  fetched `fs/namespace.c`, `fs/pnode.c`, `kernel/exit.c` and `kernel/sys.c` at v7.3 itself. For the account
  companion, Linux v7.3 `kernel/user_namespace.c` and `kernel/ucount.c` (fetched, for the `ENOSPC` of an exhausted
  namespace count and the setting that bounds it) and `mm/shmem.c` (fetched, for tmpfs's `FS_USERNS_MOUNT` and
  its fs_context operations).
- `user_namespaces(7)` and `unshare(2)` read on this host, for the mapping rules, the `setgroups` order and the
  errnos a refused namespace creation answers.
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
  `.../foundation/private_storage.rs`, `.../foundation/private_storage_replacing_a_record.rs`,
  `.../binary/binary_worktree_removal_around_a_volume_tests.rs`, and `package/git-policy/cli/SPEC.md`
  "Compatibility and degradation".

[decision]: ../decision/cli-git-zfs-worktree-design.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[owed-tests]: cli-git-zfs-worktree-design-proposal.md#acceptance-tests-the-build-still-owes
[plan]: cli-git-zfs-worktree-implementation-plan.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
