import {
  BLOCK_EXCEPTIONS,
  trafficBlockTable,
} from './tunnel-traffic-block.ts';

/**
 Words for a caught value in a message.

 @param error - Caught value.

 @returns Its text.

 @example
 ```ts
 reason({ error: new Error('x') }); // 'Error: x'
 ```
 */
export function reason({ error, }: { readonly error: unknown; },): string {
  return String(error,);
}

/**
 Message for a `down` that stopped before it removed any routing.
 When an earlier `down` had already removed routing and left the traffic block,
 the tunnel is not fully up and the message says so,
 where a first attempt that stopped here left the tunnel fully up.

 @param interfaceName - Tunnel interface.

 @param what - The step that failed.

 @param why - Why it failed.

 @param state - What this attempt did and did not change.

 @param fix - What to do about it.

 @param earlierRemoval - Whether the traffic block of an earlier `down` is in place.

 @returns The loud message.

 @example
 ```ts
 refusedMessage({ interfaceName: 'wg0', what: 'stop the route watcher', why: 'x', state: 'nothing was removed', fix: 'y', earlierRemoval: false });
 ```
 */
export function refusedMessage(
  {
    interfaceName,
    what,
    why,
    state,
    fix,
    earlierRemoval,
  }: {
    readonly interfaceName: string;
    readonly what: string;
    readonly why: string;
    readonly state: string;
    readonly fix: string;
    readonly earlierRemoval: boolean;
  },
): string {
  /**
   What the person is told about the tunnel.
   */
  const headline = earlierRemoval
    ? `an earlier down already removed part of the routing of ${interfaceName}, so the tunnel is NOT fully up, and traffic is BLOCKED `
      + `(nft table inet ${trafficBlockTable({ interfaceName, },)} drops everything that leaves the machine ${BLOCK_EXCEPTIONS}; `
      + `see what it lets through with \`sudo nft list table inet ${trafficBlockTable({ interfaceName, },)}\`); this attempt: ${state}`
    : `the tunnel ${interfaceName} is still fully up; ${state}`;
  return `error: could not ${what} (${why})\n`
    + `${headline}\n`
    + `fix: ${fix}, then run \`wg-quicker down ${interfaceName}\` again`;
}

/**
 Message for a `down` that failed after it started removing routing.
 The traffic block was installed before the first removal,
 so traffic is blocked.

 @param interfaceName - Tunnel interface.

 @param what - The step that failed.

 @param why - Why it failed.

 @returns The loud message.

 @example
 ```ts
 incompleteMessage({ interfaceName: 'wg0', what: 'delete the interface', why: 'x' });
 ```
 */
export function incompleteMessage(
  {
    interfaceName,
    what,
    why,
  }: {
    readonly interfaceName: string;
    readonly what: string;
    readonly why: string;
  },
): string {
  /**
   Table that holds the block.
   */
  const table = trafficBlockTable({ interfaceName, },);
  return `error: down ${interfaceName} failed part way: could not ${what} (${why})\n`
    + `traffic is BLOCKED (nft table inet ${table} drops everything that leaves the machine ${BLOCK_EXCEPTIONS}, and was installed before any routing was removed), `
    + 'so nothing goes outside the tunnel unprotected except what the routing sends around it by design, and the rest of the network will not work until down is finished\n'
    + `to finish: run \`wg-quicker down ${interfaceName}\` again, with this build or with the build that started the tunnel; a down that succeeds removes the block\n`
    + `to lift the block by hand: sudo nft delete table inet ${table}`;
}

/**
 Message for a `down` that removed the tunnel but left something behind.
 When the block could not be lifted,
 traffic is still blocked and the message says how to lift it.

 @param interfaceName - Tunnel interface.

 @param leftovers - What could not be finished,
 in words.

 @param blockFailure - Why the block could not be lifted,
 absent when it was lifted.

 @returns The loud message.

 @example
 ```ts
 leftoverMessage({ interfaceName: 'wg0', leftovers: ['could not revert the DNS settings (x)'] });
 ```
 */
export function leftoverMessage(
  {
    interfaceName,
    leftovers,
    blockFailure,
  }: {
    readonly interfaceName: string;
    readonly leftovers: readonly string[];
    readonly blockFailure?: string;
  },
): string {
  /**
   Table that holds the block.
   */
  const table = trafficBlockTable({ interfaceName, },);
  /**
   Everything that went wrong,
   the block first because it stops the network.
   */
  const problems = blockFailure === undefined
    ? leftovers
    : [
      `could not lift the traffic block (${blockFailure})`,
      ...leftovers,
    ];
  /**
   What is true of traffic and routing now.
   */
  const state = blockFailure === undefined
    ? 'nothing is blocked and no routing is left over; finish these by hand'
    : `traffic is still BLOCKED (nft table inet ${table} drops everything that leaves the machine ${BLOCK_EXCEPTIONS}) and the rest of the network will not work until it is lifted: sudo nft delete table inet ${table}`;
  return `error: down ${interfaceName} removed the tunnel but ${problems.join('; ',)}\n${state}`;
}
