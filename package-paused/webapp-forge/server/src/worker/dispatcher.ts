/**
 * Dispatcher: turns events into fragment rebuilds.
 *
 * Phase 1 dispatcher is **synchronous in-request**: callers (route
 * handlers, seed scripts, stress harnesses) invoke `processEvent`
 * directly after writing the event row. The work runs to completion
 * inline, so the next read sees the rebuilt fragment.
 *
 * Phase 2+ replaces this with an async worker pool (advisory locks,
 * heartbeat, debounce) that polls the events table. The interface in
 * this module stays stable for that migration: `processEvent` is the
 * single entry point both modes call.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';

import {
  type EventKind,
  getIssue,
  listIssueLabels,
  listRepoLabels,
  upsertFragmentIndexIfNewer,
} from '../data/queries.ts';
import type { Storage, } from '../storage/adapter.ts';
import type { WriteBuffer, } from '../storage/write-buffer.ts';
import {
  dependenciesFor,
  type EventInput,
  type ResolvedEventContext,
} from './dependency-graph.ts';
import type { IssueStateFacet, } from './fragment-keys.ts';
import {
  existingContentHash,
  renderFragment,
} from './render.ts';

/**
 * Tagged logger scoped to the dispatcher.
 */
const l = tagged({
  tag: 'dispatcher',
  l: logger,
},);

/**
 * Output of {@link processEvent}: counters useful to the stress harness.
 */
export type ProcessEventResult = {
  /**
   * Total fragment keys that the dependency graph mapped to.
   */
  readonly fanout: number;
  /**
   * Number of fragments whose content hash matched and were skipped.
   */
  readonly skipped: number;
  /**
   * Number of fragments that were re-rendered and written.
   */
  readonly written: number;
  /**
   * Number of fragments that lost the sequence-guard race (no-op).
   */
  readonly discarded: number;
};

/**
 * Turns one event into rebuilds.
 *
 * @param row - dispatcher inputs
 *
 * @returns counters describing what happened
 *
 * @example
 * ```ts
 * await processEvent({
 *   event: { kind: 'comment.created', resourceId: 'i1' },
 *   sequenceNumber: 3,
 *   eventId: 42,
 *   sink: storageOrBuffer,
 * });
 * ```
 */
export async function processEvent(row: {
  /**
   * Event header to dispatch.
   */
  readonly event: EventInput;
  /**
   * Per-resource sequence number that produced the event. Unused after
   * the sequence guard moved to `events.id`; kept on the signature so
   * callers do not have to change as the dispatcher learns to publish
   * per-resource metrics in Phase 2+.
   */
  readonly sequenceNumber: number;
  /**
   * Generated `events.id`; doubles as the global monotonic sequence guard for `fragment_index`.
   */
  readonly eventId: number;
  /**
   * Storage destination (write buffer in production, adapter in tests).
   */
  readonly sink: Storage | WriteBuffer;
},): Promise<ProcessEventResult> {
  /**
   * Aliases destructured up front so loop branches stay readable.
   */
  const {
    event,
    eventId,
    sink,
  } = row;
  /**
   * Issue context drives the dependency graph; null means the event is stale.
   */
  const context = await resolveContext(event,);
  if (context === null) {
    l.debug(
      `skipping event ${String(eventId,)} for unknown issue ${event.resourceId}`,
    );
    return {
      fanout: 0,
      skipped: 0,
      written: 0,
      discarded: 0,
    };
  }
  /**
   * Fragment keys this event invalidates.
   */
  const keys = dependenciesFor({
    event,
    context,
  },);
  /**
   * Per-event counters accumulated by the rebuild loop below. Held on a
   * `const` state object so the mutation stays out of function-body root.
   */
  const counters: {
    skipped: number;
    written: number;
    discarded: number;
  } = {
    skipped: 0,
    written: 0,
    discarded: 0,
  };

  // Render in declared order. Phase 2+ adds parallel rendering via
  // p-limit; for Phase 1 the synchronous in-request dispatcher only
  // sees one event at a time.
  for (const fragmentKey of keys) {
    /* oxlint-disable no-await-in-loop -- per-fragment serialisation by design */
    /**
     * Prior content hash from `fragment_index`; identical hash skips re-write.
     */
    const previousHash = await existingContentHash(fragmentKey,);
    /* oxlint-enable no-await-in-loop */
    /* oxlint-disable no-await-in-loop -- pure-DB-read render runs sequentially in Phase 1 */
    /**
     * Freshly rendered fragment body plus its content hash.
     */
    const result = await renderFragment(fragmentKey,);
    /* oxlint-enable no-await-in-loop */
    if (previousHash === result
      .contentHash) {
      counters.skipped += 1;
      continue;
    }
    // Use the globally monotonic `events.id` as the sequence guard.
    // Per-resource sequences are not safe across cross-resource fragments
    // (e.g. a filter list whose membership depends on many issues): a
    // later event for resource B can carry a smaller per-resource
    // sequence than an earlier event for resource A but still represents
    // the latest state at write time. `sequenceNumber` stays in the
    // event log for telemetry; `eventId` is what the fragment_index
    // races on.
    /* oxlint-disable no-await-in-loop -- sequence-guarded upsert must observe prior writes */
    /**
     * Upsert outcome: false means a later event already wrote this fragment.
     */
    const accepted = await upsertFragmentIndexIfNewer({
      fragmentKey,
      contentHash: result.contentHash,
      lastBuiltAt: Date.now(),
      sourceEventId: eventId,
      sourceEventSequence: eventId,
    },);
    /* oxlint-enable no-await-in-loop */
    if (!accepted) {
      l.debug(
        `sequence guard rejected fragment write: ${fragmentKey} eventId=${
          String(eventId,)
        }`,
      );
      counters.discarded += 1;
      continue;
    }
    if ('enqueue' in sink) {
      sink.enqueue({
        key: fragmentKey,
        body: result.body,
      },);
    }
    else {
      // oxlint-disable-next-line no-await-in-loop -- raw adapter sink awaits per write
      await sink.put(
        fragmentKey,
        result.body,
      );
    }
    counters.written += 1;
  }

  return {
    fanout: keys.size,
    skipped: counters.skipped,
    written: counters.written,
    discarded: counters.discarded,
  };
}

/**
 * Loads the metadata that the dependency graph needs for an issue event.
 *
 * @param event - event header
 *
 * @returns resolved context, or `null` when the issue does not exist
 */
async function resolveContext(event: EventInput,): Promise<ResolvedEventContext | null> {
  /**
   * Issue id aliased from the resource id for readability.
   */
  const issueId = event.resourceId;
  /**
   * Issue row; missing rows mean the event references a deleted issue.
   */
  const issue = await getIssue(issueId,);
  if (issue === undefined)
    return null;
  /**
   * Labels currently attached to the issue.
   */
  const issueLabels = await listIssueLabels(issueId,);
  /**
   * All labels defined on the repo for filter-list dependency expansion.
   */
  const repoLabels = await listRepoLabels(issue.repo_id,);
  /**
   * Issue state collapsed to the open/closed facet used by fragment keys.
   */
  const state: IssueStateFacet = issue.state
    === 'closed' ? 'closed' : 'open';
  return {
    repoId: issue.repo_id,
    issueLabelIds: issueLabels.map(function pickId(label,) {
      return label.id;
    },),
    repoLabelIds: repoLabels.map(function pickId(label,) {
      return label.id;
    },),
    issueState: state,
  };
}

/**
 * Re-export for tests that want to drive the dispatcher with a typed event kind.
 */
export type { EventKind, };
