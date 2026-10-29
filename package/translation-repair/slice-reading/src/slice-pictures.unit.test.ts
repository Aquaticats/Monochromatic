/**
 Tests for which pictures one slice is shown, and what a stage is told about
 a picture nobody could corroborate.

 WHAT THESE PIN is two claims bolted together. First, that
 `slicePictureNames` takes one section each way exactly the way
 `fidelity-window.ts`'s `neighbouringSource` does, including its RangeError
 contract: an index that is not a position in `slices` must never silently
 name no pictures, because that reads as a slice that shows none rather than
 the mistake it is. Second, that `slicePictures` keeps a picture nobody could
 corroborate out of the rendered prompt entirely and names it in `findings`
 instead, since a stage handed a hedge it cannot weigh is worse off than one
 simply not told the picture existed.

 BOTH READINGS TRAVEL for a picture that was corroborated, labelled by
 model, which one case here asserts on an exact rendered block: the shorter
 reading vouches for what it carries and the longer carries more, so a stage
 shown only one of them loses either the vouching or the content.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChunkPair,
  type PairedReading,
  SIDE_LEGEND,
  slicePictureContexts,
  slicePictureNames,
  slicePictures,
} from '../dist/final/node/index.mjs';
import { sliceOf, } from './content-slice-of.test-fixture.ts';

/**
 Heading `slicePictures` renders before each picture's transcriptions.

 MIRRORS THE MODULE'S OWN CONSTANT, kept private in `slice-pictures.ts`: this
 file states same literal directly rather than importing something not
 exported.
 */
const PICTURE_HEADING = 'PICTURE';

/**
 Placeholder corpus pages write for an entry's own directory.

 AN ESCAPED TEMPLATE LITERAL, so characters landing in a slice's text are
 what corpus text carries rather than an interpolation this file performs
 by accident. Mirrors `photo-reference.unit.test.ts`.
 */
const ENTRY = `\${path}`;

/**
 Cat-themed stand-in for one vision reader's model id.

 A CAST THROUGH `unknown`, since `RosterModelId` is a closed union of
 production identifiers and neither function under test validates one
 against real roster: `slicePictures` only folds a reading's model id into a
 rendered label.
 */
const WHISKERS = 'hf:cat/Whiskers' as unknown as RosterModelId;

/**
 Cat-themed stand-in for other vision reader's model id.
 */
const MARMALADE = 'hf:cat/Marmalade' as unknown as RosterModelId;

/**
 Builds one photo element naming given assets, in corpus's only form.

 @param assetNames - file names within entry's photos directory

 @returns Element as a page writes it

 @example
 ```ts
 const element = photoElement({ assetNames: ['sunbeam.webp',], },);
 ```
 */
function photoElement({ assetNames, }: { readonly assetNames: readonly string[]; },): string {
  return `<PhotoScroll photos={[ ${
    assetNames.map(function quoted(assetName,): string {
      return `'${ENTRY}/photos/${assetName}'`;
    },)
      .join(', ',)
  } ]} />`;
}

/**
 Three-slice document where every slice names a DIFFERENT picture, so
 document order is visible in a returned name list rather than hidden
 behind a repeat.
 */
const ORDERED_SLICES: readonly ChunkPair[] = [
  `Tabby suns herself on the windowsill.\n\n${photoElement({ assetNames: ['sunbeam.webp',], },)}\n`,
  `She stretches and yawns.\n\n${photoElement({ assetNames: ['stretch.webp',], },)}\n`,
  `Then she naps by the door.\n\n${photoElement({ assetNames: ['nap.webp',], },)}\n`,
].map(function toSlice(
  text,
  sliceIndex,
) {
  return sliceOf({
    text,
    sliceIndex,
  },);
},);

/**
 Three-slice document where the slices EITHER SIDE of the middle one name
 the SAME picture and the middle slice names none itself, so a returned
 name list can be checked for exactly one entry rather than two.
 */
const DUPING_SLICES: readonly ChunkPair[] = [
  `Tabby watches from the sill.\n\n${photoElement({ assetNames: ['perch.webp',], },)}\n`,
  'Then she watches birds outside.\n',
  `By evening she returns to the sill.\n\n${photoElement({ assetNames: ['perch.webp',], },)}\n`,
].map(function toSlice(
  text,
  sliceIndex,
) {
  return sliceOf({
    text,
    sliceIndex,
  },);
},);

/**
 Three-slice document whose middle slice names no picture itself but sits
 between two that do, so `slicePictures` on the middle slice renders both
 neighbours' pictures into one context block.
 */
const MULTI_PICTURE_SLICES: readonly ChunkPair[] = [
  `Tabby dozes in a sunbeam.\n\n${photoElement({ assetNames: ['sunbeam.webp',], },)}\n`,
  'She wakes and stretches.\n',
  `Then she settles for a nap.\n\n${photoElement({ assetNames: ['nap.webp',], },)}\n`,
].map(function toSlice(
  text,
  sliceIndex,
) {
  return sliceOf({
    text,
    sliceIndex,
  },);
},);

/**
 What shorter reader transcribed from sunbeam picture, vouching for what it
 carries without contradicting longer reading.
 */
const SUNBEAM_SHORT = 'A tabby cat dozes in a sunbeam.';

/**
 What longer reader transcribed from same picture, carrying more than
 shorter reading without contradicting it.
 */
const SUNBEAM_LONG = 'A tabby cat lies curled and dozing in a warm patch of sun by the window.';

/**
 What shorter reader transcribed from nap picture.
 */
const NAP_SHORT = 'The cat naps beside the door.';

/**
 What longer reader transcribed from same picture, carrying more than
 shorter reading without contradicting it.
 */
const NAP_LONG = 'The tabby cat has curled up for a nap on the mat right beside the door.';

/**
 Readings this document's slices resolve against: both neighbours of the
 middle slice corroborated, so its rendered context carries both blocks.
 */
const MULTI_PICTURE_READINGS: ReadonlyMap<string, PairedReading> = new Map<string, PairedReading>([
  [
    'sunbeam.webp',
    {
      kind: 'corroborated',
      readings: [
        {
          modelId: WHISKERS,
          text: SUNBEAM_SHORT,
        },
        {
          modelId: MARMALADE,
          text: SUNBEAM_LONG,
        },
      ],
      overlap: 0.82,
    },
  ],
  [
    'nap.webp',
    {
      kind: 'corroborated',
      readings: [
        {
          modelId: WHISKERS,
          text: NAP_SHORT,
        },
        {
          modelId: MARMALADE,
          text: NAP_LONG,
        },
      ],
      overlap: 0.77,
    },
  ],
],);

/**
 Reading naming one picture unavailable, carrying reason a finding must
 surface.
 */
const UNAVAILABLE_READINGS: ReadonlyMap<string, PairedReading> = new Map<string, PairedReading>([
  [
    'startled.webp',
    {
      kind: 'unavailable',
      reason: 'readers-disagree',
      perReader: [],
      transient: false,
      overlap: 0.04,
    },
  ],
],);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: slicePictureNames.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE PICTURES A SLICE AND BOTH NEIGHBOURS SHOW, IN DOCUMENT ORDER, since a '
            + 'slice shown only its own reference would be shown nothing about a picture a '
            + 'neighbouring slice already carries',
          fn: async () => {
            expect(slicePictureNames({
              slices: ORDERED_SLICES,
              slicePosition: 1,
            },),).toEqual([
              'sunbeam.webp',
              'stretch.webp',
              'nap.webp',
            ],);
          },
        },),

        it({
          name: 'GIVES THE FIRST SLICE ONLY ITSELF AND ITS FOLLOWER, since asking for index minus '
            + 'one at the start of a document must not read the end of the array',
          fn: async () => {
            expect(slicePictureNames({
              slices: ORDERED_SLICES,
              slicePosition: 0,
            },),).toEqual([
              'sunbeam.webp',
              'stretch.webp',
            ],);
          },
        },),

        it({
          name: 'GIVES THE LAST SLICE ONLY ITSELF AND ITS PREDECESSOR, reaching one section back '
            + 'rather than to the start of the document',
          fn: async () => {
            expect(slicePictureNames({
              slices: ORDERED_SLICES,
              slicePosition: 2,
            },),).toEqual([
              'stretch.webp',
              'nap.webp',
            ],);
          },
        },),

        it({
          name: 'NAMES A PICTURE ONCE EVEN WHEN BOTH NEIGHBOURS SHOW IT, so a stage is not handed '
            + 'the same picture twice for one slice that sits between two references to it',
          fn: async () => {
            expect(slicePictureNames({
              slices: DUPING_SLICES,
              slicePosition: 1,
            },),).toEqual(['perch.webp',],);
          },
        },),

        it({
          name: 'THROWS ON A NON-INTEGER INDEX rather than truncating it, since a fractional '
            + 'position is not a stamped mistake this can silently correct, it is a caller error',
          fn: async () => {
            expect(function askFractional() {
              return slicePictureNames({
                slices: ORDERED_SLICES,
                slicePosition: 1.5,
              },);
            },).toThrow(RangeError,);
          },
        },),

        it({
          name: 'THROWS ON A NEGATIVE INDEX rather than reading the end of the array, since a '
            + 'caller that already subtracted one would otherwise be handed the LAST slice as a '
            + 'neighbour of the first',
          fn: async () => {
            expect(function askBeforeStart() {
              return slicePictureNames({
                slices: ORDERED_SLICES,
                slicePosition: -1,
              },);
            },).toThrow(RangeError,);
          },
        },),

        it({
          name: 'THROWS ON AN INDEX PAST THE END rather than naming no pictures, because an index '
            + 'stamped elsewhere would silently name no pictures here, which reads as a slice that '
            + 'shows none rather than the mistake it is',
          fn: async () => {
            expect(function askPastEnd() {
              return slicePictureNames({
                slices: ORDERED_SLICES,
                slicePosition: ORDERED_SLICES.length,
              },);
            },).toThrow(RangeError,);
          },
        },),
      ],
    },),

    describe({
      name: slicePictures.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RENDERS EVERY CORROBORATED PICTURE INTO CONTEXT WITH BOTH READINGS LABELLED BY '
            + 'MODEL, not only the longer one, since agreement establishes the two describe the '
            + 'same picture rather than the same amount of it, and reports no finding for a picture '
            + 'that rendered',
          fn: async () => {
            /**
             What middle slice and both its neighbours resolve to.
             */
            const rendered = slicePictures({
              slices: MULTI_PICTURE_SLICES,
              slicePosition: 1,
              readings: MULTI_PICTURE_READINGS,
            },);

            /**
             Exact rendered context: two blocks in document order, each
             carrying both readings under their own model id.
             */
            const expectedContext = `${PICTURE_HEADING} sunbeam.webp\n${WHISKERS}:\n${SUNBEAM_SHORT}`
              + `\n\n${MARMALADE}:\n${SUNBEAM_LONG}\n\n${PICTURE_HEADING} nap.webp\n${WHISKERS}:\n`
              + `${NAP_SHORT}\n\n${MARMALADE}:\n${NAP_LONG}`;

            expect(rendered.context,).toBe(expectedContext,);
            expect(rendered.findings,).toEqual([],);
          },
        },),

        it({
          name: 'LEADS THE CONTEXT WITH THE SIDE LEGEND WHEN A READING MARKS CHAT SIDES, and with '
            + 'nothing when none does (class thirty-three, 2026-09-16): the reader that saw the '
            + 'picture marks which side each bubble sat on, and a judge shown bare lines guesses who '
            + 'spoke, which is how a human translation with the speakers right was revised wrong',
          fn: async () => {
            /**
             One chat picture read with side markers by both readers.
             */
            const chatReadings: ReadonlyMap<string, PairedReading> = new Map<string, PairedReading>([
              [
                'startled.webp',
                {
                  kind: 'corroborated',
                  readings: [
                    {
                      modelId: WHISKERS,
                      text: '[left] are you there\n[right] here\n[left] [sticker]',
                    },
                    {
                      modelId: MARMALADE,
                      text: '[left] are you there\n[right] here',
                    },
                  ],
                  overlap: 0.9,
                },
              ],
            ],);

            /**
             What a slice naming the chat picture is shown.
             */
            const rendered = slicePictures({
              slices: [sliceOf({
                text: `A shadow startles her off the sill.\n\n`
                  + `${photoElement({ assetNames: ['startled.webp',], },)}\n`,
                sliceIndex: 0,
              },),],
              slicePosition: 0,
              readings: chatReadings,
            },);

            expect(rendered.context.startsWith(`${SIDE_LEGEND}\n\n${PICTURE_HEADING} startled.webp\n`,),)
              .toBe(true,);
            expect(rendered.findings,).toEqual([],);

            /**
             Same slice shown readings with no side markers.
             */
            const plain = slicePictures({
              slices: MULTI_PICTURE_SLICES,
              slicePosition: 1,
              readings: MULTI_PICTURE_READINGS,
            },);
            expect(plain.context.includes('[right]',),).toBe(false,);
            expect(plain.context.startsWith(PICTURE_HEADING,),).toBe(true,);
          },
        },),

        it({
          name: 'NAMES AN UNAVAILABLE PICTURE IN FINDINGS WITH ITS REASON, and renders nothing '
            + 'about it in context, since a stage handed a hedge it cannot weigh is worse off than '
            + 'one simply not told the picture existed',
          fn: async () => {
            /**
             What one slice naming an unavailable picture resolves to.
             */
            const rendered = slicePictures({
              slices: [sliceOf({
                text: `A shadow startles her off the sill.\n\n`
                  + `${photoElement({ assetNames: ['startled.webp',], },)}\n`,
                sliceIndex: 0,
              },),],
              slicePosition: 0,
              readings: UNAVAILABLE_READINGS,
            },);

            expect(rendered.context,).toBe('',);
            expect(rendered.findings,).toEqual(['picture startled.webp: no reading, readers-disagree',],);
          },
        },),

        it({
          name: 'NAMES A PICTURE NEVER READ AS `not read`, WORDED APART FROM AN UNAVAILABLE '
            + 'READING, so a person reading findings can tell a picture whose readings were never '
            + 'gathered from one nobody could corroborate',
          fn: async () => {
            /**
             What one slice naming a picture absent from the readings map
             entirely resolves to.
             */
            const rendered = slicePictures({
              slices: [sliceOf({
                text: `She hides behind a plant pot.\n\n`
                  + `${photoElement({ assetNames: ['shadow.webp',], },)}\n`,
                sliceIndex: 0,
              },),],
              slicePosition: 0,
              readings: new Map<string, PairedReading>(),
            },);

            expect(rendered.context,).toBe('',);
            expect(rendered.findings,).toEqual(['picture shadow.webp: not read',],);
          },
        },),

        it({
          name: 'NAMES A PICTURE WITH NO TEXT AS `carries no text`, a fact about the picture rather than a '
            + 'shortfall, and renders nothing about it in context, since most of this corpus\'s pictures are '
            + 'photographs and a finding worded as missing evidence would send a reader looking for it',
          fn: async () => {
            /**
             What one slice naming a textless picture resolves to.
             */
            const rendered = slicePictures({
              slices: [sliceOf({
                text: `She dozes on the warm stones.\n\n`
                  + `${photoElement({ assetNames: ['stones.webp',], },)}\n`,
                sliceIndex: 0,
              },),],
              slicePosition: 0,
              readings: new Map<string, PairedReading>([
                [
                  'stones.webp',
                  {
                    kind: 'no-text',
                    characters: 0,
                  },
                ],
              ],),
            },);

            expect(rendered.context,).toBe('',);
            expect(rendered.findings,).toEqual(['picture stones.webp: carries no text',],);
          },
        },),

        it({
          name: 'RENDERS AN EMPTY CONTEXT AND NO FINDINGS FOR A SLICE THAT SHOWS NO PICTURES, '
            + 'which is most slices in the corpus',
          fn: async () => {
            /**
             What a single quiet slice, naming no picture itself and standing
             alone with no neighbours, resolves to.
             */
            const rendered = slicePictures({
              slices: [sliceOf({
                text: 'Tabby sleeps through the whole afternoon.\n',
                sliceIndex: 0,
              },),],
              slicePosition: 0,
              readings: new Map<string, PairedReading>(),
            },);

            expect(rendered.context,).toBe('',);
            expect(rendered.findings,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: slicePictureContexts.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEYS EVERY SLICE BY THE STAMP ITS CONSUMERS READ, IN DOCUMENT ORDER, which is the '
            + 'whole job. A stage downstream of preparation holds rows stamped with an index and no '
            + 'array to count positions in, so somebody holding the slices has to do the translation, '
            + 'and assuming it has a recorded cost. The order is asserted rather than '
            + 'sorted away, since the fold walks the document and a map that came out unordered would '
            + 'mean it had stopped doing that',
          fn: async () => {
            /**
             Fold over the three-slice document whose middle slice names no
             picture of its own.
             */
            const contexts = slicePictureContexts({
              slices: MULTI_PICTURE_SLICES,
              readings: MULTI_PICTURE_READINGS,
            },);

            expect([...contexts.keys(),],).toEqual([
              0,
              1,
              2,
            ],);
          },
        },),

        it({
          name: 'RENDERS EACH SLICE EXACTLY WHAT THE POSITIONAL CALL RENDERS IT, so the fold is a '
            + 're-keying and never a second windowing. A producer improving a translation is shown the '
            + 'block its translator was shown, and this is what says so',
          fn: async () => {
            const contexts = slicePictureContexts({
              slices: MULTI_PICTURE_SLICES,
              readings: MULTI_PICTURE_READINGS,
            },);

            for (const [slicePosition, slice,] of MULTI_PICTURE_SLICES.entries()) {
              expect(contexts.get(slice.target.sliceIndex,),).toBe(
                slicePictures({
                  slices: MULTI_PICTURE_SLICES,
                  slicePosition,
                  readings: MULTI_PICTURE_READINGS,
                },).context,
              );
            }
          },
        },),

        it({
          name: 'RENDERS AN EMPTY BLOCK WHERE NO READING IS AVAILABLE, rather than a heading over '
            + 'nothing or a hedge about pictures nobody could read. The consolidation folds a missing '
            + 'entry into this same empty string, so the two spellings of nothing agree',
          fn: async () => {
            const contexts = slicePictureContexts({
              slices: ORDERED_SLICES,
              readings: new Map<string, PairedReading>(),
            },);

            expect([...contexts.values(),],).toEqual([
              '',
              '',
              '',
            ],);
          },
        },),

        it({
          name: 'REFUSES A DOCUMENT WHOSE SLICES CLAIM ONE STAMP TWICE, which `assertSliceIndexing` '
            + 'already forbids upstream. A `Map` keeps the last of a duplicate key silently, and the '
            + 'loss would read as a producer shown the wrong slice\'s pictures rather than as a '
            + 'failure, which is the quietest available way to be wrong about what a passage depicts',
          fn: async () => {
            expect(function foldMisStampedSlices() {
              slicePictureContexts({
                slices: [
                  sliceOf({
                    text: 'Tabby sleeps through the afternoon.\n',
                    sliceIndex: 0,
                  },),
                  sliceOf({
                    text: 'Then she sleeps through the evening.\n',
                    sliceIndex: 0,
                  },),
                ],
                readings: new Map<string, PairedReading>(),
              },);
            },).toThrow(Error,);
          },
        },),
      ],
    },),
  ],
},);
