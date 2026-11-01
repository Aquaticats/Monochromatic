# cli-mvm

Ephemeral VM manager with pluggable backends.
Creates always-on virtual machines you can immediately shell into.
VMs exist from creation until destruction;
 no pause,
 stop,
 or snapshot lifecycle.
Clone replaces snapshot needs.

## Backends

mvm selects a backend per invocation with `--backend <kind>` (or the
`MVM_BACKEND` env var);
 it defaults to `libvirt`.

- **libvirt** (default):
   local QEMU/KVM virtual machines.
   Linux only.
- **hetzner**:
   Hetzner Cloud servers provisioned over the HTTP API.
   Runs on any
  platform;
   provisions real,
   billed servers.

There is no record of which backend a VM lives on,
 so pass the same `--backend`
to follow-up commands (`exec`,
 `destroy`,
 etc.) that you used to create the VM.

## Prerequisites

### libvirt backend (default)

- **Linux only**:
   depends on KVM,
   libvirt,
   and `virsh`,
   none of which are available on macOS or Windows
- KVM support (`/dev/kvm` must exist)
- `virsh` and `qemu-img` installed (`sudo dnf install libvirt qemu-img`),
  or the `org.virt_manager.virt-manager` Flatpak,
  whose `virsh` and `qemu-img` mvm uses when `virsh` is not on `PATH`
  (see [Host tools](#host-tools))
- `virtiofsd` is optional:
  without it VMs get no shared directory and files move through the guest agent
  (see [File transfer](#file-transfer))

### hetzner backend

- `HCLOUD_TOKEN` set to a Hetzner Cloud API token (read/write),
   normally loaded by mise from the monorepo-root SOPS-encrypted `.env.local.json`
- An OpenSSH client (`ssh`,
   `scp`) version 9.0 or newer on the host running mvm
- Cross-platform:
   no local hypervisor required

## Usage

```sh
# Create a new VM (downloads Ubuntu 24.04 LTS cloud image on first run)
mvm create dev-01

# Create a VM with a different image
mvm create --image fedora build-box
mvm create --image alpine lightweight
mvm create --image windows win-box

# Connect to a running VM (auto-login serial console, Ctrl+] to disconnect)
mvm shell dev-01

# List all managed VMs
mvm list

# Clone a VM (full disk copy, new hostname via cloud-init)
mvm create --from dev-01 dev-02

# Destroy a VM and all its storage
mvm destroy dev-01
```

Or via mise:

```sh
mise run //package/mvm/cli:run -- create dev-01
mise run //package/mvm/cli:run -- create --image fedora build-box
mise run //package/mvm/cli:run -- create --image windows win-box
```

## Host tools

The libvirt backend runs two host commands,
`virsh` and `qemu-img`.
mvm decides once per process how to run each of them,
in this order:

1.  The tool's environment variable,
    `MVM_VIRSH_COMMAND` or `MVM_QEMU_IMG_COMMAND`.
2.  The bare command name,
    when `virsh` is an executable in a directory of `PATH`.
3.  The tool inside the `org.virt_manager.virt-manager` Flatpak,
    when that Flatpak is installed:
    `flatpak run --command=virsh org.virt_manager.virt-manager`,
    and the same with `qemu-img`.
4.  The bare command name,
    so that the failure names what is missing.

The second and third step treat both tools as one installation:
the `qemu-img` that creates a disk comes from the same place as the libvirt that runs it.
On a host whose libvirt exists only in the Flatpak,
mvm therefore works without any configuration,
and so does the MCP server,
which shares this code.
The Flatpak must be able to read and write mvm's data directory;
its default permissions include the home directory.

### Command variables

Each variable holds a JSON array:
the executable,
then the arguments that go before the tool's own.
A JSON array is used so that an argument may contain spaces without any quoting rules.

```sh
# package/mvm/cli/README.md
export MVM_VIRSH_COMMAND='["flatpak","run","--command=virsh","org.virt_manager.virt-manager"]'
export MVM_QEMU_IMG_COMMAND='["/opt/qemu/bin/qemu-img"]'
```

The MCP server reads the same variables from its own environment,
which is set where the server is registered,
not in a shell profile:

```sh
# package/mvm/cli/README.md
claude mcp add --scope user mvm \
  --env 'MVM_VIRSH_COMMAND=["/opt/libvirt/bin/virsh"]' \
  -- node /path/to/package/mcp/mvm/dist/final/node/index.mjs
```

A variable that is set but is not a JSON array of non-empty strings stops mvm with an error naming the variable.

### Session daemon

mvm does not start libvirt's session daemon.
`virsh` starts `virtqemud` on demand,
and the daemon keeps running while a domain runs and exits after two idle minutes.
The first `virsh` call after that takes a few seconds longer.

Every `virsh` call has a deadline:
two minutes,
or the guest agent timeout plus thirty seconds for a guest agent request.
A call that cannot connect or outlasts its deadline fails with an error that shows how to start the daemon by hand,
which is how to find out why it does not come up.

mvm waits for `virsh` to exit,
not for its output streams to close.
A daemon started on demand inside the Flatpak sandbox keeps those streams open for as long as it lives,
which made one piped `virsh` call take two minutes;
see `doc/troubleshooting/mvm-libvirt-flatpak-only-host.md` in the repository root.

## File transfer

`mvm push` and `mvm pull` use one of two routes.
The route is chosen when the VM is created and recorded in its `meta.json` as `fileTransfer`.

### Shared directory

When libvirt can run `virtiofsd`,
the VM gets a `shared/` directory on the host,
mounted in the guest at `/mnt/shared` (Linux) or `Z:\` (Windows).
Push copies the host file into that directory and pull reads from it.
Only the file name of the guest path is used:
`mvm push dev-01 ./setup.sh /any/dir/setup.sh` puts the file at `/mnt/shared/setup.sh`.

### Guest agent

When libvirt finds no `virtiofsd`,
a domain with the share would not start
(`Unable to find a satisfying virtiofsd`).
mvm asks libvirt before defining the domain,
by converting a minimal domain with `virsh domxml-to-native`,
and then defines the VM without the share and without the shared memory backing the share needs.
Push and pull read and write the guest's own files through the QEMU guest agent:

- The guest path must be the file's full path in the guest,
  such as `/tmp/setup.sh` or `C:\Windows\Temp\setup.ps1`.
  Its directory must exist;
  the agent does not create directories.
- Paths with spaces and non-ASCII characters need no quoting,
  because the agent opens the path itself and no guest shell is involved.
- The file travels in chunks,
  one `virsh` call each:
  96000 bytes per write and 1 MiB per read.
  A push is therefore slow where each `virsh` call is slow,
  as it is through the Flatpak;
  measured rates are in `doc/handover/mvm-flatpak-libvirt.md` in the repository root.
- A chunk whose call gets no answer is sent again at the same position,
  for as long as the domain runs and the agent's silence lasts less than five minutes.
  A guest under load can leave the agent silent for minutes.
  When a push stops midway,
  the error says how many bytes arrived,
  and the guest file is incomplete until the push is repeated.
- After a transfer,
  the size the guest reports must equal the number of bytes moved.
- `mvm pull` holds the whole file in memory.

## Running commands

`mvm exec` and `mvm run` start the command through the QEMU guest agent and wait for it.

- Every command first prints a marker line that mvm removes from the output.
  The guest agent finds a finished command by process ID alone,
  and a reused process ID once returned an earlier command's output with exit status 0;
  only a result carrying the marker is returned.
  When no result carries it,
  mvm fails and shows the results it discarded.
  PowerShell parses a whole command before running any of it,
  so a PowerShell command that does not parse prints no marker and is reported this way,
  with the parser's message in the discarded output.
- A status request that the guest agent does not answer is asked again.
  mvm gives up when the domain is gone or when requests stay unanswered for five minutes.
- A command a signal ended reports exit status 128 plus the signal number.
- One result may hold about 3 MiB of standard output and standard error together,
  because libvirt hands over at most 4194304 characters per answer.
  A command with more output fails with an error saying so,
  and its output and exit status are lost;
  redirect large output to a file in the guest and fetch it with `mvm pull`.
- On a Windows guest the command runs in PowerShell,
  which reports status 1 when the last program it ran failed,
  whatever that program's own status was.
  End the command with `; exit $LASTEXITCODE` to get the program's status.
- `mvm create` returns once the new VM's guest agent answers,
  and waits up to five minutes for that.
  After a reboot inside the guest,
  a command fails until the agent answers again;
  repeat it.

## Available images

- **ubuntu** (default):
   Ubuntu 24.04 LTS (Noble Numbat),
   user:
   `ubuntu`
- **fedora**:
   Fedora 43 Cloud Base,
   user:
   `fedora`
- **alpine**:
   Alpine 3.23 with cloud-init,
   user:
   `alpine`
- **windows**:
   Windows Server 2025 evaluation ISO,
   user:
   `Administrator`

Each image is downloaded once and cached in `~/.local/share/mvm/images/`.
A per-image template with qemu-guest-agent pre-installed is baked on first use
(e.g. `template-ubuntu.qcow2`,
 `template-fedora.qcow2`).

## Hetzner Cloud backend

Select it per command with `--backend hetzner` (or `export MVM_BACKEND=hetzner`).
It provisions real,
 billed Hetzner servers,
 so an `HCLOUD_TOKEN` must be set.

Store `HCLOUD_TOKEN` through `mise run --raw secrets:edit`.
Commands launched through the repository's mise tasks inherit the decrypted value.

```sh
# Create a server (defaults: cheapest non-deprecated type, locations fsn1,nbg1,hel1)
mise run //package/mvm/cli:run -- --backend hetzner create dev-01

# Pick a server type and location series (first available wins)
mise run //package/mvm/cli:run -- --backend hetzner create big \
  --server-type cpx41 --location ash,hil

# Run a command (over SSH), copy files (over SCP), open a shell, then destroy
mise run //package/mvm/cli:run -- --backend hetzner exec dev-01 -- uname -a
mise run //package/mvm/cli:run -- --backend hetzner push dev-01 ./setup.sh /root/setup.sh
mise run //package/mvm/cli:run -- --backend hetzner shell dev-01
mise run //package/mvm/cli:run -- --backend hetzner destroy dev-01

# List or destroy every mvm-managed server (scoped by the mvm=true label)
mise run //package/mvm/cli:run -- --backend hetzner list
mise run //package/mvm/cli:run -- --backend hetzner destroy --all
```

Notes:

- **Images** map distro shorthands (`ubuntu`,
   `debian`,
   `fedora`,
   `rocky`,
  `centos`,
   `alma`) to the newest non-deprecated Hetzner system image of that
  flavor;
   an unrecognised value is passed through as a literal Hetzner image
  slug.
   `alpine` and `windows` are not offered by Hetzner.
- **Location fallback**:
   `--location` (or `MVM_HCLOUD_LOCATIONS`) is an ordered
  list;
   mvm advances to the next location when one is out of stock
  (HTTP 412 `resource_unavailable`).
- **Cost**:
   every operation provisions or holds billed resources.
   An ephemeral
  `run`,
   or a crash between create and destroy,
   can leave an orphan;
   sweep with
  `--backend hetzner destroy --all` (or `list` to inspect).
   Tunables:
  `MVM_HCLOUD_SERVER_TYPE`,
   `MVM_HCLOUD_LOCATIONS`.
- **Differences from libvirt**:
   `exec`/`shell` run over SSH (not the guest
  agent);
   `push`/`pull` use SCP with a real absolute remote path (not a
  virtiofs filename);
   `clone` snapshots the source live (it is not shut down)
  and the intermediate snapshot is deleted once the destination boots;
   host
  keys are not persisted because Hetzner recycles public IPv4.
- **No persistent backend record**:
   pass the same `--backend hetzner` to
  follow-up commands as you used to create the VM.

A live,
 billed integration test lives at
`src/backend/hetzner/provision.expensive.unit.test.ts`.
 It is excluded from
`test:unit` (the `.expensive.` marker) and only runs when `HCLOUD_TOKEN` is set:

```sh
mise run //package/mvm/cli:test:hetzner
```

## Custom templates

To use an image not in the built-in registry:

1. Boot your image manually (via virt-manager,
    raw qemu,
    or any other method)
2. Install `qemu-guest-agent` inside the guest and enable it to start on boot:
   mvm relies on the guest agent for command execution via `mvm exec`
3. Shut down the guest cleanly
4. Place the resulting qcow2 disk image in `~/.local/share/mvm/images/` with a descriptive name
   (e.g. `my-custom.qcow2`)
5. Create VMs from it with `mvm create --image my-custom dev-01`

mvm uses the custom template as a qcow2 backing file directly,
 skipping the
download-and-template-bake pipeline.
 The cloud-init seed defaults to `root`
with a `/bin/sh` shell for custom images.

## Architecture

- **Base images** are cached in `~/.local/share/mvm/images/`
- **Per-image templates** are stored alongside base images (e.g. `template-ubuntu.qcow2`)
- **Per-VM storage** lives in `~/.local/share/mvm/vms/<name>/`
  containing `disk.qcow2`,
   `seed.iso`,
   and metadata files
- **Disks** use qcow2 backing files from the cached template for fast creation
- **Cloud-init** seed ISOs are generated using a built-in ISO9660 writer (no `genisoimage` needed)
- **Console access** uses `virsh console` with auto-login on ttyS0 (no SSH or keys needed)
- **Networking** uses QEMU user-mode networking (SLIRP) for outbound internet access
- **Connection** uses `qemu:///session` so no root privileges or polkit prompts are needed
- `virsh` and `qemu-img` run as described in [Host tools](#host-tools)
- Files move through a virtiofs shared directory,
  or through the guest agent on hosts without `virtiofsd`
- All VM names are prefixed with `mvm-` in libvirt to avoid collisions

## VM defaults

- 4 vCPUs
- 8 GiB RAM
- 20 GiB root disk (thin-provisioned via qcow2)
- Serial console auto-login (disconnect with `Ctrl+]`)
