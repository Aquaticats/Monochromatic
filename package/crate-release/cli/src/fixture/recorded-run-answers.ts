/**
 Answers of `gh run view --json=createdAt,status,conclusion,jobs` recorded from real runs of this repository, so the
 tests that decide what a run's own answer means read the shapes GitHub really prints instead of a guess.

 Each file is the exact text `gh` printed, prettified; the run it came from is named beside it, with the date of the
 recording, so the fixture can be recorded again when the API's shape changes.

 The two release runs are the pair that showed the defect this package now guards: `42934268993` really failed, at a
 frozen `pnpm install`, and `42935168319` completed with success while a `gh run watch --exit-status` that was following
 it exited 1. The `cargo-publish` run is the one that built the archives and made the GitHub release of the version
 that release run dispatched.
 */

import cargoPublishRunPassed from './cargo-publish-run-passed.json' with { type: 'json' };
import releaseRunFailed from './release-run-failed.json' with { type: 'json' };
import releaseRunPassed from './release-run-passed.json' with { type: 'json' };
import runStillGoing from './run-still-going.json' with { type: 'json' };

/**
 `crate-release.yml` run `42935168319` of `monochromatic-git-policy-cli` 0.6.1-alpha.9: it completed with success at
 2026-11-11T01:00:32Z, and its `publish` job started the `cargo-publish` run below. Recorded 2026-11-11.
 */
export const recordedReleaseRunPassed: unknown = releaseRunPassed;

/**
 `crate-release.yml` run `42934268993` of the same release: it completed with failure at 2026-11-11T00:49:35Z, because
 the `release` job failed at its `Install Node, pnpm and the workspace dependencies` step. Recorded 2026-11-11.
 */
export const recordedReleaseRunFailed: unknown = releaseRunFailed;

/**
 `cargo-publish.yml` run `42935721432`, which the passing release run's `publish` job started: it completed with success
 after 57 jobs, three of them the archive build and publish of this crate, and the GitHub release appeared at
 2026-11-11T01:07:58Z. Recorded 2026-11-11.
 */
export const recordedCargoPublishRunPassed: unknown = cargoPublishRunPassed;

/**
 A workflow run recorded while it was still going: `nodejs/node` run `42952699825` of its `Test Linux` workflow,
 recorded 2026-11-11T03:57:20Z. Both of its jobs report `in_progress`, whose steps are a mix of `completed`,
 `in_progress` and `pending`, and its conclusion is the empty text `gh` prints for a run that has not ended.

 It comes from another repository because no run of this repository was going while the fixture was recorded; nothing
 in the answer depends on which repository ran it, and a second recording of this repository's own run can replace it
 verbatim.
 */
export const recordedRunStillGoing: unknown = runStillGoing;
