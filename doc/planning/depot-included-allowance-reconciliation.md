# Depot included-allowance reconciliation

## Result and evidence boundary

Current public pricing documents separate monthly Docker-build,
Depot CI and GitHub Actions entitlements.
It does not describe one shared CI/GitHub Actions balance or transferable credits between products.
The combined forecast should retain those named entitlements separately.
This is the documented commercial model,
not an observation of this organization's billing ledger.
No account-wide usage,
invoice,
subscription or settings endpoint was inspected.

The [live browser capture](../audit/depot-live-public-pricing-2026-10-18.evidence.json.gz)
retains the expanded pricing FAQs and all compute tabs at `2026-10-18T09:35:35.469Z`.
The [fresh source archive](../audit/depot-included-allowance-2026-10-18.sources.json.gz)
adds the unauthenticated pricing HTML and rendered text at `2026-10-18T12:23:28.074Z`,
current deployed deciding Markdown,
and the public schema inspection.
The [machine-readable reconciliation](../audit/depot-included-allowance-2026-10-18.evidence.json)
provides exact members,
selectors,
source hashes and arithmetic boundaries.

## Separate named monthly entitlements

The current pricing cards explicitly list Developer at USD 20/month:

- 500 Docker build minutes.
- 2,000 Depot CI minutes.
- 2,000 GitHub Actions minutes.
- 25 GB combined cache and registry storage.
- 1,000,000 passing test results.

Startup is USD 200/month:

- 5,000 Docker build minutes.
- 20,000 Depot CI minutes.
- 20,000 GitHub Actions minutes.
- 250 GB combined cache and registry storage.
- 1,000,000 passing test results.

These are separate rows in `featureRows[].cells` and separate entries in the pricing cards.
The expanded FAQ with `question: "Which plan is right for me?"`
lists both the CI and GitHub Actions amounts independently in its `answer`.
The [CI overview][ci] also says:

```text
https://depot.dev/docs/ci/overview.md:257
Plans also include container build minutes and GitHub Actions runner minutes.
```

The quoted text omits the source's Markdown link syntax.
It identifies additional product entitlements,
not an exchange mechanism.
Unused entitlement in one product receives no credit against another in the forecast.

## Native CI normalization

The current CI overview at line 245 explicitly bases included minutes on the smallest sandbox,
2 CPUs and 8 GB.
At line 263,
its example states that 4 CPUs consume two included minutes per elapsed minute.
Its sizing tables publish both the per-second rate and plan-minute multiplier.

For either architecture,
the standard 2-CPU sandbox is USD 0.0001/second,
equivalent to USD 0.006/elapsed minute and multiplier 1.
An 8-CPU sandbox is USD 0.0004/second,
equivalent to USD 0.024/elapsed minute and multiplier 4.
Native CI bills by the second with no one-minute minimum.
The pricing tab's shorter phrase,
`consumed at the vCPU multiple rate`,
does not override the explicit 2-CPU baseline.

The additional 18,000 native CI base minutes in Startup have a USD 108 list-rate equivalent.
That comparison concerns native CI alone.

## GitHub Actions normalization and mixed operating systems

The current [runner catalog][runners] defines billed minutes as elapsed minutes times
the catalog's minutes multiplier,
then subtracts the plan's included minutes.
Keep each class's cash rate separately:

- Linux 2 CPUs:
  USD 0.006/elapsed minute,
  multiplier 1,
  USD 0.006 per normalized minute.
- Windows 2 CPUs:
  USD 0.008/elapsed minute,
  multiplier 2,
  USD 0.004 per normalized minute.
- Windows 4 CPUs:
  USD 0.016/elapsed minute,
  multiplier 4,
  USD 0.004 per normalized minute.

The named GitHub Actions allowance is not separate Linux and Windows balances.
For a mixed workload,
sum each class's elapsed minutes times its multiplier for allowance consumption.
Public wording does not decide which class receives the included-minute credit when rates differ.
Do not multiply all remaining normalized minutes by USD 0.006.

For example,
1,000 elapsed minutes each of default Linux and Windows consume 3,000 normalized minutes.
With a 2,000-minute allowance,
a linear class-credit model leaves USD 4 to USD 6 of overage,
depending on how the included units are apportioned.
That is a conditional forecast range,
not the observed billing algorithm or an invoice.

## macOS correction and remaining limit

The earlier categorical research statement that macOS is excluded from included GitHub Actions minutes is withdrawn.
The current runner catalog supplies USD 0.08/elapsed minute but no macOS multiplier.
The public pricing tab broadly says included minutes apply to `any machine type`.
The current runner overview explicitly restricts macOS eligibility to Startup and Business.
Those statements establish the tier boundary,
but do not settle macOS normalized debit or credit treatment.

The uncapped retained search covered all deployed documentation pages and the captured context/supplemental pages.
It found no deciding current macOS allowance-exclusion clause.
The owned platform note's `Live plan eligibility and autoscaling boundary`
and the capabilities note's `Plan and independent usage pools`
describe Developer tier ineligibility;
their current macOS paragraphs did not assert an allowance exclusion.
Their new reconciliation links make this correction explicit beside that separate tier statement.
The withdrawn claim also appeared in the researcher's earlier cross-agent pricing framing.

A conservative forecast may assign no macOS allowance credit while this remains unresolved.
It must label that as an assumption.
Do not claim that any Startup macOS run necessarily makes total monthly charges exceed USD 200.
Developer's lack of macOS eligibility remains unchanged.

## Builder, storage and Tests meters

The live `Container Builds` tab publishes default 16-CPU/32-GB builders at
USD 0.04/elapsed minute with multiplier 1.
The 32-CPU/64-GB rate is USD 0.08 with multiplier 2;
64-CPU/128-GB is USD 0.16 with multiplier 4.
Larger builders require Startup or Business.
Both the pricing page and build overview bill per minute tracked per second.

The [build-parallelism guide][build-parallelism] establishes separate concurrent native builders per architecture.
The autoscaling guide publishes the regular per-minute rate for additional builders.
However,
the inspected public documents do not state an explicit multi-platform debit equation.
The CLI's `pkg/helpers/buildlist.go` reports `finishedAt - createdAt` as build duration,
which is a request envelope rather than an architecture-specific usage counter.
Its machine source sends health reports separately per architecture,
but does not expose the private billing calculation.
No architecture-specific billed interval was observed.

For conservative reservations,
model separate builder intervals rather than crediting parallelism as reduced aggregate compute.
When only a request envelope is retained,
multiply it by the maximum admitted architecture-builder count.
Keep this model assumption separate from a proven invoice-debit rule.

The earlier BuildKit campaign correctly reserved two builders at USD 0.04/minute each.
Its retained 403,938 milliseconds are container-build request windows,
not summed per-architecture builder intervals.
Multiplying every request window by the maximum two builders produces the conservative USD 0.538584 proxy.
This note's first publication incorrectly treated that as a USD 0.08 single-builder assumption
and proposed USD 0.269292 as a repricing.
That interpretation is withdrawn.
The original campaign model remains unchanged.
This reservation is not an invoice or an observation of each architecture's billable lifetime.

The expanded `Am I charged for build usage and storage?` FAQ explicitly defines one combined total for
container build cache,
Depot Cache and Depot Registry.
Those products share the storage entitlement.
Extra storage costs USD 0.20/GB/month using the month's average storage,
sampled hourly.
Repeated storage rows in product overviews do not create extra 25-GB pools.
Standard registry CDN transfer is free;
Fast CDN costs USD 0.10/GB from its first GB.
No Fast CDN setting was changed.

The expanded `How are test results billed?` FAQ grants both Developer and Startup
1,000,000 passing results/month.
Extra passing results cost USD 0.000001 each.
Failed,
errored and skipped results are free.
Do not create a second Tests allowance merely because reports can originate from either CI product.
The identity adapter's 136 retained passing rows have a USD 0.000136 list-rate result proxy before allowances;
this is separate from its compute reservation and is not an observed charge.

## SDK, agents, caps and plan comparison limits

The current [Sandbox SDK reference][sdk] says generic SDK sandboxes are billed per vCPU-second
at the Depot CI compute rate.
It does not say they consume the monthly native CI entitlement.
The [remote agent overview][agents] separately publishes USD 0.01/minute
and no included usage for remote AI agents.
The published rate relationship does not establish shared credit eligibility.

The captured public `depot/core/v1/usage.proto`
at `01dc47fdd504fe88697aec8b3033ce887588d302`
has separate build,
GitHub Actions,
storage and agent-sandbox output families.
It exposes no included-credit apportionment algorithm.
The organization-list schema supplies identity fields,
not subscription entitlements.
Neither service was called.

The pricing FAQ says a reached usage cap stops builds.
The current [container-build quickstart][build-quickstart] names a `Container Build Minutes` limit,
and the current [GitHub runner troubleshooting guide][runner-troubleshooting]
separately names a `GitHub Actions minutes limit`.
The claim that only container builds have a documented minute-cap control is retracted.
These are separately documented product controls;
their configured values and runtime enforcement were not inspected or exercised.
These sources do not specify a single aggregate dollar cap covering these separate meters,
SDK sandboxes,
storage and both CI products.
No such automatic USD 200 guarantee follows from the public wording or this local ledger.

Developer remains the settled intended baseline.
The subscription difference is USD 180/month,
while the extra native CI entitlement has a USD 108 list-rate equivalent.
That native-CI-only comparison does not prove Developer is always cheaper across products.
Startup also adds 4,500 default build minutes,
18,000 named GitHub Actions minutes,
and 225 GB of the shared storage entitlement.
The passing-result allowance is unchanged.
The parent-owned [combined model](depot-expanded-ci-cost-model.md)
must apply each entitlement to its own admitted workload and meter.

[ci]: https://depot.dev/docs/ci/overview
[runners]: https://depot.dev/docs/github-actions/runner-types
[sdk]: https://depot.dev/docs/api/sandbox-sdk-reference
[agents]: https://depot.dev/docs/agents/overview
[build-parallelism]: https://depot.dev/docs/container-builds/build-parallelism
[build-quickstart]: https://depot.dev/docs/container-builds/quickstart
[runner-troubleshooting]: https://depot.dev/docs/github-actions/troubleshooting
