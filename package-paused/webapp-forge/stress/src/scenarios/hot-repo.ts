/**
 * `hot-repo` stress scenario.
 *
 * Setup: one repo with N issues and a small palette of labels.
 * Driver: a paced burst of `comment.created` events targeted at random
 * issues over a fixed duration.
 *
 * Invariants asserted:
 *
 * - p99 rebuild latency under 5 seconds
 * - zero stale-fragment reads after the burst drains
 *
 * Tunable by `--repo-issues=N`, `--burst-events=N`, `--burst-duration-ms=N`.
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
import { renderFragment, } from '@monochromatic-dev/webapp-forge-server/ts/worker/render';

import { seedDataset, } from '@monochromatic-dev/webapp-forge-seed/ts/dataset';
import { rngInt, } from '@monochromatic-dev/webapp-forge-seed/ts/rng';

import { percentile, } from '../percentile.ts';
import type {
  Scenario,
  ScenarioResult,
} from '../types.ts';
import {
  getFlag,
  intFlag,
  P50,
  P99,
  wait,
} from './shared.ts';

/**
 * Tagged logger scoped to the hot-repo scenario.
 */
const l = tagged({
  tag: 'stress.hot-repo',
  l: logger,
},);

/**
 * Default per-repo issue count.
 */
const DEFAULT_REPO_ISSUES = 200;

/**
 * Default burst event count.
 */
const DEFAULT_BURST_EVENTS = 100;

/**
 * Default burst duration (ms).
 */
const DEFAULT_BURST_DURATION_MS = 1_000;

/**
 * Latency budget for rebuilds at p99.
 */
const P99_LATENCY_BUDGET_MS = 5_000;

/**
 * Repo seed factor mirroring `webapp-forge-seed`'s deterministicId rules.
 */
const REPO_SEED_FACTOR = 1_000_000;

/**
 * Author user id used for synthetic burst comments.
 */
const BURST_AUTHOR_ID = 'user-1000';

/**
 * Scenario knobs derived from CLI flags.
 */
type HotRepoConfig = {
  repoIssues: number;
  burstEvents: number;
  burstDurationMs: number;
};

/**
 * Reads the `--repo-issues=`/`--burst-events=`/`--burst-duration-ms=` flags.
 *
 * @returns parsed config
 *
 * @example
 * ```ts
 * const config = readConfig();
 * ```
 */
function readConfig(): HotRepoConfig {
  void getFlag;
  return {
    repoIssues: intFlag({
      name: 'repo-issues',
      fallback: DEFAULT_REPO_ISSUES,
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
 * Returns the recovered known repo id used by hot-repo's hardcoded seed.
 *
 * @param seed - the seed value seeded into `webapp-forge-seed`
 *
 * @returns repo id of the form `repo-${seed * REPO_SEED_FACTOR}`
 *
 * @example
 * ```ts
 * knownRepoIdFor(1); // 'repo-1000000'
 * ```
 */
function knownRepoIdFor(seed: number,): string {
  return `repo-${String(seed * REPO_SEED_FACTOR,)}`;
}

/**
 * Composes the deterministic issue id mirroring `webapp-forge-seed`'s
 * `deterministicId('issue-${repoId}', i)`.
 *
 * @param row - inputs
 *
 * @returns issue id
 *
 * @example
 * ```ts
 * issueIdFor({ repoId: 'repo-1000000', index: 0 }); // 'issue-repo-1000000-0'
 * ```
 */
function issueIdFor(row: {
  repoId: string;
  index: number;
},): string {
  return `issue-${row.repoId}-${String(row.index,)}`;
}

/**
 * Sleeps the remainder of `intervalMs` after the elapsed work.
 *
 * @param row - sleep parameters
 *
 * @example
 * ```ts
 * await waitInterval({ intervalMs: 25, elapsedMs: 5 });
 * ```
 */
async function waitInterval(row: {
  intervalMs: number;
  elapsedMs: number;
},): Promise<void> {
  if (row.intervalMs
    <= 0)
    return;
  /**
   * Remaining slack inside the per-event budget; floored to avoid overshoot.
   */
  const sleep = Math.max(
    0,
    Math.floor(row.intervalMs
      - row
      .elapsedMs,),
  );
  if (sleep > 0)
    await wait(sleep,);
}

/**
 * Compares storage to a from-scratch render. Returns the count of stale
 * entries.
 *
 * @param violations - mutable list to push human-readable violation messages into
 *
 * @returns stale entry count
 *
 * @example
 * ```ts
 * const stale = await countStaleFragments(violations);
 * ```
 */
async function countStaleFragments(violations: string[],): Promise<number> {
  /**
   * Storage keys covered by the index; one comparison per key.
   */
  const indexedKeys = await storage.list('',);
  /**
   * Running stale count returned to the caller.
   */
  let stale = 0;
  for (const fragmentKey of indexedKeys) {
    /* oxlint-disable no-await-in-loop -- per-key serial check */
    /**
     * Ground-truth fragment used as the equality reference for this key.
     */
    const expected = await renderFragment(fragmentKey,);
    /**
     * Persisted fragment; missing or text-mismatched counts as stale.
     */
    const stored = await storage.get(fragmentKey,);
    /* oxlint-enable no-await-in-loop */
    if (stored === undefined) {
      stale += 1;
      violations.push(`missing storage entry for ${fragmentKey}`,);
      continue;
    }
    /**
     * Decoded persisted body for text comparison.
     */
    const storedText = new TextDecoder().decode(stored,);
    /**
     * Decoded ground-truth body for text comparison.
     */
    const expectedText = new TextDecoder().decode(expected.body,);
    if (storedText !== expectedText)
      stale += 1;
  }
  return stale;
}

/**
 * Adds up `byteLength` over every storage entry in `keys`.
 *
 * @param keys - storage keys
 *
 * @returns total bytes
 *
 * @example
 * ```ts
 * const bytes = await sumStorageBytes(['a', 'b']);
 * ```
 */
async function sumStorageBytes(keys: readonly string[],): Promise<number> {
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
 * Runs the hot-repo scenario.
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
   * Scenario knobs resolved from the `--repo-issues`/`--burst-*` flags.
   */
  const config = readConfig();

  l.info(
    `seeding dataset repoIssues=${String(config.repoIssues,)} burstEvents=${
      String(config.burstEvents,)
    } burstDurationMs=${String(config.burstDurationMs,)}`,
  );

  // Use deterministic seeds so reruns hit the same issues.
  /**
   * Shared creation timestamp keeps seeded rows chronologically coherent.
   */
  const baseTimestamp = Date.now();
  /**
   * Seed summary used to bound the random issue index for the burst.
   */
  const summary = await seedDataset({
    seed: 1,
    userCount: 5,
    repoCount: 1,
    baseTimestamp,
    maxIssuesPerRepo: config.repoIssues,
  },);

  // Drain seed events first so we measure only the burst.
  /**
   * Cursor advanced past the seed events so the burst is measured in isolation.
   */
  const seedCursor = await dispatchAndFlush({
    afterEventId: getEventCursor(),
    storage,
    writeBuffer,
  },);
  setEventCursor(seedCursor,);

  /**
   * Wall-clock start used for the duration summary.
   */
  const startedAt = Date.now();
  /**
   * Per-event latency samples feeding the percentile summary.
   */
  const samples: number[] = [];
  /**
   * Invariant breaches collected for the scenario result.
   */
  const violations: string[] = [];
  /**
   * Repo id matching the deterministic seed factor used in `seedDataset`.
   */
  const knownRepoId = knownRepoIdFor(1,);

  // Fire a paced burst.
  /**
   * Target spacing between events so the burst covers `burstDurationMs`.
   */
  const intervalMs = config.burstDurationMs
    / Math
    .max(
    config.burstEvents,
    1,
  );
  for (let i = 0; i < config
    .burstEvents; i += 1) {
    /**
     * Deterministic per-iteration issue index so reruns target the same issues.
     */
    const issueIndex = rngInt({
      seed: i,
      lo: 0,
      hi: Math.max(
        summary.issues,
        1,
      ),
    },);
    /**
     * Target issue id reconstructed from the deterministic seed scheme.
     */
    const issueId = issueIdFor({
      repoId: knownRepoId,
      index: issueIndex,
    },);
    /**
     * Unique synthetic comment id keyed off the iteration index.
     */
    const commentId = `c-burst-${String(i,)}`;
    /**
     * Per-event start timestamp anchoring the latency sample.
     */
    const t0 = Date.now();
    // oxlint-disable-next-line no-await-in-loop -- paced burst by design
    await createCommentWithEvent({
      id: commentId,
      issueId,
      authorId: BURST_AUTHOR_ID,
      body: `burst comment ${String(i,)}`,
      createdAt: Date.now(),
    },);
    /* oxlint-disable no-await-in-loop -- paced burst by design */
    /**
     * Advanced cursor after the comment event has been dispatched and flushed.
     */
    const cursor = await dispatchAndFlush({
      afterEventId: getEventCursor(),
      storage,
      writeBuffer,
    },);
    /* oxlint-enable no-await-in-loop */
    setEventCursor(cursor,);
    /**
     * Per-event end timestamp; difference with `t0` is the sample.
     */
    const t1 = Date.now();
    samples.push(t1 - t0,);
    // oxlint-disable-next-line no-await-in-loop -- paced burst by design
    await waitInterval({
      intervalMs,
      elapsedMs: t1 - t0,
    },);
  }

  // Drain trailing events.
  /**
   * Cursor advanced past any trailing events so the stale-fragment check sees a quiesced state.
   */
  const finalCursor = await dispatchAndFlush({
    afterEventId: getEventCursor(),
    storage,
    writeBuffer,
  },);
  setEventCursor(finalCursor,);

  /**
   * Stale-fragment count feeding both the result and the invariant check.
   */
  const staleReadCount = await countStaleFragments(violations,);
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
   * Storage keys covered by the index; passed to `sumStorageBytes` for the byte total.
   */
  const indexedKeys = await storage.list('',);
  /**
   * Aggregate fragment-storage byte count reported alongside latency.
   */
  const fragmentBytes = await sumStorageBytes(indexedKeys,);

  if (p99 > P99_LATENCY_BUDGET_MS)
    violations.push(`p99 rebuild latency exceeded 5s: ${String(p99,)}ms`,);
  if (staleReadCount > 0)
    violations.push(`${String(staleReadCount,)} stale fragment(s)`,);

  return {
    scenario: 'hot-repo',
    durationMs,
    eventCount: config.burstEvents,
    p50,
    p99,
    fragmentsWritten: config.burstEvents,
    bytesWritten: fragmentBytes,
    staleReadCount,
    invariantViolations: violations,
  };
}

/**
 * Public scenario record.
 */
export const hotRepo: Scenario = {
  name: 'hot-repo',
  run,
};
