# vm-builder

Builds the monochromatic-dev container image,
 converts it to a qcow2 disk image,
and imports it as a libvirt VM accessible from virt-manager.

## Prerequisites

Install these on the host before running:

```sh
sudo dnf install podman libvirt-daemon-kvm libvirt-client
```

`libvirt-client` provides `virsh`,
which defines and removes the domain,
and `libvirt-daemon-kvm` brings the QEMU session daemon those calls reach.
`virt-install` is not needed:
the pipeline renders the domain XML itself and registers it with `virsh define`.
When virt-manager is installed as a Flatpak,
the pipeline also calls `flatpak` to grant it access to `output/`,
and skips that step with a warning when the `flatpak` probe fails.

The scripts define the VM through `qemu:///session`,
so they reach the user's own session daemon rather than the system libvirt service.
Root privileges are still needed for the image build,
the privileged conversion,
and copying the qcow2 into `/var/lib/libvirt/images/`.

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
The verified invocation is in
[`vm-builder-fedora-metadata.md`](../../../doc/troubleshooting/vm-builder-fedora-metadata.md).

`mise run //package/vm-builder/cli:image:check` runs the installed OdyTTY as the image's `user`
in disposable containers with networking disabled,
2 CPUs,
2 GiB of memory,
128 processes,
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

The task runs seven steps in sequence:

1.  **Build**:
    `sudo podman build` produces `localhost/monochromatic-dev:latest`
    from `Containerfile`,
    with the monorepo root as the build context,
    in rootful storage for bootc-image-builder compatibility.
    The build is bounded to 12 GiB of memory and 8 CPUs
    (`--memory=12g --cpu-period=100000 --cpu-quota=800000`,
    from `src/container-arguments.ts`);
    `podman build` takes neither a process cap nor a disk-weight option.
2.  **Convert**:
    `bootc-image-builder` converts the image to `output/qcow2/disk.qcow2`
    (rootful `sudo podman run`,
    `--privileged`,
    at the lowest disk weight,
    with 12 GiB,
    8 CPUs and 2048 processes,
    from `src/container-arguments.ts`),
    reading `disk_config/disk.toml` from a read-only `/config`
    and the rootful container store from `/var/lib/containers/storage`.
    The `output/` directory is created if absent.
3.  **Fix ownership**:
    `sudo chown --recursive` returns `output/` to the current user
    after the rootful steps.
4.  **Copy to the libvirt images directory**:
    `sudo cp` places the qcow2 at `/var/lib/libvirt/images/monochromatic-dev.qcow2`,
    whose SELinux type lets QEMU read it,
    and a second `sudo chown` gives the copy to the current user.
5.  **Remove an existing domain**:
    `virsh --connect qemu:///session dominfo` probes for the domain,
    `domstate` decides whether `destroy` precedes `undefine --nvram`,
    and a failed probe skips the removal with a warning.
6.  **Define the new domain**:
    the pipeline writes `output/domain.xml`
    and registers it with `virsh --connect qemu:///session define`.
    Nothing starts the VM.
7.  **Grant the virt-manager Flatpak access**:
    `flatpak override --user` adds a `--filesystem` entry for the package's
    `output/` directory to `org.virt_manager.virt-manager`,
    only after the `flatpak info` probe of that Flatpak succeeds.

After the script finishes,
 start the VM:

```bash
virsh --connect qemu:///session start monochromatic-dev
# or open virt-manager and double-click the VM
```

## Re-running

The task is idempotent.
If a VM named `monochromatic-dev` already exists,
it is forcibly stopped and undefined before the new domain is defined,
whatever state it is in:
`virsh undefine` removes the definition of an active domain and leaves it running,
so only a domain with no QEMU process behind it skips the stop
([`libvirt-undefine-leaves-an-active-domain-running.md`](../../../doc/troubleshooting/libvirt-undefine-leaves-an-active-domain-running.md)).
A failed `virsh dominfo` probe reads as an undefined domain,
so the removal is skipped with a warning.
The `output/` directory is recreated if absent.

`:run` copies the new qcow2 into `/var/lib/libvirt/images/` before it removes an old domain,
so the file a running domain has open is replaced first;
`:import` removes the domain before it copies.

## Re-importing without rebuilding

`mise run //package/vm-builder/cli:import` skips the build and the conversion:
it removes an existing `monochromatic-dev` domain,
copies `output/qcow2/disk.qcow2` to `/var/lib/libvirt/images/`,
defines the domain from a freshly rendered `output/domain.xml`,
and grants the virt-manager Flatpak access to `output/`.
Use it after `:run` has produced the qcow2 at least once.

## Pushing to GHCR

`mise run //package/vm-builder/cli:push` tags `localhost/monochromatic-dev:latest`
as `ghcr.io/aquaticat/monochromatic-dev:latest` with `sudo podman tag`,
pushes that tag with `sudo podman push`,
then signs it with `cosign sign` through `sudo env COSIGN_PASSWORD=`,
using the key at `package/config/cosign/cosign.key`
and `--tlog-upload=false` so the signature stays off the transparency log.
Signing is a separate task because a cosign signature is an OCI artifact
that has to reach a registry.
It needs an image built by `:run`,
`cosign` on the `PATH` that `sudo` uses,
and a completed `podman login ghcr.io`.

## Configuration

The domain XML is rendered by `src/domain-xml.ts`
from constants in `src/build-and-import.ts` and `src/import.ts`:

- Domain name:
   `monochromatic-dev`
- Memory:
   16384 MiB (16 GiB)
- vCPUs:
   8
- Machine:
   `kvm` with an x86_64 `hvm` guest,
   ACPI,
   and a UTC clock
- Firmware:
   UEFI (`<os firmware="efi">`),
   booting from the hard disk
- CPU:
   host-passthrough
- Disk:
   a file-backed qcow2 on the virtio bus as `vda`,
   sourced from `/var/lib/libvirt/images/monochromatic-dev.qcow2`,
   which is the copy of `output/qcow2/disk.qcow2`
- Network:
   QEMU user-mode networking (`<interface type="user">`) with a virtio model,
   not libvirt's `default` NAT network,
   so the guest reaches the host through QEMU's own NAT
- Display:
   SPICE with GL enabled,
   and a virtio video model with 3D acceleration
- Channel:
   a unix socket at `org.qemu.guest_agent.0` for the QEMU guest agent

Change memory and vCPUs in both `src/build-and-import.ts` and `src/import.ts`,
which repeat the same constants,
and every other value in `src/domain-xml.ts`.
The root filesystem has a minimum of 40 GiB,
from `disk_config/disk.toml`,
which also carries the TODO comments on ZFS encryption status.

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
- `mise.toml`:
   the package's tasks,
   including `run`,
   `image:build`,
   `image:check`,
   `import` and `push`
- `src/build-and-import.ts`:
   orchestration script for the full pipeline
- `src/import.ts`:
   the import steps alone,
   for a qcow2 that already exists
- `src/domain-xml.ts`:
   the libvirt domain XML both entry scripts define
- `src/domain-removal.ts`:
   decides whether `virsh destroy` has to precede `virsh undefine --nvram`
- `src/container-arguments.ts`:
   arguments and limits of the two rootful `podman` commands,
   so a test checks them without running either
- `src/exec.ts`:
   process runner for the probes and `virsh`/`flatpak` calls that need captured stdout
   (direct spawn through `nano-spawn`,
   stdout returned,
   `SubprocessError` on non-zero exit,
   signal kill,
   or spawn failure;
   optional `timeoutMs`)
- `src/logger.ts`:
   the tagged logger `exec` logs every command through
- `src/sign-and-push.ts`:
   tags,
   pushes and cosign-signs the image for GHCR
- `output/`:
   build artifacts (gitignored)
