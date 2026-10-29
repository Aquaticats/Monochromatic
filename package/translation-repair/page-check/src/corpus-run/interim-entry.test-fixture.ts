/**
 Test-only interim pages: the page and the record a pass leaves for an entry it stopped dry in the middle of, for the
 suite of `verify-published`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { createHash, } from 'node:crypto';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  publishedPagePath,
  SETTLED_BY,
} from './published-run.test-fixture.ts';

/**
 Length of a commit hash in hexadecimal.
 */
const COMMIT_HEX_LENGTH = 40;

/**
 Page an interim record's digest is taken over, unless a case says otherwise.
 */
const INTERIM_PAGE_TEXT = 'The cat waits.\n';

/**
 Lays down the page and the record a pass leaves for an entry it stopped dry in the middle of, which has no artifact.

 @param runsDir - run directory the tree and the records are under

 @param entryId - entry stopped

 @param settledSlices - slices of the page that carry a settled wording

 @param archiveSlices - slices of the page that carry the archive's text

 @param sourceSlices - slices of the page that carry the original under the note

 @param recordedPageText - text the record's digest is taken over, the page's own when absent, so a case can make the
 record describe another page

 @example
 ```ts
 await interimEntry({ runsDir, entryId: 'InterimCat5', settledSlices: 1, archiveSlices: 2, sourceSlices: 0, },);
 ```
 */
export async function interimEntry(
  {
    runsDir,
    entryId,
    settledSlices,
    archiveSlices,
    sourceSlices,
    recordedPageText = INTERIM_PAGE_TEXT,
  }: {
    readonly runsDir: string;
    readonly entryId: string;
    readonly settledSlices: number;
    readonly archiveSlices: number;
    readonly sourceSlices: number;
    readonly recordedPageText?: string;
  },
): Promise<void> {
  await mkdir(
    dirname(publishedPagePath({
      runsDir,
      entryId,
    },),),
    { recursive: true, },
  );
  await writeFile(
    publishedPagePath({
      runsDir,
      entryId,
    },),
    INTERIM_PAGE_TEXT,
  );
  /**
   Where the interim records live.
   */
  const interimDir = join(
    runsDir,
    'interim',
  );
  await mkdir(
    interimDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      interimDir,
      `${entryId}.json`,
    ),
    `${JSON.stringify(
      {
        id: entryId,
        tip: 'a'.repeat(COMMIT_HEX_LENGTH,),
        pipelineDigest: SETTLED_BY,
        corpusSha: 'b'.repeat(COMMIT_HEX_LENGTH,),
        timestamp: '2026-10-29T12:00:00.000Z',
        reason: 'dry-stop',
        settledSlices,
        archiveSlices,
        sourceSlices,
        pageSha256: createHash('sha256',)
          .update(recordedPageText,)
          .digest('hex',),
      },
      undefined,
      2,
    )}\n`,
  );
}
