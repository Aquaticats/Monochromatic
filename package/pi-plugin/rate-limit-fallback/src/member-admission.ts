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

   A skipped member is one the provider gave a time for, or one a reading found no capacity for: the
   request names it and its state when nothing answers.
   */
  readonly stop: string;
};

/**
 Decide whether one member may be dispatched.

 The provider's own statements come first: a hold it named, then the ceiling its refusal or quota
 reading reported. Only then does the pool's own unreachability hold apply, because that one is a
 guess the pool may set aside when nothing else can answer.

 @param capacity - member state this session holds

 @param now - admission time, shared by every check

 @param id - selected pool, so a record names it

 @param label - exact provider and model the decision is about

 @param askHeld - whether a member held for unreachability is asked anyway

 @returns whether the member is dispatched, skipped, or deferred

 @example
 ```ts
 admissionOf({ capacity, now: Date.now(), id: 'writer-pool', label: 'neuralwatt/deepseek-v4.1-flash-flex', askHeld: false });
 ```
 */
export function admissionOf({
  capacity,
  now,
  id,
  label,
  askHeld,
}: {
  readonly capacity: MemberCapacity;
  readonly now: number;
  readonly id: string;
  readonly label: string;
  readonly askHeld: boolean;
},): Admission {
  /**
   Logger naming this decision and the pool it belongs to.
   */
  const rl = tagged({
    tag: admissionOf.name,
    l,
  },);
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
