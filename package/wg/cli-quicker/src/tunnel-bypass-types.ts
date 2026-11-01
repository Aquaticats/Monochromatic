/**
 Address-family flag accepted by `ip`.
 */
export type BypassProto = '-4' | '-6';

/**
 Canonical route identity persisted after kernel rendering.
 */
export type BypassOwnedRoute = {
  /**
   Address family used by route command.
   */
  readonly proto: BypassProto;

  /**
   Exact displayed route tokens including ownership table and protocol.
   A state file read back must carry a default route (or the unreachable fallback default)
   whose `table` is the state's table and whose `proto` is the private protocol,
   in plain tokens,
   because the watcher deletes these routes as root.
   */
  readonly tokens: readonly string[];
};

/**
 Persisted application-bypass ownership state.
 */
export type BypassState = {
  /**
   State schema version.
   */
  readonly version: 2;

  /**
   Interface whose exempt mark owns state.
   */
  readonly interfaceName: string;

  /**
   Socket mark selected by config.
   */
  readonly mark: number;

  /**
   Dynamically allocated bypass table.
   */
  readonly table: number;

  /**
   Dynamically allocated rule preference.
   */
  readonly preference: number;

  /**
   Random owner token binding state and watcher sidecar.
   */
  readonly ownerId: string;

  /**
   Exact route fingerprints owned by lifecycle or in-progress synchronization.
   */
  readonly routes: readonly BypassOwnedRoute[];
};

/**
 Private route protocol tagging bypass-owned defaults and rules.
 
 Linux treats values at or above static as opaque;
 201 is absent from installed kernel assigned protocol constants.
 */
export const BYPASS_ROUTE_PROTOCOL = 201;

/**
 First table tried for application bypass routes,
 and the lowest table a bypass state may claim:
 the allocator never goes below it,
 so a state that names a lower table (such as `main`, 254, or any table under 256 that `ip` could read from a shorter spelling) is not ours.
 */
export const BASE_BYPASS_TABLE = 52_000;

/**
 Families covered by socket-mark policy rules.
 */
export const BYPASS_PROTOS: readonly BypassProto[] = [
  '-4',
  '-6',
];
