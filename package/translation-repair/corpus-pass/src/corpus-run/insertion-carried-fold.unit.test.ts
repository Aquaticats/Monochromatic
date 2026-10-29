/**
 Guards class one hundred ten (mikaela12, 2026-09-24): a source-only passage
 the roster found carried, whose evidence sits inside the archive span of the
 neighbouring paired slice, is folded into that carrier's source, so both
 lanes render it as part of the slice that carries it. Left apart, the
 translate lane wrote the carrier from its own source alone, the judges
 preferred the candidate without "the surrounding passage", and the guard at
 publish found the carried sentence gone. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  CarriedInsertion,
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import {
  CARRIED_FOLDED_FINDING,
  CARRIED_SHIFTED_FINDING,
  foldCarriedInsertions,
} from '@monochromatic-dev/translation-repair-insertion';
import { foldPassCarried, } from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 Heading both sides carry.
 */
const HEADING_SOURCE = '## 猫';

/**
 Archive's heading.
 */
const HEADING_TARGET = '## Cat';

/**
 Paragraph the archive folded into its next paragraph's rendering.
 */
const NAP_SOURCE = '猫在窗台睡了一下午。';

/**
 Archive's rendering of the nap, standing inside the homecoming's span.
 */
const NAP_TARGET = 'The cat slept on the sill all afternoon.';

/**
 Paragraph the roster paired with the two-paragraph archive span.
 */
const HOME_SOURCE = '后来猫回家了。';

/**
 Archive's rendering of the homecoming.
 */
const HOME_TARGET = 'Later the cat went home.';

/**
 Closing paragraph, paired one to one.
 */
const BATH_SOURCE = '猫不喜欢洗澡。';

/**
 Archive's rendering of the closing paragraph.
 */
const BATH_TARGET = 'The cat does not like baths.';

/**
 Whole original.
 */
const SOURCE_TEXT = `${HEADING_SOURCE}\n\n${NAP_SOURCE}\n\n${HOME_SOURCE}\n\n${BATH_SOURCE}\n`;

/**
 Whole archive: the nap and the homecoming share one paired span.
 */
const TARGET_TEXT = `${HEADING_TARGET}\n\n${NAP_TARGET}\n\n${HOME_TARGET}\n\n${BATH_TARGET}\n`;

/**
 Builds one content side over the text a fragment occupies.

 @param sliceIndex - where the slice stands

 @param text - whole document

 @param fragment - text the chunk covers, found in that document

 @returns Content chunk at the fragment's offsets

 @example
 ```ts
 const side = contentOver({ sliceIndex: 0, text: SOURCE_TEXT, fragment: HEADING_SOURCE, },);
 ```
 */
function contentOver(
  {
    sliceIndex,
    text,
    fragment,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
    readonly fragment: string;
  },
): ChunkPair['source'] {
  /**
   Where the fragment starts.
   */
  const startOffset = text.indexOf(fragment,);
  if (startOffset === (-1))
    throw new Error(`fixture fragment missing: ${fragment}`,);
  return {
    kind: 'content',
    sliceIndex,
    nodes: [],
    startOffset,
    endOffset: startOffset + fragment.length,
    text: fragment,
  };
}

/**
 The prepared pair the fixture describes: heading, nap (source only), the
 homecoming paired with the archive's nap-and-homecoming span, the bath.

 @returns Prepared pair with four slices

 @example
 ```ts
 const prepared = preparedPair();
 ```
 */
function preparedPair(): PreparedDocumentPair {
  return {
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 4,
    slices: [
      {
        source: contentOver({ sliceIndex: 0, text: SOURCE_TEXT, fragment: HEADING_SOURCE, },),
        target: contentOver({ sliceIndex: 0, text: TARGET_TEXT, fragment: HEADING_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 1, text: SOURCE_TEXT, fragment: NAP_SOURCE, },),
        target: makeInsertionChunk({ sliceIndex: 1, offset: TARGET_TEXT.indexOf(NAP_TARGET,), },),
      },
      {
        source: contentOver({ sliceIndex: 2, text: SOURCE_TEXT, fragment: HOME_SOURCE, },),
        target: contentOver({
          sliceIndex: 2,
          text: TARGET_TEXT,
          fragment: `${NAP_TARGET}\n\n${HOME_TARGET}`,
        },),
      },
      {
        source: contentOver({ sliceIndex: 3, text: SOURCE_TEXT, fragment: BATH_SOURCE, },),
        target: contentOver({ sliceIndex: 3, text: TARGET_TEXT, fragment: BATH_TARGET, },),
      },
    ],
  };
}

/**
 The nap admitted as carried on the given evidence.

 @param evidence - regions the roster anchored

 @returns Admission carrying the nap alone

 @example
 ```ts
 const admission = carriedOn({ evidence: [NAP_TARGET,], },);
 ```
 */
function carriedOn({ evidence, }: { readonly evidence: readonly string[]; },): InsertionAdmission {
  /**
   The nap as the roster recorded it.
   */
  const nap: CarriedInsertion = {
    position: 1,
    sliceIndex: 1,
    sourceText: NAP_SOURCE,
    evidence,
  };
  return {
    positions: new Set(),
    carried: [nap,],
    findings: [],
  };
}

/**
 Original of the flanked shape: the homecoming, then the nap, then the bath.
 */
const FLANKED_SOURCE_TEXT = `${HEADING_SOURCE}\n\n${HOME_SOURCE}\n\n${NAP_SOURCE}\n\n${BATH_SOURCE}\n`;

/**
 Archive of the flanked shape: the homecoming's span carries the nap after it,
 and the bath follows in its own span.
 */
const FLANKED_TARGET_TEXT = `${HEADING_TARGET}\n\n${HOME_TARGET}\n\n${NAP_TARGET}\n\n${BATH_TARGET}\n`;

/**
 The prepared pair where the carried nap stands between two paired slices:
 the homecoming (whose span holds the nap's rendering) before it and the
 bath after it.

 @returns Prepared pair with four slices, the nap at position 2

 @example
 ```ts
 const prepared = flankedPair();
 ```
 */
function flankedPair(): PreparedDocumentPair {
  return {
    sourceText: FLANKED_SOURCE_TEXT,
    targetText: FLANKED_TARGET_TEXT,
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 4,
    slices: [
      {
        source: contentOver({ sliceIndex: 0, text: FLANKED_SOURCE_TEXT, fragment: HEADING_SOURCE, },),
        target: contentOver({ sliceIndex: 0, text: FLANKED_TARGET_TEXT, fragment: HEADING_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 1, text: FLANKED_SOURCE_TEXT, fragment: HOME_SOURCE, },),
        target: contentOver({
          sliceIndex: 1,
          text: FLANKED_TARGET_TEXT,
          fragment: `${HOME_TARGET}\n\n${NAP_TARGET}`,
        },),
      },
      {
        source: contentOver({ sliceIndex: 2, text: FLANKED_SOURCE_TEXT, fragment: NAP_SOURCE, },),
        target: makeInsertionChunk({ sliceIndex: 2, offset: FLANKED_TARGET_TEXT.indexOf(BATH_TARGET,), },),
      },
      {
        source: contentOver({ sliceIndex: 3, text: FLANKED_SOURCE_TEXT, fragment: BATH_SOURCE, },),
        target: contentOver({ sliceIndex: 3, text: FLANKED_TARGET_TEXT, fragment: BATH_TARGET, },),
      },
    ],
  };
}

/**
 The flanked nap admitted as carried on the given evidence.

 @param evidence - regions the roster anchored

 @returns Admission carrying the nap at position 2

 @example
 ```ts
 const admission = flankedCarriedOn({ evidence: [NAP_TARGET,], },);
 ```
 */
function flankedCarriedOn({ evidence, }: { readonly evidence: readonly string[]; },): InsertionAdmission {
  /**
   The nap as the roster recorded it.
   */
  const nap: CarriedInsertion = {
    position: 2,
    sliceIndex: 2,
    sourceText: NAP_SOURCE,
    evidence,
  };
  return {
    positions: new Set(),
    carried: [nap,],
    findings: [],
  };
}

/**
 Closing paragraph of the chained shape, paired with a span that also renders
 the two carried passages before it.
 */
const PLAY_SOURCE = '猫追着毛线玩。';

/**
 Archive's rendering of the play.
 */
const PLAY_TARGET = 'The cat chased the yarn.';

/**
 Original of the chained shape: the homecoming, the nap, the bath, the play.
 */
const CHAINED_SOURCE_TEXT = `${HEADING_SOURCE}\n\n${HOME_SOURCE}\n\n${NAP_SOURCE}\n\n${BATH_SOURCE}\n\n${PLAY_SOURCE}\n`;

/**
 Archive of the chained shape: the play's span carries the nap and the bath
 ahead of it.
 */
const CHAINED_TARGET_TEXT = `${HEADING_TARGET}\n\n${HOME_TARGET}\n\n${NAP_TARGET}\n\n${BATH_TARGET}\n\n${PLAY_TARGET}\n`;

/**
 The prepared pair where two carried passages stand in a row between the
 homecoming and the play, both rendered inside the play's span.

 @returns Prepared pair with five slices, the nap at 2 and the bath at 3

 @example
 ```ts
 const prepared = chainedPair();
 ```
 */
function chainedPair(): PreparedDocumentPair {
  return {
    sourceText: CHAINED_SOURCE_TEXT,
    targetText: CHAINED_TARGET_TEXT,
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 5,
    slices: [
      {
        source: contentOver({ sliceIndex: 0, text: CHAINED_SOURCE_TEXT, fragment: HEADING_SOURCE, },),
        target: contentOver({ sliceIndex: 0, text: CHAINED_TARGET_TEXT, fragment: HEADING_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 1, text: CHAINED_SOURCE_TEXT, fragment: HOME_SOURCE, },),
        target: contentOver({ sliceIndex: 1, text: CHAINED_TARGET_TEXT, fragment: HOME_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 2, text: CHAINED_SOURCE_TEXT, fragment: NAP_SOURCE, },),
        target: makeInsertionChunk({ sliceIndex: 2, offset: CHAINED_TARGET_TEXT.indexOf(NAP_TARGET,), },),
      },
      {
        source: contentOver({ sliceIndex: 3, text: CHAINED_SOURCE_TEXT, fragment: BATH_SOURCE, },),
        target: makeInsertionChunk({ sliceIndex: 3, offset: CHAINED_TARGET_TEXT.indexOf(BATH_TARGET,), },),
      },
      {
        source: contentOver({ sliceIndex: 4, text: CHAINED_SOURCE_TEXT, fragment: PLAY_SOURCE, },),
        target: contentOver({
          sliceIndex: 4,
          text: CHAINED_TARGET_TEXT,
          fragment: `${NAP_TARGET}\n\n${BATH_TARGET}\n\n${PLAY_TARGET}`,
        },),
      },
    ],
  };
}

/**
 Kitten's question, the first of two quoted lines.
 */
const ASK_SOURCE = '「你喜欢小猫吗？」';

/**
 Kitten's second quoted line, which the archive renders inside the first
 line's quote.
 */
const PURR_SOURCE = '「被抱着的时候，小猫会呼噜！」';

/**
 Archive's quote rendering both lines in one block.
 */
const QUOTE_TARGET = '> Do you like the kitten?\n>\n> The kitten purrs when held!';

/**
 Original of the shifted shape: both quoted lines, the nap, the bath.
 */
const SHIFTED_SOURCE_TEXT = `${HEADING_SOURCE}\n\n${ASK_SOURCE}\n\n${PURR_SOURCE}\n\n${NAP_SOURCE}\n\n${BATH_SOURCE}\n`;

/**
 Archive of the shifted shape: one quote for both lines, then the nap, then
 the bath.
 */
const SHIFTED_TARGET_TEXT = `${HEADING_TARGET}\n\n${QUOTE_TARGET}\n\n${NAP_TARGET}\n\n${BATH_TARGET}\n`;

/**
 The prepared pair the pairing left one off: the question holds the whole
 quote, the purr is paired with the nap's rendering, and the nap stands
 carried with that rendering as its evidence.

 @returns Prepared pair with five slices, the nap at position 3

 @example
 ```ts
 const prepared = shiftedPair();
 ```
 */
function shiftedPair(): PreparedDocumentPair {
  return {
    sourceText: SHIFTED_SOURCE_TEXT,
    targetText: SHIFTED_TARGET_TEXT,
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 5,
    slices: [
      {
        source: contentOver({ sliceIndex: 0, text: SHIFTED_SOURCE_TEXT, fragment: HEADING_SOURCE, },),
        target: contentOver({ sliceIndex: 0, text: SHIFTED_TARGET_TEXT, fragment: HEADING_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 1, text: SHIFTED_SOURCE_TEXT, fragment: ASK_SOURCE, },),
        target: contentOver({ sliceIndex: 1, text: SHIFTED_TARGET_TEXT, fragment: QUOTE_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 2, text: SHIFTED_SOURCE_TEXT, fragment: PURR_SOURCE, },),
        target: contentOver({ sliceIndex: 2, text: SHIFTED_TARGET_TEXT, fragment: NAP_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 3, text: SHIFTED_SOURCE_TEXT, fragment: NAP_SOURCE, },),
        target: makeInsertionChunk({ sliceIndex: 3, offset: SHIFTED_TARGET_TEXT.indexOf(BATH_TARGET,), },),
      },
      {
        source: contentOver({ sliceIndex: 4, text: SHIFTED_SOURCE_TEXT, fragment: BATH_SOURCE, },),
        target: contentOver({ sliceIndex: 4, text: SHIFTED_TARGET_TEXT, fragment: BATH_TARGET, },),
      },
    ],
  };
}

/**
 The nap of the shifted shape admitted as carried on its rendering.

 @returns Admission carrying the nap at position 3

 @example
 ```ts
 const admission = shiftedCarried();
 ```
 */
function shiftedCarried(): InsertionAdmission {
  return {
    positions: new Set(),
    carried: [{ position: 3, sliceIndex: 3, sourceText: NAP_SOURCE, evidence: [NAP_TARGET, NAP_TARGET,], },],
    findings: [],
  };
}

/**
 Archive of the mirrored shift: the nap rendered in the paragraph the pairing
 gave the homecoming, and the homecoming and the bath rendered as one
 paragraph after it.
 */
const MIRRORED_TARGET_TEXT = `${HEADING_TARGET}\n\n${NAP_TARGET}\n\n${HOME_TARGET} ${BATH_TARGET}\n`;

/**
 The prepared pair the pairing left one off on the far side: the nap stands
 carried, the homecoming holds the nap's rendering alone, and the bath holds
 the homecoming's rendering beside its own.

 @returns Prepared pair with four slices, the nap at position 1

 @example
 ```ts
 const prepared = mirroredPair();
 ```
 */
function mirroredPair(): PreparedDocumentPair {
  return {
    sourceText: SOURCE_TEXT,
    targetText: MIRRORED_TARGET_TEXT,
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 4,
    slices: [
      {
        source: contentOver({ sliceIndex: 0, text: SOURCE_TEXT, fragment: HEADING_SOURCE, },),
        target: contentOver({ sliceIndex: 0, text: MIRRORED_TARGET_TEXT, fragment: HEADING_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 1, text: SOURCE_TEXT, fragment: NAP_SOURCE, },),
        target: makeInsertionChunk({ sliceIndex: 1, offset: MIRRORED_TARGET_TEXT.indexOf(NAP_TARGET,), },),
      },
      {
        source: contentOver({ sliceIndex: 2, text: SOURCE_TEXT, fragment: HOME_SOURCE, },),
        target: contentOver({ sliceIndex: 2, text: MIRRORED_TARGET_TEXT, fragment: NAP_TARGET, },),
      },
      {
        source: contentOver({ sliceIndex: 3, text: SOURCE_TEXT, fragment: BATH_SOURCE, },),
        target: contentOver({
          sliceIndex: 3,
          text: MIRRORED_TARGET_TEXT,
          fragment: `${HOME_TARGET} ${BATH_TARGET}`,
        },),
      },
    ],
  };
}

/**
 Runs the pass's fold over the nap carried on one region, keeping every line
 it logged behind its level.

 @param region - one region the roster anchored

 @returns Admission after the fold beside the lines

 @example
 ```ts
 const { admission, lines, } = passFoldOver({ region: NAP_TARGET, },);
 ```
 */
function passFoldOver(
  { region, }: { readonly region: string; },
): {
  readonly admission: InsertionAdmission;
  readonly lines: readonly string[];
} {
  /**
   Lines the fold logged.
   */
  const lines: string[] = [];
  /**
   The fold's outcome.
   */
  const { admission, } = foldPassCarried({
    paired: preparedPair(),
    admission: carriedOn({ evidence: [region,], },),
    l: levelCapturingLogger({ lines, },),
  },);
  return {
    admission,
    lines,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: foldCarriedInsertions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FOLDS a carried passage into the adjacent slice whose archive span holds every evidence region',
          fn: async () => {
            /**
             Same region quoted by three voices.
             */
            const folded = foldCarriedInsertions({
              prepared: preparedPair(),
              admission: carriedOn({ evidence: [NAP_TARGET, NAP_TARGET, NAP_TARGET,], },),
            },);
            /**
             The carrier after the fold.
             */
            const { 2: carrier, } = folded.prepared
              .slices;
            expect(carrier?.source.text,).toBe(`${NAP_SOURCE}\n\n${HOME_SOURCE}`,);
            expect(carrier?.source.startOffset,).toBe(SOURCE_TEXT.indexOf(NAP_SOURCE,),);
            expect(carrier?.source.endOffset,).toBe(SOURCE_TEXT.indexOf(HOME_SOURCE,) + HOME_SOURCE.length,);
            expect(carrier?.target.text,).toBe(`${NAP_TARGET}\n\n${HOME_TARGET}`,);
            expect(folded.prepared.slices.length,).toBe(4,);
            expect(folded.prepared.slices[1]?.target.kind,).toBe('insertion',);
            expect(folded.admission.carried,).toEqual([],);
            expect(folded.admission.folded,).toEqual([{ position: 1, sliceIndex: 1, carrierSliceIndex: 2, },],);
            expect(folded.findings,).toEqual([`${CARRIED_FOLDED_FINDING} (slice 1 into slice 2)`,],);
          },
        },),
        it({
          name: 'STANDS ASIDE where the evidence is not adjacent, not found, straddles two slices, or the source gap is not blank',
          fn: async () => {
            /**
             Evidence inside the bath slice, two positions away.
             */
            const apart = foldCarriedInsertions({
              prepared: preparedPair(),
              admission: carriedOn({ evidence: [BATH_TARGET,], },),
            },);
            expect(apart.prepared.slices[2]?.source.text,).toBe(HOME_SOURCE,);
            expect(apart.admission.carried?.length,).toBe(1,);
            expect(apart.admission.folded,).toEqual([],);
            expect(apart.findings,).toEqual([],);

            /**
             Evidence the page never carried.
             */
            const unfound = foldCarriedInsertions({
              prepared: preparedPair(),
              admission: carriedOn({ evidence: ['The dog barked.',], },),
            },);
            expect(unfound.prepared.slices[2]?.source.text,).toBe(HOME_SOURCE,);
            expect(unfound.admission.carried?.length,).toBe(1,);

            /**
             Evidence crossing from the homecoming into the bath.
             */
            const straddling = foldCarriedInsertions({
              prepared: preparedPair(),
              admission: carriedOn({ evidence: [`${HOME_TARGET}\n\n${BATH_TARGET}`,], },),
            },);
            expect(straddling.prepared.slices[2]?.source.text,).toBe(HOME_SOURCE,);
            expect(straddling.admission.carried?.length,).toBe(1,);

            /**
             Evidence in an archive paragraph no paired span holds: the page
             carries it after the bath, and no slice's target reaches it.
             */
            const unheldPair = preparedPair();
            const unheld = foldCarriedInsertions({
              prepared: {
                ...unheldPair,
                targetText: `${TARGET_TEXT}\nThe dog barked.\n`,
              },
              admission: carriedOn({ evidence: ['The dog barked.',], },),
            },);
            expect(unheld.prepared.slices[2]?.source.text,).toBe(HOME_SOURCE,);
            expect(unheld.admission.carried?.length,).toBe(1,);
            expect(unheld.asides,).toEqual([
              'slice 1 stays carried: evidence a block of the region sits in no paired slice',
            ],);

            /**
             A carrier whose source starts past a word the pairing left to nobody.
             */
            const gapped = preparedPair();
            /**
             The homecoming's source shifted one code point in.
             */
            const shifted: PreparedDocumentPair = {
              ...gapped,
              slices: gapped.slices.map(function shiftHome(slice,): ChunkPair {
                if (slice.target.sliceIndex !== 2)
                  return slice;
                return {
                  ...slice,
                  source: {
                    ...slice.source,
                    startOffset: slice.source.startOffset + 1,
                    text: slice.source.text.slice(1,),
                  },
                };
              },),
            };
            const blocked = foldCarriedInsertions({
              prepared: shifted,
              admission: carriedOn({ evidence: [NAP_TARGET,], },),
            },);
            expect(blocked.prepared.slices[2]?.source.text,).toBe(HOME_SOURCE.slice(1,),);
            expect(blocked.admission.carried?.length,).toBe(1,);

            /**
             Nothing carried at all.
             */
            const idle = foldCarriedInsertions({
              prepared: preparedPair(),
              admission: { positions: new Set(), findings: [], },
            },);
            expect(idle.admission.folded,).toBeUndefined();
            expect(idle.findings,).toEqual([],);
          },
        },),
        it({
          name: 'FOLDS a passage whose evidence straddles both neighbours into the one holding the larger share (class one hundred eleven)',
          fn: async () => {
            /**
             The voice quoted the nap and ran on into the bath's line: most of the
             region sits in the homecoming's span.
             */
            const intoHome = foldCarriedInsertions({
              prepared: flankedPair(),
              admission: flankedCarriedOn({ evidence: [`${NAP_TARGET}\n\n${BATH_TARGET}`,], },),
            },);
            expect(intoHome.prepared.slices[1]?.source.text,).toBe(`${HOME_SOURCE}\n\n${NAP_SOURCE}`,);
            expect(intoHome.prepared.slices[3]?.source.text,).toBe(BATH_SOURCE,);
            expect(intoHome.admission.carried,).toEqual([],);
            expect(intoHome.admission.folded,).toEqual([{ position: 2, sliceIndex: 2, carrierSliceIndex: 1, },],);
            expect(intoHome.findings,).toEqual([`${CARRIED_FOLDED_FINDING} (slice 2 into slice 1)`,],);
            expect(intoHome.asides,).toEqual([],);

            /**
             The voice quoted the nap's tail and the whole bath line: most of the
             region sits in the bath's span.
             */
            const intoBath = foldCarriedInsertions({
              prepared: flankedPair(),
              admission: flankedCarriedOn({ evidence: [`${NAP_TARGET.slice(-10,)}\n\n${BATH_TARGET}`,], },),
            },);
            expect(intoBath.prepared.slices[1]?.source.text,).toBe(HOME_SOURCE,);
            expect(intoBath.prepared.slices[3]?.source.text,).toBe(`${NAP_SOURCE}\n\n${BATH_SOURCE}`,);
            expect(intoBath.admission.folded,).toEqual([{ position: 2, sliceIndex: 2, carrierSliceIndex: 3, },],);
            expect(intoBath.findings,).toEqual([`${CARRIED_FOLDED_FINDING} (slice 2 into slice 3)`,],);

            /**
             A stand-aside names its reason.
             */
            const unfound = foldCarriedInsertions({
              prepared: flankedPair(),
              admission: flankedCarriedOn({ evidence: ['The dog barked.',], },),
            },);
            expect(unfound.admission.carried?.length,).toBe(1,);
            expect(unfound.asides.length,).toBe(1,);
            expect(unfound.asides[0],).toContain('slice 2',);
            expect(unfound.asides[0],).toContain('quote-not-found',);
          },
        },),
        it({
          name: 'FOLDS a chain of carried passages through a carried neighbour into the paired slice beyond it, whatever the admission order',
          fn: async () => {
            /**
             The nap listed first: on the first pass the bath's source still
             stands between the nap and the play, so the nap waits; the bath
             folds, the play's source widens, and the nap folds on the next pass.
             */
            const chained = foldCarriedInsertions({
              prepared: chainedPair(),
              admission: {
                positions: new Set(),
                carried: [
                  { position: 2, sliceIndex: 2, sourceText: NAP_SOURCE, evidence: [NAP_TARGET,], },
                  { position: 3, sliceIndex: 3, sourceText: BATH_SOURCE, evidence: [BATH_TARGET,], },
                ],
                findings: [],
              },
            },);
            expect(chained.prepared.slices[4]?.source.text,).toBe(`${NAP_SOURCE}\n\n${BATH_SOURCE}\n\n${PLAY_SOURCE}`,);
            expect(chained.prepared.slices[1]?.source.text,).toBe(HOME_SOURCE,);
            expect(chained.admission.carried,).toEqual([],);
            expect(chained.admission.folded,).toEqual([
              { position: 3, sliceIndex: 3, carrierSliceIndex: 4, },
              { position: 2, sliceIndex: 2, carrierSliceIndex: 4, },
            ],);
            expect(chained.findings,).toEqual([
              `${CARRIED_FOLDED_FINDING} (slice 3 into slice 4)`,
              `${CARRIED_FOLDED_FINDING} (slice 2 into slice 4)`,
            ],);
            expect(chained.asides,).toEqual([],);

            /**
             The bath listed first: it folds into the play, the play's source
             then abuts the nap's, and the nap folds on the same pass, so the
             passes after it find nothing left to fold.
             */
            const bathFirst = foldCarriedInsertions({
              prepared: chainedPair(),
              admission: {
                positions: new Set(),
                carried: [
                  { position: 3, sliceIndex: 3, sourceText: BATH_SOURCE, evidence: [BATH_TARGET,], },
                  { position: 2, sliceIndex: 2, sourceText: NAP_SOURCE, evidence: [NAP_TARGET,], },
                ],
                findings: [],
              },
            },);
            expect(bathFirst.prepared.slices,).toEqual(chained.prepared.slices,);
            expect(bathFirst.admission.carried,).toEqual([],);
            expect(bathFirst.admission.folded,).toEqual(chained.admission.folded,);
            expect(bathFirst.findings,).toEqual(chained.findings,);
            expect(bathFirst.asides,).toEqual([],);

            /**
             The bath still carried and not folding (its evidence nowhere): the
             nap's source does not abut the play's, so it stays carried with the
             bath's source named as the gap.
             */
            const blocked = foldCarriedInsertions({
              prepared: chainedPair(),
              admission: {
                positions: new Set(),
                carried: [
                  { position: 2, sliceIndex: 2, sourceText: NAP_SOURCE, evidence: [NAP_TARGET,], },
                  { position: 3, sliceIndex: 3, sourceText: BATH_SOURCE, evidence: ['The dog barked.',], },
                ],
                findings: [],
              },
            },);
            expect(blocked.prepared.slices[4]?.source.text,).toBe(PLAY_SOURCE,);
            expect(blocked.admission.carried?.length,).toBe(2,);
            expect(blocked.asides.length,).toBe(2,);
            expect(blocked.asides[0],).toContain('blank space',);
          },
        },),
        it({
          name: 'SHIFTS the carrier\'s own source to its far neighbour where the carrier\'s archive span renders the carried passage alone (class one hundred seventy-nine)',
          fn: async () => {
            // THE FAILURE THIS CLOSES. TianqiChen66611 (2026-09-26): the pairing
            // gave the second quoted line the archive's next paragraph, whose
            // English renders the carried passage after it; the second line's
            // English sat inside the first line's quote. The plain fold widened
            // the carrier over both sources, so the carrier rendered the second
            // line again beside the passage and the page carried it twice.
            const shifted = foldCarriedInsertions({
              prepared: shiftedPair(),
              admission: shiftedCarried(),
            },);
            expect(shifted.prepared.slices[1]?.source.text,).toBe(`${ASK_SOURCE}\n\n${PURR_SOURCE}`,);
            expect(shifted.prepared.slices[1]?.source.sliceIndex,).toBe(1,);
            expect(shifted.prepared.slices[1]?.target.text,).toBe(QUOTE_TARGET,);
            expect(shifted.prepared.slices[2]?.source.text,).toBe(NAP_SOURCE,);
            expect(shifted.prepared.slices[2]?.source.sliceIndex,).toBe(2,);
            expect(shifted.prepared.slices[2]?.target.text,).toBe(NAP_TARGET,);
            expect(shifted.prepared.slices[4]?.source.text,).toBe(BATH_SOURCE,);
            expect(shifted.admission.carried,).toEqual([],);
            expect(shifted.admission.folded,).toEqual([{ position: 3, sliceIndex: 3, carrierSliceIndex: 2, },],);
            expect(shifted.findings,).toEqual([
              `${CARRIED_FOLDED_FINDING} (slice 3 into slice 2)`,
              `${CARRIED_SHIFTED_FINDING} (slice 2's own source joins slice 1: slice 2's archive span renders slice 3's `
                + 'passage alone)',
            ],);
            expect(shifted.asides,).toEqual([],);

            /**
             The question's source ending short of a mark the pairing left to
             nobody: the purr cannot join it, so the plain fold stands.
             */
            const gapped = shiftedPair();
            /**
             The question's source cut one code point short.
             */
            const apart: PreparedDocumentPair = {
              ...gapped,
              slices: gapped.slices.map(function shortenAsk(slice,): ChunkPair {
                if (slice.target.sliceIndex !== 1)
                  return slice;
                return {
                  ...slice,
                  source: {
                    ...slice.source,
                    endOffset: slice.source.endOffset - 1,
                    text: slice.source.text.slice(0, -1,),
                  },
                };
              },),
            };
            const plain = foldCarriedInsertions({
              prepared: apart,
              admission: shiftedCarried(),
            },);
            expect(plain.prepared.slices[1]?.source.text,).toBe(ASK_SOURCE.slice(0, -1,),);
            expect(plain.prepared.slices[2]?.source.text,).toBe(`${PURR_SOURCE}\n\n${NAP_SOURCE}`,);
            expect(plain.findings,).toEqual([`${CARRIED_FOLDED_FINDING} (slice 3 into slice 2)`,],);

            /**
             Neither the heading nor the question paired: the purr has no paired
             slice before it to take its own source, so the plain fold stands.
             */
            const headless = shiftedPair();
            const noReceiver = foldCarriedInsertions({
              prepared: {
                ...headless,
                slices: headless.slices.map(function unpairOpening(slice,): ChunkPair {
                  if (slice.target.sliceIndex > 1)
                    return slice;
                  return {
                    ...slice,
                    target: makeInsertionChunk({
                      sliceIndex: slice.target.sliceIndex,
                      offset: 0,
                    },),
                  };
                },),
              },
              admission: shiftedCarried(),
            },);
            expect(noReceiver.prepared.slices[2]?.source.text,).toBe(`${PURR_SOURCE}\n\n${NAP_SOURCE}`,);
            expect(noReceiver.findings,).toEqual([`${CARRIED_FOLDED_FINDING} (slice 3 into slice 2)`,],);
          },
        },),
        it({
          name: 'SHIFTS the carrier\'s own source to the slice after it where the carrier follows the passage, the '
            + 'mirror of the TianqiChen66611 shape (class one hundred seventy-nine)',
          fn: async () => {
            /**
             The homecoming paired with the nap's rendering, the bath with the
             homecoming's and its own.
             */
            const mirrored = foldCarriedInsertions({
              prepared: mirroredPair(),
              admission: carriedOn({ evidence: [NAP_TARGET,], },),
            },);
            expect(mirrored.prepared.slices[2]?.source.text,).toBe(NAP_SOURCE,);
            expect(mirrored.prepared.slices[2]?.source.sliceIndex,).toBe(2,);
            expect(mirrored.prepared.slices[3]?.source.text,).toBe(`${HOME_SOURCE}\n\n${BATH_SOURCE}`,);
            expect(mirrored.prepared.slices[3]?.source.sliceIndex,).toBe(3,);
            expect(mirrored.admission.folded,).toEqual([{ position: 1, sliceIndex: 1, carrierSliceIndex: 2, },],);
            expect(mirrored.findings,).toEqual([
              `${CARRIED_FOLDED_FINDING} (slice 1 into slice 2)`,
              `${CARRIED_SHIFTED_FINDING} (slice 2's own source joins slice 3: slice 2's archive span renders slice 1's `
                + 'passage alone)',
            ],);
            expect(mirrored.asides,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: foldPassCarried.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LOGS A FOLD at information, with the region the passage was carried on, under the pass\'s tag',
          fn: async () => {
            /**
             The nap carried on its own rendering, inside the homecoming's span.
             */
            const folded = passFoldOver({ region: NAP_TARGET, },);
            expect({
              lines: folded.lines,
              stillCarried: folded.admission.carried,
            },).toEqual({
              lines: [
                `info [${foldPassCarried.name}] ${CARRIED_FOLDED_FINDING} (slice 1 into slice 2)`,
                `info [${foldPassCarried.name}] slice 1 carried on 1 region(s): ${JSON.stringify(NAP_TARGET,)}`,
              ],
              stillCarried: [],
            },);
          },
        },),
        it({
          name: 'WARNS A STAND-ASIDE naming why the passage stays carried, beside the region it was carried on, so a '
            + 'log reader tells a passage the fold could not place from one it folded (class one hundred eleven)',
          fn: async () => {
            /**
             The nap carried on the bath's rendering, two slices away.
             */
            const aside = passFoldOver({ region: BATH_TARGET, },);
            /**
             The lines before the region line, which should be the one warning.
             */
            const [warning = '', ...rest] = aside.lines;
            expect({
              warns: warning.startsWith(`warn [${foldPassCarried.name}] slice 1 stays carried: `,),
              rest,
              stillCarried: aside.admission.carried?.length,
            },).toEqual({
              warns: true,
              rest: [`info [${foldPassCarried.name}] slice 1 carried on 1 region(s): ${JSON.stringify(BATH_TARGET,)}`,],
              stillCarried: 1,
            },);
          },
        },),
      ],
    },),
  ],
},);
