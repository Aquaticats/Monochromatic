# zram on Linux 7.2 (Fedora 44 / Bazzite): an active zram swap device cannot be resized live

Growing swap to 64 GiB requires replacing or supplementing the device.

## Symptom

A running system wants more swap than the current zram device provides.
Every attempt to raise the active device's size fails:

```sh
# doc/troubleshooting/zram-live-swap-resize.md
sudo bash -c 'echo 64G > /sys/block/zram0/disksize'
# bash: line 1: echo: write error: Device or resource busy

sudo zramctl -s 64G /dev/zram0
# zramctl: /dev/zram0: failed to reset: Device or resource busy
```

The size is unchanged:

```sh
# doc/troubleshooting/zram-live-swap-resize.md
cat /sys/block/zram0/disksize
# 17179869184
```

`dmesg` carries the kernel's own reason:

```text
zram: Cannot change disksize for initialized device
```

`zramctl -s` fails too,
 but for a different reason:
 it resets the device first,
 and reset is refused while the device has openers
 (`zramctl: /dev/zram0: failed to reset: Device or resource busy`).
The `zramctl` message names the reset step,
 not the size step,
 which sends the reader toward "the device is busy"
 rather than "the size is immutable."
Both messages are separate failure modes and need separate enumeration.

Two surface patterns exist,
 and each produces its own error:

- Direct `disksize` write to an active or initialized device:
  `write error: Device or resource busy`,
  with `zram: Cannot change disksize for initialized device` in `dmesg`.
- `zramctl -s`:
  `failed to reset: Device or resource busy`,
  because reset is rejected before the size write is attempted.

## Root cause

`disksize` is documented as read-write and the `-EBUSY` return code is documented,
but the documentation does not say that "initialized" is a permanent state for the device's lifetime,
which is what makes the surface look like a permissions or busy problem.

The kernel documentation states the attribute is read-write:

```text
# Documentation/ABI/testing/sysfs-block-zram:1-8
What:		/sys/block/zram<id>/disksize
Date:		August 2010
Contact:	Nitin Gupta <ngupta@vflare.org>
Description:
		The disksize file is read-write and specifies the disk size
		which represents the limit on the *uncompressed* worth of data
		that can be stored in this disk.
		Unit: bytes
```

and lists `-EBUSY` with a remedy that reads as though it were a transient condition:

```text
# Documentation/admin-guide/blockdev/zram.rst:45-46
-EBUSY	  an attempt to modify an attribute that cannot be changed once
	  the device has been initialised. Please reset device first.
```

"Please reset device first" is misleading here.
`init_done()` is not a flag that a reset clears temporarily;
 it *is* the disksize:

```c
/* drivers/block/zram/zram_drv.c:103-106 */
static inline bool init_done(struct zram *zram)
{
	return zram->disksize;
}
```

So the gate in `disksize_store()` rejects any size,
 larger or smaller,
 on any device that has ever been sized:

```c
/* drivers/block/zram/zram_drv.c:2876-2894 */
static ssize_t disksize_store(struct device *dev, struct device_attribute *attr,
			      const char *buf, size_t len)
{
	unsigned long num_pages;
	u64 disksize;
	struct zcomp *comp;
	struct zram *zram = dev_to_zram(dev);
	int err;
	u32 prio;

	disksize = memparse(buf, NULL);
	if (!disksize)
		return -EINVAL;

	guard(rwsem_write)(&zram->dev_lock);
	if (init_done(zram)) {
		pr_info("Cannot change disksize for initialized device\n");
		return -EBUSY;
	}
```

This is why the `zramctl` path fails one step earlier.
`reset_store()` refuses while the device is open,
 and an active swap device is held open by the swap subsystem:

```c
/* drivers/block/zram/zram_drv.c:2945-2952 */
	zram = dev_to_zram(dev);
	disk = zram->disk;

	mutex_lock(&disk->open_mutex);
	/* Do not reset an active device or claimed device */
	if (disk_openers(disk) || zram->claim) {
		mutex_unlock(&disk->open_mutex);
		return -EBUSY;
	}
```

The same guard covers the compression algorithm and the backing device,
 so a config-only change to either is refused on a live device for the same reason.

 The two gates compose into a closed loop:
growing the size needs the device uninitialized,
uninitializing needs a reset,
and reset needs the device closed,
which an active swap device never is.
Only `swapoff` breaks the loop.
The earlier hypothesis that a "resize" ioctl or a larger `disksize` write would be honored on a live device was wrong;
the probe below disproves it directly.

The size is also not free memory in one respect that matters for planning:
the per-page metadata table is allocated up front and scales with `disksize`,
 not with usage:

```c
/* drivers/block/zram/zram_drv.c:1997-2004 */
static bool zram_meta_alloc(struct zram *zram, u64 disksize)
{
	unsigned long num_pages;

	num_pages = disksize >> PAGE_SHIFT;
	zram->table = vzalloc(array_size(num_pages, sizeof(*zram->table)));
	if (!zram->table)
		return false;
```

Each entry is two `unsigned long`s,
 so 16 bytes on x86_64:

```c
/* drivers/block/zram/zram_drv.h:80-91 */
struct zram_table_entry {
	unsigned long handle;
	union {
		unsigned long __lock;
		struct attr {
			u32 flags;
#ifdef CONFIG_ZRAM_TRACK_ENTRY_ACTIME
			u32 ac_time;
#endif
		} attr;
	};
};
```

A 64 GiB device therefore reserves a 64 GiB / 4 KiB * 16 B = 256 MiB metadata table immediately at size-set time.
Compressed page storage is allocated on demand and is separate.
Before this was measured,
"larger zram is nearly free until used" was the working assumption;
it is true of the data pool and false of the table.

## Verification

Kernel and userspace under test:

```text
uname -r:                 7.2.0-ogc6.1.fc44.x86_64
/proc/version:            #1 SMP PREEMPT_DYNAMIC Mon Aug 24 03:42:13 UTC 2026
systemd:                  259 (259.8-1.fc44)
zram-generator:           1.2.1-5.fc44.x86_64
btrfs-progs:              v7.1
util-linux zramctl:       shipped with Fedora 44 util-linux
kernel zram built:        CONFIG_ZRAM=m, CONFIG_ZRAM_MULTI_COMP=y
```

Source revision cited throughout is `torvalds/linux` master at commit `9b87fdc9af2fbfcdb5c24a64139685ef80f6573f`
(kernel `7.3.0-rc3`);
 the installed 7.2.0 kernel was probed directly at runtime and its behavior matched.
`systemd` source is `systemd/systemd` main.

The harness below uses a **throwaway** zram device created through the control interface,
so the running system's swap is never touched.
Every probe in this document was run this way,
 except the ones explicitly marked as reading the live device.

### Probe 1: uninitialized device accepts a size (positive control)

A positive control is required before any "the write failed" result means anything.

```sh
# doc/troubleshooting/zram-live-swap-resize.md
NEW=$(sudo cat /sys/class/zram-control/hot_add)
echo "created zram$NEW"

sudo bash -c "echo 256M > /sys/block/zram$NEW/disksize"
sudo cat /sys/block/zram$NEW/disksize
```

```text
created zram1
268435456
```

The write succeeds on the fresh device,
 confirming the interface works and the failure is state-dependent.

### Probe 2: the same write on the active device fails

```sh
# doc/troubleshooting/zram-live-swap-resize.md
sudo mkswap "/dev/zram$NEW" >/dev/null
sudo swapon "/dev/zram$NEW"

sudo bash -c "echo 512M > /sys/block/zram$NEW/disksize"
sudo cat /sys/block/zram$NEW/disksize
sudo dmesg | tail -1
```

```text
bash: line 1: echo: write error: Device or resource busy
268435456
zram: Cannot change disksize for initialized device
```

The same bytes that worked in probe 1 now fail.
This is the catalog for the failing pattern.

### Probe 3: initialized but inactive still fails, and reset then works

This separates "active" from "initialized."
The device is sized and never swapped on.

```sh
# doc/troubleshooting/zram-live-swap-resize.md
NEW=$(sudo cat /sys/class/zram-control/hot_add)
sudo bash -c "echo 128M > /sys/block/zram$NEW/disksize"
sudo cat /sys/block/zram$NEW/initstate      # 1

sudo bash -c "echo 256M > /sys/block/zram$NEW/disksize"
# write error: Device or resource busy

sudo bash -c "echo 1 > /sys/block/zram$NEW/reset"    # succeeds, inactive
sudo cat /sys/block/zram$NEW/disksize                # 0
sudo bash -c "echo 512M > /sys/block/zram$NEW/disksize"   # accepted
sudo cat /sys/block/zram$NEW/disksize                # 536870912
```

The resize fails while merely initialized,
 and the reset succeeds once inactive,
 which proves `swapoff` is the real prerequisite and "busy" is not the mechanism.

### Probe 4: `zramctl -s` reports the reset step, not the size step

```sh
# doc/troubleshooting/zram-live-swap-resize.md
sudo zramctl -s 256M "/dev/zram$NEW"
```

```text
zramctl: /dev/zram1: failed to reset: Device or resource busy
```

Confirms the second error variant in the symptom catalog.

### Probe 5: metadata cost is immediate, data cost is not

```sh
# doc/troubleshooting/zram-live-swap-resize.md
# before
awk '/MemFree|VmallocUsed/' /proc/meminfo
sudo bash -c "echo 64G > /sys/block/zram$NEW/disksize"
# after
awk '/MemFree|VmallocUsed/' /proc/meminfo
```

```text
before: MemFree: 8326892 kB   VmallocUsed: 257544 kB
after:  MemFree: 8104240 kB   VmallocUsed: 520232 kB
```

The observed delta is about 217 MiB of `MemFree` and about 256 MiB of `VmallocUsed`,
 against a predicted 256 MiB table for 64 GiB.
This is the measured cost of setting a size without storing a single page.

### Probe 6: the compression algorithm is gated the same way

The same `init_done()` guard protects other attributes,
 including the compression algorithm:

```c
/* drivers/block/zram/zram_drv.c:1686-1693 */
	if (!alg)
		return -EINVAL;

	guard(rwsem_write)(&zram->dev_lock);
	if (init_done(zram)) {
		pr_info("Can't change algorithm for initialized device\n");
		return -EBUSY;
	}
```

```sh
# doc/troubleshooting/zram-live-swap-resize.md
sudo bash -c "echo zstd > /sys/block/zram$NEW/comp_algorithm"
```

```text
bash: line 1: echo: write error: Device or resource busy
zram1: detected capacity change from 0 to 524288
zram: Can't change algorithm for initialized device
```

This matters for the configuration route:
changing `compression-algorithm` in the generator config and restarting the unit fails the same way,
 which is the failure reported as `systemd/zram-generator#119`
 (`Failed to configure compression algorithm into /sys/block/zram0/comp_algorithm ... Device or resource busy`).
The four attributes gated by `init_done()` are
`disksize`,
`comp_algorithm`,
`backing_dev`,
and `compressed_writeback`;
of these,
`disksize` and `comp_algorithm` are the two a normal configuration change would touch.

### Patterns that work

- Sizing a fresh device from `hot_add`,
   before any `mkswap`/`swapon`
  (probe 1).
- `reset` on an initialized but inactive device,
   then sizing it
  (probe 3).
- `swapoff` on an active device,
   then `reset`,
   then sizing.

### Patterns that fail

- `disksize` write on an active device:
   `Device or resource busy` plus
  `zram: Cannot change disksize for initialized device`.
- `disksize` write on an initialized but inactive device:
   the same message.
- `zramctl -s` on an active device:
   `failed to reset: Device or resource busy`.
- `comp_algorithm` write on an initialized device:
   `Device or resource busy` plus
  `zram: Can't change algorithm for initialized device`.

## Verified workarounds

Each workaround names what it costs.

### Workaround 1: replace the zram device live, then persist the config

This changes the size in a running system.
The cost is that the old swap contents are discarded,
 so the work needs enough free RAM for whatever it is currently holding.

```sh
# doc/troubleshooting/zram-live-swap-resize.md
# 1. Stop using the old device.
#    This faults its live contents back into RAM, so confirm headroom first.
sudo swapoff /dev/zram0

# 2. Reset it so the size becomes writable again.
sudo bash -c 'echo 1 > /sys/block/zram0/reset'

# 3. Set the new size.
sudo bash -c 'echo 64G > /sys/block/zram0/disksize'

# 4. Re-create and re-activate the swap signature.
sudo mkswap /dev/zram0
sudo swapon -p 100 /dev/zram0
```

Tradeoffs:
step 1 needs free RAM roughly equal to the old device's **uncompressed** content,
 which on a compressed-and-full device can be far larger than the compressed bytes it occupies;
step 2 discards contents even if step 1 fails partway;
step 3 reserves the metadata table immediately (about 256 MiB for 64 GiB),
 and on a compressed device the pool can grow to roughly `disksize / compression ratio`,
 so a 64 GiB device can consume several GiB of RAM under pressure.
Setting the size does not allocate the pool.

Persist it so it survives reboot.
The generator reads the first file present from
`/run/systemd/zram-generator.conf`,
 `/etc/systemd/zram-generator.conf`,
 `/usr/local/lib/systemd/zram-generator.conf`,
 `/usr/lib/systemd/zram-generator.conf`,
 so the local config in `/etc` is the one to change:

```ini
# /etc/systemd/zram-generator.conf
[zram0]
compression-algorithm=zstd
zram-size = min(ram / 2, 65536)
```

The generator's own documented apply procedure is `systemctl daemon-reload` then
`systemctl start /dev/zram0`,
 which is the boot-time path and therefore also a full replace,
 not a live resize.

### Workaround 2: add a second swap device instead of growing the first

A second zram device is created without touching the first,
 so no swap data is discarded and no RAM is needed to hold the old contents during the change.

```sh
# doc/troubleshooting/zram-live-swap-resize.md
NEW=$(sudo cat /sys/class/zram-control/hot_add)
sudo bash -c "echo 64G > /sys/block/zram$NEW/disksize"
sudo mkswap "/dev/zram$NEW"
sudo swapon -p 50 "/dev/zram$NEW"
```

Tradeoffs:
the kernel's swap preference is by priority,
 and the existing device keeps priority 100,
 so new pages go to the old device first until it fills;
the extra device adds its own metadata table,
 so two devices reserve two tables;
and this is not persistent unless a second `[zramN]` section is added to the generator config.
This option is the one that avoids the `swapoff` requirement entirely.

### Workaround 3: a disk-backed swapfile, for a machine with free disk

This is the route to a genuinely large swap area that does not compete for RAM.
On this machine it was verified working end to end.
The target filesystem matters,
 and the constraints are enforced by the kernel at `swapon` time.

```sh
# doc/troubleshooting/zram-live-swap-resize.md
# btrfs-progs creates the file with NODATACOW set and a valid signature.
sudo btrfs filesystem mkswapfile --size 64G /var/lib/swap/swapfile

sudo swapon /var/lib/swap/swapfile
swapon --show
```

```text
NAME                                 TYPE  SIZE USED PRIO
/dev/zram0                           partition 16G 16G 100
/var/lib/swap/swapfile               file      64G   0B   -1
```

Tradeoffs,
all measured on this machine:

- **It costs real disk,
   immediately.**
  The file must be preallocated ("swapfile - must be preallocated (i.e. no holes)"),
  so a 64 GiB swapfile costs 64 GiB of the filesystem's free space.
  Measured on `/var`:
   writing a 64 GiB swapfile before activation moved
  `Data,single` from `Size: 912688939008, Used: 858452963328` to
  `Size: 981408415744, Used: 858452963328`,
  a new chunk allocation of about 64 GiB,
  and `df` went from `863G used / 1.1T avail` to `927G used / 973G avail`.
  This is not a sparse file.
- **Snapshots conflict.**
  btrfs refuses to snapshot a subvolume holding an active swapfile,
  and a subvolume containing one cannot be snapshotted,
  so placing it in a snapshotted subvolume breaks snapshots.
  Put it in a subvolume that is not on the snapshot schedule.
- **Boot ordering.**
  A swapfile on a volume unlocked later in boot (for example a LUKS volume opened by a
  service on first login) is not available at swap activation time,
  and hibernation cannot use it unless it is unlocked before resume.
- **Not for hibernation here.**
  Hibernation needs the kernel to write RAM to the swap device and then find it again at
  resume via `/sys/power/resume` and `resume_offset`;
  `/sys/power/resume` is `0:0` on this machine and the root filesystem is LUKS,
  so resuming would need the initrd to unlock the volume first.
  A swapfile on the encrypted data volume cannot serve resume as configured.
- **Encrypted volume compression.**
  A mount option of `compress=` on the filesystem is harmless for correctness because
  btrfs excludes `NODATACOW` inodes from compression,
  but do not create the file by other means (plain `fallocate`,
   `dd`) and expect it to work.

#### Applied configuration on this machine

The 64 GiB target was met by keeping the 16 GiB zram device and adding a 48 GiB swapfile,
 so the split is 16 GiB of compressed RAM swap first and 48 GiB of disk swap after it.

```text
# /etc/fstab (appended; pri=10 keeps it below zram0's priority 100)
/var/lib/swap/swapfile none swap pri=10 0 0
```

The file lives on the `var` subvolume and not on `home`,
 because `snapper` snapshots `home`
 (`snapper list-configs` reports `root` -> `/var/home`)
 and an active swapfile there would make every timeline snapshot fail.
`/var/lib/swap` is also not in any `tmpfiles.d` rule,
 so nothing ages the file out;
 `/var/tmp` would have been the wrong choice,
 since `tmp.conf` carries `q /var/tmp 1777 root root 30d`.

Persistence comes from the fstab entry rather than a hand-written unit,
 and the generated unit is wired correctly:

```text
$ systemctl show var-lib-swap-swapfile.swap -p RequiredBy -p After -p SourcePath
RequiredBy=swap.target
Before=swap.target umount.target
After=system.slice var.mount systemd-remount-fs.service -.mount systemd-journald.socket
SourcePath=/etc/fstab

$ ls /run/systemd/generator/swap.target.requires/
dev-zram0.swap  var-lib-swap-swapfile.swap
```

Two details are worth knowing before copying this.
`systemd-fstab-generator` places fstab swap entries in `swap.target.requires`,
 not `swap.target.wants`,
 so looking only at `swap.target.wants` makes the entry appear to be missing.
And `After=var.mount` is the ordering that matters:
 without it the unit would race the mount and fail on a cold boot.

SELinux does not need a custom label for this.
The file inherits `var_lib_t` from its directory,
 Fedora's policy defines no `swapfile_t` for regular swapfiles,
 and activation produced no `avc` denials.

##### End-to-end verification

Swap devices are cheap to list and expensive to trust,
 so the file was exercised rather than merely activated.
A bounded container (512 MiB RAM cap,
 4 GiB memory-plus-swap cap)
 wrote a 2 GiB byte pattern,
 held it resident,
 and read it back.
The caps force roughly 1.5 GiB of that pattern out of RAM,
 and with the zram device already full the kernel had to place it in the swapfile.

```text
[1] wrote 2048 MiB under a 512m RAM cap, holding 40s
[2] swap while holding:
Filename                                Type      Size      Used      Priority
/dev/zram0                              partition 16777212  16777124  100
/var/lib/swap/swapfile                  file      50331644  1790728   10
[3] read-back verification: mismatched chunks=0
[4] exit=0
```

`Used` of 1790728 KiB on the swapfile is about 1.71 GiB arriving there under real pressure,
 and zero mismatched chunks is the read path returning identical bytes.
Together they show write and read both work,
 which listing the device cannot show.
The container was bounded on purpose:
 the caps keep a deliberate swap-forcing test from becoming a host-wide memory event.

### What a swapfile must satisfy on btrfs

The kernel checks these at activation and warns with specific strings.
This is the catalog of the checks,
 from `btrfs_swap_activate()`:

```c
/* fs/btrfs/inode.c:10344-10358 */
	if (BTRFS_I(inode)->flags & BTRFS_INODE_COMPRESS) {
		btrfs_warn(fs_info, "swapfile must not be compressed");
		ret = -EINVAL;
		goto out_unlock_mmap;
	}
	if (!(BTRFS_I(inode)->flags & BTRFS_INODE_NODATACOW)) {
		btrfs_warn(fs_info, "swapfile must not be copy-on-write");
		ret = -EINVAL;
		goto out_unlock_mmap;
	}
	if (!(BTRFS_I(inode)->flags & BTRFS_INODE_NODATASUM)) {
		btrfs_warn(fs_info, "swapfile must not be checksummed");
		ret = -EINVAL;
		goto out_unlock_mmap;
	}
```

and the placement checks:

```c
/* fs/btrfs/inode.c:10522-10538 */
		if (map->type & BTRFS_BLOCK_GROUP_PROFILE_MASK) {
			btrfs_warn(fs_info,
				   "swapfile must have single data profile");
			ret = -EINVAL;
			goto out;
		}

		if (device == NULL) {
			device = map->stripes[0].dev;
			ret = btrfs_add_swapfile_pin(inode, device, false);
			if (ret == 1)
				ret = 0;
			else if (ret)
				goto out;
		} else if (device != map->stripes[0].dev) {
			btrfs_warn(fs_info, "swapfile must be on one device");
			ret = -EINVAL;
			goto out;
		}
```

The "one device" rule is the one that depends on luck.
It does not require a single-device filesystem;
 it requires every extent of the file to live on the same device,
 and for a multi-device filesystem with unallocated space on both,
 the first extent may land on either.
This was reproduced both ways on throwaway filesystems:

```text
# balanced 3G+3G, data single: every extent landed on loop1 -> activation rejected
BTRFS warning (device loop1): swapfile must be on one device
swapon: /var/mnt/swapprobe2/swapfile: swapon failed: Invalid argument

# unbalanced 1G + 9G, data single: extents all landed on one device -> activation accepted
Adding 2097148k swap on /var/mnt/swapprobe2/sw.  Priority:-1 extents:1 across:2097148k SS
```

So on a multi-device pool,
 making the file work is a matter of which device its extents happen to land on,
 and the failure mode is a warning in `dmesg` plus a bare `Invalid argument` from `swapon`.
A file that activates today can be pinned differently after a balance,
 and a balance cannot run while the swapfile is active
 (`fs_info->swapfile_pins` blocks it),
 so an active swapfile freezes the block groups holding its extents.

## What does not work

- **Writing a larger `disksize` to the live device.**
  Rejected with `-EBUSY`;
  `init_done()` is the disksize itself,
   so this is not a transient state.
- **Using the documented `-EBUSY` remedy as written.**
  "Please reset device first" cannot be followed on an active swap device,
  because `reset_store()` requires no openers.
- **`zramctl -s` as a resize.**
  It resets first,
   and reset is refused on an active device;
  the resulting message points at reset,
   not at size.
- **Changing `compression-algorithm` in the generator config and restarting the unit.**
  The algorithm is gated by the same `init_done()` check,
  so the restart fails on a live device before it ever reaches the size.
- **Expecting `disksize` to be the RAM cost.**
  The metadata table is `num_pages * sizeof(struct zram_table_entry)`,
  allocated at size-set time.
- **`swapoff` on a full zram device as a free operation.**
  It faults the device's uncompressed contents back into RAM.
  On the machine studied,
   zram held 16 GiB of uncompressed data in about 8 GiB of compressed
  storage,
   and `MemAvailable` was about 15 GiB,
  so the swapoff needed more RAM than was available and could not be run as a casual step.
- **A swapfile on the multi-device root pool as an unambiguous fix.**
  It activated in testing,
   but activation depends on extent placement that the user does not control.
- **A swapfile on `tmpfs` (`/tmp` on this machine).**
  `swapon` needs a real block-backed file;
   `tmpfs` files have no fixed physical extents.
- **Enlarging host swap to fix cgroup OOM kills.**
  Separate failure,
   see below.

## Relationship to the container and cgroup OOM kills on this machine

This is a distinct incident from the resize limitation and must not be folded into it.

On this machine the journal showed 15 kernel OOM kills and 6 `systemd-oomd` kills over three days,
 and **every one named a memory cgroup**:

```text
oom-kill:constraint=CONSTRAINT_MEMCG,oom_memcg=/user.slice/.../libpod-....scope,task=tsgolint
systemd-oomd: Killed /user.slice/.../app.slice/claude-code-bash due to memory used (60312891392) / total (67002466304) and swap used (16207028224) / total (17179865088) being more than 90.00%
```

Neither is the same failure as "swap is too small":

- The kernel kills carry `constraint=CONSTRAINT_MEMCG`,
  so they are cgroup-limit kills,
   not global-`OOM` kills.
  Raising the host's total swap does not raise a container's `memory.max`.
- `podman` with `--memory=X` and no `--memory-swap` sets the cgroup's `memory.swap.max` to the same X,
  so the container cannot use host swap at all.
  Measured:

  ```text
  --memory=256m                        -> memory.max=268435456  memory.swap.max=268435456
  --memory=256m --memory-swap=1g       -> memory.max=268435456  memory.swap.max=805306368
  --memory=256m --memory-swap=-1       -> memory.max=268435456  memory.swap.max=max
  ```

  A `--memory=256m` container therefore gets 256 MiB of swap headroom total,
  and it OOMs inside that budget regardless of how much swap the host has.

The `systemd-oomd` kills are governed by a host-wide threshold rather than any single cgroup's limit:

```c
/* src/oom/oomd-manager.c:477-479 */
        /* Check amount of memory available and swap free so we don't free up swap when memory is still available. */
        if (oomd_mem_available_below(&m->system_context, 10000 - m->swap_used_limit_permyriad) &&
                        oomd_swap_free_below(&m->system_context, 10000 - m->swap_used_limit_permyriad)) {
```

with

```c
/* src/oom/oomd-util.c:157-164 */
bool oomd_swap_free_below(const OomdSystemContext *ctx, int threshold_permyriad) {
        uint64_t swap_threshold;

        assert(ctx);
        assert(threshold_permyriad <= 10000);

        swap_threshold = ctx->swap_total * threshold_permyriad / (uint64_t) 10000;
        return (ctx->swap_total - ctx->swap_used) < swap_threshold;
}
```

The threshold is a **percentage of total swap**.
`/etc/systemd/oomd.conf.d/20-freeze-hardening.conf` on this machine sets `SwapUsedLimit=90%`,
 and the kills fired at `swap used (16207028224) / total (17179865088)`,
 which is 94.3 percent,
 above the 90 percent line.

That means the two possible responses are not equivalent:

- Raising total swap makes the 90 percent line much further away.
  On this machine the applied 64 GiB total puts it at 57.6 GiB of used swap,
  where the old 16 GiB total put it at 14.4 GiB,
  and a fully used 16 GiB zram device went from 100 percent of total to 25 percent.
  so it suppresses this specific `systemd-oomd` trigger.
  It does **not** add real memory,
  so if the pressure that filled swap continues,
   the system now has 64 GiB of slow
  zram-backed thrash before the same threshold is reached,
  which converts fast OOM kills into long stalls.
- It does nothing for the `CONSTRAINT_MEMCG` container kills,
  which are the larger group.

None of this makes enlarging swap wrong;
 it makes "swap is too small" the wrong diagnosis for the cgroup kills,
 and enlarging swap an indirect lever on the oomd kills rather than a fix.

## Upstream filing artifact

Nothing to file.
The resize restriction is deliberate kernel architecture,
 not a defect,
 and the documentation gap is a wording issue that upstream has addressed in context.

### Upstream filing decision

1.  Upstream fault:
     no.
    The behavior is an intentional invariant.
    `init_done()` is the disksize,
    so a live resize would have to reallocate the per-page table while slots on the device
    are in use and while `slots` are addressed by `unsigned long` index,
    and it would have to decide what happens to a shrink.
    The `-EBUSY` return is a deliberate guard,
     and the kernel's own docs list it.
    The remaining friction is documentation wording,
    which is behavior-adjacent,
     not behavior.
2.  Fixability:
     yes,
    a live-resize path is technically constructible
    (grow-only,
     refusing shrink,
     reallocating under the write lock).
    This constraint therefore passes,
    which is why the prototype stage below was evaluated rather than skipped.
3.  Supported use:
     not asserted by upstream.
    The documented flow is reset-then-size,
     and zram-generator's documented apply procedure is
    a service restart,
     which is a full replace.
    No upstream statement supports live growth of an active device.
4.  Contribution policy:
     no ban found.
    `torvalds/linux` accepts patches via the mailing lists,
    and `systemd/zram-generator` accepts issues and pull requests;
    neither has a policy prohibiting AI-assisted reports.
    The `gh` CLI could not reach `api.github.com` during this investigation
    (repeated `error connecting to api.github.com`),
    so tracker searches were done through the public issue pages,
    direct `api.github.com` fetches,
    and web search instead.
5.  Upstream direction:
     leans no.
    No upstream request for live resize of an active device was found.
    Two nearby threads exist and neither asks for it:
    `systemd/zram-generator#7`
    is a systemd double-start race,
    not a resize request,
    and it was closed by adding `RemainAfterExit=yes` to the generated unit
    (commit `b2a7c8db000`);
    the `Cannot change disksize` line was a symptom of the second start,
    not the subject.
    `systemd/zram-generator#78`
    asks how to apply config changes and is answered with
    `systemctl restart systemd-zram-setup@zramN`,
    which is a replace,
     not a live resize.
    The 2014 commit `be2d1d56c82d`
    ("zram:
     drop `init_done' struct zram member")
    shows the guard is long-standing,
    though the helper keyed on different state then than it does now,
    so the current `disksize`-based form has its own later history that this investigation did not trace.
6.  Prototype:
     not pursued,
     on purpose.
    Constraints 1 and 3 do not hold:
    the behavior is not a defect and upstream does not advertise the use case,
    and with constraint 5 leaning no,
    a grow-only resize patch would be an architectural change that upstream has not asked for
    and is not needed to solve this machine's problem,
    which workaround 1 and workaround 2 both solve at the configuration layer.

The `.out-of-scope/` catalog was checked;
 no exemption matches zram,
 swap,
 btrfs swapfiles,
 or `systemd-oomd`.

Nothing is filed,
 and no comment is drafted.
The candidate additive content was the observation that
`comp_algorithm` is gated by the same check as `disksize`,
 so a config-only algorithm change also needs a reset;
that does advance `systemd/zram-generator#119`'s 2026 report slightly,
 but the mechanism is visible in the kernel source the maintainers already reason from,
 and the report's own first comment thread shows the packaging error was fixed.
The documented `-EBUSY` wording already tells the reader to reset the device,
 and `oomd.conf(5)` already states the swap threshold as a percentage of total swap.
The `systemd/zram-generator#7` and `#78` threads already carry their own resolutions.
