# Depot platform surface audit

## Status and correction

Research reopened on 2026-10-18 after the user rejected the completeness of the previous investigation.
The previous claim that only the pnpm comparison remained before interviewing was incomplete and is withdrawn.
The three proposed interview questions are also withdrawn.
No preference question or production adoption follows from this document.

This audit begins with explicit surface enumeration:
deployed documentation indexes and sitemap,
installed CLI command tree,
public API and SDK schemas,
official integration repositories,
release history,
and operational and security commitments.
Each discovered surface needs an applicability and evidence entry;
fetching a page is not equivalent to resolving its implications or exercising the feature.

## Scope and source identity

Repository applicability uses frozen source `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The audit includes expanded CI needs beyond current workflows.
The Developer baseline and complementary GitHub capacity remain settled.
Account-wide usage,
invoices,
settings,
subscription changes,
broad credentials,
Code mirroring,
paid external models,
and production routing changes remain excluded.

Any additional cloud probe must have a concrete question,
immutable source,
resource bounds,
run identity,
consumer assertions,
and a cleanup plan before execution.
Retain the actual terminal and cleanup results after execution.
Existing receipts remain evidence only for their recorded source and exercised behavior.

## Enumerated primary surfaces

The deployed [LLM index][llms] contains 120 documentation entries,
257 changelog entries,
229 blog entries,
26 resources,
and 10 customer stories.
The independently retrieved [sitemap][sitemap] contains 690 URLs.
All indexed documentation pages were retrieved as current Markdown and checked for Markdown content,
rather than accepting HTTP success alone.
The [full documentation bundle][bundle] is 931,767 bytes.

The [surface ledger](depot-platform-surface-ledger.json)
records each documentation entry,
CLI help invocation,
official repository,
release entry,
and context-index entry.
The [compressed source receipt](../audit/depot-platform-surface-sources-2026-10-18.evidence.json.gz)
retains exact public responses and pinned integration source.
Its first checkpoint contained 295 source files from 28 official repositories.
The expanded checkpoint contains 542 source files from those repositories,
including the complete installed CLI implementation and additional SDK and integration source.
Retrieval state and substantive review state are separate fields.

A further source pass captures metadata for all 93 enumerated official repositories
and 142 additional root README,
license,
policy,
and dependency-manifest files.
The public `depot/docs` repository at `e40bfe1683ae561156d415513f2a648f89e82f12` contains 117 MDX pages.
Every public-source route matched a deployed page.
The deployed GitHub Actions API,
Nx Cache,
and Xcode Cache pages are absent from that public source tree.
The compressed receipt preserves this route map and the exact public MDX source.
Claim-level content differences and supplementary implementation relevance remain under review.

The installed CLI is audited against release `v2.102.23`,
source commit `1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
Visible help traversal initially missed a single-space command column;
the `configure-docker` control exposed that parser defect and the inventory was corrected.
That correction initially covered only the root.
Re-parsing every cached node found `ci migrate secrets-and-vars`
and generated completion/help descendants of the manually dispatched Claude secrets command.
The recursive parser now has single-space,
multiple-space,
and flag-section controls.
Source inspection then exposed hidden root commands and manually dispatched Claude subcommands.
The expanded inventory contains 99 help invocations,
including the failed Cargo-wrapper help invocation described in the source gaps.

No count establishes exhaustive understanding by itself.
The source and integration reconciliation remains active.

## GitHub runner KVM documentation contradiction

The current [GitHub runner troubleshooting page][gha-troubleshooting] says
"Depot GitHub Actions runners don't currently provide `/dev/kvm`".
The current runner probe contradicts that blanket restriction.
[Run 39104064329][kvm-run],
Depot job `117606073359`,
observed `/dev/kvm` as a readable and writable character device on `depot-ubuntu-24.04`.
The `/dev/null` positive control passed and the deliberately nonexistent device failed with `ENOENT`.
The standard GitHub job in the same workflow found `/dev/kvm` but could not access it,
reporting `EACCES`.
Neither observation establishes that a guest booted.

The [independently checked evidence][kvm-evidence] binds these observations to frozen repository source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`
and harness `15ee06968996212b97111accc2013c2780ed1944`.
Both downloaded archive digests matched the GitHub API digests;
both extracted member hashes and embedded observations also matched.
The conflicting deployed page was fetched again during this verification.

The [2026-07-07 Metal announcement][metal-launch] described future migration of GitHub runners.
It cannot establish their placement in October.
The [2026-09-16 load-testing account][metal-gha-rollout] explicitly describes
"moving more GitHub Actions jobs onto Depot Metal"
and says traffic was subsequently moved.
That source establishes a rollout chronology,
not complete migration of every label,
KVM guest success,
or a capacity guarantee.
Its load test deliberately stopped before allocating real workload machines.
The [2026-09-22 autoscaler account][metal-autoscaler] also describes capacity headroom,
availability-zone constraints,
and compute/storage pools rather than a queue-free service guarantee.

Generic SDK sandboxes,
standalone CI,
GitHub runner labels,
and native Windows/macOS remain separate products and execution cases.
The generic SDK's retained missing-device observation does not override this GitHub runner result.
The separate KVM initialization and actual guest consumer probes remain active work.

## Newly exposed orchestration semantics

### Local changes and reusable workflows

[Run management][manage-runs] documents local patch upload and injection after checkout.
A job that calls a reusable workflow has no steps where the CLI can inject that patch:
it runs merge-base code without local changes and emits a warning.
The existing smoke tests of `depot ci run` do not qualify that path for an agent's uncommitted current-source request.
The next orchestration probe must bind caller,
called workflow,
checkout source,
patch contents,
and consumed output identities.

Native downstream dispatch is another distinct boundary.
The built-in job `DEPOT_TOKEN` cannot start another workflow,
and the job's OIDC token cannot directly authorize dispatch.
The documented route needs a user or organization token.
No new token or dispatch secret is created by this audit.

### Retries and parallel execution

[Native step retries][retry] apply to `run:` steps,
retry every nonzero exit without an exit-code filter,
and reuse the same sandbox.
Filesystem effects and environment files accumulate.
A step's timeout applies separately to each attempt;
the job timeout still terminates the retry loop.
`retries: N` permits `N + 1` total executions.
This differs from a fresh-job retry or a classified infrastructure-only retry.

[Parallel steps][parallel] take isolated snapshots of step environment,
outputs,
and path state,
but share the workspace,
home directory,
and tool cache.
Default failure handling cancels siblings.
`fail-fast: false` collects remaining failures;
`continue-on-error` can instead absorb a failure.
Repository tasks that format files or modify shared build directories cannot be assumed safe in such a block.
Actual state-merge and cancellation probes remain research work.

### Native event and check chronology

The [2026-08-19 change][pr-continuation] says native pull-request runs continue after new commits by default.
Supersession needs an explicit concurrency group with `cancel-in-progress: true`.
The earlier GitHub worker cancellation probe did not exercise native event supersession.

The [2026-08-21 duplicate-run fix][duplicate-checks]
preserves successful checks when another duplicate run is cancelled.
The [2026-09-23 check change][check-names]
gives explicitly named matrix combinations distinct check identities.
The [rerun correction][reruns]
reruns successful dependent jobs when upstream retry results change.
These are claims about separate event and attempt paths,
not evidence that this repository has exercised them.

The [compatibility reference][compatibility] also exposes separate limits:

- Non-Depot runner labels become `depot-ubuntu-latest`.
  Copying a job labeled `windows-latest` does not establish native Windows execution.
- Fork pull-request support remains planned.
- Release,
  check-run,
  check-suite,
  status,
  and issue events other than comments are unsupported.
- Scheduled workflows run from the default branch in UTC,
  ignore the entry's timezone,
  and have no official start-time guarantee.
- Environment-scoped secret lookup does not establish GitHub environment approval protection.
- Native `GITHUB_TOKEN` is a GitHub App token and is documented unsuitable for GitHub Packages authentication.

## API and SDK completeness boundaries

### Generated CI reference differs from Markdown pages

The sitemap includes [the CI API reference][ci-api],
which the LLM documentation index omits.
The original HTML endpoint works and identifies its generated schema.
Appending `.md` redirects to the general documentation overview with HTTP 200.
That response is not CI API documentation.

The current [CI OpenAPI schema][ci-openapi] exposes 45 methods and 152 component schemas.
The [GitHub Actions schema][gha-openapi] exposes 18 methods and 65 component schemas.
Pinned CLI and public-proto sources also contain legacy versions and additional service families.
These surfaces need method-level reconciliation;
a bundled client declaration does not establish a current public endpoint guarantee.

### Telemetry and termination need precise scopes

The [GitHub Actions API guide][gha-api] documents organization-token authentication.
Job analytics default to 30 days and allow windows up to 90 days;
log queries default to one hour and allow windows up to 30 days.
Pagination tokens bind the organization and original filters.
Telemetry can lag;
`dataThrough` reports its coverage boundary.

Utilization and runner recommendations aggregate whole UTC days even for a sub-day query.
Their threshold comparison rounds percentages and has no minimum sample count.
Duration and failure counts use the requested interval instead.
These recommendations cannot replace unchanged-input workload measurement.

`TerminateGithubActionsJob` requests termination of Depot compute.
It does not cancel the GitHub workflow.
A requested termination,
an observed terminal job,
and stopped billing remain distinct facts.
No excluded account-wide analytics method is invoked here.

### Nondefault SDK runtimes need an execution environment contract

The [custom-runtime receipt](../audit/depot-sdk-custom-runtime-2026-10-18.evidence.json)
admits the exact existing `tools-r2-git256` reference using registry challenge authentication,
manifest and configuration hashes,
and the snapshot owner's permission for read-only use.
Its Depot thin configuration describes a 150 GiB logical disk;
the descriptor and layer sizes alone do not establish that value.
This is a Depot block snapshot,
not an ordinary OCI root filesystem image.

SDK sandbox `6m8d0p0pvnrqlb8` accepted that reference and reached `running`
with the requested AMD64 architecture,
2 CPUs,
4 GiB memory,
150 GiB disk,
and two-minute expiry.
The first actual command failed with
`exec: "node": executable file not found in $PATH`.
Native CI's Mise environment cannot be assumed to accompany an SDK runtime.
The cleanup request completed,
and a fresh `Get` confirmed `finished`.
This proves reference admission and terminal cleanup,
not execution of the intended Git consumer.

One replacement sandbox is admitted with the same resource and lifetime limits.
Its consumer uses the independently measured absolute Git executable path
and SDK filesystem operations.
The added compute proxy reservation is USD 0.012,
bringing the original BuildKit proxy plus both full sandbox reservations to USD 0.562584
under the USD 2 research ceiling.
That proxy uses the documented SDK two-CPU CI rate;
it does not confirm actual invoiced compute quantities,
storage charges,
included-allowance accounting,
or an aggregate account cap.
Replacement `b2f4z72r3951f5k` also accepted the same runtime and resource limits.
Its consumer failed with `FileSystemError: the operation timed out`
before the intended absolute Git command ran.
The harness attempted `stat` and a full 20,315,448-byte executable download first;
its error collector omitted the SDK error's `syscall` and `path` fields.
The retained receipt therefore cannot establish which filesystem operation timed out.
Cleanup again completed,
and a fresh `Get` confirmed `finished`.
Custom-image command execution and filesystem transfer remained unqualified by those attempts.
The prior default-image command and small-file consumers remain separate positive controls.
The next diagnostic must execute Git first,
retain operation boundaries and the SDK error fields,
and test a small file independently of executable transfer.
A third sandbox executed the absolute Git command first
and retained separate small-file operation results with ten-second filesystem RPC deadlines.
The local error-collector positive control proves that the SDK preserves operation and path fields;
the previous collector lost them.
No full executable download or fourth allocation is admitted.
Reserving the third full lifetime raises the scoped compute proxy to USD 0.574584.
Sandbox `dmdwcf07jzx2rxj` passed the primary consumer:
absolute Git reported `git version 2.56.0`,
the 15,557-byte image manifest was read,
the two named synthetic exclusions were absent,
and a 124-byte nonce artifact survived filesystem write/read/hash verification and deletion.
Both registry reads matched the admitted immutable snapshot digest.
No full executable hash was measured in this sandbox.
A fresh `Get` confirmed `finished` after cleanup.

The optional detached-command log stream hit its five-second abort.
The fixture did not retain partial stream rows and did not reach its later historical fetch;
this is not proof that detached output was absent or unretained.
One later read-only fetch targeted only that stopped sandbox,
its detached command,
and the 19.719-second creation-to-stop window.
It returned no rows;
no create,
restart,
or timeout extension occurred.
The SDK's asynchronous capture and indexing contract means this empty result does not prove absent output.
Log replay remains unqualified.
There is no fourth allocation.

The deployed SDK reference explicitly bills sandboxes per vCPU-second at the Depot CI compute rate.
It also contains a material contract contradiction:
the creation section accepts `runtime.imageRef`,
but its `Coming soon` section says custom runtimes are unavailable and all sandboxes use the default image.
The successful owned thin-snapshot consumer disproves that blanket default-only statement.
It does not qualify arbitrary public OCI images or named runtimes.
Installed SDK source directs detached consumers to `sandbox.logs.fetch()` and `sandbox.logs.stream()`,
while the deployed page describes detached retrieval as future work.
The timed-out optional probe does not settle that narrower discrepancy.

## Live plan eligibility and autoscaling boundary

The [live public pricing capture](../audit/depot-live-public-pricing-2026-10-18.evidence.json.gz)
retains every feature row's SVG,
every FAQ answer,
and each compute tab from an isolated unauthenticated browser session.
Text-only fetching omitted the availability icons and collapsed FAQ answers.
The capture did not inspect account state,
cookies,
invoices,
or subscriptions.

Developer includes Linux and Windows GitHub runners,
multi-platform builds,
Build Insights,
API access,
autoscaling,
agent sandboxes,
test reporting,
OIDC,
egress filtering,
and community support.
macOS runners require Startup or Business;
the [GitHub runner overview][github-runners] independently confirms this plan boundary.
A successful Startup-trial macOS job does not establish Developer eligibility.
The [allowance reconciliation](depot-included-allowance-reconciliation.md)
withdraws the earlier categorical macOS allowance exclusion.
Startup macOS credit and normalized-debit treatment remain unresolved;
Developer tier ineligibility is a separate documented boundary.
GPU builds,
custom regions,
dedicated infrastructure,
static outbound IPs,
PrivateLink,
and AWS deployment are Business features in this capture.
This eligibility statement is separate from ordinary CI device probes and their lack of GPU devices.
The managed GPU documentation requires customer AWS capacity and a provider-prepared custom runner AMI;
it does not expose a Developer GPU sandbox label.

The FAQ defines storage billing as the month's average combined build-cache,
Depot Cache,
and registry storage,
sampled hourly.
Developer's 25 GB is shared among those products.
The project schema's minimum `keep_gb: 25` applies per architecture,
not to total project storage or a reserved allocation.
The pricing page lists usage caps but does not document one aggregate dollar cap covering all products.
The older cap announcements separately describe build minutes and GitHub Actions minutes.
The complete current container quickstart and GitHub runner troubleshooting bodies
also describe those separate controls.
A categorical container-only interpretation is retracted;
the source does not establish one aggregate dollar cap,
current configured limits,
or termination of already running work.

The [autoscaling documentation][build-autoscaling]
and the captured `depot/core/v1/project.proto` agree that `max_concurrent_builds`
means builds per instance before adding an instance.
It does not limit the total instance count.
The audited create/update schemas expose no total-instance ceiling.
Autoscaling remains off under this campaign's maximum-builder bound;
there is no actual scale-out claim.
The executed Bake graph counts as one build per project,
regardless of its target count.

Additional builders clone the main cache but do not merge their new layers back.
Work on different builders is not deduplicated.
The docs exclude temporary clones from cache storage charges while retaining normal compute charges.
These consequences make project separation,
larger builders,
and scale-out distinct CI architecture choices.

## Registry lifecycle and export surface

[Registry v2][registry] supports arbitrary OCI artifacts,
organization subdomains,
project-saved builds,
and direct server-to-server image copying with `depot push`.
The [quickstart][registry-quickstart] documents repository creation on first push,
wildcard-scoped registry access tokens,
and the distinction between those tokens and one-hour project pull tokens.
The latter only cover project repository paths.

Standard transfer is documented free;
Fast CDN transfer costs USD 0.10 per GB without an included allowance.
Registry storage shares the plan's storage allowance with caches.
Artifacts can be as large as 50 GB.
These are documented capabilities,
not measurements of this repository's images or transfers.

Tagged retention is opt-in.
Image age is measured from push time rather than last pull;
project-saved images have their own retention setting and default to unlimited.
Manifest deletion can precede physical blob reclamation by up to 31 hours.
Deleting a shared-layer tag does not prove equivalent storage reduction.

[Pull-through caching][pull-through] has further consequences:

- Upstream credentials are organization-level configuration.
  They are not created or inspected in this audit.
- Pull-through mapping can only be chosen when creating the repository.
- A cached mutable tag is not revalidated against upstream on pull.
- Unreferenced cached blobs can become eligible for reclamation after six idle hours.
- Cached content therefore does not guarantee operation through an upstream outage.

Public registry schemas expose deletion,
retention,
token,
upstream,
and manifest-detail operations.
Their authorization and cleanup behavior still need an isolated artifact proof
before relying on those operations for resource reclamation.
No broad registry token,
mirror,
or policy change follows from source discovery.

## Security and operational commitments under review

The current [security documentation][security] distinguishes project builder caches,
repository-scoped Actions caches,
organization roles,
encrypted CI secrets,
and ephemeral GitHub runner instances.
Build permission also gives effective access to a project's builder and cache contents.
That trust boundary affects shared dependency and compiler caches.

[AI documentation][ai] says failed CI attempts receive model-generated summaries from already masked logs.
Sherlock can inspect broader authenticated settings and usage when used in the dashboard.
No authenticated Sherlock session is opened for this audit,
and no account-access restriction is bypassed through an assistant.

The [terms][terms] are effective 2025-06-27.
They distinguish general service terms from beta previews,
permit beta behavior changes,
state API abuse restrictions without a fixed numerical request budget,
and provide no general uptime warranty.
Cancellation requires contacting support;
ordinary deletion of account content can take up to 90 days,
with encrypted-backup and legal-retention exceptions.
Those statements do not substitute for per-product artifact retention or immediate erasure proof.

The [public status service][status] and its machine-readable incident feed were retrieved.
Recent records include CI start failures,
sandbox dispatch problems,
queue delays,
and US/EU build disk-I/O incidents.
Status chronology,
pagination behavior,
component identities,
and published uptime denominators remain under audit.
No availability percentage is inferred from an incomplete incident count.

## Network integrations and dependency automation

The deployed [egress-filtering contract][egress] permits Linux runner rules for IPs,
CIDRs,
and hostnames.
It documents automatic control-plane allowances,
loopback access,
deny-before-allow ordering,
and DNS results pinned in `/etc/hosts`.
These filters are incompatible with Tailscale,
and a root process can change them.
They are not an isolation boundary against privileged untrusted build code.
No organization network setting is read or changed in this audit.

The [Tailscale integration][tailscale] covers GitHub runners,
container builders,
and native CI jobs as ephemeral nodes.
It excludes Windows runners.
Connection requires tailnet ACL changes,
an OAuth client with auth-key write permission,
and organization configuration.
Disconnecting prevents future joins but leaves in-flight jobs connected until completion;
immediate disconnection requires removing those nodes in Tailscale.
The custom SDK controls explicitly disabled tailnet joining and do not exercise this integration.

[Private-resource access][private-resources] offers VPN routes as alternatives to static addressing.
VPC peering and dedicated static outbound addresses require Business deployments.
Its blanket description of default public addresses as AWS addresses must be reconciled
with the later Metal rollout before being used as an allowlist specification.
No current origin-IP inventory or successful private-network consumer is established here.

The current [Dagger integration page][dagger] explicitly marks its engine integration deprecated
and scheduled for removal.
Its described implementation creates a separate persistent-cache engine VM,
requires a Dagger Cloud token,
and charges USD 0.04 per minute in addition to runner time.
This is a documented integration with a deprecation notice,
not an executed Dagger consumer or a reason to create another paid service account.

The [Dependabot integration][dependabot] documents a GitHub self-hosted-runner setting
and automatic `depot-ubuntu-latest` routing.
It specifically requires token authentication when Dependabot jobs invoke Depot builds,
and says their OIDC authentication is unavailable.
The frozen `.github` tree has workflows but no Dependabot configuration.
No dependency-update job,
GitHub setting change,
or token provision is exercised.
This page does not establish equivalent behavior for every pull-request workflow authored by a bot.

The [account documentation][account] also exposes a role-contract discrepancy:
its Member role can manage projects,
whereas the security page describes a User role that can view projects and build,
with project management reserved to Owners.
Those differently named role descriptions must not be collapsed into a proven permission matrix.
Account and organization deletion are documented as manual support requests.
No account deletion,
member change,
or authenticated role inspection is performed.

## Test candidate and upload identities

The deployed [test-splitting guide][test-split] requires each candidate to match a JUnit `file`
or `classname` value.
`timings-type: testname` calculates each candidate's duration from individual test cases;
it does not document test names as a third candidate identity.
The guide's Go example therefore supplies package candidates while using individual-case timings.

`split-key` distinguishes independent logical suites and must agree across shards of one suite.
`report-key` distinguishes multiple report uploads within one job;
ordinary matrix jobs do not require per-shard report keys.
The deployed [reporting guide][test-results] describes report keys as upload-invocation deduplication,
while suites are derived from JUnit content.

Pinned CLI `pkg/cmd/tests/run.go` forwards the split key into candidate selection
and the report key separately into report upload.
`proto/depot/testresults/v1/test_results.proto` exposes `split_key`,
candidate identity,
and timing identity in `SplitTestsRequest`,
but no report key.
It also requires identical candidate sets across shard requests sharing the authenticated execution,
split key,
shard count,
and identity choices.
This public client source does not decide the private backend's cross-upload history lookup scope.

The [current physical Nextest cohort](../audit/depot-physical-test-shards-2026-10-18.evidence.json)
reports test-specific `testName` values
but the same `className`,
`monochromatic-git-policy-cli`.
Submitting the test names as `classname` candidates does not satisfy the documented JUnit mapping.
The parent experiment retained the passing physical partitions separately from the zero-timing result.
Its actual-binary-classname positive controls found timing data,
which qualifies the original miss as a harness identity mismatch.
Report ingestion alone did not qualify the per-test lookup.

The [source-bound identity adapter](depot-tests-identity-adapter-research.md)
subsequently completed actual run `st40hvmw90`.
The current Nextest reporter and configuration had no built-in classname-format option.
The narrow adapter preserved every raw XML byte and uploaded explicitly derived reports.
The three unchanged seeds and two fresh physical shard jobs retained 136 passing API rows,
with each derived classname equal to its unchanged exact runnable name.
Both shard jobs reported timings for all 34 candidates,
executed disjoint 17-case sets,
and passed the downstream consumer.
The [diagnosis and runnable patch](../troubleshooting/depot-nextest-test-identity.md)
retain positive and adversarial controls,
the committed guard-removal test,
source policy and cleanup.
This qualifies the selected ordinary passing single-binary shape;
it makes no complete-suite or elapsed-time improvement claim.

## Owned BuildKit campaign and exporter boundaries

The [BuildKit checkpoint receipt](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json)
binds the isolated project,
immutable fixture source,
native build results,
registry content,
and actual consumers.
The new service fixture supplies no repository performance comparison.

Reading the historical project with the existing user login was rejected:
`This endpoint requires an organization token, but a user token was provided`.
No organization token was created.
The separately authorized research project `27m5m3vjlv` was created with explicit `HARDWARE_16X32`,
five-minute build timeout,
`cachePolicy.keepGb=25`,
one-day cache retention,
and no autoscaling request.
The create response reports 26,843,545,600 retained-cache bytes for that field value.
The public schema rejects a `keepGb` value smaller than 25.
The create response proves these initial bounds without account-wide inspection.

Request `7wjrw9szj0` failed with `multiple outputs currently unsupported`.
Installed CLI source appends an image exporter for `--save`,
then rejects an exporter list containing multiple entries.
The [exporter troubleshooting record](../troubleshooting/depot-buildkit-exporters.md)
contains the deciding source and verified single-export workaround.

Two subsequent fixture failures remain separate evidence:
`xj1l6kmvbg` rejected an uppercase unresolved named-context reference,
and `xm26rbm5kt` reached Node 24.22.0 on both architectures but rejected malformed generated TypeScript.
The generator had placed a literal newline inside a JavaScript quoted string.
The previous local serialization repair was wrong and is withdrawn.
The corrected harness JSON-decodes every exec-form Node program,
checks its TypeScript and JavaScript syntax with a deliberately invalid control,
then runs the actual service under the pinned Node image before another remote build.

The pinned local service returned `/health` 200,
`/nonce` 200,
and a missing-path control 404.
Responses matched the runtime nonce,
frozen repository source,
harness commit,
and Node 24.22.0.
The container had two CPUs,
512 MiB memory,
64 processes,
no network,
and no host bind mounts.
Controlled server closure and container removal passed.

Single-export save build `fxzj58r8m2` then passed in 28.003 seconds at fixture source
`5cfc1d90f92154cf62fa95789fad9bd7deeda208`.
The exact registry index digest is
`sha256:4f2c182b14f816dfe9ab08f531a8d49378b5f76767b8f56f7c9154f05d0ff3e7`.
Independent registry reads verified the index,
its `linux/amd64` and `linux/arm64` child manifests,
their configuration hashes,
and their gzip layer media types.
SPDX and SLSA v0.2 in-toto statement subjects match the corresponding runtime-manifest digests.
Signature verification and policy enforcement remain separate surfaces.

The image-list API returns full registry references in its `tag` field.
The post-build harness initially compared that field to a bare tag and failed.
A scoped read reconciled the already successful build.
The service was not rebuilt to repair that assertion.
The CLI's intermediate `containerimage.descriptor.digest` also differs from its final
`containerimage.digest` for this multi-platform result.
The final digest was independently checked against actual registry bytes.

Anonymous manifest access returned 401.
The temporary pull token used the observed Basic authentication challenge and successfully read the image.
Cross-origin signed blob redirects were followed without forwarding the Authorization header.
The token value is excluded from committed evidence.
The CLI reference describes a build-ID form while the registry overview describes project-path read access.
The token's successful read of the subsequently created CI image establishes cross-build project access.
Cross-project isolation remains unexercised.

Podman 5.8.7 pulled the immutable remote image and ran its unmodified entrypoint and command.
The health,
nonce,
and missing-path HTTP assertions passed,
and controlled stop exited zero.
However,
`podman healthcheck run` returned 125 with `has no defined healthcheck`,
even though the raw OCI configuration contains the Docker `Healthcheck` extension.
The aggregate consumer remains failed.
The [pinned Podman source diagnosis](../troubleshooting/podman-oci-healthchecks.md)
now explains the omitted metadata;
the parent-run Docker consumer honored the same original image's check.

The expired initial eight-minute wall-clock window is retained.
The authorized recovery used a fresh five-minute request guard,
the same builder sizes,
and the same USD 1 timestamp-compute ceiling.
That model is not proof of an invoice cap or aggregate account expenditure.
The shared images and project were held for independent native CI and GitHub consumers.
The parent's final consumer cohort completed and released that hold on 2026-10-18.
The builder project remains solely for the remaining bounded source/lifecycle probes,
followed by exact owned-resource cleanup.

Build `d0xtstnptt` exercised a multi-node local filesystem exporter with the base stage forced to execute.
The exported x64 identity reports 16 CPUs,
32,997,220,352 memory bytes,
and AMD EPYC 9R45.
The ARM identity reports 16 CPUs,
33,020,604,416 memory bytes,
and Neoverse-V2.
Both report Node 24.22.0 and the exact committed harness and frozen repository source.
This is actual builder geometry and local-export evidence.

Build `31w6c0wlvk` produced a native single-architecture OCI archive without `--save`.
Skopeo separately copied the saved multi-architecture registry image into an OCI archive with all digests preserved.
Archive consumers verified every reachable blob's size and digest,
the runtime program,
the healthcheck configuration,
and platform identities.
Native OCI statements contain empty `subject` arrays;
their descriptor annotations associate the attestation manifest with the AMD64 runtime digest.
The registry-derived archive retains SPDX and provenance statements whose subjects match both runtime digests.
These representations are not equivalent for consumers that require in-toto subject fields.
The deciding exporter source and consequences remain under research.

## Derived CI image consumer control

A separate campaign built an ordinary OCI job image from the exact owned service index.
The original service image stays unchanged and retained for active consumers.
Parent-run GitHub job `117622633314` in workflow run `39109462308` retained the service's healthcheck,
reached healthy state,
then printed checkout's GitHub REST download fallback.
The [later direct original-image probe](../audit/depot-original-service-git-2026-10-18.evidence.json)
confirmed that the immutable AMD64 image could not resolve `git --version`:
Node reported `spawnSync git ENOENT`,
and every default `PATH` candidate was absent.
The valid Node command and deliberately absent command controls passed.
The mount-free,
network-disabled local container used 2 CPUs and 2 GiB and was removed.
This proves an image prerequisite gap on that default path.
The historical checkout fallback alone does not identify every initialization failure,
and this local control does not observe the historical job environment or the original ARM image.
The job reached its five-minute guard during archive-download retries.
This differs from the local Podman healthcheck loss and does not establish a shared runtime limitation.

Derived build `6hb1g0j21b` passed with AMD64 and ARM64 outputs at source
`0b471538cd549cb2181088803c1e0a6d542d791e`.
Its distinct immutable index digest is
`sha256:aab45816f04afec0aed6ccd0daea92e6288a4a35c65368438678bad30b57f346`.
Actual local build and runtime Git controls passed before remote allocation.
The derived image adds Git 2.39.5,
curl 7.88.1,
CA certificates,
GNU tar 1.34,
gzip 1.12,
xz 5.4.1,
and zstd 1.5.4.
Git meets checkout's minimum;
Git-policy tests still require their separate Git 2.56 toolchain.
Both derived configurations use `CMD ["bash"]` and have no active service healthcheck.
`/opt/depot-research/image-tools.json` records each platform's actual tools,
Debian package versions,
immutable base,
and derived source identity.
The audited action inputs are `jdx/mise-action@c2a87611a18de5b3828c5652fe268e992400cb5c`,
Mise `2026.10.9`,
`install: false`,
and `cache: false`.
Checkout is pinned to `de0fac2e4500dabe0009e67214ff5f5447ce83dd`.
The consumer harness source is `1438ecde4b643a606e0a75f85067e3d0153e9bb5`.
The local control caught the original service's inherited `BUILD_HARNESS_SHA` environment value
overriding a derived argument with the same name.
The corrected fixture uses a distinct `CI_IMAGE_HARNESS_SHA` argument to identify the derived build.
The original service's environment remains inherited and separately identified.

Independent registry reads verified both derived configurations and their SPDX/provenance subject bindings.
The token issued for original build `fxzj58r8m2` before the derived build existed
successfully read the derived index,
configurations,
attestations,
and final filesystem layers.
This proves cross-build read access inside the same project;
cross-project isolation remains unexercised.
After positive authenticated reads,
an owned empty blob-upload initiation returned HTTP 403 with registry error `DENIED`.
No upload was created.

That checkpoint preceded the Node26 correction and the completed Cargo/forwarding controls.
The timestamp-compute model covers 139,103 milliseconds across native requests,
giving USD 0.1854707 at the declared two-builder rate.
This is a request-window estimate,
not billed usage or an aggregate invoice guarantee.
No account-wide billing or usage query is authorized or required by this campaign.

## Repository Node26 image prerequisite correction

The subsequent cloud job-image consumers reached checkout,
then the frozen Node `26.12.0` loader failed with a missing `libatomic.so.1`.
The base Node `24.22.0` check did not exercise the repository's different runtime.
The failed CI image remains retained.
A further owned derivative adds Debian `libatomic1`
and executes both exact architecture archives from the frozen root `mise.lock` during its build.
The archive hashes,
ELF machine identifiers,
dynamic-library resolution,
TypeScript execution,
and runtime versions must pass before that derivative receives consumer credit.
The second Node installation is removed within the same build step.

The isolated source `7590bcde15e5ceef4dc36f4e2c3d975e1c1b833f` contains the executed correction.
Build `1ng4qs1pmq` completed in 31,529 milliseconds and produced immutable index
`sha256:853b99050d5420698a02d7cb10978be8824dc0999c9ace08ec453706da54ec31`.
The actual Node `26.12.0` module executed on both AMD64 and Arm64 builders
after SHA256 verification against the frozen lock.
The probes exercised TypeScript,
crypto,
and top-level `await`,
and verified ELF machine 62 or 183 and resolved `libatomic.so.1`.
Both architectures report Debian `libatomic1` version `12.2.0-14+deb12u1`.
Base Node remains `24.22.0`.
The temporary repository Node installation was removed in the same step
and does not appear in the final filesystem layer.

Independent registry reads verified the corrected configurations and their attestation subject bindings.
The original-build token read the new index and both final layers;
an empty blob-upload initiation again returned HTTP 403 `DENIED`.
The corrected `/opt/depot-research/image-tools.json` hashes are
`53d2d2c340ef8214c8b9caf2da0b94c38327c63655d4656729a3b54c5d1e7f96` for AMD64
and `1229b983f4c529867d6a0fb1d0b1f1c1ba00f8b941c04d599ecd3d6845f3afa2` for Arm64.
They contain 4,154 and 4,179 bytes respectively.
The [BuildKit receipt](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json)
records this derivative separately from the retained missing-library image.
The parent owns the subsequent full checkout and Mise consumers.

The same research branch now includes the frozen firstparty `truepeak-core` Cargo cache-mount fixture
from source `7f22f25adb`.
Its mount controls distinguish an empty offline failure,
an online seed,
offline target reuse,
offline registry reuse with a fresh target,
and changed-flag invalidation.
Remote build `lskskmk4hn` exercised the empty-cache offline negative control.
Cargo returned its expected exit 101 in 27 milliseconds,
with no compiler artifacts and absent registry and target markers.
The outer guarded build passed in 19,670 milliseconds.
The coherent continuation used the same harness and frozen crate for every case:

- Seed `12139n3wbg`:
  12 Cargo compiler-artifact records,
  none fresh;
  Cargo 3,120 milliseconds and outer build 17,147 milliseconds.
- Offline warm `x33zr9szff`:
  all 12 records fresh;
  Cargo 376 milliseconds and outer build 13,781 milliseconds.
- Offline target reset `x98z3btb21`:
  all 12 records recompiled using the retained registry cache;
  Cargo 3,005 milliseconds and outer build 16,755 milliseconds.
- Offline changed flags `71x7kn2wqr`:
  all 12 records recompiled using the original target mount;
  Cargo 3,096 milliseconds and outer build 18,193 milliseconds.

Each case consumed a new run nonce,
so the warm result executed Cargo rather than returning a cached Docker layer.
The mount markers prove the registry/target reuse and the separate empty target.
The lock hash stayed unchanged.
These are Cargo artifact records,
not the prior sccache request counter.
This cohort proves the narrow firstparty compilation/cache mechanism;
it does not run nextest or the optional service/fingerprint graph.
There is only one sample per case,
so the recorded durations do not establish a speedup ratio or run-to-run spread.
The actual local prerequisite control started pinned Node `24.22.0`,
Rust `1.97.1`,
and Cargo `1.97.1`
with the frozen lock hash under a two-CPU,
two-GiB,
network-disabled runtime.

The parent-authorized cumulative timestamp-compute model ceiling is now USD 2.
Its fresh forecast includes the seven observed request intervals,
the Cargo cohort,
the forwarding graph,
and the immutable remote-Git controls.
The owned project remains `HARDWARE_16X32`,
with at most two architecture builders and a five-minute server build timeout.
This model does not inspect or constrain unseen account charges.

## Current Cargo wrapper and historical feature chronology

The installed root CLI exposes `depot cargo`.
Its help invocation entered the command and failed because sccache was absent;
the command-tree inventory retains that failure.
The [current-source diagnosis](../troubleshooting/depot-cargo-cache-wrapper.md)
and real isolated CLI controls establish a local Cargo/sccache wrapper,
including its automatic job-token fallback,
credential precedence,
argument forwarding,
and removal of inherited `SCCACHE_*` tuning.
It is not a separate remote Rust execution pool.

The full captured changelog also exposes Cache Explorer downloads,
Dockerfile Hadolint/Semgrep integration,
builder CNI isolation,
separate historical build-minute and GitHub-minute caps,
and Datadog's transition to general availability.
Their applicability and deciding-current-source reconciliation are active work.
The secret/SSH and multi-target Bake controls are now exercised in the forwarding campaign.
Immutable remote Git contexts are exercised in the source-selection controls.

## Actual immutable Git and local-overlay controls

The same committed Bake source,
`b0e3247a279507c759964162ec9eb816e3dcc402`,
was used for a local context and a remote Git context pinned to that full commit and fixture subdirectory.
The local filesystem contained an invented overlay marker during both requests.
Build `4tlgnm8sgp` exported that local marker and its exact content hash.
Build `m62041tlnr` exported the committed marker and committed content hash instead.
The fixture was restored byte-for-byte after the controls.
Both actual consumers passed.
This qualifies source selection for the tested immutable Git build context;
it does not qualify loading Bake's HCL graph itself from a remote repository.

## Dockerfile lint qualification and current gaps

The installed CLI's `--lint` integration injects the Dockerfile into scanner images.
Its source configures Hadolint `2.12.0`
and Semgrep `1.34.1`
with the dynamically fetched `p/dockerfile` rulepack.
It does not scan the repository's TypeScript or Rust sources.
Build `h6vjhtznxb` rejected the explicit-root fixture with `DL3002`;
build `5l1vftdxsk` accepted the non-root fixture and exported the exact bound file.

The Semgrep-specific missing-user fixture did not qualify:
build `pjlxhhkvxh` accepted `Dockerfile.semgrep`,
and canonical-filename control `1ktx2jdfqg` also accepted the same JSON-form `CMD`
without the expected finding.
Both results remain failed positive controls in the receipt,
rather than being counted as security coverage.
The source-configured tags were independently resolved for local diagnosis;
the native lint log does not report the runtime scanner digest.

A mount-free local container using the resolved Semgrep image reproduced the canonical non-detection.
Its positive single-pattern controls matched `FROM $BASE` and `CMD ...`,
while the current rule's `CMD $...VARS` pattern did not match the same file.
The scanned file's bytes were independently checked inside that container.
The pinned engine's source represents JSON argv and shell commands differently.
A separate current-scanner control now detects the same missing-user input with the exact fixed rule pack:
official Semgrep `1.181.0`,
image digest `sha256:81474464c1f07483ba0682a44787fd93c91397419762a04abe2bafaae396304b`,
reports one finding and no errors.
The `CMD $...VARS` control also changes from no match to one match,
while `FROM $BASE` matches in both versions.
The current image resolves `semgrep` through its configured `PATH`;
the old integration's absolute `/usr/local/bin/semgrep` path fails with exit 127 in that image.
That fixture failure is retained separately from the corrected consumer.
Updating only the image version is therefore not a qualified integration fix.
The precise upstream engine change and the complete Depot integration upgrade remain unqualified.
Separately,
the suffix filename was excluded from Semgrep's scanned paths.
The CLI's source also discards scanner process-wait errors and silently skips malformed JSON output.
Those source error-handling boundaries are distinct from the reproduced pattern non-detection.

All native cases and bounded local controls are retained in the
[BuildKit evidence](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json)
and its compressed source archive.
The local scanner containers used two CPUs,
two GiB of memory,
and no host mounts;
their terminal absence was checked.

## Owned OCI retention before cleanup

The independent native CI and GitHub image consumers are terminal;
their [service and job-container evidence](../audit/depot-service-containers-2026-10-18.evidence.json)
preserves the prerequisite failures and corrected consumers.
The parent released the image/project hold after removing and checking its exact owned secrets.

The original service,
Git-capable derivative,
and Node-26-qualified derivative were then copied into a private OCI layout with
`skopeo copy --all --preserve-digests`.
All three original index digests were retained.
Every reachable descriptor size and SHA256 was checked:
57 unique blobs totaling 260,363,228 bytes.
The complete source and output graphs were preserved before deleting the solely owned registry tags and project.
All six exact owned tags were removed,
then authenticated requests for each original index returned `404`.
The explicit project-deletion request succeeded;
the same digest requests again returned `404`.
The receipt retains the initial collector errors without repeating tag deletion.
No physical-erasure or unseen-storage-billing conclusion follows from logical deletion.

An empty installed-CLI `depot image list --output json` result is plain `No images found` with exit status zero.
The source returns that message before selecting its JSON formatter.
The JSON-only cleanup collector therefore stopped after its successful tag deletion.
Recovery checked the retained operation and source-defined empty result,
then continued with the remaining project deletion.
The failed preflight used the opaque pull token as a raw Bearer credential;
the already captured registry challenge requires Basic authentication with username `x-token`.
That caller defect did not indicate token expiry or revocation.

## Actual Bake graph, mount lifetime and CNI controls

Build `tf3tp8b4tx`,
source `b0e3247a279507c759964162ec9eb816e3dcc402`,
ran the reviewed `forwarding-graph` Bake group rather than only printing its configuration.
Its source and named-context evidence targets executed on AMD64 and Arm64.
Both architectures consumed the expected synthetic secret hash and forwarded SSH-agent identity.
The following `RUN` had no secret file and no `SSH_AUTH_SOCK`.
The owned keyless agent answered only identity-list requests and closed without errors.
No private key or SSH signing operation was used;
authenticated Git-over-SSH is outside this result.

Separate terminal controls rejected the missing required mounts:

- Build `c92mvpvs0t`:
  `secret fixture: not found`.
- Build `mqpxcrcbgn`:
  `unset ssh forward key fixture`.
  The initial assertion expected different wording;
  the retained terminal log was reconciled without another allocation.

The graph's independent AMD64 network targets each bound loopback port `38429`.
Their successful bind intervals overlapped by 20,008 milliseconds,
their hashed kernel boot identities matched,
and their network-namespace identities differed.
A local same-namespace collision first produced `EADDRINUSE`.
This is actual concurrent build-container network-isolation evidence.
It does not establish separate hardware per target,
host-escape resistance,
or cross-organization cache isolation.

The graph finished in 34,293 milliseconds.
The generated programs passed an annotated-input syntax preflight,
with the [Node check-path correction](../troubleshooting/node-eval-module-detection.md) retained separately.
The positive and rejected-mount receipts are in the BuildKit evidence archive.

## Agents, Code, and customer AWS are distinct optional products

The complete deployed [Agents overview][agents-overview]
and [Claude quickstart][agents-quickstart] describe Claude Code sessions,
persistent filesystem and conversation state,
resume,
and fork.
Default remote agents use 2 CPUs and 4 GB,
cost USD 0.01 per minute on all plans,
and have no included agent usage.
The overview describes automatic build/cache integration as planned.
That wording does not qualify an already working integration.
The quickstart requires Anthropic authentication and organization-scoped agent secrets.
No model session or agent secret was created by this research.

Installed CLI source `pkg/cmd/claude/claude.go`
implements `--local` by running the local Claude executable,
watching its conversation JSONL,
and uploading that JSONL continuously and at exit.
Its authentication preflight calls `ListSessions` for the organization.
Local execution therefore does not mean conversation data stays local.
`pkg/cmd/claude/remote.go` also enumerates agent sandboxes when resuming;
its initialization wait timeout explicitly leaves the remote sandbox running.
These are source-reviewed data and lifecycle boundaries,
not exercised model or account-wide session operations.
The root help control exits through local Claude help before that authentication path.

The [Code overview][code-overview] describes a free public beta with standalone repositories and GitHub mirrors.
Its CI event support is push-only,
with each newly pushed commit eligible to trigger a run.
The [Code quickstart][code-quickstart] says GitHub remains the source of truth for mirrors
and pushes through Depot are forwarded to GitHub.
It describes upstream import when the mirror refreshes,
without establishing refresh cadence.
The overview still contains author TODOs for reliability,
scale,
and mirror behavior;
those are not measured guarantees.
The quickstart also leaves exact token scope as an author TODO.
No Code repository,
mirror,
token,
or connection was created.
Potential Git transport benefits remain unmeasured.

The retained public schema `proto/depot/code/v1beta1/code.proto`
exposes standalone/mirror `CreateRepository`
and `DeleteRepository`.
The deletion comment promises repository refs become unreachable
and the operation cannot be undone.
It does not promise immediate physical object erasure.
The Code product needs its own reference,
credential,
event,
and retention qualification before becoming a CI origin.
Existing GitHub-backed CI receipts do not qualify those different contracts.

The [Managed overview][managed-overview]
and [AWS setup][managed-aws] describe a Business-only,
customer-AWS product using a dedicated empty subaccount.
The published bootstrap creates two roles with `AdministratorAccess`
and `DeletionPolicy: Retain`.
Deleting that example stack does not promise deleting its retained roles.
PrivateLink,
VPC,
IAM,
KMS,
S3,
quota,
and provider delegation are separate operating prerequisites.
No AWS resource or account setting was changed.
The [Managed GPU guide][managed-gpu] requires customer GPU capacity,
a provider-built AMI,
and a unique runner label.
That capability is distinct from hosted Business GPU container builders,
the Developer baseline,
and the retained standard CI device observations.

These optional products have concrete repository applicability and authorization boundaries in the ledger.
Reviewing them does not require adopting them or manufacturing unrelated workloads.
That checkpoint still had generic page/command entries.
The complete deployed-page reconciliation now replaces those retrieval-only states in the ledger.

## Selected repository owners actually execute on Linux ARM64 CI

The [ARM64 functional profile](depot-arm64-workload-research.md)
and [original-archive receipt](../audit/depot-arm64-functional-2026-10-18.evidence.json)
close a different boundary from ARM image hello-world execution.
Native run `p4xj5dsh3s` passed the frozen all-feature release Truepeak owner
with 89 passing and 2 deliberately skipped tests.
Every JUnit classname,
name,
and status tuple matched the 91 AMD64 identities.
The built `p-limit-fork` unit owner passed the same 11 runtime describe groups as its AMD64 comparison.
Node and the resolved Rust command entrypoint had ARM ELF machine `183`,
and the compiler reported the expected nightly commit and `aarch64-unknown-linux-gnu` host.
The original ZIP,
source hashes,
source-bound fixture,
and comparison inputs are retained.
This is one functional profile,
without whole-repository coverage,
unchanged timing repetitions,
or cache-benefit credit.

## Unreleased CLI changes expose additional research cases

The open [CLI upgrade pull request][cli-upgrade] documents differences from the installed release:
remote Bake handling,
source-policy enforcement,
metadata merging,
combined load and output behavior,
context isolation,
and Docker driver cleanup selection.
These are upstream author's claims until reconciled with deciding source and applicable controls.
An unmerged change receives no deployed capability credit.
The installed Docker configuration uninstall path will not be exercised against shared state.

## Complete deployed-page contract reconciliation

The complete current bodies of all 120 indexed documentation pages have a substantive applicability review.
The ledger retains 119 explicit `sourceReview` records
and the separate split-tests candidate-identity/physical-adapter review.
No generic retrieval-only documentation state remains.
The parent independently reviewed eighteen native CI pages and eleven Cache integration pages;
their exact source hashes,
findings,
and repository applicability are joined in durable source archives.

The CLI applicability map covers 99 distinct actual help/parser invocations.
Ninety-eight returned help successfully;
the Cargo wrapper instead rejected its missing local `sccache` prerequisite.
The API map covers 31 captured protobuf service declarations,
208 method declarations,
and 63 deployed CI/GHA OpenAPI operations.
Declarations duplicated across source packages remain separate source evidence,
not a count of uniquely deployed endpoints.
These maps classify repository uses,
token/settings restrictions,
and exact runtime limits.
They do not claim every method,
example,
format,
or optional integration was executed.

The complete container-build pass records soft cache GC limits,
cache-parent retention,
autoscaling's per-instance threshold,
Compose/Bake/default-Docker distinctions,
public log sharing,
and installed exporter restrictions.
The complete GitHub runner pass records deprecated Dagger support,
Dependabot's separate credential/settings route,
Linux-only root-modifiable egress rules,
Mac fixed-capacity queues,
and the difference between CLI analytics and dashboard flaky-test cards.
Source-only optional products do not receive runtime acceptance credit.

Current runner-type pricing explicitly lists Linux two-CPU at `USD 0.006/minute`,
Linux four-CPU at `USD 0.012/minute`,
Windows four-CPU at `USD 0.016/minute`,
and Mac eight-CPU at `USD 0.08/minute`.
Windows/Linux allowance multipliers differ from their cash rates;
Mac debit/credit mechanics remain unresolved.
No silent universal normalized-minute cash price is used.

The [native automatic-authentication follow-up](depot-native-buildkit-auth-research.md)
now separately qualifies a local exporter
and an image-only project-registry consumer using only injected native job credentials.
All three source-bound native runs and build requests are terminal,
all three original ZIPs are retained,
and the solely owned image/project deletion has fresh absence controls.

## Working-report allocation completeness

The narrow completeness review compares the working report captured at
SHA-256 `ae19f1ece34860bf483bf54b2a713ee314a1cf340ccbdab1c97d9018e9a08970`
with all 120 substantively reviewed deployed documentation pages and the linked actual consumers.
It is not a whole-report sufficiency sign-off.
The [ledger](depot-platform-surface-ledger.json) records the evidence-linked gaps and paste-ready paragraphs
under `reportAllocationReview`,
mapped to the existing six report sections.
No report edits,
cloud operations,
service activation or settings changes are part of this review.

The material omissions are allocation and operating dispositions:

- Native event/check ownership,
  protected-environment and reusable-workflow compatibility,
  retry persistence and parallel shared state.
- Tests report producers and identity lineage,
  exact attempts,
  source-only analytics,
  temporary SSH,
  logs and resource metrics.
- Native thin snapshots,
  ordinary OCI job images,
  immutable dependency/target archives,
  organization-global Disk and custom SDK runtimes as distinct formats and trust boundaries.
- Actual BuildKit contexts and immutable source selection,
  attestations versus signature trust,
  bounded autoscaling,
  registry promotion,
  pull-through,
  CDN transfer and retention/deletion.
- Trusted versus untrusted cache access,
  the measured lack of pnpm-store benefit,
  Linux-only root-modifiable egress and the separate Dependabot credential route.
- Controlled workload admission under the USD 200 ceiling and the account-access restriction,
  with retention/export and unknown external charges kept explicit.
- Source-only Managed/customer AWS,
  GPU/custom GHA images,
  Code/mirrors,
  Agents/model credentials,
  deprecated Dagger and absent-owner integrations.

The priority is trust,
budget and lifecycle conditions,
then concrete product-to-owner allocation,
then explicit optional-product dispositions.
The first group prevents unreviewed authority or spend;
the second turns measured consumers into a usable proposal;
the third prevents research coverage from being mistaken for product activation.
The repository's Codex fixture does not qualify Depot Agents,
and successful custom SDK thin-image execution does not qualify every OCI or named runtime.

Generic ledger tasks that still described all-page and CLI reconciliation as unfinished
are preserved under `supersededOpenGaps` and replaced with the current factual limits.
This does not mark every marketing/context page,
private backend method or optional integration as exercised.
Independent whole-report review remains queued for the separately owned operating-system and semantic results.

## Follow-up qualification and upstream diagnostic work

- Preserve the source/deployment distinction for the mapped schemas,
  bundled CLI protobuf versions,
  SDK exports,
  and unexercised methods.
- Reconcile the executed BuildKit graph,
  Git contexts,
  mounts,
  exports,
  and image consumers against each documented feature.
  Preserve the native OCI empty attestation subject-array distinction,
  unexecuted autoscaling,
  and lack of measured lazy-pull startup improvement.
- Preserve the built-in Dockerfile lint qualification boundary:
  Hadolint has a working positive control,
  but the configured old Semgrep image did not match the selected missing-user control.
  The separate current pinned scanner detects the same control.
  The [source and diagnostic record](../troubleshooting/depot-builtin-semgrep-engine.md)
  now includes a source-derived invocation candidate and 14 local controls.
  This does not count as a compiled or repaired complete BuildKit integration.
- The [empty image-catalog JSON diagnostic](../troubleshooting/depot-image-list-empty-output.md)
  now includes the minimal upstream source patch and 14 passing actual-command controls.
  The contribution gates and local draft are recorded;
  the installed CLI is unchanged and no external issue or PR was sent.
- Keep successful service/job-container receipts separate from the
  [parent's orchestration evidence](../audit/depot-native-orchestration-2026-10-18.evidence.json)
  and [isolated job-policy/history controls](../audit/depot-job-policy-history-2026-10-18.evidence.json).
  Content-bound first and full-rerun artifact consumers passed.
  Native job-level continued failure preserved downstream success but left aggregate/check-suite failure;
  the paired GitHub workflow succeeded.
  An isolated step-level continuation completed the native workflow successfully.
  The earlier timeout-confounded observation does not supersede those isolated controls.
  Retain incident causality limits and timeout/process boundaries separately.
- The Cargo wrapper checks for local `sccache` before providing delegated help.
  This machine's help invocation failed with `sccache not found in PATH`.
  That is a local prerequisite failure,
  not evidence against the separately exercised cloud compiler-cache route.
- Reconcile security statements with later infrastructure changes,
  plan eligibility,
  artifact deletion,
  exports,
  and support commitments.
- Reconcile SDK custom thin-image success and default-image lifecycle controls
  with the conflicting deployed reference,
  separate file-snapshot semantics,
  secret authorization,
  retained-log gaps,
  and unexercised ordinary OCI or named runtimes.
- Keep source-only network,
  Code,
  Agents,
  managed AWS,
  external CI integrations,
  and absent language owners distinct from the recorded actual consumers.
- Finish public status-history pagination and component coverage before interpreting reliability;
  no uptime denominator is currently established.
- Reconcile the completed identity-adapter receipt into the complete test-producer applicability map.
  Its selected passing single-binary report shape does not qualify arbitrary JUnit or complete-suite reporting.
- Reconcile the completed [ARM64 functional profile](depot-arm64-workload-research.md)
  into each relevant source-owned workload entry.
  It qualifies selected functional owners,
  with one 8-CPU job and no performance ratio.

The source/evidence ledger distinguishes discovered,
retrieved,
source-audited,
exercised,
blocked,
and unresolved surfaces.
No completion or interview-readiness conclusion is made.

## Native SSH acceptance proposal

A local reseal was interrupted when its read-only lookup adapter treated a JSON `null` as an array.
The adapter now accepts the CLI's observed no-match `null`,
and actual exact-source/repository positive and negative lookup controls pass.
The source archive and public metadata are reconciled.
No SSH job was dispatched by this preflight failure or its repair.

The [SSH proposal receipt](../audit/depot-native-ssh-acceptance-2026-10-18.evidence.json)
and [source archive](../audit/depot-native-ssh-acceptance-2026-10-18.sources.json.gz)
separate source review and local controls from provider execution.
No native SSH job has been allocated under this proposal.

The deployed debugger guide distinguishes raw sandbox access from a paused workflow environment.
The CLI interactive terminal uses `OpenPtySession`;
its noninteractive `--info --output json` returns connection metadata.
The proposed actual controls use OpenSSH for direct and paused shells,
and a genuine local PTY for the native CLI RPC transport.
The pinned tmate action is equivalent to the CLI injection source,
without claiming execution of the mutable `--ssh-after-step` injection.

The isolated fixture preserves application `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Its native job requests `2` CPU and `8` GiB,
has a `3` minute job limit and a `2` minute paused-step limit.
The source-gated controller requests cancellation after `150` seconds,
starts bounded TERM/KILL cleanup at each `15` second leg deadline,
reaps its owned clients and verifies fresh exact-run terminal status.
One correction after terminal cleanup would reserve `USD 0.036` across both jobs at the documented list rate.
This reservation is scoped arithmetic,
not an invoice or aggregate account cap.
Root final source/admission review remains pending.
The admission pins the exact controller,
owned-process helper,
askpass and remote-consumer hashes,
plus the manifest fields that select paths,
source and resource limits.
The remote program is read once and reused from its admitted bytes.
The `210` second timer starts cleanup;
there is no independent hard timer over the whole controller and its cleanup.
Each controlled command retains its own bounded termination and reaping.

Local controls validate synthetic askpass delivery and prompt rejection,
source/challenge file consumers,
paused environment rejection and canonical workspace identity.
A mount-free network-disabled local container proves a UID `0` writer can restore UID `1000` and GID `1000`
before a workflow reader consumes the `0600` result.
The unrepaired root-owned `0600` control returns `EACCES`.
The local container is terminal and absent.
Local `script` controls prove actual stdin/stdout TTY identity,
challenge response and propagation of child exits `0` and `7`.
TERM-ignoring direct and separate-session children are killed and reaped;
every recorded owned PID is absent.
A lost dispatch response is recorded as potentially allocated,
with exact repository/source/time and workflow/job reconciliation before a correction.
These are local fixture controls,
not native SSH runtime qualifications.

The live API password stays in controller memory and one private askpass socket.
The tmate target is a bearer capability and remains private.
The workflow consumer verifies actual writer identity,
reader ownership,
challenge hashes and resume state.
Cleanup proves only the owned tmate socket and continue-file absence plus terminal CI status.
No exact tmate process absence,
SSH credential revocation,
or instantaneous VM deallocation is claimed.

[llms]: https://depot.dev/llms.txt
[sitemap]: https://depot.dev/sitemap.xml
[bundle]: https://depot.dev/llms-all.txt
[cli-upgrade]: https://github.com/depot/cli/pull/627
[gha-troubleshooting]: https://depot.dev/docs/github-actions/troubleshooting
[kvm-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/39104064329
[kvm-evidence]: ../audit/depot-github-kvm-documentation-contradiction-2026-10-18.evidence.json
[metal-launch]: https://depot.dev/blog/announcing-depot-metal
[metal-gha-rollout]: https://depot.dev/blog/how-we-load-tested-depot-metal-before-scaling-up
[metal-autoscaler]: https://depot.dev/blog/autoscaling-depot-metal
[github-runners]: https://depot.dev/docs/github-actions/overview
[build-autoscaling]: https://depot.dev/docs/container-builds/how-to-guides/autoscaling
[manage-runs]: https://depot.dev/docs/ci/how-to-guides/manage-workflow-runs
[retry]: https://depot.dev/docs/ci/how-to-guides/retry-steps
[parallel]: https://depot.dev/docs/ci/how-to-guides/parallel-steps
[pr-continuation]: https://depot.dev/changelog/2026-08-19-depot-ci-pr-run-cancellation
[duplicate-checks]: https://depot.dev/changelog/2026-08-21-duplicate-pull-request-checks
[check-names]: https://depot.dev/changelog/2026-09-23-accurate-github-checks
[reruns]: https://depot.dev/changelog/2026-09-23-depot-ci-rerun-fixes
[compatibility]: https://depot.dev/docs/ci/compatibility
[ci-api]: https://depot.dev/docs/api/ci/reference
[ci-openapi]: https://depot.dev/openapi/depot-ci/v1.json
[gha-openapi]: https://depot.dev/openapi/depot-github-actions/v1.json
[gha-api]: https://depot.dev/docs/api/github-actions-api
[registry]: https://depot.dev/docs/registry/overview
[registry-quickstart]: https://depot.dev/docs/registry/quickstart
[pull-through]: https://depot.dev/docs/registry/pull-through-cache
[security]: https://depot.dev/docs/security
[ai]: https://depot.dev/docs/ai-at-depot
[terms]: https://depot.dev/terms-of-service
[status]: https://status.depot.dev/
[account]: https://depot.dev/docs/account
[egress]: https://depot.dev/docs/github-actions/how-to-guides/egress-filtering
[tailscale]: https://depot.dev/docs/integrations/tailscale
[private-resources]: https://depot.dev/docs/github-actions/how-to-guides/access-private-resources
[dagger]: https://depot.dev/docs/github-actions/integrations/dagger
[dependabot]: https://depot.dev/docs/github-actions/integrations/dependabot
[test-split]: https://depot.dev/docs/ci/how-to-guides/split-tests
[test-results]: https://depot.dev/docs/ci/observability/depot-ci-test-results
[agents-overview]: https://depot.dev/docs/agents/overview
[agents-quickstart]: https://depot.dev/docs/agents/claude-code/quickstart
[code-overview]: https://depot.dev/docs/code/overview
[code-quickstart]: https://depot.dev/docs/code/quickstart
[managed-overview]: https://depot.dev/docs/managed/overview
[managed-aws]: https://depot.dev/docs/managed/on-aws
[managed-gpu]: https://depot.dev/docs/managed/using-gpus
