/**
 The session state one routing decision reads, and the failure raised before any member answers.
 @module
 */

import type { MemberCapacity, } from './member-capacity.ts';
import type { RoutingRegistry, } from './quota-endpoints.ts';

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

//endregion Session state and routing errors
