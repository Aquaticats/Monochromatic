# `mvm` libvirt backend fails on a host whose libvirt exists only in the virt-manager 5.1.0 Flatpak

## Status

Diagnosed on 2026-10-05 and fixed in `mvm` the same day.
`mvm` now finds `virsh` and `qemu-img` inside the Flatpak without configuration,
no longer waits two minutes on a piped `virsh` call,
defines domains without the virtiofs share where libvirt finds no `virtiofsd`,
and moves files through the guest agent there.
The changes are described in `Fix`;
commits,
tests
and the real runs on this host are in `doc/handover/mvm-flatpak-libvirt.md`.

The `mvm` MCP server runs a built artifact in a long-lived process.
It was rebuilt,
and a scratch client exercised the rebuilt server's tools,
but each running Claude Code session still holds a server process with the old code,
so the tools as a session calls them were not exercised;
see section `What the human must do for the MCP server` of that handover.

## Symptom

Before the fix,
every `mvm` libvirt operation failed before it reached a virtual machine.
The `mvm` MCP tool `list_vms` failed twice with this message,
recorded in the `Host bridges` section of `doc/handover/scanner-native-verification.md`:

```text
Command failed: virsh --connect 'qemu:///session' list --all
```

`mvm list` from the repository root reproduced it on 2026-10-05 and exited 1.
The final lines of its output, verbatim:

```text
SubprocessError: Command failed: virsh --connect 'qemu:///session' list --all
    at getErrorInstance (file:///var/home/user/Monochromatic/package/mvm/cli/dist/final/node/ephemeral-run-B0EZr1RU.mjs:3:15068)
    at getResultError (file:///var/home/user/Monochromatic/package/mvm/cli/dist/final/node/ephemeral-run-B0EZr1RU.mjs:3:14881)
    at spawnSubprocess (file:///var/home/user/Monochromatic/package/mvm/cli/dist/final/node/ephemeral-run-B0EZr1RU.mjs:4:704)
```

Before those lines Node printed one minified line of the bundled source,
several thousand characters long.
Neither the message nor the printed stack said that `virsh` was not found.

With `virsh` bridged into the Flatpak,
three further failures appeared,
each hiding the next:

- A `virsh` call whose output is piped took about two minutes when no session daemon was running.
  One piped `virsh version` took 122 seconds in the first scanner run;
  the same call measured here took 128 seconds.
- Creating a VM failed at start:

  ```text
  error: Failed to start domain 'mvm-fsnative-20261005'
  error: operation failed: Unable to find a satisfying virtiofsd
  ```

- With the virtiofs share removed,
  `mvm create` still exited 1,
  15 seconds after starting the domain:

  ```text
  guest agent on fp-alpine-20261005 did not respond within 15s
  ```

  The domain kept running and its agent answered half a minute later.

## Root cause

Four independent causes.
The `mvm` source cited in this section is the state at commit `3871dd434`,
before the fix.

### Host executables by bare name

`mvm` ran host executables by bare name from `PATH`.
`package/mvm/cli/src/virsh.ts:30`:

```ts
export function virsh({ args, }: { readonly args: readonly string[]; },): Promise<string> {
  return spawn({
    command: 'virsh',
    args: [
      '--connect',
      LIBVIRT_URI,
      ...args,
    ],
  },);
}
```

Disk images came from bare `qemu-img`,
`package/mvm/cli/src/create.ts:178`:

```ts
  await spawn({
    command: 'qemu-img',
```

This host provides neither executable outside a Flatpak sandbox.
Probes run on 2026-10-05:

- `command -v virsh virtiofsd` finds neither.
- `command -v qemu-img` finds `/home/user/.local/share/mise/shims/qemu-img`,
  but running it fails:
  `mise ERROR No version is set for shim: qemu-img`.
  The shim belongs to an Android SDK tool that is not active in this repository.
- `flatpak list` shows `org.virt_manager.virt-manager` 5.1.0
  and its extension `org.virt_manager.virt_manager.Extension.Qemu` 9.2.0.
- `flatpak run --command=virsh org.virt_manager.virt-manager --version` prints `12.4.0`.

The spawn failure surfaced as `Command failed` because `nano-spawn` 2.1.0 wraps it:
`source/result.js` builds `new SubprocessError(`Command failed: ${command}`, {cause: error})`,
and the operating system's `ENOENT` is only in that cause.

### A piped `virsh` call waits for the session daemon's sandbox

`virsh` starts libvirt's session daemon on demand:
with no daemon running,
`flatpak run --command=virsh org.virt_manager.virt-manager --connect qemu:///session version`
leaves `/app/bin/virtqemud --timeout=120` behind.
Inside a Flatpak that daemon lives in the sandbox of the `virsh` call that started it,
and the sandbox's first process,
which inherited the call's standard output and standard error,
stays alive as long as the daemon does.

Measured on 2026-10-05 with the call's output on pipes
(`probe-daemon-timing.log` in the scratch directory named in the handover):

```text
    234 ms  before: (no libvirt daemon process)
   7377 ms  stdout data (94 bytes)
   7644 ms  exit code=0 signal=null
   7782 ms  after exit: 242713 /app/bin/virtqemud --timeout=120
 128043 ms  close code=0
```

`virsh` exited after 7.6 seconds;
the pipes closed after 128 seconds,
when the idle daemon exited.
While the pipes were open the process list showed
`/usr/bin/bwrap --args 73 -- virsh --connect qemu:///session version` still running,
with the daemon as the only other process of that Flatpak instance.

`mvm` waited for the pipes,
not for the exit,
because `nano-spawn` 2.1.0 does:
`source/result.js` awaits `once(instance, 'close')`,
and Node emits `close` only when the process has ended and its stdio streams are closed.

This document's version from before the fix called the pipe explanation
"an inference, not a measurement".
The timeline and the process list are the measurement.
The 122 seconds were not a slow daemon start:
the daemon answered within 8 seconds.

The daemon is not tied to the process that ran `virsh`.
A second probe ran `virsh version` from a short-lived Node process with output sent to files,
and the daemon was still running after that process had exited
(`probe-survive-latency.log`).
libvirt's manual page for `virtqemud` says of `--timeout`:
"Exit after timeout period (in seconds),
provided there are neither any client connections nor any running domains."

### The virtiofs share needs a `virtiofsd` that libvirt can find

`mvm` always gave a domain a virtiofs share for file transfer,
`package/mvm/cli/src/domain-xml.ts:221`:

```ts
  // virtiofs shared directory for host-guest file transfer
  if (sharedDir !== undefined) {
```

```ts
            tag: 'driver',
            attrs: { type: 'virtiofs', },
```

and shared memory backing for it,
`package/mvm/cli/src/domain-xml.ts:273`:

```ts
  // virtiofs requires shared memory backed by memfd
  if (sharedDir !== undefined) {
```

libvirt 12.4.0 (tag `v12.4.0`, commit `eb0b2e768d22`) prepares every virtiofs device before a start.
`src/qemu/qemu_extdevice.c:113`:

```c
    for (i = 0; i < vm->def->nfss; i++) {
        virDomainFSDef *fs = vm->def->fss[i];

        if (fs->fsdriver == VIR_DOMAIN_FS_DRIVER_TYPE_VIRTIOFS) {
            if (qemuVirtioFSPrepareDomain(driver, fs) < 0)
                return -1;
        }
    }
```

`src/qemu/qemu_virtiofs.c:469` looks for a `virtiofsd` when the XML names neither a socket nor a binary:

```c
    } else {
        if (qemuVhostUserFillDomainFS(driver, fs) < 0)
            return -1;
    }
```

and `src/qemu/qemu_vhost_user.c:483` walks the vhost-user descriptor files,
failing when none describes a usable file system daemon:

```c
    for (i = 0; i < nvus; i++) {
        qemuVhostUser *vu = vus[i];

        if (vu->type != QEMU_VHOST_USER_TYPE_FS)
            continue;

        fs->binary = g_strdup(vu->binary);

        /* skip binaries that can't report their capabilities */
        if (qemuVhostUserFillFSCapabilities(&fs->caps,
                                            vu->binary) == -1)
            continue;
        break;
    }

    if (i == nvus) {
        virReportError(VIR_ERR_OPERATION_FAILED, "%s",
                       _("Unable to find a satisfying virtiofsd"));
        goto end;
    }
```

Inside the virt-manager Flatpak,
`command -v virtiofsd` finds nothing,
and neither `/app/libexec/virtiofsd` nor `/usr/libexec/virtiofsd` exists,
so the walk finds nothing and the start fails.

Converting domain XML to a QEMU command line goes through the same preparation without starting anything:
`qemuConnectDomainXMLToNative` calls `qemuProcessCreatePretendCmdPrepare` (`src/qemu/qemu_driver.c:6388`),
which calls `qemuProcessPrepareDomain` (`src/qemu/qemu_process.c:9129`),
which calls `qemuExtDevicesPrepareDomain` (`src/qemu/qemu_process.c:7160`).
That is what makes `virsh domxml-to-native` usable as a check before a domain is defined;
see `Fix`.

### The guest agent got 15 seconds

`package/mvm/cli/src/virsh-wait.ts:35`:

```ts
const DEFAULT_AGENT_TIMEOUT_MS = 15_000;
```

`mvm create` waited that long for the guest agent after `virsh start`.
On 2026-10-05,
with the host busy (load average between 30 and 60),
a fresh Alpine 3.23 VM's agent first answered 52 seconds after `mvm create` began,
about 45 seconds after the start
(`probe-agent-ready.alpine.log`).
The Windows runs recorded in `doc/handover/scanner-native-verification.md` waited for the agent by hand;
one of their logs shows it answering on the 16th attempt at 5-second intervals after a reboot.

## Verification

Versions:
virt-manager Flatpak 5.1.0 with libvirt 12.4.0 and QEMU 11.0.1 inside it,
`nano-spawn` 2.1.0,
Node 26.10.0,
mise 2026.10.0.
Diagnosis at repository commit `3871dd434`;
fix verified at the commits listed in `doc/handover/mvm-flatpak-libvirt.md`.

Harness for the diagnosis,
run with no `virtqemud` process alive:

```sh
# doc/troubleshooting/mvm-libvirt-flatpak-only-host.md
command -v virsh virtiofsd
flatpak run --command=virsh org.virt_manager.virt-manager --version
# Takes about two minutes although virsh itself finishes in seconds:
flatpak run --command=virsh org.virt_manager.virt-manager --connect qemu:///session version | cat
```

Harness for the virtiofs check,
with `<dir>` an existing directory under the home directory,
which the Flatpak can read:

```sh
# doc/troubleshooting/mvm-libvirt-flatpak-only-host.md
cat > <dir>/with.xml <<'XML'
<domain type="kvm">
  <name>mvm-virtiofs-probe</name>
  <memory unit="MiB">64</memory>
  <vcpu>1</vcpu>
  <memoryBacking><source type="memfd" /><access mode="shared" /></memoryBacking>
  <os><type arch="x86_64">hvm</type></os>
  <devices>
    <filesystem type="mount" accessmode="passthrough">
      <driver type="virtiofs" />
      <source dir="<dir>" />
      <target dir="mvm-shared" />
    </filesystem>
  </devices>
</domain>
XML
flatpak run --command=virsh org.virt_manager.virt-manager --connect qemu:///session \
  domxml-to-native qemu-argv --xml <dir>/with.xml
```

### Works

- `flatpak run --command=virsh org.virt_manager.virt-manager` and `--command=qemu-img`.
- A `virsh` call whose output goes to files:
  it returns when `virsh` exits,
  2.5 to 8 seconds with no daemon running and 0.2 to 0.4 seconds with one,
  of which a bare `flatpak run --command=true` takes about 0.35 seconds.
- `virsh domxml-to-native qemu-argv` for a minimal domain without a `filesystem` device:
  exit 0,
  prints a QEMU command line,
  defines nothing.
- With the fix:
  `mvm list`,
  `create`,
  `exec`,
  `push`,
  `pull`
  and `destroy` on this host with no configuration,
  for an Alpine guest and a Windows guest;
  results in `doc/handover/mvm-flatpak-libvirt.md`.

### Fails

- Before the fix,
  any `mvm` libvirt command or MCP tool with the default `PATH`:
  `Command failed: virsh --connect 'qemu:///session' list --all`.
- A domain with a virtiofs share,
  at start and at `virsh domxml-to-native`:
  `Unable to find a satisfying virtiofsd`.
- `virsh domxml-to-native` for mvm's full domain XML without the share:
  `Unable to create tap device vnet0: Operation not permitted`.
  The conversion rewrites network interfaces,
  so the check uses a minimal domain that holds only the share.
- Before the fix,
  `mvm create` when the agent needs more than 15 seconds:
  `guest agent on <name> did not respond within 15s`.

## Fix

Applied in `package/mvm/cli`.

### Tool commands

`package/mvm/cli/src/libvirt-tools.ts` chooses the command for each tool:
the JSON array in `MVM_VIRSH_COMMAND` or `MVM_QEMU_IMG_COMMAND`,
else the bare name when `virsh` is on `PATH`,
else the tool inside the `org.virt_manager.virt-manager` Flatpak when `flatpak info` knows it.
The CLI and the MCP server share that code.
A missing executable is reported with its name,
the command `mvm` tried,
and the three ways to provide it,
and the CLI prints that message instead of an uncaught error.

### Waiting for the exit

`package/mvm/cli/src/spawn.ts` no longer uses `nano-spawn`.
It sends the command's output to two files in a private temporary directory,
completes on the process's `exit` event,
then reads the files.
A daemon left behind by the command cannot hold the call open.

`mvm` does not start the session daemon.
`virsh` starts it on demand,
it outlives the process that ran `virsh`,
and it keeps running while a domain runs.
Every `virsh` call has a deadline of two minutes,
or the guest agent timeout plus 30 seconds.
A call that cannot connect,
or is still running at its deadline,
fails with an error that names the command to start the daemon by hand.

### Starting the daemon by hand

Only needed to find out why the daemon does not come up.
Run it in the background with its output sent to a file,
not to a pipe:

```sh
# doc/troubleshooting/mvm-libvirt-flatpak-only-host.md
flatpak run --command=virtqemud org.virt_manager.virt-manager --verbose > virtqemud.log 2>&1 &
```

libvirt then starts `virtlogd` and `virtstoraged` in the same sandbox as needed,
each with a 120-second idle exit.
Stop all three by process ID when done.

### Domains without the share

Before defining a domain,
`package/mvm/cli/src/file-transfer-route.ts` writes a minimal domain holding only the share and its memory backing
and runs `virsh domxml-to-native qemu-argv` on it.
When that fails with `Unable to find a satisfying virtiofsd`,
the VM is defined without the `filesystem` device and without `memoryBacking`,
its cloud-init seed has no `mvm-shared` mount,
and its `meta.json` records `"fileTransfer": "guest-agent"`.
Any other failure of the check keeps the share,
as before.

`memoryBacking` is left out together with the share because it exists only for the share:
virtiofs needs guest memory that `virtiofsd` can map.
The second scanner run removed both elements by hand.
Whether a domain with the memory backing and without the share starts on this host was not tested.

### Files through the guest agent

For such a VM,
`push` and `pull` use the guest agent's file commands
(`package/mvm/cli/src/guest-file-transfer.ts`).
The guest path must be absolute.
Limits and measured rates are in `doc/handover/mvm-flatpak-libvirt.md`,
section `File transfer route`.

### Agent wait

The default wait for a new VM's guest agent is five minutes instead of 15 seconds.
The wait ends as soon as the agent answers.

## Verified workarounds

Superseded by `Fix`.
Kept because the two scanner runs used them and their evidence refers to them.

### Bridge `virsh` and `qemu-img` into the Flatpak with shims

The Windows-native scanner run on 2026-10-05 used these scratch shims,
retained at `/home/user/temp/agent/scanner-windows-native-20261005/shim/`,
with that directory first on `PATH` for the `mvm` CLI:

```js
// /home/user/temp/agent/scanner-windows-native-20261005/shim/virsh
#!/usr/bin/env node
// Scratch bridge: this host's only libvirt client lives in the virt-manager Flatpak, while mvm spawns bare `virsh`.
const { spawnSync } = require('node:child_process');
const result = spawnSync(
  'flatpak',
  ['run', '--command=virsh', 'org.virt_manager.virt-manager', ...process.argv.slice(2)],
  { stdio: 'inherit' },
);
if (result.error !== undefined) {
  throw result.error;
}
process.exitCode = result.status === null ? 1 : result.status;
```

The `qemu-img` shim is identical with `--command=qemu-img`.

Tradeoffs:

- The `mvm` MCP server keeps its own `PATH`,
  so the MCP tools still fail;
  only the CLI benefits.
- The shim does nothing about the two-minute wait.
  Both runs avoided it by starting the session daemon first,
  with the command in `Starting the daemon by hand`.

### Redefine the domain without the share and move files over loopback HTTP

Both runs let `mvm create` fail at start,
then ran `virsh undefine`,
removed the `filesystem` device from a copy of `domain.xml`
(the second run also removed `memoryBacking`),
and ran `virsh define` and `virsh start` on the copy.
Files moved over HTTP served on host loopback,
which a user-mode network guest reaches at `10.0.2.2`.

Tradeoffs:

- `push_to_vm` and `pull_from_vm` stay unusable;
  every transfer is a hand-written download in the guest.
- The HTTP server exposes whatever directory it serves to the guest and to the host's loopback;
  serve a private scratch directory holding only the files to transfer.
- The guest needs an HTTP client,
  and the download command differs per guest system.

## What does not work

- The `mise` shim for `qemu-img`:
  it resolves to an inactive tool and fails.
- Expecting the Flatpak to provide `virtiofsd`:
  neither checked path exists inside it,
  and the domain cannot start with a virtiofs device.
- Asking `virsh domcapabilities` whether virtiofs works.
  On this host it lists `virtiofs` under `<filesystem supported='yes'>` although no `virtiofsd` exists
  (`domcaps.xml` in the first scanner run's scratch directory),
  so it cannot serve as the check.
- Looking for a `virtiofsd` executable from `mvm`.
  libvirt looks inside its own sandbox,
  through descriptor files,
  and `mvm` would look on the host.
- Reading a `virsh` call's output from pipes and waiting for them to close;
  see `A piped virsh call waits for the session daemon's sandbox`.
  The scratch driver for the real runs made the same mistake once and waited 125 seconds for `virsh list`.
- Feeding guest agent commands to one long-lived `virsh` through standard input,
  to save the cost of starting `flatpak run` for every chunk of a file.
  `virsh` reads its input a line at a time:
  a 64 KiB line took 3.7 seconds,
  a 2 MiB line 16.8 seconds,
  slower than one `virsh` call per 96000 bytes
  (`probe-stdin-long-line.log`).
- Asking the guest agent for an answer larger than libvirt hands over.
  The daemon returns the whole answer to `virsh` as one string
  (`remote_string result` in `src/remote/qemu_protocol.x:55` of libvirt 12.4.0),
  and a string may be 4194304 characters,
  `src/remote/remote_protocol.x:49` and `:52`:

  ```c
  const REMOTE_STRING_MAX = 4194304;
  ```

  ```c
  typedef string remote_nonnull_string<REMOTE_STRING_MAX>;
  ```

  A `guest-file-read` of 3 MiB,
  whose base64 answer is that long before its JSON wrapping,
  failed with `error: Unable to encode message payload`
  (raised in `virNetMessageEncodePayload`, `src/rpc/virnetmessage.c:381`),
  and so did the status of a guest command that had printed 3.5 MiB.
  The agent drops a finished command's result once it has answered,
  so that result is lost.
  A larger answer does more damage.
  `src/hypervisor/qemu_agent.c:57` and `:385`:

  ```c
  #define QEMU_AGENT_MAX_RESPONSE (10 * 1024 * 1024)
  ```

  ```c
          if (agent->bufferLength >= QEMU_AGENT_MAX_RESPONSE) {
              virReportSystemError(ERANGE,
                                   _("No complete agent response found in %1$d bytes"),
                                   QEMU_AGENT_MAX_RESPONSE);
  ```

  A `guest-file-read` of 8 MiB failed with that message,
  and every later agent command for the domain failed with
  `Guest agent is not responding: QEMU guest agent is not available due to an error`
  until the domain was destroyed.
  `mvm` reads 1 MiB at a time,
  and reports a command whose output is too large as such.
- Writing one guest file through several agent handles at once.
  The Windows guest agent opens files with read sharing only
  (`share_mode = FILE_SHARE_READ` in `qmp_guest_file_open`,
  `qga/commands-win32.c` of QEMU's current source),
  so a second handle for writing is refused.

## Upstream filing decision

`.out-of-scope/` was checked on 2026-10-05;
no entry covers libvirt,
Flatpak
or virt-manager.

1.  Is it really upstream's fault?
    No.
    `mvm` is this repository's tool.
    It assumed host executables,
    waited on pipes a sandboxed daemon legitimately keeps,
    and assumed a `virtiofsd`.
    libvirt's on-demand daemon start,
    Flatpak keeping a sandbox alive while a process runs in it,
    and libvirt refusing a share it cannot serve are each working as documented.
    Whether Flathub's virt-manager should ship `virtiofsd` was not investigated.
2.  Can upstream fix it?
    Not applicable.
3.  Is the use case supported?
    Not applicable;
    the Flatpak is not presented as a provider of host command-line tools.
4.  Would the repository welcome a contribution?
    Not applicable.
5.  Would they likely fix it?
    Not applicable.
6.  Is a minimal fix prototyped?
    Not applicable upstream;
    the fix is in this repository and described in `Fix`.

Decision:
nothing to file upstream,
no duplicate search was needed,
and no draft is kept.
