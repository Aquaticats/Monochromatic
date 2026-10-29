/**
 Tests for how `verify-published` reads and judges one entry: the artifact and
 the page read from a throwaway runs directory, judged, and put as one line plus
 a line for each finding.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  consistencyColumnOf,
  judgePublishedEntry,
} from '../../dist/final/node/index.mjs';

import {
  artifactPathOf,
  artifactTextWithoutArchive,
  ENTRY,
  OTHER_BUILD,
  publishedRun,
  SETTLED_BY,
  sourceDroppedRunOf,
  TARGET_DOC,
} from '@monochromatic-dev/translation-repair-page-check/ts/corpus-run/published-run.test-fixture.ts';

/**
 Rewrites the fixture page, replacing one phrase.

 @param pagePath - page to rewrite

 @param from - phrase the page carries

 @param to - phrase to put there

 @example
 ```ts
 await rewritePage({ pagePath, from: 'The cat has a bowl.', to: 'The cat has.', },);
 ```
 */
async function rewritePage(
  {
    pagePath,
    from,
    to,
  }: {
    readonly pagePath: string;
    readonly from: string;
    readonly to: string;
  },
): Promise<void> {
  await writeFile(
    pagePath,
    (await readFile(
      pagePath,
      'utf8',
    )).replace(
      from,
      to,
    ),
  );
}

await describe({
  name: judgePublishedEntry.name,
  children: [
    it({
      name: 'REPORTS a page that carries every wording at the length its artifact implies, and says nothing more',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'agreed-weighed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=22',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS THE SOURCE-DROPPED LINE for a page the composition ships without the original of a slice: the '
        + 'entry, page-source-unparseable and the slice, and agrees',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        const {
          artifactText,
          pageText,
        } = sourceDroppedRunOf({ entryId: ENTRY, },);
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: ENTRY,
          },),
          artifactText,
        );
        await writeFile(
          pagePath,
          pageText,
        );

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'agreed-weighed',
          lines: [
            'CatEntry1: wordings=1 silent=1 source-shipped=0 chars=43=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=22',
            'CatEntry1: page-source-unparseable, the page ships without the source text of slices [1]',
          ],
        },);
      },
    },),
    it({
      name: 'REPORTS a page agreeing with an artifact another build settled without a word about the other build',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: OTHER_BUILD,
        },),).toEqual({
          agreement: 'agreed-weighed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=0 consistency-groups=0 read-at-generation=22',
          ],
        },);
      },
    },),
    it({
      name: 'NAMES an artifact that predates the stored archive text as unweighed rather than printing a size',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: ENTRY,
          },),
          artifactTextWithoutArchive({ entryId: ENTRY, },),
        );

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'agreed-unweighed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=UNWEIGHED(artifact predates stored archive text) '
              + 'missing=0 consistency-groups=0 read-at-generation=22',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS the wrong length, the missing slice and the other build that read it, for a page that lost text',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rewritePage({
          pagePath,
          from: 'The cat has a bowl.',
          to: 'The cat has.',
        },);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: OTHER_BUILD,
        },),).toEqual({
          agreement: 'disagreed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=74/expected 81 '
              + 'missing=1 consistency-groups=0 read-at-generation=22',
            '  WRONG LENGTH: page is -7 characters off what the archive plus every slice change comes to. '
              + 'Text no slice decided on was lost or added',
            '  MISSING slice 1, 35 characters the page does not carry in order',
            `  READ BY ANOTHER BUILD: settled by ${ SETTLED_BY }, read by ${ OTHER_BUILD }; `
              + 'a pass started here on this build rewrites the page to this reading',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS no word about another build when the build that settled the artifact is the one reading',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rewritePage({
          pagePath,
          from: 'The cat has a bowl.',
          to: 'The cat has.',
        },);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=74/expected 81 '
              + 'missing=1 consistency-groups=0 read-at-generation=22',
            '  WRONG LENGTH: page is -7 characters off what the archive plus every slice change comes to. '
              + 'Text no slice decided on was lost or added',
            '  MISSING slice 1, 35 characters the page does not carry in order',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS the missing slice without a wrong length for a page of the right length with a wording changed',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rewritePage({
          pagePath,
          from: 'The cat has a bowl.',
          to: 'The cat has a ball.',
        },);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
              + 'missing=1 consistency-groups=0 read-at-generation=22',
            '  MISSING slice 1, 35 characters the page does not carry in order',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS the wrong length in the singular for a page one character too long',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rewritePage({
          pagePath,
          from: 'The cat has a bowl.\n',
          to: 'The cat has a bowl.\n\n',
        },);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=82/expected 81 '
              + 'missing=0 consistency-groups=0 read-at-generation=22',
            '  WRONG LENGTH: page is 1 character off what the archive plus every slice change comes to. '
              + 'Text no slice decided on was lost or added',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS the wrong length in the singular for a page one character too short',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rewritePage({
          pagePath,
          from: 'The cat has a bowl.',
          to: 'The cat has a bowl',
        },);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: [
            'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=80/expected 81 '
              + 'missing=1 consistency-groups=0 read-at-generation=22',
            '  WRONG LENGTH: page is -1 character off what the archive plus every slice change comes to. '
              + 'Text no slice decided on was lost or added',
            '  MISSING slice 1, 35 characters the page does not carry in order',
          ],
        },);
      },
    },),
    it({
      name: 'REFUSES an entry whose artifact is absent, naming the class that refused it',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await rm(artifactPathOf({
          runsDir,
          entryId: ENTRY,
        },),);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: ['CatEntry1: REFUSED by Error',],
        },);
      },
    },),
    it({
      name: 'REFUSES an entry whose page is absent, naming the class that refused it',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rm(pagePath,);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: ['CatEntry1: REFUSED by Error',],
        },);
      },
    },),
    it({
      name: 'REFUSES an artifact that is not JSON, naming the class and quoting nothing of the file',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: ENTRY,
          },),
          '{ not json',
        );

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: ['CatEntry1: REFUSED by SyntaxError',],
        },);
      },
    },),
    it({
      name: 'REFUSES an artifact that parses as JSON but is not a settled artifact, naming the class',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: ENTRY,
          },),
          '{}',
        );

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: ['CatEntry1: REFUSED by ArtifactParseError',],
        },);
      },
    },),
    it({
      name: 'MARKS the expected length a floor, since a filled anchor puts separators in the page nobody counted',
      fn: async () => {
        await using published = await publishedRun({
          documents: {
            sourceText: '---\ntitle: 猫的一天\n---\n\n## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n',
            targetText: TARGET_DOC,
          },
          lanesAnswered: true,
        },);
        const { runsDir, } = published;

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'agreed-weighed',
          lines: [
            'CatEntry1: wordings=3 silent=0 source-shipped=0 chars=113/expected 111+separators '
              + 'missing=0 consistency-groups=0 read-at-generation=22',
          ],
        },);
      },
    },),
    it({
      name: 'PRINTS a missing slice of one character in the singular',
      fn: async () => {
        await using published = await publishedRun({
          documents: {
            sourceText: '猫。\n',
            targetText: 'x\n',
          },
        },);
        const {
          runsDir,
          pagePath,
        } = published;
        await rewritePage({
          pagePath,
          from: 'x',
          to: 'y',
        },);

        expect(await judgePublishedEntry({
          runsDir,
          entryId: ENTRY,
          thisBuild: SETTLED_BY,
        },),).toEqual({
          agreement: 'disagreed',
          lines: [
            'CatEntry1: wordings=1 silent=0 source-shipped=0 chars=2=expected '
              + 'missing=1 consistency-groups=0 read-at-generation=22',
            '  MISSING slice 0, 1 character the page does not carry in order',
          ],
        },);
      },
    },),
    describe({
      name: consistencyColumnOf.name,
      children: [
        it({
          name: 'PRINTS HOW MANY GROUPS THE CONSISTENCY SECTION HOLDS AND THE GENERATION IT WAS READ AT, so a replay of an '
            + 'old artifact shows that the section was read, empty or not',
          fn: async () => {
            expect([
              consistencyColumnOf({ groups: 0, generation: 15, },),
              consistencyColumnOf({ groups: 1, generation: 16, },),
              consistencyColumnOf({ groups: 12, generation: 17, },),
            ],).toEqual([
              'consistency-groups=0 read-at-generation=15',
              'consistency-groups=1 read-at-generation=16',
              'consistency-groups=12 read-at-generation=17',
            ],);
          },
        },),
      ],
    },),
  ],
},);
