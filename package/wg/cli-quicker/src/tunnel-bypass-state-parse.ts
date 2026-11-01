import { BypassStateError, } from './errors.ts';
import { isValidInterfaceName, } from './interface-name.ts';
import {
  BYPASS_ROUTE_PROTOCOL,
  type BypassOwnedRoute,
  type BypassState,
} from './tunnel-bypass-types.ts';

/**
 Largest unsigned integer accepted by Linux mark,
 route table,
 and rule preference fields.
 */
const MAX_UINT32 = 0xFF_FF_FF_FF;

/**
 Checks unknown object shape.
 
 @param value - Candidate JSON value.
 
 @returns Whether value supports field guards.
 
 @example
 ```ts
 isRecord({ version: 1 }); // true
 ```
 */
function isRecord(value: unknown,): value is Record<PropertyKey, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Checks finite unsigned integer range.
 
 @param value - Candidate numeric field.
 
 @returns Whether Linux integer field can represent value.
 
 @example
 ```ts
 isUint32(52000); // true
 ```
 */
function isUint32(value: unknown,): value is number {
  return ((typeof value) === 'number')
    && Number.isInteger(value,)
    && (value >= 0)
    && (value <= MAX_UINT32);
}

/**
 Checks persisted interface identity against the rule for interface names.
 The detached watcher trusts this name:
 it puts it into commands that run as root, such as `resolvectl dns <name>` and `wg show <name>`,
 and a state file can be written by whoever chose the runtime directory,
 so the name gets the same check as a name given on the command line
 (`isValidInterfaceName`, which also refuses a name that starts with `-`, because a command would read it as an option).

 @param value - Candidate interface name.

 @returns Whether the name is acceptable.

 @example
 ```ts
 isInterfaceName('wg0'); // true
 ```
 */
function isInterfaceName(value: unknown,): value is string {
  return ((typeof value) === 'string') && isValidInterfaceName({ interfaceName: value, },);
}

/**
 Checks nonempty ownership identity.
 
 @param value - Candidate owner token.
 
 @returns Whether token can distinguish lock ownership.
 
 @example
 ```ts
 isOwnerId('owner'); // true
 ```
 */
function isOwnerId(value: unknown,): value is string {
  return ((typeof value) === 'string') && (value !== '');
}

/**
 Narrows unknown arrays without exposing mutable element type.
 
 @param value - Candidate JSON array.
 
 @returns Whether value is array with unknown elements.
 
 @example
 ```ts
 isUnknownArray([]); // true
 ```
 */
function isUnknownArray(value: unknown,): value is readonly unknown[] {
  return Array.isArray(value,);
}

/**
 One route token as `ip route show` prints it:
 an address, an interface name, a keyword or a number.
 A token never starts with a dash, so `ip` cannot read it as an option,
 and it holds no space, quote, glob or other shell or control character.
 */
const ROUTE_TOKEN = /^[\w.:%@/][\w.:%@/-]*$/u;

/**
 Checks that a persisted route is a bypass-owned default route of the claimed table.
 The detached watcher, which runs as root, deletes these routes with `ip route delete <tokens>`,
 and a state file can be written by whoever chose the runtime directory,
 so the tokens may name only what this owner installed:
 a default route (or the unreachable fallback default),
 in the claimed table,
 with the private protocol.

 @param tokens - Candidate route tokens.

 @param table - Table the state claims.

 @returns Whether every token is plain,
 the route starts as a default route,
 every `table` key is followed by the claimed table,
 every `proto` key is followed by the private protocol,
 and both keys are present.

 @example
 ```ts
 isOwnedRouteShape({ tokens: ['default', 'dev', 'eth0', 'table', '52000', 'proto', '201'], table: 52_000 }); // true
 ```
 */
function isOwnedRouteShape(
  {
    tokens,
    table,
  }: {
    readonly tokens: readonly string[];
    readonly table: number;
  },
): boolean {
  if (!tokens.every(function isPlainToken(token,): boolean {
    return ROUTE_TOKEN.test(token,);
  },))
    return false;
  if ((tokens[0] !== 'default') && (!((tokens[0] === 'unreachable') && (tokens[1] === 'default'))))
    return false;
  /**
   Values that follow each key, per key.
   */
  const tableValues = tokens.flatMap(function afterTable(token, index,): readonly string[] {
    return token === 'table' ? [tokens[index + 1] ?? '',] : [];
  },);
  /**
   Values that follow the protocol key.
   */
  const protocolValues = tokens.flatMap(function afterProtocol(token, index,): readonly string[] {
    return token === 'proto' ? [tokens[index + 1] ?? '',] : [];
  },);
  return (tableValues.length > 0)
    && tableValues.every(function isClaimedTable(value,): boolean {
      return value === String(table,);
    },)
    && (protocolValues.length > 0)
    && protocolValues.every(function isBypassProtocol(value,): boolean {
      return value === String(BYPASS_ROUTE_PROTOCOL,);
    },);
}

/**
 Parses persisted route fingerprints.

 @param value - Candidate route array.

 @param path - State path named in diagnostics.

 @param table - Table the state claims,
 which every route must name.

 @returns Validated immutable route identities.

 @throws {@link BypassStateError} when route shape is invalid.

 @example
 ```ts
 parseOwnedRoutes({ value: [], path: '/tmp/state', table: 52_000 });
 ```
 */
function parseOwnedRoutes(
  {
    value,
    path,
    table,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly table: number;
  },
): readonly BypassOwnedRoute[] {
  if (!isUnknownArray(value,))
    throw new BypassStateError(`Invalid application-bypass route state: ${path}`,);
  /**
   Validated route copies accumulated from JSON array.
   */
  const routes: BypassOwnedRoute[] = [];
  for (const route of value) {
    if ((!isRecord(route,))
      || ((route.proto !== '-4') && (route.proto !== '-6'))
      || (!isUnknownArray(route.tokens,))
      || (route.tokens
        .length
        === 0)
      || (!route.tokens
        .every(function stringToken(token,): token is string {
          return (typeof token) === 'string';
        },))
      || (!isOwnedRouteShape({
        tokens: route.tokens,
        table,
      },))) {
      throw new BypassStateError(`Invalid application-bypass route state: ${path}`,);
    }
    routes.push({
      proto: route.proto,
      tokens: [...route.tokens,],
    },);
  }
  return routes;
}

/**
 Parses JSON with application-state diagnostic identity.
 
 @param text - JSON state text.
 
 @param path - Path named in diagnostics.
 
 @returns Parsed unknown value.
 
 @throws {@link BypassStateError} when JSON is malformed.
 
 @example
 ```ts
 parseStateJson({ text: '{}', path: '/tmp/state' });
 ```
 */
function parseStateJson(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new BypassStateError(
      `Invalid application-bypass state JSON at ${path}: ${String(error,)}`,
    );
  }
}

/**
 Parses and validates persisted bypass state.
 
 @param text - JSON state text.
 
 @param path - Path named in diagnostics.
 
 @returns Validated state.
 
 @throws {@link BypassStateError} when JSON or state shape is invalid.
 
 @example
 ```ts
 parseBypassState({ text: '{"version":2}', path: '/tmp/state' });
 ```
 
 @internal
 */
export function parseBypassState(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): BypassState {
  /**
   Parsed JSON before field validation.
   */
  const value: unknown = parseStateJson({
    text,
    path,
  },);
  if ((!isRecord(value,))
    || (value.version !== 2)
    || (!isInterfaceName(value.interfaceName,))
    || (!isUint32(value.mark,))
    || (value.mark === 0)
    || (!isUint32(value.table,))
    || (value.table === 0)
    || (!isUint32(value.preference,))
    || (value.preference === 0)
    || (!isOwnerId(value.ownerId,))
    || (!('routes' in value))) {
    throw new BypassStateError(`Invalid application-bypass state: ${path}`,);
  }
  return {
    version: 2,
    interfaceName: value.interfaceName,
    mark: value.mark,
    table: value.table,
    preference: value.preference,
    ownerId: value.ownerId,
    routes: parseOwnedRoutes({
      value: value.routes,
      path,
      table: value.table,
    },),
  };
}
