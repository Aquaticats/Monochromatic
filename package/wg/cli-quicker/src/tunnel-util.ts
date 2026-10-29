import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { run, } from './runner.ts';

/**
 Module logger for shared tunnel helpers.
 */
const l = tagged({ tag: 'tunnel-util', },);

/**
 Reports whether one address or prefix is IPv6 (contains a colon).
 
 @param prefix - Address or CIDR prefix.
 
 @returns True for IPv6 forms.
 
 @example
 ```ts
 isV6({ prefix: 'fd00::1/128' });
 ```
 */
export function isV6({ prefix, }: { readonly prefix: string; },): boolean {
  return prefix.includes(':',);
}

/**
 Selects the `ip` protocol flag for a prefix family.
 
 @param prefix - Address or CIDR prefix.
 
 @returns `-6` for IPv6, otherwise `-4`.
 
 @example
 ```ts
 protoFlag({ prefix: '::/0' });
 ```
 */
export function protoFlag({ prefix, }: { readonly prefix: string; },): '-4' | '-6' {
  return isV6({ prefix, },) ? '-6' : '-4';
}

/**
 Extracts the numeric prefix length from a CIDR string.
 
 @param prefix - CIDR prefix.
 
 @returns Prefix length, or 0 when absent.
 
 @example
 ```ts
 prefixLength({ prefix: '10.0.0.0/8' });
 ```
 */
export function prefixLength({ prefix, }: { readonly prefix: string; },): number {
  /**
   Index of the prefix-length separator, or -1 when absent.
   */
  const slash = prefix.indexOf('/',);
  return slash === (-1) ? 0 : Math.trunc(
    Number(prefix.slice(slash + 1,),),
  );
}

/**
 Executes hook commands, expanding `%i` to the interface name, like wg-quick.
 
 @param hooks - Hook command strings in declaration order.
 
 @param interfaceName - Interface name substituted for `%i`.
 
 @example
 ```ts
 await executeHooks({ hooks: ['echo up %i'], interfaceName: 'wg0' });
 ```
 */
export async function executeHooks(
  {
    hooks,
    interfaceName,
  }: {
    readonly hooks: readonly string[];
    readonly interfaceName: string;
  },
): Promise<void> {
  /**
   Function-scoped logger for hook execution.
   */
  const fl = tagged({
    tag: executeHooks.name,
    l,
  },);
  /* oxlint-disable eslint/no-await-in-loop -- Hooks run sequentially in declaration order, matching wg-quick. */
  for (const hook of hooks) {
    /**
     Hook text with every `%i` replaced by the interface name.
     */
    const expanded = hook.split('%i',)
      .join(interfaceName,);
    fl.debug(`hook: ${expanded}`,);
    await run({
      command: 'sh',
      args: [
        '-c',
        expanded,
      ],
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
