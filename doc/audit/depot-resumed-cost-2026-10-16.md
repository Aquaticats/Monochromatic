# Depot expanded execution cost follow-up

## Result and scope

The resumed experiments add measured browser suites,
native feature coverage,
prepared desktop execution,
Depot Tests,
and a useful standalone sandbox experiment.
They support a broader allocation estimate than the initial compilation trial.
They still do not establish the entire account's monthly bill.
The [replay evidence](depot-resumed-cost-2026-10-16.evidence.json)
retains each selected revision,
stage duration,
cancellation allocation,
rate,
and source hash.

Repeating the observed seven-day change stream for a 30-day month gives these measured-workload subtotals:

- Run every affected revision:
  Developer USD **133.33**;
  Startup USD **200.00**.
- Immediately cancel superseded work within its workload:
  Developer USD **125.01**;
  Startup USD **200.00**.
- Run the newest affected revision each UTC day on Depot,
  retaining push validation on GitHub:
  Developer USD **20.00**;
  Startup USD **200.00**.

These include subscription and modeled compute overage,
with the additional charges and exclusions described in this report.
They assume overage is permitted;
hard-start limits can leave work unstarted instead.
No plan change or production routing is implemented.

## What the model charges

The source window is October 8 through October 14,
2026,
with 1,143 observed push events and 1,133 distinct revisions.
Selection follows the retained affected-package and consumer analysis,
not every push for every job.
[Change-stream evidence](depot-change-stream-2026-10-15.md)

- Native audio checks replace the earlier core-only baseline:
  157.920 seconds on 4 vCPUs,
  including the feature matrix and Clippy,
  selected by 68 events.
- Logger build,
  types,
  and unit checks retain their earlier 45.918-second,
  2-vCPU measurement and 50 selected events.
- Android host checks retain 100.149 seconds on 4 vCPUs;
  cloud APK creation retains 231 seconds on a 16-vCPU GitHub runner.
  Both use the 54 selected Android events.
  Device execution remains on the local disposable emulator.
- Logger and filesystem browser checks add 151.586 seconds on 4 vCPUs,
  selected by 50 events.
  This includes the observed WebKit failure;
  a failing result is not free compute.
- DOM browser checks add 117.823 seconds on 4 vCPUs,
  selected by 50 events.
- Prepared desktop execution replaces both the separate container compilation
  and the earlier full-suite GitHub-runner job.
  Each selected desktop revision pays for 301.271 seconds of preparation on 16 vCPUs,
  then one fresh consumer on 2 vCPUs.
  The consumer baseline is the 286.550-second mean of two unchanged-input runs,
  observed at 296.995 and 276.105 seconds.
  The source snapshot must be rebuilt for each selected revision;
  installed base packages are reusable.
  Desktop selection contains 59 events.

The [expanded execution report](depot-expanded-execution-2026-10-16.md)
and [earlier measured baselines](depot-expanded-cost-2026-10-15.md)
provide the job evidence.
The [prepared desktop report](depot-prepared-desktop-2026-10-16.md)
records snapshot contents,
fresh service startup,
interaction checks,
and cleanup.

The desktop pipeline is sequential in the cancellation model.
If a new affected revision arrives during preparation,
the cancelled revision incurs no consumer runtime.
If it arrives during the consumer,
only elapsed consumer time is charged.
Cancellation is instantaneous in this scenario;
actual delivery delay and billing minima remain unmeasured.
An independent control includes cancellation before preparation finishes,
during consumption,
and a complete pipeline.
The new replay also reproduces all 15 workload totals from the earlier model.
The observed consumer spread is 20.890 seconds.
Using either consumer observation instead of their mean gives every-affected Developer subtotals
from USD 133.06 to USD 133.59.
This is observed sample sensitivity,
not a confidence interval for future jobs.

## Published rates and additional usage

[Depot public pricing][pricing] lists Developer at USD 20 per month
with 2,000 CI base minutes,
2,000 GitHub-runner base minutes,
500 container-build minutes,
25 GB storage,
and 1,000,000 passing test results.
Startup costs USD 200 with ten times those compute and storage allowances;
its passing-test allowance remains 1,000,000.
Compute overages are USD 0.006 per CI base minute,
USD 0.006 per GitHub-runner base minute,
and USD 0.04 per container-build minute.

The measured every-affected workload uses 15,759.741 CI base minutes
and 7,128 GitHub-runner base minutes per modeled month.
The daily selection uses 1,857.029 CI base minutes and 924 GitHub-runner base minutes.
The source-specific desktop preparation accounts for most of the additional CI usage.
These numbers do not include unrelated account workloads.

Browser image preparation took 137.525 seconds on 4 vCPUs.
Charging that shared preparation once per month,
weekly,
or daily adds at most USD 0.0275,
USD 0.1179,
or USD 0.8252 in compute overage.
It is charged once per image refresh,
not once per browser consumer.
This measurement starts from the already prepared Node/dependency image;
initial preparation of that base and refreshed desktop package images remains outside this subtotal.

Storage at 25,
50,
100,
or 250 GB average adds USD 0,
5,
15,
or 45 to Developer at the published USD 0.20 per excess GB-month.
All those scenarios remain inside Startup's 250 GB allowance.
They are sensitivities,
not measurements of the account's combined billable storage.
[Public storage pricing][pricing]

## Tests and standalone experiments

The [Depot Test experiment](depot-test-feature-2026-10-16.md)
retrieved 1,848 rows across 14 job attempts,
including 1,834 passing results.
At the published overage rate,
those passing rows have a USD 0.001834 equivalent if no allowance remains.
Failed,
errored,
and skipped results are free;
test execution compute is separate.
The trial's 1,364 vCPU-seconds have a USD 0.0682 compute-overage equivalent.
These are one-time trial observations,
not an additional monthly execution frequency.
[Test pricing][pricing]

Uploading the known passing cases from the modeled native,
browser,
DOM,
and desktop suites would produce approximately 131,661 passing results per modeled month
when every affected revision completes.
That is a named-suite subtotal,
not evidence that the entire account fits the test allowance.
Duplicate ingestion is a verified risk in
[#1046](https://github.com/Aquaticats/Monochromatic/issues/1046).

The [standalone linker experiment](depot-linker-experiment-2026-10-16.md)
used a 1,204.403-second,
2-vCPU sandbox session,
including setup,
interactive pauses,
and artifact retrieval.
Its public compute-overage equivalent is USD 0.1204403.
Repeating that observed session once,
five times,
or twenty times per day for 30 days gives USD 3.61,
18.07,
or 72.26 before allowance attribution and storage.
These frequencies are illustrative;
the trial does not establish how often the user will request comparable experiments.
[Sandbox compute pricing](https://depot.dev/docs/api/overview#sandbox-api)

## Recommendation and remaining uncertainty

Ranking:
Developer with intentional GitHub routing > Developer with broader paid execution > Startup.
Routing ranks first because daily Depot selection fits the measured compute allowances
while preserving push validation on GitHub.
Broader Developer execution ranks ahead of Startup because the measured every-affected subtotal
is USD 133.33 before the explicitly unpriced work,
below Startup's subscription alone.
The ranking remains provisional until the omitted workload and account scopes are priced.
It is not a claim that daily Depot checks provide the same feedback timing as per-push Depot checks.

The account usage endpoint rejects the current CLI user token and requires an organization token.
The user cannot supply that token or a signed-in browser for this check.
They subsequently clarified that further account access was deliberately rejected.
Account inspection is intentionally excluded from trial completion.
The trial therefore cannot verify the complete invoice,
combined storage scope,
standalone sandbox allowance attribution,
or an aggregate spending cap.
The existing per-product hard-start limits are not evidence of an aggregate USD 200 safeguard.
The [account findings](../handover/depot-trial-issues.md#package-reuse-and-account-observability)
retain this access boundary.

Other packages,
remaining native applications,
platform release checks,
fuzzing and conformance,
extra retries,
artifact retention,
taxes,
and production integration remain outside these totals.
No historical equivalent music workflow exists for this source window,
so the report cannot convert the new coverage into a measured monthly latency saving.
The added execution evidence improves allocation planning;
it does not erase those limits.

Independent review found that this follow-up's initial draft used USD 0.004 for Linux runner overage.
The [current runner documentation](https://depot.dev/docs/github-actions/runner-types)
specifies USD 0.006 per base minute.
The corrected model uses that rate and the two-consumer mean;
the earlier cost report already used the correct rate.
The [independent verification](depot-resumed-cost-verification-2026-10-16.evidence.json)
checks every allocated stage,
source hash,
and duration sensitivity.

[pricing]: https://depot.dev/pricing
