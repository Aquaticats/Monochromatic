# Step 23's host-kernel qualification run, on `7.3.5-1-cachyos`

**Non-final draft by an agent,
unapproved.**
It records one round's guest measurements,
with their controls and with what did not run,
on the kernel release this host boots.
Only the owner declares a release qualified,
and no shipped constant,
allowlist or design changed in the round that wrote this.

The plan's step 23 asks for one thing
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 23:
The host-kernel qualification run"):
the custom-template guest the design describes,
and then item 1's encrypted file-backed fixture with the built mount,
unmount,
refusal and namespace controls on that release,
printing `uname --kernel-release` and `zfs version` before any pool exists.
The design's paragraph is "Qualify `7.3.5-1-cachyos` in a disposable VM,
not on the host pool",
under "Acceptance tests the build still owes"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`),
and it is the paragraph this record answers.

The host stayed read only:
no command here opened the host's `/dev/zfs`,
ran a `zfs` or `zpool` verb that changes anything,
or touched the pool `zpcachyos`.
Every pool named below is a file-backed image inside the guest,
and the guest was destroyed through `mvm` when the run ended.

## The template

The image is `~/.local/share/mvm/images/cachyos-7.3.5-1-zfs-2.4.4.qcow2`,
in the folder `mvm`'s custom-template route reads
(`package/mvm/cli/README.md`,
"Custom templates").
Its name says what it holds:
`cachyos`,
the kernel release `7.3.5-1`,
and OpenZFS `2.4.4`.

```text
# qemu-img info and sha256sum on the finished image
file format: qcow2
virtual size: 20 GiB (21474836480 bytes)
disk size: 2.11 GiB
4093509632 bytes
938598df250f29bf83d9cfd635baee4b52ea593aebb2071df9601d306c3085f3
```

It boots one partition of ext4 on BIOS,
with no EFI system partition and no boot manager entry,
through `grub-install --target=i386-pc`
and a serial console on `ttyS0`.

### What it carries

- `linux-cachyos` `7.3.5-1`,
  its headers `7.3.5-1`,
  `linux-cachyos-zfs` `7.3.5-1`
  (the module built for exactly that kernel,
  `zfs-kmod-2.4.4-1`),
  and `zfs-utils` `2.4.4-3`
  (the userspace,
  `zfs-2.4.4-1`).
  The four packages are the whole of the release's ZFS,
  and `pacman --query` in the booted guest names all four.
- `qemu-guest-agent`,
  enabled at boot,
  so `mvm exec`,
  `push` and `pull` reach it.
- `rust` `1:1.100.0-1`,
  which is the release the recipe pins,
  and `cargo-nextest` `0.9.151-1.1`,
  which is one release behind the recipe's pin (`0.9.152`) and was not moved.
- `git` `2.56.1-1.1`,
  `pahole` `1:1.32-2`,
  `strace` `7.3-1`,
  `python`.
- The unpacked OpenZFS `2.4.4` release tree at `/root/zfs-2.4.4`,
  because the live request-number assertions read its headers
  (`MONOCHROMATIC_ZFS_FIXTURE_SOURCE`).
- `/opt/openzfs-2.4.4`,
  holding `sbin` and `bin` as links to `/usr/bin`
  and `lib` to `/usr/lib`,
  so the recipe's OpenZFS prefix is the package's own userspace;
  `/root/.cargo/bin/{cargo,rustc,cargo-nextest}` as links to the packages;
  an empty `/root/.rustup`,
  which the account episode's copy finds rather than misses;
  the account `ubuntu` with a locked password;
  `/home/zfs-step-2-fixture`;
  `/mnt/shared` in `fstab` as a virtiofs share with `nofail`;
  and `zfs` named in `/etc/modules-load.d/zfs.conf`,
  so the module is loaded at boot rather than by the first command that needs it.

### How it was built, and where every root command ran

The host runs no root command for it.
One throwaway `mvm` libvirt guest,
`zfs-step-23-image-build`,
made from `mvm`'s Ubuntu 24.04 image,
was the machine whose root built the image:
`pacstrap` and `arch-chroot` ran inside it,
onto a disk image the guest itself attached with `losetup`.
The host's own part is unprivileged:
`mvm create`,
`mvm push`,
`mvm exec`,
and a `cp` of the finished file into the image folder,
because `mvm pull` refuses a file above 2 GiB
and this one is 3.8 GiB.

The build driver is kept with this record
(`keep/template-build/build-cachyos-template.ts` in the round's folder),
and its step logs are in `keep/image-build-logs/`.
In order,
the commands that ran as root inside that guest were:

```text
# guest steps: seed-prep.log
apt-get update
apt-get --yes --no-install-recommends install curl zstd e2fsprogs qemu-utils util-linux
# bootstrap-download.log
curl --fail --location --output /root/archbootstrap.tar.zst \
  https://geo.mirror.pkgbuild.com/iso/latest/archlinux-bootstrap-x86_64.tar.zst
curl --fail --location --output /root/zfs-2.4.4.tar.gz \
  https://github.com/openzfs/zfs/releases/download/zfs-2.4.4/zfs-2.4.4.tar.gz
sha256sum --check --strict   # 2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1
tar --extract --file /root/archbootstrap.tar.zst --directory /root/archbootstrap --numeric-owner
mv /root/archbootstrap/root.x86_64 /root/archbootstrap/root
# bootstrap-config.log, through the bootstrap's own arch-chroot
pacman --noconfirm -Syu
pacman --noconfirm --needed -S cachyos-keyring cachyos-mirrorlist archlinux-keyring
# target.log
truncate --size 20G /root/image/cachyos.img
losetup --find --show --partscan /root/image/cachyos.img
sfdisk --quiet --wipe always "$loop"      # label: dos, one bootable type=83 partition
mkfs.ext4 -F -L cachyos "${loop}p1"
mount "${loop}p1" /root/image/target
mount --rbind /root/image /root/archbootstrap/root/root/image
# pacstrap.log
pacstrap -C /etc/pacman-image.conf -K /root/image/target \
  base mkinitcpio grub e2fsprogs gnupg inetutils qemu-guest-agent git rust cargo-nextest \
  base-devel pahole strace acl attr procps-ng psmisc util-linux findutils grep sed gawk tar \
  gzip zstd curl ca-certificates which less iproute2 iputils lsof python
# configure.log, inside arch-chroot /root/image/target
pacman --noconfirm -U /root/pkgs/linux-cachyos-7.3.5-1-x86_64_v4.pkg.tar.zst \
  /root/pkgs/linux-cachyos-headers-7.3.5-1-x86_64_v4.pkg.tar.zst \
  /root/pkgs/linux-cachyos-zfs-7.3.5-1-x86_64_v4.pkg.tar.zst \
  /root/pkgs/zfs-utils-2.4.4-3-x86_64.pkg.tar.zst
useradd --create-home --uid 1000 --shell /bin/bash ubuntu
tar --extract --file /root/zfs-2.4.4.tar.gz --directory /root
systemctl enable qemu-guest-agent.service systemd-networkd.service systemd-resolved.service
systemctl enable serial-getty@ttyS0.service
mkinitcpio -P
grub-install --target=i386-pc --recheck "$loop"
grub-mkconfig -o /boot/grub/grub.cfg
# convert.log
umount /root/image/target
losetup --detach "$loop"
qemu-img convert -f raw -O qcow2 /root/image/cachyos.img /root/cachyos-7.3.5-1-zfs-2.4.4.qcow2
cp /root/cachyos-7.3.5-1-zfs-2.4.4.qcow2 /mnt/shared/
```

The four packages are the host's own cached `pacman` packages,
copied into the guest as files:
the repositories carry only newer releases
(`7.3.5-2` on `cachyos` and `7.3.6-2` on `cachyos-znver4`),
the host itself has since installed `linux-cachyos` `7.3.6-2`
and booted `7.3.5-2-cachyos`,
and `archive.cachyos.org` answered with a placeholder page,
so the pinned release is available here only from that cache.
That is a limit of this build,
not of the image:
a run elsewhere needs those four packages from the archive they were withdrawn from.

## Versions printed before any pool existed

The guest was destroyed after the run,
so this is the log the run kept
(`keep/guest/versions-before-any-pool.log`),
taken before any arm and before any pool existed:

```text
# uname --kernel-release
7.3.5-1-cachyos
# uname --all
Linux zfs-step-23-qualification 7.3.5-1-cachyos #1 SMP PREEMPT_DYNAMIC Tue, 17 Nov 2026 19:45:21 +0000 x86_64 GNU/Linux
# cat /sys/module/zfs/version
2.4.4-1
# zfs version
zfs-2.4.4-1
zfs-kmod-2.4.4-1
# pools before any arm
# datasets before any arm
# pacman --query linux-cachyos linux-cachyos-zfs linux-cachyos-headers zfs-utils
linux-cachyos 7.3.5-1
linux-cachyos-zfs 7.3.5-1
linux-cachyos-headers 7.3.5-1
zfs-utils 2.4.4-3
# /sys/kernel/btf/zfs
present, 570234 bytes
# /sys/kernel/btf/vmlinux
present, 6579400 bytes
# lsmod
zfs                  6713344  0
spl                   167936  1 zfs
```

The design's paragraph says installed packages alone do not qualify the run,
so the booted release,
the loaded module and the userspace are what it verifies:
the three lines above are the host's own release,
`/sys/module/zfs/version` answering `2.4.4-1` from the module the kernel loaded,
and `zfs version` naming both halves `2.4.4-1`.

## The guest and how the arms ran

One guest,
`zfs-step-23-qualification`,
made from the template with
`mvm --backend libvirt create --image cachyos-7.3.5-1-zfs-2.4.4 zfs-step-23-qualification`,
four processors and 7.9 GiB of memory,
destroyed through `mvm` at the end.
The settling command's driver makes and destroys a guest of its own,
`zfs-step-23-qualification-settling`,
from the same template and with `--image` and `--kernel-release`,
so the run used two guests one after the other and destroyed both.
The arms are the drivers the plan's steps and their measurement records name,
run in those guests:

- step 2's arms,
  the binding crate's live suites,
  through `package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`;
- step 8's mount control and step 9's namespace and unmount controls,
  through `package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`;
- step 16's snapshot batch through the same recipe,
  and its settling command through `package/git-policy/cli/e2e/settling-command-guest.ts`.

The recipe and the two wrapper drivers gained the smallest change that lets them run on this
template and nothing else:
`--image` names an `mvm` custom template,
`--kernel-release` names the release it must boot,
and with an image named the recipe checks what the template already carries
instead of installing Debian packages and building the module from source.
Without `--image` every step is what it was,
and the two wrapper drivers forward the two flags to the recipe that owns their guest.
Commit `93486b3f96` carries the change.

One guest preparation was made by hand and is not the recipe's:
a 6 GiB `/swapfile`,
the size the step 16 settling driver itself creates.
The recipe's own build of the wrapper,
at two compiling jobs in step 16's arm,
filled the guest's 7.9 GiB and the kernel killed a `rustc` at 5.06 GiB inside
`qemu-guest-agent.service`,
which `mvm exec` reports as a lost result
(`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`).
With the swap file the same build finished.
Step 8's driver and step 16's settling driver already create that file themselves.

The image sets one fixed hostname,
`cachyos-7.3.5-1-zfs-2.4.4`,
and `mvm` names a guest only in its own bookkeeping:
nothing in this template renames the guest the way the Ubuntu image's cloud-init does.
Every driver guards its arms with `test "$(hostname)" = <guest name>`,
so a guest made from this template must be renamed to its `mvm` name before any arm runs:
`printf '%s\n' <name> > /etc/hostname` and `hostname <name>` in the guest.
That was done in both guests,
and it is the one preparation the drivers do not do for themselves.
Installing `cloud-init` in the template would remove it,
and it is the template's gap rather than a driver's.

## The arms

### Step 2's arms: the live suites of the binding

The root suite ran 250 tests,
249 passed and one failed,
9 skipped:

```text
# keep/guest/step-2-binding-live-suites/root-suite-full.log
Summary [ 8.408s] 250 tests run: 249 passed, 1 failed, 9 skipped
FAIL  compatibility_check::live_gate_tests::the_running_modules_declaration_answers_what_the_arm_expects
```

Every live arm that opens `/dev/zfs` ran;
none returned with a `SKIP live_` line.
The measurements they printed name the fixture and the results:

```text
MEASUREMENT gate expected=compatible release=2.4.4-1 module_btf=570234 verdict=compatible base=6579400 module=570234
MEASUREMENT host pair: base 6579400 bytes (header 32), module 570234 bytes (header 32)
MEASUREMENT guest kernel=7.3.5-1-cachyos module=2.4.4-1 srcversion=A3553C308C29FD4B6595A72
MEASUREMENT userspace=zfs-2.4.4-1 zfs-kmod-2.4.4-1
MEASUREMENT reply initial=262144 requested=553024 properties=174 value_bytes=3000
MEASUREMENT recordsize=32768 source=zfsstep2-record
MEASUREMENT before-promote checkout snapshots=[... @earliest, @origin, @checkout-later ...] bookmark=(8026697813883247965, 187)
MEASUREMENT after-promote checkout origin=.../promotion@origin promoted origin='' moved=[...]
MEASUREMENT promoted-batch own=...@own-one,...@own-two carried=0 refused=Some(17) elements=1 kept=[...]
MEASUREMENT dependent clone origin=.../promotion@with-dependent
MEASUREMENT replacement guids before=362142037970935851 after=1766502869204319390
MEASUREMENT key-unloaded clone keystatus=1 origin=.../promotion@origin
MEASUREMENT key-loaded keystatus=2 (1 is unavailable)
MEASUREMENT encryption=8 keyformat=1
MEASUREMENT grants clone-parent ul$1000=["create", "mount"] ud$1000=["destroy", "mount", "userprop"] checkout ul$1000=["snapshot"] ud$1000=["clone", "destroy", "mount"]
MEASUREMENT snapshot-ioctl created=2 carried=0 names=...
```

The account suite is item 1's shape by the crate's own route:
nine tests,
all nine passed.
The driver's `run` step stops when the root suite fails,
so it never reached the account half;
this suite ran through the driver's own account script instead,
its `chown`,
its two copies of the toolchain home,
its `runuser` and its `nextest` filter,
copy for copy,
and its log is `guest-account-suite.log` beside the driver's root-suite log.

```text
# keep/guest/step-2-binding-live-suites/account-suite-full.log
Summary [ 0.126s] 9 tests run: 9 passed, 250 skipped
MEASUREMENT account promoted mixed-batch refused=Some(17) elements=1 kept=6
MEASUREMENT account promoted own-batch destroyed=2 carried=0 kept=4
MEASUREMENT account clone with-props origin=.../checkout@checkout-later accepted=1
MEASUREMENT account snapshot created=1 carried=0 guid=11924478808735516666
```

`the_account_destroys_two_own_snapshots_in_one_batch` passed there,
which is item 1's call on this release:
the account holding exactly the proposed grants destroyed two snapshots made on the
unpromoted clone in one request,
with an empty per-name list.
It printed no measurement line of its own;
the batch it submits is the one item 1 records
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
"Item 1").

The one failure is the gate arm's own assertion,
not the gate's verdict:

```text
MEASUREMENT gate expected=compatible release=2.4.4-1 module_btf=570234 verdict=compatible base=6579400 module=570234
thread '...the_running_modules_declaration_answers_what_the_arm_expects' panicked at src/compatibility_check/live_gate_tests.rs:105:13:
assertion `left == right` failed: a 6.8 guest's tools write the oldest header
  left: 32
 right: 24
```

The layout checker answered `compatible` on this module,
which is what the arm was set to expect.
The assertion after it pins the module's BTF header length to `24`,
the length the Ubuntu `6.8` guest's own tools write,
and the packaged `linux-cachyos-zfs` module declares the current `32`
(as the base file does).
The checker accepts both,
which its `mod_tests` and `comparison_tests` prove,
so the refusal is the arm's pin and not the module.
Another module's test is not this round's to change,
so the mismatch is left standing and named here.

### Step 16's snapshot batch

Three arms,
all three passed,
`keep/guest/step-16-snapshot-batch`:

```text
Summary [ 0.235s] 3 tests run: 3 passed, 10645 skipped
MEASUREMENT snapshot-batch requests=3 names=6 budget-bytes=164 encoded-bytes=492
PASS  snapshot_batch_arm_one_request_destroys_the_whole_set
PASS  snapshot_batch_arm_a_budget_splits_the_set_across_requests
PASS  snapshot_batch_arm_a_hold_names_one_and_keeps_every_name
```

The pool was `zfsstep16` on a 1 GiB image,
`aes-256-gcm` with a raw key,
`recordsize=32K`,
`compression=zstd-3`,
legacy mountpoints and an unmounted clone parent,
and no fixture mount was left standing:

```text
# keep/guest/step-16-snapshot-batch/step16-teardown.log
mounts left below the step 16 fixture root:
```

The arm's first attempt ended in that out-of-memory kill;
the numbers above are the run after the swap file was added.

### Step 16's settling command

Its own driver ran `prepare` and `run` on a guest of its own,
`zfs-step-23-qualification-settling`,
and the arm passed
(`keep/guest/step-16-settling-command`):

```text
Summary [ 0.389s] 1 test run: 1 passed, 10646 skipped
SETTLING stands=refused
SETTLING gone=settled records=3
SETTLING checkout=records-only registration=kept
SETTLING done
```

So a clone that stands refuses the settling command and nothing is removed,
a clone a person destroyed outside the wrapper is settled with its registration and its three
records,
and a slot whose checkout has begun keeping its registration,
its pointer and the person's file keeps them while only the records go.
That guest's own teardown was empty,
because the arm destroys the pool it made
(`keep/guest/step-16-settling-command/teardown-evidence.log`):

```text
== pools standing before teardown
== datasets standing before teardown
== mounts below the arms' fixture roots before teardown
```

### Step 8's mount control

The sweep's own step ran alone,
as its driver provides for a round whose arm is one of them
(`keep/guest/step-8-mount-sweep`),
against a store dataset `git-policy-cli-store/store` on a file-backed pool:

```text
Summary [ 177.624s] 2 tests run: 2 passed (1 slow), 1358 skipped
MEASUREMENT sweep-attach-ms=159
MEASUREMENT child-bound-listing-ms=10659 children=4097 bound=4096
```

The two numbers are what step 8's measurement record times,
and the arm's 24 `SWEEP` lines are its assertions.
The ones this release could have answered differently are these:

```text
SWEEP attached item=1 slot=one origin=checkout-snapshot mount-row=310 303 0:61 / /root/sweep-checkout/.claude/worktrees/one rw,nosuid,nodev,relatime - zfs ... rw,xattr,noacl,casesensitive
SWEEP first-attach outcome=attached slot=one
SWEEP attached twice=1 slot=one standing=true
SWEEP second-request answered=done mounts=1 nonce=ffeeddccbbaa99887766554433221100 mount=310
SWEEP no-configuration shape=no-file attached slot=one
SWEEP several=two-pools refused slot=one mounted=false
SWEEP refusal item=5 step=key-unavailable receipt=refused
SWEEP refusal item=2 step=intent
SWEEP refusal item=2 step=clone-origin
SWEEP rollback unmount=EBUSY mount=331 standing=true
SWEEP rollback unmount=ok mount=331 standing=false
SWEEP bound stopped=true children=4097
SWEEP stale flags-changed=refused slot=c705
SWEEP stale replaced=refused slot=c705 reused=true attempt=1
SWEEP stale swapped=refused slot=c705 source=.../c705-moved-away
SWEEP done pool=git-policy-cli-store layout=design slot=one
```

So the mount the handler attaches stands at the slot the request names,
a second request is answered from the standing mount rather than mounted twice,
the unloaded key refuses before a mount,
both intent refusals hold,
a busy unmount is told apart from a completed one,
the 4096-child bound stops the sweep,
and every stale-mount shape is refused.

### Step 9's namespace and unmount controls

The same driver's whole run repeated the sweep and then ran step 9's two arms
(`keep/guest/step-9-namespace-and-unmount`).
Every arm passed:

```text
# adoption-arm.log, plan step 4's arm beside them
Summary [ 0.315s] 4 tests run: 4 passed, 1356 skipped
ADOPTION recorded device=65025 inode=1199144
ADOPTION refused=wrong-device
ADOPTION refused=link
ADOPTION refused=mode-widened
ADOPTION refused=slot-differs
# guest-sweep.log
Summary [ 224.943s] 2 tests run: 2 passed (1 slow), 1358 skipped
# guest-holder-scan.log, the namespace control
Summary [ 214.534s] 1 test run: 1 passed (1 slow), 1359 skipped
# guest-unmount.log
Summary [ 1.883s] 4 tests run: 4 passed, 1356 skipped
```

The namespace control's own lines name each namespace it could and could not reach:

```text
SCAN baseline own=1 holders=0 incomplete=0 guid=4731537578919153748
SCAN HOLDER other-namespace=1 namespace=mnt:[4026532408] mount=/root/holder-scan-checkout/.claude/worktrees/one
SCAN H1 intent=mount holders=2 at-the-slot=2 own=1 clear=false
SCAN H1 intent=unmount holders=2 at-the-slot=2 own=1 clear=false
SCAN unreadable-link incomplete=250 holders=0 clear=false
SCAN thread-namespace found=true namespace=mnt:[4026532410] thread=/proc/36287/task/36288
SCAN leader-ended found=true namespace=mnt:[4026532409] leader-link=unreadable leader-state=Z
SCAN chrooted-lowest found=true namespace=mnt:[4026532410] lowest-pid=36309 lowest-omits-mount=true
SCAN confined-consumer=held holders=2 namespace=mnt:[4026532410]
SCAN churn scans=20 holder-kept=20 task-ended-named=0
MEASUREMENT holder-scan-ms=small=3 large=7 tasks-small=247 tasks-large=447
SCAN detached-mount invisible=true holders=0 destroy-refused=true refused-while-pinned=true
SCAN detached-mount after-release=destroyed
SCAN pinned-namespace invisible=true holders=0 incomplete=0 note=the-scan-cannot-see-a-namespace-no-task-holds
```

So the built handler found the holder in another mount namespace and in each of the shapes the
arm makes,
counted the unreadable link as incomplete rather than as clear,
found nothing through a namespace no task holds and said so,
and measured a scan of about 250 and about 450 live tasks at 3 and 7 milliseconds.
The unmount arm's lines cover the plain unmount,
the busy one that reattaches and ends its holder,
two at once,
the reopened handle,
and four kill points:

```text
UNMOUNT plain=unmounted receipt=done destroy=ok
UNMOUNT busy=reattached holder=ended
UNMOUNT reattach-then-unmounted=true
UNMOUNT two-at-once=both-unmounted
UNMOUNT handle-reopened=true
UNMOUNT kill=staged finished=unmounted
UNMOUNT kill=between-unmounts finished=unmounted
UNMOUNT kill=bind-over-slot finished=unmounted
UNMOUNT kill=none finished=unmounted
UNMOUNT kill-points=done
```

The store's publication probe ran too,
because the run includes it.
Its lines are the ones step 7's measurement record reads,
and they answer the same way here:
a publication that fsyncs the file and its folder moves the device on every sample,
and the same append without an fsync moves it on none:

```text
PROBE publication of one slot record: stage, write, fsync, link, unlink, folder fsync: n=60 ms min=0.484 median=0.783 max=2.826 | samples where vda1 moved=60 | samples with a ZIL commit of the home dataset=60 | median ZIL commits=2.0
PROBE same append without fsync, window as short as the round's control: n=60 ms min=0.008 median=0.013 max=0.792 | samples where vda1 moved=0 | samples with a ZIL commit of the home dataset=0
```

The long rows above are elided in the middle where a row is a mount row;
every line's first field is verbatim.

### The teardown evidence

The guest was destroyed through `mvm` after the arms
(`keep/guest/mvm-destroy.log`),
and `mvm list` then reported `no VMs found`
(`keep/guest/mvm-list-after-all-guests.log`).
The settling command's guest was destroyed the same way
(`keep/guest/step-16-settling-command/mvm-destroy.log`).
Before it went,
the guest was asked what stood in it
(`keep/guest/teardown-evidence.log`):
two file-backed pools of the arms' own,
`git-policy-cli-store` and `zfsstep2-account`,
the datasets under them,
and four mounts:

```text
== pools standing before teardown
git-policy-cli-store
zfsstep2-account
== mounts below the arms' fixture roots before teardown
165 40 0:55 / /home/zfs-step-2-fixture/zfsstep2-account/home rw,relatime shared:235 - zfs zfsstep2-account/home rw,xattr,posixacl,casesensitive
275 165 0:56 / /home/zfs-step-2-fixture/zfsstep2-account/home/checkout rw,relatime shared:242 - zfs zfsstep2-account/home/checkout rw,xattr,posixacl,casesensitive
303 40 0:60 / /root/holder-scan-checkout rw,relatime - zfs git-policy-cli-store/step9/monochromatic-git-checkout rw,xattr,noacl,casesensitive
310 40 0:61 / /root/sweep-checkout rw,relatime - zfs git-policy-cli-store/step8/monochromatic-git-checkout rw,xattr,noacl,casesensitive
```

They are the arms' checkouts and the account fixture,
which those arms leave attached on purpose so a later arm finds the layout they made;
none of them is a host mount,
and the disk they stood on is gone with the guest.
The host was never asked to change anything,
so there is no teardown of host state to show.

## What differs from the results of the earlier guests

- The kernel release.
  Every earlier guest in this effort booted `6.8.0-147-generic`
  and carried OpenZFS `2.4.4` built from source inside it;
  this one boots `7.3.5-1-cachyos` and carries the distribution's module,
  built for exactly that kernel,
  and the distribution's userspace.
  Both report `2.4.4-1`,
  and every live arm that passed above passed on the host's own release.
- The module's BTF header length is `32` here and was `24` in the Ubuntu guest,
  because the module's own build tools write it.
  The layout checker answers `compatible` on both;
  the gate arm's assertion expects the `24` its own guest writes,
  and failed on the difference.
- `cargo-nextest` is `0.9.151` here and the recipe pins `0.9.152`.
  The packages are the distribution's,
  and `rustc` is the pinned `1.100.0`,
  so the rejection of nextest's archive route is not read here:
  the recipe installs nextest only when the guest lacks it.
- The guest has no swap of its own,
  and the wrapper's release test build needs more than
  the 7.9 GiB it has:
  `rustc` reached 5.06 GiB of anonymous memory before the kernel killed it.
  The two drivers that build that crate already create a 6 GiB swap file for this reason;
  step 16's batch arm through the binding's recipe does not,
  and needed the file added.
- The item 1 fixture's grants reply printed the same shape the measurement record has,
  `ud$1000=["destroy", "mount", "userprop"]` on the clone parent and
  `ul$1000=["snapshot"] ud$1000=["clone", "destroy", "mount"]` on the checkout,
  so the delegation root made of the design's list is the one the arms ran under.
- The promoted batch,
  the mixed batch with a dependent clone and the replacement under a
  new clone name answered what items 17,
  18 and 12 record on the Ubuntu guest:
  `0` and an empty per-name list for the account's own snapshots,
  `EEXIST` (`17`) with every name kept for the set holding a dependent clone,
  and a replacement that gets a new GUID.
- A guest's hostname.
  `mvm`'s Ubuntu image is renamed by its cloud-init seed,
  and this template has no cloud-init,
  so a guest boots the image's fixed hostname and must be renamed before a driver's
  `hostname` guard lets its arms run.

## What this run did not do, and why

- It did not run the gate arm's patched-module expectations
  (`member:dds_flags`,
  `member:zi_iotype`,
  `type:zc_inject_record`).
  Those are step 6's arms and they rewrite the module to refuse it,
  which step 23's paragraph does not ask for,
  and the arm as written pins its header length to the Ubuntu guest's.
- It did not run the binding's `run` step to completion.
  That step stops at the root suite's one failing arm,
  so the account suite was run with the driver's own account script,
  as the section above says,
  rather than by the step itself.
- It did not run the allowlist's negative control.
  The project chose the start probe instead,
  and the design says that control is not owed.
- It did not repeat the delegated batch with the earlier round's `python3` harness,
  which drove `lzc_destroy_snaps` directly.
  This image's packages carry no `libzfs_core` Python binding
  (`python3 -c 'import libzfs_core'` refuses),
  so item 1's call went through the binding crate's own request instead,
  which is the "through the binding" form step 2's own measurements ask for.
- Nothing ran on the host's pool,
  and no host `zfs` or `zpool` verb ran at all.
- The guest's own teardown left the step 2 account pool `zfsstep2-account` standing
  and recorded it,
  as the recipe's teardown step does;
  the guest was destroyed with its disks afterwards,
  so nothing of it remains here.

## What is left

- The gate arm's pin to `24`,
  named above.
  Whether it becomes a release-independent assertion or keeps a per-guest expectation
  is the owner's,
  and it is the one thing this run cannot call qualified.
- `cargo-nextest` one release behind the recipe's pin in the template.
- The template is reproducible only from this host's `pacman` cache,
  because the four pinned packages are no longer on any CachyOS mirror.
  A build elsewhere needs them from an archive that still serves them.
- The template's fixed hostname.
  A guest must be renamed to its `mvm` name before a driver runs,
  because the image carries no cloud-init;
  installing it,
  or teaching each driver the rename,
  is a later change.
- The image is not committed anywhere:
  it lives in the image folder the route reads,
  and its build driver is kept in this round's folder rather than in the repository,
  because the plan makes step 23 a run and not a shipped program.
- The owner declares the release qualified,
  or does not.
