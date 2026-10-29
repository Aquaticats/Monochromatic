import {
  BYTES_PER_GIB,
  BYTES_PER_MIB,
} from '@monochromatic-dev/module-const/ts';

import type { Band, } from './types.ts';

/**
 Default ceiling on total bytes any single probe may download before it aborts
 cleanly (folding into a wider range), keeping the tool cheaper than the clone
 it estimates.
 */
export const DEFAULT_MAX_PROBE_BYTES: number = BYTES_PER_GIB;

/**
 Default wall-clock budget, in seconds, across all probes.
 */
export const DEFAULT_MAX_PROBE_SECONDS = 60;

/**
 Default cap on how many commits the deepen probe may walk back.
 */
export const DEFAULT_MAX_DEEPEN_COMMITS = 256;

/**
 Default cap on objects a partial clone may receive before aborting.
 */
export const DEFAULT_MAX_PARTIAL_OBJECTS = 500_000;

/**
 Default size above which the local pack-objects measurement is skipped in
 favour of the cheap `count-objects` size-pack proxy, since delta-compressing
 everything costs repack-level CPU and memory.
 */
export const DEFAULT_MAX_PACK_BYTES: number = 2 * BYTES_PER_GIB;

/**
 Commits requested per `git fetch --deepen` step.
 */
export const DEEPEN_STEP_COMMITS = 16;

/**
 Number of deepen steps taken before extrapolating.
 */
export const DEEPEN_STEPS = 4;

/**
 Threshold (in objects) above which the tree:0 commit-count partial clone is
 skipped, since an enormous commit graph would make even commits-only heavy.
 */
export const COMMIT_COUNT_OBJECT_GUARD = 200_000;

/**
 Milliseconds per second, for converting the seconds budget to a deadline.
 */
export const MS_PER_SECOND = 1_000;

/**
 Smallest pack a single empty commit history can plausibly add, used to floor
 marginal-bytes-per-commit estimates away from zero.
 */
export const MIN_MARGINAL_BYTES = 64;

/**
 Default metric contract line stating numerator and denominator.
 */
export const METRIC_DEFAULT =
  'shallow=clone --depth 1 (default branch) vs full=clone (all branches+tags)';

/**
 Metric contract line under `--default-branch-only`.
 */
export const METRIC_DEFAULT_BRANCH_ONLY =
  'shallow=clone --depth 1 (default branch) vs full=clone (default branch, full history)';

/**
 Scope line, always printed so object-db size is never conflated with a full
 checkout or network cost.
 */
export const SCOPE =
  'git object database only; excludes working tree, submodules, and Git LFS payloads';

/**
 Multiplier prior interval for the last-resort estimator, expressed as a band
 around the shallow tip size. Wide on purpose; lowest confidence.
 */
export const PRIOR_MULTIPLIER: Band = {
  lo: 1.2,
  point: 6,
  hi: 30,
};

/**
 Lower MiB count for the no-tip prior band.
 */
const PRIOR_ABSENT_LO_MIB = 1;

/**
 Central MiB count for the no-tip prior band.
 */
const PRIOR_ABSENT_POINT_MIB = 50;

/**
 Upper MiB count for the no-tip prior band.
 */
const PRIOR_ABSENT_HI_MIB = 500;

/**
 Wider multiplier band used when even the tip size is unknown, so a first
 snapshot can still emit a prior-only range rather than nothing.
 */
export const PRIOR_ABSENT_TIP_BYTES: Band = {
  lo: PRIOR_ABSENT_LO_MIB * BYTES_PER_MIB,
  point: PRIOR_ABSENT_POINT_MIB * BYTES_PER_MIB,
  hi: PRIOR_ABSENT_HI_MIB * BYTES_PER_MIB,
};
