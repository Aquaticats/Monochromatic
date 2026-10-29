import { caught, } from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '../dist/final/node/index.mjs';

//region Stated refusal message
// MESSAGE OF THE STATED REFUSAL A READ THREW, confirming it is the package's
// own stated-refusal class rather than some other failure.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several command-line and grade-sheet
// tests kept their own copy of this reader; all now import it from here.

/**
 Message of the stated refusal a read threw.

 @param read - read that must refuse

 @returns Refusal's message

 @throws Error when the read threw something other than a stated refusal

 @example
 ```ts
 const said = statedRefusalMessage({ read: () => lineOf({ command: 'corpus-pass', typed: ['--olny',], },), },);
 ```
 */
export function statedRefusalMessage({ read, }: { readonly read: () => unknown; },): string {
  /**
   What the read threw.
   */
  const refusal = caught(read,);
  if (!(refusal instanceof StatedRefusalError)) {
    throw new Error(
      `Expected the read to throw a StatedRefusalError, but it threw ${
        Error.isError(refusal,) ? `${refusal.name}: ${refusal.message}` : typeof refusal
      }`,
    );
  }
  return refusal.message;
}

//endregion Stated refusal message
