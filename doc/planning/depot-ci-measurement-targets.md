# Depot CI measurement and relative targets

Status:
work item 8 first measurement.
Samples:
Depot-managed 8x32 run `39199930641` and GitHub-hosted advisory run `39202293518`.

## Metrics

- Job wall time.
- Repository-tool installation time.
- Fast-subset build time.
- Fast-subset unit time.
- Paid compute estimate for the Depot sample.

## Observed samples

Depot-managed 8x32 run `39199930641`:

- job wall:
  10 minutes 12 seconds;
- install repository tools:
  2 minutes 52 seconds;
- build fast subset:
  4 minutes 10 seconds;
- unit fast subset:
  2 minutes 41 seconds;
- build plus unit:
  6 minutes 51 seconds.

GitHub-hosted advisory run `39202293518`:

- job wall:
  21 minutes 28 seconds from advisory core start to completion;
- install repository tools:
  7 minutes 10 seconds;
- build fast subset:
  7 minutes 22 seconds;
- unit fast subset:
  6 minutes 15 seconds;
- build plus unit:
  13 minutes 37 seconds.

Native weekly matrix repeat samples:

- run `dmjf38bcz0`:
  14 minutes 22 seconds;
- run `r2mxbc40rh`:
  14 minutes 39 seconds;
- spread:
  17 seconds,
  about 2 percent.

The native repeats are a different workload from the fast subset.
They measure provider repeatability,
not the fast-subset target.

## Relative targets

These are provisional targets for the same five-owner fast subset:

- Depot build plus unit:
  at most 0.75 times the GitHub-hosted time.
- Depot install tools:
  at most 0.75 times the GitHub-hosted time.
- Depot job wall:
  at most 0.90 times the GitHub-hosted time.

The first sample is below every target:

- build plus unit:
  411 seconds against 817 seconds,
  ratio 0.50;
- install tools:
  172 seconds against 430 seconds,
  ratio 0.40;
- job wall:
  612 seconds against 1,288 seconds,
  ratio 0.48.

## Cost

The Depot 8x32 rate used by the evaluation is USD 0.024 per minute.
At 10.2 minutes the fast-subset sample is about USD 0.245 nominal.
Public GitHub standard compute is USD 0 for this public repository.
The comparison is runtime,
not a claim that Depot is cheaper.

## Limitations

- One sample per provider.
- The providers ran on different dates and runner states.
- The GitHub sample includes the changed-path static step before the build.
  The static step itself had no changed files and took under one second.
- No run-to-run spread was measured.
  These targets are provisional until the weekly matrix supplies repeated samples.
- The complete build and unit profile is not measured because it needs the prepared environment.

## Current decision

Adopt the provisional runtime targets.
Record a second sample when the weekly native matrix completes and revisit only if the ratio
moves outside the target by more than the observed noise.
