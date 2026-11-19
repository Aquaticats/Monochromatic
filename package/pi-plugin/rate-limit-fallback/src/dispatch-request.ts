/**
 Stream the first available member without mixing messages from different attempts.
 @module
 */

import type {
  Api,
  AssistantMessageEvent,
  Model,
  SimpleStreamOptions,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { memberOptions, } from './member-attempt-options.ts';
import {
  capacityOf,
  noteRateLimit,
  observedCeiling,
  readAtRefusal,
  type MemberCapacity,
} from './member-capacity.ts';
import {
  poolMembers,
  type Member,
} from './model-pools.ts';
import { policyStopMessage, } from './policy-stop-message.ts';
import type { RoutingRegistry, } from './quota-endpoints.ts';
import {
  hasOpenAiPolicyOpening,
  refusalReset,
  headerReset,
  isPolicyStop,
  isRateLimit,
  isTransientRefusal,
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
export class PoolDispatchError extends Error {
  /**
   Distinguishes pool dispatch failures from errors raised by member adapters.
   */
  override readonly name = 'PoolDispatchError';
}

/**
 Logger for dispatch decisions. It names members, decisions and the policy stop's own text; it never
 logs credentials, quota bodies or any other raw provider message. Pi writes these records as JSONL to
 the nearest `node_modules/.monochromatic/<day>/` or `~/.local/state/monochromatic/log`.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

//endregion Session state and routing errors

//region Request stream

/**
 Start each request at the head of its pool, and hand off a 429, a transient provider failure (503 or
 Cloudflare 520 to 526), or a policy stop that arrived before output.

 @param session - source of the live model catalog and previous quota evidence

 @param model - selected pool model

 @param context - full Pi transcript to give each attempt

 @param options - caller options including abort signal and observers

 @returns normalized events from exactly one member, or a terminal setup failure

 @throws PoolDispatchError when routing is impossible or a provider omits its terminal event

 @example
 ```ts
 for await (const event of dispatchRequest({ session, model, context })) render(event);
 ```
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
  /**
   Ordered members that pool id names, or undefined when the package offers no such pool.
   */
  const members = poolMembers(id,);
  if (members.length === 0)
    throw new PoolDispatchError(`Unknown rate-limit-fallback model: ${id}`,);
  /**
   Cancellation that applies to every attempt of this logical request.
   */
  const { signal, } = options ?? {};
  /**
   Observations to report if the pool cannot answer.
   */
  const unavailable: {
    /**
     Attributed policy stop the pool set aside, kept apart from later refusals so a rate limit cannot
     replace it.
     */
    policyStop?: Extract<AssistantMessageEvent, { type: 'error'; }>;
    /**
     Last refusal the pool set aside, a 429 or a transient provider failure, as it arrived.
     */
    lastRefusal?: Extract<AssistantMessageEvent, { type: 'error'; }>;
    /**
     Named reasons that registered members were not admitted.
     */
    admissionStops: string[];
  } = { admissionStops: [], };
  /**
   Options to share with request preparation, omitting an absent argument.
   */
  const supplied = options === undefined ? {} : { options, };
  /**
   Admit and stream one physical member before the pool checks any later member.

   @param member - next provider and model from the ordered pool

   @returns whether the stream emitted a result that ends this logical request
   */
  async function* tryMember(member: Member,): AsyncGenerator<AssistantMessageEvent, boolean> {
    signal?.throwIfAborted();
    /**
     Exact member from Pi's current model registry.
     */
    const resolveModel = session.registry
      .find
      .bind(session.registry,);
    /**
     Physical model resolved by Pi or undefined when the member is not configured.
     */
    const physical = resolveModel(
      member.provider,
      member.id,
    );
    /**
     Lookup key for the exact provider and model, not the selected pool alias.
     */
    const label = `${member.provider}/${member.id}`;
    if (physical === undefined) {
      rl.debug(`${id}: missing registry model ${label}`,);
      return false;
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
    /**
     Admission time, shared by the hold and observed-cap checks.
     */
    const admittedAt = Date.now();
    if (capacity.retryAt > admittedAt) {
      rl.debug(`${id}: ${label} has a provider-reported retry window`,);
      unavailable.admissionStops
        .push(`${label} is held until ${new Date(capacity.retryAt,).toISOString()}`,);
      return false;
    }
    /**
     A refusal's own cap when one is in force, else the published ceiling an earlier refusal read.

     The observed cap governs alone: the refusal describes the account as it stands, while the
     published number is the plan's promise, which an exception such as NeuralWatt's pay-as-you-go
     mode overrides.
     */
    const observed = observedCeiling({
      capacity,
      now: admittedAt,
    },);
    /**
     Published ceiling from an earlier refusal's reading, absent until then.
     */
    const published = capacity.evidence
      .concurrency;
    /**
     Cap this admission must stay under; unbounded when nothing has been learned yet.
     */
    const ceiling = Number.isNaN(observed,) ? (published ?? Infinity) : observed;
    if (capacity.active >= ceiling) {
      if (ceiling === 0) {
        // A reading that reports no capacity excludes the member for the rest of the session, because
        // admission never dispatches it again and so no later refusal reads a fresh ceiling. The turn
        // says nothing about this when another member answers, so the record names the member, the
        // ceiling and the reading's time at a level a person reads.
        rl.info(`${id}: ${label} is excluded for the rest of this session: the reading at ${new Date(capacity.concurrencyReadAt,).toISOString()} reported a concurrency ceiling of zero`,);
      }
      rl.debug(`${id}: ${label} local concurrency reached published or observed cap ${ceiling}`,);
      unavailable.admissionStops
        .push(ceiling === 0
          ? `${label} has a published concurrency ceiling of zero`
          : `${label} is at its local concurrency ceiling (${capacity.active} of ${ceiling} in use)`,);
      return false;
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
      /**
       Provider-supplied deadline, or zero if unavailable.
       */
      resetAt: number;
      /**
       HTTP status exposed by the physical adapter.
       */
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
            // Every response supplies whatever timing it names: a success's depleted-window headers,
            // and a refusal's Retry-After, which a 503 or Cloudflare status may carry as well as a 429.
            evidence.resetAt = headerReset({
              provider: physical.provider,
              headers: new Headers(response.headers,),
              now: Date.now(),
              status,
            },);
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
      for await (const event of stream) {
        if (event.type === 'start') {
          buffered.start = event;
          continue;
        }
        /**
         A 429 or a transient status takes precedence over generic policy labels, but not OpenAI's
         policy opening.
         */
        const rateLimited = (event.type === 'error') && isRateLimit({
          message: event.error,
          ...(evidence.status === undefined ? {} : { status: evidence.status, }),
        },);
        /**
         Whether this attempt refused with a transient provider status.
         */
        const transient = (event.type === 'error')
          && (!rateLimited)
          && isTransientRefusal({
            message: event.error,
            ...(evidence.status === undefined ? {} : { status: evidence.status, }),
          },);
        if ((event.type === 'error') && isPolicyStop({
          message: event.error,
          rateLimited,
          transient,
        },)) {
          /**
           Terminal record naming the member and keeping the advisory, with retry wording removed so
           Pi's own retry classifier does not restart this request.
           */
          const stopped = {
            ...event,
            error: {
              ...event.error,
              errorMessage: policyStopMessage({
                memberLabel: label,
                message: event.error
                  .errorMessage
                  ?? '',
              },),
            },
          };
          /**
           Items the terminal message carries; Pi saw no output while this is zero.
           */
          const terminalItems = event.error
            .content
            .length;
          if ((!committed) && (terminalItems === 0)
            && (physical.provider === 'openai')
            && hasOpenAiPolicyOpening(event.error,))
            {
            // Pi has seen no output, so the same transcript may still be answered further along.
            // Only the OpenAI provider's own opening moves on; another member's content stop, even
            // one that opens with the same sentence, ends the request at that member.
            rl.debug(`${id}: ${label} stopped the request under its content policy before output; trying the next member: ${event.error
              .errorMessage
              ?? ''}`,);
            unavailable.policyStop = stopped;
            return false;
          }
          if (buffered.start !== undefined)
            yield buffered.start;
          lease.release();
          yield stopped;
          return true;
        }
        /**
         A provider that cannot serve this request now, but may serve the next: the owner's decision
         of 2026-11-19 moves it on like a 429, so Pi's own retry never restarts at that member.
         */
        if (transient && (!committed)
          && (event.error
            .content
            .length
            === 0))
          {
          signal?.throwIfAborted();
          /**
           Observation time used to compare provider deadlines.
           */
          const movedAt = Date.now();
          if (evidence.resetAt > movedAt)
            capacity.retryAt = Math.max(
              capacity.retryAt,
              evidence.resetAt,
            );
          rl.debug(`${id}: ${label} refused with a transient provider status before output; trying the next member`,);
          unavailable.lastRefusal = event;
          return false;
        }
        if (rateLimited && (!committed)
          && (event.error
            .content
            .length
            === 0))
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
            : refusalReset({
              provider: physical.provider,
              message: event.error,
              now,
            },);
          // Read the provider's own quota now that it refused, bounded by the read's deadline. The
          // evidence describes the plan at the moment of refusal and governs the next admission.
          await readAtRefusal({
            capacity,
            registry: session.registry,
            model: physical,
            ...supplied,
          },);
          signal?.throwIfAborted();
          noteRateLimit({
            capacity,
            provider: physical.provider,
            errorMessage: event.error
              .errorMessage
              ?? '',
            retryAt,
          },);
          rl.debug(`${id}: ${label} returned HTTP 429 before output`,);
          unavailable.lastRefusal = event;
          return false;
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
          return true;
        }
        yield event;
        committed = true;
      }
      throw new PoolDispatchError(`${label} closed without a terminal assistant event`,);
    }
  }
  for (const member of members) {
    if (yield* tryMember(member,))
      return;
  }
  /**
   The failure the pool owes: a stop it set aside, which ends the run so Pi's own retry cannot
   restart at the member that refused, then the last refusal as it arrived.
   */
  const owed = unavailable.policyStop ?? unavailable.lastRefusal;
  if (owed !== undefined) {
    yield owed;
    return;
  }
  if (unavailable.admissionStops
    .length
    > 0)
    throw new PoolDispatchError(`There is no eligible ${id} member: ${unavailable.admissionStops
      .join('; ',)}.`,);
  throw new PoolDispatchError(`No ${id} member is registered in Pi. Check the member model catalogs.`,);
}

//endregion Request stream
