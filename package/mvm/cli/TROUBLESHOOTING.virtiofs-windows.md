# Troubleshooting: virtiofs on Windows Server guests

This document exists because making virtiofs work on a Windows Server Core guest
took an entire evening of screaming into the void.
If you are reading this,
 you have my sympathy.

## The dependency chain from hell

virtiofs on Windows requires **four** separate components,
 installed in the correct order,
from three different sources,
 none of which tell you about the others.

1. **viofs kernel driver**:
    from the virtio-win ISO,
    installed via `pnputil`
2. **VirtioFsSvc** (user-mode service):
    from `virtio-win-gt-x64.msi` on the same ISO
3. **WinFsp** (Windows File System Proxy):
    a **separate download** from `github.com/winfsp/winfsp`,
   not included anywhere on the virtio-win ISO,
    not mentioned in any Red Hat documentation
   that you will find in the first 30 minutes of searching
4. **QEMU guest agent**:
    from `qemu-ga-x86_64.msi` on the virtio-win ISO,
   needed so the host can actually talk to the guest to verify any of this worked

None of these installers check for or install their dependencies.
VirtioFsSvc will happily register as a Windows service,
 report `Automatic` start type,
and then silently time out every single boot because WinFsp is not installed.
No error in the event log.
 No warning.
 Just `A timeout was reached (30000 milliseconds)`.

## What `virtio-win-gt-x64.msi` actually installs (and does not install)

Despite being called the "all-in-one guest tools" installer:

- It installs VirtIO drivers (balloon,
   network,
   serial,
   storage,
   etc.)
- It installs `virtiofs.exe` (the VirtioFsSvc service binary)
- It **does not** install the QEMU guest agent (`qemu-ga-x86_64.msi` is separate)
- It **does not** install WinFsp

Read that again.
 The "all-in-one" MSI does not install two of the four things
you need for virtiofs to work.
 The naming is a psychological attack.

## The autounattend FirstLogonCommands race condition

Windows autounattend `<FirstLogonCommands>` execute as `<SynchronousCommand>` entries
with `<Order>` numbers.
 They run sequentially.
 Sounds safe.

The trap:
 mvm's template creation code calls `waitForGuestAgent()` which polls
the QEMU guest agent channel.
 The moment the guest agent service starts (Order 2 in our case),
`waitForGuestAgent` returns and the template code **immediately shuts down the VM**
to proceed with the VirtIO disk bus switchover.

Any FirstLogonCommands with Order greater than 2 can be skipped by template shutdown.
They are registered,
 they appear in the setup log as "Set command",
and they vanish into the ether when the VM powers off.

This is why WinFsp and the real VirtioFsSvc setup are done via `guest-exec` after
the agent is up.
 Autounattend still contains a best-effort Order 3 service start,
but template creation does not rely on that command completing.

## The MSI serialisation trap

Windows Installer (`msiexec`) enforces a global mutex.
Only one MSI installation can run at a time.
If you chain two `Start-Process msiexec ... -Wait` calls in the same PowerShell command
inside a `<CommandLine>`,
 the second one may silently fail to start
or produce no log file whatsoever.

Diagnosis:
 if your MSI install log (`/log C:\something.log`) does not exist at all,
`msiexec` never launched.
 Not "launched and failed.
" Never launched.
Check whether another `msiexec` is still holding the mutex from a previous install.

## The CommandLine length bomb

If you try to combine too many operations into a single `<CommandLine>` element
in the autounattend XML,
 Windows Setup will fail with:

> Windows could not complete the installation.
>  To install Windows on this computer,
>  restart the installation.

No further detail.
 No log.
 No indication which command was the problem.
The XML parses fine.
 The PowerShell syntax is valid.
 The command is under 8191 characters.
Windows Setup simply gives up.

Keep individual `<CommandLine>` entries short.
 When in doubt,
 split into multiple Orders.
Then remember the race condition above and cry.

## PowerShell hash literals require actual newlines

PowerShell hash literal syntax `@{ key = value }` works on a single line
for simple cases:
 `@{ memory = "4G"; cpus = 4 }`.

Multi-line hash literals with nested `@{}` blocks **require actual newlines**
between entries.
 Semicolons are not equivalent.
 PowerShell will report
`The hash literal was incomplete` and point to what looks like valid syntax.

If you are constructing PowerShell commands by joining array elements with `'; '`,
switch to `'\n'` when hash literals are involved.

## PowerShell treats native command stderr as errors

When a native executable (like `mise.exe`) writes progress output to stderr,
PowerShell wraps each line in an `ErrorRecord` and sets `$LASTEXITCODE` to non-zero
even when the actual process exit code was 0.

Fix:
 `$ErrorActionPreference = "Continue"; & program.exe args 2>$null; exit $LASTEXITCODE`

The `2>$null` suppresses the stderr output.
The `$ErrorActionPreference` prevents PowerShell from treating it as terminating.
The `exit $LASTEXITCODE` forwards the real exit code.

## mise.exe requires the Visual C++ runtime

`mise.exe` (and probably many other Rust-compiled Windows binaries) requires
`vcruntime140.dll`.
 Windows Server Core does not include it.
The error code is `0xC0000135` (`STATUS_DLL_NOT_FOUND`),
 which surfaces as exit code
`-1073741515` with completely empty stdout and stderr.

Fix:
 install the VC++ redistributable before running mise:

```powershell
Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vc_redist.x64.exe" -OutFile "$env:TEMP\vc_redist.x64.exe"
Start-Process -FilePath "$env:TEMP\vc_redist.x64.exe" -ArgumentList "/install","/quiet","/norestart" -Wait
```

## `mise where` vs `mise which`

`mise where node` returns the **install root** (e.g. `.../node/26.3.1`).
`mise which node` returns the **full path to the executable** (e.g. `.../node/26.3.1/bin/node.exe`).

If you use `mise where` to set PATH,
 you get the directory without `bin/`.
`node` will not be found.

Fix:
 `$env:PATH = ((& mise.exe which node 2>$null) | Split-Path) + ";" + $env:PATH`

## `mise.jdx.dev` serves stale releases

The convenience URL `https://mise.jdx.dev/mise-latest-windows-x64.zip` served
version `2025.10.13` when the latest release was `2026.3.17`.

Fix:
 resolve the actual latest version via GitHub redirect,
 then download the versioned zip:

```powershell
$r = Invoke-WebRequest -Uri "https://github.com/jdx/mise/releases/latest" -MaximumRedirection 0 -UseBasicParsing -ErrorAction SilentlyContinue
$version = ($r.Headers.Location -split "/tag/")[1]
$url = "https://github.com/jdx/mise/releases/download/$version/mise-$version-windows-x64.zip"
```

## Guest agent runs as SYSTEM

The QEMU guest agent executes commands as `NT AUTHORITY\SYSTEM`.
This means:

- `$env:HOME` is not set (or points to `C:\WINDOWS\system32\config\systemprofile`)
- `$env:USERPROFILE` is `C:\WINDOWS\system32\config\systemprofile`
- mise installs tools under `systemprofile`,
   not `Administrator`
- There is no login shell,
   no PATH customisation,
   no profile loading

Every command must explicitly set environment variables.
Nothing you learned about "just set HOME" from Linux applies here.

## WinFsp MSI version numbering

WinFsp release tags are `v2.1` but the MSI filename is `winfsp-2.1.25156.msi`
(with a build number).
 You cannot construct the download URL from the tag alone.
Use the GitHub API to enumerate release assets and find the `.msi` that is not `tests`.

## The guest-file-write chunk size limit

The QEMU guest agent `guest-file-write` command accepts base64-encoded data
in the `buf-b64` field.
 This data is passed via `virsh qemu-agent-command`
which serialises the entire JSON payload as a command-line argument.

Linux has a `MAX_ARG_STRLEN` of 131072 bytes per argument.
A 512 KB chunk of raw data becomes ~700 KB of base64,
 which exceeds this limit.

Fix:
 use 48 KB raw chunks (~65 KB base64).
 The WinFsp MSI is ~1.5 MB,
requiring ~32 chunks.
 It takes about 2 seconds.
 This is fine.

## Timeline of this investigation

For posterity,
 and as a warning to others:

1. Tried adding `virtio-win-gt-x64.msi` as the sole MSI in autounattend → 40 minute timeout,
    guest agent never started (because the MSI does not install the guest agent)
2. Tried installing both MSIs in a single PowerShell command → second MSI silently never executed
3. Tried separate FirstLogonCommands Orders → guest agent came up at Order 3,
    template code shut down VM,
    Orders 4+ never ran
4. Moved guest tools install before guest agent start → same race,
    template shutdown kills it
5. Installed guest tools via `exec()` after agent is up →
   `exec()` used `/bin/bash` because no VM metadata exists during template creation
6. Used `virsh` directly with `powershell.exe` → VirtioFsSvc installed but wouldn't start
7. Discovered WinFsp dependency (not documented anywhere obvious)
8. Downloaded WinFsp,
    pushed via guest-file-write → chunk too large for virsh args
9. Fixed chunk size → everything works

Total attempts:
 9.
 Total template rebuilds:
 ~12.
 Total time:
 several hours.
The final working approach is ~50 lines of code.
