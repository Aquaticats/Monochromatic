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
import { refusalOf, } from './attempt-refusal.ts';
import {
  admissionOf,
} from './member-admission.ts';
import {
  capacityOf,
  noteRateLimit,
  noteUnreachable,
  occupySeat,
  readAtRefusal,
} from './member-capacity.ts';
import {
  poolMembers,
  type Member,
} from './model-pools.ts';
import { policyStopMessage, } from './policy-stop-message.ts';
import type { PoolObservations, } from './pool-observations.ts';
import { NO_TRANSPORT_CODE, } from './provider-wire-text.ts';
import { estimateRequestTokens, } from './request-size.ts';
import {
  PoolDispatchError,
  type RoutingSession,
} from './routing-session.ts';
import {
  hasOpenAiPolicyOpening,
  refusalReset,
  headerReset,
  isPolicyStop,
  type ResponseEvidence,
  UNREACHABLE_MEMBER_HOLD_MS,
} from './rate-limit-evidence.ts';

//region Logging

/**
 Logger for dispatch decisions. It names members, decisions and the policy stop's own text; it never
 logs credentials, quota bodies or any other raw provider message. Pi writes these records as JSONL to
 the nearest `node_modules/.monochromatic/<day>/` or `~/.local/state/monochromatic/log`.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

//endregion Logging

//region Request stream

/**
 Start each request at the head of its pool, and hand off a 429, a transient provider failure (503 or
 Cloudflare 520 to 526), or a policy stop that arrived before output.

 A member whose own reported window cannot hold this request is passed over before anything is sent
 to it, for this request alone: the next member that can hold it is asked instead. The pool's
 advertised window is its head member's, so a later member's smaller window no longer decides when Pi
 compacts; it decides only which requests that member is asked.

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
   Estimated input tokens this request carries, read once: every attempt of one logical request sends
   the same transcript, and a retry after Pi compacts the turn is a new request with its own size.
   */
  const requestTokens = estimateRequestTokens(context,);
  /**
   Cancellation that applies to every attempt of this logical request.
   */
  const { signal, } = options ?? {};
  /**
   Observations to report if the pool cannot answer, one kind per way a request can fail to settle.
   */
  const unavailable: PoolObservations = { admissionStops: [], };
  /**
   Members this request passes over because their last request could not be delivered, kept so the
   pool can ask them anyway when no other member was tried. A member whose own window cannot hold
   this request is not one of them: asking it could only end in its own rejection.
   */
  const deferred: Member[] = [];
  /**
   What this request has done so far, held in one object so no name leaks past its step.
   */
  const progress = { attempts: 0, };
  /**
   Options to share with request preparation, omitting an absent argument.
   */
  const supplied = options === undefined ? {} : { options, };
  /**
   Admit and stream one physical member before the pool checks any later member.

   @param member - next provider and model from the ordered pool

   @param askHeld - whether a member held after an undelivered request is asked anyway

   @returns whether the stream emitted a result that ends this logical request
   */
  async function* tryMember({
    member,
    askHeld,
  }: {
    readonly member: Member;
    readonly askHeld: boolean;
  },): AsyncGenerator<AssistantMessageEvent, boolean> {
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
     Whether this member may be dispatched now.
     */
    const admission = admissionOf({
      capacity,
      now: Date.now(),
      id,
      label,
      askHeld,
      memberWindow: physical.contextWindow,
      requestTokens,
    },);
    if (admission.deferMember) {
      deferred.push(member,);
      return false;
    }
    if (admission.stop !== '') {
      unavailable.admissionStops
        .push(admission.stop,);
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
    const evidence: ResponseEvidence = { resetAt: 0, };
    progress.attempts += 1;
    /**
     Local admission held until that physical stream terminates.
     */
    using lease = occupySeat(capacity,);
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
        if (event.type !== 'error') {
          if (buffered.start !== undefined) {
            yield buffered.start;
            delete buffered.start;
          }
          if ((event.type === 'done') && (evidence.resetAt > Date.now()))
            capacity.retryAt = Math.max(
              capacity.retryAt,
              evidence.resetAt,
            );
          if (event.type === 'done') {
            lease.release();
            yield event;
            return true;
          }
          // A content event is now exposed, so a later failure ends the request at this member.
          yield event;
          committed = true;
          continue;
        }
        /**
         A 429 or a transient status takes precedence over generic policy labels, but not OpenAI's
         policy opening; a transport failure carries neither.
         */
        const refusal = refusalOf({
          message: event.error,
          ...(evidence.status === undefined ? {} : { status: evidence.status, }),
        },);
        if (isPolicyStop({
          message: event.error,
          rateLimited: refusal.rateLimited,
          transient: refusal.transient,
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
        if (refusal.transient && (!committed)
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
        if (refusal.rateLimited && (!committed)
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
        /**
         A request that never reached the member, which the owner's decision covers by the same rule:
         nothing was shown, so another member may answer, and Pi's own retry would ask the same
         unreachable member again.
         */
        if ((refusal.unreachableCode !== NO_TRANSPORT_CODE) && (!committed)
          && (event.error
            .content
            .length
            === 0))
          {
          signal?.throwIfAborted();
          noteUnreachable({
            capacity,
            now: Date.now(),
          },);
          rl.warn(`${id}: ${label} could not be reached before any output (${refusal.unreachableCode}); holding it ${UNREACHABLE_MEMBER_HOLD_MS} ms and trying the next member`,);
          unavailable.lastRefusal = event;
          return false;
        }
        // A terminal failure Pi may not re-ask elsewhere: the request ends at this member.
        lease.release();
        yield event;
        return true;
      }
      throw new PoolDispatchError(`${label} closed without a terminal assistant event`,);
    }
  }
  for (const member of members) {
    if (yield* tryMember({
      member,
      askHeld: false,
    },))
      return;
  }
  // No member was dispatched at all: ask the ones held after an undelivered request, so a request no
  // member answered still ends on the refusal that member gave.
  if ((progress.attempts === 0) && (deferred.length > 0)) {
    for (const member of deferred) {
      if (yield* tryMember({
        member,
        askHeld: true,
      },))
        return;
    }
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
