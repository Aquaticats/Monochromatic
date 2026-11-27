/**
 Decide whether one member may be dispatched now, from the evidence the pool holds about it.
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  type MemberCapacity,
  observedCeiling,
} from './member-capacity.ts';
import { UNREACHABLE_MEMBER_HOLD_MS, } from './rate-limit-evidence.ts';

//region One admission decision

/**
 Logger for admission decisions. It names members and times; it never logs credentials or bodies.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

/**
 What the pool may do with a member at this moment.

 Neither field set means the member is dispatched now.
 */
export type Admission = {
  /**
   Asked only if no other member can be tried, because its request could not be delivered recently.
   */
  readonly deferMember: boolean;
  /**
   Plain sentence naming why the member is skipped, or empty when it is not.

   A skipped member is one whose window cannot hold this request, one the provider gave a time for, or
   one a reading found no capacity for: the request names it and its state when nothing answers.
   */
  readonly stop: string;
};

/**
 Decide whether one member may be dispatched.

 The request's own size comes first, because nothing about the member's state can widen a window: a
 member whose reported window cannot hold this request never receives it, and the pool asks the next
 member instead. That is the owner's decision of 2026-11-27
 (`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`,
 "Skip a member too small for the request, and advertise the head's window"). It is decided per
 request and nothing about it is kept, so the same member is asked again for the next request.

 The provider's own statements come next: a hold it named, then the ceiling its refusal or quota
 reading reported. Only then does the pool's own unreachability hold apply, because that one is a
 guess the pool may set aside when nothing else can answer.

 @param capacity - member state this session holds

 @param now - admission time, shared by every check

 @param id - selected pool, so a record names it

 @param label - exact provider and model the decision is about

 @param askHeld - whether a member held for unreachability is asked anyway

 @param memberWindow - window that member reports, or zero when it reports none

 @param requestTokens - input tokens this request is estimated to carry

 @returns whether the member is dispatched, skipped, or deferred

 @example
 ```ts
 admissionOf({ capacity, now: Date.now(), id: 'writer-pool', label: 'neuralwatt/deepseek-v4.1-flash-flex', askHeld: false, memberWindow: 1_048_560, requestTokens: 600_000 });
 ```
 */
export function admissionOf({
  capacity,
  now,
  id,
  label,
  askHeld,
  memberWindow,
  requestTokens,
}: {
  readonly capacity: MemberCapacity;
  readonly now: number;
  readonly id: string;
  readonly label: string;
  readonly askHeld: boolean;
  readonly memberWindow: number;
  readonly requestTokens: number;
},): Admission {
  /**
   Logger naming this decision and the pool it belongs to.
   */
  const rl = tagged({
    tag: admissionOf.name,
    l,
  },);
  if ((memberWindow > 0) && (requestTokens > memberWindow)) {
    // Neither a provider's timing nor the pool's own hold can widen a member's window, so a request
    // this member cannot hold is not sent to it at all. The stop names the member, the window it
    // reports and the request's estimated size, so a pool that answers nothing says which member was
    // left out and by how much. The member is asked again for the next request: nothing is recorded.
    rl.debug(`${id}: ${label} reports ${String(memberWindow)} tokens of window and this request is estimated at ${String(requestTokens)} tokens; trying the next member`,);
    return {
      deferMember: false,
      stop: `${label} was not asked: its window is ${String(memberWindow)} tokens and this request is estimated at ${String(requestTokens)} tokens`,
    };
  }
  if (capacity.retryAt > now) {
    rl.debug(`${id}: ${label} has a provider-reported retry window`,);
    return {
      deferMember: false,
      stop: `${label} is held until ${new Date(capacity.retryAt,).toISOString()}`,
    };
  }
  if ((!askHeld) && (capacity.unreachableUntil > now)) {
    // The provider named no time, because its request never arrived: the pool holds the member for a
    // while of its own so the next request does not pay the same wall, and asks it last when nothing
    // else can answer, so an outage no member survived still returns that member's own refusal.
    rl.debug(`${id}: ${label} could not be reached within the last ${UNREACHABLE_MEMBER_HOLD_MS} ms; asking it last`,);
    return {
      deferMember: true,
      stop: '',
    };
  }
  /**
   A refusal's own cap when one is in force, else the published ceiling an earlier refusal read.
   */
  const observed = observedCeiling({
    capacity,
    now,
  },);
  /**
   Published ceiling from an earlier refusal's reading, absent until then.
   */
  const published = capacity.evidence
    .concurrency;
  /**
   Cap this admission must stay under; unbounded when nothing has been learned yet.

   The observed cap governs alone: the refusal describes the account as it stands, while the
   published number is the plan's promise, which an exception such as NeuralWatt's pay-as-you-go
   mode overrides.
   */
  const ceiling = Number.isNaN(observed,) ? (published ?? Infinity) : observed;
  if (capacity.active < ceiling)
    return {
      deferMember: false,
      stop: '',
    };
  if (ceiling === 0) {
    // A reading that reports no capacity excludes the member for the rest of the session, because
    // admission never dispatches it again and so no later refusal reads a fresh ceiling. The turn
    // says nothing about this when another member answers, so the record names the member, the
    // ceiling and the reading's time at a level a person reads.
    rl.info(`${id}: ${label} is excluded for the rest of this session: the reading at ${new Date(capacity.concurrencyReadAt,).toISOString()} reported a concurrency ceiling of zero`,);
  }
  rl.debug(`${id}: ${label} local concurrency reached published or observed cap ${ceiling}`,);
  return {
    deferMember: false,
    stop: ceiling === 0
      ? `${label} has a published concurrency ceiling of zero`
      : `${label} is at its local concurrency ceiling (${capacity.active} of ${ceiling} in use)`,
  };
}

//endregion One admission decision
