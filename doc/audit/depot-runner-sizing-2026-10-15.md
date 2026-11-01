# Depot runner sizing, October 15, 2026

## Answer

The user asked whether a bigger Depot runner would help the music-player build.
The comparison separates runner CPU allocation from Cargo's concurrent compilation jobs.
The user explicitly clarified that the overall migration is not apples to apples.
This controlled build comparison answers allocation efficiency only;
the [trial brief](../planning/depot-ci-trial.md) evaluates the broader desired coverage and workflow.
Yes:
16 CPUs with 16 Cargo jobs averaged 246.6 seconds cold,
compared with 526.7 seconds at four CPUs and four jobs.
That is a 2.14-fold speedup and 280.1 seconds saved per measured cold build.
Keeping Cargo at four jobs on the 16-CPU runner averaged 476.6 seconds,
so increasing compilation concurrency matters substantially.
All configurations passed all ten runtime assertions.
[Main execution](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116503),
[main evidence](depot-runner-sizing-2026-10-15.38822116503.evidence.json)

My recommendation is 16 CPUs and 16 jobs for cold builds that determine completion.
Eight CPUs and eight jobs are a reasonable source-rebuild choice when dependencies are cached.
The 32-CPU allocation saved less additional time than its added cost justified at the user's valuation;
the observed 64-CPU allocations were slower and more expensive than 16 CPUs.
For the separate container product,
use 16 Cargo jobs on the existing default builder before considering a hardware or plan upgrade.
These are recommendations from the trial,
not production configuration changes.
[Large-runner evidence](depot-runner-sizing-2026-10-15.38822116510.evidence.json),
[container evidence](depot-container-concurrency-2026-10-15.evidence.json)

## Measurement method

The trial uses the Alpine release build and production runtime from branch `trial/depot-20261015`.
The GitHub runner matrix executes revision `ef616a2900269c52c61d24f816520504a3d74dd2`.
It compares CPU/job pairs `4/4`,
`8/4`,
`8/8`,
`16/4`,
`16/8`,
`16/16`,
`32/16`,
`32/32`,
`64/16`,
and `64/64`.
Each pair gets one ephemeral runner and two rounds with separate Cargo cache namespaces.
Each round contains a cold build,
two unchanged repetitions,
a CLI-name source change,
and an unchanged repetition of the changed source.
Every exported executable must print the expected version inside the production Alpine runtime.
[Main execution](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116503),
[large-runner execution](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116510)

The original matrices failed because Cargo could accept stale artifacts after BuildKit replayed older source timestamps.
Their failed source-change observations are excluded from the corrected comparison.
The fix hashes copied source content and cleans the local application packages when the content changes,
while preserving registry dependency artifacts.
The [cache investigation](../troubleshooting/cargo-buildkit-source-timestamps.md)
records the failing control and remedy.

Cold means empty Cargo caches for that round.
The second round can reuse the base image and operating-system page cache.
The measurements time the Bake invocation,
including export on the runner;
they exclude workflow setup and cleanup.
Two observations on one runner measure within-runner spread,
not the distribution across independently allocated machines.
The CPU model and memory observations are retained with every configuration.
The 32-CPU jobs reported `AMD EPYC 9R45`;
the 64-CPU jobs reported `Intel(R) Xeon(R) 6975P-C`.
Consequently,
the allocation results cannot isolate core count from processor and host differences.
[Large-runner evidence](depot-runner-sizing-2026-10-15.38822116510.evidence.json)

## Measured runner results

Values are arithmetic means in seconds;
cold ranges show the two observations.
Source rebuilds change the CLI name and rebuild both local packages under the conservative cache fix.
They do not represent every possible source edit.
The linked evidence contains every sample,
unchanged repetition,
hardware probe,
and job interval.
[Main evidence](depot-runner-sizing-2026-10-15.38822116503.evidence.json),
[large-runner evidence](depot-runner-sizing-2026-10-15.38822116510.evidence.json)

- Four CPUs/four jobs:
  cold 526.7 seconds,
  range 510.7 to 542.7;
  source rebuild 204.0 seconds.
- Eight CPUs/four jobs:
  cold 510.5 seconds,
  range 510.47 to 510.50;
  source rebuild 216.8 seconds.
- Eight CPUs/eight jobs:
  cold 320.4 seconds,
  range 314.3 to 326.5;
  source rebuild 142.1 seconds.
- Sixteen CPUs/four jobs:
  cold 476.6 seconds,
  range 466.8 to 486.4;
  source rebuild 203.3 seconds.
- Sixteen CPUs/eight jobs:
  cold 303.3 seconds,
  range 293.9 to 312.7;
  source rebuild 142.3 seconds.
- Sixteen CPUs/sixteen jobs:
  cold 246.6 seconds,
  range 245.6 to 247.6;
  source rebuild 122.5 seconds.
- Thirty-two CPUs/sixteen jobs:
  cold 224.2 seconds,
  range 218.8 to 229.7;
  source rebuild 113.6 seconds.
- Thirty-two CPUs/thirty-two jobs:
  cold 212.0 seconds,
  range 206.6 to 217.3;
  source rebuild 115.2 seconds.
- Sixty-four CPUs/sixteen jobs:
  cold 292.5 seconds,
  range 286.2 to 298.9;
  source rebuild 151.3 seconds.
- Sixty-four CPUs/sixty-four jobs:
  cold 323.4 seconds,
  range 318.1 to 328.7;
  source rebuild 207.4 seconds.

The 16.2-second cold difference between `4/4` and `8/4`
is smaller than the baseline's 32.0-second within-runner spread.
It does not establish a hardware-only improvement.
At sixteen CPUs,
increasing jobs from four to sixteen saved 230.0 seconds cold and 80.8 seconds for the source rebuild.
The unchanged baseline repeats across the full matrix took 0.759 to 1.618 seconds;
they do not justify larger allocations.
[Main evidence](depot-runner-sizing-2026-10-15.38822116503.evidence.json),
[large-runner evidence](depot-runner-sizing-2026-10-15.38822116510.evidence.json)

## Cost and allocation recommendation

Published Linux runner rates are USD $0.003 per allocated CPU-minute.
Build seconds divided by 60,
multiplied by allocated CPUs and USD $0.003,
estimate gross compute usage,
before included allowances and excluding setup.
These are modeled charges,
not invoice entries.
[Depot runner prices](https://depot.dev/docs/github-actions/runner-types)

The user's CAD $8.975/hour valuation and dated CAD $1.4196/USD conversion give approximately USD $6.322/hour.
Combined cost adds that elapsed-time value to gross compute cost only when the build determines relevant completion.
When a required Windows job finishes later,
reducing Linux time may save none of the user's valued elapsed time.
The [trial report](depot-trial-2026-10-15.md)
records the valuation,
exchange-rate source,
and exact-commit platform overlap measurements.

For cold builds,
the full-size job-count configurations have these modeled costs:

- `4/4`:
  USD $0.105 compute,
  USD $1.030 including valued completion time.
- `8/8`:
  USD $0.128 compute,
  USD $0.691 combined.
- `16/16`:
  USD $0.197 compute,
  USD $0.630 combined.
- `32/32`:
  USD $0.339 compute,
  USD $0.711 combined.
- `64/64`:
  USD $1.035 compute,
  USD $1.603 combined.

Ranking for cold-build combined cost:
`16/16` > `8/8` > `32/32` > `4/4` > `64/64`.
Sixteen beats eight because the 73.8-second saving outweighs the extra USD $0.069 compute.
Eight beats thirty-two because the additional 108.5-second saving does not repay USD $0.211 extra compute.
Thirty-two beats four because its 314.8-second saving outweighs USD $0.234 extra compute.
Four beats sixty-four because the latter's 203.3-second saving does not repay USD $0.930 extra compute.
The intermediate job-count variants do not beat `16/16` in the complete cold ranking.
[Calculated costs and raw samples](depot-runner-sizing-2026-10-15.38822116503.evidence.json),
[larger allocation costs](depot-runner-sizing-2026-10-15.38822116510.evidence.json)

For the source rebuild,
`8/8` has the lowest point estimate:
USD $0.057 compute and USD $0.306 combined,
versus `16/16` at USD $0.098 compute and USD $0.313 combined.
The combined difference is less than one US cent and smaller than the cost effect of the observed `16/16` spread.
Treat these as an unresolved economic tie,
with eight conserving included minutes and sixteen finishing about 19.6 seconds sooner.
Neither thirty-two nor sixty-four improves the combined estimate.
When Linux does not determine completion,
omit its elapsed-time value from the calculation;
four CPUs have the lowest measured compute cost for these builds.
[Main evidence](depot-runner-sizing-2026-10-15.38822116503.evidence.json),
[large-runner evidence](depot-runner-sizing-2026-10-15.38822116510.evidence.json)

## Container product distinction

Depot's default container builder has 16 CPUs and 32 GB RAM on every plan.
Larger container builders require Startup or Business.
This is separate from selecting a larger GitHub Actions runner.
[Container builder specifications](https://depot.dev/docs/container-builds/overview)

The initial container trial used four Cargo jobs.
The follow-up measures 16 jobs on the existing default builder,
without changing its allocation or subscription.
It uses two fresh Cargo cache namespaces,
unchanged repetitions,
and the production runtime assertion.
Its elapsed times include downloading the artifact to the local client,
so they are not directly comparable to the runner-local exports.
The [initial trial report](depot-trial-2026-10-15.md)
retains the original four-job observation and its scope.

The 16-job cold builds took 268.558 and 265.417 seconds,
averaging 266.987 seconds with a 3.141-second spread.
Unchanged repetitions took 22.297 to 37.654 seconds including local export.
All six runtime checks passed.
[First cold build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/5s3508wm66),
[second cold build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/8m5xhj3pb9),
[complete container evidence](depot-container-concurrency-2026-10-15.evidence.json)

The original four-job cold observation took 468 seconds,
201 seconds more than this new mean.
That original baseline has no cold repetition and predates the source-content cache fix,
so this is an observed difference rather than a controlled causal speedup estimate.
The repeated 16-job results establish that the default builder can execute this workload with that concurrency.
This probe covers cold and unchanged inputs;
the runner matrix separately verifies repeated source changes.
It supports using the existing builder before paying for a larger container allocation.
[Initial trial](depot-trial-2026-10-15.md),
[container evidence](depot-container-concurrency-2026-10-15.evidence.json)

## Completion and limits

The corrected matrix produced 100 successful checked outputs,
including both source changes in every configuration.
The original failed matrices and the intervening mise setup failures remain separate historical evidence.
The [cache investigation](../troubleshooting/cargo-buildkit-source-timestamps.md)
records the now-passing full application verification.
The report's two-sample estimates support this allocation recommendation;
they do not establish independent-host reliability,
monthly usage,
or a reason to move from Developer to Startup.
It also cannot decide whether newly enabled checks and experiments justify additional spending.
That decision concerns the expanded target workload,
not parity with today's GitHub jobs.
The [full trial report](depot-trial-2026-10-15.md)
retains broader coverage and account-access limits.
