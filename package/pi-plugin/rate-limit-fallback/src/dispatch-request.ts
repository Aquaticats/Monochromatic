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
  options: SimpleStreamOptions | undefined;
},): SimpleStreamOptions {
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
  options: SimpleStreamOptions | undefined;
},): AsyncGenerator<AssistantMessageEvent> {
  const rl = tagged({
    tag: dispatchRequest.name,
    l,
  },);
  const { id, } = model;
  if ((id !== 'writer') && (id !== 'reviewer'))
    throw new PoolDispatchError(`Unknown rate-limit-fallback model: ${id}`,);
  const { signal, } = options ?? {};
  let lastRateLimit: Extract<AssistantMessageEvent, { type: 'error'; }> | undefined;
  let nextRetryAt = Infinity;
  let localsFull = false;
  for (const member of POOLS[id]) {
    signal?.throwIfAborted();
    const physical = session.registry
      .find(
      member.provider,
      member.id,
      );
    const label = `${member.provider}/${member.id}`;
    if (physical === undefined) {
      rl.debug(`${id}: missing registry model ${label}`,);
      continue;
    }
    if ((physical.provider !== member.provider) || (physical.id !== member.id))
      throw new PoolDispatchError(`Pi registry returned a different model for ${label}`,);
    const capacity = capacityOf({
      members: session.members,
      label,
    },);
    if (capacity.retryAt <= Date.now())
      await refreshCapacity({
        capacity,
        registry: session.registry,
        model: physical,
        options,
      },);
    signal?.throwIfAborted();
    if (capacity.retryAt > Date.now()) {
      rl.debug(`${id}: ${label} has a provider-reported retry window`,);
      nextRetryAt = Math.min(
        nextRetryAt,
        capacity.retryAt,
      );
      continue;
    }
    const ceiling = Math.min(
      capacity.overage?.concurrency ?? Infinity,
      capacity.evidence.concurrency ?? Infinity,
    );
    if (capacity.active >= ceiling) {
      rl.debug(`${id}: ${label} local concurrency reached published or observed cap ${ceiling}`,);
      capacity.needsProbe = true;
      localsFull = true;
      continue;
    }
    rl.debug(`${id}: dispatching to ${label}`,);
    const forwarded = memberOptions({
      model: physical,
      options,
    },);
    let responseReset = 0;
    let status: number | undefined;
    capacity.active += 1;
    const seat = {
      occupied: true,
      release() {
        if (!seat.occupied)
          return;
        seat.occupied = false;
        capacity.active -= 1;
      },
      [Symbol.dispose]() { seat.release(); },
    };
    using lease = seat;
    {
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
            status = response.status;
            if ((status === 429) || ((status >= 200) && (status < 300))) {
              responseReset = headerReset({
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
      let start: Extract<AssistantMessageEvent, { type: 'start'; }> | undefined;
      let committed = false;
      let limited = false;
      for await (const event of stream) {
        if (event.type === 'start') {
          start = event;
          continue;
        }
        if ((event.type === 'error') && isPolicyStop(event.error,)) {
          if (start !== undefined)
            yield start;
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
            status,
          },))
          {
          signal?.throwIfAborted();
          const now = Date.now();
          const retryAt = responseReset > now
            ? responseReset
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
          lastRateLimit = event;
          limited = true;
          break;
        }
        if (start !== undefined) {
          yield start;
          start = undefined;
        }
        if ((event.type === 'done') && (responseReset > Date.now()))
          capacity.retryAt = Math.max(
            capacity.retryAt,
            responseReset,
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
  if (lastRateLimit !== undefined) {
    yield lastRateLimit;
    return;
  }
  if (Number.isFinite(nextRetryAt,))
    throw new PoolDispatchError(`HTTP 429: no eligible ${id} member until ${new Date(nextRetryAt,).toISOString()}`,);
  if (localsFull)
    throw new PoolDispatchError(`HTTP 429: ${id} reached its local provider concurrency ceiling`,);
  throw new PoolDispatchError(`No ${id} member is registered in Pi. Check the member model catalogs.`,);
}

//endregion Request stream
