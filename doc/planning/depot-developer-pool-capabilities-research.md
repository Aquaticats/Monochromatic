# Depot Developer platform capabilities research

Researched on 2026-10-18 for using Depot alongside GitHub and expanding CI coverage.
Developer is the intended subscription baseline.
This research does not change the subscription,
account settings,
credentials,
or production routing.
It separates current documentation,
exercised behavior,
and remaining acceptance work.
[Fetched source hashes and runnable read-only harnesses][research-evidence] retain the primary-source collection.

## Plan and independent usage pools

[Developer costs USD 20/month][pricing] and includes one user,
unlimited advertised build concurrency,
500 Docker build minutes,
2,000 Depot CI minutes,
2,000 GitHub Actions runner minutes,
25 GB shared cache and registry storage,
and 1,000,000 passing test results.
The pricing page describes a separate included bucket for each product.
Unused CI minutes must not be assumed to pay for container builds,
runner minutes,
storage,
or agents.

The [Depot Metal announcement][metal] describes infrastructure already used automatically
by Depot CI and Sandboxes at no additional cost.
It is not another pool to configure.
The announcement's 30% performance improvement and sub-second microVM startup are vendor claims,
not measurements of Monochromatic workloads or guaranteed job-admission latency.
Its description of GitHub runner and container-build migration as future work
does not establish when those separate products changed infrastructure.

- [Depot CI][ci]:
  Linux x86_64 and Arm64 sandboxes with 2 to 64 vCPUs and 8 to 256 GB memory.
  Included minutes use the 2-vCPU/8-GB size as their base.
  Overage is USD 0.00005/vCPU-second,
  equivalent to USD 0.006 per 2-vCPU minute.
  A 16-vCPU sandbox consumes eight base minutes per wall minute,
  leaving 250 wall minutes in that pool.
- [GitHub Actions runners][runners]:
  Developer supports Linux and Windows.
  Linux 2-vCPU runners cost USD 0.006/minute at multiplier 1;
  16-vCPU runners cost USD 0.048/minute at multiplier 8.
  Windows 2-vCPU runners cost USD 0.008/minute at multiplier 2.
  Windows 4-vCPU runners cost USD 0.016/minute at multiplier 4,
  so the included 2,000 runner minutes cover 500 wall minutes if used exclusively on that size.
  Windows has a different USD-per-normalized-minute rate from Linux.
  The public sources do not explain how a mixed Linux/Windows allowance is apportioned in dollars.
  Track allowance consumption as the sum of each class's wall minutes times its multiplier;
  retain the class's own cash rate separately for forecasting overage.
  The overview's flat USD 0.006/minute line is incomplete for Windows;
  the runner catalog explicitly publishes the distinct Windows rates and multipliers.
- [Remote container builds][builds]:
  Developer includes 16-vCPU/32-GB builders at 500 minutes/month;
  overage is USD 0.04/minute.
  Builders with 32 or 64 vCPUs require Startup or Business.
- [Agent sandboxes][agents]:
  USD 0.01/minute on all plans,
  with no included agent allowance.
  This product is distinct from the generic Sandbox SDK billed at the CI compute rate.
- [Shared storage][registry]:
  container layer caches,
  Depot Cache,
  and registry images share the 25 GB allowance.
  Extra storage costs USD 0.20/GB/month.
  [Depot Cache][cache] uses hourly storage snapshots averaged over the month.
- [Test results][test-results]:
  extra passing results cost USD 0.000001 each.
  Failed,
  errored,
  and skipped results are free.

The [runner plan matrix][runners] puts macOS on Startup and Business.
Developer does not provide native macOS runners even though the current Startup trial exercised them.
The [current allowance reconciliation](depot-included-allowance-reconciliation.md)
withdraws the earlier categorical macOS allowance exclusion.
Startup macOS debit and credit treatment remain unresolved.
The trial's availability is not proof of Developer eligibility.
The [native evaluation][native-evaluation] remains the evidence for actual Windows and macOS consumer behavior.

## Depot CI orchestration and coverage expansion

[Depot CI][ci] runs GitHub Actions YAML from `.depot/workflows/` using its own scheduler and compute.
It can report a GitHub check per job while GitHub retains repository hosting and its other workflows.
Copying automatically triggered workflows creates two execution paths;
deployments,
releases,
and artifact mutations would need an explicit single owner.

The [current compatibility reference][compatibility] documents job dependencies,
matrices,
`max-parallel`,
workflow and job concurrency,
timeouts,
service containers,
JavaScript/composite/Docker actions,
outputs,
and reusable workflows.
Cross-repository reusable workflows support nested calls,
inputs,
outputs,
and matrices,
using full 40-character commit SHAs or version tags.
Branch references and abbreviated SHAs are unsupported for those cross-repository calls.
Private source repositories have additional caller and GitHub App access restrictions.

Depot CI [parallel steps][parallel] can share one allocated sandbox across independent commands,
with sequential subgroups and configurable fail-fast behavior.
The branches still share that sandbox's CPU,
memory,
filesystem,
and process limits.
[Step retries][retry] reuse the same sandbox rather than resetting its state;
timeouts apply separately to each attempt,
so the attempt count affects the maximum runtime.
These are documented capabilities;
this note does not claim all corresponding repository workloads were exercised.

Potential additions to CI include package-level type/lint/test gates,
built-artifact consumers,
Linux architecture and distribution checks,
browser integrations,
and independently runnable VM or Android coverage.
These proposals need a coverage inventory and actual consumer definitions,
including intentionally skipped or unsupported cases.
A successful container command does not prove a VM guest or a native desktop interaction.

The [CI sandbox documentation][ci] explicitly advertises nested virtualization and `/dev/kvm`.
That capability belongs to Depot CI sandboxes.
It must not be inferred for Depot GitHub runners or generic SDK sandboxes from the product name.
The SDK default-image probe in this research did not expose a `/dev/kvm` character device.

Existing standalone CI evidence is stronger than device availability.
Run `vq299blkbw`,
using a prepared image on `16x64`,
created an Ubuntu KVM guest and destroyed it successfully on 2026-10-17.
The retained QEMU daemon log contains `-accel kvm`;
the preparation log records the actual guest agent ready at `04:39:59.768Z`.
The command exited zero at `04:39:59.774Z`,
and destruction exited zero at `04:40:00.167Z`.
The retained artifact was rehashed on 2026-10-18:
21,474 bytes,
SHA-256 `606cb2a627cc1dcf9bc99d4bb310e9e9ae00c05fcd4ef90b4dc951c0559663e1`.
[The existing VM recovery evidence][vm-recovery] retains the source,
harness,
artifact,
and original task boundaries;
[the platform evidence][research-evidence] retains this read-only recheck.
No additional cloud allocation was started.

The same run failed a later Windows guest-agent prerequisite.
Neither authored `vmsync` lifecycle test body executed.
Private domain cleanup was verified,
but the retained daemon termination requests lack subsequent PID-exit observations.
Those limits do not erase the actual Ubuntu KVM execution,
and its prepared image and allocation do not prove every CI profile has the same prerequisites.
The [coherent Android evaluation][coherent-report] consumed a Depot-produced APK on a local emulator.
It does not prove cloud Android execution.
That Android consumer,
the CI KVM guest,
and the generic SDK device probe are separate execution boundaries.

The compatibility reference still describes fork-triggered pull request support as planned,
and deployment environments as unsupported.
It also documents that `secrets.GITHUB_TOKEN` in Depot CI cannot authenticate to GitHub Packages.
Requesting a supported permission does not prove the GitHub App installation granted it.
These are reasons to retain distinct GitHub jobs where they serve the required trigger,
trust,
or release behavior.

## GitHub Actions runners and network controls

[Depot-managed GitHub runners][runners] preserve GitHub's workflow scheduler and job/check surfaces.
They require an organization-owned repository.
Linux x86 runners use AMD EPYC hardware,
Arm runners use Graviton4,
and Windows runners use Intel hardware.
The [runner catalog][runner-types] documents sizes and disk capacity by label.
The catalog warns that Windows runners lack Hyper-V,
and that macOS capacity is not fully elastic.
Advertised unlimited concurrency does not guarantee a particular allocation latency.

[Linux egress filtering][egress] is listed as included on the Developer pricing matrix.
It supports IP,
CIDR,
and hostname rules with explicit precedence and automatic service allowlists.
Hostnames are resolved and pinned in `/etc/hosts`.
It is incompatible with the documented Tailscale integration and does not cover Windows or macOS.
The guide also states that a root process can modify the rules.
It therefore does not establish an immutable boundary against privileged workload code.
No organization network configuration was changed in this research.

The [private-resource integration][private-resources] can connect ephemeral runners to a Tailscale network.
That needs OAuth credentials and network policy,
which remain separate configuration decisions rather than prerequisites for using ordinary CI.

## Cache mechanisms and their different trust boundaries

- [GitHub Actions cache replacement][gha-cache] automatically redirects compatible cache API operations
  on Depot-managed runners to Depot Cache.
  It is explicitly unavailable on ordinary GitHub-hosted or other self-hosted runners.
  Cache entries are repository-scoped,
  but there is no enforced branch isolation.
  Trusted and untrusted work must not be assumed to have separate namespaces merely because their branches differ.
- [Generic Depot Cache][cache] supports explicit tool integrations such as sccache,
  Bazel,
  Go,
  Gradle,
  Maven,
  Pants,
  Turborepo,
  Nx,
  and moonrepo.
  These backends can be used outside Depot compute with the relevant authentication.
  This is a distinct integration path from replacing the Actions cache API.
- [Durable CI cache disks][cache-disks] are beta and currently free during beta.
  They persist a filesystem without an archive upload/restore cycle.
  Names are global across the Depot organization,
  including repositories.
  Simultaneous mounts support reads and writes,
  but overlapping application writes are not atomic.
  The guide recommends single-writer or partitioned-write patterns.
  Public fork jobs skip mounting the disk.
  Default retention is 14 days;
  post-beta pricing is not established by the current page.
- [Remote BuildKit caches][builds] persist per project and architecture.
  They are separate from the general cache retention setting.
  Project defaults allow a 50-GB layer cache,
  which already exceeds Developer's shared 25-GB included storage.
  A project retention limit is not a guaranteed hard physical-storage ceiling:
  BuildKit can retain parent layers needed by newer layers.
- [CI snapshots][snapshots] capture a reusable filesystem into Depot Registry.
  They can preserve installed browsers,
  system packages,
  toolchains,
  and pulled container layers.
  Process environment variables do not persist,
  and checkout can delete preinstalled untracked files.
  Snapshot consumers therefore need explicit environment and source-integrity checks.
  These CI images are distinct from Business-only custom GitHub runner AMIs.

The cache overview advertises a default 14-day retention policy with no size limit.
Available general cache size settings run from 25 to 500 GB or unlimited,
and time settings are 7,
14,
or 30 days.
Included storage is an allowance,
not the default eviction limit.
No actual account retention or usage configuration was inspected.

[Container autoscaling][autoscaling] is available on all plans.
By default,
concurrent builds for one project and architecture share one builder.
Additional autoscaled builders start from clones of the main cache;
new layers on those builders are not written back to the main cache.
Temporary cache clones do not count toward storage,
but scaled compute is charged at normal rates.
Throughput,
individual build latency,
and subsequent cache warmth therefore need separate measurements.

## Remote builds, registries, and artifact consumers

[Remote builds][builds] can run from local development,
GitHub-hosted runners,
Depot-managed runners,
or Depot CI.
This lets GitHub retain orchestration while Depot supplies container compilation and persistent layer caches.
Native amd64 and Arm builders can run in parallel,
and [Bake][bake] can describe related targets and dependencies.
Build outputs can be pushed,
loaded,
exported,
or retained in the Depot Registry.
Installed CLI help confirms `--load`,
`--push`,
`--save`,
`--metadata-file`,
`--output`,
`--platform`,
`--provenance`,
and `--sbom` exist.
Their presence is not a new execution result for this repository's release artifact chain.

An allocation ledger should record both orchestration compute and remote-builder compute
when a paid runner waits for a remote build.
The separate meters do not document a waiver for idle waiting in the client job.
Download,
extraction,
runtime checks,
and provenance verification belong in the consumer result,
not just the producer's successful build status.

[Depot Registry][registry] is OCI-compatible and included on all plans.
It supports ordinary repositories as well as saved build output.
Standard CDN transfer is free.
Fast CDN costs USD 0.10/GB of outbound data from the first GB,
with no included transfer allowance.
[Pull-through caching][pull-through] is another documented option for registry dependencies;
its freshness,
upstream authentication,
and retention must be exercised before assigning it a production role.
The guide says cached mutable tags are not checked for upstream changes on a pull.
Use a version tag or digest for source identity;
the guide recommends digests when a specific upstream version matters.
Pull-through blobs without a pushed-manifest reference become eligible for cleanup
after six hours without repository access.
Pull-through configuration is chosen when creating a repository and cannot be changed later.
Upstream connections and credentials are organization-level resources,
so no mirror was created as part of this read-only inventory.

Tagged-image retention is opt-in.
Untagged manifests default to three days,
while saved-build image retention defaults to unlimited and is managed per project.
Deleting an image can take up to 31 hours to reclaim its unreferenced layers,
and shared layers remain while another image needs them.
A deletion is therefore not an immediate storage-cost reset.
CI artifact retention and its exact storage-billing relationship were not established
by the artifact CLI reference checked here.

## Test reporting, history, and timing-based splitting

[Depot test reporting][test-results] ingests JUnit XML,
retains per-test failures,
errors,
skips,
captured output,
and historical context,
and supplies dashboard and CLI views.
Analytics include recurring failures,
same-SHA pass/fail observations,
and duration statistics.
Passing results consume the monthly test-result allowance even when compute is otherwise included.

[Timing-based splitting][split-tests] uses explicit runnable candidates matched to JUnit file or class identities.
First runs fall back to file-size or deterministic weights when there is no timing history.
Multi-axis matrices need an intentional split scope;
the default flattened matrix can split across an unintended axis.
Vitest and Playwright are documented cases where filename-looking candidates need class-name timing identities.
The mechanism cannot manufacture physical test shards or missing test discovery.
The suite's authored,
discovered,
executed,
and reported identities still need to reconcile.

`depot tests split`,
`run`,
and `report` are documented only inside Depot CI or Depot-managed GitHub jobs.
The [GitHub reporting action][gha-tests] logs a warning and skips upload in unsupported runner environments.
Retaining a GitHub-hosted job therefore does not automatically add that job to Depot's result analytics.
Historical test coverage in the existing trial must not be replaced by a successful aggregate upload alone.

## Generic Sandbox SDK and remote agents

The deployed [Sandbox SDK reference][sandbox] describes a public beta,
with command streaming,
filesystem access,
explicit resources and timeouts,
and graceful stop versus immediate cancellation.
Default expiry is 120 minutes including provisioning;
the maximum lifetime is 24 hours.
An ordinary `stop` acknowledgement can precede terminal cleanup;
`blocking: true` waits,
and this research also checked the final state through a fresh lookup.

The [SDK probe evidence][sdk-evidence] records default amd64 behavior,
architecture-specific probes,
output-stream and nonzero-exit controls,
filesystem artifact round trips,
and terminal cleanup.
No repository,
model credentials,
organization secrets,
tailnet,
or Depot Cache was injected.
The selected existing auth token stayed in process and was not placed in command arguments or artifacts.

The initial amd64 command and artifact checks passed,
but an auxiliary assertion failed because the default image did not expose the assumed cgroup-v2 files.
The corrected resource query used `/proc` and reported two CPUs and 4,018,836 KiB `MemTotal`.
Both amd64 allocations reported 150 GB disk after a 100-GB request.
That observation does not establish a general disk-rounding policy.

Default Arm64 and an explicitly selected existing Depot Registry Arm64 image
also passed command,
stream,
exit-code,
and artifact checks.
Their guest `uname` reported `aarch64`,
with two online CPUs and 4,018,344 KiB `MemTotal`.
Both lacked a `/dev/kvm` character device and reported 150 GB disk in provider metadata.
All four created sandboxes were independently confirmed `finished` by ID.

An explicit `docker.io/library/alpine:3.22` runtime was rejected with Connect error code 3:
`runtime.image_ref must be a Depot registry image (registry.depot.dev); public registries are not supported`.
The installed SDK's description of an arbitrary OCI image reference did not establish public-registry support.
The [SDK README][sdk-source] explicitly documents the Depot Registry requirement.
The [runtime boundary note][sdk-troubleshooting] records the exact diagnostic and accepted controls.

The SDK is applicable to disposable agent validation and programmatic artifact consumers.
It is not evidence that every Depot CI facility exists in a raw sandbox.
The installed types expose explicit architecture,
cache/tailnet opt-outs,
scoped secret injection,
and snapshots.
Those options are separately documented or exercised;
this research did not create persistent SDK snapshots with unverified deletion/retention semantics.
The SDK README describes snapshots of the home directory and writable `/workspace`,
excluding home caches and default toolchains.
That is a different capture boundary from a full CI sandbox image.
The installed secret-injection contract says command output is not masked.

Time-filtered SDK list queries returned zero rows even with explicit terminal states,
while fresh direct lookups returned all four known sandbox IDs.
The null listing failed its positive control and cannot prove the absence of allocations.
The rejected public-image request returned no sandbox ID;
no stronger absence claim is made from those list results.
Durable create receipts and direct ID lookups are the verified cleanup path for these probes.

[Remote agents][agents] are a separate product that currently documents Claude Code sessions,
persistent files and conversation history,
resume/fork behavior,
and a 2-vCPU/4-GB default.
The [quickstart][agent-quickstart] requires Anthropic credentials or a Claude Code OAuth token.
Depot sandbox compute does not include model billing.
No remote model session or recurring agent was started.
Using this product is a capability option,
not a prerequisite for expanding CI.

## Debugging, APIs, credentials, and source hosting

[Depot CI's management interface][manage-runs] supports job/run status,
logs,
metrics,
artifacts,
retries,
reruns,
and scoped cancellation.
SSH/debug sessions and [`depot sandbox` commands][compute-cli] interact with existing compute.
The latter do not create or stop compute instances.
The [API overview][api] separates CI,
managed GitHub runners,
container builds,
Code,
and Sandbox APIs.
It also notes that core project get/update/list operations require an organization token;
the presence of a user login does not imply every API accepts that token class.

Installed Depot CLI 2.102.23 and GitHub CLI 2.102.0 were inspected through actual help invocations.
A repository-filtered `depot ci run list` returned a real completed Monochromatic run.
This confirms a scoped read path,
not a subscription or aggregate-budget read.
Cancel help exposes run,
workflow,
and job scopes;
the public CLI implementation forwards those requests to the corresponding RPCs.
It does not independently prove when billing stops after an acknowledgement.

[Depot CI OIDC][oidc] uses issuer `https://identity.depot.dev` and a Depot-specific subject claim.
GitHub runner orchestration retains GitHub's issuer.
Cloud trust policies must distinguish those identities if both execution paths need access.
Automatic secret migration is not necessary for a pool that runs only non-secret validation.

[Depot Code][code] is public beta and free during beta.
It supports standalone repositories and GitHub mirrors served from Depot infrastructure.
The deployed page says standalone repositories and CI push triggers are enabled by default for all organizations.
It documents only push events as repository-generated triggers,
and each new commit in a push can create runs.
It also supports an explicit CLI run path without pushing.
Mirroring may reduce clone overhead,
but no replication-lag or provenance-failure measurement was made here.
No Code repository or mirror was created.

[Depot Managed][managed] deploys the data plane into the customer's AWS environment on Business.
Dedicated infrastructure,
GPU compute,
custom GitHub runner AMIs,
and related enterprise capabilities
are discovery results rather than available Developer baseline capacity.
Nothing in this inventory requires adopting every discovered product.

## Budget enforcement and trial transition limits

The [pricing FAQ][pricing] documents self-imposed usage caps,
and the [usage-cap announcement][caps] describes monthly minute limits that prevent new builds from starting.
The runner troubleshooting guide similarly describes admission stopping at the configured limit.
These sources do not establish an atomic all-product dollar reservation mechanism,
a guarantee that already allocated jobs are killed at the cap,
or a cap encompassing storage,
test results,
registry transfer,
model providers,
and GitHub charges.
No configured cap or actual invoice was inspected.

Under a USD 200 all-provider ceiling,
the USD 20 base fee leaves USD 180 before other providers,
taxes,
and additional product charges.
The following are calculated single-product illustrations,
each assuming all other overage and provider costs are zero.
They are mutually incompatible maxima,
not simultaneous allocations or recommended account caps:

- 32,000 total CI base minutes including the 2,000-minute allowance.
- 5,000 total default remote-builder minutes including the 500-minute allowance.
- 32,000 total Linux base runner minutes including the 2,000-minute allowance.
- 11,750 total Windows 4-vCPU wall minutes,
  assuming the documented multiplier consumes the allowance before that runner's overage rate applies.
- 18,000 remote-agent minutes with no included agent allowance.
- 925 GB average shared storage including the 25-GB allowance.
- 181,000,000 passing results including the 1,000,000-result allowance.
- 1,800 GB Fast CDN outbound registry transfer with no included allowance.

Concurrency reservations,
maximum runtimes,
retries,
in-flight cancellation,
cache/image retention,
and the cadence of cost reconciliation are separate design inputs.
A workflow timeout bounds one allocation's runtime;
it is not a monthly spending cap.
A scheduler's concurrency limit does not account for another provider's simultaneous work.

The public trial pages advertise seven days without requiring a card.
Public client copy says billing details are required to continue builds after a trial.
The [terms][terms] permit plan changes,
warn that downgrading can remove content/features/capacity,
and say subscriptions have no partial-month or downgrade refund.
They do not establish this account's trial expiry,
billing attachment,
plan-change effective time,
or retention behavior on moving from this trial to Developer.
No subscription transition was performed.

## Evidence limits and remaining research decisions

The public `depot/docs` main checkout was at commit
`e40bfe1683ae561156d415513f2a648f89e82f12`,
dated 2026-09-11.
It contradicted deployed 2026-10-18 pages on reusable workflows,
Sandbox public-beta/runtime availability,
Code default enablement,
and permission details.
Direct HTTP 200 bodies and rendered text resolved those claims in favor of current deployed documentation.
Docs PR 188's contributor comment says documentation is authored in `depot/app` and synchronized to the public mirror.
The [source-drift note][docs-drift] records the correction and reproducible comparison.
The deployed SDK page's `imageRef` option omits the README's explicit registry-host restriction.
Its coming-soon paragraph still describes only the default image,
despite the accepted explicit Depot-image probe.
Current documentation therefore still needs consumer verification where its own sections differ.

Unresolved acceptance work is concrete:

- Complete the workload inventory against desired coverage,
  including current source fixes rather than treating frozen historical failures as current defects.
- Measure fresh-job warm and changed-source cache paths with identical consumer checks,
  including trusted versus untrusted cache admission and the cost of restore/save/export.
- Exercise required check naming,
  path filtering,
  cancellation,
  superseded commits,
  retry semantics,
  and explicit provider fallback on disposable workflows.
- Verify timing-based test identities and reporting on the repository's own test producers.
- Exercise the required authored VM gates and cloud Android workloads on the selected CI sandbox.
  Reuse the actual Ubuntu KVM guest as platform evidence,
  while retaining the unexecuted historical lifecycle bodies and local Android consumer boundaries.
- Assign retention and cleanup owners for registries,
  snapshots,
  caches,
  disks,
  and evidence artifacts.
- Resolve budget policy across products and providers,
  including headroom for allocated work and storage whose deletion is delayed.
- Confirm the intended paid-plan transition only when the user is ready to settle that account action.

Preferences about automatic versus deliberate acceleration,
required coverage,
accepted queue waits,
and tradeoffs between broad sampling and recurring gates
remain grilling topics after the actionable research and probes are reconciled.

[pricing]: https://depot.dev/pricing
[ci]: https://depot.dev/docs/ci/overview
[runners]: https://depot.dev/docs/github-actions/overview
[runner-types]: https://depot.dev/docs/github-actions/runner-types
[builds]: https://depot.dev/docs/container-builds/overview
[agents]: https://depot.dev/docs/agents/overview
[registry]: https://depot.dev/docs/registry/overview
[cache]: https://depot.dev/docs/cache/overview
[test-results]: https://depot.dev/docs/ci/observability/depot-ci-test-results
[native-evaluation]: ../audit/tech-depot-complete-ci-trial-vet-2026-10-18.md
[compatibility]: https://depot.dev/docs/ci/compatibility
[parallel]: https://depot.dev/docs/ci/how-to-guides/parallel-steps
[retry]: https://depot.dev/docs/ci/how-to-guides/retry-steps
[egress]: https://depot.dev/docs/github-actions/how-to-guides/egress-filtering
[private-resources]: https://depot.dev/docs/github-actions/how-to-guides/access-private-resources
[gha-cache]: https://depot.dev/docs/cache/integrations/github-actions
[cache-disks]: https://depot.dev/docs/ci/how-to-guides/cache-disks
[snapshots]: https://depot.dev/docs/ci/how-to-guides/custom-images
[autoscaling]: https://depot.dev/docs/container-builds/how-to-guides/autoscaling
[bake]: https://depot.dev/docs/container-builds/how-to-guides/docker-bake
[pull-through]: https://depot.dev/docs/registry/pull-through-cache
[split-tests]: https://depot.dev/docs/ci/how-to-guides/split-tests
[gha-tests]: https://depot.dev/docs/github-actions/observability/github-actions-test-results
[sandbox]: https://depot.dev/docs/api/sandbox-sdk-reference
[sdk-evidence]: ../audit/depot-developer-sandbox-sdk-2026-10-18.evidence.json
[sdk-troubleshooting]: ../troubleshooting/depot-sandbox-runtime-registry-boundary.md
[sdk-source]: https://github.com/depot/sandbox-sdk
[agent-quickstart]: https://depot.dev/docs/agents/claude-code/quickstart
[manage-runs]: https://depot.dev/docs/ci/how-to-guides/manage-workflow-runs
[compute-cli]: https://depot.dev/docs/cli/reference/compute
[api]: https://depot.dev/docs/api/overview
[oidc]: https://depot.dev/docs/ci/oidc
[code]: https://depot.dev/docs/code/overview
[managed]: https://depot.dev/docs/managed/overview
[caps]: https://depot.dev/blog/usage-caps
[terms]: https://depot.dev/terms-of-service
[docs-drift]: ../troubleshooting/depot-documentation-source-drift.md
[research-evidence]: ../audit/depot-developer-platform-research-2026-10-18.evidence.json
[vm-recovery]: ../audit/depot-vm-recovery-2026-10-17.evidence.json
[coherent-report]: ../audit/tech-depot-complete-ci-trial-vet-2026-10-17.md
[metal]: https://depot.dev/blog/announcing-depot-metal
