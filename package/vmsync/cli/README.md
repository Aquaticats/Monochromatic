# cli-vmsync

Multi-hypervisor VM image manager with incremental block-level sync.
Manages disk images across KVM (qcow2) and Hyper-V (vhdx) formats,
automatically syncing changes after each boot session.

## Usage

```bash
# Import an existing disk image (any format qemu-img supports)
vmsync import alpine-3.21-cloud.qcow2
vmsync import fedora-server.raw --name fedora-dev

# Boot a VM (auto-detects KVM on Linux, Hyper-V on Windows)
vmsync boot alpine

# Manually sync if boot was interrupted
vmsync sync alpine

# Inspect VM state
vmsync status alpine
vmsync list

# Update boot settings
vmsync config alpine --memory 8G --cpus 8
```

## How it works

### Import

1. Detects the source image format via `qemu-img info`
2. Validates the image contains an EFI System Partition (UEFI-only requirement)
3. Converts to both `base.qcow2` (KVM) and `base.vhdx` (Hyper-V)
4. Stores both formats in `~/.local/share/vmsync/<name>/` with a `vmsync.jsonc` config

### Boot + incremental sync

**On Linux (KVM):
**

1. Creates a qcow2 overlay backed by `base.qcow2`
2. QEMU boots from the overlay;
    all writes go to the overlay,
    reads fall through to the base
3. After shutdown,
    the overlay contains only changed blocks
4. `qemu-img map` identifies which blocks were written (depth 0 in the overlay)
5. Changed blocks are copied to `base.vhdx` via NBD block-level patching
6. The overlay is committed back into `base.qcow2` and deleted

For a 100 GB disk with 2 GB of changes,
 sync takes ~4 seconds instead of ~5 minutes.

**On Windows (Hyper-V):
**

1. Creates a temporary Gen2 VM pointing to `base.vhdx`
2. Hyper-V boots and writes directly to the vhdx
3. After shutdown,
    the vhdx is compared by checksum
4. If changed,
    a full conversion to qcow2 is performed

Hyper-V checkpoint-based differencing (`.avhdx`) for incremental sync is a future optimization.

### Architecture constraints

- **UEFI only**:
   images must contain an EFI System Partition.
   BIOS/Gen1 is not supported.
- **NAT networking**:
   user-mode networking on KVM (`-netdev user`),
   Default Switch on Hyper-V.
- **Virtio devices**:
   GPU,
   NIC,
   keyboard,
   and mouse use virtio drivers on KVM.
- **Generation 2**:
   Hyper-V VMs are created as Gen2 with Secure Boot disabled.

## Data layout

```text
~/.local/share/vmsync/
  alpine/
    vmsync.jsonc       # config and sync state
    base.qcow2         # KVM-ready image
    base.vhdx          # Hyper-V-ready image
    overlay.qcow2      # transient, exists only during KVM boot
```

## Dependencies

- `qemu-img`,
   `qemu-system-x86_64`,
   `qemu-nbd`:
   image conversion,
   KVM boot,
   block patching
- `nbd` kernel module:
   Linux block device access for incremental sync
- `fdisk`:
   UEFI validation at import time
- `sha256sum` (Linux) / `certutil` (Windows):
   checksum computation
- `powershell`:
   Hyper-V VM management on Windows
- OVMF firmware (`edk2-ovmf` or equivalent):
   UEFI boot on KVM

## Configuration

`vmsync.jsonc` supports JSONC (JSON with `//` and `/* */` comments):

```jsonc
{
  // VM identity
  "name": "alpine",
  "importedFrom": "/tmp/alpine-3.21-cloud.qcow2",
  "importedAt": "2026-03-27T12:00:00.000Z",

  // Disk geometry
  "diskSizeBytes": 2147483648,

  // Boot settings (shared across hypervisors)
  "boot": {
    "memory": "4G",
    "cpus": 4,
  },

  // Managed by the CLI -- do not edit manually
  "state": {
    "lastBootHypervisor": "kvm",
    "lastBootAt": "2026-03-27T14:30:00.000Z",
    "synced": true,
    "checksums": {
      "qcow2": "sha256:abc...",
      "vhdx": "sha256:def...",
    },
  },
}
```
