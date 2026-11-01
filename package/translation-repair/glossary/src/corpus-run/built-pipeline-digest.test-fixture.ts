/**
 The digest of the built output, which the as-built suites of the score
 commands expect the pool to print as the pipeline that read it.

 TEST SUPPORT, NOT PACKAGE SOURCE. No child is started here; the as-built
 suites start theirs through `runBuiltCommand` of
 `child-environment.test-fixture.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { builtDirectoryOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/built-package-directory.test-fixture.ts';
import { digestPipeline, } from '../../dist/final/node/index.mjs';

/**
 Digest of the built output of the package that owns the command, which the pool prints as the pipeline that read it.

 @param packageEntry - URL of the built index of the package that owns the command the case starts, as
 `import.meta.resolve` gives it for that package's name

 @returns Digest text, as the pool's `POOL read by pipeline` line carries it

 @example
 ```ts
 const stamp = await builtPipelineDigest({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-probe'), },);
 ```
 */
export async function builtPipelineDigest({ packageEntry, }: { readonly packageEntry: string; },): Promise<string> {
  /**
   Stamp of the directory the built commands and the index sit in.
   */
  const { digest, } = await digestPipeline({ dir: builtDirectoryOf({ packageEntry, },), },);
  return digest;
}
