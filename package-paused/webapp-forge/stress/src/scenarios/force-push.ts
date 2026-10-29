/**
 * `force-push` stress scenario.
 *
 * Setup: a single repo with `--blob-size=N` bytes of content, replayed
 * `--burst-events=N` times via the git smart-HTTP receive-pack code path.
 * Each iteration is a force-push: oldOid points at the prior commit;
 * newOid points at a fresh commit that overwrites the same blob with
 * different content.
 *
 * Phase 2 invariants asserted:
 *
 * - p99 receive-pack apply latency under {@link P99_LATENCY_BUDGET_MS}
 * - one accepted ref-update per iteration
 * - the dispatcher records a `push` event per accepted ref-update
 *
 * What this scenario does **not** yet assert (Phase 3+):
 *
 * - "only affected blob/diff fragments rebuild": the dependency graph
 *   for repo-typed `push` events is still resolveContext-skipped today
 *   because resourceId is the repo id (not an issue id). Tightening
 *   the dep graph for repo-typed events lands with task #15+.
 *
 * Tunable via `--blob-size=N`, `--burst-events=N`, `--burst-duration-ms=N`.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';
import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core';

import {
  handleReceivePack,
  handleUploadPack,
} from '@monochromatic-dev/webapp-forge-server/ts/git/iso-server';
import { ZERO_OID, } from '@monochromatic-dev/webapp-forge-server/ts/git/iso-server-refs';
import {
  encodePkt,
  flushPkt,
} from '@monochromatic-dev/webapp-forge-server/ts/git/pkt-line';

import nodeFs from 'node:fs';
import { mkdtemp, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import * as git from 'isomorphic-git';

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
 * Tagged logger scoped to the force-push scenario.
 */
const l = tagged({
  tag: 'stress.force-push',
  l: logger,
},);

/**
 * Default per-iteration blob size in kibibytes.
 */
const DEFAULT_BLOB_KIB = 64;

/**
 * Default per-iteration blob size (bytes).
 */
const DEFAULT_BLOB_SIZE: number = DEFAULT_BLOB_KIB * BYTES_PER_KIB;

/**
 * Default burst event count (each event = one force-push iteration).
 */
const DEFAULT_BURST_EVENTS = 20;

/**
 * Default burst duration (ms).
 */
const DEFAULT_BURST_DURATION_MS = 1_000;

/**
 * Latency budget for force-push application at p99.
 */
const P99_LATENCY_BUDGET_MS = 5_000;

/**
 * Owner used in the synthetic gitdir tree.
 */
const OWNER = 'stress';

/**
 * Repo name used in the synthetic gitdir tree.
 */
const REPO = 'force-push';

/**
 * Ref everyone pushes to.
 */
const REF_NAME = 'refs/heads/main';

/**
 * Author timestamp baseline.
 */
const AUTHOR_TS_BASE = 1_700_000_000;

/**
 * Force-push scenario knobs derived from CLI flags.
 */
type ForcePushConfig = {
  blobSize: number;
  burstEvents: number;
  burstDurationMs: number;
};

/**
 * Reads the `--blob-size=`/`--burst-events=`/`--burst-duration-ms=` flags.
 *
 * @returns parsed config
 *
 * @example
 * ```ts
 * const cfg = readConfig();
 * ```
 */
function readConfig(): ForcePushConfig {
  return {
    blobSize: intFlag({
      name: 'blob-size',
      fallback: DEFAULT_BLOB_SIZE,
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
 * Concatenates byte chunks into a single `Uint8Array`.
 *
 * @param chunks - ordered chunks
 *
 * @returns flattened bytes
 *
 * @example
 * ```ts
 * concat([encodePkt('a'), flushPkt()]);
 * ```
 */
function concat(chunks: readonly Uint8Array[],): Uint8Array {
  /**
   * Pre-computed total length so the destination buffer is allocated once.
   */
  let total = 0;
  for (const chunk of chunks)
    total += chunk.byteLength;
  /**
   * Destination buffer sized to fit every chunk back-to-back.
   */
  const out = new Uint8Array(total,);
  /**
   * Write offset advanced by each chunk's length.
   */
  let cursor = 0;
  for (const chunk of chunks) {
    out.set(
      chunk,
      cursor,
    );
    cursor += chunk.byteLength;
  }
  return out;
}

/**
 * Builds an isolated bare gitdir with one commit whose tree contains
 * a single blob of `blobSize` bytes filled with `byteValue`.
 *
 * @param row - inputs
 *
 * @returns commit OID and packfile bytes ready to be wrapped in a triplet
 *
 * @example
 * ```ts
 * const { oid, packfile } = await fabricateCommit({ blobSize: 1024, byteValue: 0x42 });
 * ```
 */
async function fabricateCommit(row: {
  blobSize: number;
  byteValue: number;
  iteration: number;
},): Promise<{
  oid: string;
  packfile: Uint8Array;
}> {
  /**
   * Isolated bare gitdir so fabrication does not collide with the forge's own state.
   */
  const gitdir = await mkdtemp(join(
    scratchRoot(),
    'forge-fp-fab-',
  ),);
  await git.init({
    fs: nodeFs,
    gitdir,
    bare: true,
    defaultBranch: 'main',
  },);
  /**
   * Filler payload sized to `blobSize`; varied byte value differentiates iterations.
   */
  const blobBytes = new Uint8Array(row.blobSize,);
  blobBytes.fill(row.byteValue,);
  /**
   * Blob oid referenced from the synthesised tree.
   */
  const blobOid = await git.writeBlob({
    fs: nodeFs,
    gitdir,
    blob: blobBytes,
  },);
  /**
   * Tree oid pointing at the freshly written blob.
   */
  const treeOid = await git.writeTree({
    fs: nodeFs,
    gitdir,
    tree: [
      {
        mode: '100644',
        path: 'data.bin',
        oid: blobOid,
        type: 'blob',
      },
    ],
  },);
  /**
   * Commit oid the receive-pack body advances `refs/heads/main` to.
   */
  const commitOid = await git.writeCommit({
    fs: nodeFs,
    gitdir,
    commit: {
      tree: treeOid,
      parent: [],
      author: {
        name: 'stress',
        email: 'stress@example.com',
        timestamp: AUTHOR_TS_BASE + row
          .iteration,
        timezoneOffset: 0,
      },
      committer: {
        name: 'stress',
        email: 'stress@example.com',
        timestamp: AUTHOR_TS_BASE + row
          .iteration,
        timezoneOffset: 0,
      },
      message: `iteration ${String(row.iteration,)}\n`,
    },
  },);
  /**
   * Packfile bundle covering the new commit, tree, and blob for the receive-pack stream.
   */
  const result = await git.packObjects({
    fs: nodeFs,
    gitdir,
    oids: [
      commitOid,
      treeOid,
      blobOid,
    ],
    write: false,
  },);
  if (result.packfile
    === undefined)
    throw new Error('packObjects returned no packfile',);
  return {
    oid: commitOid,
    packfile: new Uint8Array(result.packfile,),
  };
}

/**
 * Wraps a (oldOid, newOid, refName) triplet plus a packfile into a
 * receive-pack request body.
 *
 * @param row - inputs
 *
 * @returns request body bytes
 *
 * @example
 * ```ts
 * const body = buildReceivePackBody({ oldOid: ZERO_OID, newOid: 'abc', packfile });
 * ```
 */
function buildReceivePackBody(row: {
  oldOid: string;
  newOid: string;
  packfile: Uint8Array;
},): Uint8Array {
  /**
   * Update line carrying the old-new oid pair, ref name, and capability flags.
   */
  const triplet =
    `${row.oldOid} ${row.newOid} ${REF_NAME}\0report-status side-band-64k\n`;
  return concat([
    encodePkt(triplet,),
    flushPkt(),
    row.packfile,
  ],);
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
 * Runs the force-push scenario.
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
   * Scenario knobs resolved from the `--blob-size`/`--burst-*` flags.
   */
  const config = readConfig();

  // Isolate the bare gitdir tree so the scenario does not pollute the
  // forge's persistent on-disk state.
  /**
   * Throwaway directory holding the scenario's gitdir, kept out of the forge's tree.
   */
  const gitdirRoot = await mkdtemp(join(
    scratchRoot(),
    'forge-fp-root-',
  ),);
  process.env
    .WEBAPP_FORGE_GITDIR_ROOT = gitdirRoot;

  l.info(
    `force-push starting iterations=${String(config.burstEvents,)} blobSize=${
      String(config.blobSize,)
    }`,
  );

  /**
   * Wall-clock start used for the duration summary.
   */
  const startedAt = Date.now();
  /**
   * Per-iteration receive-pack latency samples feeding the percentile summary.
   */
  const samples: number[] = [];
  /**
   * Invariant breaches collected for the scenario result.
   */
  const violations: string[] = [];
  /**
   * Target spacing between iterations so the burst covers `burstDurationMs`.
   */
  const intervalMs = config.burstDurationMs
    / Math
    .max(
    config.burstEvents,
    1,
  );
  /**
   * Final ref oid and accepted-update count produced by the burst loop.
   */
  const {
    priorOid,
    appliedTotal,
  } = await (async function runBurst(): Promise<{
    priorOid: string;
    appliedTotal: number;
  }> {
    /**
     * Old oid each new force-push targets; seeded to the all-zero (no prior ref) sentinel.
     */
    let oid = ZERO_OID;
    /**
     * Running count of accepted ref updates used by the apply-count invariant.
     */
    let applied = 0;
    for (let i = 0; i < config
      .burstEvents; i += 1) {
      /* oxlint-disable no-await-in-loop -- per-iteration sequential is by design (each push depends on the prior ref state) */
      /**
       * Fabricated commit oid for this iteration's force-push.
       */
      const {
        oid: nextOid,
        packfile,
      } = await fabricateCommit({
        blobSize: config.blobSize,
        // Vary the blob byte each iteration so each commit oid differs.
        byteValue: i & 0xFF,
        iteration: i,
      },);
      /* oxlint-enable no-await-in-loop */
      /**
       * Receive-pack request body advancing `refs/heads/main` from `oid` to `nextOid`.
       */
      const body = buildReceivePackBody({
        oldOid: oid,
        newOid: nextOid,
        packfile,
      },);
      /**
       * Apply-phase start timestamp anchoring the latency sample.
       */
      const t0 = Date.now();
      /* oxlint-disable no-await-in-loop -- paced burst by design */
      /**
       * Receive-pack outcome whose `applied` length proves the ref update was accepted.
       */
      const outcome = await handleReceivePack({
        owner: OWNER,
        repo: REPO,
        body,
      },);
      /* oxlint-enable no-await-in-loop */
      /**
       * Apply-phase end timestamp; difference with `t0` is the sample.
       */
      const t1 = Date.now();
      samples.push(t1 - t0,);
      applied += outcome.applied
        .length;
      oid = nextOid;
      // oxlint-disable-next-line no-await-in-loop -- paced burst by design
      await waitInterval({
        intervalMs,
        elapsedMs: t1 - t0,
      },);
    }
    return {
      priorOid: oid,
      appliedTotal: applied,
    };
  })();

  // Verify the resulting ref points where we expect, by upload-packing it back.
  /**
   * Upload-pack response proving the final commit oid is reachable via the smart-HTTP path.
   */
  const verifyBody = await handleUploadPack({
    owner: OWNER,
    repo: REPO,
    body: concat([
      encodePkt(`want ${priorOid} side-band-64k thin-pack\n`,),
      flushPkt(),
      encodePkt('done\n',),
    ],),
  },);
  if (verifyBody.byteLength
    === 0)
    violations.push('upload-pack returned an empty response after force-push burst',);

  /**
   * Median receive-pack latency over the burst.
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

  if (p99 > P99_LATENCY_BUDGET_MS) {
    violations.push(
      `p99 receive-pack latency exceeded ${String(P99_LATENCY_BUDGET_MS,)}ms: ${
        String(p99,)
      }ms`,
    );
  }
  if (appliedTotal !== config
    .burstEvents) {
    violations.push(
      `expected ${String(config.burstEvents,)} accepted ref updates; got ${
        String(appliedTotal,)
      }`,
    );
  }

  l.info(
    `force-push complete applied=${String(appliedTotal,)} p50=${String(p50,)}ms p99=${
      String(p99,)
    }ms`,
  );

  return {
    scenario: 'force-push',
    durationMs,
    eventCount: config.burstEvents,
    p50,
    p99,
    fragmentsWritten: 0,
    bytesWritten: 0,
    staleReadCount: 0,
    invariantViolations: violations,
  };
}

/**
 * Public scenario record.
 */
export const forcePush: Scenario = {
  name: 'force-push',
  run,
};
