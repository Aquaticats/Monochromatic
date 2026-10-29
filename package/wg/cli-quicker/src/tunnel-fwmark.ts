import {
  runAllowingFailure,
} from './runner.ts';

/**
 Result of probing an interface fwmark for its policy table.
 */
export type FwmarkProbe = {
  /**
   Whether a usable positive table number is present.
   */
  readonly found: boolean;

  /**
   Policy table number when `found`, else `0`.
   */
  readonly table: number;
};

/**
 Discovers the live interface fwmark, which names its policy table.
 
 @param interfaceName - Interface to query.
 
 @returns A result whose `found` flag guards the policy `table` number.
 
 @example
 ```ts
 await readFwmark({ interfaceName: 'wg0' });
 ```
 */
export async function readFwmark(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<FwmarkProbe> {
  /**
   Live fwmark reported by `wg show fwmark`.
   */
  const result = await runAllowingFailure({
    command: 'wg',
    args: [
      'show',
      interfaceName,
      'fwmark',
    ],
  },);
  /**
   Trimmed fwmark text, `off`/`0` meaning unset.
   */
  const text = result.stdout
    .trim();
  /**
   Parsed table number carried in the fwmark.
   */
  const table = Math.trunc(Number(text,),);
  /**
   Whether a usable positive table number is present.
   */
  const found = (text !== 'off') && (text !== '0')
    && (text !== '')
    && Number.isSafeInteger(table,)
    && (table > 0);
  return {
    found,
    table: found ? table : 0,
  };
}
