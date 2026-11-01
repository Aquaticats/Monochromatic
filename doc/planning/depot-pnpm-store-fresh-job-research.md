# Fresh-job pnpm-store cache research

## Question and workload

Determine whether pnpm-store archive restoration helps the current workspace install
on GitHub-hosted Ubuntu and Depot GitHub Actions Ubuntu runners.
This is separate from the historical semantic-bridge measurement
in [the cache policy](../decision/ci-caching.md#pnpm-store-not-cached).
It is also separate from Cargo artifact caches,
container layers,
prepared snapshots and persistent disks.

The experiment uses the existing semantic-bridge workflow's installation and artifact consumers:
`pnpm install --frozen-lockfile`,
then the semantic plugin build,
host lifecycle test and external-consumer test.
The installer receives an explicitly isolated `--store-dir`;
every measurement begins in a fresh allocation with an empty project store.
Node `26.12.0` and pnpm `12.14.0` match the frozen source's `mise.lock`.

## Experimental boundary

The isolated branch is `research/depot-pnpm-cache-20261018`.
It replaces the dispatchable `music-player-windows.yml` only on that branch
(on `main` that workflow moved into the `test-windows` job of `.github/workflows/music-player-release.yml`
on 2026-10-22).
The workflow has no push or pull-request trigger.
It does not change production routing or publication.

The pools are `ubuntu-24.04` and `depot-ubuntu-24.04-4`.
Record actual CPU,
memory and image versions rather than assuming identical hardware.
Both use the same pinned `actions/cache/restore` and `actions/cache/save` code.
The former uses GitHub's cache service;
the latter uses Depot's transparent Actions cache integration.

Run repeated cache-disabled jobs,
repeated cold jobs with deliberately absent keys,
a cold producer,
then repeated independent warm consumers.
Check the producer manifest,
dependency hashes and runner identity after restoration.
Preserve application-consumer failures as failures,
even when dependency preparation succeeds.
An additional branch probe tests the documented difference in cache branch isolation
without writing to another branch's source or existing cache keys.

Report restore,
install,
save,
artifact-consumer and complete-job intervals separately.
Report run-to-run spread before a speedup claim.
Tool setup,
cache-size guarding and artifact export remain visible in complete-job timing.

## Cache storage and cleanup

Repository-scoped `gh api repos/Aquaticats/Monochromatic/actions/cache/usage`
reported `9,113,717,494` active bytes across `14` caches before this experiment.
Usage had reached `10,843,494,612` bytes by the first dispatch,
because concurrent production caches were added.
The repository storage-limit endpoint returned HTTP `402` with
`Please ensure your account has a valid payment method on file to access this service.`
No payment method or account setting is changed to inspect that endpoint.
The actual repository limit remains unverified.

Remove only the completed native experiment's cache `9109200675`,
whose `942,152,318` bytes belong to
`refs/heads/trial/depot-native-long-path-20261018T035939491Z`.
The retained [native cleanup receipt](../audit/depot-native-worktree-cleanup-2026-10-18.evidence.json)
establishes that branch's experiment ownership.
Record the exact entry before deletion and confirm that identifier is absent afterward.
This is temporary experiment data,
not a production cache.
After that cleanup,
the new producer is blocked if repository usage exceeds `10,000,000,000` bytes.
A temporary compression check limits the single GitHub experiment archive to `500,000,000` bytes
before it can be saved.
The check's temporary file is removed;
the Actions cache stores the original pnpm-store directory.
The final repeated cohort's unique prefix is `research-20261018-pnpm-cache-v3`.

No production or unrelated cache entry may be deleted for this experiment.
After evidence capture,
remove only this experiment's GitHub cache entry by its recorded key and identifier.
Depot's experimental dependency archive remains subject to its documented retention policy.
Do not claim a cache deletion API without verifying it.

## Evidence status

The harness is committed on its isolated branch.
The initial run `39099461826` failed at tool installation,
before dependency setup or cache measurement.
Mise `2026.10.9` reported `No lockfile URL found` for explicitly versioned requests,
although the frozen lock contains their URLs under the configured `latest` specifiers.
The corrected harness requests `mise install node pnpm` under `MISE_LOCKED=1`
and asserts the resulting exact versions before measuring anything.
The [request-binding investigation](../troubleshooting/mise-lock-pins-repo-tool-version.md)
records the deciding source and verified repair.

Corrected cold run `39099920300` passed every job on immutable harness
`f8727149508e091e77799c2b25313f3bb9b40589`.
Each pool used separate allocations for the repeated samples.
Workspace installation,
the plugin build and both existing artifact-consumer tasks passed.
This control uses an explicit cache miss,
not a cache-disabled workflow.
That preliminary cohort did not establish a warm-cache benefit.
The completed comparative result uses a later single-source cohort.

The owned historical cache deletion completed and its identifier disappeared from the cache listing.
Repository usage after cleanup was `9,947,540,746` bytes across `16` entries.
These are separately observed usage snapshots,
not evidence that all other cache state stayed fixed between requests.
The command deleted only identifier `9109200675`;
it did not select production entries by age or key prefix.

The initial seed run `39100149452` stalled in the parent-authored archive-size guard,
before either save step.
Local reproduction with Zstd `1.5.7` identified the harness's unsupported `--output` option;
the supported output spelling is `-o`.
With a larger input,
the compressor's exit left the tar producer blocked because the guard waited for both exits
before checking either result.
This is a research-harness fault,
not a cache-service failure.
Both ordinary and force-cancellation requests were submitted.
GitHub subsequently confirmed both jobs and the workflow `cancelled` at `05:32:36Z`.
This confirms terminal workflow state,
not the VM destruction or billing-stop timestamp.
The final log request returned `log not found: 117594589204`;
the local reproduction establishes the harness defect without recovering the old cloud step's diagnostic.

Corrected source `165e0fb3f7` uses the supported option,
an abortable stream pipeline and child cleanup on failure.
A real 1 MiB fixture produces a 166-byte archive successfully.
The same actual guard with an intentionally invalid compressor option exits with failure promptly
instead of hanging.
The corrected workflow has a 5-minute job limit and a 2-minute archive-guard limit.

The coherent cohort is complete on source `2a155eec4935e5df225cb7a90af573357c3a94dc`.
It repeats cache-disabled and explicit-miss controls before seeding,
then runs fresh warm consumers and the separate branch-isolation probe.
The [verified receipt](../audit/depot-pnpm-fresh-jobs-2026-10-18.evidence.json)
retains the exact source hashes,
terminal job metadata,
step intervals and original artifact-member hashes.
All 16 final-cohort jobs passed;
all 16 downloaded artifact ZIPs matched GitHub's SHA-256 digests.
The initial tool-install and archive-guard failures remain separate retained outcomes.

## Completed comparative result

The workflow name is `Depot fresh-job pnpm store research`.
Its comparative jobs are `pnpm store github off sample 1`,
`pnpm store github off sample 2`,
`pnpm store github warm sample 1`,
`pnpm store github warm sample 2`
and the corresponding jobs with `depot` replacing `github`.
Both pools use `actions/cache` revision `27d5ce7f107fe9357f9df03efb73ab90386fccae`;
GitHub-hosted jobs use GitHub Actions cache,
while Depot runners use Depot's Actions cache integration.

The dependency-preparation boundary is the sum of the restore and install step intervals.
GitHub reports these timestamps at one-second resolution.
This boundary excludes the manifest-verification step,
producer save,
tool setup,
checkout and application consumers.
The independent pnpm-command timer excludes restoration.

- GitHub-hosted `ubuntu-24.04`:
  cache-disabled preparation is 9 to 10 seconds;
  warm restore plus install is 11 to 12 seconds,
  including a 5 to 6 second restore.
  The pnpm command alone falls from 8.075 to 9.177 seconds disabled
  to 4.833 to 5.694 seconds warm.
- Depot `depot-ubuntu-24.04-4`:
  cache-disabled preparation is 3 seconds in both samples;
  warm restore plus install is 4 to 5 seconds,
  including a 2 second restore.
  The pnpm command alone falls from 2.496 to 2.789 seconds disabled
  to 1.864 to 1.881 seconds warm.

All warm installs report 582 reused packages and zero downloads.
The plugin build,
semantic-bridge host consumer and external-package consumer pass in every install-bearing job.
The restored producer has the same source,
lock and tool identities and a different runner name.
Restoration therefore works;
its overhead consumes the observed install-command saving in this workload.
There is no demonstrated net dependency-preparation benefit on either pool.
Keep this pnpm archive disabled by default pending contrary workload-specific evidence.
That recommendation does not apply to compiler caches,
prepared environments or different package-install workloads.

The repeated explicit-miss controls also pass.
Their restore-plus-install boundaries are 8 to 9 seconds on GitHub and 2 to 4 seconds on Depot.
These are a separate control from disabling the action entirely.

Complete jobs take 44 to 46 seconds disabled and 47 to 48 seconds warm on GitHub,
versus 30 to 31 seconds disabled and 19 to 20 seconds warm on Depot.
Depot checkout also changes from 15 to 16 seconds disabled to 3 to 4 seconds warm,
before the restore step.
The complete-job difference therefore does not isolate pnpm-cache benefit.
No cause is assigned to that checkout change.

Each comparative phase has only two repetitions per pool.
All jobs report four logical CPUs,
but GitHub's observations include Intel Xeon Platinum 8573C,
AMD EPYC 7763 and AMD EPYC 9V74 hosts.
Depot reports AMD EPYC 9R45 hosts.
The source is unchanged;
hardware and image equivalence are not established.
The observed ranges include that provider variation and do not establish a universal speed multiplier.

## Producer and isolation result

The final runs are:

- Cache disabled:
  `39101112370`.
- Explicit miss:
  `39101194478`.
- Seed:
  `39101280802`.
- Warm:
  `39101349744`.
- Branch-isolation probe:
  `39101439840`.

Each seed save takes 4 seconds in GitHub's step timestamps.
The experimental compression guard adds 4 seconds on GitHub and 2 seconds on Depot.
Its checked archives are `310713133` and `310733763` bytes respectively,
both under the declared `500000000`-byte bound.
Preparation,
save and storage remain costs even when later consumers hit.

The second branch uses the same source and restore key.
GitHub returns a miss;
Depot returns the first branch's archive and passes its producer-manifest assertions.
Those jobs deliberately skip installation and application consumers.
This verifies the tested branch-access difference,
not untrusted-writer isolation or every repository authorization boundary.

## Cleanup and cost boundary

The final receipt includes every retained parent experiment's Depot job timestamps,
including failed and cancelled attempts.
At the published Linux four-CPU label rate,
the 13 retained Depot jobs yield a USD `0.2242` timestamp-price proxy;
the successful final cohort's Depot jobs contribute USD `0.0446`.
These are not invoices,
actual allowance consumption,
storage-inclusive totals or proof of allocation billing boundaries.

The GitHub seed created cache `9110766939`,
keyed only to this experiment,
with `312091627` recorded bytes.
It was present in the observed listing at `05:43`,
then absent before cleanup.
A second exact key-and-ref query returned zero entries.
No deletion command was executed for this entry,
and the cause of its disappearance is not established.
The older owned native cache deletion remains a distinct completed action.
No production cache was selected for deletion.
No Depot cache deletion was attempted;
the archive remains subject to documented service retention,
with the actual account policy uninspected.

The isolated worktree was removed after inspecting its ignored generated rules cache
and confirming no root `HEAD`,
`config`,
`hooks`,
`objects` or `refs` entries existed.
Both pushed research branches remain at the exact final source.
All owned parent workflow runs are terminal.
