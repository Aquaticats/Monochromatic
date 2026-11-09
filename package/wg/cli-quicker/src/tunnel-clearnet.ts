import { isValidInterfaceName, } from './interface-name.ts';
import {
  IP_HOST_RULE_TOOLS,
  parseOwnedHostRules,
  type HostRuleTools,
  type OwnedHostRule,
} from './tunnel-bypass-hosts.ts';
import {
  BYPASS_STATE_ABSENT,
  readBypassState,
} from './tunnel-bypass-state.ts';
import type {
  BypassProto,
  BypassState,
} from './tunnel-bypass-types.ts';

/**
 Address families whose rules are listed.
 */
const FAMILIES: readonly BypassProto[] = [
  '-4',
  '-6',
];

/**
 What the routing already sends around the tunnel by design,
 read from what the routing itself matches on:
 the socket mark of the ownership state,
 which the policy rule `fwmark <mark> table <bypass table>` matches
 and which the cgroup programs of the updates slice and of the exempted applications set on their sockets,
 and the destinations of the `to <address> table <bypass table>` rules of the update hosts.
 */
export type Clearnet = {
  /**
   The exempt mark whose packets take the bypass table.
   Absent when the interface has no application bypass.
   */
  readonly mark?: number;

  /**
   The update-host destinations that take the bypass table.
   */
  readonly hosts: readonly OwnedHostRule[];

  /**
   The physical default interfaces, one entry per family and interface,
   read from the `dev` of the routes the bypass table copies from the main table.
   A reply from an update host reaches a forwarding machine through one of them,
   so the traffic block lets a forwarded reply through only when it arrives on one.
   */
  readonly interfaces: readonly PhysicalInterface[];
};

/**
 A physical interface that carries the default route of one address family.
 */
export type PhysicalInterface = {
  /**
   Address family of the default route.
   */
  readonly family: BypassProto;

  /**
   Interface name,
   valid as an interface name.
   */
  readonly name: string;
};

/**
 Reads the physical default interfaces from the routes the state owns.
 A route without a `dev`,
 such as the unreachable fallback,
 and a `dev` that is not a valid interface name name no interface.

 @param state - Ownership state whose routes copy the physical defaults.

 @returns Each family and interface once,
 in the order of the routes.

 @example
 ```ts
 physicalInterfaces({ state });
 ```
 */
export function physicalInterfaces({ state, }: { readonly state: BypassState; },): readonly PhysicalInterface[] {
  /**
   Interfaces found so far, keyed by family and name.
   */
  const found = new Map<string, PhysicalInterface>();
  for (const route of state.routes) {
    /**
     Position of the `dev` key,
     and -1 when the route has none.
     */
    const devIndex = route.tokens
      .indexOf('dev',);
    if (devIndex === (-1))
      continue;
    /**
     Value of the `dev` key,
     absent when the route ends at the keyword.
     */
    const name = route.tokens[devIndex + 1];
    if ((name === undefined) || (!isValidInterfaceName({ interfaceName: name, },)))
      continue;
    /**
     Interface the route names.
     */
    const physical: PhysicalInterface = {
      family: route.proto,
      name,
    };
    found.set(
      `${route.proto} ${name}`,
      physical,
    );
  }
  return [...found.values(),];
}

/**
 Lists the owned host rules of one family.

 @param family - Address family.

 @param state - Claimed table and preference.

 @param tools - Listing tool.

 @returns The owned destinations.

 @example
 ```ts
 await ownedHosts({ family: '-4', state, tools });
 ```
 */
async function ownedHosts(
  {
    family,
    state,
    tools,
  }: {
    readonly family: BypassProto;
    readonly state: BypassState;
    readonly tools: HostRuleTools;
  },
): Promise<readonly OwnedHostRule[]> {
  return parseOwnedHostRules({
    text: await tools.list({ family, },),
    family,
    state,
  },);
}

/**
 What reading the routing found.
 */
export type ClearnetRead =
  | {
    /**
     The ownership state is absent:
     the interface has no application bypass,
     or an earlier `down` already removed it.
     */
    readonly bypass: false;
  }
    | {
    /**
     The ownership state is there.
     */
      readonly bypass: true;

    /**
     What the routing sends around the tunnel.
     */
      readonly clearnet: Clearnet;
    };

/**
 Reads what the routing sends around the tunnel,
 from the persisted ownership state and the kernel's rules.
 The mark is the one the installed rules match,
 which a changed config may no longer name.

 @param interfaceName - Tunnel interface.

 @param readState - Reader of the ownership state,
 replaceable in tests.

 @param tools - Rule listing,
 replaceable in tests.

 @returns The mark and the update-host destinations,
 or that there is no ownership state.

 @throws {@link BypassStateError} when the ownership state cannot be read or belongs to another interface.

 @throws {@link CommandError} when `ip rule show` fails.

 @example
 ```ts
 await readClearnet({ interfaceName: 'wg0' });
 ```
 */
export async function readClearnet(
  {
    interfaceName,
    readState = readBypassState,
    tools = IP_HOST_RULE_TOOLS,
  }: {
    readonly interfaceName: string;
    readonly readState?: typeof readBypassState;
    readonly tools?: HostRuleTools;
  },
): Promise<ClearnetRead> {
  /**
   State of the application bypass,
   if any.
   */
  const state = await readState({ interfaceName, },);
  if (state === BYPASS_STATE_ABSENT)
    return { bypass: false, };
  /**
   Owned destinations of each family.
   */
  const perFamily = await Promise.all(FAMILIES.map(function listFamily(family,): Promise<readonly OwnedHostRule[]> {
    return ownedHosts({
      family,
      state,
      tools,
    },);
  },),);
  return {
    bypass: true,
    clearnet: {
      mark: state.mark,
      hosts: perFamily.flat(),
      interfaces: physicalInterfaces({ state, },),
    },
  };
}
