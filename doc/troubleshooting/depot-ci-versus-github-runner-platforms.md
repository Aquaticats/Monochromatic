# Depot's 2026-10-17 product documentation distinguishes CI sandboxes from native GitHub runners

## Symptom

The evaluation summary said macOS and Windows required another provider.
That conclusion was too broad.
The collected native jobs used GitHub-hosted runners;
their placement did not establish that Depot could not supply native compute.

## Root cause

The report generalized a restriction of standalone Depot CI to Depot's separate GitHub Actions runner product.
The [CI overview](https://depot.dev/docs/ci/overview) states:

> Depot CI doesn't provide sandboxes for macOS or Windows.

The [GitHub runner catalog](https://depot.dev/docs/github-actions/runner-types)
separately lists Windows Server runners and Apple Silicon macOS runners.
GitHub Actions remains the workflow orchestrator when those Depot runners supply compute.
This is a product-scope correction,
not an upstream implementation defect.
No claim about an internal control-plane code path is made.

## Verification

The current CI overview,
compatibility page,
GitHub runner overview and runner catalog were fetched on 2026-10-17.
The [correction evidence](../audit/depot-platform-product-correction-2026-10-17.evidence.json)
records the URLs and distinct claims.

The supported documentation catalog includes:

- Linux x86_64 and Arm64 sandboxes in standalone Depot CI.
- Windows Server 2022 and 2025 runners used by GitHub Actions.
- macOS 14 and 15 on M2 and macOS 26 on M4 used by GitHub Actions.

The rejected inference catalog includes:

- Applying standalone Depot CI's OS restriction to every Depot product.
- Treating our GitHub-hosted execution as a measurement of Depot's native runners.
- Treating documented runner availability as proof that this repository's consumers pass there.
- Treating Apple Silicon macOS as an equivalent Intel macOS performance baseline.

These are documentation and retained-placement checks.
Actual Depot native-runner execution is separate follow-up work.
An empty repository runner listing is not a failure proof for ephemeral runners.

## Verified correction and tradeoff

Describe the orchestrator and compute provider separately.
Depot can supply Windows and macOS compute through GitHub Actions;
standalone Depot CI does not provide those sandboxes.
The provider trial must exercise the real native consumers before assigning parity or performance credit.

The user subsequently instructed execution of that missing evaluation.
Preserve the historical source cases and original results,
and record any new runner architecture or harness binding separately.
Do not change subscriptions or inspect account-wide settings.

## What does not work

Switching a Windows or macOS label into a standalone Depot CI workflow does not establish native OS execution.
The [CI compatibility documentation](https://depot.dev/docs/ci/compatibility)
also explains that unrecognized labels become Depot Ubuntu.

The prior conclusion that the existing compute-provider split was necessary is withdrawn.
The original trial placement remains a historical fact.

## Upstream filing decision

1.  No upstream fault is established:
    the product documentation distinguishes these services.
2.  No upstream implementation change is required to correct this report.
3.  Native GitHub runner use is documented;
    repository compatibility still needs execution evidence.
4.  Contribution acceptance was not evaluated because no upstream defect is alleged.
5.  No upstream fix expectation is asserted.
6.  No upstream prototype is needed for our product-scope error.

There is nothing to file upstream.
The correction belongs in this repository's report and execution ledger.

## Current GitHub runner KVM observation contradicts deployed troubleshooting

On 2026-10-18,
the current [Depot runner troubleshooting page][gha-kvm-doc] still said:

> Depot GitHub Actions runners don't currently provide `/dev/kvm`.

Actual [GitHub workflow run 39104064329][gha-kvm-run],
Depot job `117606073359`,
reported `/dev/kvm` as a character device with read and write access on `depot-ubuntu-24.04`.
The same workflow's standard GitHub job found the device but received `EACCES`.
The valid `/dev/null` control and deliberately nonexistent path established that the device probe could
distinguish permitted,
denied,
and absent devices.

The [retained verification receipt][gha-kvm-evidence] independently checks GitHub archive digests,
extracted member hashes,
and payload identity.
It binds both observations to frozen source `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`
and harness `15ee06968996212b97111accc2013c2780ed1944`.
The conflicting documentation was fetched again while verifying the artifact.

The blanket missing-KVM inference is withdrawn for that exercised Depot runner label.
Device presence and access do not establish successful VM initialization,
guest execution,
Android acceptance,
or behavior of every runner size and architecture.
The generic SDK's separate missing-device observation remains specific to its own sandboxes.

The [2026-09-16 Metal rollout account][gha-metal-rollout] describes moving GitHub Actions jobs onto Depot Metal.
Consequently,
the July announcement's future-migration wording cannot establish October placement.
The underlying implementation cause of the documentation conflict is not proven.
Use the label-specific runtime receipt for the observed fact,
and retain documentation conflicts explicitly rather than turning the page into a universal capability ban.

No upstream message was sent.
The mismatch is documented locally while actual KVM initialization and guest verification proceed separately.

## Standard GitHub KVM access disappears after successful QEMU controls

The fifth raw diagnostic,
GitHub run `39115506319`,
passed ordinary-user `q35` KVM after a named device ACL grant,
then failed a later `none` probe with:

```text
Could not access KVM kernel module: Permission denied
qemu-system-x86_64: failed to initialize kvm: Permission denied
```

The [per-probe follow-up][github-kvm-access-run],
harness `fb143279d6adc8a2dd6f0b6c77ad584ae5a2daef`,
records the actual boundary on standard GitHub `ubuntu-24.04`.
Its 46 snapshots show that `/dev/kvm` retains inode `434`,
owner `0` and group `993`,
while the named `user:1001:rw-` entry disappears after successful QEMU execution.
Both `q35` and `none` report KVM enabled when the ACL is regranted before each probe.
Neither machine type establishes a general KVM absence.
The grant also survives an idle interval before the next successful probe.
The root identity control does not restore ordinary-user access.

QEMU `8.2.2`,
source `11aa0b1ff`,
opens the device before the KVM ioctl stage in `accel/kvm/kvm-all.c:2363`:

```text
# qemu/qemu, accel/kvm/kvm-all.c
s->fd = qemu_open_old("/dev/kvm", O_RDWR);
```

The observed failure occurs at that device-open boundary.
Installed device metadata has `seat` and `uaccess` tags,
but the ACL writer and triggering event are not yet isolated.
Stat,
ACL and access reads are sequential observations rather than an atomic snapshot.
Do not attribute the change to PAM,
udev or a private provider mechanism from temporal correlation alone.

The bounded raw workaround is a fresh named ACL grant before each QEMU invocation.
Its tradeoff is that this does not protect a libvirt operation that opens the device repeatedly.
A supplementary device-group control and the actual repository VM consumer are the next isolated experiment.
The proposed group applies only to the disposable child process;
it does not change the runner account or group database.

The first group-profile run `39124932077` passed its descriptor controls:
idle,
read-only open/close and read-write open/close all retained the named ACL.
Simple descriptor open/close alone therefore does not reproduce the QEMU-linked loss in this scope.
The supplementary child actually acquired group `993` while retaining UID/GID `1001`.
Its repository consumer then stopped at CLI import,
before any guest or KVM initialization:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package '@monochromatic-dev/caught-value-core'
```

The diagnostic-only workflow had omitted the original full VM workflow's workspace setup.
The source-backed harness correction restores `mise install node pnpm`
and `mise run prepare:pnpm:install -- --frozen-lockfile` from harness `1438ecde4b`.
The dependency omission is separate from the device ACL incident.
The failed group profile's private domains are empty and both owned daemon PIDs exited;
no passing repository guest result follows from it.

The process-discovery positive control found its actual live paused TCG process,
then found it absent after QMP quit.
The final control reports no remaining QEMU process.
This was a raw capability diagnosis with no application guest,
not a passing repository VM lifecycle result.

No upstream report was sent.
The client-open boundary is source-backed,
but responsibility for the ACL mutation is unresolved.
No public backend patch is claimed;
the directly verified regrant control is the minimal current remedy.
Standard GitHub KVM use is measured here rather than inferred from a vendor promise.
Contribution-policy and maintainer-intent conclusions await an identified component;
no upstream fault is assigned to an unidentified writer.

## Corrected actual GitHub guest lifecycle

The corrected [eighth control][github-kvm-group-replay]
at `64d6a666e63e4773ee56666148ae9c8ba371877a`
restores the original workspace prerequisites and passes the actual frozen `mvm` owner.
The child retains UID/GID `1001`,
adds actual device group `993` and makes no group-database change.
Its KVM monitor reports present and enabled.
Guest stdout contains the nonce,
the nonexistent-path stderr control returns exit `2`,
and a 51-byte host shared-directory CLI push/pull roundtrip matches
`07442f8c71b3a17811a517cd5fceea7977e97e6f4a17d6a95a7602b7dc7daae8`.
This does not independently prove guest payload visibility or guest writes.
The 2026-10-18 independent receipt review found that the harness only creates a guest directory,
then pushes and pulls the same shared-host filename.
Frozen `package/mvm/cli/src/file-transfer.ts:282` copies the input into `sharedHostPath`:

```typescript
// package/mvm/cli/src/file-transfer.ts:282
await copyFile(
  hostPath,
  sharedHostPath,
);
```

Its pull path reads that host file at `package/mvm/cli/src/file-transfer.ts:409`:

```typescript
// package/mvm/cli/src/file-transfer.ts:409
const content = await readFile(sharedHostPath,);
```

The guest execution and cleanup assertions remain valid.
The historical archive bytes remain unchanged;
guest payload read/transformation/write needs its separate consumer.

Cleanup finds no private domain,
both owned daemon PIDs exited and the error list is empty.
The [public sequence receipt][mvm-sequence-evidence]
retains all eight original ZIPs:
the native failed and successful attempts,
the paired initial standard/Depot GitHub jobs and the four standard-GitHub diagnostic jobs.
GitHub hashes match the advertised API digests;
native archives have API-size,
local SHA256 and CRC verification.

Standard GitHub,
Depot's GitHub runner and native Depot CI now each have a passing actual guest lifecycle in this cohort.
The earlier failed standard-GitHub libvirt capability check remains a failed historical observation.
It cannot establish that provider's general inability to execute this workload.
The successful group-based replay qualifies a scoped process-identity workaround;
it does not identify the ACL writer or establish that simple descriptor close caused the mutation.
No Android emulator ran in any of these cloud controls.

The final job-interval ledger totals 619 observed job seconds.
Depot allocations have USD 0.0498 nominal compute value at list rates before allowances;
the standard public-repository GitHub compute rate is zero.
The separate all-provider four-CPU equivalent proxy is USD 0.1238.
These are interval-based planning quantities,
not an invoice or measured billable VM lifetime.
Historical admission reservations remain preserved separately.

[gha-kvm-doc]: https://depot.dev/docs/github-actions/troubleshooting
[gha-kvm-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/39104064329
[gha-kvm-evidence]: ../audit/depot-github-kvm-documentation-contradiction-2026-10-18.evidence.json
[gha-metal-rollout]: https://depot.dev/blog/how-we-load-tested-depot-metal-before-scaling-up
[github-kvm-access-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/39123865841
[github-kvm-group-replay]: https://github.com/Aquaticats/Monochromatic/actions/runs/39125868595
[mvm-sequence-evidence]: ../audit/depot-mvm-kvm-2026-10-18.evidence.json
