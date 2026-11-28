/**
 What one request observed while it looked for a member to answer it.
 @module
 */

import type { AssistantMessageEvent, } from '@earendil-works/pi-ai';

//region The observations one request keeps

/**
 A terminal error event, the only failure a pool can hand back.
 */
export type TerminalError = Extract<AssistantMessageEvent, { type: 'error'; }>;

/**
 The failures and skips one logical request recorded before it settles.

 Held apart by kind, because the pool's answer is not simply the last thing that happened: a policy
 stop outranks a later refusal, and a request whose members were all passed over owes no member's
 error at all. One request fills this in while it tries its members, so its entries are written.
 */
export type PoolObservations = {
  /**
   Attributed policy stop the pool set aside, kept apart from later refusals so a rate limit cannot
   replace it.
   */
  policyStop?: TerminalError;
  /**
   Last refusal the pool set aside, a 429 or a transient provider failure, as it arrived.
   */
  lastRefusal?: TerminalError;
  /**
   Named reasons that registered members were not admitted, each naming its member and its state.
   */
  admissionStops: string[];
};

//endregion The observations one request keeps
