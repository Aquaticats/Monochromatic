# Pretrial change stream for expanded coverage replay

## Result and interpretation

The seven complete UTC days from October 8 through October 14,
2026 contain 1,143 `forbidden-strings` workflow runs triggered by pushes to `main`,
representing 1,133 distinct head commits.
These workflow records provide the replay's observed arrival stream;
the interval does not include the October 15 trial workload.
The [evidence file](depot-change-stream-2026-10-15.evidence.json)
retains every observed event,
its tree changes,
its classification,
API query provenance,
and pagination validation.

There are no Windows,
Alpine,
or release music-player workflow runs in this interval.
Those workflows were introduced on October 15 UTC.
The empty historical interval is absent observed coverage,
not a latency improvement or evidence that the expanded target is inexpensive.
The prior-week stream supports a proposed coverage-demand replay;
it cannot establish music-player CI time saved against that week's production jobs.
[Unfiltered workflow controls and introduction commits](depot-change-stream-2026-10-15.evidence.json)

## Source and completeness

The source is GitHub's workflow-run API,
queried with `branch=main`,
`event=push`,
and daily `created` windows for the
[`forbidden-strings` workflow](../../.github/workflows/forbidden-strings.yml).
That workflow runs on main pushes without a path filter.
Its file is byte-identical at the preceding boundary and the final sampled head.
Every daily response was paginated with `per_page=100`;
returned and unique run counts equal each query's `total_count`.
No daily query exceeded the API's 1,000-result filtered-search limit,
so no subday partition was needed.
The collected run numbers are consecutive across the entire observed interval.
[API semantics][runs-api],
[retrieval ledger](depot-change-stream-2026-10-15.evidence.json)

The preceding boundary is run `37704307262`,
created October 7 at `23:48:15Z`,
at head `e2771102e4c82449294c6659b987e0f772af0106`.
It supplies the first observed tree comparison;
it is not counted among the seven days.
Every one of the 1,134 distinct required local commit objects was present before differencing.
No transition moves to a head outside the previous head's descendant history.
The API collection uses 42 recorded queries,
including completeness controls and current branch-rule reads.
[Boundary run](https://github.com/Aquaticats/Monochromatic/actions/runs/37704307262),
[retrieval ledger](depot-change-stream-2026-10-15.evidence.json)

The evidence is complete relative to the workflow history returned by GitHub.
It cannot recover deleted workflow runs or pushes for which GitHub never created a run.
The timestamp is workflow `created_at`,
not the exact push-event timestamp or a commit's author/committer time.
Distinct runs with the same unchanged head remain distinct observations;
the classification identifies those transitions instead of silently charging another build.
[API semantics][runs-api],
[event records](depot-change-stream-2026-10-15.evidence.json)

## Observed arrivals and direct changes

Observed push-run counts by UTC day are:

- October 8: 74.
- October 9: 131.
- October 10: 36.
- October 11: 62.
- October 12: 77.
- October 13: 149.
- October 14: 614.

These counts sum to 1,143.
The retained scan conclusions are 1,031 successes,
109 cancellations,
and 3 failures.
Those are the scan workflow's conclusions,
not the result of a hypothetical expanded suite.
[Daily counts and scan conclusions](depot-change-stream-2026-10-15.evidence.json)

Direct changes appear in overlapping groups:

- Music-player desktop application or its end-to-end harness:
  30 events.
- Android application:
  33 events.
- Shared audio core or its benchmark sidecar:
  7 events.
- Logger or its fuzz sidecar:
  no direct events.
- Root/repository configuration:
  44 events.
- Other packages:
  496 events.
- Documentation-classified paths only:
  545 events.
- No tree change from the preceding observed head:
  10 events.

An event can affect more than one runtime group,
so the group counts must not be added to obtain push count.
Documentation-only and unchanged-tree observations are separate classifications.
The evidence's `method.selector` records what counts as documentation and root configuration;
it is a reviewable selection policy,
not proof that documentation never affects a package's behavior.
[Direct-group definitions and per-event paths](depot-change-stream-2026-10-15.evidence.json)

The tree comparison uses consecutive observed main heads,
including deletions and without rename collapsing.
It does not use first-parent commit counts as a substitute for pushes.
100 observations introduce more than one reachable commit relative to the preceding head.
A pushed merge can therefore aggregate changes from several commits into one arrival.
Each event retains `introducedCommitCount`,
`removedCommitCount`,
`previousObservedHeadSha`,
and `changedPaths` for inspection.
[Transition evidence](depot-change-stream-2026-10-15.evidence.json)

## Proposed affected-check mapping

The projected groups implement a conservative interpretation of the
[expanded coverage proposal](../planning/depot-expanded-coverage.md).
They are not claims that these checks executed historically.
Shared audio changes select both applications;
application integration changes also select the core suite.
Compositor changes select desktop integration.
Root mise/toolchain inputs select native,
Android,
and TypeScript groups.
Root TypeScript dependency/configuration/task-utility changes select TypeScript and browser groups.
The separate direct classifications allow a later replay to compare narrower validated selection.
[Selector and per-event reasons](depot-change-stream-2026-10-15.evidence.json)

Under that proposed selector,
the week's candidate group counts are:

- Music-player desktop: 59.
- Android: 54.
- Shared audio core: 68.
- Representative logger host checks: 50.
- Browser groups: 50.
- Other packages: 523.

There are 21 events changing root tool inputs and 33 changing root TypeScript inputs;
those sets can overlap.
The logger count comes from shared-input or consumer selection despite no direct logger edits.
It represents a candidate representative suite,
not all TypeScript execution required by those changes.
The `otherPackage` count must not be priced as one uniform job:
affected packages and their tasks differ.
[Projected counts](depot-change-stream-2026-10-15.evidence.json)

Each event also carries the reverse-consumer closure from the repository's package manifests
at the exact `snapshotCommit` recorded in the evidence.
That closure includes declared dependency,
development dependency,
peer dependency,
and optional dependency edges.
It is current-manifest evidence for the proposed future workflow,
not a reconstruction of every historical package graph.
Undeclared source imports and renamed historical packages remain a selection limitation.
The shared Rust/audio edges are recorded explicitly rather than inferred from npm manifests.
[Manifest snapshot and event closures](depot-change-stream-2026-10-15.evidence.json),
[desktop dependency](../../package/music-player/desktop-app/Cargo.toml),
[Android dependency](../../package/music-player/android-app/rust/Cargo.toml)

## Why the platform baseline is empty

Each of the three workflow endpoints returned zero runs for every requested day.
An independent unfiltered query then returned each workflow's complete history,
with returned counts equal to total counts:

- Windows:
  33 runs;
  earliest run created October 15 at `01:54:11Z`.
  [Earliest Windows run](https://github.com/Aquaticats/Monochromatic/actions/runs/38668138999)
- Alpine:
  13 runs;
  earliest run created October 15 at `03:13:45Z`.
  [Earliest Alpine run](https://github.com/Aquaticats/Monochromatic/actions/runs/38675162941)
- Release:
  29 runs;
  earliest run created October 15 at `00:03:20Z`.
  [Earliest release run](https://github.com/Aquaticats/Monochromatic/actions/runs/38657771731)

Git history independently places their introduction on October 15 UTC.
The commits display October 14 in their recorded `-04:00` timezone,
which is why a local-date reading can accidentally include them in the wrong daily interval.
The evidence preserves those original timestamp strings and the API's UTC creation timestamps.
[Introduction and query controls](depot-change-stream-2026-10-15.evidence.json)

There are consequently no historical job records or exact-head platform pairs for this requested week.
The daily union of the empty music-job set is recorded as zero,
with `latencyComparisonAvailable=false`.
No run's `updated_at` is used as an execution end time.
No job endpoint was needed because the run set was empty.
For a nonempty replay baseline,
job `started_at`/`completed_at` records,
including rerun attempts,
would be necessary.
[Job API semantics][jobs-api],
[empty interval records](depot-change-stream-2026-10-15.evidence.json)

October 15 platform observations remain a separate reference in the
[trial report](depot-trial-2026-10-15.md#monthly-plan-scenarios).
They must not be silently inserted into the pretrial week or presented as seven days of comparable runtime history.
Historical durations also belong to their actual revisions,
hardware,
cache state,
and executed checks;
they are not invoices or measured costs of the proposed broader coverage.

## Current completion requirements

The current branch API reports protection enabled but empty required-status-check `contexts` and `checks` arrays.
The effective branch-rule endpoint returns a Copilot review-on-push rule,
with no required-status-check rule in that response.
These current reads do not establish historical enforcement settings.
The future coverage proposal's required work therefore must be specified explicitly;
it cannot be inferred from an existing GitHub required-check list.
[Branch response and effective rules](depot-change-stream-2026-10-15.evidence.json),
[branch-rule API][rules-api]

The relevant current workload definitions remain the
[expanded inventory](../planning/depot-expanded-coverage.md),
the [Windows workflow](../../.github/workflows/music-player-windows.yml),
the [Alpine workflow](../../.github/workflows/music-player-alpine.yml),
and the [release workflow](../../.github/workflows/music-player-release.yml).
The inventory distinguishes unit/type/lint,
native,
browser,
desktop interaction,
Android device,
and packaged-consumer completion.
Its missing tests and unmeasured environments must remain visible when replaying candidate triggers.

## Inputs for the monthly replay

Use the chronologically ordered `events` with their observed timestamps,
revision identifiers,
direct groups,
projected groups,
and package closures.
Apply task-specific measured durations and resource sizes separately.
Deduplicate shared tasks selected by several changed groups within one event.
Treat unchanged trees,
documentation-only changes,
and explicit releases according to their respective policies.
Model superseding eligible intermediate validation separately from release runs,
and state the assumed cancellation and scheduling policy.
[Replay inputs](depot-change-stream-2026-10-15.evidence.json),
[agreed trial boundaries](../planning/depot-ci-trial.md)

The seven-day distribution is a historical sample with a large October 14 contribution.
A repeated-week monthly scenario is a projection,
not a measured future month.
Report sensitivity to that concentration,
broader test authoring,
new environments,
reruns,
storage,
and agent-requested experiments.
Neither the observed push count nor a direct music-player count alone prices the repository-wide target.

## Verification

Validation matched every retained run ID against the collected API rows,
recounted every daily direct/projected group,
checked chronological head continuity and consecutive run numbers,
and checked all 28 daily query totals against their returned pages.
An independent `git diff --name-status --no-renames` extraction matched 21 selected tree transitions,
covering daily boundaries,
each observed direct category,
and a push introducing multiple commits.
The three nonempty unfiltered workflow histories provide controls for the empty date-filtered results.
Interval-union controls returned the expected durations for overlapping,
disjoint,
and contained intervals,
as well as zero for the empty set.
These controls validate calculation behavior;
they do not create missing historical job observations.

The evidence JSON was formatted with the repository's dprint configuration.
The Markdown report was rendered through Pandoc's CommonMark reader,
with its lists and links inspected in the resulting HTML.

[runs-api]: https://docs.github.com/en/rest/actions/workflow-runs#list-workflow-runs-for-a-workflow
[jobs-api]: https://docs.github.com/en/rest/actions/workflow-jobs#list-jobs-for-a-workflow-run
[rules-api]: https://docs.github.com/en/rest/repos/rules#get-rules-for-a-branch
