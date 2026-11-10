# Portable VM dev environment

Reasoning behind the architecture and decisions in [doc/todo/vm-dev-environment.md](../todo/vm-dev-environment.md).

## Why a VM at all

A VM provides a complete,
 bootable,
 self-contained development environment
that runs on any hypervisor.
Unlike dotfile managers or setup scripts that assume a compatible host OS,
a VM image carries the entire OS,
 packages,
 and configuration.
The portability boundary is the hypervisor,
 not the distro.

## Why immutable

Immutability is not just for long-lived systems.
Config drift happens in hours during a debugging session:
a quick `dnf install strace`,
 a manual edit to `/etc/something`,
a package that pulls in unexpected dependencies.
An immutable root prevents this class of accident regardless of VM lifespan.

`rpm-ostree install` provides the escape hatch:
install a package,
 staged for next boot,
 atomic,
 reversible.
The mutation is explicit and tracked rather than silent.

## Why ucore-hci

### Considered and rejected

Bazzite:
 gaming-focused desktop (Steam,
 MangoHud,
 gamescope,
 controller drivers).
Several GB of irrelevant packages in every image.
Starting from a desktop image and removing things is harder than starting
from a server image and adding things.

Fedora Kinoite:
 the obvious immutable KDE choice,
 but poor documentation,
no dedicated GitHub repo,
 and includes default desktop packages we do not want.
Also inherits Fedora's third-party repo fragility:
Terra (developer.
fyralabs.
com/terra) lagged behind a Fedora release,
blocking work for hours at billable rates.

openSUSE Kalpa:
 immutable KDE on Tumbleweed (rolling),
 which sounds ideal.
In practice:
 still alpha after years,
 single maintainer ("sfalken"),
officially listed as experimental.
Aeon (the GNOME variant) is more mature but still only Release Candidate.
Kalpa strongly discourages RPM installs via transactional-update,
pushing everything to Flatpak,
 which conflicts with installing
system-level dev tools as packages.

Alpine Linux:
 musl libc as a forcing function for portable choices is appealing.
Smallest base image,
 fastest package manager,
 no third-party repo dependency.
Disqualified by the lack of transactional package management:
`apk add` mutates the root directly with no staging,
 rollback,
 or atomic application.
No mechanism equivalent to `rpm-ostree install` (stage for next boot).

openSUSE MicroOS (server):
 mature,
 immutable,
 transactional-update with btrfs snapshots.
Viable as a base but would need the same KDE layering as ucore-hci,
without ucore's pre-built HCI stack.

NixOS:
 the most declaratively pure option.
Entire OS defined in a single Nix expression.
Rejected due to concerns about security posture
(historically slow CVE response,
 world-readable Nix store,
community security practices).

### Why ucore-hci won

ucore-hci is Fedora CoreOS with batteries included for hyperconverged infrastructure:
libvirt,
 KVM,
 virsh,
 virt-install,
 cockpit-machines,
 podman,
 ZFS,
 tailscale.

Advantages over the alternatives:

- Well-documented:
   dedicated [GitHub repo][ucore-repo] with issues,
   releases,
   examples
- HCI stack pre-integrated:
   libvirt/KVM for nested virtualization,
  podman for containers,
   ZFS for storage,
   cockpit for web management
- Fedora CoreOS base:
   immutable,
   rpm-ostree,
   automatic staged updates,
  Ignition/Butane for declarative first-boot provisioning
- Minimal:
   server image with no desktop,
   no gaming stack,
   no bloat
- Additive:
   KDE Plasma is layered on top via the Containerfile,
  adding only the packages we choose

[ucore-repo]: https://github.com/ublue-os/ucore

## Why KDE layered on a server base

Starting from a desktop image (Kinoite,
 Bazzite) means inheriting
hundreds of packages chosen for a general-purpose desktop user.
Starting from a server image (ucore-hci) means adding only:
`plasma-desktop`,
 `sddm`,
 `konsole`,
 `dolphin`,
and the specific dev tools needed.

The result is a smaller,
 faster,
 more predictable image
where every installed package was an explicit choice.

## Why two layers (image + provisioner)

Baking everything into one container image was the initial approach.
It fails for several reasons:

- Cargo builds are slow:
   `cargo:coreutils` (with unix features),
  `cargo:fd-find`,
   `cargo:fastmod`,
   `cargo:llmfit`:
  each compiles from source in the Containerfile.
  First build takes 30-60 minutes.
  Any change to the mise config layer invalidates the cache and rebuilds everything.
- Image size explodes:
   system packages + Rust toolchain + Node + Bun + Deno +
  llama.
  cpp + all cargo-installed tools = 15-20+ GB image before conversion.
- Iteration is painful:
   change one dotfile,
   rebuild the container,
  re-run bootc-image-builder (rootful podman,
   slow).
  10+ minute feedback loop for a config tweak.
- Wrong abstraction boundary:
   Containerfiles are good for OS composition
  (swap packages,
   enable services).
   User-level dev environments
  (mise toolchains,
   editor configs,
   dotfiles) are a different problem
  that changes at a different cadence.

The two-layer split:

- Layer:
  Container image;
  What:
  OS,
  KDE,
  system packages,
  mise binary,
  user account;
  Changes:
  Rarely;
  Rebuild cost:
  ~10 min build + convert
- Layer:
  First-login provisioner;
  What:
  mise toolchains,
  dotfiles,
  ghostty,
  librewolf;
  Changes:
  Often;
  Rebuild cost:
  ~5 min run

User-level changes never trigger an image rebuild.
Image changes never require re-provisioning user tools.

## Why file-enforcer for the provisioner

The monorepo already has [file-enforcer](../../package/file-enforcer):
a declarative tool for syncing derived files.
Its primitives map directly to provisioner needs:

- file-enforcer primitive:
  `exec()`;
  Provisioner use:
  Clone repos,
  run `mise install`,
  install flatpaks
- file-enforcer primitive:
  `overwrite()` / `overwriteEach()`;
  Provisioner use:
  Place dotfiles into `~/.config/`
- file-enforcer primitive:
  `cat()`;
  Provisioner use:
  Read dotfile sources
- file-enforcer primitive:
  Content-based write skipping;
  Provisioner use:
  Idempotent re-runs for free
- file-enforcer primitive:
  Platform-aware `exec()` dispatch;
  Provisioner use:
  Cross-platform if needed later

Writing the provisioner as a file-enforcer config
reuses existing infrastructure instead of inventing a new tool.
The only runtime dependency is Node + file-enforcer,
both available after `mise install`.

## Why ZFS native encryption for FDE

ucore-hci ships ZFS.
 Adding LUKS below ZFS would mean two separate encryption layers:
LUKS for block-level FDE,
 then ZFS on top.
ZFS native encryption handles FDE within the same layer that manages snapshots,
send/receive,
 and compression;
 encrypted data stays encrypted in snapshots and
during `zfs send` replication without a separate key-management ceremony.

ZFS native encryption also provides per-dataset granularity:
 swap and `/tmp` datasets
can be unencrypted while `/home` and `/var` are encrypted,
which matches a dev VM's actual threat model (protect user data and secrets,
not kernel modules).

Key management is a passphrase prompt at boot (`keyformat=passphrase`).
The VM is started manually from a hypervisor console,
 so a prompt is not disruptive.
No TPM or auto-unlock mechanism:
 avoids vTPM device requirements and keeps
the setup portable across hypervisors.

## Why ghostty is a COPR RPM, not an AppImage

This section records the original choice.
On 2026-11-02 the user selected OdyTTY as the interim desktop and VM terminal.
The VM now installs OdyTTY's upstream RPM with a verified SHA-256,
and the Ghostty COPR was removed.
The RPM integration rationale remains applicable,
but OdyTTY publishes release RPMs without a package repository,
so its version and checksum must be updated together in the Containerfile.
See [the migration record](../handover/odytty-interim-switch.md).

The original plan was to install ghostty as an AppImage in the first-login provisioner.
The scottames/ghostty COPR provides an official RPM for Fedora,
 which is a better fit:

- Immutable OS alignment:
   an RPM baked into the container image is managed by rpm-ostree
  and updated atomically;
   an AppImage in `~/Applications/` is invisible to the OS and
  requires a separate update mechanism (`appimaged`,
   AM/AppMan,
   or a manual download script)
- No AppImage runtime dependency:
   AppImages require FUSE or `--appimage-extract-and-run`;
  neither is guaranteed in a fresh ucore-hci image
- Simpler provisioner:
   ghostty moves from Phase 2 (first-login) to Phase 1 (image),
  removing three provisioner tasks and one open question
- COPR risk is acceptable:
   scottames/ghostty is a single-package COPR maintained by the
  same person consistently;
   unlike Terra (which repackages upstream software),
   this COPR
  tracks ghostty releases directly

## Why virt-manager is a native RPM, not Flatpak

virt-manager communicates with libvirtd via a Unix socket (`/var/run/libvirt/libvirt-sock`).
Flatpak sandboxing blocks direct socket access by default:
a Flatpak virt-manager cannot reach the host libvirtd without manual Flatseal overrides,
which defeats the goal of a zero-touch provisioned environment.

virt-manager is available in Fedora's official repos and is a natural desktop companion
to the HCI stack already present in ucore-hci (libvirt,
 KVM,
 virsh,
 cockpit-machines).
Baking it into the container image puts VM management at the same layer as the hypervisor
infrastructure it manages,
 and makes it available immediately after first login
without a provisioner step.

## Why LibreWolf and KeePassXC are native RPMs, not Flatpak

KeePassXC's browser integration communicates with LibreWolf
via native messaging (a Unix socket / D-Bus path between the browser extension
and the desktop application).
Flatpak sandboxing breaks this path:
a Flatpak LibreWolf cannot reach a native KeePassXC,
 and vice versa.
Both applications must be native RPMs for the integration to work.

LibreWolf is not in Fedora's official repos,
so the Containerfile adds the [LibreWolf RPM repo][librewolf-repo] at build time.
This is a third-party repo dependency,
 but a qualitatively different risk than Terra.
Terra is a third-party repo packaging other people's software (third-party of third-party):
when Fedora releases,
 Terra maintainers must rebuild packages they do not control.
LibreWolf's repo is maintained by the LibreWolf project itself (first-party of third-party):
they control both the software and the packaging,
 so releases are never blocked
by an intermediary catching up.
 In practice,
 LibreWolf's repo has never lagged behind.

[librewolf-repo]: https://rpm.librewolf.net

## Why a single dotfiles repo

Per-tool repos (like `Aquaticat/nvim`) made sense when nvim was the only config
complex enough to warrant version control.
For single-file configs (ghostty,
 contour,
 xremap,
 git,
 mise,
 crush),
separate repos add overhead with no benefit:
separate clone commands,
 separate git history,
 no atomic cross-tool commits.

One `Aquaticat/dotfiles` repo with subdirectories.
nvim may stay separate due to its complexity (plugins,
 Lua config,
 lazy.
nvim).

## Why KDE config is not portable

`~/.config/` contains ~40 KDE-generated files:
`plasmarc`,
 `kwinrc`,
 `kdeglobals`,
 `kglobalshortcutsrc`,
 etc.
These encode machine-specific state:
display IDs,
 GPU device paths,
 monitor geometry and arrangement,
 device UUIDs.

Porting these between machines (host to VM,
 VM to VM)
causes more breakage than it prevents.
KDE regenerates sane defaults on fresh profiles.
The few settings worth preserving
(custom global shortcuts,
 theme preferences)
are faster to set manually once than to debug
when they reference hardware that does not exist in the VM.

## Why secrets stay manual

`gh auth login` is interactive and takes 30 seconds.
Copying an SSH key from a USB stick takes another 30 seconds.

Every alternative adds complexity without proportional benefit:

- Encrypted vaults (age/sops):
   tooling complexity,
  a master password to remember,
   vault file to keep synced
- Virtiofs injection from host:
   couples the provisioner to a specific hypervisor,
  kills portability
- Automated auth flows:
   fragile,
   break when APIs change,
  still need initial credentials bootstrapped somehow

The manual approach has zero infrastructure,
works on any machine,
 and secrets never touch a repo or disk image.

## Why not CI/CD for image builds

GitHub Actions for building the container image was considered
(the [image-template][image-template] repo includes workflows for this).
Rejected because:

- Slow:
   building a desktop image with KDE + dev tools on CI runners
  is slow and resource-constrained
- Storage risk:
   large container images and disk artifacts
  stored on GitHub consume quota and risk account restrictions
- Testing impossible:
   verifying KDE starts,
   SDDM works,
  podman runs inside the image requires a hypervisor,
   not a CI runner
- Unnecessary:
   this is a private dev environment for one person,
  not a distribution.
   Local builds with `podman build` + `bootc-image-builder` suffice.

[image-template]: https://github.com/ublue-os/image-template

## Why manual provisioner trigger

Systemd user services with "done" flag files and `.bash_profile` hooks
add indirection and debugging pain.

- Flag file pattern is fragile:
  what if you want to re-run after updating the script?
  Now you need to know to delete the flag.
- `.bash_profile` triggers on every login,
   needs guard logic,
  can interfere with non-interactive sessions.
- Systemd user services run before the user's shell is ready,
  complicating interactive auth steps.

Manual trigger (`node setup-dev.config.ts`) costs one command typed once.
You see the output,
 you can ctrl-c if something breaks,
and re-running is just running the command again.
