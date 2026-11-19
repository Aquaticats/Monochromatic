# CachyOS kernel upgrade without a reboot deletes the running kernel's modules, so dm_crypt, nft and veth cannot load

Status at 2026-11-18T12:12:38-05:00:
agent-written;
not reviewed or approved by the owner.
The workaround was used for one boot and leaves nothing behind;
a reboot into the new kernel is the real fix,
and it has not happened yet.

## Symptom

On 2026-11-18 the machine had booted `7.3.4-1-cachyos` that morning.
Opening the old LUKS drives with `cryptsetup open --readonly` then failed on every attempt:

```text
device-mapper: reload ioctl on old_nvme_main (252:0) failed: Invalid argument
device-mapper: reload ioctl on old_nvme_expansion (252:0) failed: Invalid argument
```

The kernel log named the missing target:

```text
[ 3786.434524] device-mapper: table: 252:0: crypt: unknown target type
[ 3786.434528] device-mapper: ioctl: error adding target to table
```

Other first uses failed the same way,
each because its module was not loaded yet:

- `mount -o ro,loop .../Archive.sqfs`:
  `failed to set up loop device` (`loop`, `squashfs`).
- `ip link add fwt0 type veth peer name fwt1`:
  `Error: Unknown device type.` (`veth`).
- OpenSnitch's nftables rules needed `nfnetlink_queue`,
  `nft_queue`,
  `nft_ct`,
  `nft_log` and `nft_chain_nat`,
  none of which was loaded;
  of the netfilter modules checked,
  only `nf_tables`,
  `nft_compat`,
  `nft_limit` and `nf_conntrack` were,
  for ufw.

Modules loaded before the upgrade kept working.

## Root cause

`/var/log/pacman.log` shows the kernel package upgraded during the session,
after the boot:

```text
[2026-11-18T10:50:01-0500] [ALPM] upgraded linux-cachyos (7.3.4-1 -> 7.3.5-1)
[2026-11-18T10:50:04-0500] [ALPM] upgraded linux-cachyos-headers (7.3.4-1 -> 7.3.5-1)
[2026-11-18T10:50:04-0500] [ALPM] upgraded linux-cachyos-zfs (7.3.4-1 -> 7.3.5-1)
```

The module tree lives inside the kernel package
(`pacman -Ql linux-cachyos` lists `/usr/lib/modules/7.3.5-1-cachyos/vmlinuz` and the `.ko.zst` files beside it),
so the upgrade removed the 7.3.4 files like any other package's old files.
Afterwards:

```console
$ uname -r
7.3.4-1-cachyos
$ ls /usr/lib/modules
6.18.60-2-cachyos-lts
7.3.5-1-cachyos
```

`modprobe` and the kernel's on-demand loader look in `/usr/lib/modules/$(uname -r)`,
which no longer exists,
so every module the running kernel had not loaded yet is unavailable until a reboot.
`/proc/config.gz` confirms each failing feature is a module on this kernel,
not built in:

```text
CONFIG_DM_CRYPT=m
CONFIG_NETFILTER_NETLINK_QUEUE=m
CONFIG_NFT_QUEUE=m
CONFIG_NFT_CT=m
CONFIG_NFT_LOG=m
```

This is how pacman upgrades work,
not a CachyOS defect.

## Verification

Kernel `7.3.4-1-cachyos` (running) and `7.3.5-1-cachyos` (installed),
2026-11-18.
The matching old package was still in the cache:
`/var/cache/pacman/pkg/linux-cachyos-7.3.4-1-x86_64_v4.pkg.tar.zst`.

Before the workaround,
`dmesg` showed the `crypt: unknown target type` lines in "Symptom",
and `lsmod` listed none of the modules involved.
After it,
`lsmod` listed `dm_crypt`,
`nfnetlink_queue`,
`nft_queue`,
`nft_ct`,
`nf_conntrack_netlink`,
`nft_reject_inet`,
`nft_log`,
`nft_chain_nat`,
`nf_nat`,
`nft_nat`,
`nft_masq`,
`nft_redir`,
`nft_fib_inet` and `veth`.
The drives then opened and mounted read-only,
OpenSnitch loaded its nftables rules,
and the `veth` pair for its inbound test was created.
`loop` and `squashfs` were not loaded,
because the archive image was not needed.

Every extracted module reported the running kernel's version magic,
for example:

```console
$ modinfo -F vermagic .../kernel/drivers/md/dm-crypt.ko.zst
7.3.4-1-cachyos SMP preempt mod_unload
```

## Verified workarounds

### Load modules from the cached package, without touching the system

Extract the running kernel's package into a private scratch root,
build its module index there,
and point `modprobe` at it,
which also resolves dependencies
(`dm_crypt` needs `encrypted-keys`,
`trusted`,
`tee` and `asn1_encoder`):

```bash
# load-running-kernel-modules-from-cache.sh
set -euo pipefail
root="${HOME}/temp/agent/running-kernel-modules-$(uname -r)"
mkdir --parents "${root}"
cd -- "${root}"
bsdtar -xf "/var/cache/pacman/pkg/linux-cachyos-$(uname -r | sed 's/-cachyos$//')-x86_64_v4.pkg.tar.zst" \
  "usr/lib/modules/$(uname -r)"
# depmod -b reads <base>/lib/modules, not <base>/usr/lib/modules
ln -sfn usr/lib lib
depmod -b "${root}" "$(uname -r)"
modprobe -d "${root}" --show-depends dm_crypt
sudo modprobe -d "${root}" dm_crypt
```

Without the `lib` link,
`depmod -b` fails with
`could not open directory .../lib/modules/7.3.4-1-cachyos: No such file or directory`.
The extracted tree took 172 MiB on 2026-11-18.

Tradeoffs:

- Modules load only when asked for by name;
  automatic loading,
  such as a new USB device or a filesystem's first mount,
  still fails until the reboot.
- The package must be the exact build that is running.
  Check `modinfo -F vermagic` against `uname -r` before loading;
  the pacman cache may already have pruned it.
- The modules stay loaded until the reboot,
  which also ends the workaround.
  Nothing is written outside the scratch root.

### Reboot into the new kernel

This is the real fix:
the new kernel's modules are installed.
On this machine commit and push first,
because a reboot can make data on its SSD unreadable.

### Keep the old modules across upgrades (not installed)

Arch's `extra/kernel-modules-hook` package
(`pacman -Si kernel-modules-hook` reports version `0.1.7-3`)
saves the running kernel's module directory before a kernel upgrade and restores it afterwards.
Its pacman hooks trigger on `usr/lib/modules/*/vmlinuz`,
which `linux-cachyos` ships.
The pre-transaction hook copies `/usr/lib/modules/$(uname -r)` to a backup
([10-linux-modules-pre.hook][hook-pre]):

```ini
# 10-linux-modules-pre.hook
[Trigger]
Operation = Install
Operation = Upgrade
Type = Path
Target = usr/lib/modules/*/vmlinuz

[Action]
Description = Saving Linux kernel modules...
When = PreTransaction
Depends = rsync
Exec = /bin/sh -c 'KVER="${KVER:-$(uname -r)}"; if test -e "/usr/lib/modules/${KVER}"; then rsync -AHXal --delete-after "/usr/lib/modules/${KVER}" /usr/lib/modules/backup/; fi'
```

and the post-transaction hook copies it back
([10-linux-modules-post.hook][hook-post]).
The package also installs a cleanup service and tmpfiles entry,
`linux-modules-cleanup`.
It was not installed or tested on this machine.
Tradeoff:
the restored directory is no longer owned by any package.

## What does not work

- `cryptsetup open` retries:
  every attempt fails the same way while `dm_crypt` cannot load.
- Plain `depmod -b <root>` on a tree extracted to `<root>/usr/lib/modules`:
  it looks under `<root>/lib/modules` and fails.

## Upstream filing decision

Nothing to file.
`.out-of-scope/` (checked 2026-11-18) has no pacman or kernel entry,
and no upstream search was run,
because the upstream-filing constraints fail at the first:

1.  Is it upstream's fault?
    No.
    pacman removing an old package version's files on upgrade is its normal operation,
    and the kernel package owns the module tree.
2.  Can upstream fix it?
    Not applicable.
3.  Is the use case supported?
    The supported path is rebooting after a kernel upgrade,
    or installing `kernel-modules-hook`.
4.  Would the repository welcome the contribution?
    Not evaluated.
5.  Will they likely fix it?
    Not evaluated.
6.  Is there a prototype?
    Not applicable;
    `kernel-modules-hook` already exists.

[hook-pre]: https://github.com/saber-nyan/kernel-modules-hook/blob/d5bb3c3750b8e9f7a45a9e3e7893b42e66cbd084/10-linux-modules-pre.hook#L1-L11
[hook-post]: https://github.com/saber-nyan/kernel-modules-hook/blob/d5bb3c3750b8e9f7a45a9e3e7893b42e66cbd084/10-linux-modules-post.hook#L1-L12
