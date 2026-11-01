# Depot Sandbox SDK 0.1.0-beta.7: public OCI runtime rejected at create

## Symptom

A real `Sandbox.create` request with `runtime.imageRef` set to `docker.io/library/alpine:3.22`
returned `ConnectError`,
code `3`,
with this exact message:

```text
# Depot SandboxService CreateSandbox response observed on 2026-10-18
[invalid_argument] runtime.image_ref must be a Depot registry image (registry.depot.dev); public registries are not supported: docker.io/library/alpine:3.22
```

No sandbox ID was returned for that request.
Default amd64/Arm64 images and an explicitly selected existing Depot Registry Arm64 image worked.
The [evidence][evidence] preserves both the rejection and accepted controls.

## Root cause and source trace

The earlier reading of "arbitrary OCI image reference" was too broad.
At `depot/sandbox-sdk` commit `0e740b788d2613128b2ba6ed9b4129333244ee56`,
`src/types.ts:67` uses that phrase:

```ts
// depot/sandbox-sdk/src/types.ts:65 at 0e740b788d2613128b2ba6ed9b4129333244ee56
/**
 * Selects which runtime a sandbox boots into. Today the only supported option
 * is `imageRef`, an arbitrary OCI image reference.
 *
 * The `named` variant is reserved for a future catalog of curated runtimes
 * (for example `"node24"`) that the server would resolve by name. It is not
 * usable yet: the server currently rejects any `named` value, so do not set it
 * until the catalog ships.
 */
```

The same repository's `README.md:81` explicitly supplies the missing registry boundary:

```md
<!-- depot/sandbox-sdk/README.md:81 at 0e740b788d2613128b2ba6ed9b4129333244ee56 -->
A custom `runtime.imageRef` must be hosted on `registry.depot.dev` on either architecture;
any other registry fails create with `InvalidArgument`.
```

In `src/sandbox.ts:156`,
`Sandbox.create` forwards the selected runtime to the RPC:

```ts
// depot/sandbox-sdk/src/sandbox.ts:160 at 0e740b788d2613128b2ba6ed9b4129333244ee56
const response = await client.rpc.createSandbox({
  name: opts.name,
  resources: opts.resources,
  runtime: opts.runtime !== undefined ? runtimeToProto(opts.runtime) : undefined,
  // Remaining request fields are omitted from this excerpt.
})
```

The observed API diagnostic matches the README contract.
The server validation implementation is not public in the source checked here,
so this note does not claim an internal architecture makes public registries impossible.

## Verification

The official package `@depot/sandbox` 0.1.0-beta.7 was installed without lifecycle scripts
in private scratch,
and its installed declarations and implementations were read before use.
Existing CLI authentication was captured in process;
tokens were not printed,
put in command arguments,
or retained in receipts.
The evidence includes complete runnable Mise task definitions under `harnesses`,
plus the package version and dependency lock used in private scratch.
The default/Arm64 consumer probes and the direct-ID/list comparison are separate tasks,
so failed observation assumptions remain visible rather than being overwritten.

Each successful allocation requested 2 vCPUs,
4,096 MB memory,
a 100-GB disk,
a two-minute expiry,
no tailnet,
no Depot Cache,
and no injected secrets.
The provider reported 150 GB disk for all four created allocations.
[Docs PR 188][disk-note] separately notes a 150-GB server floor;
no disk-sizing implementation was inspected.

Working catalog:

- Default amd64 image:
  stdout/stderr separated correctly,
  expected exit `7` preserved,
  and UTF-8 artifact write/read/copy/stat/delete controls passed.
- Repeated default amd64 image:
  the same consumer controls passed,
  with two guest CPUs and `MemTotal` of 4,018,836 KiB.
- Default Arm64 image:
  `uname -m` returned `aarch64`,
  two guest CPUs and `MemTotal` of 4,018,344 KiB;
  command streams,
  exit status,
  and artifact round trips passed.
- Explicit Depot Registry Arm64 image:
  `library.registry.depot.dev/base:ubuntu-24.04-arm64-20261014212854`
  passed the same controls.

Rejected catalog:

- Public Docker Hub image `docker.io/library/alpine:3.22`:
  create rejected with code `3`.
- An auxiliary host-container assumption in the initial probe failed:
  `/sys/fs/cgroup/cpu.max` and `/sys/fs/cgroup/memory.max` did not exist.
  The SDK command and artifact consumers had passed;
  a subsequent `/proc` query replaced the unsupported observation assumption.

Each created sandbox was stopped with `blocking: true`,
then a fresh lookup by ID independently confirmed `finished`.
The [platform research note][research] describes the implications for CI design.

## Verified workaround

Use the default runtime when it supplies the required tools,
or select an existing image hosted on Depot Registry that matches the requested architecture.
The explicit Depot Registry Arm64 control exercised the latter path.
The tradeoff is the additional image publication,
provenance,
storage,
and retention responsibility when a custom image must first be prepared.
This investigation did not publish a replacement image or registry mirror.

Record created IDs durably and verify their terminal status directly.
That cleanup pattern worked for all four allocations.
It does not establish that an arbitrary allocation can be rediscovered by a list query.

## Separate listing and device observations

These observations are separate incidents;
no common cause with runtime rejection is asserted.

Time-filtered `Sandbox.list` queries returned zero rows,
including a query explicitly listing every lifecycle status.
Fresh `Sandbox.get` calls returned all four known IDs in `finished` state.
The null list failed its positive control,
so it was not used to prove that the rejected request allocated nothing.
The cause of the list/get discrepancy remains unestablished.

The client source at `src/sandbox.ts:193` forwards list filters without a local inventory:

```ts
// depot/sandbox-sdk/src/sandbox.ts:193 at 0e740b788d2613128b2ba6ed9b4129333244ee56
const response = await client.rpc.listSandboxes({
  pageSize: opts.pagination?.pageSize,
  pageToken: opts.pagination?.pageToken,
  filter: opts.filter !== undefined ? filterToProto(opts.filter) : undefined,
})
```

Both measured architectures lacked a `/dev/kvm` character device in this default SDK image.
That is not a finding about Depot CI's separate nested-virtualization capability,
nor proof of an architecture-wide prohibition on SDK virtualization.
No guest VM or Android emulator was launched by these SDK probes.

## What does not work

- Reading a type comment as permission to use any public registry.
- Treating a missing cgroup-v2 file as evidence that command execution or cleanup failed.
- Trusting an empty list when it cannot return known positive-control objects.
- Treating a stop acknowledgement as proof of cleanup without a terminal-state observation.
- Inferring Developer subscription eligibility from operations performed during the Startup trial.

## Upstream filing decision

The `.out-of-scope/` files were checked and contain no matching Depot exemption.
The SDK's issue listing was empty;
its open/closed PR list and README were read.
The docs snapshot PR and its contributor comment were also read.
No issue or comment was sent.

1.  Upstream fault:
    the registry rejection matches the explicit README contract.
    The list/get discrepancy is a separate observation whose cause is not established.
2.  Fixability:
    the client wording could be clarified,
    but no public server source establishes a behavioral fix for registry admission or listing.
3.  Supported use case:
    the README supports default and Depot-hosted custom images,
    exactly the successful controls.
4.  Contribution policy:
    the SDK README and available repository files showed no restriction
    on AI-assisted research;
    this is not treated as authorization to send an external message.
5.  Upstream movement:
    the SDK already documents the runtime restriction,
    and docs PR 188 already records the disk-floor discrepancy.
6.  Prototype:
    no product patch was built because the rejected path was outside its documented contract,
    and the list/get observation lacks an isolated source cause.
    The consumer controls and direct-ID cleanup were implemented and run.

Nothing fileable is proposed for the documented registry boundary.
The unresolved listing observation remains in the local research evidence rather than an unsupported root-cause report.

[evidence]: ../audit/depot-developer-sandbox-sdk-2026-10-18.evidence.json
[disk-note]: https://github.com/depot/docs/pull/188
[research]: ../planning/depot-developer-pool-capabilities-research.md
