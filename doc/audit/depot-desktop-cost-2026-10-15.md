# Desktop interaction cost overlay for the Depot trial

## Result and scope

The subsequent successful desktop run took 410 seconds and passed 117 checks,
with no failures and one uninjectable check.
Its separate [recovery replay](depot-desktop-recovery-2026-10-15.evidence.json)
gives Developer USD 160.28 for every affected revision with container validation retained,
USD 149.33 with superseded cancellation,
and USD 23.38 for daily-latest Depot allocation.
Startup's corresponding totals are USD 205.71,
USD 200,
and USD 200 if overage is allowed.
Replacing the separate container workload gives Developer USD 135.28,
USD 126.83,
and USD 23.38 respectively;
the coverage-equivalence limitation still applies.
The portal correction changes the input,
so this is a new scenario rather than a timing-improvement estimate.
The original failed-run measurement and calculations are retained in this document.

### Retained failed-run scenario

Adding the measured desktop interaction job to every affected revision produces a
**USD 166.56 monthly Developer subtotal** when the separate desktop container build remains.
Replacing that container workload with the interaction job produces **USD 141.55**.
Startup reaches **USD 211.98** in either case because the combined runner demand exceeds its allowance.
These are compute scenarios for the measured subset,
not a complete bill or authorization to migrate production.
[Calculation evidence](depot-desktop-cost-2026-10-15.evidence.json)

The priced desktop job failed one assertion.
All seven sessions reached terminal summaries:
116 checks passed,
one failed,
and one was not injectable.
The result prices an observed execution,
not a validated complete suite.
[Measured job](https://github.com/Aquaticats/Monochromatic/actions/runs/38831098294/job/116749813259),
[retained session evidence](depot-desktop-interaction-2026-10-15.evidence.json)

## Measured input

GitHub's job API records source `aa25cce3386f50873acb1921892d5a730e042b29`,
runner `depot-ubuntu-24.04-16`,
start `2026-10-16T00:43:27Z`,
and completion `2026-10-16T00:50:48Z`.
The full job took **441 seconds**,
or **7.35 elapsed minutes**.
The interaction step took 199 seconds;
substituting that step for the full interval would omit checkout,
dependencies,
application and compositor builds,
and job overhead.
[Job API](https://api.github.com/repos/Aquaticats/Monochromatic/actions/jobs/116749813259)

Depot's sixteen-CPU Linux runner consumes eight base runner minutes per elapsed minute.
This attempt therefore represents **58.8 base runner minutes**,
worth **USD 0.3528** at the overage rate before included usage.
Depot documents second-based runner accounting without a one-minute minimum.
The API duration remains a usage proxy rather than an independently retrieved Depot invoice.
[Runner multiplier and rates](https://depot.dev/docs/github-actions/runner-types),
[runner billing](https://depot.dev/docs/github-actions/overview#pricing)

The seven-day input contains 59 proposed `musicDesktop` arrivals:
4,
3,
3,
12,
15,
6,
and 16 on October 8 through October 14 UTC respectively.
Those are selected changes from the observed push stream,
not desktop jobs that actually ran during that week.
The projection repeats this mix over 30 days.
[Change-stream selection](depot-change-stream-2026-10-15.md),
[selected events and source hashes](depot-desktop-cost-2026-10-15.evidence.json)

## Keep additional work separate from replacement

The baseline already prices a separate desktop container build.
The interaction job builds its own desktop executable and compositor inside its 441-second interval.
The additional-work scenario retains both executions.
The replacement scenario removes only the baseline's desktop container minutes,
then adds the complete interaction job.
It does not subtract the interaction job's native build steps a second time.
[Baseline workloads](depot-expanded-cost-2026-10-15.md#what-the-replay-measures),
[job steps and scenario inputs](depot-desktop-cost-2026-10-15.evidence.json)

Replacement is an accounting option,
not proven coverage equivalence:
the interactive runner job does not establish the same container artifact contract.
Artifact handoff and packaged-container validation were not exercised by this substitution.
The evidence retains the exact baseline rows and source hash,
so later edits to the broader cost report cannot silently change these totals.
[Replacement definition and retained baseline](depot-desktop-cost-2026-10-15.evidence.json)

## Monthly scenarios

Each subtotal includes the existing measured audio,
logger,
Android host,
and Android APK workloads.
Plan allowances are applied once to each product's combined usage.
Developer includes 2,000 runner minutes and costs USD 20;
Startup includes 20,000 and costs USD 200.
Runner overage is USD 0.006 per base minute.
Container and Depot CI allowances remain separate.
[Plan allowances and overage](https://depot.dev/pricing),
[baseline snapshot and formulas](depot-desktop-cost-2026-10-15.evidence.json)

### Every affected revision

The desktop overlay consumes `59 * 441 / 60 * 8 * 30 / 7 = 14,868` base runner minutes.
Together with the baseline's 7,128 APK runner minutes,
the total is **21,996 runner minutes per modeled month**.
Keeping the separate container workload gives Developer **USD 166.56**;
replacing it gives **USD 141.55**.
Startup is **USD 211.98** for either treatment.
[Every-affected calculation](depot-desktop-cost-2026-10-15.evidence.json)

### Cancel unfinished superseded checks

Immediate cancellation at the next selected desktop arrival shortens ten of 59 modeled attempts.
The desktop total becomes 23,765 consumed seconds for the week,
or **13,580 monthly base runner minutes**.
Combined runner use is **20,457.143 minutes**.
Developer is **USD 154.54** with the container workload retained,
or **USD 132.03** with it replaced.
Startup is **USD 202.74** for either treatment.
This assumes immediate replacement and excludes cancellation delay and reruns.
[Per-attempt cancellation ledger](depot-desktop-cost-2026-10-15.evidence.json)

### Latest affected revision each UTC day

Seven desktop attempts in the sampled week become **1,764 monthly base runner minutes**.
Combined runner use is **2,688 minutes**.
Developer becomes **USD 24.13** and Startup remains **USD 200**,
whether the container workload is retained or replaced,
because either container total remains within the applicable allowance.
This policy allocates only daily selected revisions to Depot;
affected-push validation on GitHub and explicit release or agent-requested runs remain separate work.
[Daily allocation calculation](depot-desktop-cost-2026-10-15.evidence.json)

The every-affected and cancellation cases exceed Startup's included runner allowance.
Their dollar figures assume paid overage is permitted.
If an account hard limit rejects new work at 20,000 minutes,
these are demand estimates that would leave work unstarted,
not completed-work invoices at a capped price.
[Account limit boundaries](depot-expanded-cost-2026-10-15.md#account-snapshot-and-stopping-behavior),
[runner allowance behavior](https://depot.dev/docs/github-actions/overview#pricing)

## Coverage and uncertainty

The latest full trial records:

- Wide:
  75 passed and one not injectable.
- Narrow:
  10 passed.
- Dialog:
  one passed and one failed.
- Auto-play:
  four passed.
- Quit:
  one passed.
- States:
  13 passed.
- Visual:
  12 passed.

The failed `browse/ctrl-shift-o` assertion reports
`no dialog window covered the player (0 pixels differ)`.
The non-injectable `double-shift/repeat` case remains a skip,
not a passed interaction assertion.
The full check records and terminal summaries are retained as a follow-up without replacing the original failure record.
[Session evidence](depot-desktop-interaction-2026-10-15.evidence.json)

The earlier job `116746502593` took 403 seconds and aborted the wide session after 69 passing assertions;
its dialog check also failed.
The later fixture repair allowed the wide session to complete,
but the jobs have different inputs and do not measure unchanged-input timing spread.
The 441-second scenario therefore has no repeated-run confidence interval.
[Original and follow-up observations](depot-desktop-interaction-2026-10-15.evidence.json)

Dialog repairs may change duration.
The trial does not measure physical GPU or audio hardware,
the remaining platform and browser suites,
other-package execution,
device tests,
release validation,
agent experiments,
reruns,
storage,
transfer,
or taxes.
Those exclusions prevent treating either Developer subtotal as the complete expanded workload's cost.
[Scenario limitations](depot-desktop-cost-2026-10-15.evidence.json),
[broader workload exclusions](depot-expanded-cost-2026-10-15.md)

## Verification

The calculation reselects all 59 desktop events from the retained history,
checks daily totals,
uses the job API's full interval,
and parses each initial check result once rather than counting the printed summary table again.
Every session's parsed totals match its terminal summary.
The original interaction record remains unchanged outside its appended `followups` field.
[Source hashes and terminal counts](depot-desktop-cost-2026-10-15.evidence.json)

Independent recomputation checks each scenario's consumed duration,
normalized minutes,
plan charges,
and container removal.
Cancellation controls consume 120 seconds without replacement and 70 with it;
an arrival exactly at completion preserves both completed attempts.
Markdown rendering and JSON formatting are checked before handoff.
[Calculation controls](depot-desktop-cost-2026-10-15.evidence.json)
