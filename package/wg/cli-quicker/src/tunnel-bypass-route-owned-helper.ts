import {
  CommandError,
  BypassRouteError,
} from './errors.ts';
import { runAllowingFailure, } from './runner.ts';
import { splitWords, } from './text.ts';
import {
  normalizePhysicalDefaultRoute,
  removeTokenPair,
  type FamilyRoute,
} from './tunnel-bypass-route-physical.ts';
import {
  BYPASS_STATE_ABSENT,
  readBypassState,
} from './tunnel-bypass-state.ts';
import { isAbsentTableDiagnostic, } from './tunnel-table-diagnostic.ts';
import {
  BYPASS_PROTOS,
  BYPASS_ROUTE_PROTOCOL,
  type BypassOwnedRoute,
  type BypassProto,
  type BypassState,
} from './tunnel-bypass-types.ts';

/**
 Adds table and protocol ownership attributes to desired route.
 
 @param route - Stable desired route.
 
 @param table - Claimed bypass table.
 
 @returns Route tokens accepted by `ip route replace`.
 
 @example
 ```ts
 ownedRouteTokens({ route, table: 52000 });
 ```
 */
export function ownedRouteTokens(
  {
    route,
    table,
  }: {
    readonly route: FamilyRoute;
    readonly table: number;
  },
): readonly string[] {
  return [
    ...route.tokens,
    'table',
    String(table,),
    'proto',
    String(BYPASS_ROUTE_PROTOCOL,),
  ];
}

/**
 Reads displayed defaults in claimed table.
 
 @param table - Persisted table ownership.
 
 @param ownedOnly - Whether kernel filters by reserved bypass protocol.
 
 @returns Exact displayed routes with families.
 
 @example
 ```ts
 await readDefaultRoutes({ table: 52000, ownedOnly: true });
 ```
 */
export async function readDefaultRoutes(
  {
    table,
    ownedOnly,
  }: {
    readonly table: number;
    readonly ownedOnly: boolean;
  },
): Promise<readonly FamilyRoute[]> {
  /**
   Owned-route listings for both families.
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
       Exact route-show arguments retained for failure translation.
       */
      const args = [
        proto,
        'route',
        'show',
        'table',
        String(table,),
        ...(ownedOnly
          ? [
            'proto',
            String(BYPASS_ROUTE_PROTOCOL,),
          ]
          : []),
        'default',
      ];
      /**
       Routes filtered by exact table and protocol.
       */
      const result = await runAllowingFailure({
        command: 'ip',
        args,
      },);
      if ((result.exitCode !== 0) && (!isAbsentTableDiagnostic({
        proto,
        exitCode: result.exitCode,
        stderr: result.stderr,
      },))) {
        throw new CommandError({
          command: 'ip',
          args,
          exitCode: result.exitCode,
          stderr: result.stderr,
        },);
      }
      return {
        proto,
        stdout: result.exitCode === 0 ? result.stdout : '',
      };
    })();
  },),);
  /**
   Exact route tokens suitable for deletion.
   */
  const routes: FamilyRoute[] = [];
  for (const {
    proto,
    stdout,
  } of listings) {
    for (const line of stdout.split('\n',)) {
      if (line.trim() !== '') {
        routes.push({
          proto,
          tokens: [
            ...splitWords({ line: line.trim(), },),
            ...(ownedOnly
              ? [
                'proto',
                String(BYPASS_ROUTE_PROTOCOL,),
              ]
              : []),
            'table',
            String(table,),
          ],
        },);
      }
    }
  }
  return routes;
}

/**
 Reads protocol-tagged defaults from claimed table.
 
 @param table - Claimed table.
 
 @returns Canonical owned-route candidates.
 
 @example
 ```ts
 await readOwnedRoutes({ table: 52000 });
 ```
 */
export async function readOwnedRoutes(
  { table, }: { readonly table: number; },
): Promise<readonly FamilyRoute[]> {
  return await readDefaultRoutes({
    table,
    ownedOnly: true,
  },);
}

/**
 Reads every default regardless of route protocol.
 
 @param table - Claimed table.
 
 @returns Canonical table defaults.
 
 @example
 ```ts
 await readAllDefaultRoutes({ table: 52000 });
 ```
 */
export async function readAllDefaultRoutes(
  { table, }: { readonly table: number; },
): Promise<readonly FamilyRoute[]> {
  return await readDefaultRoutes({
    table,
    ownedOnly: false,
  },);
}

/**
 Builds exact canonical identity from displayed route.
 
 @param route - Family and exact tokens.
 
 @returns Family-prefixed token stream.
 
 @example
 ```ts
 exactRouteKey({ route });
 ```
 */
export function exactRouteKey(
  { route, }: { readonly route: FamilyRoute; },
): string {
  /**
   Fresh token copy before default-library join boundary.
   */
  const tokens = [...route.tokens,];
  return `${route.proto}:${tokens.join(' ',)}`;
}

/**
 Builds stable comparison key for family route.
 
 @param proto - Address family.
 
 @param text - Space-separated route tokens.
 
 @returns Family-prefixed normalized text.
 
 @example
 ```ts
 routeKey({ proto: '-4', text: 'default via 192.0.2.1' });
 ```
 */
export function routeKey(
  {
    proto,
    text,
  }: {
    readonly proto: BypassProto;
    readonly text: string;
  },
): string {
  /**
   Stable tokens excluding source ownership attributes.
   */
  const normalized = normalizePhysicalDefaultRoute({ line: text, });
  if (normalized[0] !== 'unreachable')
    return `${proto}:${normalized.join(' ',)}`;
  return `${proto}:${removeTokenPair({
    tokens: removeTokenPair({
      tokens: normalized,
      key: 'dev',
    },),
    key: 'pref',
  },)
    .join(' ',)}`;
}

/**
 Copies state with exact owned-route fingerprints.
 
 @param state - Ownership fields retained.
 
 @param routes - Route identities replacing persisted set.
 
 @returns Fresh immutable state.
 
 @example
 ```ts
 stateWithRoutes({ state, routes: [] });
 ```
 */
export function stateWithRoutes(
  {
    state,
    routes,
  }: {
    readonly state: BypassState;
    readonly routes: readonly FamilyRoute[];
  },
): BypassState {
  /**
   Fresh route copies detached from caller containers.
   */
  const ownedRoutes: BypassOwnedRoute[] = [];
  for (const route of routes) {
    ownedRoutes.push({
      proto: route.proto,
      tokens: [...route.tokens,],
    },);
  }
  return {
    version: state.version,
    interfaceName: state.interfaceName,
    mark: state.mark,
    table: state.table,
    preference: state.preference,
    ownerId: state.ownerId,
    routes: ownedRoutes,
  };
}

/**
 Unions route fingerprints by exact family and token identity.
 
 @param first - Existing canonical fingerprints.
 
 @param second - Intended transition fingerprints.
 
 @returns Deduplicated route copies.
 
 @example
 ```ts
 unionRoutes({ first: [], second: [] });
 ```
 */
export function unionRoutes(
  {
    first,
    second,
  }: {
    readonly first: readonly FamilyRoute[];
    readonly second: readonly FamilyRoute[];
  },
): readonly FamilyRoute[] {
  /**
   Exact identities already copied.
   */
  const seen = new Set<string>();
  /**
   Deduplicated route copies.
   */
  const routes: FamilyRoute[] = [];
  for (const group of [
    first,
    second,
  ]) {
    for (const route of group) {
      /**
       Exact route identity for deduplication.
       */
      const key = exactRouteKey({ route, },);
      if (seen.has(key,))
        continue;
      seen.add(key,);
      routes.push({
        proto: route.proto,
        tokens: [...route.tokens,],
      },);
    }
  }
  return routes;
}

/**
 Reports whether displayed route carries reserved bypass protocol.
 
 @param route - Canonical displayed route.
 
 @returns Whether `proto 201` pair is present.
 
 @example
 ```ts
 hasBypassProtocol({ route });
 ```
 */
export function hasBypassProtocol(
  { route, }: { readonly route: FamilyRoute; },
): boolean {
  /**
   Protocol key position in displayed token stream.
   */
  const protocolIndex = route.tokens
    .indexOf('proto',);
  return (protocolIndex !== (-1))
    && (route.tokens[protocolIndex + 1] === String(BYPASS_ROUTE_PROTOCOL,));
}

/**
 Rejects table defaults absent from persisted owner fingerprints.
 
 @param state - Current persisted ownership state.
 
 @example
 ```ts
 await assertRecordedDefaults({ state });
 ```
 */
export async function assertRecordedDefaults(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  /**
   Exact route identities authorized by state.
   */
  const recorded = new Set(state.routes
    .map(function routeIdentity(route,): string {
      return exactRouteKey({ route, },);
    },),);
  /**
   Stable route shapes authorize kernel-added fields during interrupted transition.
   */
  const recordedShapes = new Set(state.routes
    .map(function routeShape(route,): string {
    /**
     Fresh token copy before joining through default-library boundary.
     */
      const tokens = [...route.tokens,];
      return routeKey({
        proto: route.proto,
        text: tokens.join(' ',),
      },);
    },),);
  /**
   Current defaults across every protocol.
   */
  const defaults = await readAllDefaultRoutes({ table: state.table, },);
  /**
   Defaults not authorized by state owner.
   */
  const unexpected = defaults.filter(function unrecorded(route,): boolean {
    if (recorded.has(exactRouteKey({ route, },),))
      return false;
    if (!hasBypassProtocol({ route, }))
      return true;
    /**
     Fresh token copy before semantic comparison.
     */
    const tokens = [...route.tokens,];
    return !recordedShapes.has(routeKey({
      proto: route.proto,
      text: tokens.join(' ',),
    },),);
  },);
  if (unexpected.length > 0) {
    throw new BypassRouteError(
      `Refusing to mutate bypass table ${String(state.table,)} containing unowned default route.`,
    );
  }
}

/**
 Reads current persisted state and verifies owner continuity.
 
 @param requested - State identity supplied by caller.
 
 @returns Latest persisted route fingerprints.
 
 @example
 ```ts
 await currentOwnedState({ requested: state });
 ```
 */
export async function currentOwnedState(
  { requested, }: { readonly requested: BypassState; },
): Promise<BypassState> {
  /**
   State refreshed for watcher retries and transition recovery.
   */
  const current = await readBypassState({ interfaceName: requested.interfaceName, },);
  if (current === BYPASS_STATE_ABSENT)
    throw new BypassRouteError(`Bypass state disappeared for ${requested.interfaceName}.`,);
  if (current.ownerId !== requested.ownerId)
    throw new BypassRouteError(`Bypass state owner changed for ${requested.interfaceName}.`,);
  return current;
}
