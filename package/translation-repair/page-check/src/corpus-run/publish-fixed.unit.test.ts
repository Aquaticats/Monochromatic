/**
 Tests for publishing one settled entry as a page in the mirrored corpus tree.

 DRIVEN THROUGH THE REAL ASSEMBLER, `spliceSlices`, rather than a stub of it.
 The publisher's whole job is to put the deciders' answers back where they
 came from, and every way of getting that wrong lives in the join between a
 reading and the span it belongs to: an index read positionally, an offset
 recovered by searching for text, a slice written twice. A fixture assembler
 would agree with whatever the publisher did.

 THE ARCHIVE HERE IS A WHOLE LITTLE DOCUMENT, not one paragraph, because the
 property that matters most is what the publisher does NOT touch. A page
 assembled correctly is byte-identical outside the slices that were replaced,
 and only text either side of a replaced span can show that.

 THE TWO PURE SUBJECTS LIVE IN `publish-fixed-replacements.unit.test.ts`, apart
 from these. A file is abandoned once any describe in it fails, so while they
 shared one, a break in the replacement builder left every case here unrun and
 unreported: the runner named one narrow failure where the real blast radius
 was every page the pass writes.

 ONE CASE PROVES A BRANCH THE CORPUS CANNOT REACH. No slice in any settled
 artifact on disk carries an archive that holds no wording, 249 of 249 at the
 last count, so the silent readings are unreachable there and a measurement
 over real output can only report zero. Whether the publisher does the right
 thing when a decider ships nothing into a passage the original speaks is
 therefore settled here, by a fixture built to be in that state.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { existsSync, } from 'node:fs';
import {
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { wrapReplacementText, } from '@monochromatic-dev/translation-repair-typography';
import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  SliceSpliceError,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type DestinationCheck,
  fixedPagePath,
  type PublishDefect,
  PublishedPageDisagreesError,
  publishFixedPage,
  UnparseablePageError,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { artifactWithAnUnfilledAnchor, } from './would-ship-unfilled-anchor.test-fixture.ts';
import { sourceShippedArtifact, } from './source-shipped.test-fixture.ts';

//region The archive this entry starts from

/**
 Opening paragraph, which no case ever replaces.

 ITS JOB IS TO STAY PUT. Every case that replaces a later slice asserts this
 text survived unchanged, which is what distinguishes a publisher that wrote
 one span from one that rebuilt the document out of the pieces it knew about.
 */
const OPENING = '## Description\n\nA tabby who kept the bookshop company for eleven years.\n';

/**
 Middle paragraph, the slice the deciders act on.
 */
const ARCHIVE_MIDDLE = '\nShe slept on the counter by the till.\n';

/**
 Decided wording ending on two characters beyond the first plane, each two
 UTF-16 units and one character: the page it makes is 180 characters, 182
 units.
 */
const PAWED_MIDDLE = '\nShe slept on the counter by the till, paws up \u{1F43E}\u{1F43E}.\n';

/**
 Closing paragraph, which no case replaces either.
 */
const CLOSING = '\n## Remembered by\n\nEveryone who came in out of the rain.\n';

/**
 Whole archive English, as the corpus holds it.

 ENDS IN EXACTLY ONE NEWLINE, and every case that publishes an untouched
 entry asserts the published bytes still do. Nothing in the publisher may
 normalize a document's ending: the corpus went to subprocess-level lengths
 to preserve it on the way in.
 */
const ARCHIVE = `${OPENING}${ARCHIVE_MIDDLE}${CLOSING}`;

/**
 Source page the archive translates, linking nowhere, so the destination check
 has nothing to report unless a case says otherwise.
 */
const SOURCE_PAGE = '## 简介\n\n一只在书店陪了十一年的虎斑猫。\n';

/**
 Where the middle paragraph starts.
 */
const MIDDLE_START = OPENING.length;

/**
 Where it ends.
 */
const MIDDLE_END = MIDDLE_START + ARCHIVE_MIDDLE.length;

/**
 Wording a decider settled on for that middle slice.
 */
const DECIDED_MIDDLE = '\nShe slept on the counter beside the till, in the sun.\n';

/**
 Closing paragraph of an archive that rendered the source's home link at
 another address, the shape the either-rendering rule of 2026-09-04 accepts.
 */
const CLOSING_LINKED = '\n## Remembered by\n\nEveryone who came in out of the rain, and [her page](https://example.net/tabby).\n';

/**
 Archive whose closing carries that rendering.
 */
const ARCHIVE_LINKED = `${OPENING}${ARCHIVE_MIDDLE}${CLOSING_LINKED}`;

//endregion The archive this entry starts from

//region Fixtures

/**
 Builds one pair from a translation-side span.

 THE SOURCE SIDE ALWAYS SAYS SOMETHING, deliberately: `spliceSlices` reads the
 original to decide whether writing nothing into a place is a deletion or a
 passage lost, so a silent source would leave that passage-lost refusal
 untestable. Its offsets are the Chinese document's and nothing here reads
 them, since only the translation side is written into.

 @param target - translation-side chunk, carrying the span to write into

 @returns Pair the publisher may write into

 @example
 ```ts
 const pair = pairOver({ target: { sliceIndex: 0, ... }, },);
 ```
 */
function pairOver(
  {
    target,
    sourceText = '她睡在收银台上。',
  }: {
    readonly target: Record<string, unknown>;
    readonly sourceText?: string;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex: target.sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: sourceText,
    },
    target,
  } as unknown as ChunkPair;
}

/**
 Builds the whole document's slices, opening and closing included.

 SLICED WHOLE RATHER THAN AT THE ONE PARAGRAPH UNDER TEST, because the
 assembler refuses any other shape: it reads a slice's index as its position,
 so a list holding only the middle slice is a caller disagreeing with the
 slicer. Prepared documents are sliced whole too, which is what makes this the
 faithful fixture rather than the convenient one.

 @returns Three pairs covering the archive end to end

 @example
 ```ts
 const slices = documentSlices();
 ```
 */
function documentSlices(): readonly ChunkPair[] {
  return [
    pairOver({
      target: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: MIDDLE_START,
        text: OPENING,
      },
    },),
    pairOver({
      target: {
        sliceIndex: 1,
        nodes: [],
        startOffset: MIDDLE_START,
        endOffset: MIDDLE_END,
        text: ARCHIVE_MIDDLE,
      },
    },),
    pairOver({
      target: {
        sliceIndex: 2,
        nodes: [],
        startOffset: MIDDLE_END,
        endOffset: ARCHIVE.length,
        text: CLOSING,
      },
    },),
  ];
}

/**
 Builds the slices of the archive whose closing carries the moved link.

 @returns Three pairs covering that archive end to end

 @example
 ```ts
 const slices = linkedDocumentSlices();
 ```
 */
function linkedDocumentSlices(): readonly ChunkPair[] {
  return [
    ...documentSlices().slice(
      0,
      2,
    ),
    pairOver({
      target: {
        sliceIndex: 2,
        nodes: [],
        startOffset: MIDDLE_END,
        endOffset: ARCHIVE_LINKED.length,
        text: CLOSING_LINKED,
      },
    },),
  ];
}

/**
 Builds a document whose middle section the archive never translated.

 THE SECOND SLICE IS A ZERO-WIDTH PLACE at the boundary rather than a span
 over wording, which is what a preparation produces where a source section has
 no translation to pair with. The archive's own middle paragraph is left
 unsliced, so a rendering written here lands AHEAD of it rather than over it.

 @returns Two pairs, the second of them a place rather than wording

 @example
 ```ts
 const slices = documentSlicesWithAGap();
 ```
 */
function documentSlicesWithAGap(): readonly ChunkPair[] {
  return [
    pairOver({
      target: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: MIDDLE_START,
        text: OPENING,
      },
    },),
    pairOver({
      target: {
        kind: 'insertion',
        sliceIndex: 1,
        nodes: [],
        startOffset: MIDDLE_START,
        endOffset: MIDDLE_START,
        text: '',
      },
    },),
  ];
}

/**
 Builds an artifact whose one slice reads as the wording given.

 GOES THROUGH THE CONTEST rather than the consolidation, because the contest
 is the shortest path to a chosen wording and this file is about what happens
 AFTER a decider spoke, not about which decider spoke. Whether each stage is
 read correctly is `would-ship-text.unit.test.ts`.

 @param translateText - wording the translate lane offered and the contest picked

 @param incumbentKind - whether the archive holds wording at this slice

 @returns Artifact the publisher reads

 @example
 ```ts
 const artifact = artifactShipping({ translateText: DECIDED_MIDDLE, },);
 ```
 */
function artifactShipping(
  {
    translateText,
    incumbentKind = 'present',
  }: {
    readonly translateText: string;
    readonly incumbentKind?: 'present' | 'absent';
  },
): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 1,
        incumbentKind,
        incumbentText: (incumbentKind === 'present') ? ARCHIVE_MIDDLE : '',
        repairText: ARCHIVE_MIDDLE,
        translateText,
        laneRelation: 'both-differ',
        repairOutcome: {
          kind: 'decided',
          acceptedText: ARCHIVE_MIDDLE,
        },
        translateOutcome: {
          kind: 'decided',
          acceptedText: translateText,
        },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'different',
        },
        repairDelivery: { kind: 'replacement-shipped', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
    ],
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [
        {
          sliceIndex: 1,
          verdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          ballots: [],
          usable: 3,
        },
      ],
    },
  } as unknown as WouldShipSource;
}

/**
 Builds an artifact whose consolidation kept the archive at slice 1 because
 no wording there passed the rule (owner, 2026-09-27, "Keep archive, ship"),
 over a contest that chose the translate lane's refused wording.

 @param incumbentKind - whether the archive holds wording at this slice

 @returns Artifact the publisher reads

 @example
 ```ts
 const artifact = artifactKeepingTheArchive({ incumbentKind: 'present', },);
 ```
 */
function artifactKeepingTheArchive(
  { incumbentKind, }: { readonly incumbentKind: 'present' | 'absent'; },
): WouldShipSource {
  return {
    ...artifactShipping({
      translateText: DECIDED_MIDDLE,
      incumbentKind,
    },),
    consolidation: {
      kind: 'settled',
      slices: [
        {
          sliceIndex: 1,
          terminal: 'incumbent-only',
          shipped: { kind: 'archive', },
          rewrapped: false,
          demoted: false,
          verdicts: [],
          gate: { kind: 'not-asked', },
          polish: {
            kind: 'not-run',
            reason: 'unsafe-baseline',
          },
        },
      ],
    },
  } as unknown as WouldShipSource;
}

/**
 Characters the disagreeing fixture claims the archive holds beyond what it
 does, chosen large enough to be unambiguous and small enough to be a
 plausible drift rather than a rewrite.
 */
const OVERSTATED_BY = 5;

/**
 Builds an artifact whose comparison row claims MORE archive wording at the
 slice than the archive actually holds there.

 MODELS A KNOWN FAILURE CLASS RATHER THAN A TYPO: an artifact and the publisher
 disagreeing about what a slice covers is exactly the state that cost XIEPT2
 four hours and forty-eight minutes, and it is invisible to every check that
 reads only one of the two. The wording still ships and still lands in order,
 so the occurrence scan passes and only the arithmetic notices.

 @returns Artifact that disagrees with the archive the publisher splices

 @example
 ```ts
 const artifact = artifactOverstatingTheArchive();
 ```
 */
function artifactOverstatingTheArchive(): WouldShipSource {
  /**
   The honest fixture, whose one row is then overstated.
   */
  const honest = artifactShipping({ translateText: DECIDED_MIDDLE, },) as unknown as {
    readonly comparison: readonly Record<string, unknown>[];
  };

  return {
    ...honest,
    comparison: honest.comparison
      .map(function overstate(row,): Record<string, unknown> {
        return {
          ...row,
          incumbentText: `${ARCHIVE_MIDDLE}${'x'.repeat(OVERSTATED_BY,)}`,
        };
      },),
  } as unknown as WouldShipSource;
}

/**
 Publishes one entry and reads back what landed.

 @param artifact - settled entry to publish

 @param publishDir - tree root to write into

 @param slices - pairs the entry was prepared into

 @returns Path written and the bytes at it

 @example
 ```ts
 const { text, } = await publishAndRead({ artifact, publishDir, },);
 ```
 */
async function publishAndRead(
  {
    artifact,
    publishDir,
    slices = documentSlices(),
    sourceText = SOURCE_PAGE,
  }: {
    readonly artifact: WouldShipSource;
    readonly publishDir: string;
    readonly slices?: readonly ChunkPair[];
    readonly sourceText?: string;
  },
): Promise<{ readonly path: string; readonly text: string; readonly destinations: DestinationCheck; }> {
  /**
   Where the publisher put it, and what the page carries of the source's links.
   */
  const {
    path,
    destinations,
  } = await publishFixedPage({
    artifact,
    slices,
    archiveText: ARCHIVE,
    sourceText,
    entryId: 'BookshopCat',
    publishDir,
    l: tagged({ tag: 'publish-test', },),
  },);

  return {
    path,
    text: await readFile(
      path,
      'utf8',
    ),
    destinations,
  };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: publishFixedPage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'WRITES THE DECIDED WORDING INTO THE SLICE IT NAMES AND NOWHERE ELSE, leaving the paragraphs '
              + 'either side byte for byte as the archive had them. A publisher that rebuilt the document '
              + 'from the pieces it knew about would pass an assertion about the replaced slice alone while '
              + 'quietly dropping everything no decider mentioned',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            const { text, } = await publishAndRead({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              publishDir: tree.path,
            },);

            expect(text,).toBe(`${OPENING}${DECIDED_MIDDLE}${CLOSING}`,);
            expect(text.startsWith(OPENING,),).toBe(true,);
            expect(text.endsWith(CLOSING,),).toBe(true,);
          },
        },),

        it({
          name: 'COUNTS THE PAGE IT WROTE IN CHARACTERS, one for each character beyond the first plane, as the '
            + 'published page check counts the same page, never in the UTF-16 units that count each as two',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);
            /**
             What the publisher logged.
             */
            const messages: string[] = [];

            await publishFixedPage({
              artifact: artifactShipping({ translateText: PAWED_MIDDLE, },),
              slices: documentSlices(),
              archiveText: ARCHIVE,
              sourceText: SOURCE_PAGE,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: capturingLogger({ messages, },),
            },);

            expect(messages,).toEqual(['publish: wrote 1 slice into a page of 180 characters',],);
          },
        },),

        it({
          name:
            'REPUBLISHES AN ENTRY NO DECIDER CHANGED BYTE FOR BYTE, trailing newline included. The fixed '
              + 'version of an unchanged page IS the page, which is what makes the tree a drop-in set rather '
              + 'than a patch; it is also the only assertion that catches an assembler that rewrote line '
              + 'endings or trimmed an ending nobody asked it to touch',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            const { text, } = await publishAndRead({
              artifact: artifactShipping({ translateText: ARCHIVE_MIDDLE, },),
              publishDir: tree.path,
            },);

            expect(text,).toBe(ARCHIVE,);
            expect(text.endsWith('\n',),).toBe(true,);
            expect(text.endsWith('\n\n',),).toBe(false,);
          },
        },),

        it({
          name:
            'CREATES EVERY DIRECTORY ON THE WAY DOWN, since the tree root is made once per pass and the '
              + 'two levels under it are per entry. Without this the first entry of every run fails on a '
              + 'missing parent, which is a failure that looks like a broken document rather than a missing '
              + 'directory',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            const { path, } = await publishAndRead({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              publishDir: tree.path,
            },);

            expect(path,).toBe(join(
              tree.path,
              'people',
              'BookshopCat',
              'page.en.md',
            ),);
          },
        },),

        it({
          name:
            'OVERWRITES A PAGE ALREADY SITTING THERE, which is what a resumed pass does to every entry it '
              + 'settles again. Refusing would make the second run of any interrupted pass fail on its own '
              + 'output, and skipping would leave a page from an older pipeline claiming to be this run\'s',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            const path = fixedPagePath({
              publishDir: tree.path,
              entryId: 'BookshopCat',
            },);
            await rm(
              path,
              { force: true, },
            );

            const first = await publishAndRead({
              artifact: artifactShipping({ translateText: 'stale wording from an earlier pipeline\n', },),
              publishDir: tree.path,
            },);
            expect(first.text.includes('stale wording',),).toBe(true,);

            const second = await publishAndRead({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              publishDir: tree.path,
            },);
            expect(second.text,).toBe(`${OPENING}${DECIDED_MIDDLE}${CLOSING}`,);
            expect(second.text.includes('stale wording',),).toBe(false,);
          },
        },),

        it({
          name:
            'FILLS A PASSAGE THE ARCHIVE NEVER TRANSLATED, writing into the zero-width place the '
              + 'preparation left for it. This is the case that cannot be assembled from the artifact alone: '
              + 'an absent incumbent is the empty string, which matches at every offset, so only the stored '
              + 'span can say where the rendering goes',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            const { text, } = await publishAndRead({
              artifact: artifactShipping({
                translateText: DECIDED_MIDDLE,
                incumbentKind: 'absent',
              },),
              publishDir: tree.path,
              slices: documentSlicesWithAGap(),
            },);

            // Written INTO the gap rather than over the paragraph after it: the
            // archive's own middle paragraph is still there, with the filled
            // passage ahead of it.
            expect(text,).toBe(`${OPENING}${DECIDED_MIDDLE}${ARCHIVE_MIDDLE}${CLOSING}`,);
          },
        },),

        it({
          name: 'PRESERVES accepted source-only hard breaks through wrapping, insertion assembly and disk publication',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);
            /** Cat verse stands in for a source-only closing poem. */
            const sourceText = '> 猫醒了。  \n> 鸟唱了。';
            /** Run the real wrapper before the structural and publication boundaries. */
            const translated = wrapReplacementText({ text: '> The cat wakes.  \n> The bird sings.', },);
            expect(validateTranslatedSlice({ sourceText, candidateText: translated, },).kind,).toBe('valid',);
            /** The artifact and assembler must agree on every render-bearing byte. */
            const { text, } = await publishAndRead({
              artifact: artifactShipping({ translateText: translated, incumbentKind: 'absent', },),
              publishDir: tree.path,
              slices: documentSlicesWithAGap(),
              sourceText,
            },);
            expect(text,).toContain(translated,);
            expect(text,).toContain('wakes.  \n> The bird sings.',);
          },
        },),
        it({
          name:
            'PUBLISHES AN ENTRY WHOSE DECIDERS LEFT AN ANCHOR UNFILLED AND WHOSE ARTIFACT RECORDS NO ORIGINAL '
              + 'FOR IT, leaving the gap exactly as the archive had it (an artifact that records the original '
              + 'ships it under a note: `source-shipped-page.unit.test.ts`). It used to refuse, on the argument that '
              + 'a half-document is worse than a failed entry because only the failure gets retried. `XIEPT2` '
              + 'settled that: the refusal '
              + 'landed after four hours and forty-eight minutes of calls, kept neither page nor artifact, '
              + 'and a retry meets the same passage and the same judges. The archive already carries this '
              + 'gap, so the page loses nothing that was ever there and keeps every slice the run did buy',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             Page the publisher wrote over an archive with an unfilled anchor in it.
             */
            const published = await publishAndRead({
              artifact: artifactWithAnUnfilledAnchor(),
              publishDir: tree.path,
              slices: documentSlicesWithAGap(),
            },);

            expect(published.text,).toBe(ARCHIVE,);
          },
        },),

        it({
          name:
            'SHIPS THE ARCHIVE AT A SLICE WHERE NO WORDING PASSED THE RULE, over the lane the contest '
              + 'chose, and reports it (owner, 2026-09-27, "Keep archive, ship"). Over present wording the '
              + 'page reads as the archive there; over an anchor the archive left silent, nothing is written, '
              + 'since an empty string at an anchor the original speaks is a passage lost to the splice',
          fn: async () => {
            /**
             Both archive states, each published into its own tree.
             */
            const outcomes = await Promise.all(([
              {
                incumbentKind: 'present',
                slices: documentSlices(),
              },
              {
                incumbentKind: 'absent',
                slices: documentSlicesWithAGap(),
              },
            ] as const).map(async function publishKeepingTheArchive(
              {
                incumbentKind,
                slices,
              }: {
                readonly incumbentKind: 'present' | 'absent';
                readonly slices: readonly ChunkPair[];
              },
            ): Promise<{
              readonly text: string;
              readonly checks: readonly string[];
              readonly message: string;
            }> {
              await using tree = await scratchDir({ prefix: 'publish-fixed-', },);
              /**
               The publication, which ships the archive at slice 1 and reports it.
               */
              const published = await publishFixedPage({
                artifact: artifactKeepingTheArchive({ incumbentKind, },),
                slices,
                archiveText: ARCHIVE,
                sourceText: SOURCE_PAGE,
                entryId: 'BookshopCat',
                publishDir: tree.path,
                l: tagged({ tag: 'publish-test', },),
              },);
              return {
                text: await readFile(
                  published.path,
                  'utf8',
                ),
                checks: published.defects.map(function checkOf({ check, },): string {
                  return check;
                },),
                message: published.defects[0]?.message ?? '',
              };
            },),);
            for (const outcome of outcomes) {
              expect(outcome.text,).toBe(ARCHIVE,);
              expect(outcome.checks,).toStrictEqual(['no-valid-wording',],);
              expect(outcome.message,).toContain('slice 1',);
            }
            expect(outcomes.length,).toBe(2,);
          },
        },),

        it({
          name:
            'STILL LEAVES THE SPLICE FREE TO REFUSE A BLANK RENDERING AT AN ANCHOR, which is the guard '
              + 'that caught this and is not being weakened. What changed is upstream of it: a slice '
              + 'deliberately left unfilled no longer presents itself as a rendering of blank text, so the '
              + 'guard now only ever sees a caller that really did claim one',
          fn: async () => {
            /**
             What the splice refused when handed the row this publisher no longer
             emits, held so the class and the wording can both be asserted.
             */
            const refusalOfWritingNothingAtAnAnchor = async (): Promise<string> => spliceSlices({
              targetText: ARCHIVE,
              slices: documentSlicesWithAGap(),
              replacements: [
                {
                  sliceIndex: 1,
                  replacementText: '',
                },
              ],
            },);

            await expect(refusalOfWritingNothingAtAnAnchor(),).rejects.toBeInstanceOf(SliceSpliceError,);
            await expect(refusalOfWritingNothingAtAnAnchor(),).rejects.toThrow(
              'has no translation and writes none',
            );
          },
        },),

        it({
          name: 'SHIPS A PAGE RENAMING A CONTRIBUTOR and reports the defect, since a settled page ships with its '
            + 'defects reported (the owner, 2026-09-27)',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);
            /**
             Existing English attribution establishing chosen public handle.
             */
            const contributorArchive = 'Contributors for this entry: [Snow](https://example.test/snow)\n';
            /**
             Candidate literally respelling contributor while retaining destination.
             */
            const renamed = 'Contributors for this entry: [Snowflake](https://example.test/snow)\n';
            /**
             Baseline artifact shape reused with single whole-page attribution slice.
             */
            const baseline = artifactShipping({ translateText: renamed, });
            /**
             Comparison row adapted to attribution fixture.
             */
            const [baselineRow,] = baseline.comparison;
            if (baselineRow === undefined)
              throw new Error('publisher fixture has no comparison row',);
            const artifact = {
              ...baseline,
              comparison: [{
                ...baselineRow,
                sliceIndex: 0,
                incumbentText: contributorArchive,
                repairText: contributorArchive,
                translateText: renamed,
                repairOutcome: {
                  kind: 'decided',
                  acceptedText: contributorArchive,
                },
                translateOutcome: {
                  kind: 'decided',
                  acceptedText: renamed,
                },
              },],
              laneSelection: {
                kind: 'contested',
                slices: [{
                  sliceIndex: 0,
                  verdict: {
                    kind: 'lane-won',
                    lane: 'translate',
                  },
                  ballots: [],
                  usable: 0,
                },],
              },
            } as WouldShipSource;
            /**
             The publication, which ships the page and reports the renaming.
             */
            const published = await publishFixedPage({
              artifact,
              slices: [pairOver({
                target: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: contributorArchive.length,
                  text: contributorArchive,
                },
              },),],
              archiveText: contributorArchive,
              sourceText: '条目贡献：小雪\n',
              entryId: 'BookshopContributors',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
            },);
            expect(published.defects.map(function checkOf({ check, },): string {
              return check;
            },),).toStrictEqual(['contributor-names',],);
            expect(await readFile(
              published.path,
              'utf8',
            ),).toBe(renamed,);
          },
        },),

        it({
          name:
            'REFUSES A READING NAMING A SLICE THE PREPARATION NEVER PRODUCED, instead of writing the page '
              + 'without it. The two sides are stamped by different stages, and it is known what '
              + 'index disagreement costs: silently dropping the reading would publish archive wording at a '
              + 'slice the deciders replaced',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             What publishAndRead refused with, held so the class and the wording
             can both be asserted: `.rejects` awaits afresh on every matcher call,
             so one promise serves two assertions and no capture helper is needed.
             */
            const refusalOfPublishingPastTheSlices = publishAndRead({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              publishDir: tree.path,
              slices: [documentSlices()[0] as ChunkPair,],
            },);

            await expect(refusalOfPublishingPastTheSlices,).rejects.toBeInstanceOf(SliceSpliceError,);
            await expect(refusalOfPublishingPastTheSlices,).rejects.toThrow('no slice 1 to write into',);
          },
        },),
      ],
    },),

    describe({
      name: `${publishFixedPage.name} ships a page that disagrees with its artifact, with the defect reported`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'WRITES THE PAGE AND NAMES THE DISAGREEMENT when the artifact and the archive disagree about '
              + 'what a slice covers (owner, 2026-09-27, "Ship with defect reported"). It used to refuse the '
              + 'entry, so the archive\'s page shipped in its place; the check is one-sided, so a failure is a '
              + 'defect in assembly, and it now ships on the `DEFECTS` line like the content checks',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             Where this case publishes.
             */
            const { path: publishDir, } = tree;

            /**
             What the publisher returned, or raised.
             */
            const outcome = await (async (): Promise<unknown> => {
              try {
                return await publishFixedPage({
                  artifact: artifactOverstatingTheArchive(),
                  slices: documentSlices(),
                  archiveText: ARCHIVE,
                  sourceText: SOURCE_PAGE,
                  entryId: 'BookshopCat',
                  publishDir,
                  l: tagged({ tag: 'publish-test', },),
                },);
              } catch (error) {
                return error;
              }
            })();

            expect(outcome,).not.toBeInstanceOf(PublishedPageDisagreesError,);
            /**
             The page-agreement defects the publisher reported.
             */
            const reported = ((outcome as { readonly defects?: readonly PublishDefect[]; }).defects ?? [])
              .filter(function isAgreement({ check, },): boolean {
                return check === 'page-agreement';
              },);
            expect(reported,).toHaveLength(1,);
            expect(reported[0]?.message,).toContain('characters off',);

            /**
             Where the page landed.
             */
            const writtenAt = fixedPagePath({
              publishDir,
              entryId: 'BookshopCat',
            },);

            expect(existsSync(writtenAt,),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${publishFixedPage.name} destinations`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHIPS A PAGE DROPPING A SOURCE DESTINATION and reports the defect, since a settled page ships '
            + 'with its defects reported (the owner, 2026-09-27; ledger E1)',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             The publication, which ships the page and reports the loss.
             */
            const published = await publishFixedPage({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              slices: documentSlices(),
              archiveText: ARCHIVE,
              sourceText: `${SOURCE_PAGE}\n她的主页：https://example.org/tabby。\n`,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
            },);

            expect(published.defects.map(function checkOf({ check, },): string {
              return check;
            },),).toStrictEqual(['destinations',],);
            expect(published.destinations.dropped.length,).toBe(1,);
            expect(existsSync(published.path,),).toBe(true,);
          },
        },),

        it({
          name: 'NAMES THE SLICE whose original carries a destination the page drops, so the reported defect can '
            + 'be traced (ledger E1: XingZ6011 lost 96 minutes to a refusal that named neither)',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);
            /**
             The original's middle, linking her page.
             */
            const linkedSource = '她睡在收银台上。她的主页：https://example.org/tabby。';

            /**
             The publication, which ships the page and reports where the loss sits.
             */
            const published = await publishFixedPage({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              slices: documentSlices().map(function linkMiddle(pair,): ChunkPair {
                return (pair.target.sliceIndex === 1)
                  ? pairOver({
                    target: { ...pair.target, },
                    sourceText: linkedSource,
                  },)
                  : pair;
              },),
              archiveText: ARCHIVE,
              sourceText: `${SOURCE_PAGE}\n${linkedSource}\n`,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
            },);

            expect(published.defects.length,).toBe(1,);
            expect(published.defects[0]?.message.includes('slice 1',),).toBe(true,);
          },
        },),

        it({
          name: 'PUBLISHES a page keeping the archive rendering of a source destination, and names it',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             Page whose closing keeps the archive's address for the source's home link.
             */
            const published = await publishFixedPage({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              slices: linkedDocumentSlices(),
              archiveText: ARCHIVE_LINKED,
              sourceText: `${SOURCE_PAGE}\n她的主页：https://example.org/tabby。\n`,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
            },);

            expect(existsSync(published.path,),).toBe(true,);
            expect(published.destinations.dropped,).toStrictEqual([],);
            expect(published.destinations.page,).toStrictEqual(['https://example.net/tabby',],);
            expect(published.destinations.findings,).toStrictEqual(['destinations-archive-rendering',],);
          },
        },),

        it({
          name: 'reports nothing dropped for a source that links nowhere, which is the control',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             Page published from the linkless source page.
             */
            const published = await publishAndRead({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              publishDir: tree.path,
            },);

            expect(published.destinations,).toStrictEqual({
              source: [],
              page: [],
              dropped: [],
              findings: [],
            },);
          },
        },),
        it({
          name: 'REFUSES a page whose decided wording curls a JSX string literal, so the grammar floor '
            + 'sits between the would-ship reading and the disk (2026-09-06)',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             What the publisher threw for the wording no grammar reads.
             */
            const thrown = await (async (): Promise<unknown> => {
              try {
                await publishFixedPage({
                  artifact: artifactShipping({
                    translateText: `\n> <PhotoScroll photos={[\u{201C}\${path}/photos/photo3.webp\u{201D}]} />\n`,
                  },),
                  slices: documentSlices(),
                  archiveText: ARCHIVE,
                  sourceText: SOURCE_PAGE,
                  entryId: 'BookshopCat',
                  publishDir: tree.path,
                  l: tagged({ tag: 'publish-test', },),
                },);
                return undefined;
              } catch (error) {
                return error;
              }
            })();

            /**
             Where the page would have landed.
             */
            const path = fixedPagePath({
              publishDir: tree.path,
              entryId: 'BookshopCat',
            },);

            expect(thrown,).toBeInstanceOf(UnparseablePageError,);
            expect((thrown as Error).message,).not.toContain('photo3',);
            expect(existsSync(path,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: `${publishFixedPage.name} sealed spans`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHIPS A PAGE WRITING OVER A SPAN THE ARCHIVE\'S NOTE SEALS and reports the archive-original defect, '
            + 'since a sealed span ships as the archive has it (the owner\'s rule of 2026-09-08); the same page '
            + 'published with no span sealed reports nothing, the control. The pass hands the spans through '
            + 'persistSettledEntry, and this is the one place they speak',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             Checks the page failed with the middle paragraph sealed.
             */
            const sealed = await publishFixedPage({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              slices: documentSlices(),
              archiveText: ARCHIVE,
              sourceText: SOURCE_PAGE,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
              archiveOriginalSpans: [{
                startOffset: MIDDLE_START,
                endOffset: MIDDLE_END,
                note: 'The cat wrote this part in English.',
              },],
            },);
            /**
             Checks the same page failed with nothing sealed.
             */
            const unsealed = await publishFixedPage({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              slices: documentSlices(),
              archiveText: ARCHIVE,
              sourceText: SOURCE_PAGE,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
            },);

            expect([
              sealed,
              unsealed,
            ].map(function checksOf({ defects, },): readonly string[] {
              return defects.map(function checkOf({ check, },): string {
                return check;
              },);
            },),).toStrictEqual([
              ['archive-original',],
              [],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${publishFixedPage.name} a page with no body`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS A PAGE THAT SHIPS NO BODY WHILE THE ORIGINAL HAS ONE, whether the page is empty or only front '
            + 'matter, since a 0-byte page shipped with no defect and no warning for an archive that held no body '
            + '(commitment of the shipping design: a passage with no wording ships its original under a note). It '
            + 'ships, reported like the other content checks, and the same publisher reports nothing for a page '
            + 'with a body or for an original with none',
          fn: async () => {
            await using tree = await scratchDir({ prefix: 'publish-fixed-', },);

            /**
             Original with front matter and a body.
             */
            const original = '---\nname: 猫\n---\n\n## 简介\n\n一只虎斑猫。\n';
            /**
             Original that holds front matter and nothing else.
             */
            const originalWithoutBody = '---\nname: 猫\n---\n';
            /**
             Names of the defects reported for one page cut into no slices.

             @param archiveText - page the slices index, which nothing replaces

             @param sourceText - original the page is weighed against

             @returns Names of the defects reported

             @example
             ```ts
             const checks = await checksOfBodilessPage({ archiveText: '', sourceText: original, },);
             ```
             */
            async function checksOfBodilessPage(
              {
                archiveText,
                sourceText,
              }: {
                readonly archiveText: string;
                readonly sourceText: string;
              },
            ): Promise<readonly string[]> {
              /**
               What the publisher returned.
               */
              const published = await publishFixedPage({
                artifact: sourceShippedArtifact({ rows: [], },),
                slices: [],
                archiveText,
                sourceText,
                entryId: 'BookshopCat',
                publishDir: tree.path,
                l: tagged({ tag: 'publish-test', },),
              },);
              return published.defects.map(function checkOf({ check, },): string {
                return check;
              },);
            }

            expect(await checksOfBodilessPage({ archiveText: '', sourceText: original, },),).toContain(
              'page-without-body',
            );
            expect(await checksOfBodilessPage({
              archiveText: '---\nname: Cat\n---\n',
              sourceText: original,
            },),).toContain('page-without-body',);
            // CONTROLS: an original with no body reports nothing of the kind, and nor does a page with a body.
            expect(await checksOfBodilessPage({ archiveText: '', sourceText: originalWithoutBody, },),).not
              .toContain('page-without-body',);
            /**
             Page with a body, published through the full entry.
             */
            const withBody = await publishFixedPage({
              artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
              slices: documentSlices(),
              archiveText: ARCHIVE,
              sourceText: SOURCE_PAGE,
              entryId: 'BookshopCat',
              publishDir: tree.path,
              l: tagged({ tag: 'publish-test', },),
            },);
            expect(withBody.defects.map(function checkOf({ check, },): string {
              return check;
            },),).not
              .toContain('page-without-body',);
          },
        },),
      ],
    },),
  ],
},);
