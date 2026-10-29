/**
 Deterministic mutant identity.
 
 @example
 ```ts
 mutantId({ file: 'src/a.ts', start: 2, end: 3, operator: 'arithmetic', replacement: '-' });
 ```
 */

import { createHash, } from 'node:crypto';

/**
 Hex length keeping ids short yet collision-safe within one package run.
 */
const ID_HEX_LENGTH = 16;

/**
 Derives a deterministic mutant id from its defining coordinates.
 
 Identical coordinates always produce identical ids across runs and
 containers, which is what makes taint-driven re-runs and survivor
 confirmation able to target one specific mutant.
 
 @param options - Coordinates defining one mutant.
 
 @returns Stable hex id.
 
 @example
 ```ts
 mutantId({ file: 'src/a.ts', start: 2, end: 3, operator: 'arithmetic', replacement: '-' });
 // '9f8e...'(16 hex chars)
 ```
 */
export function mutantId(options: {
  readonly file: string;
  readonly start: number;
  readonly end: number;
  readonly operator: string;
  readonly replacement: string;
},): string {
  return createHash('sha256',)
    .update(JSON.stringify([
      options.file,
      options.start,
      options.end,
      options.operator,
      options.replacement,
    ],),)
    .digest('hex',)
    .slice(
    0,
    ID_HEX_LENGTH,
    );
}
