# Namespace CI migration vet report

- Status:
   complete for the no-migration finding;
   managed-runner provider selection remains blocked
- Lifecycle phase:
   screened;
   no new provider reached validated-finalist status
- Subject:
   Namespace CI migration
- Decision scope:
   whether migrating Monochromatic GitHub Actions execution to Namespace avoids repository splitting and
  fixes measured CI pressure
- Started:
   2026-08-19
- Last updated:
   2026-08-19
- Governing skill commit:
   `a05818ad70a40e5769a36de669697ba109891b31`
- Governing skill SHA-256:
   `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`
- Compatibility fingerprint:
   `4274789320726bb9dc12e712353cae5246b2f89a31f16c6cf267ec42d5f77bb7`
- Active audit owner:
   current pi session
- Prior compatible report:
   none found

## Answer

No.
Namespace can make selected jobs faster and give them better caches,
but migrating runners does not fix the repository's dominant measured problem:
GitHub is creating too many jobs before any runner provider becomes relevant.

Do not split the repository for this.
Do not perform a full Namespace migration now.
First change the event and task-selection layer,
then benchmark Namespace on any remaining slow job.

This is a negative finding about migration scope,
not a claim that Namespace is a poor runner service.
Namespace appears technically capable of running this repository's Linux,
Windows,
and macOS jobs,
but it is an executor overlay rather than a complete CI architecture.

## Material finding

The week beginning 2026-08-12 contains 511 commits.
The GitHub Actions API hit its 1,000-run response cap for that interval,
and the latest 1,000 runs contained 483 CodeQL push runs.
The peak observed start rate was 131 workflow runs in one hour.

The cause is now traced in
`doc/troubleshooting/github-codeql-protected-branch-run-amplification.md`:

1. Classic GitHub branch-protection pattern `*` matches every non-default branch.
2. All 25 live branches were protected when measured.
3. GitHub documents that CodeQL default setup runs on each push to the default branch or any protected branch
   (<https://docs.github.com/en/code-security/concepts/code-scanning/setup-types>).
4. Each CodeQL run fans out into `actions`,
    `c-cpp`,
    and `javascript-typescript` jobs.
5. Default setup exposes no repository workflow file in which to add a concurrency group.

Namespace's documented migration is a `runs-on` label change
(<https://namespace.so/docs/solutions/github-actions/migration>).
GitHub separately documents that default setup can target a labeled runner
(<https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/edit-default-setup>).
Neither setting changes the protected-branch trigger.
A Namespace-labeled CodeQL runner would therefore execute the same repeated analyses on different hardware.

## Scope assumption

The user's phrase “all my problems” is not a measurable requirement by itself.
This report interprets it from the live repository as:

- avoid a repository split;
- reduce redundant push work;
- preserve CodeQL and GitHub-native security outputs;
- preserve Linux,
  Windows,
  and macOS host evidence;
- preserve publishing,
  OIDC,
  attestations,
  SARIF,
  secrets,
  and merge-queue behavior;
- leave room for future fuzz and mutation jobs;
- improve feedback time only after work selection is correct.

If the intended problem is instead a named latency target,
that target requires a separate controlled runner benchmark.
No latency target appears in the request or repository decisions.

## Compatibility fingerprint input

```json
{"baseCategories":["Managed service or SaaS"],"decisionScope":"Whether migrating Monochromatic GitHub Actions execution to Namespace avoids repository splitting and fixes measured CI pressure.","deployment":{"executor":"Managed cloud runners","orchestration":"GitHub Actions","repository":"Public GitHub.com monorepo"},"hardConstraints":["Do not require splitting the repository.","Keep GitHub Actions orchestration and GitHub-native integrations.","Preserve required Linux x64, macOS arm64, and Windows x64 host-evidence jobs.","Securely execute public-repository pull requests without persistent cross-job state."],"incumbent":"GitHub Actions standard hosted runners as of 2026-08-19","overlays":["high-trust CI execution","incumbent dependency replacement","multi-platform claim","sensitive credentials"],"schemaVersion":1,"subject":"Namespace CI migration","trustBoundary":"Third-party managed runners execute repository code, GitHub tokens, publishing credentials, and OIDC identities."}
```

## Measured repository context

### Repository shape

Measured from the current tree on 2026-08-19:

- 156 `package.json` files;
- 28 `Cargo.toml` files;
- 3 `build.gradle.kts` files;
- 86,147,067 tracked file bytes;
- 165,129 KiB in the local packed Git object store;
- 12 tracked files under `.github/workflows/`;
- one GitHub-managed CodeQL default-setup workflow.

There is no general repository build,
lint,
or unit-test workflow in `.github/workflows/`.
The current files cover publishing,
CodeQL and Scorecard integration,
forbidden strings,
TOML fuzzing,
and selected host-evidence lanes.
Changing runners does not create the missing broad CI coverage or an affected-package task graph.

### Live Actions sample

The latest 1,000 workflow runs returned on 2026-08-19 contained:

- 967 successful runs;
- 10 failed runs;
- 13 skipped runs;
- 10 runs still active at the sample point.

CodeQL accounted for 483 push runs in that sample:

- 246 `Push on main` runs;
- 237 `Push on translation-repair-rebased` runs.

Two inspected CodeQL runs show why trigger count matters:

- Main run `32204388026` used 6 minutes 38 seconds of summed job execution across its three language jobs.
- Working-branch run `32279201781` used 15 minutes 17 seconds of summed job execution,
  including 12 minutes 32 seconds in `Perform CodeQL Analysis` for JavaScript and TypeScript.

These are direct `gh run view` measurements,
not estimates of Namespace performance.

### GitHub-native boundaries

The workflows use:

- `merge_group`;
- repository and publishing secrets;
- GitHub OIDC;
- GitHub artifact attestations;
- SARIF upload;
- GitHub release and artifact APIs;
- pinned GitHub Actions commits;
- Linux,
  Windows,
  and macOS host behavior.

Namespace retains GitHub Actions orchestration,
so its runner model is compatible in principle.
Runtime parity remains unverified for this repository.

## Hard constraints

- Keep the repository monolithic.
- Keep GitHub Actions orchestration and GitHub-native integrations.
- Preserve required Linux x64,
  macOS arm64,
  and Windows x64 host-evidence jobs.
- Securely execute public-repository pull requests without persistent cross-job state.
- Preserve publishing credentials,
  OIDC identities,
  provenance attestations,
  and SARIF uploads.
- Reduce redundant work rather than only processing the same work faster.
- Do not present an account-linked service as validated without exercising this repository on it.

## Frozen soft criteria

The user supplied no relative weights beyond keeping the monorepo,
which is already a hard constraint.
Every soft criterion therefore has weight 1:

- redundant-work control;
- queue and concurrency behavior;
- warm and cold execution speed;
- cache effectiveness for pnpm,
  Rust,
  Gradle,
  and container builds;
- GitHub workflow compatibility and migration effort;
- security isolation and credential boundary;
- platform coverage;
- reliability and support;
- price for a public,
  high-frequency repository;
- exit cost and configuration portability;
- human auditability of runner-side components.

No score was calculated because no new candidate completed the hard-gate and runtime-validation path.
Hard-gate failures and blocked validation cannot be converted into soft points.
Sensitivity analysis is therefore not applicable.

## Discovery record

### Query schedule

The initial schedule covered:

- Namespace GitHub runner architecture,
  migration,
  isolation,
  profiles,
  platforms,
  caching,
  Docker builds,
  security,
  privacy,
  terms,
  support,
  billing,
  limits,
  status,
  and data residency;
- GitHub hosted-runner limits,
  public-repository billing,
  self-hosted runner security,
  CodeQL default and advanced setup,
  labeled runners,
  workflow concurrency,
  and path-filter limits;
- the Namespace GitHub organization and runner-side actions;
- broader searches for Namespace alternatives,
  Depot,
  Blacksmith,
  RunsOn,
  BuildJet,
  WarpBuild,
  Ubicloud,
  and Actions Runner Controller;
- funding,
  layoffs,
  incidents,
  reviews,
  billing,
  support,
  cancellation,
  security history,
  and open-source sponsorship.

The one expansion round added:

- `ephemeral micro VM`;
- `restricted access level`;
- `cache poisoning`;
- `default setup labeled runner`;
- `wildcard branch protection`;
- `OSS sponsorship`;
- `runner image provenance`;
- `control plane incident`.

### Search results and saturation

The web search provider returned capped result sets without a total count or page cursor.
GitHub Marketplace searches did not surface a public Namespace app listing with an inspectable permission manifest.
The Namespace status page exposed history from May through August 2026,
but repeated `page` parameters returned the same interval.

The required broad source class therefore did not satisfy the choosing-technology pagination and saturation rule.
Provider selection terminates as **discovery blocked**.
This does not block the narrower capability finding that changing runner labels leaves GitHub triggers unchanged.
That finding follows directly from GitHub and Namespace primary documentation.

## Candidate ledger

### GitHub Actions standard hosted runners

- Discovery source:
   repository configuration and `doc/decision/ci-provider.md`.
- Base category:
   managed service or SaaS.
- Overlays:
   incumbent retention,
  high-trust CI execution,
  sensitive credentials,
  and multi-platform.
- Current state:
   live baseline,
  not a new adoption candidate.
- Screening result:
   retains category fit for orchestration and execution.
- Validation:
   live repository runs confirm current consumer-boundary behavior.

### Namespace managed GitHub runners

- Discovery source:
   user's request and Namespace primary documentation.
- Base category:
   managed service or SaaS.
- Overlays:
   incumbent executor replacement,
  high-trust CI execution,
  sensitive credentials,
  and multi-platform.
- Screening result as an “all CI problems” migration:
   **fail,
  category mismatch**.
- Screening result as a selective acceleration overlay:
   pending runtime and trust-boundary validation.

### GitHub self-hosted runners and Actions Runner Controller

- Discovery source:
   existing CI decision and GitHub's open-source ARC project.
- Base category:
   GitHub managed control plane plus inspectable local runner technology.
- Screening result for the immediate run flood:
   **fail,
  category mismatch**.
- Reason:
   self-hosting changes execution capacity,
  not CodeQL default-setup triggers.
- Additional cost:
   runner lifecycle,
  platform capacity,
  logs,
  patching,
  and public-pull-request isolation become operator responsibilities
  (<https://docs.github.com/en/actions/reference/runners/self-hosted-runners>).

### Depot

- Discovery source:
   broader managed-runner search.
- Base category:
   managed service or SaaS.
- Screening result for the immediate run flood:
   **fail,
  category mismatch**.
- Depot supports Linux,
  Windows,
  and macOS and documents ephemeral EC2 runners,
  accelerated caches,
  and label-only migration
  (<https://depot.dev/docs/github-actions/overview>).
- It would still receive the same GitHub workflow jobs.

### RunsOn

- Discovery source:
   repository-host and broader searches.
- Base category:
   proprietary control software deployed into the user's AWS account.
- Screening result for full platform replacement:
   **fail**.
- RunsOn's current platform reference says macOS is not supported
  (<https://runs-on.com/docs/runners/>).
- It also changes execution capacity,
  not trigger selection.

### Blacksmith,
WarpBuild,
BuildJet,
Ubicloud,
and other runner services

- Discovery source:
   broader managed-runner searches and independent comparison leads.
- State:
   discovered leads only.
- Screening result for the direct question:
   **category mismatch**.
- Reason:
   a runner provider is downstream of GitHub event and workflow generation.
- No provider ranking is made because discovery and equal-depth validation did not complete.

## Namespace evidence

### Platform and migration fit

Namespace's current GitHub Actions guide supports:

- Linux AMD64;
- Linux ARM64;
- Windows AMD64;
- macOS ARM64;
- profile and inline runner labels;
- GitHub Actions workflow compatibility.

Primary source:
<https://namespace.so/docs/solutions/github-actions>.

Migration requires installing the Namespace GitHub app,
associating selected repositories,
creating profiles,
and changing runner labels
(<https://namespace.so/docs/solutions/github-actions/migration>).
The app registration and service control plane remain new privileged dependencies.
Exact public app permissions were not found outside the installation flow.

### CodeQL fit

GitHub default setup can target an existing labeled runner
(<https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/edit-default-setup>).
A Namespace profile could therefore execute current CodeQL default setup without first moving to advanced setup.

This is technically possible but solves the wrong layer.
The trigger rule remains:
each push to the default branch or any protected branch
(<https://docs.github.com/en/code-security/concepts/code-scanning/setup-types>).

### Performance evidence

Namespace publishes customer stories reporting faster Rust and C++ CI,
including Zed and DuckDB
(<https://namespace.so/customers/zed> and <https://namespace.so/customers/duckdb>).
These are vendor-authored case studies and are not independent validation.

RunsOn's competitor-operated benchmark,
last updated in July and August 2026,
measured Namespace ahead of standard GitHub runners in single-thread CPU,
4 GB cache transfer,
and random-write disk performance
(<https://runs-on.com/benchmarks/github-actions-cpu-performance/>,
<https://runs-on.com/benchmarks/github-actions-cache-performance/>,
and <https://runs-on.com/benchmarks/github-actions-disk-performance/>).
That source discloses RunsOn's interest,
and its Namespace samples do not execute this repository.
It supports a trial hypothesis,
not a migration conclusion.

### Caching fit

Namespace provides:

- transparent compatibility with GitHub Actions cache;
- local cache volumes;
- toolchain and action-download caching;
- container-image caching;
- Git mirror caching through an alternative checkout action;
- main-branch-only cache write protection;
- Gradle,
  Rust,
  pnpm,
  and other framework integrations.

Primary sources:
<https://namespace.so/docs/solutions/github-actions/caching> and
<https://namespace.so/docs/architecture/storage/cache-volumes>.

This repository could benefit in future broad build and test jobs.
Current CodeQL query execution is not shown in Namespace documentation as a cache-volume integration,
so no CodeQL cache-speed claim is made.

### Concurrency and price

GitHub states that public repositories receive free and unlimited use of standard hosted runners
(<https://docs.github.com/en/actions/reference/runners/github-hosted-runners> and
<https://docs.github.com/en/billing/concepts/product-billing/github-actions>).
The Free plan allows 20 concurrent standard jobs,
with at most 5 concurrent macOS jobs
(<https://docs.github.com/en/actions/reference/limits>).

Namespace meters compute by unit minute.
Its current standard plans are:

- Developer:
  pay as used,
  32 Linux vCPU,
  12 macOS vCPU,
  and no Windows runners;
- Team:
  $100 per month,
  100,000 included unit minutes,
  64 Linux vCPU,
  24 macOS vCPU,
  and 32 Windows vCPU;
- Business:
  $250 per month,
  250,000 included unit minutes,
  160 Linux vCPU,
  48 macOS vCPU,
  and 80 Windows vCPU.

Primary sources:
<https://namespace.so/pricing.md> and
<https://namespace.so/docs/architecture/compute/resource-limits>.

A full migration must use Team or higher because the repository has required Windows host-evidence jobs.
Namespace's open-source customer stories mention an OSS sponsorship program and provide
`opensource@namespacelabs.com`,
but no public eligibility,
capacity,
renewal,
or pricing terms were found.
Sponsorship cannot be treated as granted.

### Security boundary

Namespace documents:

- ephemeral micro-VM execution with a clean environment per public-repository job
  (<https://namespace.so/blog/completing-the-circle-with-github-runners>);
- SOC 2 Type II status with the report available by Trust Center request
  (<https://namespace.so/docs/workspaces/security>);
- default cross-workspace network isolation
  (<https://namespace.so/docs/architecture/networking/security>);
- branch-restricted cache commits to reduce cache poisoning
  (<https://namespace.so/docs/solutions/github-actions/caching>);
- profile access levels,
  including a restricted mode for untrusted code
  (<https://namespace.so/docs/reference/github-actions/runner-configuration>);
- egress policies on Linux and macOS
  (<https://namespace.so/docs/security/egress-filtering>).

Important limits remain:

- permissive is the default runner access level;
- Windows is absent from the documented egress-policy platforms;
- standard runner images include Namespace integration software whose complete source-to-image mapping was not found;
- the GitHub app and Namespace control plane are privileged and proprietary;
- runtime behavior was not exercised with this repository.

### Optional local actions

The following repositories were cloned and inspected at exact commits:

- `namespacelabs/nscloud-setup` at `df198f982fcecfb8264bea3f1274b56a61b6dfdc`;
- `namespacelabs/nscloud-cache-action` at `c5f8dab7560444c4bf8dbc64f1b203431873c547`;
- `namespacelabs/nscloud-checkout-action` at `66f2dc6f6c42a8ac6c4e53473c4840006822831e`.

The setup and checkout repositories carry inspectable source and permissive license metadata.
The cache-action repository and package manifest exposed no license.
Its default path can install supporting CLI software when a system binary is absent.
The optional cache action therefore did not pass the license
and artifact-provenance gate for adoption in this repository.
Basic runner use and transparent GitHub cache compatibility do not require adopting that action.

### Reliability and service history

The official status history exposed 50 unique incident entries from May through 2026-08-19.
The list mixes Namespace incidents with explicitly named GitHub and Docker Hub upstream incidents
(<https://namespace-status.com/history>).
Examples include Namespace control-plane errors on 2026-08-12 and delayed runner assignment on 2026-08-19
(<https://namespace-status.com/incidents/01KZV98B7BD2KHCJF8W2RZBMCT> and
<https://namespace-status.com/incidents/01M0C1A8DYK12S79CASN8VXRMS>).

The visible interval does not cover the required 12 months,
and page parameters did not reveal older history.
No standard-plan SLA was found;
the support page directs custom SLA requests to sales
(<https://namespace.so/support>).
Reliability therefore remains a validation gap,
not a pass or fail.

### Company continuity

Namespace announced $23 million total Seed and Series A funding led by NEA on 2026-03-23
(<https://namespace.so/blog/series-a>).
NEA independently confirms that it led the round
(<https://www.nea.com/blog/namespace-cicd-is-dead-agents-need-computers>).
Namespace says it owns and operates hardware,
racks,
and networking
(<https://namespace.so/docs/architecture/compute>).

No primary layoff disclosure was found in the 24-month search.
That absence is low-signal and is not evidence that no layoffs occurred.
Current headcount was not established from a primary source.

### Terms,
privacy,
and exit

Namespace's terms:

- permit service changes and limits;
- permit account suspension or termination at Namespace's discretion;
- provide automatic renewal for paid plans;
- generally do not provide prorated refunds;
- cap aggregate liability at the greater of $100 or fees paid in the prior 12 months.

Primary source:
<https://namespace.so/terms>,
valid from 2024-08-31.

The privacy policy is older,
effective 2022-03-25.
It says Namespace collects product telemetry including invoked commands,
may use data to improve services,
uses service providers,
and may transfer personal data across borders
(<https://namespace.so/privacy>).

Namespace documents APIs for logs,
metrics,
artifacts,
and cache management,
but no complete standard-plan account export procedure or API compatibility policy was found.
Terms say account termination may destroy associated content.
The runner-label integration is easy to reverse,
but cache volumes,
custom images,
metrics,
and Namespace-specific actions increase exit work.

### Residency and access

Namespace says regional preferences are available on all plans,
while region-exclusive data residency requires Enterprise.
Operational metadata remains globally replicated even under exclusive workload residency
(<https://namespace.so/docs/workspaces/data-residency>).

The current repository has no stated residency requirement,
so this is not a hard-gate failure.

## Evidence and execution records

### Live repository measurement

- Candidate:
   incumbent baseline and all runner alternatives.
- Host:
   Linux `7.1.5-ogc5.1.fc44.x86_64`,
   x86_64.
- Tools:
   GitHub CLI 2.97.0,
   jq 1.8.1,
   and curl 8.18.0.
- Working directory:
   `/var/home/user/Monochromatic`.
- Credentials:
   existing read access through GitHub CLI;
   no secrets were printed or transferred to candidates.
- Repository mutations:
   none.

Repository-shape commands:

```console
find package -name package.json -type f | wc --lines
find package -name Cargo.toml -type f | wc --lines
find package -name build.gradle.kts -type f | wc --lines
git ls-files -z | du --files0-from=- --total --bytes
git count-objects --verbose
git ls-files '.github/workflows/*'
```

Actions-demand commands:

```console
git rev-list --count --since='2026-08-12T00:00:00Z' HEAD

gh api --paginate \
  'repos/Aquaticat/Monochromatic/actions/runs?per_page=100&created=%3E%3D2026-08-12'

gh run view 32279201781 \
  --json jobs,workflowName,createdAt,updatedAt,conclusion,url

gh run view 32204388026 \
  --json jobs,workflowName,createdAt,updatedAt,conclusion,url
```

Configuration commands:

```console
gh api repos/Aquaticat/Monochromatic/code-scanning/default-setup

gh api graphql --raw-field query='query {
  repository(owner: "Aquaticat", name: "Monochromatic") {
    branchProtectionRules(first: 100) {
      nodes { id pattern matchingRefs(first: 100) { nodes { name } } }
    }
  }
}'

gh api repos/Aquaticat/Monochromatic/branches --paginate \
  --jq 'map({name,protected})'
```

All commands exited successfully.
The first malformed GraphQL draft failed to parse and was replaced by the successful query recorded here;
it produced no repository mutation.

### Namespace source inspection

- Candidate:
   Namespace optional GitHub Actions.
- Isolation:
   private scratch directories under `${HOME}/temp/agent/` with mode 700 on the scratch root.
- Network:
   GitHub clone access only.
- Ambient credentials:
   GitHub CLI clone authentication;
   no candidate binary received credentials.
- Execution:
   no lifecycle,
   install,
   build,
   test,
   generated,
   native,
   Wasm,
   or downloaded candidate command was run.
- Writes:
   clone directories only,
   outside this repository.
- Stop condition:
   source and metadata available for static inspection.

Clone commands:

```console
gh repo clone namespacelabs/nscloud-setup \
  "${HOME}/temp/agent/nscloud-setup-2026-08-19" -- --depth 1

gh repo clone namespacelabs/nscloud-cache-action \
  "${HOME}/temp/agent/nscloud-cache-action-2026-08-19" -- --depth 1

gh repo clone namespacelabs/nscloud-checkout-action \
  "${HOME}/temp/agent/nscloud-checkout-action-2026-08-19" -- --depth 1
```

The exact revisions appear in `Optional local actions`.
No external-execution manifest was needed beyond this static clone boundary because candidate code was not executed.

### GitHub documentation source inspection

GitHub documentation was cloned at `a34bf588b9e6eff791e173fdd3a726dfab26f888` with blob filtering and sparse checkout.
Relevant source paths were read without running repository code:

- `content/code-security/concepts/code-scanning/setup-types.md:16-20`;
- `content/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/edit-default-setup.md:27-35`;
- `data/reusables/actions/actions-group-concurrency.md:7-20`;
- `data/reusables/actions/workflows/triggering-a-workflow-paths5.md:8-14`.

### Status-history measurement

The official history page was fetched read-only.
Unique incident links in its visible interval were counted with:

```console
curl --silent --show-error --location 'https://namespace-status.com/history' \
  | rg --only-matching '/incidents/[A-Z0-9]+' \
  | sort --unique \
  | wc --lines
```

The command returned 50.
This measures entries,
not duration,
severity,
or Namespace-attributable incidents.

### Runtime validation boundary

A Namespace consumer-boundary run would require installing its GitHub app,
creating an account or workspace,
creating runner profiles,
and routing repository jobs to those profiles.
The evaluation request did not adopt Namespace or authorize that external service mutation.
No runtime claim is inferred from static configuration or vendor benchmarks.

## Hard-gate outcomes

### Namespace as a complete CI migration

Outcome:
 **fail**.

Reason:
Namespace changes job execution after GitHub has generated the job.
It does not remove protected-branch CodeQL triggers,
create an affected-package graph,
or add missing broad build,
lint,
and test coverage.
It therefore cannot fix all measured CI problems.

### Namespace as a selective accelerator

Outcome:
 **pending,
not validated**.

It passes preliminary platform and GitHub Actions compatibility checks.
It does not pass the complete validation gate because:

- no Namespace account or GitHub app was linked;
- no exact runner profile was exercised;
- required macOS and Windows host evidence was not run;
- publishing,
  attestations,
  OIDC,
  SARIF,
  and secrets were not exercised;
- precise app permissions and standard-image provenance remain incomplete;
- full 12-month status history was unavailable;
- no consumer-boundary benchmark exists for this repository.

### Repository split

Outcome:
 **rejected as unnecessary**.

Classic branch protection pattern `*` and CodeQL setup type can be changed without changing repository topology.
Future package CI can select affected tasks inside the monorepo.
GitHub's current path-filter diff limit is 3,000 files,
not the older 300-file limit,
and pushes over 1,000 commits run filtered workflows unconditionally
(<https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax>).
Those limits warrant robust in-job task selection for large changes,
not a repository split.

## Scoring and sensitivity

No candidate was scored.

- Namespace failed category fit as a complete solution.
- Namespace did not complete runtime validation as a selective accelerator.
- Other provider discovery did not saturate and no alternative completed equal-depth validation.
- The live GitHub incumbent is a baseline,
  not a newly selected finalist in this question.

There is consequently no arithmetic ranking that can legitimately choose a provider.
The no-migration finding is stable under every soft-criterion weight because it follows from a category mismatch:
runner speed cannot suppress upstream trigger generation.

After trigger remediation,
a selective-runner decision could change with measured wall time,
queue time,
cache hit ratio,
sponsorship terms,
and the user's latency-versus-governance preference.
Those inputs require a new controlled evaluation.

## Options with pros and cons

### Keep GitHub execution and repair orchestration

Pros:

- attacks the measured source of repeated work;
- keeps public standard runner compute free;
- adds no new app,
  service,
  credential,
  or data-retention boundary;
- preserves current Linux,
  Windows,
  macOS,
  OIDC,
  attestations,
  and SARIF behavior;
- keeps the repository monolithic.

Cons:

- advanced CodeQL setup means maintaining a workflow file;
- GitHub standard runners retain their current CPU,
  disk,
  cache,
  and concurrency limits;
- narrowing wildcard branch protection is a governance change with force-push,
  deletion,
  and conversation-resolution consequences.

### Trial Namespace on one remaining slow job after remediation

Pros:

- limits trust,
  migration,
  and billing exposure;
- can measure this repository on Namespace hardware rather than importing customer-story results;
- preserves a per-job rollback to GitHub labels;
- can test cache and observability where they have a plausible effect.

Cons:

- still requires the Namespace app and account;
- does not reduce workflow triggers;
- Windows requires Team or higher unless Namespace grants different sponsorship terms;
- hybrid runner images can expose platform differences;
- a useful trial requires explicit before-state run-to-run variance and consumer-boundary tests.

### Fully migrate current jobs to Namespace now

Pros:

- offers one managed runner provider across current operating systems;
- offers faster hardware,
  cache volumes,
  remote builders,
  and richer metrics;
- uses GitHub Actions syntax rather than replacing orchestration.

Cons:

- pays or depends on sponsorship to accelerate redundant work;
- adds a privileged GitHub app and proprietary control plane;
- cannot fix missing task selection or missing test coverage;
- moves free public execution to metered capacity;
- has not been validated at this repository's publishing and host-evidence boundaries;
- increases exit work if Namespace-specific cache,
  checkout,
  images,
  artifacts,
  or APIs are adopted.

## Operational ranking

This is an order of investigation,
not a managed-runner vendor selection:

1. Keep the monorepo and repair GitHub orchestration.
2. If measured latency remains after redundant runs are removed,
   trial Namespace on one representative slow job.
3. Consider a full runner migration only if that trial passes every platform,
   security,
   reliability,
   and cost boundary.

The first option outranks the second because it removes work,
while the second only accelerates work and remains unvalidated.
The second outranks the third because a bounded trial can prove or reject the benefits
before expanding the trust and billing boundary.

## Post-evaluation implementation

The user subsequently chose and confirmed the orchestration policy.
Commit `000d58464dccc3088e8fd4476692e10bdcf64d2c`:

- added `.github/workflows/codeql.yml`;
- replaced CodeQL push analysis with a daily `03:17 UTC` batch;
- retained pull-request analysis targeting `main`;
- added manual dispatch;
- preserved the three prior language groups,
  extended queries,
  and remote-plus-local threat coverage;
- added supersession cancellation to seven replaceable validation workflows;
- left release,
  publishing,
  Scorecard,
  Claude,
  and branch-protection configuration unchanged.

GitHub recognized the advanced workflow as ID `338017899`.
The default-setup API then accepted `state: not-configured`.
Manual advanced run `32284761248` completed all three language jobs and uploaded their analyses successfully.

Three later main pushes created their path-matched non-CodeQL workflows but no CodeQL run.
This provides a positive control for event processing and a negative result for both the retired dynamic workflow and
advanced workflow.
Manual final-newline run `32285688509` was cancelled by superseding run `32285692427`,
which then completed successfully;
this exercises the new validation concurrency through GitHub Actions.

Disposable draft pull request #450 exercised the remaining pull-request boundary.
CodeQL run `32288000285` was cancelled after a new revision created run `32288031286` in the same PR-ref group.
All three language jobs in the superseding run succeeded and uploaded analyses on `refs/pull/450/merge`.
The first final-newline and forbidden-strings runs were likewise cancelled,
and both superseding runs succeeded.
The probe was closed without merging and its branches were deleted.

Classic branch-protection patterns `main` and `*` retain required-check enforcement,
but both return empty required check-context lists;
disabling default CodeQL setup therefore left no stale required check name.
The first natural scheduled CodeQL run,
`32330328949`,
completed successfully on 2026-08-20.
It started at `04:01:16 UTC`,
44 minutes and 16 seconds after the configured `03:17 UTC` trigger.
All three language jobs succeeded and uploaded analyses on `refs/heads/main`;
the only annotations were the documented `MissingPushHook` warnings.

The toml-edit-fuzz verification exposed and corrected an existing pnpm bootstrap cycle and stale coverage baseline.
Final run `32286257673` passed installation,
build,
type checking,
unit tests,
fuzz smoke,
TOML conformance,
and the deterministic coverage gate.
The diagnosis and fixes are recorded in `doc/troubleshooting/toml-edit-fuzz-workflow.md`.

The CodeQL action emits `MissingPushHook` because the requested workflow
has pull-request analysis without a push trigger.
Pinned action source at `github/codeql-action` tag `v4.37.7`,
`src/workflow.ts:190-199`,
shows that this diagnostic ignores scheduled and manual default-branch analysis.
The annotation is an accepted consequence of the chosen batching policy,
not an analysis failure.

The complete after-state evidence is maintained in
`doc/troubleshooting/github-codeql-protected-branch-run-amplification.md`.

## Terminal result

- Direct question:
   Namespace would **not** fix all current CI problems.
- Repository topology:
   keep the monorepo.
- Adoption:
   do not migrate now.
- New-provider selection:
   no recommendation because discovery saturation,
  high-trust inspection,
  and runtime validation did not complete.
- Revisit trigger:
   redundant work has been removed and a named job still misses a measured feedback target.
