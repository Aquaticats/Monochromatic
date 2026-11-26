/**
 Read one terminal error event the way routing does, once, before the pool decides anything.
 @module
 */

import type { AssistantMessage, } from '@earendil-works/pi-ai';
import {
  isRateLimit,
  isTransientRefusal,
  unreachableCode,
} from './rate-limit-evidence.ts';
import { NO_TRANSPORT_CODE, } from './provider-wire-text.ts';

//region What a failure can be

/**
 Fields one terminal failure is read through.
 */
type FailureEvidence = Pick<Readonly<AssistantMessage>, 'errorMessage' | 'stopReason'>;

/**
 What one failed attempt reported, as far as routing can read it.

 A refusal is at most one of these: an HTTP 429 the member states, a transient provider status it
 states, or a transport failure with no status at all. A 429 outranks the other two, because a body
 that names a transient status or a transport code cannot change the status the refusal opened with.
 */
export type AttemptRefusal = {
  /**
   Whether the member refused with an HTTP 429, which reads its quota endpoint and sets a hold.
   */
  readonly rateLimited: boolean;
  /**
   Whether the member refused with a transient provider status the owner's decision moves on.
   */
  readonly transient: boolean;
  /**
   Transport code the failure names when the request never reached the member, or
   {@link NO_TRANSPORT_CODE}.
   */
  readonly unreachableCode: string;
};

/**
 Reading for an event that is not a terminal failure, so nothing routes on it.

 @example
 ```ts
 NO_REFUSAL.rateLimited; // false
 ```
 */
export const NO_REFUSAL: AttemptRefusal = {
  rateLimited: false,
  transient: false,
  unreachableCode: NO_TRANSPORT_CODE,
};

/**
 Classify one terminal error event.

 @param message - failed provider message, which never represents a successful reply here

 @param status - actual HTTP status when Pi's adapter exposed it

 @returns the refusal kind, with the transport code when there is one

 @example
 ```ts
 refusalOf({ message: { stopReason: 'error', errorMessage: '429: slow down' } }); // rateLimited
 ```
 */
export function refusalOf({
  message,
  status,
}: {
  readonly message: FailureEvidence;
  readonly status?: number;
},): AttemptRefusal {
  /**
   Status the adapter exposed, shared by every reading below.
   */
  const exposed = status === undefined ? {} : { status, };
  /**
   Whether the member stated a rate limit.
   */
  const rateLimited = isRateLimit({
    message,
    ...exposed,
  },);
  /**
   Whether the member stated a status it cannot serve the request under now.
   */
  const transient = (!rateLimited) && isTransientRefusal({
    message,
    ...exposed,
  },);
  // A refusal that carries a status is the member's answer, so a transport code inside its body is
  // not a member that could not be reached.
  return {
    rateLimited,
    transient,
    unreachableCode: (rateLimited || transient)
      ? NO_TRANSPORT_CODE
      : unreachableCode({
        message,
        ...exposed,
      },),
  };
}

//endregion What a failure can be
