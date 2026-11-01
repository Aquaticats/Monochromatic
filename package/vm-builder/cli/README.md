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
    privileged)
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
it is forcibly stopped and undefined before the new image is imported.
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
