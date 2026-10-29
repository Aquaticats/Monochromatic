import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import {
  DEEPEN_STEP_COMMITS,
  DEEPEN_STEPS,
  DEFAULT_MAX_DEEPEN_COMMITS,
  MIN_MARGINAL_BYTES,
} from './constants.ts';
import { clamp, } from './format.ts';
import { isMeasured, } from './measure.ts';
import { objectsDirSize, } from './objects-size.ts';
import { spawnResult, } from './spawn.ts';

/**
 Marginal compressed bytes per commit, with a spread, plus how far the probe
 walked and whether it stopped at the commit cap (making `observedCommits` a
 lower bound on the branch).
 */
export type DeepenResult = {
  readonly marginalLo: number;
  readonly marginalPoint: number;
  readonly marginalHi: number;
  readonly observedCommits: number;
  readonly hitCap: boolean;
};

/**
 Sentinel returned by {@link probeDeepen} when no usable commit delta is observed.
 */
export const NO_DEEPEN: unique symbol = Symbol('git-clone-size/deepen-commit-delta-absent',);

/**
 Lower clamp on the repack pack-layout bias factor; a single full pack is never
 assumed to be more than this much smaller than the incremental fetch growth.
 */
const MIN_BIAS_FACTOR = 0.25;

/**
 Counts commits reachable from HEAD on the (single-branch) shallow clone.
 
 @param clonePath - bare shallow clone directory
 
 @returns commit count, or 0 when the command fails
 */
async function countCommits({ clonePath, }: { readonly clonePath: string; },): Promise<number> {
  /**
   Captured `rev-list --count HEAD` output and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      clonePath,
      'rev-list',
      '--count',
      'HEAD',
    ],
  },);
  /**
   Parsed commit count; 0 on any parse or command failure.
   */
  const count = Math.trunc(Number(stdout,),);
  return (exitCode === 0) && Number.isFinite(count,) ? count : 0;
}

/**
 Floors a bytes-per-commit figure at {@link MIN_MARGINAL_BYTES} so an
 empty-commit run never collapses the estimate to zero.
 
 @param value - raw bytes-per-commit figure
 
 @returns the value, never below the floor
 */
function floored(value: number,): number {
  return Math.max(
    MIN_MARGINAL_BYTES,
    value,
  );
}

/**
 Reduces a marginal-bytes-per-commit series to a corrected point and spread.
 The point applies the repack pack-layout bias (incremental fetches overstate
 per-commit cost); the raw maximum is kept as a conservative upper bound.
 
 @param marginals - per-step bytes-per-commit samples
 
 @param biasFactor - repacked/raw object-store ratio in (0, 1]
 
 @returns floored lo/point/hi marginal bytes per commit
 */
function summarizeMarginals(
  {
    marginals,
    biasFactor,
  }: {
    readonly marginals: readonly number[];
    readonly biasFactor: number;
  },
): {
  readonly lo: number;
  readonly point: number;
  readonly hi: number;
} {
  /**
   Arithmetic mean of the raw marginals.
   */
  const mean = (function meanMarginal(): number {
    /**
     Marginal total isolated inside local mutation scope.
     */
    let total = 0;
    for (const value of marginals)
      total += value;
    return total / marginals.length;
  })();
  return {
    hi: floored(Math.max(...marginals,),),
    lo: floored(Math.min(...marginals,) * biasFactor,),
    point: floored(mean * biasFactor,),
  };
}

/**
 Bounded `git fetch --deepen` probe on an existing shallow clone. Each step
 deepens by a fixed commit count and records object-store growth and the
 commit delta, yielding marginal compressed bytes per commit and its variance.
 A final bounded `git repack -adq` of the temp clone measures the
 incremental-pack bias (raw fetch growth overstates a single full-clone pack),
 which corrects the marginal. NEVER unshallows; stops at the commit cap.
 
 @param clonePath - bare shallow clone to deepen (mutated in the temp dir only)
 
 @param maxDeepenCommits - cap on commits walked before stopping
 
 @param signal - abort signal enforcing the wall-clock budget
 
 @returns marginal estimate and walk metadata, or {@link NO_DEEPEN} when no usable
   delta was observed (single-commit history or immediate failure)
 
 @example
 ```ts
 const deepen = await probeDeepen({ clonePath: shallow.clonePath });
 ```
 */
export async function probeDeepen(
  {
    clonePath,
    maxDeepenCommits = DEFAULT_MAX_DEEPEN_COMMITS,
    signal,
  }: {
    readonly clonePath: string;
    readonly maxDeepenCommits?: number;
    readonly signal: AbortSignal;
  },
): Promise<DeepenResult | typeof NO_DEEPEN> {
  /**
   Tagged logger naming the deepen probe.
   */
  const rl = tagged({
    tag: probeDeepen.name,
    l: logger,
  },);

  /**
   Baseline object-store size before any deepening; unmeasurable here means no
   marginal can be derived, so the probe yields nothing rather than anchoring
   on a fabricated zero.
   */
  const baseBytes = await objectsDirSize({ repoPath: clonePath, },);
  if (!isMeasured(baseBytes,)) {
    rl.debug('deepen base object store unmeasured',);
    return NO_DEEPEN;
  }
  /**
   Side-effecting cursor over the deepen walk: latest commit count and bytes.
   */
  const state = {
    commits: await countCommits({ clonePath, },),
    bytes: baseBytes,
  };
  /**
   Per-step marginal bytes-per-commit samples.
   */
  const marginals: number[] = [];
  /**
   Whether the walk stopped at the commit cap (lower-bound commit count).
   */
  const meta = { hitCap: false, };

  /* oxlint-disable eslint/no-await-in-loop -- deepen steps are inherently sequential: each `git fetch --deepen` extends the prior shallow state, and the marginal measurement needs the object-store growth between consecutive steps; parallelising would defeat the per-step delta. */
  for (let step = 0; step < DEEPEN_STEPS; step += 1) {
    if (state.commits >= maxDeepenCommits) {
      meta.hitCap = true;
      break;
    }
    await spawnResult({
      signal,
      command: 'git',
      args: [
        '-C',
        clonePath,
        'fetch',
        '--deepen',
        String(DEEPEN_STEP_COMMITS,),
      ],
    },);
    /**
     Commit count after this deepen step.
     */
    const commits = await countCommits({ clonePath, },);
    /**
     Object-store bytes after this deepen step.
     */
    const bytes = await objectsDirSize({ repoPath: clonePath, },);
    if (!isMeasured(bytes,))
      break;
    /**
     New commits gained this step; non-positive means history root reached.
     */
    const deltaCommits = commits - state.commits;
    if (deltaCommits <= 0)
      break;
    marginals.push((bytes - state.bytes) / deltaCommits,);
    state.commits = commits;
    state.bytes = bytes;
  }
  /* oxlint-enable eslint/no-await-in-loop */

  if (marginals.length === 0) {
    rl.debug('deepen observed no usable commit delta',);
    return NO_DEEPEN;
  }

  /**
   Raw object-store bytes before the corrective repack.
   */
  const rawBytes = state.bytes;
  await spawnResult({
    signal,
    command: 'git',
    args: [
      '-C',
      clonePath,
      'repack',
      '-adq',
    ],
  },);
  /**
   Object-store bytes after consolidating into a single pack.
   */
  const repackedBytes = await objectsDirSize({ repoPath: clonePath, },);
  /**
   Repacked/raw ratio, the incremental-pack bias term, clamped sanely; an
   unmeasurable repacked size disables the correction (factor 1).
   */
  const biasFactor = (isMeasured(repackedBytes,) && (rawBytes > 0))
    ? clamp({
      value: repackedBytes / rawBytes,
      min: MIN_BIAS_FACTOR,
      max: 1,
    },)
    : 1;
  rl.debug(`deepen marginal bias factor ${biasFactor.toFixed(2,)} over ${String(state.commits,)} commits`,);

  /**
   Corrected marginal summary.
   */
  const summary = summarizeMarginals({
    marginals,
    biasFactor,
  },);
  return {
    hitCap: meta.hitCap,
    marginalHi: summary.hi,
    marginalLo: summary.lo,
    marginalPoint: summary.point,
    observedCommits: state.commits,
  };
}
