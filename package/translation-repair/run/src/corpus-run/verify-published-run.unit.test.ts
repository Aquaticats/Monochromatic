/**
 Tests for how `verify-published` reads a whole run: the artifacts and the
 published tree of a throwaway runs directory listed, paired, judged entry by
 entry and reported, with the exit code the command leaves.

 What is printed is read off `console.log` through a diverting capture, which
 is process-wide, so the suite runs one case at a time.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type DisposableSandbox,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { UnansweredContestSliceError, } from '@monochromatic-dev/translation-repair-page-check';
import {
  judgePublishedEntries,
  verifyPublishedRun,
} from '../../dist/final/node/index.mjs';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  artifactPathOf,
  artifactTextWithoutArchive,
  BUILT_PIPELINE_DIR,
  builtPipelineDigest,
  declineEntry,
  publishedPagePath,
  publishedRun,
  SETTLED_BY,
  strayPage,
} from '@monochromatic-dev/translation-repair-page-check/ts/corpus-run/published-run.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { interimEntry, } from '@monochromatic-dev/translation-repair-page-check/ts/corpus-run/interim-entry.test-fixture.ts';

/**
 Ending of the closing line, which says how many pages had no length to check.
 */
const UNWEIGHED_NOTE = 'UNWEIGHED because the artifact predates the stored archive text';

/**
 Entry the fixture settles, named so a run of several keeps one order.

 @param index - position among the entries

 @returns The entry id

 @example
 ```ts
 const entryId = catEntry({ index: 2, },);
 ```
 */
function catEntry({ index, }: { readonly index: number; },): string {
  return `CatEntry${ String(index,).padStart(
    3,
    '0',
  ) }`;
}

/**
 What one verification returned and printed.

 @example
 ```ts
 const verified: Verified = { code: 0, lines: [], };
 ```
 */
type Verified = {
  /**
   Exit code the command would leave.
   */
  readonly code: number;

  /**
   Lines it printed, in order.
   */
  readonly lines: readonly string[];
};

/**
 Verifies a runs directory as the built pipeline reads it and keeps what was
 printed.

 @param sinon - calling case's own sandbox, whose stub captures the printing

 @param runsDir - run directory to verify

 @returns The exit code and the printed lines

 @example
 ```ts
 const { code, lines, } = await verified({ sinon: ctx.sinon, runsDir, },);
 ```
 */
async function verified(
  {
    sinon,
    runsDir,
  }: {
    readonly sinon: DisposableSandbox;
    readonly runsDir: string;
  },
): Promise<Verified> {
  using printed = divertingConsoleLog({ sinon, },);

  /**
   What the verification returned, before the capture is read.
   */
  const code = await verifyPublishedRun({
    runsDir,
    pipelineDir: BUILT_PIPELINE_DIR,
  },);
  return {
    code,
    lines: printed.lines,
  };
}

await describe({
  name: verifyPublishedRun.name,
  concurrency: 1,
  children: [
    it({
      name: 'REPORTS a run whose one page agrees with its artifact in three lines and returns 0',
      fn: async (ctx) => {
        await using published = await publishedRun();
        const { runsDir, } = published;

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0',
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            `verify-published: 1 of 1 page carries every wording its artifact promised; 1 of those at the length it implies, 0 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'REPORTS several pages in the plural, each entry in the order the run lists them',
      fn: async (ctx) => {
        await using published = await publishedRun({
          entryIds: [
            catEntry({ index: 1, },),
            catEntry({ index: 2, },),
          ],
        },);
        const { runsDir, } = published;

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: matched=2 settledWithNoPage=0 pageWithNoArtifact=0 declined=0',
            'CatEntry001: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            'CatEntry002: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            `verify-published: 2 of 2 pages carry every wording their artifacts promised; 2 of those at the length it implies, 0 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS sixty entries in the order the run lists them, whichever page is read first',
      fn: async (ctx) => {
        /**
         Entry ids in the order the run lists them.
         */
        const entryIds = Array.from(
          { length: 60, },
          function entryAt(_unused, index,): string {
            return catEntry({ index, },);
          },
        );
        await using published = await publishedRun({ entryIds, },);
        const { runsDir, } = published;

        /**
         The entry lines of the report, which start with the id.
         */
        const entryLines = (await verified({
          sinon: ctx.sinon,
          runsDir,
        },)).lines
          .filter(function isEntryLine(line,): boolean {
            return line.startsWith('CatEntry',);
          },)
          .map(function idOf(line,): string {
            return line.slice(
              0,
              line.indexOf(':',),
            );
          },);

        expect(entryLines,).toEqual(entryIds,);
      },
    },),
    it({
      name: 'COUNTS an entry whose artifact will not read as a page that disagrees, and says one page carries its wordings in the singular',
      fn: async (ctx) => {
        await using published = await publishedRun({
          entryIds: [
            catEntry({ index: 1, },),
            catEntry({ index: 2, },),
          ],
        },);
        const { runsDir, } = published;
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: catEntry({ index: 2, },),
          },),
          '{ not json',
        );

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: matched=2 settledWithNoPage=0 pageWithNoArtifact=0 declined=0',
            'CatEntry001: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            'CatEntry002: REFUSED by SyntaxError',
            `verify-published: 1 of 2 pages carries every wording its artifact promised; 1 of those at the length it implies, 0 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'COUNTS a weighed page, an unweighed page and a page that lost text apart in the closing line, with the page\'s findings',
      fn: async (ctx) => {
        await using published = await publishedRun({
          entryIds: [
            catEntry({ index: 1, },),
            catEntry({ index: 2, },),
            catEntry({ index: 3, },),
          ],
        },);
        const { runsDir, } = published;
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: catEntry({ index: 2, },),
          },),
          artifactTextWithoutArchive({ entryId: catEntry({ index: 2, },), },),
        );
        await rm(
          publishedPagePath({
            runsDir,
            entryId: catEntry({ index: 3, },),
          },),
        );
        await writeFile(
          publishedPagePath({
            runsDir,
            entryId: catEntry({ index: 3, },),
          },),
          'The cat has gone.\n',
        );

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: matched=3 settledWithNoPage=0 pageWithNoArtifact=0 declined=0',
            'CatEntry001: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            'CatEntry002: wordings=2 silent=0 source-shipped=0 chars=UNWEIGHED(artifact predates stored archive text) '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            'CatEntry003: wordings=2 silent=0 source-shipped=0 chars=18/expected 81 '
              + 'missing=2 consistency-groups=0 read-at-generation=21',
            '  WRONG LENGTH: page is -63 characters off what the archive plus every slice change comes to. '
              + 'Text no slice decided on was lost or added',
            '  MISSING slice 0, 43 characters the page does not carry in order',
            '  MISSING slice 1, 35 characters the page does not carry in order',
            `  READ BY ANOTHER BUILD: settled by ${ SETTLED_BY }, read by ${ await builtPipelineDigest() }; `
              + 'a pass started here on this build rewrites the page to this reading',
            `verify-published: 2 of 3 pages carry every wording their artifacts promised; 1 of those at the length it implies, 1 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'NAMES the settled entries a missing published tree leaves unpublished, in the plural',
      fn: async (ctx) => {
        await using published = await publishedRun({
          entryIds: [
            catEntry({ index: 1, },),
            catEntry({ index: 2, },),
          ],
        },);
        const { runsDir, } = published;
        await rm(
          join(
            runsDir,
            'fixed',
          ),
          { recursive: true, },
        );

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: NO PUBLISHED TREE (ENOENT). 2 settled entries are unpublished, and the next pass started in '
              + 'this runs directory writes each from its artifact',
            'verify-published: matched=0 settledWithNoPage=2 pageWithNoArtifact=0 declined=0',
            '  SETTLED AND NEVER PUBLISHED: CatEntry001. The archive ships for it until the next pass started in this runs '
              + 'directory writes it from its artifact',
            '  SETTLED AND NEVER PUBLISHED: CatEntry002. The archive ships for it until the next pass started in this runs '
              + 'directory writes it from its artifact',
            `verify-published: 0 of 0 pages carry every wording their artifacts promised; 0 of those at the length it implies, 0 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'NAMES the one settled entry a missing published tree leaves unpublished, in the singular',
      fn: async (ctx) => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await rm(
          join(
            runsDir,
            'fixed',
          ),
          { recursive: true, },
        );

        expect((await verified({
          sinon: ctx.sinon,
          runsDir,
        },)).lines[0],).toBe(
          'verify-published: NO PUBLISHED TREE (ENOENT). 1 settled entry is unpublished, and the next pass started in '
            + 'this runs directory writes each from its artifact',
        );
      },
    },),
    it({
      name: 'NAMES a page with no artifact apart from a declined entry that has a page, and counts the declined',
      fn: async (ctx) => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await strayPage({
          runsDir,
          entryId: 'StrayCat2',
        },);
        await strayPage({
          runsDir,
          entryId: 'DeclinedCat3',
        },);
        await declineEntry({
          runsDir,
          entryId: 'DeclinedCat3',
        },);
        await declineEntry({
          runsDir,
          entryId: 'DeclinedCat4',
        },);

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=2 declined=2',
            '  DECLINED AND PUBLISHED ANYWAY: DeclinedCat3. The archive\'s note says the page is the author\'s own English, '
              + 'so no page should stand here; the next pass started in this runs directory removes it',
            '  PUBLISHED AND NOT SETTLED: StrayCat2. It ships as it stands, with no artifact to check it against, until a '
              + 'pass settles the entry again and overwrites it',
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            `verify-published: 1 of 1 page carries every wording its artifact promised; 1 of those at the length it implies, 0 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'NAMES a page a dry stop shipped for an entry with no artifact as interim, with its slice counts, and not as '
        + 'a stray page',
      fn: async (ctx) => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await interimEntry({
          runsDir,
          entryId: 'InterimCat5',
          settledSlices: 1,
          archiveSlices: 2,
          sourceSlices: 0,
        },);

        expect(await verified({
          sinon: ctx.sinon,
          runsDir,
        },),).toEqual({
          code: 0,
          lines: [
            'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=1 declined=0',
            '  INTERIM PAGE: InterimCat5. A dry stop shipped it with 1 settled slice, 2 archive slices and 0 original '
              + 'slices; it stays open until a pass settles the entry again and overwrites it',
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=21',
            `verify-published: 1 of 1 page carries every wording its artifact promised; 1 of those at the length it implies, 0 ${ UNWEIGHED_NOTE }`,
          ],
        },);
      },
    },),
    it({
      name: 'NAMES an interim page whose bytes are not the page its record was written for, so a page edited or '
        + 'replaced after the stop does not pass as the interim page',
      fn: async (ctx) => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await interimEntry({
          runsDir,
          entryId: 'InterimCat6',
          settledSlices: 0,
          archiveSlices: 2,
          sourceSlices: 1,
          recordedPageText: 'The cat waited somewhere else.\n',
        },);

        expect((await verified({
          sinon: ctx.sinon,
          runsDir,
        },)).lines,).toContain(
          '  INTERIM PAGE DISAGREES WITH ITS RECORD: InterimCat6. The page on disk is not the page the record was '
            + 'written for; the next pass that reaches the entry overwrites it',
        );
      },
    },),
    it({
      name: 'RETURNS 2 and says nothing was verified when the run has no artifacts directory',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'verify-published-run-bare-', },);

        expect(await verified({
          sinon: ctx.sinon,
          runsDir: scratch.path,
        },),).toEqual({
          code: 2,
          lines: [
            'verify-published: NOTHING VERIFIED, no artifacts directory under the run (ENOENT). No page was read and no '
              + 'artifact was compared, so this is not a clean run',
          ],
        },);
      },
    },),
    it({
      name: 'RETURNS 2 and says nothing was verified when the artifacts directory holds no artifact',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'verify-published-run-hollow-', },);
        await mkdir(join(
          scratch.path,
          'artifacts',
        ),);

        expect(await verified({
          sinon: ctx.sinon,
          runsDir: scratch.path,
        },),).toEqual({
          code: 2,
          lines: [
            'verify-published: NOTHING VERIFIED, the artifacts directory holds no settled artifact. No page was read and no '
              + 'artifact was compared, so this is not a clean run',
          ],
        },);
      },
    },),
    it({
      name: 'REJECTS with the decline directory\'s own refusal when the directory cannot be listed, before any entry is read',
      fn: async (ctx) => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await writeFile(
          join(
            runsDir,
            'declined',
          ),
          'not a directory',
        );
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

        /**
         What the verification rejected with.
         */
        const refusal = await rejectionOf({
          promise: verifyPublishedRun({
            runsDir,
            pipelineDir: BUILT_PIPELINE_DIR,
          },),
        },);

        expect(String(refusal,),).toBe(
          'DeclinedEntriesUnreadableError: the decline directory could not be listed (ENOTDIR); a pass reading it as no '
            + 'declines would re-run every declined entry',
        );
        expect(printed.lines,).toEqual([]);
      },
    },),
    it({
      name: 'REJECTS with the first listed entry\'s judgement when two judgements throw and the later entry\'s '
        + 'throws first',
      fn: async () => {
        /**
         The two refusals the judgements end in.
         */
        const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
        /**
         What judging both entries rejected with.
         */
        const refusal = await rejectionOf({
          promise: judgePublishedEntries({
            entryIds: [
              catEntry({ index: 1, },),
              catEntry({ index: 2, },),
            ],
            judge: async function throwsSecondFirst({ entryId, },): Promise<never> {
              return await ((entryId === catEntry({ index: 1, },))
                ? refuseAfterThat(new UnansweredContestSliceError({ sliceIndex: 3, },),)
                : refuseAtOnce(new UnansweredContestSliceError({ sliceIndex: 5, },),));
            },
          },),
        },);

        expect(refusal,).toBeInstanceOf(UnansweredContestSliceError,);
        expect(String(refusal,),).toBe(
          'UnansweredContestSliceError: slice 3 differs across lanes and the contest names it nowhere',
        );
      },
    },),
  ],
},);
