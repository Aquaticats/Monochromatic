import { runAllowingFailure, } from './runner.ts';

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
