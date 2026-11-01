# `mvm` on a host whose libvirt lives only in the virt-manager Flatpak

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mvm` is now `package/mvm/cli`.

## Purpose and state

The repository's VM tool `mvm` (`package/cli/mvm`) could not reach libvirt on this machine,
whose only libvirt and QEMU are inside the `org.virt_manager.virt-manager` Flatpak
and which has no `virtiofsd`.
On 2026-10-05 the human decided to teach `mvm` to work here
instead of installing host packages or keeping scratch shims.
This handover records what was changed,
the evidence for each change,
the choices that are open to the human's veto,
and what is left.

State on 2026-10-05:
the `mvm` command-line program works on this host with no configuration,
for an Alpine guest and a Windows guest,
verified by real runs that create a VM,
run commands,
push and pull files with matching hashes,
and destroy the VM.
The `mvm` MCP server was rebuilt but the running server process was not reloaded,
so the MCP tools are unverified;
see `What the human must do for the MCP server`.

The diagnosis of the original failures,
with source citations,
is in `doc/troubleshooting/mvm-libvirt-flatpak-only-host.md`.

## What to inspect and how to respond

- `Decisions open to veto` lists each choice made without asking.
  Say which to change;
  silence keeps them.
- `What the human must do for the MCP server` is the one step only the human can take.
- `Running the scanner's Windows suite with mvm alone` says which steps of
  `doc/handover/scanner-native-verification.md`,
  section `Repeating the Windows run`,
  no longer need the scratch bridges.
- `What remains` lists unfinished and unverified items.

## Requirements

From the delegation of 2026-10-05,
in order:

1.  Fix two `mvm exec` defects,
    each with a failing test first:
    one unanswered status poll must not fail a command that is still running,
    and a status read must not return an earlier command's result when the guest reuses a process ID.
2.  Make the `virsh` and `qemu-img` commands configurable for the command-line program and the MCP server,
    including handling of the session daemon.
3.  Provide a file transfer route for hosts without `virtiofsd`,
    with the virtiofs route unchanged.
4.  When a host tool is missing,
    name the executable and how to configure it.

Constraints kept:
every VM created was destroyed,
the six libvirt domains that existed before were not touched,
no template image changed (hashes in `Real runs on this host`),
only generated test files were pushed into guests,
and no third-party dependency was added.

## Guest command results

Code:
`package/cli/mvm/src/guest-exec.ts`,
`guest-exec-status.ts`,
`guest-exec-errors.ts`,
`exec-shell.ts`
and `agent-command.ts`.
`mvm exec`,
`mvm run`
and the Windows template build all go through `runGuestCommand`.

### One unanswered status poll

libvirt waits 5 seconds for a guest agent answer unless told otherwise
(`QEMU_AGENT_WAIT_TIME` in `src/hypervisor/qemu_agent.c:714` of libvirt 12.4.0),
and the old code let one such timeout fail the whole call.

Now every agent request passes `--timeout 60`.
A status request that fails without an answer from the agent is asked again;
`virsh` runs with `LC_ALL=C`,
and a failure counts as an answer from the agent only when its text contains
`unable to execute QEMU agent command`.
After an unanswered request `mvm` reads the domain's state:
when the domain is shut off,
crashed
or unknown to libvirt,
the wait ends at once with `GuestExecStatusUnavailableError`.
Otherwise it keeps asking,
and gives up with the same error,
which names the guest process ID,
when requests stay unanswered for five minutes.

### Reused process ID

The guest agent keeps an exited command's result until it is read and finds it by process ID alone:
QEMU's guest agent appends each started command to a list and searches that list from its head.
A result whose status was never read is therefore returned for a later command with the same process ID.

Every command now starts by printing a line that holds a fresh 128-bit random marker.
Only a finished result whose output begins with that line is returned,
with the line removed.
A finished result without it is discarded,
logged as a warning,
and the status is asked again,
which makes the agent return the next result it holds for that process ID.
When the agent then no longer knows the process ID,
`mvm` fails with `GuestExecAttributionError`,
which lists every discarded result.
No result is ever returned by elimination.

A shell that stops before running anything prints no marker.
PowerShell parses a whole command first,
so a PowerShell text that does not parse ends this way;
the parser's message is in the discarded output the error shows.
On a POSIX shell the marker line is a separate first line,
so a syntax error later in the text still follows the marker and the failing result is returned as the command's own.

### Tests

`package/cli/mvm/src/guest-exec.unit.test.ts` runs the built package against a stand-in `virsh`
(`fixture.fake-virsh.ts`) that simulates the agent's process list.
Its cases,
by name:

- `returns the command result when one status poll times out while the command still runs`
- `returns the started command's own result when the agent still holds an older result for the same process ID`
- `refuses to return a result when the agent forgets the process ID after reporting only an older result`
- `reports a lost result when the agent knows no result for the started process`
- `ties a command with a shell syntax error to its own failing result`
- `reports the shell convention exit status for a command a signal ended`
- `passes a command with non-ASCII text through an ASCII-only argument and returns its UTF-8 output`
- `runs a Windows command through PowerShell and strips the marker line with its CRLF`
- `shows the shell error when a PowerShell text does not parse and so prints no marker`
- `stops at the first unanswered status request when libvirt no longer knows the domain`
- `stops when the domain shut off while the command ran`
- `gives up with the process ID once status requests stay unanswered past the limit`
- `says nothing ran when the agent refuses the launch`
- `says the command may be running when the launch gets no answer`
- `asciiJson escapes every non-ASCII character and stays valid JSON`
- `says the output is too large for libvirt when the finished result cannot be handed over, without asking again`

The first two were written first and failed against the old code;
that run is kept as `red-1.log` in the scratch directory named in `Evidence`.

### Output size

One result holds about 3 MiB of standard output and standard error together,
because the status answer carries both base64-encoded and libvirt hands over at most 4194304 characters.
A larger result is lost:
the agent drops it once it has answered.
`mvm` reports that as `GuestExecOutputTooLargeError` instead of asking again.
On a real Alpine guest,
2 MiB of output came back whole,
and 3.5 MiB failed in 1.6 seconds with that error,
after which the agent answered the next command.

Real guests confirmed the parts a stand-in cannot:
the agent's wording for an unknown process ID
(`error: guest agent command failed: unable to execute QEMU agent command 'guest-exec-status': PID ld does not exist`
from qemu-ga 10.1.3 on Alpine,
with `lld` in place of `ld` from the Windows agent 110.0.2),
exit status 3 with separate output streams on both guests,
non-ASCII output,
and a PowerShell parse error reported with the parser's message
(`An empty pipe element is not allowed.`).
Neither defect was reproduced on a real guest,
because both need a guest under load or a reused process ID;
the stand-in reproduces them on demand.

## Host tool commands

Code:
`package/cli/mvm/src/libvirt-tools.ts`,
`virsh.ts`,
`qemu-img.ts`,
`spawn.ts`,
`spawn-errors.ts`
and `libvirt-errors.ts`.

### Mechanism chosen

Each tool's command is decided once per process,
in this order:

1.  `MVM_VIRSH_COMMAND` or `MVM_QEMU_IMG_COMMAND`,
    a JSON array holding the executable and the arguments that go before the tool's own.
2.  The bare names `virsh` and `qemu-img`,
    when `virsh` is an executable in a directory of `PATH`.
3.  Both tools inside the virt-manager Flatpak,
    when `flatpak info --show-ref org.virt_manager.virt-manager` succeeds.
4.  The bare names,
    so that the failure names what is missing.

The MCP server bundles this code,
so it resolves the tools the same way,
and on this host neither program needs any setting.

### Alternatives considered

How the package took configuration before:
only through environment variables (`MVM_BACKEND`,
`MVM_HCLOUD_SERVER_TYPE`,
`MVM_HCLOUD_LOCATIONS`,
`HCLOUD_TOKEN`).
It reads no configuration file,
and the repository's configuration-file module `module-conf-fork` has no consumer.

- Environment variables plus detection of the Flatpak,
  the choice.
  For:
  matches how the package is configured,
  works for the command-line program and the MCP server alike,
  needs nothing on this host,
  and still allows any other command.
  Against:
  detection is one more thing that can surprise;
  the chosen command is logged at debug level and shown in every not-found message.
- Environment variables only.
  For:
  no detection.
  Against:
  the MCP server is registered with an empty environment,
  so the human would have to register it again,
  and every shell would need the exports.
- A configuration file under `~/.config`.
  For:
  set once for every caller.
  Against:
  a new file format,
  location
  and precedence rule in a package that has none.
- Command-line options and MCP tool arguments.
  For:
  explicit per call.
  Against:
  repeated on every call and added to every tool's schema.
- Detection only.
  For:
  nothing to learn.
  Against:
  no way to use a libvirt anywhere else.
- Shims on `PATH`,
  shipped by the package.
  Against:
  the MCP server's `PATH` does not include them,
  which is the failure the scanner runs hit.
- Variables set in the repository's `mise.toml`.
  Against:
  they reach `mise` tasks only,
  not the MCP server or a bare `mvm`.

Ranking:
variables plus detection > variables only > configuration file > options and tool arguments
> detection only > shims > `mise` variables.
Variables plus detection beats variables only because this host then needs no setup and no new registration.
Variables only beats a configuration file because the package already reads variables and no file.
A file beats options because it is set once.
Options beat detection only because they can still name another libvirt.
Detection only beats shims because it reaches the MCP server.
Shims beat `mise` variables because they at least work outside `mise` tasks.

### Session daemon

Decision:
`mvm` does not start the daemon.
Evidence,
measured on this host:

- `virsh` starts `virtqemud --timeout=120` on demand;
  the first call then takes 2.5 to 8 seconds and later calls 0.2 to 0.4 seconds.
- The daemon outlives the process that ran `virsh`,
  keeps running while a domain runs,
  and exits after two idle minutes.
- The two-minute wait the scanner runs avoided by starting the daemon first was not a slow start.
  A piped `virsh` call finished in 7.6 seconds,
  and its pipes stayed open for 128 seconds because the Flatpak sandbox of that call lives as long as the daemon in it.
  `nano-spawn` waits for the pipes.
  `spawn.ts` now sends output to files in a private temporary directory and completes when the process exits.

A daemon that `mvm` started would be one more process to own and stop,
for no measured gain.
What `mvm` does instead:
every `virsh` call has a deadline (two minutes,
or the agent timeout plus 30 seconds),
and a call that cannot connect or outlasts its deadline fails with
`LibvirtSessionUnavailableError` or `LibvirtUnresponsiveError`,
whose message shows the command that starts the daemon by hand.
On this host that command is
`flatpak run --command=virtqemud org.virt_manager.virt-manager --verbose`.

### Tests and real run

`package/cli/mvm/src/libvirt-tools.unit.test.ts` covers the choice of commands,
the variable's format,
and the runner:
a command that leaves a process holding its output open for 60 seconds must return when the command exits,
a deadline,
exit status and signal reporting,
and a `virsh` that outlasts its deadline.
`package/cli/mvm/src/cli-libvirt.unit.test.ts` runs the built program with a stand-in `virsh` named by the variable,
with a stand-in `flatpak` and no `virsh` on `PATH`,
and with a `virsh` that cannot connect.

Real run:
`mvm --verbose list` on this host,
with no shim on `PATH`,
no variable set
and no daemon running,
exited 0 in 8.3 seconds and logged
`virsh runs as ["flatpak","run","--command=virsh","org.virt_manager.virt-manager"] (flatpak)`.

## File transfer route

Code:
`package/cli/mvm/src/file-transfer-route.ts`,
`guest-file.ts`,
`guest-file-transfer.ts`,
`file-transfer.ts`,
and the callers `create.ts` and `clone.ts`.

### Detection

Before a domain is defined,
`chooseFileTransferRoute` writes a minimal domain that holds only the virtiofs share and its memory backing
and runs `virsh domxml-to-native qemu-argv --xml` on it.
libvirt prepares the share for that conversion exactly as for a start,
so a host without `virtiofsd` answers `Unable to find a satisfying virtiofsd`,
and nothing is defined or started.
Only that message selects the guest agent route;
any other outcome keeps the share.
`virsh domcapabilities` cannot serve as the check:
on this host it lists `virtiofs` as supported.

A VM on the guest agent route is defined without the `filesystem` device and without `memoryBacking`,
its cloud-init seed has no mount for the share,
and its `meta.json` holds `"fileTransfer": "guest-agent"`.
A `meta.json` without the field means the share,
so VMs created before this change keep working.

### Route chosen

The guest agent's file commands,
through one handle,
one command per chunk.

- Guest agent file commands,
  the choice.
  For:
  nothing is needed in the guest beyond the agent `mvm` already requires,
  the agent opens the path itself so spaces and non-ASCII names need no quoting,
  the same code serves Windows and Linux,
  and no host port is opened.
  Against:
  slow where each `virsh` call is slow.
- A file server on host loopback that the guest downloads from.
  For:
  fast.
  Against:
  needs a download program in the guest and a different command per guest system,
  a guest path quoted for that guest's shell,
  a listener on the host for the duration,
  and user-mode networking;
  pull needs an upload path as well.
- A 9p share.
  Against:
  no 9p driver was found for the Windows guest,
  so it would serve Linux guests only.
  This was not tested.
- A dedicated virtio-serial channel with a helper in the guest.
  Against:
  the helper must be installed in every template.
- One long-lived `virsh` fed commands on standard input.
  Against:
  measured slower than one call per chunk (0.12 MiB per second),
  because `virsh` reads a long line slowly.
- Several write handles at once.
  Against:
  the Windows agent opens a file without write sharing,
  so a second writer is refused.

Ranking:
agent file commands > loopback file server > 9p > virtio-serial helper > `virsh` on standard input > parallel handles.
The agent route beats the file server because it works for every guest with no guest-side command and no listener;
the file server stays the better tool for very large files and can be added later as an accelerator.
The file server beats 9p because it also serves Windows.
9p beats a helper because it needs nothing installed in Linux guests.
A helper beats `virsh` on standard input because the latter was measured slower than the chosen route.
`virsh` on standard input beats parallel handles because it at least works on Windows.

### Limits

From the QEMU guest agent protocol reference (version 11.1.50, fetched 2026-10-05) and measured here:

- One write carries 96000 bytes.
  The chunk travels base64-encoded inside one command-line argument of `virsh`,
  and Linux limits one argument to 131072 bytes.
- One read asks for 1 MiB.
  The protocol allows 48 MB,
  but the answer passes through libvirt:
  reads of 1 MiB and 2 MiB were answered,
  reads of 3 MiB and 4 MiB failed with `Unable to encode message payload`
  (libvirt allows one string 4194304 bytes,
  `REMOTE_STRING_MAX` in `src/remote/remote_protocol.x:49`),
  and a read of 8 MiB failed with
  `No complete agent response found in 10485760 bytes`
  (`QEMU_AGENT_MAX_RESPONSE` in `src/hypervisor/qemu_agent.c:57`),
  after which every agent command for that domain failed with
  `QEMU guest agent is not available due to an error`.
  `mvm` never asks for more than 1 MiB.
- A chunk whose command gets no answer is asked again,
  one second apart,
  after seeking to the chunk's absolute offset,
  so a chunk the guest already applied is not written twice at a shifted position.
  The asking ends when the agent answers,
  when the domain is shut off,
  crashed
  or unknown to libvirt,
  or when the silence has lasted five minutes
  (`package/cli/mvm/src/guest-file-retry.ts`).
  The first version asked three times in quick succession,
  which the first Windows run showed to be too few.
- After a push the size the guest reports must equal the host file's size.
  A failure names how many bytes arrived;
  the guest file is then incomplete.
- The guest path must be absolute,
  and its directory must exist.
  On the shared-directory route only the file name of the guest path is used,
  as before.
- `mvm pull` holds the whole file in memory before writing it.

Measured rates are in `Real runs on this host`.

### Tests

`package/cli/mvm/src/cli-file-transfer.unit.test.ts` runs the built program against the stand-in `virsh`,
which simulates guest files (`fixture.fake-guest-files.ts`).

The shared-directory route was pinned first,
and those tests passed against the build from before the change
(`transfer-against-old-build.log`):
push into the shared directory,
push to a Windows VM,
pull,
create with the share and its memory backing,
and create keeping the share when the check fails for another reason.
The domain XML generated with the share is byte-identical before and after
(`golden-domain-xml.before.txt` and `golden-domain-xml.after.txt`,
same SHA-256).

The guest agent route:
create without the share,
a binary file whose guest path has spaces and non-ASCII characters,
a Windows path where the end of the file shows only as an empty read,
an empty file,
exactly two write chunks,
exactly one read chunk,
several read chunks,
a write and a read that got no answer although the guest applied them,
an agent that stays silent for several commands and then answers,
a domain that shut off during the silence,
a silence that reaches the limit,
a write the guest refuses midway,
a write the guest cuts short,
a missing directory,
a missing file,
and relative paths.

Not covered by a test:
`mvm create --from` on the guest agent route;
see `Real runs on this host` for the real clone.

## Diagnostics for a missing host tool

Code:
`ExecutableNotFoundError` and `isMissingExecutable` in `package/cli/mvm/src/spawn-errors.ts`,
`toolNotFoundRemedy` in `libvirt-tools.ts`,
and the top-level handler in `cli.ts`.

With neither `virsh` nor the Flatpak,
`mvm list` now prints this and exits 1,
with no stack:

```text
mvm: The executable `virsh` was not found in any directory of PATH.
```

followed by the command `mvm` tried and the ways to provide the tool:
the host package,
the virt-manager Flatpak,
or `MVM_VIRSH_COMMAND`,
with the note that the MCP server reads the variable from its own environment.
A configured path that does not exist is reported as such,
naming the variable.
`qemu-img`,
`unzip`
and the OpenSSH programs of the Hetzner backend are reported the same way,
and `mvm shell` no longer exits quietly when `virsh` is missing.
`--verbose` prints the error with its stack and cause.

Tests:
the `Missing tools` cases of `cli-libvirt.unit.test.ts`
and the `isMissingExecutable` cases of `libvirt-tools.unit.test.ts`.
The OpenSSH paths are covered by the type check and the linter only.

## Waiting for a new VM's guest agent

`mvm create` waited 15 seconds for the guest agent and then failed,
leaving the domain running.
On this host an Alpine guest's agent first answered 52 seconds after `create` began.
The default wait is now five minutes and ends as soon as the agent answers
(`DEFAULT_AGENT_TIMEOUT_MS` in `package/cli/mvm/src/virsh-wait.ts`).

## Real runs on this host

All runs used the built program `package/cli/mvm/dist/final/node/cli.mjs`
with no shim on `PATH`,
no command variable,
and no daemon started by hand.
Every VM has the generated defaults:
4 virtual CPUs,
8192 MiB of memory,
a thin qcow2 overlay over the cached template (20 GiB for Alpine, 40 GiB for Windows),
and user-mode networking.
The host was under heavy load from other work throughout (load average between 30 and 106),
so the rates are lower bounds.

### Alpine

VM `fp-alpine-20261005`,
domain `mvm-fp-alpine-20261005`,
Alpine 3.23 with qemu-ga 10.1.3.

- `mvm create --image alpine` exited 0 in 88 seconds.
  The domain has no `filesystem` device and no `memoryBacking`,
  and `meta.json` holds `"fileTransfer": "guest-agent"`.
- `mvm exec` returned `uname -a`,
  exit status 3 with `out` and `err` on their own streams,
  and `héllo 漢字` unchanged.
- Three files were pushed to `/var/tmp/mvm test dir/`,
  hashed in the guest with `sha256sum`,
  pulled back,
  and hashed again.
  All three SHA-256 values agree for each file:
  - `small text.txt`,
    32 bytes,
    `b508ecc8e5de97fcefb26542ede1e53d60ff4cc464472359904fa0fb0bb6e1d6`.
  - `tëst fïle 漢字 😀.bin`,
    1048593 random bytes,
    `70bd6cdc3125fc7bb0eb3694d8a79bb904921a59881b31d0c13badbe8113f5ac`;
    push 3.6 seconds,
    pull 1.1 seconds.
  - `big-64MiB.bin`,
    67108864 random bytes,
    `345cc913261952971e6b07276e0b9a66386a17b13838364d2b0a1089e22544c0`;
    push 163 seconds (0.39 MiB per second),
    pull 21.8 seconds (2.9 MiB per second).
- The read sizes in `Limits` were measured in this run with direct agent commands.
  The 8 MiB read disabled the domain's agent,
  which ended the run's remaining checks;
  they were repeated in the second Alpine run.
- `mvm destroy` exited 0.
  `template-alpine.qcow2` had SHA-256
  `5894e8e5e5a64bfe7c487e3725958e0d7ef785c3ea8878454722f7aecb1f6936` and the same modification time before and after.

### Second Alpine run

VM `fp-alpine2-20261005`,
for what the first run did not reach.
`mvm create` exited 0 in 45 seconds.

- Transfer failures are reported without a stack,
  each exiting 1:
  - A push into a directory that does not exist quotes the agent's answer
    (`failed to open file '/no/such/dir/file.txt' (mode: 'wb'): No such file or directory`)
    and adds that the directory must exist because the agent does not create directories.
  - A push to the relative path `file.txt` is refused before the guest is touched,
    with the reason and an example path.
  - A push of a host file that does not exist reports the host path.
  - A pull of a guest file that does not exist quotes the agent's answer and writes no host file.
- A 32-byte push over a 1 MiB file left exactly the 32 bytes.
- Write sizes with direct agent commands:
  96000,
  98100
  and 98250 bytes were written;
  98400 bytes,
  a 131271-character argument,
  failed in the host kernel with `E2BIG`.
  The 96000 bytes `mvm` uses are close to the largest possible.
- Command output:
  2 MiB of output came back whole in one second.
  3.5 MiB of output failed after 30 seconds.
  The result is lost in that case,
  because the agent drops a finished result once it has answered with it,
  and libvirt refuses to pass an answer that large.
  The next command was answered normally.
  At the time of this run the failure read as a result that
  "was read by something else";
  `GuestExecOutputTooLargeError` now names the cause,
  see `Commits`.
- Reboot inside the guest:
  after `reboot`,
  five `mvm exec` calls over 47 seconds each exited 1 with
  `The guest agent ... did not confirm that the command started`,
  and the call at 52 seconds succeeded.
  The file pushed before the reboot had its SHA-256 afterwards,
  in the guest and after a pull.
- Clone:
  `mvm create --from fp-alpine2-20261005 fp-alpine2c-20261005` exited 1,
  because `qemu-img convert` cannot read the disk of a running VM:
  `Failed to get shared "write" lock`.
  The code before this work ran the same `qemu-img convert`,
  so this is not new;
  see `What remains`.
  The failed clone left an empty directory `~/.local/share/mvm/vms/fp-alpine2c-20261005`,
  which `mvm destroy` could not remove because no such domain existed;
  it was removed by hand.
- `mvm destroy` of the VM exited 0,
  and `template-alpine.qcow2` was unchanged.

### Windows

Three VMs from `template-windows.qcow2`,
Windows Server 2025 (`Microsoft Windows NT 10.0.26100.0`) with guest agent 110.0.2.
Commands run in PowerShell as `nt authority\system`.
The template had SHA-256
`0ff984dc6b93c3b4577d0836bae5cf76b1b905e4a7ceecce16ee066e23c40cb9`
and the same modification time before and after each run.

First VM,
`fp-windows-20261005`:

- `mvm create --image windows` exited 0 in 41 seconds,
  with no `filesystem` device and no `memoryBacking`.
- `mvm exec` returned exit status 3 with `out` and `err` on their own streams.
  `Get-Date |` was reported with PowerShell's `An empty pipe element is not allowed.`
- `small text.txt` (32 bytes) and `tëst fïle 漢字 😀.bin` (1048593 bytes) were pushed to `C:\mvm test dir\`,
  hashed in the guest with `Get-FileHash`,
  pulled back,
  and all three SHA-256 values agree for each
  (the values listed under `Alpine`).
  The 1 MiB file took 5.0 seconds to push and 4.2 seconds to pull.
- The 64 MiB push failed.
  After 46080000 bytes one write got no answer within 60 seconds,
  the next two tries failed within 25 seconds with
  `guest agent didn't respond to synchronization within '5' seconds`,
  and the transfer gave up,
  as the code then did after three tries.
  The agent answered nothing for at least 2 minutes 52 seconds,
  until the VM was destroyed;
  the host's load average was between 70 and 106.
  Whether the guest or its agent stalled was not established.
  This led to the commit that keeps asking for up to five minutes.

Second VM,
`fp-windows2-20261005`,
with the build from before that commit:

- `mvm create` exited 0 in 108 seconds.
- `big-64MiB.bin` was pushed whole in 348 seconds (0.18 MiB per second) at the first try
  and pulled in 16.8 seconds (3.8 MiB per second).
  The SHA-256 on the host,
  from `Get-FileHash` in the guest,
  and of the pulled file is
  `345cc913261952971e6b07276e0b9a66386a17b13838364d2b0a1089e22544c0`.
  The slowest stretch of that push,
  twice as slow as the others,
  fell about six minutes after the guest booted,
  the same age at which the first VM's agent went silent.
- A push into `C:\no such dir\` quoted the agent's `The system cannot find the path specified.`,
  and a pull of a missing file quoted `The system cannot find the file specified.` and wrote no host file.

Third VM,
`fp-windows3-20261005`,
with the final build:

- `mvm create` exited 0 in 76 seconds.
- `cmd.exe /c exit 5` gave exit status 1,
  with nothing on standard error.
  That is PowerShell's status for a command whose last program failed,
  not an `mvm` error.
  `cmd.exe /c exit 5; exit $LASTEXITCODE` gave 5,
  and `cmd.exe /c "echo building & exit 101"; exit $LASTEXITCODE` gave 101 with `building` on standard output.
- The 1 MiB file was pushed to `C:\Windows\Temp\mvm tëst 漢字.bin` in 12.7 seconds,
  pulled in 3.6 seconds,
  and came back with its SHA-256.

The patient retry was not seen at work on a real guest:
no command went unanswered in the second and third VM.
Its evidence is the tests named in `Commits`.

### VMs created and destroyed

- `fp-alpine-20261005`,
  first attempt:
  `create` failed at the 15-second agent wait;
  destroyed with `mvm destroy`.
  This run led to the longer wait.
- `fp-ready-alpine`:
  created to time the agent's first answer (52 seconds);
  destroyed.
- `fp-alpine-20261005`,
  second attempt:
  the Alpine run;
  destroyed.
- `fp-alpine2-20261005`:
  the second Alpine run;
  destroyed.
  Its clone `fp-alpine2c-20261005` never became a domain;
  the empty directory the failed clone left was removed by hand.
- `fp-windows-20261005`,
  `fp-windows2-20261005`
  and `fp-windows3-20261005`:
  the Windows runs;
  each destroyed with `mvm destroy`,
  including the first,
  whose push had failed.
- `fp-mcp-20261005`:
  created and destroyed through the rebuilt MCP server's tools;
  see `What the human must do for the MCP server`.

Directories that were under `~/.local/share/mvm/vms/` before this work and are still there,
all empty:
`ephemeral-8583ea1d`,
`ephemeral-be959ed3`,
`test-win`
and `wgq-bpf-test`.
They were not touched.

After each run `virsh list --all --name` showed exactly the six domains that existed before:
`bazzite-labwc-test`,
`cachyos-zfs-layout-validation`,
`cachyos-zfs-nodesktop-clean-validation`,
`cachyos-zfs-nodesktop-validation`,
`cachyos-zfs-usb-recovery-validation`
and `cachyos-zfs-validation`.

### Processes after the runs

No daemon or server was started by hand in any run.
`virsh` started `virtqemud`,
and libvirt started `virtlogd` and `virtstoraged`,
each with `--timeout=120`.
A process listing 70 seconds after the last VM was destroyed still showed the three;
a listing about a minute later showed none of them,
no QEMU process of an `mvm` domain,
and no driver of a run.
None had to be stopped.

## What the human must do for the MCP server

The MCP server lives in `package/mcp/mvm`,
which this work did not edit.
It bundles the source of `package/cli/mvm`,
so it had to be rebuilt:
`mise run //package/mcp/mvm:buildAndTest` exited 0 and wrote
`package/mcp/mvm/dist/final/node/index.mjs` at 23:21:49 UTC on 2026-10-05,
after the last source change of this work.

What the human must do:
end and start again every Claude Code session that should use the `mvm` tools.
Each session started its own server process,
`node /home/user/Monochromatic/package/mcp/mvm/dist/final/node/index.mjs`,
before the rebuild,
and that process keeps the old code in memory;
four such processes were running when this was written.
A new session starts a new process from the rebuilt file.
Reconnecting the `mvm` server from a session's `/mcp` panel should do the same without ending the session;
that was not tried.
The registration itself needs no change:
its empty environment is enough,
because the Flatpak is found without any variable.

Until then the tools run the old code.
From this session,
after the rebuild,
`list_vms` still answered
`Error: Command failed: virsh --connect 'qemu:///session' list --all`.
The MCP tools,
as a session calls them,
are therefore unverified.

What was verified instead:
a scratch client (`mcp-probe.ts`) started the rebuilt server with the registration's command
and spoke its protocol over standard input and output,
one request per process.
With VM `fp-mcp-20261005` (Alpine, 4 virtual CPUs, 8192 MiB):

- `list_vms` answered `No VMs found.` before and listed the VM as `running` after `create_vm`.
- `create_vm` with `image: alpine` created and started the VM.
- `exec_in_vm` returned `Linux` on standard output,
  `err` on standard error,
  and exit code 4.
- `push_to_vm` wrote the 1 MiB test file to `/var/tmp/mcp tëst 漢字.bin`,
  `sha256sum` in the guest gave
  `70bd6cdc3125fc7bb0eb3694d8a79bb904921a59881b31d0c13badbe8113f5ac`,
  and `pull_from_vm` brought back 1048593 bytes with the same SHA-256.
- `destroy_vm` destroyed the VM,
  and `list_vms` no longer showed it.

Not exercised through the server:
`run_in_vm`,
`update_templates`,
a Windows guest,
and the Hetzner backend.

One thing in that package is now out of date and is not this work's to edit:
the descriptions of `push_to_vm` and `pull_from_vm` in `package/mcp/mvm/src/tools-transfer.ts`
say that libvirt writes and reads "via the virtiofs shared mount".
On a VM without the share,
the guest path is the file's full path in the guest,
as the tools' `guestPath` argument already says.

## Running the scanner's Windows suite with mvm alone

The first consumer is the scanner's Windows suite,
`doc/handover/scanner-native-verification.md`,
section `Repeating the Windows run`.
The `mvm` command-line program alone can now do that whole run:
no shim,
no daemon started by hand,
no hand-edited domain,
no hand-written guest agent client,
and no MCP tool.
That statement rests on the Windows runs recorded here,
which exercised create,
commands,
push,
pull
and destroy;
the scanner suite itself was not run.
Step by step:

1.  Shims on `PATH` and a session daemon started first:
    neither is needed.
    Leave `PATH` alone and start nothing.
2.  Create,
    then undefine and redefine without the share:
    `mvm --backend libvirt create --image windows <name>` is the whole step.
    It exits 0 with the domain running,
    defined without `filesystem` and `memoryBacking`,
    with 4 virtual CPUs and 8192 MiB.
3.  Wait for the agent,
    run commands with a 60-second timeout,
    retried polls
    and a nonce:
    `mvm create` returns only once the agent answers,
    and `mvm exec <name> -- <PowerShell text>` does the rest itself.
    No nonce is needed in the command.
4.  License check,
    `slmgr.vbs /rearm`
    and reboot:
    each is one `mvm exec`.
    There is no command that waits for the agent after a reboot;
    repeat a cheap `mvm exec` until it exits 0.
    On Alpine such calls failed for 47 seconds after `reboot` and then succeeded;
    this was not tried on Windows.
5.  Toolchain:
    `mvm exec` runs the installers.
    The archives can be pushed with `mvm push <name> <host file> 'C:\w\<file>'`,
    at about 0.2 MiB per second on this host;
    an archive of 250 MiB would take about twenty minutes at that rate.
    The loopback file server of that section is not needed but stays much faster for the large archives.
6.  Sources:
    `mvm push` the archive (its directory must exist in the guest),
    then check its hash and extract it with `mvm exec`.
7.  Test run:
    one `mvm exec`.
    End the command with `; exit $LASTEXITCODE`,
    or a failing `cargo test` reports exit status 1 instead of 101.
    Keep the command's output under about 3 MiB,
    or redirect it to a file and `mvm pull` that.
    `mvm exec` waits as long as the command runs,
    asking again whenever a status request gets no answer.
8.  Positive control:
    `mvm push` the variant file over the original,
    rerun step 7,
    push the original back.
9.  Teardown:
    `mvm --backend libvirt destroy <name>` destroys and undefines the domain with its disk and removes its directory.
    There is no file server or daemon to stop unless one was started;
    the daemons that `virsh` started exit two minutes after the last domain is gone.
    Check the template's hash as before.

Every follow-up command must repeat `--backend libvirt` only if `MVM_BACKEND` is set to something else;
libvirt is the default.

## Decisions open to veto

Each was adopted without asking because the delegation asked for a choice and a record.

- The variable names `MVM_VIRSH_COMMAND` and `MVM_QEMU_IMG_COMMAND`,
  their JSON-array form,
  and the order in `Mechanism chosen`:
  a `virsh` on `PATH` wins over the Flatpak.
- `mvm` does not start the session daemon.
- The guest agent is the transfer route when libvirt finds no `virtiofsd`,
  and no loopback file server was built.
- On that route a guest path must be absolute,
  where the shared-directory route uses only the file name.
- Every guest command prints a marker line first.
  The line never reaches the caller,
  but the command text that runs in the guest is two lines longer than what was passed.
- The default wait for a new VM's agent is five minutes.
- A file transfer keeps asking a silent guest agent for five minutes before it fails,
  the same limit a running guest command has.
  A push to a guest that never answers again therefore takes that long to fail.
- The command-line program prints `mvm: <message>` and sets exit status 1 itself,
  instead of letting the error reach Node's uncaught handler.
  The repository's rule is to put error text in the thrown error and to set an exit status only for non-standard ones;
  the text is still in the thrown error,
  and the handler exists so that the user sees it without a stack through bundled code.
- `spawn.ts` uses `node:child_process` directly.
  `nano-spawn` stays a dependency of the package for the interactive `mvm shell` console
  and the Hetzner backend's `ssh` calls.

## What remains

Unverified:

- The MCP tools as a Claude session calls them,
  until the sessions are restarted.
- The retry through agent silence on a real guest;
  only tests show it.
  The scanner's Windows run on 2026-10-06 made 1,343 status requests during its mutation campaign
  and none went unanswered,
  so it did not exercise the retry either.

Observed since,
by the scanner's Windows run on 2026-10-06
(`doc/handover/scanner-native-verification.md`, section `Windows run with mvm alone`):

- The scanner's Windows suite ran with the built `mvm` command-line program alone:
  no PATH shims,
  no daemon started by hand,
  no MCP tools.
  No `mvm` command behaved differently from this document.
- A reboot of a Windows guest followed by `mvm exec`:
  a command sent 10 seconds after `shutdown /r` exited 1 with `GuestExecAttributionError` after 42.8 seconds,
  and the next one succeeded 76 seconds after the reboot request.
  As on Alpine,
  a caller repeats a cheap command until it succeeds;
  `mvm` has no wait command.
- The missing-`ssh` messages of the Hetzner backend,
  which no test or run exercises.
- The `claude mcp add` example in the package README,
  which follows that command's help text and was not run.
- Whether a domain with `memoryBacking` and without the share starts on this host;
  `mvm` removes both together.

Not done:

- No faster route for large pushes.
  A push runs at 0.2 to 0.4 MiB per second here,
  one `flatpak run` per 96000 bytes.
  A loopback file server as an accelerator is the candidate named in `Route chosen`.
- `mvm pull` on the guest agent route holds the file in memory.
- The cause of the Windows agent's silence was not established.

Found and left as they were,
because they are outside the four goals and each needs a decision:

- `mvm create --from <name>` fails while the source VM runs,
  with `Failed to get shared "write" lock` from `qemu-img convert`,
  and `mvm` has no command that stops a VM.
  The code before this work behaved the same.
  The options are a forced-share copy of a live disk,
  which can copy an inconsistent file system,
  or shutting the source down for the copy.
- A failed clone or create leaves the VM's directory behind,
  and `mvm destroy <name>` exits 1 for a name without a domain,
  so the directory stays.
  The four empty directories listed in `VMs created and destroyed` are older leftovers of that kind.
- `mvm create` leaves the domain running when the wait for the agent fails.
- The transfer tool descriptions in `package/mcp/mvm`,
  named in `What the human must do for the MCP server`.
- The linter reports 10 errors in this package,
  all `test-import(require-eventual-artifact)` in test files this work did not touch
  (the Hetzner tests,
  `backend/registry.unit.test.ts`
  and `iso9660.unit.test.ts`).
  There were 12 before;
  the two in `virsh-polling.unit.test.ts` went away when that file was changed to import the built package.

## Commits

On `main`,
in order:

- `f0d86a092`,
  `fix(cli-mvm)`:
  return only the started guest command's result,
  and ask again when a status poll gets no answer.
- `2d68ff937`,
  `feat(cli-mvm)`:
  choose the `virsh` and `qemu-img` commands per host,
  and stop waiting on inherited output streams.
- `930d69fbb`,
  `feat(cli-mvm)`:
  move files through the guest agent when libvirt finds no `virtiofsd`.
- `bf8f57ab0`,
  `fix(cli-mvm)`:
  wait up to five minutes for a new VM's guest agent.
- `b3c21676d`,
  `fix(cli-mvm)`:
  name the executable that was not found and how to provide it.
- `466df149a`,
  `docs(cli-mvm)`:
  the package README.
- `95d283d0e`,
  `fix(cli-mvm)`:
  keep a file transfer going while the guest agent is silent,
  and name a command output too large for libvirt.
  Its tests,
  each seen failing against the build from before it (`red-2.log`):
  `push keeps asking while the guest agent stays silent for several commands, then finishes without shifting the data`,
  `push stops at once when the domain shut off while the guest agent was silent`,
  `push gives up when the guest agent stays silent for the limit, and says for how long`,
  and
  `says the output is too large for libvirt when the finished result cannot be handed over, without asking again`.
- `787e4879d` and `70fe56efa`,
  `docs(cli-mvm)`:
  the limits the real runs measured,
  in the package README,
  and a wording fix of two of its list items.
- `a195a01b9`,
  `docs(troubleshooting)`:
  `doc/troubleshooting/mvm-libvirt-flatpak-only-host.md`.
- This handover,
  as `docs(handover)`.

State of the package's checks at `95d283d0e`:
`mise run //package/cli/mvm:buildAndTest` exits 0,
`mise run //package/cli/mvm:lint:types` exits 0,
and `mise run //package/cli/mvm:lint:oxlint` reports no warning and the 10 errors described in `What remains`.

## Evidence

Scratch directory,
private to the user:
`/home/user/temp/agent/mvm-flatpak-20261005/`.

- `red-1.log`:
  the two defect tests failing against the old code.
- `transfer-against-old-build.log`:
  shared-directory tests passing and guest agent tests failing against the build from before the transfer change.
- `golden-domain-xml.before.txt`,
  `golden-domain-xml.after.txt`:
  the generated domain XML with the share.
- `probe-daemon-timing.log`,
  `probe-survive-latency.log`,
  `probe-stdin-long-line.log`,
  `probe-agent-ready.alpine.log`:
  the measurements cited in the troubleshooting document.
- `real-list-1.log`:
  the real `mvm list`.
- `real-fp-alpine-20261005.first/`,
  `real-fp-alpine-20261005/`,
  `real-fp-alpine2-20261005/`,
  `real-fp-windows-20261005/`,
  `real-fp-windows2-20261005/`,
  `real-fp-windows3-20261005/`,
  `real-fp-mcp-20261005/`:
  one directory per real run,
  each with `run.log` (every command, its exit status and output) and `results.json`.
- `real-linux-run.ts`,
  `real-linux-run-2.ts`,
  `real-windows-run.ts`,
  `real-windows-run-2.ts`,
  `real-windows-run-3.ts`,
  `mcp-probe.ts`:
  the drivers of those runs.
- `red-2.log`:
  the four tests of commit `95d283d0e` failing against the build from before it.
- `bt-13.log`,
  `types-13.log`,
  `oxlint-13.log`:
  the last build with tests,
  type check
  and lint of the package.
- `mcp-bt-2.log`:
  the last build and test of the MCP server.

libvirt's source was read at tag `v12.4.0`,
commit `eb0b2e768d22f951f251e6e2ed4f00da9d27f444`,
cloned to `/home/user/temp/agent/libvirt-20261005`.
