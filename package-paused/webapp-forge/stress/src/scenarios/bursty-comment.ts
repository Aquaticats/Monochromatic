/**
 * `bursty-comment` stress scenario.
 *
 * Setup: one repo, one issue, no extra labels.
 * Driver: sustained event stream on the same issue for `--burst-duration-ms`
 *         (default 1 second to keep CI fast; the plan's full scenario uses 60s).
 *
 * Phase 1 has no debounce yet (the plan's 1/sec coalescing is Phase 2+),
 * so this scenario asserts:
 *
 * - rebuild count equals event count
 * - final fragment matches a from-scratch ground-truth render
 *
 * The plan's stricter "rebuild count <= 60 (1/sec debounce)" invariant
 * activates in Phase 2 once the dispatcher debounce lands.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';

import {
  createCommentWithEvent,
  createIssueWithEvent,
  insertRepo,
  insertUser,
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
import {
  issueDetailKey,
} from '@monochromatic-dev/webapp-forge-server/ts/worker/fragment-keys';
import { renderFragment, } from '@monochromatic-dev/webapp-forge-server/ts/worker/render';

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
 * Tagged logger scoped to the bursty-comment scenario.
 */
const l = tagged({
  tag: 'stress.bursty-comment',
  l: logger,
},);

/**
 * Default burst event count.
 */
const DEFAULT_BURST_EVENTS = 100;

/**
 * Default burst duration (ms).
 */
const DEFAULT_BURST_DURATION_MS = 1_000;

/**
 * User id reserved for the bursty target issue.
 */
const BURSTY_USER_ID = 'user-bursty';

/**
 * Repo id reserved for the bursty target.
 */
const BURSTY_REPO_ID = 'repo-bursty';

/**
 * Issue id reserved for the bursty target.
 */
const BURSTY_ISSUE_ID = 'issue-bursty';

/**
 * Runs the bursty-comment scenario.
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
   * Resolved `--burst-events` flag controlling the comment count per burst.
   */
  const burstEvents = intFlag({
    name: 'burst-events',
    fallback: DEFAULT_BURST_EVENTS,
  },);
  /**
   * Resolved `--burst-duration-ms` flag controlling the wall-clock budget for the burst.
   */
  const burstDurationMs = intFlag({
    name: 'burst-duration-ms',
    fallback: DEFAULT_BURST_DURATION_MS,
  },);

  /**
   * Shared creation timestamp keeps user, repo, and issue chronologically aligned.
   */
  const now = Date.now();
  await insertUser({
    id: BURSTY_USER_ID,
    login: BURSTY_USER_ID,
    createdAt: now,
  },);
  await insertRepo({
    id: BURSTY_REPO_ID,
    ownerId: BURSTY_USER_ID,
    name: BURSTY_REPO_ID,
    createdAt: now,
  },);
  await createIssueWithEvent({
    id: BURSTY_ISSUE_ID,
    repoId: BURSTY_REPO_ID,
    number: 1,
    authorId: BURSTY_USER_ID,
    title: 'Bursty target',
    createdAt: now,
  },);

  // Drain initial issue.created event so the scenario only measures comment fanout.
  /**
   * Cursor advanced past the seed `issue.created` event so the burst is measured in isolation.
   */
  const initialCursor = await dispatchAndFlush({
    afterEventId: getEventCursor(),
    storage,
    writeBuffer,
  },);
  setEventCursor(initialCursor,);

  /**
   * Wall-clock start used to compute total scenario duration.
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
   * Target spacing between comment dispatches so the burst covers `burstDurationMs`.
   */
  const intervalMs = burstDurationMs / Math
    .max(
    burstEvents,
    1,
  );

  for (let i = 0; i < burstEvents; i += 1) {
    /**
     * Per-event start timestamp anchoring both the sample and the sleep delay.
     */
    const t0 = Date.now();
    // oxlint-disable-next-line no-await-in-loop -- paced burst by design
    await createCommentWithEvent({
      id: `c-bursty-${String(i,)}`,
      issueId: BURSTY_ISSUE_ID,
      authorId: BURSTY_USER_ID,
      body: `bursty ${String(i,)}`,
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
    samples.push(Date.now()
      - t0,);
    if (intervalMs > 0) {
      /**
       * Remaining slice of the per-event budget; floored so the sleep never overshoots.
       */
      const sleep = Math.max(
        0,
        Math.floor(intervalMs - (Date.now()
          - t0),),
      );
      if (sleep > 0) {
        // oxlint-disable-next-line no-await-in-loop -- paced burst by design
        await wait(sleep,);
      }
    }
  }

  /**
   * Storage key whose fragment must match a from-scratch render after the burst.
   */
  const detailKey = issueDetailKey({
    repoId: BURSTY_REPO_ID,
    issueId: BURSTY_ISSUE_ID,
  },);
  /**
   * Ground-truth fragment used as the equality reference.
   */
  const expected = await renderFragment(detailKey,);
  /**
   * Fragment the dispatcher actually persisted; missing or stale triggers a violation.
   */
  const stored = await storage.get(detailKey,);
  if (stored === undefined)
    violations.push('issue detail fragment missing after burst',);
  else {
    /**
     * Decoded ground-truth body; text comparison is friendlier for diff failures.
     */
    const expectedText = new TextDecoder().decode(expected.body,);
    /**
     * Decoded persisted body for the equality check.
     */
    const storedText = new TextDecoder().decode(stored,);
    if (expectedText !== storedText)
      violations.push('final issue detail fragment does not match ground truth',);
  }

  /**
   * Median latency over the recorded burst samples.
   */
  const p50 = percentile({
    samples,
    p: P50,
  },);
  /**
   * Tail latency; the report uses both medians together.
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
  l.info(
    `bursty-comment complete burstEvents=${String(burstEvents,)} durationMs=${
      String(durationMs,)
    } p50=${String(p50,)} p99=${String(p99,)}`,
  );

  return {
    scenario: 'bursty-comment',
    durationMs,
    eventCount: burstEvents,
    p50,
    p99,
    fragmentsWritten: burstEvents,
    bytesWritten: stored?.byteLength
      ?? 0,
    staleReadCount: 0,
    invariantViolations: violations,
  };
}

/**
 * Public scenario record.
 */
export const burstyComment: Scenario = {
  name: 'bursty-comment',
  run,
};
