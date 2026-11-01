# Depot native CI capability review

## Evidence boundary

This review applies Depot's deployed native-CI documentation to the expanded repository evaluation,
without activating production workflows or changing subscriptions.
The parent read 18 complete pages,
verified their 156,769 captured bytes against individual SHA-256 identities,
and recorded 95 claim-level findings.
The [source review receipt](../audit/depot-native-ci-document-source-review-2026-10-18.evidence.json.gz)
joins those findings to the deployed source identities.
The [platform ledger](depot-platform-surface-ledger.json)
keeps retrieval separate from substantive review and preserves the independently reviewed split-tests guide.

These are first-party descriptions of supported behavior,
not demonstrations that every feature works for this repository.
Actual execution credit comes from the source-bound receipts linked in the
[evaluation report](depot-expanded-ci-evaluation.md).
An integration can be researched and ruled outside this task without being installed.

## Execution products and source identity

[Depot's native overview](https://depot.dev/docs/ci/overview)
describes Switchyard orchestration and Ubuntu Linux sandboxes on AMD64 and ARM64.
Native CI has no macOS or Windows sandbox;
Depot's operating-system runners are a separate GitHub Actions product.
The repository's Developer baseline can therefore retain GitHub orchestration while adding Depot execution,
with Mac eligibility evaluated separately from Windows eligibility and native Linux capacity.
The overview's advertised startup times are not a measured repository speed guarantee.

The [quickstart](https://depot.dev/docs/ci/quickstart)
keeps original GitHub workflows running after copying workflows and local Actions into `.depot/`.
Automatic native triggers are registered server side when the workflow file is under
`.depot/workflows/` on the default branch,
its trigger matches the event,
and the Code Access app is connected.
The deployed docs do not require a pull-request merge event,
and a native `pull_request` gate in this repository ran from a non-default base
([registration research](depot-native-registration-research.md)).
This creates duplicate deployment or publication risk for side-effecting workflows;
the research uses bounded explicit dispatches instead.
Migration also copies supporting non-symlink files,
so the destination needs source and consumer checks rather than a workflow-file count.

Correction (2026-10-19):
the earlier text said "Default-branch merging registers automatic
native triggers."
The deployed docs state three conditions:
the file is under
`.depot/workflows/` on the default branch,
the trigger matches the event,
and the GitHub
connection exists.
They do not require a pull-request merge event,
and the repository's native
`pull_request` gate on a non-default base contradicts the sentence as a general rule.
The
original sentence is preserved in this note.

The [compatibility matrix](https://depot.dev/docs/ci/compatibility)
qualifies the quickstart's migration claims.
Nonstandard runner labels can become a default Linux label,
unsupported permissions can be ignored,
and cross-repository reusable workflows require supported immutable references and appropriate repository access.
JavaScript Action runtime compatibility is separate from the job's Node on `PATH`.
Installing locked Node 26 does not by itself change the runtime of an Action.
Rendered workflow destinations and actual source checkout therefore need explicit verification.

## Scheduling and failure semantics

The [parallel-step guide](https://depot.dev/docs/ci/how-to-guides/parallel-steps)
describes concurrent branches that share the filesystem,
home and tool caches while receiving snapshots of step environment and output state.
State is merged after a parallel block;
concurrent mutation is not filesystem isolation.
Fail-fast cancellation and continue-on-error affect the resulting aggregate outcome.
The parent's coherent graph runs inside one shell process,
so it does not independently qualify every native parallel-step merge or cancellation rule.

The [retry guide](https://depot.dev/docs/ci/how-to-guides/retry-steps)
supports retrying `run` steps,
with additional attempts,
backoff and per-attempt timeout behavior.
Retries reuse job state and can accumulate environment,
output and filesystem changes.
They must not hide reproducible frozen-source failures or be mistaken for fresh-input samples.
The repeated graph deliberately retains failing owners and uses no automatic test retry.

The [workflow-run guide](https://depot.dev/docs/ci/how-to-guides/manage-workflow-runs)
distinguishes local patches,
full reruns and failed-job retries that also reconsider downstream work.
A native job's injected token cannot dispatch another workflow,
and job OIDC is not a substitute for dispatch authorization.
Broad organization or user tokens are not introduced to create a self-dispatch loop.
Cancellation and terminal workflow status remain separate from process absence,
VM deallocation and billing release.

## Cache and prepared-image boundaries

[Cache disks](https://depot.dev/docs/ci/how-to-guides/cache-disks)
have organization-wide names and can be mounted by concurrent jobs.
Conditional writes do not turn shared mutable directories into atomic caches.
Safe designs need a single writer,
partitioned keys or immutable content-addressed data,
with source/toolchain identity and a real restored-artifact consumer.
The guide's beta terms and retention description are not a measured price or retention guarantee.
Cache disks are distinct from Depot Cache,
BuildKit's cache and the registry's combined priced storage allowance.

The [custom-image guide](https://depot.dev/docs/ci/how-to-guides/custom-images)
describes filesystem snapshots stored in Depot Registry and selected alongside a sandbox size.
Environment variables and `PATH` changes do not persist as a complete running process environment.
Preinstalled tools need absolute-path or setup verification,
and a precloned checkout still needs the current source applied.
Prepared-image success does not establish a fresh install time or justify a stale output hit.
The parent's repeated graph intentionally restores no prepared snapshot.

## Credentials and protected environments

The [secrets guide](https://depot.dev/docs/ci/how-to-guides/manage-secrets-and-variables)
supports selectors for repository,
branch,
workflow and environment names,
with documented matching and precedence rules.
Matching an environment name for secret selection is separate from GitHub environment approval integration,
which the compatibility matrix marks unsupported.
The existing exact temporary-selector and deletion controls do not exercise every glob or precedence conflict.
Bulk secret migration is unnecessary for this evaluation and would expand the credential scope.

The [OIDC guide](https://depot.dev/docs/ci/oidc)
uses Depot's issuer and subject format for external cloud federation,
with explicit `id-token: write` and audience requirements.
Its subject includes stable organization,
repository and ref segments followed by a changing sandbox identity.
A future trust policy would match the stable boundary and verify the cloud provider's actual deciding rules.
No AWS,
GCP or Azure trust relationship is created here.
Automatic native test-service authentication,
native BuildKit authentication,
GitHub project OIDC and workflow dispatch remain separate authorization contracts.

## Tests and evidence consumers

The [test-results guide](https://depot.dev/docs/ci/observability/depot-ci-test-results)
accepts real JUnit output through the report Action,
with native service authentication that does not require a stored `DEPOT_TOKEN` secret.
Exact attempt IDs avoid the ambiguity of latest-attempt job lookups.
Passing result events are a usage unit,
not a count of unique test bodies or covered branches.
The existing timed shard adapters and retained XML establish their recorded identities and assertions;
they do not establish every advertised historical analytics or deduplication behavior.

The [GitHub checks guide](https://depot.dev/docs/ci/observability/github-checks)
maps native jobs to checks on a commit.
Check names alone do not identify the provider,
job attempt or actual pull-request merge checkout.
The evaluation keeps App identity,
source SHA,
check outcome and actual merge-consumer evidence joined.
The retained native job-level continue-on-error mismatch also prevents assuming that every green downstream job
means a successful overall workflow.

The [logs guide](https://depot.dev/docs/ci/observability/depot-ci-logs)
supports job and attempt correlation,
per-step retrieval and following logs.
Raw Actions logs and signed artifact URLs remain private;
public evidence retains safe selected diagnostics,
hashes and consumer assertions.
An empty log response without a retention or completeness guarantee does not prove no execution.

The [metrics guide](https://depot.dev/docs/ci/observability/depot-ci-metrics)
describes job and step resource information that can inform sandbox sizing.
Resource graphs are not invoices.
Critical-path comparisons must retain setup and actual branch overlap rather than sum isolated phase timings.
No account-wide usage or settings page is inspected to obtain these observations.

## Debugging and automatic AI processing

The [SSH guide](https://depot.dev/docs/ci/how-to-guides/debug-with-ssh)
distinguishes direct sandbox access from a paused workflow-step environment.
Its tmate configuration can allow anyone with the connection string to enter the session.
Printing a connection command does not demonstrate a successful SSH consumer,
and no live session string belongs in public evidence.
This evaluation claims only the debugging controls it actually exercised.

The [coding-agent guide](https://depot.dev/docs/ci/how-to-guides/coding-agents)
describes patching local changes into CI and using diagnosis output in a repair loop.
Native CI also provides automatic AI summaries from logs.
The prohibition on explicit paid model calls does not mean that Depot performs no vendor-side AI processing.
The research preserves immutable application cohorts and does not grant an agent blanket write permissions
or let a repair loop silently change assertions to produce a green result.

## Integrations that remain unactivated

The [Datadog integration](https://depot.dev/docs/ci/observability/datadog-ci-visibility)
requires both an organization switch and a Datadog secret for native pipeline emission.
Its native ingest differs from GitHub Actions webhooks,
is best-effort,
omits skipped work and maps cancellation into error states.
No Datadog setting,
credential or telemetry sink is enabled;
the retained first-party logs and test evidence supply the current audit requirement.

The [Origin integration](https://depot.dev/docs/ci/integrations/origin)
distinguishes Origin-hosted repositories from GitHub-authoritative mirrors.
Its forge-specific compatibility findings can be advisory even when a publication or security-report mode fails.
Those limitations must not be generalized to GitHub-backed Depot CI.
This repository remains GitHub-authoritative;
no Origin subscription,
repository mirror,
forge connection or secret import is introduced.

## Implications for the adoption decision

The settled direction is an additional execution pool and broader CI coverage.
Native Linux orchestration,
Depot GitHub runners,
BuildKit,
registry artifacts,
test results and the distinct cache systems need source-bound consumers and separate cost accounting.
Unactivated optional integrations stay explicit rather than being counted as executed capabilities.
Production required checks,
retention choices and recurring workload allocation remain interview decisions after the factual queue closes.
