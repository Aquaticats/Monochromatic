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
- **ovh**:
   OVHcloud Public Cloud instances provisioned over the HTTP API.
   Runs on any platform;
   provisions real,
   billed instances,
   metered by the second.

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
- `systemd-run` from systemd,
  with which mvm starts the session daemon in a user unit of its own;
  without it,
  or when it cannot reach systemd's user manager,
  mvm lets `virsh` start the daemon the way it did before and warns
  (see [Session daemon](#session-daemon))
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

### ovh backend

- The five names from [Making the keys](#making-the-keys) set
  (`OVH_ENDPOINT`,
   `OVH_APPLICATION_KEY`,
   `OVH_APPLICATION_SECRET`,
   `OVH_CONSUMER_KEY`,
   `MVM_OVH_PROJECT_ID`),
   normally loaded by mise from the monorepo-root SOPS-encrypted `.env.local.json`
- An OpenSSH client (`ssh`,
   `scp` and `ssh-keygen`) version 9.0 or newer on the host running mvm,
   with a data directory its `ssh-keygen` may write to:
   see [Keypair storage](#keypair-storage)
- A Public Cloud project used for nothing else,
   recommended in [The project](#the-project)
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

## Choosing a machine's size

`mvm create` and `mvm run` take `--vcpus <count>` and `--memory <GiB>`.
Each is optional and each works alone.
The two numbers are floors,
never exact demands:

> the machine has at least that many virtual CPUs and at least that much memory.

Without either option every backend does exactly what it did before,
so no existing command changes.

A size named outright with `--server-type` (or `MVM_OVH_FLAVOR`, or
`MVM_HCLOUD_SERVER_TYPE`) cannot be combined with either number:
the two can disagree,
and the create refuses naming both and how to choose one.
`mvm create --from` clones a machine and keeps the source's size,
so a floor beside it is refused rather than quietly ignored.

### On the libvirt backend

A libvirt guest gets exactly the numbers asked:

```sh
# package/mvm/cli/README.md -- two vCPUs and 4 GiB of memory
mvm create --vcpus 2 --memory 4 dev-01
```

That defines a domain with two vCPUs and `4096` MiB.
Without the options a guest keeps the [VM defaults](#vm-defaults),
four vCPUs and 8 GiB.

The guest lives in the cgroup of `mvm-libvirt-session.service`,
whose `MemoryMax` is three quarters of the machine's total memory
([Session daemon](#session-daemon)).
A guest that asks for more than that cap is not refused:
the domain is accepted and the guest starts.
The cap limits how much the whole unit may touch,
not how much one domain may declare,
and a guest seldom touches every page it was given.
What the cap does is bind every guest together:
once they touch more than those three quarters,
the kernel's out-of-memory killer ends a process inside a guest
instead of letting the host's own work be swapped out.
mvm refuses only what it can prove wrong,
and what a guest will touch later is not something it can prove.

`mvm list` shows the numbers a guest was given,
read from the metadata mvm wrote beside its disk.
A guest made before mvm recorded them shows no size at all
rather than a default it never had.

### On the hetzner backend

The same rule runs over server types,
from the fields Hetzner's own server type object carries:

- a type is a candidate only when it is not deprecated,
  is offered in at least one of the requested locations,
  and is sold with **shared** virtual CPUs;
  a plan sold with `dedicated` cores is a different product and is never picked
  without being named
  (Hetzner: "Every dedicated resources instance has its own dedicated CPU
  resources", [server types][hetzner-server-types]);
- of those, only types whose `cores` and `memory` are both at least what was
  asked;
- the smallest candidate wins:
  cores, then memory, then disk, then the name.

```sh
# package/mvm/cli/README.md -- at least two vCPUs and 4 GB of memory, fsn1 then nbg1
mvm --backend hetzner create --vcpus 2 --memory 4 build-box
```

Before it creates anything the command prints the type it picked with its
numbers, for example `cx23 (2 cores, 4 GB of memory, 40 GB of disk)`.
The types and their numbers come from the catalogue at call time:
Hetzner publishes them per plan
([deprecated plans][hetzner-deprecated-plans] carries the table's shape:
`CX22` is 2 vCPUs, 4 GB of RAM and 40 GB of SSD).

### On the ovh backend

Among the flavors the region has in stock,
the backend keeps only families OVHcloud sells as general-purpose Linux
machines,
keeps those whose vCPUs and memory are both at least what was asked,
and takes the smallest:
vCPUs, then memory, then disk, then the plain size before its `-flex` twin,
then the name.

```sh
# package/mvm/cli/README.md -- at least two vCPUs and 4 GiB of memory in BHS5
mvm --backend ovh create --vcpus 2 --memory 4 build-box
```

Before it creates anything the command prints the size it picked with its
numbers, for example
`flavor c3-4 with 2 vCPUs, 4 GiB of memory and 50 GB of disk`.
That is the answer a live `BHS5` catalogue gives for those floors.

Families kept,
by the catalogue's own names:

- `b2` and `b3`, general purpose;
- `c2` and `c3`, compute optimized;
- `r2` and `r3`, memory optimized;
- `d2`, discovery,
  the entry-level shared-resource family.

Families left out:

- GPU and accelerator families,
  which OVHcloud's worker-node guide names as `GPU (T2-*, L4-*, L40S-*)`,
  and the `t1` family the catalogue files under `ovh.raid-nvme.t1`.
  The region also carries one image whose `flavorType` restricts it to
  `g1,g2,g3,t1,t2,i1,a100,l4,rtx5000,l40s`;
- the storage-shaped `i1` family,
  which OVHcloud calls **Storage Optimized**
  and sells for NVMe input and output rather than for memory;
- bare metal sizes, named `bm-*` and called **Metal Instances**;
- anything whose flavor targets another operating system,
  which a `win-*` name and a `windows` `osType` mark;
- any family this build does not know.
  A family added to the catalogue after this code was written is therefore
  refused rather than booted unseen,
  which is what keeps a GPU machine out even when it is the only in-stock size
  that would fit.

The rule is a keep list and not a drop list on purpose:
a name nobody has seen yet cannot be picked by accident.

A size equal to its `-flex` twin in vCPUs, memory and disk is ordered with the
plain one first.
OVHcloud documents flex instances as the ones whose local disk stays at 50 GB so
the size can be resized up and down
([resize an instance][ovh-flex]).
mvm never resizes a machine,
it destroys and remakes one,
so the flex benefit is unused;
and the same guide says a flex instance cannot be turned back into a classic
one,
a door mvm has no reason to walk through for a machine whose life is one
command.

`mvm list` shows the vCPUs and memory of each instance,
read from its own region's flavor list.
One flavor list is read per region in play, never one per instance.

[hetzner-deprecated-plans]: https://docs.hetzner.com/cloud/servers/deprecated-plans/
[hetzner-server-types]: https://docs.hetzner.com/cloud/servers/overview#:~:text=Every%20dedicated%20resources%20instance%20has%20its%20own%20dedicated%20CPU%20resources
[ovh-flex]: https://docs.ovhcloud.com/en/guides/public-cloud/compute/resize-instance-manager#:~:text=Flex%20instances%20allow%20resizing%20to%20higher%20or%20lower%20models

## What the `mvm` command runs

`mvm` runs this package's TypeScript source,
`src/cli.ts`,
and Node strips the types on the way in.
No build output stands between the command and its code,
so a change that lands in the source reaches the next run,
and no worktree runs a build older than the source beside it (issue 6655).
`mise run //package/mvm/cli:run` runs the same file.

`dist/final/node/cli.mjs` is a launcher for that source,
never a copy of the program:
a `node_modules/.bin/mvm` shim written by an install older than this manifest names that path,
so the build writes a file there that runs the source beside it.
`pnpm install` in a checkout writes a shim naming the source directly,
and `mise run //package/mvm/cli:build` refreshes the launcher in a worktree whose build is
older than this change.
The ways weighed,
their costs and the measurements:
[`doc/what-the-mvm-command-runs.md`](doc/what-the-mvm-command-runs.md).

## Host tools

The libvirt backend runs two host commands,
`virsh` and `qemu-img`,
and starts libvirt's session daemon by itself
(see [Session daemon](#session-daemon)).
mvm decides once per process how to run each of the two,
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

Before its first `virsh` call in a process,
mvm asks libvirt whether a session daemon answers:
it runs `virsh list --all` with `LIBVIRT_AUTOSTART=0`,
the setting that stops libvirt from starting a daemon on demand for that call.
A daemon that answers is used as it is,
in whatever cgroup it runs.

When none answers,
mvm starts `virtqemud --timeout=120` itself,
as the transient user unit `mvm-libvirt-session.service`,
through `systemd-run --user --unit=mvm-libvirt-session --collect`.
The daemon and every guest it starts then live in that unit's cgroup,
not in the cgroup of the command that ran mvm.
Otherwise `virsh` starts the daemon inside that command's cgroup,
and the command's memory cap,
time limit or cleanup kills the daemon and every guest of every session;
see `doc/troubleshooting/libvirt-session-daemon-runs-in-the-capped-scope-of-whichever-command-starts-it.md` in the repository root.
On 2026-11-21 the project chose this for issue 6338.

One daemon,
whoever starts first:
the unit name is the single owner.
A second mvm that starts at the same moment gets systemd's "already loaded" answer
and waits for the daemon the first one began instead of failing.
A start that finds the name still loaded,
as it is while the previous daemon is exiting,
is tried again every quarter of a second until a daemon answers or thirty seconds pass,
so that caller waits where it used to fail.
mvm also runs every `virsh` call with `LIBVIRT_AUTOSTART=0`,
so no call ever lets `virsh` start a daemon in mvm's cgroup;
when a daemon has exited between two calls,
mvm starts it again and repeats the call once.

The daemon runs with the user manager's environment,
not with the environment of the command that started it,
so a variable set in a shell profile is not in its environment.

The unit's settings,
each with its basis:

- `MemoryMax` is three quarters of this machine's total memory,
  read from `MemTotal` in `/proc/meminfo` when the unit starts,
  which is 46.3 GiB on a machine with 61.7 GiB (`MemoryMax=47364M`):
  the owner chose "Three quarters,
  six guests" on 2026-11-28,
  replacing the project's choice of "half of total" for issue 6338 on 2026-11-21,
  so every guest together may use three quarters of the machine and everything outside the unit gets the rest
  (rule `LQM` in `AGENTS.md`,
  cited beside the value in `package/mvm/cli/src/session-daemon-memory.ts`,
  and recorded in `doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
  "The unit's cap").
  Measured on 2026-11-21:
  the unit with `virtqemud` and `virtlogd` held 11.6 MiB (peak 14 MiB),
  one running guest's `virtiofsd` processes held 1.7 MiB and 15.4 MiB,
  a unit with one guest whose domain asked for 8 GiB charged a peak of 990 MB,
  and a unit holding that guest and another session's 8 GiB guest charged 2.7 GB.
  A guest is charged what it touches,
  not the memory its domain asks for.
  Read on 2026-11-28:
  a guest that builds the wrapper's crate inside itself is charged several GiB,
  where the measurement of 2026-11-21 put one guest's unit at 990 MB,
  and eight guests that between them had touched 58.2 GiB swapped against a unit allowed half the machine
  (`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
  "The guests' unit at its own memory cap,
  2026-11-28").
  A host whose `/proc/meminfo` cannot be read fails the start with an error naming the file.
- `OOMPolicy=continue`:
  when the guests' memory exceeds the cap,
  the kernel kills a process of its choosing inside the unit,
  usually the largest guest,
  and `continue` keeps systemd from stopping the unit over that kill.
  systemd's default,
  `OOMPolicy=stop`,
  would stop the unit and kill every guest,
  which is the failure the unit exists to prevent.
  This holds against the kernel's per-process kill only:
  `systemd-oomd` acts on a whole cgroup,
  and this unit is one large candidate for it.
  `ManagedOOMPreference=avoid` on the unit is a candidate to evaluate later,
  not a change that has been measured.
- `ExitType=cgroup`:
  systemd keeps the unit,
  and its cgroup,
  while any guest process still runs after the daemon exits,
  as a guest outlived the daemon that started it before this unit existed.
  `KillMode` stays systemd's default `control-group`,
  so a stop of the unit still kills every process in it.
  When the daemon exits while guests run,
  the name stays loaded:
  mvm reports that instead of starting a second unit,
  because a second unit would be a second cap over the same guests.
  A person can still stop the unit,
  which kills those guests,
  and a later mvm then starts a fresh daemon.
- No `RuntimeMaxSec=`:
  a time limit on the command that started the daemon is one of the ways guests died before.
- `virtqemud --timeout=120`:
  libvirt's own idle limit for a daemon it forks,
  so the daemon exits two idle minutes after the last guest stops,
  and `--collect` then removes the unit.
  A running guest keeps the daemon up.

Never stop `mvm-libvirt-session.service` while a guest runs.
The unit is one for every session on the machine,
so a stop kills every session's guests,
not only yours,
and the unit needs no stop:
the daemon exits two idle minutes after the last guest stops and `--collect` removes it.
`systemctl --user list-units` lists it,
and a cleanup that stops the units it started must leave this one alone.

The cap belongs to the whole unit,
not to each guest:
six guests at mvm's default memory of 8 GiB each exceed it on a machine with 61.7 GiB,
and the kernel then kills the guest it picks rather than the daemon.
Because the cap is three quarters of the machine's total,
guests can never take more than three quarters of it,
and everything outside the unit is handled by the machine as it was before.
The last quarter is not reserved for the session:
it is simply memory the unit may not have.

`MVM_VIRTQEMUD_COMMAND` names the command that runs `virtqemud`,
as a JSON array,
for a command that runs `virsh` under a name that does not say `virsh`.
Unset,
mvm replaces `virsh` with `virtqemud` in each argument of `MVM_VIRSH_COMMAND`,
which covers a bare `virsh`,
a path such as `/usr/bin/virsh`,
and the Flatpak's `--command=virsh`.

When the daemon does not come up,
mvm fails with an error naming the unit it started,
how to read what the unit reported
(`journalctl --user-unit=mvm-libvirt-session.service`,
`systemctl --user status mvm-libvirt-session.service`),
and how to start the daemon by hand to see why.
When a unit of that name failed earlier and still holds the name,
the error also names `systemctl --user reset-failed mvm-libvirt-session.service`,
which unloads it so the next start can use the name.
mvm starts the daemon in the unit whenever it can,
and does not fall back to letting `virsh` start it:
that is the cgroup problem again.
Two cases leave mvm no unit to use,
and there it falls back to the old on-demand start and warns,
naming this defect and the troubleshooting document,
and saying that the cap and the separation from the cgroup are not in place:
no `systemd-run` on `PATH`,
and a `systemd-run --user` that reports it cannot reach the user manager,
which a caller whose `XDG_RUNTIME_DIR` is not the user manager's gets.
For those calls mvm sets `LIBVIRT_AUTOSTART=1` itself,
so libvirt starts the daemon on demand as it did before,
whatever the caller's own setting.
A caller with its own runtime directory also cannot see the daemon a unit would start,
so a fully isolated run needs no unit.

Starting the daemon in the Flatpak is untested:
no Flatpak installation was available to try it,
and whether `flatpak run` keeps the daemon inside the unit's cgroup is unknown.

Every `virsh` call has a deadline:
two minutes,
or the guest agent timeout plus thirty seconds for a guest agent request.
A call that cannot connect or outlasts its deadline fails with an error that shows how to start the daemon by hand,
which is how to find out why it does not come up.

mvm waits for `virsh` to exit,
not for its output streams to close.
Any process a `virsh` call leaves behind can keep those streams open for as long as it lives;
the daemon started on demand inside the Flatpak sandbox did,
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
- On a systemd guest a command runs as a child of the guest agent,
  in the agent's unit `qemu-guest-agent.service`,
  so the guest kernel's out-of-memory killer ends it there when the guest runs out of memory.
  systemd's default `OOMPolicy=stop` then stops that unit,
  which kills the agent and the command with it,
  and the agent holds every started command's process ID and captured output in memory only,
  so every command started before the stop loses its result,
  and `guest-exec-status` answers that the process ID does not exist.
  mvm's systemd Linux guests carry
  `/etc/systemd/system/qemu-guest-agent.service.d/mvm-command-oom.conf`,
  which sets `OOMPolicy=continue`:
  the kernel's kill ends the one process and not the agent,
  the command keeps its own result,
  and a command a signal ended reports 128 plus the signal number.
  The file is written when an image's template is baked,
  so a template baked by an earlier mvm keeps the default
  until it is baked again with `mvm update`,
  or by deleting `~/.local/share/mvm/images/template-ubuntu.qcow2`
  once no running guest's disk is backed by it.
  `doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md` in the repository root
  holds the readings,
  the source lines,
  and what the setting cannot do.
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

### Fedora guests and SELinux

Fedora labels `/usr/bin/qemu-ga` `virt_qemu_ga_exec_t`,
and its own unit turns `qemu-guest-agent.service` into the `virt_qemu_ga_t`
domain.
That domain may not read the virtiofs share,
write the guest's own files,
execute a file it wrote,
or open a TCP connection,
so `mvm push`,
`mvm exec` and `mvm pull` are refused there.
The Fedora template therefore starts the agent through `/usr/bin/env`,
which the policy turns into `unconfined_service_t`,
and the guest's SELinux stays enforcing.

The drop-in lives in the template,
so a template baked by an earlier mvm keeps the confined agent.
Bake it again once no running guest's disk is backed by it:

```sh
# package/mvm/cli/README.md
rm ~/.local/share/mvm/images/template-fedora.qcow2
```

The next `mvm create --image fedora` bakes it again.
Do not delete or replace that file while a Fedora guest created from it runs:
those disks use it as their qcow2 backing file.
A custom template of another SELinux distro needs the same drop-in by hand;
see `doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md` in the
repository root.

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

# Ask for at least two vCPUs and 4 GB of memory; the smallest type that fits wins
mise run //package/mvm/cli:run -- --backend hetzner create --vcpus 2 --memory 4 build-box

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
- **Server types**:
   without `--server-type` and without `--vcpus` or `--memory`,
  the cheapest non-deprecated type offered in the target locations is used.
   With either number,
  the smallest type that is not deprecated,
  is sold with shared virtual CPUs,
  and has at least the asked cores and memory wins;
  see [Choosing a machine's size](#on-the-hetzner-backend).
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

## OVHcloud Public Cloud backend

Select it per command with `--backend ovh` (or `export MVM_BACKEND=ovh`).
It provisions real,
 billed OVHcloud Public Cloud instances,
 so the five names in [Making the keys](#making-the-keys) must be set.

Every instance this backend makes is on the **hourly** price.
OVHcloud meters that price by the second,
 so its billing guide says "Every second of use is billed,
 beyond an initial 1-minute minimum",
 and the meter starts when the instance reaches the `ACTIVE` status
 ([Information regarding Public Cloud billing options][billing]).
The meter stops only when the instance is deleted:
a machine that is merely stopped or paused still bills,
and so does a snapshot left behind,
which is billed per GB per month
([the same guide][billing]).
For that reason mvm only ever creates,
 lists,
 connects to,
 and deletes;
 it never stops,
 pauses,
 shelves or resumes an instance,
 and it never asks for monthly billing,
 which bills each commenced month even if the instance is deleted first
([the same guide][billing]).
The decision behind this backend is
`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md` in the
repository root.

### Making the keys

OVHcloud's own client names one page per API host for making keys
(`README.rst` of [python-ovh][python-ovh],
 "Create script credentials"):

- Europe: <https://eu.api.ovh.com/createToken/>
- North America,
  which this backend uses by default:
  <https://ca.api.ovh.com/createToken/>
- United States: <https://api.us.ovhcloud.com/createToken/>

On that page,
give the script four rights,
one per HTTP method,
each on `/cloud/project/<your project ID>/*`:

- `GET`,
   to list and read instances,
   flavors,
   images,
   regions,
   quotas and keys
- `POST`,
   to create an instance,
   upload a key and snapshot an instance
- `PUT`,
   which none of the calls mvm makes today needs,
   but which keeps one key usable when one is added,
   and which OVHcloud's own form offers beside the others
- `DELETE`,
   to delete an instance,
   a snapshot or a key

The page answers with three keys:
the application key (`AK`),
the application secret (`AS`) and the consumer key (`CK`).

### Storing the keys

Store the five names with `mise run --raw secrets:edit`:

- `OVH_ENDPOINT`:
  the API host,
  either `ovh-ca` (the default when unset),
  `ovh-eu` or `ovh-us`,
  or a complete `https://` API root
- `OVH_APPLICATION_KEY`:
  the `AK`
- `OVH_APPLICATION_SECRET`:
  the `AS`
- `OVH_CONSUMER_KEY`:
  the `CK`
- `MVM_OVH_PROJECT_ID`:
  the Public Cloud project's identifier,
  the `serviceName` every API path carries

Commands launched through the repository's mise tasks inherit the decrypted
values.
`mvm --backend ovh list` with any of them missing names the first missing one
and exits non-zero.

### The project

Use a Public Cloud project for nothing else.
OVHcloud's schema gives an instance no label,
so the backend recognises its own machines by the `mvm-` name prefix inside the
one project,
and `destroy --all` deletes every instance and every snapshot carrying that
prefix.
A machine someone else made,
and happened to name with the same prefix,
would be swept with them;
a project used for nothing else is what keeps that from being possible.

The key itself can be limited to that project,
and the quota of instances,
cores and memory is per region:
`mvm --backend ovh update` reports each Canadian region's smallest flavor in
stock and how much of its instance quota is used.

### Keypair storage

mvm keeps one ed25519 keypair for this backend under
`~/.local/share/mvm/ovh/`,
and `MVM_OVH_DATA_DIR` moves it,
so two OVHcloud accounts never share a private key.
The keypair is generated with the host's `ssh-keygen`,
which on a host whose AppArmor profile confines `ssh-keygen` to `~/.ssh`
cannot write there:
point `MVM_OVH_DATA_DIR` at a directory that program may write to,
or widen the profile.

`MVM_OVH_SSH_COMMAND` and `MVM_OVH_SCP_COMMAND` replace the `ssh` and `scp`
executables with a JSON array naming the program and its leading arguments,
as `MVM_VIRSH_COMMAND` does for libvirt.

### Usage

```sh
# Create an instance (defaults: the project's Canadian regions, the smallest
# flavor in stock there, the newest plain Ubuntu image that flavor boots)
mise run //package/mvm/cli:run -- --backend ovh create dev-01

# Pick a flavor and a region series (the first region with stock wins)
mise run //package/mvm/cli:run -- --backend ovh create big \
  --server-type b3-16 --location bhs5,gra9

# Run a command (over SSH), copy files (over SCP), open a shell, then destroy
mise run //package/mvm/cli:run -- --backend ovh exec dev-01 -- uname -a
mise run //package/mvm/cli:run -- --backend ovh push dev-01 ./setup.sh /tmp/setup.sh
mise run //package/mvm/cli:run -- --backend ovh shell dev-01
mise run //package/mvm/cli:run -- --backend ovh destroy dev-01

# List or destroy every mvm-owned instance and snapshot (scoped by the mvm-
# name prefix inside the project)
mise run //package/mvm/cli:run -- --backend ovh list
mise run //package/mvm/cli:run -- --backend ovh destroy --all

# Ask for at least two vCPUs and 4 GiB of memory; the smallest size that fits wins
mise run //package/mvm/cli:run -- --backend ovh create --vcpus 2 --memory 4 build-box

# Report each Canadian region's smallest flavor in stock and its instance quota
mise run //package/mvm/cli:run -- --backend ovh update
```

OVHcloud keeps an instance it has deleted in the project's listing for a while,
with the status `DELETED`.
`list`,
`destroy --all` and every lookup by name leave such an instance out,
because it runs nowhere and its name is free again.

### Differences from the Hetzner backend

- **The regions are the project's own**:
   by default every region whose `countryCode` field is `ca`,
   whose status is `UP`,
   and whose `services` hold an `instance` service that is `UP`,
   in name order,
   so no region name is baked into mvm.
   `--location` or `MVM_OVH_REGIONS` replaces that list,
   with the precedence `--location` and `MVM_HCLOUD_LOCATIONS` have.
- **Flavors,
   not server types**:
   `--server-type` takes an OVHcloud flavor name or identifier and
  `MVM_OVH_FLAVOR` sets it for a run;
   without it and without a floor the smallest flavor in stock in the region is
  used,
  by virtual CPUs and then memory.
   `--vcpus` or `--memory` replaces that walk with the size rule,
  which also keeps special families out;
  see [Choosing a machine's size](#on-the-ovh-backend).
   A region with no size that fits is skipped before anything is created,
  and the next region is asked for the same floors.
- **The login account is the image's own,
   never `root`**:
   mvm reads the image's `user` field,
   so an Ubuntu image logs in as `ubuntu` and a Debian image as `debian`.
   A pushed path such as `/root/setup.sh` is not writable;
   push to `/tmp` or to that account's home directory.
- **`clone` snapshots the running source**,
   boots a new instance from the snapshot in the source's region,
  then deletes the snapshot on the way out,
   including when the clone fails.
   `destroy --all` sweeps snapshots carrying the prefix as a backstop.
- **A create that cannot reach its instance deletes it** before reporting the
  failure,
   so a failed create leaves nothing billed;
   when that delete also fails,
   the error says the instance keeps being billed and prints the one command
   that removes it,
   `mvm --backend ovh destroy <name>`.
- **No label and no backend record**:
   the schema has no label to scope by and nothing records which backend a VM
  lives on,
   so pass the same `--backend ovh` to follow-up commands.
- **No image cache**:
   OVHcloud manages images server-side,
   so `update` reports what the project offers and builds nothing locally.
- **A shorthand stands for a plain image**:
   `--image ubuntu` takes the newest active Ubuntu image that carries no
  preinstalled software and is no variant of another image,
   so it never takes a GPU driver image such as
  "Ubuntu 26.04 - NVIDIA - v595" or a "- UEFI" variant,
   both of which OVHcloud's catalogue publishes after the plain release.
   An image that states a larger disk,
   more memory or another flavor family than the chosen flavor has is left out.
   To boot one of those images,
   pass its whole name to `--image`.

A live,
 billed integration test lives at
`src/backend/ovh/provision.expensive.unit.test.ts`.
 It is excluded from
`test:unit` (the `.expensive.` marker) and only runs when all five names are set.
 It creates one instance in the smallest flavor the region offers,
 runs one command,
 pushes and pulls one file,
 destroys the instance,
 and fails if any mvm-owned machine is left:

```sh
mise run //package/mvm/cli:test:ovh
```

[billing]: https://docs.ovhcloud.com/en/guides/public-cloud/cross-functional/analyze-billing
[python-ovh]: https://github.com/ovh/python-ovh

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

## Free page reporting

Every guest's domain carries a virtio memory balloon that reports the pages the
guest has freed to the host:

```xml
<!-- package/mvm/cli: the device every domain mvm writes carries -->
<memballoon model="virtio" freePageReporting="on"/>
```

Without that element libvirt adds a plain virtio balloon,
and QEMU then leaves its `free-page-reporting` property at its default,
which is off.
A guest's freed page stays charged to the host for the guest's whole life,
even though the guest calls the memory free.
On the host measured for the decision (libvirt 12.9.0,
QEMU 11.1.3,
an Ubuntu 24.04 guest after one build):

- 7.98 GiB charged at rest with reporting off,
  5.52 GiB with it on,
  2.46 GiB less;
- after the guest dropped its own page cache,
  3.24 GiB charged with reporting on against 7.98 GiB with it off.

What it cannot return is a page the host has already swapped out,
and a build's peak is unchanged because live pages have nothing to give back.
The fault cost when a guest reuses a returned page is unmeasured.
The project chose the device on 2026-11-25;
`doc/decision/mvm-guests-report-free-pages.md` in the repository root records
the readings and their limits,
and `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md` holds the
measurement.

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

These are what a libvirt guest gets when the create asks for nothing.
`--vcpus` and `--memory` override them;
see [Choosing a machine's size](#choosing-a-machines-size).

- 4 vCPUs
- 8 GiB RAM
- 20 GiB root disk (thin-provisioned via qcow2)
- Serial console auto-login (disconnect with `Ctrl+]`)
