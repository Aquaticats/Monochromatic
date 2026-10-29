# Portable VM dev environment

Declarative,
 immutable,
 rebuildable VM image for development.
Built on ucore-hci (Fedora CoreOS + libvirt/KVM/ZFS/cockpit/podman)
with KDE Plasma layered on top via a custom Containerfile.

## Architecture

```text
ucore-hci (immutable, rpm-ostree, Fedora CoreOS base)
  + KDE Plasma, SDDM (baked into custom image)
  + system dev packages (baked into custom image)
  + ghostty RPM via scottames/ghostty COPR (baked into custom image)
  + mise (baked, /usr/local/bin)
  + user account (baked)
  |
  + mise toolchains (first-login, ~/. local/share/mise/)
  + dotfiles (first-login, ~/.config/)
  + librewolf RPM via LibreWolf repo (baked into custom image)
  + secrets (manual, gh auth login / SSH keys)
```

Two artifacts define the entire environment:

1. **Custom container image** (`Containerfile` + TypeScript orchestration in `src/build-and-import.ts`):
    OS layer,
    rarely changes
2. **First-login provisioner** (file-enforcer config in this monorepo):
    user layer,
    changes often

## Decisions made

- **Base image**:
   `ghcr.io/ublue-os/ucore-hci:stable`:
   well-documented,
  has its own GitHub repo,
   includes HCI stack (libvirt,
   KVM,
   ZFS,
   cockpit,
   podman)
- **Not Bazzite**:
   too much gaming bloat for a dev VM
- **Not Fedora Kinoite**:
   poor documentation,
   no dedicated repo,
   includes unwanted defaults
- **Not openSUSE Kalpa**:
   alpha status,
   single maintainer,
   officially unmaintained
- **Not Alpine**:
   no transactional package management (`apk add` mutates root directly)
- **Not mutable distro**:
   immutability provides defense-in-depth against config drift,
  even on short-lived VMs
- **KDE layered on server base**:
   add only what we need rather than subtract what we don't
- **Custom image (not rpm-ostree layering at boot)**:
   faster VM startup,
  reproducible,
   KDE is part of the base not a fragile layer
- **Dotfiles baked into image**:
   `package/config/dotfiles/` package with subdirectories
  (ghostty,
   mise).
   COPYed into `~/.config/` at image build time via widened build context.
  Only two configs needed;
   a separate repo and provisioner step are overkill.
- **KDE config not portable**:
   let KDE regenerate defaults on each VM,
  machine-specific state (display IDs,
   GPU paths,
   monitor geometry) causes more breakage than it prevents
- **No cloud-init**:
   everything is baked into the image or handled by the first-login provisioner
- **Secrets stay manual**:
   `gh auth login` and SSH key setup after provisioning,
  no encrypted vaults or automation for secrets
- **Provisioner lives in this monorepo**:
   versioned alongside the rest of the dev tooling,
  uses file-enforcer primitives (`exec`,
   `overwrite`,
   `cat`)
- **Provisioner triggered manually**:
   run `setup-dev` once after first login,
  no systemd user services or `.bash_profile` hacks
- **Host-level FDE only**:
   rely on host OS full-disk encryption (BitLocker on Windows,
  LUKS on Linux host) rather than per-image encryption.
  ZFS native encryption is not supported at any layer (osbuild,
   bootc,
   bootc-image-builder).
  btrfs-on-LUKS2 through bootc-image-builder is blocked (blueprint schema doesn't expose LUKS;
  bootc's `tpm2-luks` has an open deadlock bug bootc-dev/bootc#2089).
  QEMU LUKS-encrypted qcow2 works but VHDX has no format-level encryption equivalent
  (MS-VHDX spec has zero cipher/key fields),
   breaking vmsync cross-platform parity.
- **plasma-workspace meta-package**:
   full KDE desktop set rather than minimal plasma-desktop,
  since ucore-hci is a server base with no desktop components
- **Provisioner in vm-builder**:
   no `src/provision.ts` or `:provision` task exists yet.
  Dotfiles are baked into the image by `Containerfile`;
   a future first-login
  provisioner still needs to be created if flatpak or user-layer setup remains required.
- **Nvim deprecated by editord**:
   no nvim config clone needed in the provisioner
- **Skip Ignition/Butane**:
   everything is baked into the image or handled by the provisioner;
  Ignition adds no value beyond what the Containerfile already provides
- **Cosign key for future use**:
   set up cosign even though builds are local-only,
  in case images need to be pushed to GHCR later

## Phase 1: custom container image

### Containerfile

Based on [ublue-os/image-template][image-template] pattern.
Uses `ghcr.io/ublue-os/ucore-hci:stable` as base.

Lives at `package/vm-builder/cli/` in this monorepo.
Run via:
 `mise run //package/vm-builder/cli:run`

[image-template]: https://github.com/ublue-os/image-template

- [x] Create package at `package/vm-builder/cli/` in this monorepo
- [x] Write Containerfile (single-stage,
       rpm-ostree install on ucore-hci base)
- [x] Fill in `build-and-import.ts` orchestration (build,
       convert,
       copy to libvirt images,
       import via virsh define)

### System packages to bake in (`Containerfile`)

All packages below are installed and verified in the Containerfile.

KDE Plasma desktop (full `plasma-workspace` meta-package):

- [x] `plasma-workspace`,
   `sddm`,
   `konsole`,
   `dolphin`,
   `virt-manager`
- [x] `graphical.target` set as default (symlink in `/usr/lib/systemd/system/`,
      not `/etc/`:
   ostree discards `/etc/` symlinks during deployment)
- [x] SDDM enabled via symlink in `graphical.target.wants/`
- [x] SDDM auto-login configured for user `user`

Browser + password manager:

- [x] LibreWolf repo (`https://repo.librewolf.net/librewolf.repo`:
       domain changed from `rpm.librewolf.net` in 2026)
- [x] `librewolf`,
   `keepassxc`

Terminal:

- [x] Ghostty COPR repo (scottames/ghostty)
- [x] `ghostty`

Dev tools (system-level,
 mise handles the rest):

- [x] `helix`,
   `strace`,
   `inotify-tools`,
   `python3-devel`,
   `potrace`,
   `msitools`

Infrastructure:

- [x] `mise` installed as user `user` at `~/.local/bin/mise`
- [x] User account `user` with password `password`,
       `wheel` group,
       passwordless sudo
- [x] Dotfiles baked into image from `package/config/dotfiles/` (ghostty,
       mise configs)
- [x] bootc install config:
       btrfs root filesystem (`/usr/lib/bootc/install/00-ucore-dev.toml`)

### Build and convert

- [x] Fill in Containerfile,
       verify `mise run //package/vm-builder/cli:run` succeeds end-to-end
- [x] Write `disk_config/disk.toml` (40 GiB root,
       btrfs)
- [x] Boot the qcow2,
       verify KDE Plasma starts with SDDM auto-login
- [ ] Verify podman runs,
       libvirt works inside the VM
- [ ] Verify all installed apps launch (ghostty,
       librewolf,
       keepassxc,
       helix)

Implementation notes discovered during build:

- qcow2 must be copied to `/var/lib/libvirt/images/` for SELinux `virt_image_t` context
  (files in `$HOME` have `user_home_t` which QEMU's `svirt_t` domain cannot read)
- `getenforce` returns "Disabled" inside the Claude Code sandbox but SELinux is actually enforcing on the host
- Flatpak virt-manager needs `org.virt_manager.virt_manager.Extension.Qemu` for `qemu:///session`
- UEFI domains require `--nvram` flag when undefining via `virsh undefine`
- LibreWolf repo URL changed:
   `rpm.librewolf.net` -> `repo.librewolf.net` (2026)
- `findUp('package.json')` returns relative paths when the file is in cwd;
   wrap with `resolve()`

### Full-disk encryption; decided: host-level FDE

**Decision:
** rely on host OS full-disk encryption instead of per-image encryption.

Research findings (March 2026):

- **ZFS native encryption**:
   not supported at any layer.
  osbuild has zero ZFS stages (`mkfs` stages:
   ext4,
   xfs,
   btrfs,
   fat only).
  bootc's `Filesystem` enum has three variants:
   Xfs,
   Ext4,
   Btrfs.
  ucore-hci excludes ZFS from the initramfs;
   ZFS is runtime-only for `/var` mounts.
  Open issue bootc-dev/bootc#1240 documents the failure.
- **btrfs-on-LUKS2**:
   osbuild has `org.osbuild.luks2.format` stages internally,
  but the blueprint `config.toml` schema does not expose LUKS as a partition type.
  bootc's `--block-setup tpm2-luks` has a semaphore deadlock bug (bootc-dev/bootc#2089).
- **QEMU LUKS-encrypted qcow2**:
   works (supported since QEMU 2.10,
   XTS-AES-256),
  but VHDX has no format-level encryption (MS-VHDX spec rev 8.0 has zero cipher fields).
  Microsoft's Shielded VMs use guest-side BitLocker,
   not VHDX encryption.
  This breaks vmsync's cross-platform sync parity.

- [x] ~~Determine if bootc-image-builder supports ZFS pool encryption~~:
   no,
   not at any layer
- [x] ~~Evaluate alternatives~~:
   host-level FDE chosen over per-image encryption

### Ignition/Butane config; decided: skip

Ignition adds no value beyond what the Containerfile already provides.
The user account is baked in,
 and SSH keys are handled manually post-boot.

- [x] ~~Decide if Ignition adds value~~:
   no,
   skipped

## Phase 2: first-login provisioner

No `src/provision.ts` or `:provision` mise task exists yet.
 Dotfiles are baked into the
image by `Containerfile`,
 and image-time mise installs only the global tools declared in
`package/config/dotfiles/mise/config.toml`.
 A future first-login provisioner still needs
to be created if flatpak or user-layer setup remains required.

### Dotfiles package

Lives at `package/config/dotfiles/` in this monorepo.
Only two configs needed:
 ghostty and mise.

- [x] Create `package/config/dotfiles/` with:
  - `ghostty/config`
  - `mise/config.toml` (global mise config)
- [x] ~~Archive `Aquaticat/nvim`~~:
   nvim is deprecated by editord in the monorepo
- [x] ~~Decide which `~/.config/` directories to include~~:
   only ghostty and mise

### Provisioner script tasks

Runs as user after first login.
 Idempotent (safe to re-run).

- [x] ~~Copy dotfiles~~:
   baked into image at build time via Containerfile COPY
- [x] Run image-time `mise install` for baked global config tools in `Containerfile`
- [ ] Decide whether a future first-login provisioner should run monorepo `mise install` after clone
- [ ] Install flatpaks via `flatpak install --user -y`:
      Flatseal,
       Fastmail,
       Gear Lever,
       KColorChooser,
       KeePassXC,
      Nextcloud,
       OBS,
       RustDesk,
       Ungoogled Chromium

### Provisioner trigger sequence

```bash
# 1. Manual secrets setup
gh auth login
ssh-keygen -t ed25519  # or copy key from USB

# 2. Clone monorepo
gh repo clone Aquaticat/Monochromatic ~/Monochromatic

# 3. Install dev toolchains (mise is pre-installed in image)
cd ~/Monochromatic
mise trust && mise install

# 4. Future, not implemented yet: run first-login provisioner if one is added
# mise run //package/vm-builder/cli:provision
```

## Phase 3: verification

- [x] Boot fresh VM from qcow2
- [x] Verify KDE Plasma starts with SDDM auto-login
- [ ] Run provisioner sequence end-to-end
- [ ] Verify mise tools work:
       `bun --version`,
       `node --version`,
       `cargo --version`
- [ ] Verify podman works:
       `podman run --rm alpine echo hello`
- [ ] Verify libvirt works:
       `virsh list --all`
- [ ] Verify ghostty launches (installed as RPM,
       not AppImage)
- [ ] Verify host-level FDE protects the qcow2 at rest
- [ ] Verify librewolf launches
- [ ] Verify KeePassXC launches
- [ ] Verify `rpm-ostree install <package>` stages for next boot and activates after reboot
- [ ] Destroy VM,
       rebuild from scratch,
       confirm full reproducibility

## Phase 4: iteration workflow

Once the base image is stable,
 day-to-day changes fall into two categories:

**OS-level changes** (rare):
 edit Containerfile or TypeScript build orchestration,
 rebuild image,
 convert to qcow2.
Examples:
 new system package,
 display manager config,
 kernel module.

**User-level changes** (frequent):
 edit provisioner or dotfiles,
 re-run provisioner.
Examples:
 new mise tool,
 editor config change,
 new dotfile.
No image rebuild needed:
 just re-run the provisioner in the existing VM.

## Open questions

- [x] ~~Where does the provisioner live?
  ~~:
   future provisioner belongs in the vm-builder package;
   no file exists yet
- [x] Custom image repo:
       `package/vm-builder/cli/` in this monorepo
- [x] ~~KDE Plasma package set~~:
   `plasma-workspace` meta-package (full desktop)
- [x] ~~Which flatpaks?
  ~~:
   Flatseal,
   Fastmail,
   Gear Lever,
   KColorChooser,
   KeePassXC,
      Nextcloud,
   OBS,
   RustDesk,
   Ungoogled Chromium
- [x] ~~Disk encryption~~:
   host-level FDE only (ZFS/LUKS/qcow2-LUKS all ruled out;
   see Phase 1)
- [x] ~~Nvim repo~~:
   nvim deprecated by editord
- [x] ~~Cosign key setup~~:
   key pair at `package/config/cosign/`,
   signing step in `build-and-import.ts`
