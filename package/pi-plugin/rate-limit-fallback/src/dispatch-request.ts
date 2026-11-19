/**
 Stream the first available member without mixing messages from different attempts.
 @module
 */

import {
  clampThinkingLevel,
  type Api,
  type AssistantMessageEvent,
  type Model,
  type SimpleStreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  capacityOf,
  noteRateLimit,
  refreshCapacity,
  type MemberCapacity,
} from './member-capacity.ts';
import { POOLS, } from './model-pools.ts';
import type { RoutingRegistry, } from './quota-endpoints.ts';
import {
  headerReset,
  isPolicyStop,
  isRateLimit,
  recommendedReset,
} from './rate-limit-evidence.ts';

//region Session state and routing errors

/**
 Session-scoped registry and the retry evidence collected from it.
 */
export type RoutingSession = {
  /**
   Registry supplied by Pi, not by a credential loader.
   */
  readonly registry: RoutingRegistry;
  /**
   Capacity and reset evidence keyed by exact physical model.
   */
  readonly members: Map<string, MemberCapacity>;
};

/**
 Failure before any member could produce a terminal event.
 */
export class PoolDispatchError extends Error {}

/**
 Logger for dispatch decisions; raw provider messages and credentials are not logged.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

//endregion Session state and routing errors

//region Attempt options

/**
 Project caller options onto a physical request without leaking virtual-provider credentials.

 @param model - member to dispatch

 @param options - outer request options

 @returns credential-free options; registry adds credentials for this member
 */
function memberOptions({
  model,
  options,
}: {
  model: Model<Api>;
  options?: SimpleStreamOptions;
},): SimpleStreamOptions {
  /**
   Separate virtual-provider credentials and member-specific constraints from neutral options.
   */
  const {
    apiKey,
    headers,
    env,
    maxTokens,
    reasoning,
    ...neutral
  } = options ?? {};
  void apiKey;
  void headers;
  void env;
  /**
   Requested reasoning level after applying the physical model's capabilities.
   */
  const clamped = clampThinkingLevel(
    model,
    reasoning ?? 'off',
  );
  return {
    ...neutral,
    ...(maxTokens === undefined ? {} : {
      maxTokens: model.maxTokens > 0 ? Math.min(
        maxTokens,
        model.maxTokens,
      ) : maxTokens,
    }),
    ...(clamped === 'off' ? {} : { reasoning: clamped, }),
  };
}

//endregion Attempt options

//region Request stream

/**
 Start each request at the head of its pool and retry only uncommitted HTTP 429s.

 @param session - source of the live model catalog and previous quota evidence

 @param model - selected pool model

 @param context - full Pi transcript to give each attempt

 @param options - caller options including abort signal and observers

 @returns normalized events from exactly one member, or a terminal setup failure

 @throws PoolDispatchError when routing is impossible or a provider omits its terminal event
 */
export async function* dispatchRequest({
  session,
  model,
  context,
  options,
}: {
  session: RoutingSession;
  model: Model<Api>;
  context: TranscriptContext;
  options?: SimpleStreamOptions;
},): AsyncGenerator<AssistantMessageEvent> {
  /**
   Tagged logger that names the current operation without exposing credentials.
   */
  const rl = tagged({
    tag: dispatchRequest.name,
    l,
  },);
  /**
   Selected pool id, not the id sent to a physical provider.
   */
  const { id, } = model;
  if ((id !== 'writer') && (id !== 'reviewer'))
    throw new PoolDispatchError(`Unknown rate-limit-fallback model: ${id}`,);
  /**
   Cancellation that applies to every attempt of this logical request.
   */
  const { signal, } = options ?? {};
  /**
   Observations to report if the pool cannot answer.
   */
  const unavailable: {
    /** Last discarded 429. */
    lastRateLimit?: Extract<AssistantMessageEvent, { type: 'error'; }>;
    /** Earliest known reset among held members. */
    nextRetryAt: number;
    /** Whether local concurrency prevented admission. */
    localsFull: boolean;
  } = {
    nextRetryAt: Infinity,
    localsFull: false,
  };
  /**
   Options to share with request preparation, omitting an absent argument.
   */
  const supplied = options === undefined ? {} : { options, };
  for (const member of POOLS[id]) {
    signal?.throwIfAborted();
    /**
     Exact member from Pi's current model registry.
     */
    const physical = session.registry
      .find(
      member.provider,
      member.id,
      );
    /**
     Lookup key for the exact provider and model, not the selected pool alias.
     */
    const label = `${member.provider}/${member.id}`;
    if (physical === undefined) {
      rl.debug(`${id}: missing registry model ${label}`,);
      continue;
    }
    if ((physical.provider !== member.provider) || (physical.id !== member.id))
      throw new PoolDispatchError(`Pi registry returned a different model for ${label}`,);
    /**
     Member-specific state for rate evidence and local admission.
     */
    const capacity = capacityOf({
      members: session.members,
      label,
    },);
    if (capacity.retryAt <= Date.now())
      await refreshCapacity({
        capacity,
        registry: session.registry,
        model: physical,
        ...supplied,
      },);
    signal?.throwIfAborted();
    if (capacity.retryAt > Date.now()) {
      rl.debug(`${id}: ${label} has a provider-reported retry window`,);
      unavailable.nextRetryAt = Math.min(
        unavailable.nextRetryAt,
        capacity.retryAt,
      );
      continue;
    }
    /**
     Most restrictive credible ceiling among published and observed values.
     */
    const ceiling = Math.min(
      capacity.overage?.concurrency ?? Infinity,
      capacity.evidence.concurrency ?? Infinity,
    );
    if (capacity.active >= ceiling) {
      rl.debug(`${id}: ${label} local concurrency reached published or observed cap ${ceiling}`,);
      capacity.needsProbe = true;
      unavailable.localsFull = true;
      continue;
    }
    rl.debug(`${id}: dispatching to ${label}`,);
    /**
     Caller options stripped of credentials and scoped to the physical model.
     */
    const forwarded = memberOptions({
      model: physical,
      ...supplied,
    },);
    /**
     Response evidence shared between the callback and stream consumer.
     */
    const evidence: {
      /** Provider-supplied deadline, or zero if unavailable. */
      resetAt: number;
      /** HTTP status exposed by the physical adapter. */
      status?: number;
    } = { resetAt: 0, };
    capacity.active += 1;
    /**
     Local admission held until that physical stream terminates.
     */
    const seat = {
      occupied: true,
      release(): void {
        if (!seat.occupied)
          return;
        seat.occupied = false;
        capacity.active -= 1;
      },
      [Symbol.dispose](): void { seat.release(); },
    };
    /**
     Idempotent cleanup for cancellation, error, and terminal output.
     */
    using lease = seat;
    {
      /**
       Normalized stream whose terminal event decides whether to stop or hand off.
       */
      const stream = session.registry
        .streamSimple(
        physical,
        context,
        {
          ...forwarded,
          async onResponse(
            response,
            responseModel,
          ) {
            /**
             Status observed on this callback, before awaiting the caller's callback.
             */
            const { status, } = response;
            evidence.status = status;
            if ((status === 429) || ((status >= 200) && (status < 300))) {
              evidence.resetAt = headerReset({
                provider: physical.provider,
                headers: new Headers(response.headers,),
                now: Date.now(),
                status,
              },);
            }
            await forwarded.onResponse?.(
              response,
              responseModel,
            );
          },
        },
        );
      /**
       Start event held before a physical attempt emits output.
       */
      const buffered: { start?: Extract<AssistantMessageEvent, { type: 'start'; }>; } = {};
      /**
       Whether a content event from this attempt has been exposed.
       */
      let committed = false;
      /**
       Distinguishes intentional handoff from a malformed stream closing early.
       */
      let limited = false;
      for await (const event of stream) {
        if (event.type === 'start') {
          buffered.start = event;
          continue;
        }
        if ((event.type === 'error') && isPolicyStop(event.error,)) {
          if (buffered.start !== undefined)
            yield buffered.start;
          // Remove incidental retry wording so Pi's own retry classifier does not restart this request.
          lease.release();
          yield {
            ...event,
            error: {
              ...event.error,
              errorMessage: 'The provider stopped the request under its content policy.',
            },
          };
          return;
        }
        if ((event.type === 'error') && (!committed)
          && (event.error
            .content
            .length
            === 0)
          && isRateLimit({
            message: event.error,
            ...(evidence.status === undefined ? {} : { status: evidence.status, }),
          },))
          {
          signal?.throwIfAborted();
          /**
           Observation time used to compare provider deadlines.
           */
          const now = Date.now();
          /**
           Earliest time this member should be reconsidered after the refusal.
           */
          const retryAt = evidence.resetAt > now
            ? evidence.resetAt
            : recommendedReset({
              provider: physical.provider,
              message: event.error,
              now,
            },);
          noteRateLimit({
            capacity,
            provider: physical.provider,
            errorMessage: event.error
              .errorMessage
              ?? '',
            retryAt,
          },);
          rl.debug(`${id}: ${label} returned HTTP 429 before output`,);
          unavailable.lastRateLimit = event;
          limited = true;
          break;
        }
        if (buffered.start !== undefined) {
          yield buffered.start;
          delete buffered.start;
        }
        if ((event.type === 'done') && (evidence.resetAt > Date.now()))
          capacity.retryAt = Math.max(
            capacity.retryAt,
            evidence.resetAt,
          );
        if ((event.type === 'done') || (event.type === 'error')) {
          lease.release();
          yield event;
          return;
        }
        yield event;
        committed = true;
      }
      if (!limited)
        throw new PoolDispatchError(`${label} closed without a terminal assistant event`,);
    }
  }
  if (unavailable.lastRateLimit !== undefined) {
    yield unavailable.lastRateLimit;
    return;
  }
  if (Number.isFinite(unavailable.nextRetryAt,))
    throw new PoolDispatchError(`HTTP 429: no eligible ${id} member until ${new Date(unavailable.nextRetryAt,).toISOString()}`,);
  if (unavailable.localsFull)
    throw new PoolDispatchError(`HTTP 429: ${id} reached its local provider concurrency ceiling`,);
  throw new PoolDispatchError(`No ${id} member is registered in Pi. Check the member model catalogs.`,);
}

//endregion Request stream
