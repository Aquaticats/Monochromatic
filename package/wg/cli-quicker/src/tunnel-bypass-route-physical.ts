import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { run, } from './runner.ts';
import { splitWords, } from './text.ts';
import {
  BYPASS_PROTOS,
  type BypassOwnedRoute,
  type BypassProto,
} from './tunnel-bypass-types.ts';

export { isAbsentTableDiagnostic, } from './tunnel-table-diagnostic.ts';

/**
 Route tokens associated with address family.
 */
export type FamilyRoute = BypassOwnedRoute;

/**
 Metric for unreachable fallback when physical family has no default.
 */
const UNREACHABLE_METRIC = 42_760;

/**
 Module logger for bypass route synchronization.
 */
const l = tagged({ tag: 'tunnel-bypass-route', },);

/**
 Removes one key and following value from route token stream.
 
 @param tokens - Route tokens to sanitize.
 
 @param key - Attribute key whose pair is removed.
 
 @returns Fresh tokens without selected pair.
 
 @example
 ```ts
 removeTokenPair({ tokens: ['default', 'proto', 'dhcp'], key: 'proto' });
 ```
 */
export function removeTokenPair(
  {
    tokens,
    key,
  }: {
    readonly tokens: readonly string[];
    readonly key: string;
  },
): readonly string[] {
  /**
   Fresh output excluding selected attribute and value.
   */
  const output: string[] = [];
  /**
   Cursor over flat route token stream.
   */
  const cursor = { index: 0, };
  while (cursor.index < tokens.length) {
    /**
     Current token,
     guaranteed by loop bound.
     */
    const token = tokens[cursor.index] ?? '';
    if (token === key) {
      cursor.index += 2;
      continue;
    }
    output.push(token,);
    cursor.index += 1;
  }
  return output;
}

/**
 Removes volatile and owner-specific attributes from route line.
 
 @param line - One `ip route show` line.
 
 @returns Stable route tokens suitable for another table.
 
 @example
 ```ts
 normalizePhysicalDefaultRoute({ line: 'default via 192.0.2.1 proto dhcp' });
 ```
 
 @internal
 */
export function normalizePhysicalDefaultRoute(
  { line, }: { readonly line: string; },
): readonly string[] {
  /**
   Raw route tokens.
   */
  const tokens = splitWords({ line: line.trim(), },);
  return removeTokenPair({
    tokens: removeTokenPair({
      tokens: removeTokenPair({
        tokens,
        key: 'table',
      },),
      key: 'proto',
    },),
    key: 'expires',
  },);
}

/**
 Reads current main-table physical defaults for both families.
 
 @returns Stable default route token lists with family.
 
 @example
 ```ts
 await readPhysicalDefaults();
 ```
 */
export async function readPhysicalDefaults(): Promise<readonly FamilyRoute[]> {
  /**
   Main-table listings for both address families.
   */
  const listings = await Promise.all(BYPASS_PROTOS.map(function readFamily(
    proto: BypassProto,
  ): Promise<{
    readonly proto: BypassProto;
    readonly stdout: string
  }> {
    return (async function read(): Promise<{
      readonly proto: BypassProto;
      readonly stdout: string;
    }> {
      /**
       Main-table default listing.
       */
      const { stdout, } = await run({
        command: 'ip',
        args: [
          proto,
          'route',
          'show',
          'table',
          'main',
          'default',
        ],
      },);
      return {
        proto,
        stdout,
      };
    })();
  },),);
  /**
   Stable routes preserving main-table output order.
   */
  const routes: FamilyRoute[] = [];
  for (const {
    proto,
    stdout,
  } of listings) {
    for (const line of stdout.split('\n',)) {
      if (line.trim() === '')
        continue;
      /**
       Normalized default route tokens.
       */
      const tokens = normalizePhysicalDefaultRoute({ line, },);
      if (tokens[0] === 'default')
        routes.push({
          proto,
          tokens,
        },);
    }
  }
  return routes;
}

/**
 Produces desired owned defaults,
 adding unreachable fallback for absent family.
 
 @param physical - Current physical defaults.
 
 @returns Desired route tokens for both address families.
 
 @example
 ```ts
 desiredFamilyRoutes({ physical: [] });
 ```
 */
export function desiredFamilyRoutes(
  { physical, }: { readonly physical: readonly FamilyRoute[]; },
): readonly FamilyRoute[] {
  /**
   Desired routes copied away from caller-owned collection.
   */
  const desired: FamilyRoute[] = [];
  for (const route of physical) {
    desired.push({
      proto: route.proto,
      tokens: [...route.tokens,],
    },);
  }
  for (const proto of BYPASS_PROTOS) {
    /**
     Whether current family has at least one physical default.
     */
    const present = physical.some(function sameFamily(route,): boolean {
      return route.proto === proto;
    },);
    if (present)
      continue;
    desired.push({
      proto,
      tokens: [
        'unreachable',
        'default',
        'metric',
        String(UNREACHABLE_METRIC,),
      ],
    },);
  }
  return desired;
}
