# The taskless mount namespace, measured on the step 23 template's kernel

**Non-final draft by an agent,
unapproved.**
It records one round's guest measurements on the kernel the step 23 template boots,
with their controls and with what did not run.
The owner declares the work complete,
and the owner's answer of 2026-11-26 kept the refusal until a guest of `6.19` or newer measured the walk;
this record is that measurement and what the build does now.

## What was asked

Step 17's decision record says (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
"The two refusals the built arm keeps"):

> A taskless namespace,
> a mount namespace whose tasks are gone and which still holds the copy,
> can be found only through `listns`,
> which every guest the arms use answers with `ENOSYS`;
> the arm refuses it with `holder-unendable`.
> Asked whether to keep that refusal until a guest of 6.19 or newer measures the walk,
> the owner answered "Keep the refusal (Recommended)".

The forced removal's design says what the arm needs to measure it
(`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
L18):

> The taskless-namespace arm needs a guest of `6.19` or newer,
> and a caller as root in the host's own namespaces needs a CachyOS guest with `linux-cachyos-zfs` and a guest agent,
> which the project must build.

## A measurement this round did not take

`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
"T0-10 with the handler's privileges",
already recorded on 2026-11-23,
in a guest booting `7.0.0-43-generic`,
that `listns` lists a mount namespace kept alive by a bind of its `nsfs` file,
that `open_by_handle_at` on its id gives a descriptor,
and that `setns` enters it.
That is the first half of the walk and not the arm:
the namespace held a tmpfs mark and no copy of a clone,
and nothing unmounted anything inside it.
So the owner's condition was met by that record for the listing,
and not for what the arm does with what it lists.
This round measures the second half on the kernel the step 23 template boots,
and the arm ends the holder there.

## The template, and the guest made from it

The image the step 23 record names was on this host when the round began
(`~/.local/share/mvm/images/cachyos-7.3.5-1-zfs-2.4.4.qcow2`,
4,093,509,632 bytes),
and its sha256 is the one the record names:

```text
# sha256sum on this host, 2026-11-27T21:14-05:00
938598df250f29bf83d9cfd635baee4b52ea593aebb2071df9601d306c3085f3
```

Two guests were made from it,
one after the other,
and each was destroyed through `mvm` when its work was done:
`zfs-r10-listns-walk`
made at 2026-11-27T21:42-05:00,
and `zfs-r10-taskless-final` made at 2026-11-28T02:40-05:00,
each with
`mvm --backend libvirt create --image cachyos-7.3.5-1-zfs-2.4.4 <name>`,
four processors and 7.9 GiB of memory.
The second was made because the arm's phase changed after the first run,
and the lines quoted below are the second's.
Two preparations the template does not do for itself were made by hand,
both of them the record's own list of what the template lacks:

- the image's fixed hostname,
  `cachyos-7.3.5-1-zfs-2.4.4`,
  was replaced with the guest's `mvm` name,
  because every driver's arms are guarded by `test "$(hostname)" = <guest name>`;
- a 6 GiB `/swapfile` was added,
  because the template carries no swap and one `rustc` of this crate's tests has been measured at 5.06 GiB of anonymous
  memory inside a guest of this size.

What the guest answered about itself before any pool existed
(`mvm exec` in `zfs-r10-listns-walk`,
2026-11-27T21:45-05:00;
`zfs-r10-taskless-final` answered the same):

```text
# hostname
zfs-r10-listns-walk
# uname --kernel-release
7.3.5-1-cachyos
# cat /sys/module/zfs/version
2.4.4-1
# lsmod
zfs                  6713344  0
spl                   167936  1 zfs
# /proc/self/uid_map
         0          0 4294967295
# CapEff
000001ffffffffff
# zfs version
zfs-2.4.4-1
zfs-kmod-2.4.4-1
```

The module was read before it was relied on:
the recipe's own template preparation loads `zfs` with `modprobe --quiet zfs 2>/dev/null || true`,
so a load that failed says nothing there,
and this round read `/sys/module/zfs/version` and `uname --kernel-release` itself.
Both answered,
so that silent `modprobe` hid nothing from this round.
The first guest was destroyed at 2026-11-28T00:46-05:00
and the second at 2026-11-28T07:07-05:00,
and `mvm --backend libvirt list` then named no guest `zfs-r10-`.

## What the kernel answered

A probe of this round's own asked the kernel directly,
as root in the guest's initial user and pid namespaces
(`listns-walk-probe.py`,
kept with the round's folder,
run at 2026-11-27T21:43-05:00 in `zfs-r10-listns-walk`).
It makes one mount namespace whose only task leaves it,
and keeps it alive by a descriptor it opened on the task's own `/proc` link:

```text
# NS_GET_ID on this process's own mount namespace=8 errno=0
# listns pages=2 ids=8 end-errno=0 (None)
# listed=[8, 10, 36, 43, 48, 57, 63, 76]
# this process's own id is listed=True
# the kept namespace is mnt:[4026532328] and NS_GET_ID answered 82 errno=0
# tasks in the kept namespace=[]
# after the task went: listns pages=2 ids=9 end-errno=2
# the kept id 82 is listed=True
# open_by_handle_at on the kept id answered a descriptor naming mnt:[4026532328]
# inside the kept namespace: rows=27 mark=['296 211 0:55 / /mnt/r10-mark rw,relatime shared:236 - tmpfs tmpfs rw,size=1024k,inode64,huge=advise']
# unmount inside the kept namespace answered=0 errno=2
# inside the kept namespace after the unmount: mark rows=[]
# after every descriptor of the kept namespace went: ids=8
# the kept id 82 is still listed=False
```

What that says,
line by line:

- `NS_GET_ID` answers the id of a mount namespace descriptor,
  and the initial mount namespace's id is `8`,
  which is `MNT_NS_INIT_ID`;
- the listing gave eight ids in two calls;
  the first call filled its page and the second answered an empty page with **no** errno,
  so a page that adds nothing is how this kernel can end the listing without saying `ENOENT`;
- with one more namespace alive the listing answered nine ids and the call after them answered `ENOENT` (`2`),
  which is the end of the list;
- the namespace the task left is listed,
  and no task of the machine is in it,
  which is the kind the design calls a taskless namespace;
- a handle of `{16 bytes, FILEID_NSFS, id in the first eight bytes, zeroes after}`
  opens a descriptor naming `mnt:[4026532328]`;
- `setns` on that descriptor enters the namespace,
  its table holds the tmpfs mark,
  and an unmount inside it is accepted;
- when the last descriptor of that namespace closed,
  it died and the listing no longer held it,
  which is the design's F13 read once more.

## What the arm did

The arm is the forced act's guest arm,
`the_forced_act_guest_arm_tests` in `src/native/worktree_clone_forced_removal/the_forced_act_guest_arms_tests.rs`,
whose new eighth phase makes such a namespace,
finds it through the walk,
runs the preflight and both stages on it,
and reports the kernel's own answers.
One run,
2026-11-28T06:52-05:00,
in `zfs-r10-taskless-final`,
against the file-backed pool the arm builds,
of the code this round commits:

```text
FORCEDACT guest=zfs-r10-taskless-final kernel=7.3.5-1-cachyos zfs=zfs-2.4.4-1
FORCEDACT phase=taskless-namespace listns=9 held=9 taskless=0 holders-before=0 holders=1 id=97 identity=mnt:[4026532369] control=busy route=in-place-by-the-handle mount=334 holders-copy=gone hosts-copy=stays unmount=accepted destroy=gone caller-job=alive ms=144
FORCEDACT phase=arm-finished phases=8 result=every-phase-held
# the test binary's own summary of the same run
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11028 filtered out; finished in 31.70s
```

Each field is an answer the phase read,
not a claim about one:

- `listns=9 held=9 taskless=0`:
  the listing answered nine mount namespaces and the tasks' own namespaces answered the same nine ids,
  so at that moment no listed id was taskless;
  the namespace the phase then made is the one `holders=1` finds,
  which is why this pair is the control the next field rests on;
- `holders-before=0 holders=1`:
  before the phase made its own namespace the walk found no taskless namespace holding a copy at this slot,
  and after it found exactly one;
- `id=97 identity=mnt:[4026532369]`:
  the id the listing gave and the namespace it named;
- `control=busy`:
  while that namespace held the copy,
  a `zfs destroy` of the clone asked in a namespace of the control's own was refused,
  so the copy there is what kept the dataset busy;
- `route=in-place-by-the-handle`:
  the preflight's own route for this namespace,
  from the receivers closure;
- `mount=334 holders-copy=gone hosts-copy=stays`:
  the copy went from the namespace no task holds and the host's own copy at the slot stayed;
- `unmount=accepted destroy=gone`:
  the host's unmount was accepted afterwards and the head destroy answered gone;
- `caller-job=alive`:
  the caller's session's other job lived through it.

The walk's own sizes on this kernel:
in the probe's guest the listing answered ten ids in two calls and the call after them answered `ENOENT`;
the walk's whole phase,
listing,
opening,
table read,
preflight and both stages,
took 144 ms of the arm's clock.
That is a duration of one phase and not a comparison:
no control arm was run beside it,
so it is context and not a finding.

## What the build does now

`worktree_clone_forced_removal/holder_endings.rs` names the kind's ending
`Ending::EnterAndTakeTheCopyOut`,
with `TASKLESS_SHOWN_BY` beside it saying which run on which kernel showed it,
and `ends_blockers` answers false for it,
because such a namespace has no task to signal.
The row's superseded reason stays in the same file as `TASKLESS_NOT_BUILT`.

The walk is two new files of that folder:

- `the_namespace_listing.rs` makes the kernel's calls:
  `listns` with `struct ns_id_req` for `MNT_NS` of the caller's user namespace,
  one page after another until `ENOENT` or a page that adds nothing,
  `NS_GET_ID` on each task's `/proc/<pid>/ns/mnt`,
  and `open_by_handle_at` with a `FILEID_NSFS` handle of the listed id;
- `taskless_namespace_walk.rs` decides:
  the ids no task holds,
  their tables read by a child that enters each one,
  the copy at the slot,
  and the design's two words,
  `holder-unendable` for a kernel that answers `ENOSYS`
  and `holder-blind` for a read it could not make.

The preflight needed no change:
a taskless namespace's table slots into the tables it already reads,
and the route is the receivers closure's,
which is why the phase reported `in-place-by-the-handle` from the preflight's own answer.

## The control: the usual Ubuntu guest

The owner's answer kept the refusal for a kernel without `listns`,
so the control is that such a kernel still answers the refusal.
One guest of the recipe's own kind,
`zfs-r10-listns-control`,
was made from `mvm`'s Ubuntu 24.04 image with no custom template
at 2026-11-28T00:47-05:00 and prepared by the recipe itself:
Ubuntu's packages,
OpenZFS `2.4.4` built from source for that guest's kernel,
and the Rust toolchain.
It answered about itself:

```text
# uname --kernel-release
6.8.0-147-generic
# /sys/module/zfs/version
2.4.4-1
# zfs version
zfs-2.4.4-1
zfs-kmod-2.4.4-1
```

The probe there answered the two calls the walk makes:

```text
# NS_GET_ID on this process's own mount namespace=None errno=25
# listns pages=1 ids=0 end-errno=38 (ENOSYS)
```

which is `ENOTTY` to the id read and `ENOSYS` to the listing,
the generation before the namespace tree exists.

The arm itself ran in that guest,
against the file-backed pool it builds,
at 2026-11-28T06:36-05:00:

```text
FORCEDACT guest=zfs-r10-listns-control kernel=6.8.0-147-generic zfs=zfs-2.4.4-1
FORCEDACT phase=taskless-namespace-unlistable listns=ENOSYS refusal=holder-unendable ending=enter-and-take-the-copy-out caller-job=alive ms=68
FORCEDACT phase=arm-finished phases=8 result=every-phase-held
# the test binary's own summary of the same run
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11028 filtered out; finished in 33.03s
```

So on `6.8.0-147-generic` the arm answers the listing `ENOSYS` and refuses with the design's
`holder-unendable`,
while the row it reads still names the ending this build has for a kernel that lists.

That control found a defect in the phase and not in the walk:
its first run in that guest failed,
because the phase counted the holders with the walk before it asked whether the kernel lists namespaces at all,
and a kernel of `6.8` panicked at that count
(`the walk answers on this kernel: TheKernelHasNoListns { errno: 38 }`).
The phase now asks the listing first,
and the fix is commit `43b005f57e`.
The run quoted above is the one after it.

The guest was destroyed through `mvm` at 2026-11-28T06:37-05:00.

## Deciding against the host's own kernel

The host boots `7.3.5-2-cachyos`,
which has `listns` too,
and the crate's own controls read it there
(`the_namespace_listing_tests.rs` and `taskless_namespace_walk_tests.rs`
list the machine's namespaces and read its tasks' ids,
which are reads and change nothing).
No mount,
`zfs` verb or `/dev/zfs` open ran on the host,
which the rules above this round forbid,
so the host's kernel is where the calls were proved to exist and nowhere that they were acted on.

## What this does not show

- A namespace kept alive by a bind of its `nsfs` file,
  which is the shape the 2026-11-23 record measured,
  was not measured again here;
  this round kept its namespace by a descriptor the arm opened.
  Both are the same kernel class and only the pinning differs.
- A listed id reused after its namespace died,
  and the act's refusal of a round whose id points at another namespace:
  the design records that ids may be reused,
  and the walk reopens each id beside its descriptor,
  but no arm made a reuse happen.
- A taskless namespace whose own user namespace is not the initial one:
  the walk's opener is the handler's root,
  and whether the kernel answers a handle for such an id with a caller outside its user namespace is measured nowhere.
- A listed id whose namespace holds the copy but whose table could not be read,
  which the walk refuses `holder-blind`:
  the phase reads a table that answers.

## What is left, and whose it is

- The decision record's sentence that the arm refuses `holder-unendable`
  is now the answer only on a kernel that has no `listns`.
  Amending that record is the owner's or the coordinating session's;
  this round changed the build and this planning record.
- The step 23 record's own "What is left" keeps a step 19 question and a `cargo-nextest` gap;
  neither is this round's.
- The arm reached by no command still stands:
  nothing in this build offers `--force`,
  and the new ending is reached only by the guest arm and by the folder's controls.
