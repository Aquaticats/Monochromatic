import { CommandError, } from './errors.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';

/**
 Reports whether interface link currently exists.
 
 @param interfaceName - Interface to probe.
 
 @returns Whether link is present.
 
 @example
 ```ts
 await linkExists({ interfaceName: 'wg0' });
 ```
 */
export async function linkExists(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<boolean> {
  /**
   Link-show probe for interface.
   */
  const result = await runAllowingFailure({
    command: 'ip',
    args: [
      'link',
      'show',
      'dev',
      interfaceName,
    ],
  },);
  return result.exitCode === 0;
}

/**
 Reports whether a failed `ip link add` was refused because the link was already there.

 The kernel answers `RTNETLINK answers: File exists` and creates nothing,
 so the link that refused this start belongs to the start that made it first,
 and a rollback must leave it alone.

 Every other failure is uncertain: the kernel may have made the link and the answer
 may have been lost, as it is when the deadline or an interrupt ends `ip` after the
 kernel call. `up` claims the link in that case, so the rollback removes it.

 @param failure - What the `ip link add` threw.

 @returns Whether the refusal says the link already existed.

 @example
 ```ts
 linkAlreadyExisted({ failure: error });
 ```
 */
export function linkAlreadyExisted({ failure, }: { readonly failure: unknown; },): boolean {
  if (!(failure instanceof CommandError))
    return false;
  return failure.stderr
    .includes('File exists',);
}

/**
 Creates the WireGuard kernel link for the interface,
 and records it as made by this start when this start made it.

 A refused `ip link add` that says the link already existed made nothing:
 the link that refused it belongs to the start that made it first,
 so recording it would make a rollback take down another start's tunnel.
 Any other failure is uncertain: the kernel may have made the link and the answer
 may have been lost, as it is when the deadline or an interrupt ends `ip` after the
 kernel call, so the link is recorded when it is there and the rollback removes it.

 The failure is rethrown either way, so a step that failed still fails.

 @param interfaceName - Interface to create.

 @param recordMade - Records the link as this start's, for the rollback.

 @throws What `ip link add` threw, which is what ends the step.

 @example
 ```ts
 await createLink({ interfaceName: 'wg0', recordMade });
 ```
 */
export async function createLink(
  {
    interfaceName,
    recordMade,
  }: {
    readonly interfaceName: string;
    readonly recordMade: () => void;
  },
): Promise<void> {
  try {
    await run({
      command: 'ip',
      args: [
        'link',
        'add',
        'dev',
        interfaceName,
        'type',
        'wireguard',
      ],
    },);
    recordMade();
  }
  catch (error: unknown) {
    if ((!linkAlreadyExisted({ failure: error, },)) && (await linkExists({ interfaceName, },)))
      recordMade();
    throw error;
  }
}
