/**
 Hand-written stand-ins for Synthetic's quota bodies, used by the quota suites.
 @module
 */

/**
 Hand-written stand-in for Synthetic's `/v2/quotas` weekly window.

 The recorded schema change documents the key names and their types only
 (`doc/troubleshooting/synthetic-quota-weekly-block-renamed.md`), so the shape here is inferred from
 it, and a window at zero remaining means depleted only by that inference.

 @param percentRemaining - share of the weekly block left, zero meaning depleted

 @param nextRegenAt - regeneration time the window reports

 @returns quota response the fixture transport returns

 @example
 ```ts
 syntheticWeeklyWindow({ percentRemaining: 0, nextRegenAt: new Date(0).toISOString() });
 ```
 */
export function syntheticWeeklyWindow({
  percentRemaining,
  nextRegenAt,
}: {
  readonly percentRemaining: number;
  readonly nextRegenAt: string;
},): Response {
  return Response.json({
    planWeeklyLimit: {
      percentRemaining,
      nextRegenAt,
    },
  },);
}

/**
 Hand-written stand-in for Synthetic's `/v2/quotas` subscription block, inferred the same way.

 That a request count at or above the reported limit means depleted is an inference, not a
 documented rule.

 @param requests - requests the snapshot counts against the window

 @param limit - published request allowance for the window

 @param renewsAt - renewal time the account reports

 @returns quota response the fixture transport returns

 @example
 ```ts
 syntheticSubscription({ requests: 1, limit: 1, renewsAt: new Date(0).toISOString() });
 ```
 */
export function syntheticSubscription({
  requests,
  limit,
  renewsAt,
}: {
  readonly requests: number;
  readonly limit: number;
  readonly renewsAt: string;
},): Response {
  return Response.json({
    subscription: {
      requests,
      limit,
      renewsAt,
    },
  },);
}
