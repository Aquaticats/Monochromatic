# vm-builder

Builds the monochromatic-dev container image,
 converts it to a qcow2 disk image,
and imports it as a libvirt VM accessible from virt-manager.

## Prerequisites

Install these on the host before running:

```sh
sudo dnf install podman libvirt-client virt-install
sudo usermod -aG libvirt $USER   # re-login after this
```

The scripts define the VM through `qemu:///session`;
 root privileges are still
needed for copying the qcow2 into `/var/lib/libvirt/images/`.

## Usage

The image installs OdyTTY `0.17.5` from its upstream x86-64 RPM,
verifies its SHA-256,
and seeds the existing OdyTTY preferences and KDE/XDG terminal defaults.
The version matches the user's interim desktop choice;
change the version and checksum together when updating it.
The Ghostty COPR is no longer configured.

Build the image without replacing a VM:

```sh
# package/vm-builder/cli
mise run //package/vm-builder/cli:image:build
```

The optional `--build-arg FEDORA_DIRECT_MIRROR=true` uses Fedora's direct download endpoint
for the Fedora release and updates repositories when a selected mirror delays the build.
Package signature checks remain enabled.
This opt-in also selects that endpoint for those repositories in the resulting VM image;
the default build retains Fedora's metalink mirror selection.

`mise run //package/vm-builder/cli:image:check` runs the installed OdyTTY as the image's `user`
in disposable containers with networking disabled,
2 CPUs,
2 GiB of memory,
and the lowest disk weight (`--blkio-weight` and `--cgroup-conf io.weight=`,
both from the `lowest_disk_weight` variable of `mise.toml`,
which a test checks against the shared
`@monochromatic-dev/container-limit-core` constant;
these rootful containers sit outside the task's unit,
so the launcher's `IOWeight` never reaches them).
It prints the version,
effective OdyTTY preferences,
KDE/XDG defaults,
and desktop entry for inspection.
It does not boot or replace a VM.

The full build,
conversion,
and import task follows.
It replaces an existing `monochromatic-dev` VM as described in Re-running.

```bash
mise run //package/vm-builder/cli:run
```

The script runs four steps in sequence:

1. **Build**:
    `sudo podman build` produces `localhost/monochromatic-dev:latest`
   from `Containerfile` in rootful storage for bootc-image-builder compatibility
2. **Convert**:
    `bootc-image-builder` converts the image to
   `output/qcow2/disk.qcow2` (rootful `sudo podman`,
    privileged,
    at the lowest disk weight,
   with 12 GiB,
   8 CPUs and 2048 processes,
   from `src/container-arguments.ts`)
3. **Fix ownership**:
    restores `output/` from root:
   root to current user
4. **Import**:
    `virt-install --import` registers the qcow2 as a libvirt domain

After the script finishes,
 start the VM:

```bash
virsh --connect qemu:///session start monochromatic-dev
# or open virt-manager and double-click the VM
```

## Re-running

The script is idempotent.
 If a VM named `monochromatic-dev` already exists,
it is forcibly stopped and undefined before the new image is imported,
whatever state it is in:
`virsh undefine` removes the definition of an active domain and leaves it running,
so only a domain with no QEMU process behind it skips the stop
([`libvirt-undefine-leaves-an-active-domain-running.md`](../../../doc/troubleshooting/libvirt-undefine-leaves-an-active-domain-running.md)).
The `output/` directory is recreated if absent.

## Configuration

VM defaults (edit `src/build-and-import.ts` to change):

- Memory:
   16 GiB
- vCPUs:
   8
- Disk:
   `output/qcow2/disk.qcow2` (qcow2,
   grown from 40 GiB root)
- Network:
   `default` (NAT via libvirt)
- Display:
   SPICE + QXL

Disk layout is controlled by `disk_config/disk.toml`.
See the TODO comments in that file for ZFS encryption status.

## Files

- `Containerfile`:
   OS image definition (ucore-hci base,
   KDE,
   apps,
   dev tools,
   user,
   mise,
   dotfiles)
- `disk_config/disk.toml`:
   bootc-image-builder disk layout
- `src/build-and-import.ts`:
   orchestration script
- `src/exec.ts`:
   private process runner for the probes and `virsh`/`flatpak` calls
   (direct spawn through `nano-spawn`,
   stdout returned,
   `SubprocessError` on non-zero exit,
   signal kill,
   or spawn failure;
   optional `timeoutMs`)
- `output/`:
   build artifacts (gitignored)
