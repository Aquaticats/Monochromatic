//region Ordered page reads
// A PAGE READER WHOSE REFUSALS END IN AN ORDER A CASE CHOOSES.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. A site that reads several pages gets one
// refusal per page, each naming its own page. A case hands the site this
// reader in place of `readCorpusFile`: every read refuses with git's text for
// an absent path and the probe's answer the case names (a commit the clone
// holds, so each page is absent there, or one it lacks), the reads whose path
// holds `endsLast` refuse only after every other read has, and the rest refuse
// at once.

import {
  CorpusReadError,
  type readCorpusFile,
} from '../../dist/final/node/index.mjs';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';

/**
 Makes a reader whose every read refuses, in an order the case fixes.

 @param endsLast - text a path holds when its read refuses after the others

 @param commit - what the probe of the pinned commit found, which decides
 whether each refusal is a page absent at a held commit or a commit the clone
 lacks

 @returns A reader to pass in place of `readCorpusFile`

 @example
 ```ts
 const readFile = pageReadsRefusingLastFor({ endsLast: '/page.md', commit: 'held', },);
 ```
 */
export function pageReadsRefusingLastFor(
  {
    endsLast,
    commit,
  }: {
    readonly endsLast: string;
    readonly commit: ConstructorParameters<typeof CorpusReadError>[0]['commit'];
  },
): typeof readCorpusFile {
  /**
   The two refusals the reads end in.
   */
  const {
    refuseAtOnce,
    refuseAfterThat,
  } = refusalOrder();
  return async function refusesLastFor(
    {
      pin,
      relPath,
    }: Parameters<typeof readCorpusFile>[0],
  ): Promise<string> {
    /**
     What git says of a path the commit does not carry, with the probe's
     answer the case named.
     */
    const absent = new CorpusReadError({
      detail: `${pin.commitSha}:${relPath}`,
      cause: { stderr: `fatal: path '${relPath}' does not exist in '${pin.commitSha}'`, },
      commit,
    },);
    return await (relPath.includes(endsLast,) ? refuseAfterThat(absent,) : refuseAtOnce(absent,));
  };
}

//endregion Ordered page reads
