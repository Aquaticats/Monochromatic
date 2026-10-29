# Libvirt 10.0.0 rejects long private guest-agent socket paths in Depot CI

## Symptom

Depot run `99tl7zjfgp` installed Ubuntu package `libvirt-daemon 10.0.0-2ubuntu8.19`
and started private session daemons.
Preparing an Ubuntu guest failed with this libvirt diagnostic:

```text
error: internal error: UNIX socket path '/home/runner/temp/agent/depot-complete-vm-home-OEnKBv/runtime/libvirt/qemu/run/channel/1-mvm-template-setup/org.qemu.guest_agent.0' too long
```

The run's private domain list was empty before preparation and after cleanup.
No guest execution is established by this attempt.

## Root cause

The trial put `XDG_RUNTIME_DIR` inside a long private home path.
Libvirt appends its own state and guest-channel directories.
Upstream tag `v10.0.0`,
commit `1899d7df34576e414c49e4d91c68e6e2348f37f2`,
contains the relevant source;
this trace does not claim every Ubuntu patch is identical to that tag.

`src/util/virutil.c:531` appends the library directory:

```c
// libvirt, src/util/virutil.c:531 at v10.0.0
return g_build_filename(g_get_user_runtime_dir(), "libvirt", NULL);
```

`src/qemu/qemu_conf.c:190` builds session state and channel directories:

```c
// libvirt, src/qemu/qemu_conf.c:190 at v10.0.0
rundir = virGetUserRuntimeDirectory();
cfg->stateDir = g_strdup_printf("%s/qemu/run", rundir);
cfg->swtpmStateDir = g_strdup_printf("%s/swtpm", cfg->stateDir);
cfg->channelTargetDir = g_strdup_printf("%s/channel", cfg->stateDir);
```

`src/qemu/qemu_command.c:4965` rejects a path that does not fit `sockaddr_un.sun_path`:

```c
// libvirt, src/qemu/qemu_command.c:4965 at v10.0.0
if (virStrcpyStatic(addr.sun_path, dev->data.nix.path) < 0) {
    virReportError(VIR_ERR_INTERNAL_ERROR,
                   _("UNIX socket path '%1$s' too long"),
                   dev->data.nix.path);
```

The same operating-system boundary is documented for the nested compositor in
[nested Wayland footguns](nested-wayland-gui-test-footguns.md).
Changing the VM disk image cannot shorten this host socket address.

## Verification

The failing catalog includes the full path and provider run in Symptom.
Successful preparation boundaries in that run were the source bundle build,
package installation,
both daemon starts,
empty private session listing,
and the actual source `mvm --help` invocation.
Those successes do not prove guest boot.
The retained `logger-vm-evidence` artifact contains each command's complete stdout and stderr.

Trial harness `906bf436de` moved sockets to a unique directory under the inherited `XDG_RUNTIME_DIR`
and asserted the complete worst-case channel address was shorter than 108 bytes.
Retry `8wfm7bfgjv` found that environment variable absent and stopped at the harness assertion:

```text
AssertionError [ERR_ASSERTION]: Cloud user runtime directory must exist before creating private libvirt sockets
```

This is a separate failed setup assumption,
not another libvirt socket-length rejection.
It occurred before VM command recording and produced no VM evidence artifact.

## Verified socket layout and tradeoffs

Harness `295045dd82` on branch `trial/depot-complete-vm-20261016`
creates `/run/user/<uid>` with owner-only permissions when missing,
then allocates a unique `v-` directory inside it.
Only sockets and daemon PID files use that runtime path.
Images and configuration remain in the private home under `temp/agent`.
The full channel address must still fit the byte-length assertion.
Run `7gxcgml68r` crossed the runtime setup gate,
then exposed another long socket under the private configuration directory:
the QEMU `monitor.sock`.
Harness `0bcfa40820` also moves private configuration into the short runtime directory
and checks that complete monitor address.
Run `rv6hfj76w9` crossed both socket-length gates and progressed through template preparation.
The final prepared guest then failed with a different diagnostic:
QEMU could not connect to its virtiofs socket because the connection was refused.
That result does not establish why virtiofsd exited.
Diagnostic harness `01c948cac2` retains private QEMU and virtiofs daemon logs before cleanup.
Both socket failures are tracked in issue `#1074`;
the independently verified missing-runtime repair is closed in `#1075`.
Run `rv6hfj76w9` actually booted the template,
received `guest agent on template-setup is ready`,
shut the guest down,
and saved its standalone template image.
Those executed boundaries verify the short runtime and monitor paths,
so `#1074` is closed.
They do not establish a working shared-directory guest or lifecycle suite.

The recipe is for a disposable runner with sudo,
not a request to change the user's real libvirt state.
It uses a separate `qemu:///session`,
asserts an empty initial domain list,
and permits cleanup only for the explicitly owned trial domain names.
Guest preparation and teardown remain independently required.

## Separate virtiofs primary-group prerequisite

Run `22b5683gx8`,
attempt `w1zscshgq1`,
used diagnostic harness `01c948cac225276eedf76c5c062227eea9ba90c3`
and unchanged historical source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`.
Its `logger-vm-evidence` artifact is 18,429 bytes,
SHA-256 `05e1a751216a14332fa458a4c3538cd5869f0bdff9574abb5e553216546c3581`.
The retained `daemon-mvm-depot-preparation-ubuntu-fs0-virtiofsd.log`
identifies why the shared-directory daemon did not accept QEMU's connection:

```text
[2026-10-16T04:09:02Z ERROR virtiofsd::sandbox] sandbox: couldn't setup id mappings: write to uid map failed: newuidmap: Target process 3476 is owned by a different user: uid:1001 pw_uid:1001 st_uid:1001, gid:1001 pw_gid:117 st_gid:1001
```

The process and target GID are 1001,
while passwd requires primary GID 117.
This is the same ownership guard independently traced for
[rootless Podman](depot-rootless-podman-group-identity.md),
but a different consumer boundary.
The observed virtiofs daemon diagnostic establishes this VM preparation failure;
the Podman result alone would not establish it.

The current inspected Shadow revision `e9bc595fd20558cf51aef096db262232090e8fb8`
still rejects the mismatch in `src/newuidmap.c:132`:

```c
/* Shadow, src/newuidmap.c:132 */
if ((getuid() != pw->pw_uid) ||
    (!getdef_bool("GRANT_AUX_GROUP_SUBIDS") && (getgid() != pw->pw_gid)) ||
    (pw->pw_uid != st.st_uid) ||
    (getgid() != st.st_gid)) {
```

That source guard and the actual daemon log support a group-identity rejection.
They do not establish why the prepared sandbox inherited this group,
or establish a libvirt or Shadow defect.
The deprecated `-o` option warning in the same log is separate from the fatal diagnostic.
Changing only socket paths cannot correct the observed primary-group mismatch.

Harness `a683cd54f9` records runner process and passwd group identities,
then relaunches the disposable harness with the same UID and passwd primary GID when needed.
It uses `sudo --non-interactive --preserve-env setpriv`
with `--reuid`,
`--regid`,
and `--init-groups`.
A marker prevents recursive relaunch;
an assertion rejects a second mismatch.
This candidate changes only the disposable cloud process identity,
not passwd,
the workstation account,
or historical source.
The actual invocation in `vq299blkbw` reached final Ubuntu guest creation successfully;
the recorded `prepare-ubuntu` command exited zero at `2026-10-17T04:39:59.774Z`,
followed by successful `destroy-preparation-ubuntu` at `04:40:00.167Z`.
This exercises the shared-directory guest boundary that failed in the fifth run.
The retained identities keep UID 1001
and change primary GID from 1001 to the passwd GID 117.
The actual virtiofs daemon log records `Client connected, servicing requests`
and a normal client disconnect during guest destruction.
The separate `Couldn't set the process uid as root: -1` warning is retained;
successful client service does not explain or remove that warning.
The Ubuntu preparation result verifies this consumer-side group normalization.
The VM consumer gap is tracked in
[issue #1088](https://github.com/Aquaticats/Monochromatic/issues/1088).

Recovery dispatch `042gflb8jn`
attempted to upload the local harness change over provider checkout `01c948cac2`.
The injected patch failed before VM setup:

```text
error: patch failed: package/dev-script/task-util/mise.toml:549
error: package/dev-script/task-util/mise.toml: patch does not apply
```

No VM evidence artifact was created.
This is provider preparation overhead,
not a test of primary-group normalization.
The next attempt must begin only after the harness push completes
and `git ls-remote` matches its local committed revision.
Clean recovery run `vq299blkbw` uses remotely verified harness
`a683cd54f98b11add860c1bab48b33d14883a370`.
Its dispatch reported no local patch upload.
The run retains the 16-CPU,
64-GiB disposable cloud allocation,
75-minute provider timeout,
and 60-minute experiment deadline with cleanup reserve.
The run is terminal with a separate failed Windows prerequisite.
The group mismatch issue `#1088` is resolved by its measured Ubuntu consumer proof;
the Windows timeout remains open in `#1109`.

The fifth run built the historical source bundle,
again measured the original package task's separate working-directory failure,
booted and shut down the template,
and failed final guest preparation.
Cleanup errors were empty,
and the private domain list was empty after cleanup.
No Ubuntu/Windows lifecycle test pass follows from this attempt.
The original task remains tracked by `#1076`;
running the source suite from the repository root cannot replace that task result.

## Separate Windows template verification timeout

In `vq299blkbw`,
Windows installation reached an answering guest agent at `04:48:20.244Z`.
The WinFsp and VirtIO guest-tools installation commands returned,
then the template shut down at `04:48:49.712Z`.
The redefined VirtIO disk configuration started its guest-agent wait at `04:48:49.838Z`.
The `prepare-windows` command exited 1 at `04:53:51.603Z` with:

```text
mvm: guest agent on template-setup did not respond within 300s
```

At frozen source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`,
`package/mvm/cli/src/template-windows-virtio.ts:57` rebuilds the domain for the post-install pass:

```ts
// package/mvm/cli/src/template-windows-virtio.ts:57
const virtioXml = domainXml({
  diskPath,
  name: TEMPLATE_VM_NAME,
  osFamily: 'windows',
},);
await defineVm({
  vmDir,
  xml: virtioXml,
},);
await startVm({ name: TEMPLATE_VM_NAME, },);
```

`package/mvm/cli/src/template-windows-virtio.ts:69`
then requests the configured verification timeout:

```ts
// package/mvm/cli/src/template-windows-virtio.ts:69
await waitForGuestAgent({
  name: TEMPLATE_VM_NAME,
  timeoutMs: VIRTIO_VERIFY_AGENT_TIMEOUT_MS,
},);
```

`package/mvm/cli/src/config.ts:87` supplies its measured deadline:

```ts
// package/mvm/cli/src/config.ts:87
export const VIRTIO_VERIFY_AGENT_TIMEOUT_MS = 300_000;
```

`package/mvm/cli/src/virsh-wait.ts:121` emits the timeout after failed guest pings:

```ts
// package/mvm/cli/src/virsh-wait.ts:121
if (elapsed >= timeoutMs) {
  throw new Error(
    `guest agent on ${name} did not respond within ${
      String(timeoutMs / MS_PER_SECOND,)
    }s`,
    { cause: error, },
  );
}
```

This locates the observed failure after the disk switch.
It does not establish whether the guest failed to boot,
the storage driver failed,
or the agent service failed.
Increasing the timeout is untested.
There is no verified workaround for this separate prerequisite.
The issue is [#1109](https://github.com/Aquaticats/Monochromatic/issues/1109).
No Windows application fix or additional VM attempt is included in this diagnosis.

## Terminal source and coverage verification

The terminal group-recovery attempt retains artifact
`01a14835-9090-7ae6-9802-21e1bc74a1ee`,
21,474 bytes,
SHA-256 `606cb2a627cc1dcf9bc99d4bb310e9e9ae00c05fcd4ef90b4dc951c0559663e1`.
Its lifecycle record reports unchanged source HEAD and empty tracked status.
The source lockfiles and original lifecycle test hashes match the immutable Git blobs.
Cleanup errors are empty,
and successful private libvirt listings are empty before setup and after cleanup.
Daemon lifecycle records confirm requested `SIGTERM` signals;
they do not retain a subsequent PID-exit probe.
The verified cleanup boundary is the empty private domain list,
not independently observed daemon exit.

The configured original test identities are:

- `vmsync lifecycle (Linux) > full CLI lifecycle`:
  unexecuted.
- `vmsync lifecycle (Windows) > full CLI lifecycle`:
  unexecuted.

The original package task failed during top-level Linux setup,
before registering its test bodies,
with the helper working-directory defect tracked by `#1076`.
The supplementary repository-root source-suite command was never reached
because Windows template verification failed first.
Ubuntu guest preparation is a positive environment check;
it does not replace either unexecuted lifecycle test.

The [recovery evidence](../audit/depot-vm-recovery-2026-10-17.evidence.json)
records the clean pushed replay route,
unchanged source inputs,
actual normalized identities,
terminal provider usage,
and coverage status.
Final coherent pipeline replay remains a separate measurement step;
no replay was dispatched as part of this handoff.

## Clean route for final coherent replay

The initial minimal route was committed and remotely verified at
`0cb7b85ec0c70438636f15d43f021d9cf3d52b8e`
on branch `trial/depot-complete-vm-20261016`.
On 2026-10-17,
the cohort-bound route superseded that preparation at
`b2dcdc40d8ae2cfb1f7e13001db28ad8f17626b3`,
with a clean worktree and matching remote branch OID.
The standalone artifact extension superseded that route on 2026-10-17 at
`8719ad3db0167574a5a8324132fa9a0bb74fd098`,
also clean and remotely matched on the same branch.
Workflow `.depot/workflows/depot-complete-vm-original.yml`,
job `vm-original`,
runs the natural build,
`//package/dev-script/task-util:trial:depot:vm:original`,
and the separate standalone consumer.
The original-task adapter invokes only the authored source command
`mise run //package/vmsync/cli:test:e2e`.
That source task owns its existing bundle-build dependency.
The logger manifest owner corrected the old inherited `test:unit` plus explicit file argument
to the authored `test:e2e` route and its build prerequisite;
the historical old manifest remains development provenance.
The corrected manifest is in `.depot/manifests/logger.json`
on the logger harness branch,
with correction commit `0095ae37c74af1b5d720af3544876aee897be1ad`
and SHA-256 `4f9bcf0a94e09ecda8eae84c8e5ff9d89f15f0bd398df8136f0771011d3931f7`.
The current VM route carries the logger owner's subsequent corrected manifest bytes,
SHA-256 `d8cf02e46bafd03ab4f410ccf2a259a29c68c757af1cddf81735258372025af9`.
The later digest includes the logger owner's other prerequisite corrections;
its authored VM selection remains `test:e2e` with no arguments.
The recovery evidence compares its selected task,
arguments,
file identity,
and bundle-build prerequisite with the minimal VM route.

The final adapter allocates private HOME,
configuration,
cache,
data,
and runtime directories before invoking the original task.
It preserves the actual command result,
both configured test identities,
and initial/final source identity with input hashes.
It requires the known helper-resolution failure to remain a failed required check.
It does not run the supplementary Ubuntu or Windows preparation cycle.
Those environment capabilities and the Windows limitation remain separate diagnostic evidence.

The workflow retains `logger-vm-original-evidence`
from `~/temp/agent/depot-complete-vm-original-evidence/`.
Its declared allocation is 16 CPUs and 64 GiB,
with a 20-minute provider timeout and 600-second source-command bound.
The task definition parsed through mise,
its inline Node module passed syntax checking,
and its committed workflow/source-task selection matched the corrected route.
The natural build and both rootless/default-rootful standalone consumer modes passed local actual-task controls.
These checks do not establish a new provider result.

### Standalone natural-bundle consumer

Task `trial:depot:vm:build` invokes the unchanged source task
`mise run //package/vmsync/cli:build:js:node`.
It requires a successful cohort check and a matching source-before receipt,
checks source identity before and after the build,
and retains `natural-build.json` plus `vmsync/index.mjs`.
The schema-version-one receipt binds the actual build command,
exit,
timestamps,
source identity pair,
cohort/request metadata,
case-manifest digest,
source/harness revisions,
bundle bytes/SHA-256,
and source-selected Node executable bytes/SHA-256/version.
The original authored `test:e2e` still owns and executes its natural build dependency.

Task `trial:depot:vm:consume` is an additional required workflow step.
Its condition uses `always()` with successful `cohort`,
`source_before`,
and `vm_build` outcomes.
It can therefore consume the actual build after `vm_original` fails,
while the original source gate continues to fail the job.
It validates the successful natural build receipt,
all cohort/source/harness/case joins,
both source identities,
the retained artifact path and exact bytes,
and the actual source-selected Node bytes/version before creating a container.

The container receives only the copied bundle,
its build receipt,
Node,
and the ELF loader/shared libraries reported by `ldd`.
It has no image layer,
source mount,
`node_modules`,
network,
provider token,
or GitHub token.
It uses a read-only private root filesystem with a disposable `:O` overlay,
two CPUs,
2147483648 bytes of memory,
512 PIDs,
and a 60-second container bound.
The actual bundle must pass `--help` and empty `list`.
The consumer records actual cgroup limits,
memory peak and peak/limit fraction,
network interfaces,
complete command outputs/statuses,
and the build receipt/bundle hashes in `bundle-consumer.json`.

The original `MODULE_NOT_FOUND` result and both configured lifecycle identities remain separate.
Neither `vmsync lifecycle (Linux) > full CLI lifecycle`
nor `vmsync lifecycle (Windows) > full CLI lifecycle` was executed by these smoke checks.
No guest,
Windows fixture,
emulator,
or real libvirt session is used.
The source-root absolute-file probe reached its actual Linux download command
and stopped under the intentional no-network boundary;
it is not a lifecycle pass.

The [artifact evidence](../audit/depot-vm-artifact-consumer-2026-10-17.evidence.json)
retains 19 actual build/source/consumer controls,
two actual default-rootful controls,
and the 28 cohort/source regression controls at the final ref.
Only the GitHub contents response is synthetic in the artifact controls;
the natural build,
original failure,
and copied-bundle commands execute unchanged.
Both consumer modes report an absent owned container,
unchanged copied lower-tree inputs,
and removed private rootfs.
The rootless memory peak was 44994560 bytes (fraction 0.0209522),
and the default rootful peak was 44720128 bytes (fraction 0.0208244).
Provider compute and account usage remain unmeasured by these local controls.

### Standalone rootfs cleanup failure

Harness `02ea2eb5675c68677b694163966f3707eac71fb1` passed both bundle smoke commands,
then Node `rmSync` emitted `EACCES, Permission denied: '<private-rootfs>'`.
The container was already absent.
The remaining runtime directories belonged to mapped UID/GID `524288`;
`etc` had mode `0755` and its zero-byte placeholder files had mode `0700`.
The adapter had supplied a plain `--rootfs` directory,
then incorrectly assumed every resulting inode remained removable by the outer caller.
The failed record and fresh remaining-inode inventory are retained separately in the artifact evidence.

Podman 5.8.7,
commit `c593b672bf3db1173aebea565ebf1a724ea196dc`,
was inspected in the read-only clone `~/temp/agent/podman-20261017`.
Its `libpod/container_internal.go:1847` creates `etc`,
then line 1853 assigns container-root ownership when that directory was newly created:

```go
// Podman libpod/container_internal.go:1847 at c593b672bf3d.
err = unix.Mkdirat(dirfd, "etc", 0o755)
// The intervening error check is omitted from this excerpt.
if err == nil && (rootUID != 0 || rootGID != 0) {
    err = unix.Fchownat(dirfd, "etc", rootUID, rootGID, unix.AT_SYMLINK_NOFOLLOW)
}
```

The same file's overlay branch at line 1778 calls `overlay.Mount`,
and line 1809 changes the active mountpoint to the overlay:

```go
// Podman libpod/container_internal.go:1793 and line 1809 at c593b672bf3d.
overlayMount, err := overlay.Mount(contentDir, c.config.Rootfs, overlayDest,
    c.RootUID(), c.RootGID(), c.runtime.store.GraphOptions())
mountPoint = overlayMount.Source
```

The verified adapter repair uses `--rootfs <private-rootfs>:O`.
Runtime changes now belong to disposable Podman overlay storage,
and the copied lower tree remains unchanged and removable by its caller.
The tradeoff is requiring working Podman overlay storage.
The adapter inventories the actual lower tree after the container exits,
checks container absence,
and records rootfs removal;
a cleanup error still fails the consumer.
Merely using `--rm` did not remove the externally supplied plain rootfs.
Merely retaining the pre-run file list missed runtime-created inodes.

#### Upstream filing decision

1.  This was the repository adapter's cleanup assumption,
    not a demonstrated Podman defect.
2.  The adapter can select documented overlay storage and verify its cleanup.
3.  Podman's installed `podman-run` documentation supports `--rootfs :O`.
4.  External contribution review is inapplicable because no upstream report is proposed.
5.  The measured failure affected our new consumer's cleanup after successful smoke commands.
6.  Actual allowed/rejected controls and both rootless/rootful cleanup positives are retained.

All-state current-repository searches for rootfs cleanup,
Vmsync EACCES,
and mapped-UID cleanup found no matching incident.
Repository [issue 1134](https://github.com/Aquaticats/Monochromatic/issues/1134)
tracks the adapter defect and its verified repair.
No Podman upstream report or fileable draft is warranted.

### Common cohort binding

The workflow requires string inputs `cohort_id`,
`manifest_sha256`,
and `requested_at`.
The cohort identifier accepts 1 to 128 characters from `[A-Za-z0-9._-]`.
The digest is lowercase hexadecimal SHA-256 of the exact outer common manifest bytes.
The timestamp must be a valid UTC ISO date ending in `Z`;
seconds and millisecond precision are accepted without rewriting the shared input.

Task `trial:depot:vm:cohort` checks the actual harness `HEAD` against `GITHUB_SHA`,
rejects tracked harness edits and index masks,
and validates the authored VM selection in the logger case manifest.
The implementation on the VM branch is `package/dev-script/task-util/mise.toml:807`:

```js
// package/dev-script/task-util/mise.toml, VM harness branch
assert.equal(harnessRevision,process.env.GITHUB_SHA,'Actual harness HEAD must equal GITHUB_SHA');
```

It reads `doc/audit/depot-complete-cohort-${cohort_id}.json` from `main`
using GitHub's contents object response,
decodes its base64 content,
and checks the decoded bytes against the supplied digest before parsing the JSON.
The manifest must be below 1 MiB.
The decoding boundary is `package/dev-script/task-util/mise.toml:830`:

```js
// package/dev-script/task-util/mise.toml, VM harness branch
const bytes=Buffer.from(response.content,'base64');
```

The parsed manifest must match `cohortId`,
`cases.logger.sourceRevision`,
`cases.logger.baseRevision`,
and `cases.logger.manifestSha256`.
Route `routes['logger.vm']` must name case `logger`,
the original workflow path,
and the actual harness revision.
`requested_at` is retained from dispatch rather than read from the frozen manifest.
Subsequent phase guards rehash the retained outer bytes and require identical cohort/request metadata.

### Source and result receipts

The route-equivalent source guard copies the reviewed helper's configured metadata enumeration,
Git-blob comparison,
NUL path boundaries,
and rejection of all assume-unchanged/skip-worktree flags.
It hashes 650 configured inputs for the current logger selection.
Indexed exclusions remain incompatible with this immutable-source contract.
The source guard is `package/dev-script/task-util/mise.toml:884` on the VM branch:

```js
// package/dev-script/task-util/mise.toml, VM harness branch
assert(
  indexEntries.every(entry=>entry[0]!=='S'&&!/[a-z]/.test(entry[0])),
  'Immutable source index must not hide tracked inputs with assume-unchanged or skip-worktree flags',
);
```

The before record is taken before dependency preparation and preserved across repeated before checks.
The always-run after step verifies source identity even when the authored task fails.
Both identity records retain the same cohort object and harness revision.
The artifact directory contains `cohort.json`,
the exact `common-manifest.json` bytes,
`common-manifest-fetch.json` with the contents API provenance,
`logger-source-before.json`,
`logger-source-after.json`,
raw source stdout/stderr,
and `results.json`.
The results use schema version 2 and distinguish outer `cohort.manifestSha256`
from `caseManifestSha256` and historical `sourceRevision`.
They retain the authored command,
timing,
actual exit status,
both configured test identities,
source identities,
and unexercised runtime status.
The collector must join `providerJobUrl` to actual Depot run/job/attempt metadata.

### Disposable preparation controls

The [cohort preparation evidence](../audit/depot-vm-cohort-preparation-2026-10-17.evidence.json)
retains 28 actual-task controls at the pushed harness revision.
Positive before/after identities agree.
Invalid input syntax,
wrong outer bytes or case/route bindings,
changed cohort/request metadata,
wrong source,
ordinary and root Cargo edits,
and unchanged or edited index masks are rejected.
The orchestration control substitutes only the authored source subprocess,
retains its fixture failure,
and leaves both lifecycle tests unexecuted.
The contents API response is also an explicit fixture because no final common manifest is published yet.
These controls do not re-execute the natural build,
source failure,
or guest lifecycle.

The first private-HOME fixture hid the existing global Git LFS filters
and failed the source guard on hydrated media.
The retained fixture-environment control compares filtered blob identities with and without
`GIT_CONFIG_GLOBAL` pointing to the existing configuration,
using private LFS storage.
A later `git status` returned clean in both environments;
that cached null result alone does not establish equal filter behavior.
Preserving the Git configuration and isolating Mise's fixture configuration allowed the controls to pass.
The failed first output remains in the private fixture directory.
No shared historical checkout was modified.

The runnable private fixture is
`~/temp/agent/depot-vm-cohort-controls-20261017/mise.toml`.
Final dispatch,
the actual published common-manifest fetch,
the authored source/build execution,
and provider collection remain held for the coherent cohort.

## What does not work

- A long private home is appropriate for image isolation but may exceed UNIX socket address limits.
- Assuming every cloud shell inherits `XDG_RUNTIME_DIR` failed in the measured retry.
- Successful daemon startup does not verify paths created only when a guest starts.
- A missing artifact does not erase a failed provider attempt from cost accounting.
- Uploading the VM harness as a local patch repeated the injected-checkout patch failure;
  the prepared harness must be clean and remotely committed before dispatch.

## Upstream filing decision

1.  Upstream fault is not established:
    libvirt rejects a path outside the host socket representation,
    and the harness chose the path.
2.  The consumer can choose a shorter runtime path.
3.  Session daemons and their runtime locations are documented in the
    [libvirtd manual](https://libvirt.org/manpages/libvirtd.html).
4.  Contribution-policy assessment is unnecessary without a proposed upstream change.
5.  The measured impact is failed guest preparation,
    followed by successful normalized Ubuntu creation
    and a separate Windows guest-agent timeout.
    The original lifecycle test bodies remain unexecuted.
6.  The original failure is reproduced in the provider log;
    actual template boot verified the corrected socket paths.
    Actual normalized Ubuntu creation verified the primary-group workaround.
    The Windows guest-agent timeout has no established upstream cause.

A GitHub mirror issue search for `socket path too long` returned no matches.
That result is not evidence that the behavior has never been reported elsewhere.
No upstream issue or comment is warranted from this consumer configuration failure.
There is no fileable upstream draft.
