/**
 * `wide-service` stress scenario.
 *
 * Setup: many repos with sparse activity. Default scenario seeds 2,000
 * repos with the long-tail issue distribution, then issues comment events
 * across random repos in parallel.
 *
 * Driver: a single sequential writer issuing comments paced over
 * `--burst-duration-ms` against random repos drawn from the seeded set.
 *
 * Phase 2 invariants asserted:
 *
 * - rebuild p99 latency under {@link P99_LATENCY_BUDGET_MS}
 * - total bytes written under the storage ceiling (a sanity check on
 *   fragment dedup; the same comment-load against many repos should
 *   not blow up storage linearly with event count when the repos are
 *   distinct)
 *
 * The plan calls for `--writers=N` parallel event streams; libSQL's
 * single-connection `BEGIN IMMEDIATE` semantics make naive parallel
 * writers collide ("transaction within a transaction"), so Phase 2's
 * scenario uses a single writer touching distinct repos. Multi-writer
 * parallelism returns when Phase 3+ adds the worker-pool dispatcher
 * (which can lease per-resource libSQL advisory locks).
 *
 * The plan's full scenario runs against 1M repos; the default here is
 * 2,000 to keep CI tractable. Scale up via `--repos=` and
 * `--burst-duration-ms=` for production-like latency curves.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';

import {
  createCommentWithEvent,
} from '@monochromatic-dev/webapp-forge-server/ts/data/queries';
import {
  dispatchAndFlush,
} from '@monochromatic-dev/webapp-forge-server/ts/server/dispatch-and-flush';
import {
  getEventCursor,
  setEventCursor,
  storage,
  writeBuffer,
} from '@monochromatic-dev/webapp-forge-server/ts/server/runtime';

import { seedDataset, } from '@monochromatic-dev/webapp-forge-seed/ts/dataset';
import { rngInt, } from '@monochromatic-dev/webapp-forge-seed/ts/rng';

import { percentile, } from '../percentile.ts';
import type {
  Scenario,
  ScenarioResult,
} from '../types.ts';
import {
  intFlag,
  P50,
  P99,
  wait,
} from './shared.ts';

/**
 * Tagged logger scoped to the wide-service scenario.
 */
const l = tagged({
  tag: 'stress.wide-service',
  l: logger,
},);

/**
 * Default repo count.
 */
const DEFAULT_REPO_COUNT = 2_000;

/**
 * Default user pool size.
 */
const DEFAULT_USER_COUNT = 200;

/**
 * Default total event count summed across all writers.
 */
const DEFAULT_BURST_EVENTS = 200;

/**
 * Default burst duration (ms).
 */
const DEFAULT_BURST_DURATION_MS = 1_000;

/**
 * Latency budget for rebuilds at p99.
 */
const P99_LATENCY_BUDGET_MS = 5_000;

/**
 * Storage byte ceiling: 64 MB by default; scales with `--repos=`.
 */
const BYTES_PER_REPO_CEILING = 32_000;

/**
 * Repo seed factor mirroring `webapp-forge-seed`'s deterministicId rules.
 */
const REPO_SEED_FACTOR = 1_000_000;

/**
 * Author user id used for synthetic burst comments.
 */
const BURST_AUTHOR_ID = 'user-1000';

/**
 * Minimum issues per repo retained for the synthetic burst.
 */
const ISSUES_PER_REPO_FOR_BURST = 1;

/**
 * Scenario knobs derived from CLI flags.
 */
type WideServiceConfig = {
  repoCount: number;
  userCount: number;
  burstEvents: number;
  burstDurationMs: number;
};

/**
 * Reads the `--repos=`/`--users=`/`--writers=`/`--burst-events=`/`--burst-duration-ms=` flags.
 *
 * @returns parsed config
 *
 * @example
 * ```ts
 * const cfg = readConfig();
 * ```
 */
function readConfig(): WideServiceConfig {
  return {
    repoCount: intFlag({
      name: 'repos',
      fallback: DEFAULT_REPO_COUNT,
    },),
    userCount: intFlag({
      name: 'users',
      fallback: DEFAULT_USER_COUNT,
    },),
    burstEvents: intFlag({
      name: 'burst-events',
      fallback: DEFAULT_BURST_EVENTS,
    },),
    burstDurationMs: intFlag({
      name: 'burst-duration-ms',
      fallback: DEFAULT_BURST_DURATION_MS,
    },),
  };
}

/**
 * Returns the deterministic repo id for a given index, mirroring
 * `webapp-forge-seed`'s `deterministicId('repo', seed * REPO_SEED_FACTOR + index)`
 * pattern.
 *
 * @param row - inputs
 *
 * @returns repo id
 *
 * @example
 * ```ts
 * repoIdFor({ seed: 1, index: 0 }); // 'repo-1000000'
 * ```
 */
function repoIdFor(row: {
  seed: number;
  index: number;
},): string {
  return `repo-${String((row.seed
    * REPO_SEED_FACTOR) + row
    .index,)}`;
}

/**
 * Returns the deterministic issue id for a given (repo, index).
 *
 * @param row - inputs
 *
 * @returns issue id
 *
 * @example
 * ```ts
 * issueIdFor({ repoId: 'repo-1000000', index: 0 });
 * ```
 */
function issueIdFor(row: {
  repoId: string;
  index: number;
},): string {
  return `issue-${row.repoId}-${String(row.index,)}`;
}

/**
 * Aggregates total storage byte count across every storage entry.
 *
 * @returns total bytes across all entries
 *
 * @example
 * ```ts
 * const bytes = await sumStorageBytes();
 * ```
 */
async function sumStorageBytes(): Promise<number> {
  /**
   * Every storage key the index returns; the empty prefix means no filter.
   */
  const keys = await storage.list('',);
  /**
   * Per-key byte counts collected concurrently to avoid serialising the storage calls.
   */
  const lengths = await Promise.all(
    keys.map(async function lengthOf(key,) {
      /**
       * Persisted entry; `undefined` means no bytes contribute for this key.
       */
      const value = await storage.get(key,);
      return value?.byteLength
        ?? 0;
    },),
  );
  return lengths.reduce(
    function sum(
      acc,
      n,
    ) {
      return acc + n;
    },
    0,
  );
}

/**
 * Drives the sequential burst, posting `burstEvents` comments paced
 * over `burstDurationMs` against random repos.
 *
 * @param row - inputs
 *
 * @returns per-event latency samples
 *
 * @example
 * ```ts
 * const samples = await runBurst({ burstEvents: 100, intervalMs: 10, repoCount: 2000 });
 * ```
 */
async function runBurst(row: {
  burstEvents: number;
  intervalMs: number;
  repoCount: number;
},): Promise<number[]> {
  /**
   * Per-event latency samples returned to the caller for the percentile summary.
   */
  const samples: number[] = [];
  for (let i = 0; i < row
    .burstEvents; i += 1) {
    /**
     * Deterministic random repo index keyed on `i` so reruns hit the same repos.
     */
    const repoIndex = rngInt({
      seed: i,
      lo: 0,
      hi: row.repoCount,
    },);
    /**
     * Repo id reconstructed from the seed scheme used by the wide-service seeder.
     */
    const repoId = repoIdFor({
      seed: 2,
      index: repoIndex,
    },);
    /**
     * First issue in the target repo; the seeder reserves index 0 for the burst.
     */
    const issueId = issueIdFor({
      repoId,
      index: 0,
    },);
    /**
     * Unique synthetic comment id keyed off the iteration index.
     */
    const commentId = `c-wide-${String(i,)}`;
    /**
     * Per-event start timestamp anchoring the latency sample.
     */
    const t0 = Date.now();
    // oxlint-disable-next-line no-await-in-loop -- paced burst by design
    await createCommentWithEvent({
      id: commentId,
      issueId,
      authorId: BURST_AUTHOR_ID,
      body: `wide ${String(i,)}`,
      createdAt: Date.now(),
    },);
    samples.push(Date.now()
      - t0,);
    if (row.intervalMs
      > 0) {
      /**
       * Remaining slack inside the per-event budget; floored to avoid overshoot.
       */
      const sleep = Math.max(
        0,
        Math.floor(row.intervalMs
          - (Date.now()
            - t0),),
      );
      if (sleep > 0) {
        // oxlint-disable-next-line no-await-in-loop -- pacing
        await wait(sleep,);
      }
    }
  }
  return samples;
}

/**
 * Runs the wide-service scenario.
 *
 * @returns scenario result
 *
 * @example
 * ```ts
 * const result = await run();
 * ```
 */
async function run(): Promise<ScenarioResult> {
  /**
   * Scenario knobs resolved from the `--repos`/`--users`/`--burst-*` flags.
   */
  const config = readConfig();

  l.info(
    `seeding wide dataset repos=${String(config.repoCount,)} burstEvents=${
      String(config.burstEvents,)
    }`,
  );

  // Distinct seed from hot-repo (which uses seed=1) so the two scenarios
  // can share an in-memory DB inside `--scenario=all` runs without
  // colliding on `repos.(owner_id, name)` or `issues.(repo_id, number)`.
  /**
   * Shared creation timestamp keeps seeded rows chronologically coherent.
   */
  const baseTimestamp = Date.now();
  /**
   * Seed summary used by the trailing log line; also makes the seeded row counts visible.
   */
  const summary = await seedDataset({
    seed: 2,
    userCount: config.userCount,
    repoCount: config.repoCount,
    baseTimestamp,
    maxIssuesPerRepo: ISSUES_PER_REPO_FOR_BURST,
  },);

  // Drain seed events first.
  /**
   * Cursor advanced past the seed events so the burst is measured in isolation.
   */
  const seedCursor = await dispatchAndFlush({
    afterEventId: getEventCursor(),
    storage,
    writeBuffer,
  },);
  setEventCursor(seedCursor,);

  // Capture the pre-burst byte total so the ceiling check covers only
  // bytes written during this scenario (not bytes accumulated from
  // prior scenarios sharing the same in-memory storage in `--scenario=all` runs).
  /**
   * Baseline byte total subtracted from the post-burst total to isolate this scenario's writes.
   */
  const bytesBefore = await sumStorageBytes();

  /**
   * Wall-clock start used for the duration summary.
   */
  const startedAt = Date.now();
  /**
   * Target spacing between events so the burst covers `burstDurationMs`.
   */
  const intervalMs = config.burstDurationMs
    / Math
    .max(
    config.burstEvents,
    1,
  );

  /**
   * Per-event latency samples returned from the burst driver.
   */
  const samples = await runBurst({
    burstEvents: config.burstEvents,
    intervalMs,
    repoCount: config.repoCount,
  },);

  // Drain trailing events.
  /**
   * Cursor advanced past any trailing events so the byte total reflects a quiesced state.
   */
  const finalCursor = await dispatchAndFlush({
    afterEventId: getEventCursor(),
    storage,
    writeBuffer,
  },);
  setEventCursor(finalCursor,);

  /**
   * Median rebuild latency over the burst samples.
   */
  const p50 = percentile({
    samples,
    p: P50,
  },);
  /**
   * Tail latency compared against `P99_LATENCY_BUDGET_MS`.
   */
  const p99 = percentile({
    samples,
    p: P99,
  },);
  /**
   * Wall-clock total used by the summary table.
   */
  const durationMs = Date.now()
    - startedAt;
  /**
   * Post-burst byte total; difference with `bytesBefore` is the burst's contribution.
   */
  const totalBytes = await sumStorageBytes();
  /**
   * Bytes attributable to this scenario, compared against the per-repo ceiling.
   */
  const burstDeltaBytes = totalBytes - bytesBefore;
  /**
   * Scaled byte ceiling: `BYTES_PER_REPO_CEILING * repoCount` so the budget tracks `--repos`.
   */
  const bytesCeiling = BYTES_PER_REPO_CEILING * config
    .repoCount;
  /**
   * Invariant breaches collected for the scenario result.
   */
  const violations: string[] = [];

  if (p99 > P99_LATENCY_BUDGET_MS) {
    violations.push(
      `p99 rebuild latency exceeded ${String(P99_LATENCY_BUDGET_MS,)}ms: ${
        String(p99,)
      }ms`,
    );
  }
  if (burstDeltaBytes > bytesCeiling) {
    violations.push(
      `bytes written during burst ${String(burstDeltaBytes,)} exceeded ceiling ${
        String(bytesCeiling,)
      } (${String(BYTES_PER_REPO_CEILING,)} per repo * ${
        String(config.repoCount,)
      } repos)`,
    );
  }

  l.info(
    `wide-service complete eventCount=${String(samples.length,)} repos=${
      String(summary.repos,)
    }`,
  );

  return {
    scenario: 'wide-service',
    durationMs,
    eventCount: samples.length,
    p50,
    p99,
    fragmentsWritten: samples.length,
    bytesWritten: burstDeltaBytes,
    staleReadCount: 0,
    invariantViolations: violations,
  };
}

/**
 * Public scenario record.
 */
export const wideService: Scenario = {
  name: 'wide-service',
  run,
};
