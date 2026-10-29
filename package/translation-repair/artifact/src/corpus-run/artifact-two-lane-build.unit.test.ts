/**
 Tests for the two-lane settled artifact.

 WHAT THESE PIN is what the builder DERIVES rather than accepts. Version 1
 took a status and two counts beside the result they described, so a caller
 could state a status the result contradicted and counts nothing had counted.
 The version 2 builder takes only what cannot be computed from the run, and
 everything else, the preparation identity and the whole lane comparison
 included, comes off the preparation and the two ledgers.

 They also pin the shape the generation exists for: no lane at the top level,
 and a lane selection that says out loud that nobody has picked one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  type PreparationIdentity,
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  ArtifactPreparationMismatchError,
  buildSettledTwoLaneArtifact,
  NO_CONSISTENCY,
  NO_PAGE_ASSEMBLY,
} from '../../dist/final/node/index.mjs';

/**
 Archive wording of the first slice, which the archive does translate.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Original of that slice.
 */
const SOURCE_NAP = '猫猫在窗台上睡觉。';

/**
 Original of the second slice, which the archive never translated.
 */
const SOURCE_BOWL = '猫猫有自己的碗。';

/**
 Built pipeline these fixtures claim to have run under.
 */
const DIGEST = 'sha256-tree-v1:'.concat('c'.repeat(64,),) as unknown as PipelineDigest;

/**
 Two prepared slices: one the archive translates, one it never did.

 @returns Pairs shaped as preparation produces them

 @example
 ```ts
 const slices = catSlices();
 ```
 */
function catSlices(): readonly ChunkPair[] {
  return [
    {
      source: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: SOURCE_NAP.length,
        text: SOURCE_NAP,
      },
      target: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: ARCHIVE_NAP.length,
        text: ARCHIVE_NAP,
      },
    },
    {
      source: {
        sliceIndex: 1,
        nodes: [],
        startOffset: SOURCE_NAP.length,
        endOffset: SOURCE_NAP.length + SOURCE_BOWL.length,
        text: SOURCE_BOWL,
      },
      target: makeInsertionChunk({
        sliceIndex: 1,
        offset: ARCHIVE_NAP.length,
      },),
    },
  ];
}

/**
 Preparation both lanes claim to have run over.

 @returns Preparation shaped as `prepareDocumentPair` returns one

 @example
 ```ts
 const prepared = catPreparation();
 ```
 */
function catPreparation(): PreparedDocumentPair {
  return {
    sourceText: SOURCE_NAP + SOURCE_BOWL,
    targetText: ARCHIVE_NAP,
    slices: catSlices(),
    lineStructuredSliceIndices: new Set<number>(),
    declaredNames: [],
    alignmentFindings: [],
    alignmentPairCount: 2,
  } as unknown as PreparedDocumentPair;
}

/**
 Name the cat preparation gives itself, which the driver stamps on both
 ledgers it builds.

 @returns Identity of {@link catPreparation}'s slicing

 @example
 ```ts
 const identity = catIdentity();
 ```
 */
function catIdentity(): PreparationIdentity {
  return preparationIdentity({ prepared: catPreparation(), },);
}

/**
 Repair lane ledger: it mended the first slice and had no work at the anchor.

 @returns Two rows, one per prepared slice

 @example
 ```ts
 const rows = repairLedger();
 ```
 */
function repairLedger(): readonly SliceDeliveryRecord[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_NAP,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_NAP,
      outcome: {
        kind: 'decided',
        acceptedText: 'The cat is asleep on the windowsill.',
      },
      shippedText: 'The cat is asleep on the windowsill.',
      delivery: { kind: 'replacement-shipped', },
    },
    {
      sliceIndex: 1,
      sourceText: SOURCE_BOWL,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: { kind: 'not-applicable', },
      shippedText: '',
      delivery: { kind: 'gap-remains', },
    },
  ];
}

/**
 Translate lane ledger: it kept the archive's first slice and filled the gap.

 @returns Two rows, one per prepared slice

 @example
 ```ts
 const rows = translateLedger();
 ```
 */
function translateLedger(): readonly SliceDeliveryRecord[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_NAP,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_NAP,
      outcome: {
        kind: 'decided',
        acceptedText: ARCHIVE_NAP,
      },
      shippedText: ARCHIVE_NAP,
      delivery: { kind: 'incumbent-retained', },
    },
    {
      sliceIndex: 1,
      sourceText: SOURCE_BOWL,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: {
        kind: 'decided',
        acceptedText: 'The cat has a bowl of its own.',
      },
      shippedText: 'The cat has a bowl of its own.',
      delivery: { kind: 'replacement-shipped', },
    },
  ];
}

/**
 What both lanes returned over that preparation.

 @returns Driver result shaped as `runDocumentLanes` returns one

 @example
 ```ts
 const lanes = catLanes();
 ```
 */
function catLanes(): DocumentLanesResult {
  return {
    alignmentFindings: [],
    repair: {
      repairedText: 'The cat is asleep on the windowsill.',
      status: 'repaired',
      sliceCount: 2,
    },
    translate: {
      translatedText: `${ARCHIVE_NAP}\n\nThe cat has a bowl of its own.`,
      status: 'complete',
      sliceCount: 2,
    },
    repairDelivery: { preparationIdentity: catIdentity(), records: repairLedger(), },
    translateDelivery: { preparationIdentity: catIdentity(), records: translateLedger(), },
  } as unknown as DocumentLanesResult;
}

/**
 Builds the artifact these cases read.

 @returns Artifact over the cat preparation and both lanes

 @example
 ```ts
 const artifact = catArtifact();
 ```
 */
function catArtifact(): ReturnType<typeof buildSettledTwoLaneArtifact> {
  return buildSettledTwoLaneArtifact({
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: NO_CONSISTENCY,
    entryId: 'CatEntry1',
    tip: 'a'.repeat(40,),
    pipelineDigest: DIGEST,
    corpusSha: 'b'.repeat(40,),
    callConfig: { perCallTimeoutMs: 600_000, },
    durationMs: 1_234,
    prepared: catPreparation(),
    lanes: catLanes(),
    laneSelection: { kind: 'pending-human-decision', },
    consolidation: { kind: 'not-run', },
  },);
}

await describe({
  name: buildSettledTwoLaneArtifact.name,
  children: [
    it({
      name:
        'nests both lanes and puts NEITHER at the top level, which is the whole point of the generation: '
          + 'leaving the repair fields where they were and appending a translate block would answer the '
          + 'open question invisibly, since every existing reader would go on taking the top-level fields '
          + 'as the output and never learn a second lane existed',
      fn: async () => {
        /**
         Artifact over both lanes.
         */
        const artifact = catArtifact();
        expect(Object.keys(artifact.lanes,)
          .toSorted(),).toEqual([
            'repair',
            'translate',
          ],);

        // None of version 1's top-level lane fields survive.
        for (const absent of [
          'status',
          'issues',
          'repairedText',
          'changedSliceIndices',
          'withdrawnSliceIndices',
          'acceptedCount',
        ]) {
          expect(Object.hasOwn(
            artifact,
            absent,
          ),).toBe(false,);
        }
      },
    },),
    it({
      name:
        'says out loud that nobody has picked a lane, rather than leaving the field out: an absent winner '
          + 'would make "not decided yet" and "written before anyone asked" the same absence, which is the '
          + 'defect class this generation exists to end',
      fn: async () => {
        expect(catArtifact().laneSelection,).toEqual({ kind: 'pending-human-decision', },);
      },
    },),
    it({
      name:
        'DERIVES the preparation identity from the preparation rather than taking it, so the identity an '
          + 'artifact records is the identity of the thing the artifact describes',
      fn: async () => {
        /**
         Artifact over the cat preparation.
         */
        const artifact = catArtifact();
        expect(artifact.preparation
          .identity,).toMatch('sha256-preparation-v2:',);

        // Two builds of the same preparation agree, which is what makes a
        // resumed pass comparable with the one it resumed.
        expect(catArtifact().preparation
          .identity,).toBe(artifact.preparation
            .identity,);
      },
    },),
    it({
      name:
        'DERIVES the comparison from the two ledgers rather than accepting one, since a comparison '
          + 'supplied beside the ledgers it describes could disagree with them and a reader would have no '
          + 'way to tell which of the two to believe',
      fn: async () => {
        /**
         Artifact over one mended slice and one filled gap.
         */
        const artifact = catArtifact();
        expect(artifact.comparison,).toHaveLength(2,);
        expect(artifact.comparison
          .map(function toVerdict(row,): string {
            return row.laneRelation;
          },),).toEqual([
            'repair-only',
            'translate-only',
          ],);

        // And the anchor reads as the repair lane having had no work rather
        // than as the two lanes disagreeing about a wording.
        expect(artifact.comparison[1]?.decisionComparison,).toEqual({
          kind: 'not-comparable',
          undecidedLanes: ['repair',],
        },);
      },
    },),
    it({
      name:
        'records the preparation ONCE, with the slice count both lanes are out of and both measures of '
          + 'the original: the character count for reading a log and the byte count band classification '
          + 'actually takes, which are routinely mistaken for each other',
      fn: async () => {
        /**
         Artifact over a two-slice preparation.
         */
        const artifact = catArtifact();
        expect(artifact.preparation
          .sliceCount,).toBe(2,);
        expect(artifact.preparation
          .sourceChars,).toBe((SOURCE_NAP + SOURCE_BOWL).length,);
        // Han text runs about three bytes per character, so the byte count is
        // the larger of the two and no reader can confuse them by value.
        expect(artifact.preparation
          .sourceBytes,).toBeGreaterThan(artifact.preparation
            .sourceChars,);
      },
    },),
    it({
      name:
        'REFUSES to build an artifact for a run whose two ledgers cannot be compared, because an entry '
          + 'whose lanes disagree about their own preparation has nothing worth writing',
      fn: async () => {
        /**
         Repair ledger whose anchor row agrees with the preparation on every
         per-slice fact and contradicts ITSELF: a lane with no work to do
         there, over a passage the archive never had, reported as a
         replacement the document carries.
         */
        const incoherent = {
          preparationIdentity: catIdentity(),
          records: repairLedger()
            .map(function shipTheAnchor(record, position,): SliceDeliveryRecord {
              return (position === 1)
                ? {
                  ...record,
                  delivery: { kind: 'replacement-shipped', },
                }
                : record;
            },),
        };

        expect(function lanesDisagree() {
          buildSettledTwoLaneArtifact({
            pageAssembly: NO_PAGE_ASSEMBLY,
            consistency: NO_CONSISTENCY,
            entryId: 'CatEntry1',
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            corpusSha: 'b'.repeat(40,),
            callConfig: {},
            durationMs: 1,
            prepared: catPreparation(),
            lanes: {
              ...catLanes(),
              repairDelivery: incoherent,
            },
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: { kind: 'not-run', },
          },);
        },).toThrow();
      },
    },),
    it({
      name:
        'REFUSES two ledgers that agree with each other and cover FEWER slices than the preparation, '
          + 'which the comparison passes: it checks the two against each other, and a pair of equally '
          + 'short ledgers line up perfectly while describing a document with slices missing',
      fn: async () => {
        /**
         Both lanes truncated to the same single row, so nothing about them
         disagrees except with the preparation they are filed under.
         */
        const bothShort = {
          ...catLanes(),
          repairDelivery: {
            preparationIdentity: catIdentity(),
            records: repairLedger()
              .slice(0, 1,),
          },
          translateDelivery: {
            preparationIdentity: catIdentity(),
            records: translateLedger()
              .slice(0, 1,),
          },
        };

        /**
         What bothLedgersAreShort raised, read for its class as well as its wording.
         */
        const refusalOfBothLedgersAreShort = caught(function bothLedgersAreShort() {
          buildSettledTwoLaneArtifact({
            pageAssembly: NO_PAGE_ASSEMBLY,
            consistency: NO_CONSISTENCY,
            entryId: 'CatEntry1',
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            corpusSha: 'b'.repeat(40,),
            callConfig: {},
            durationMs: 1,
            prepared: catPreparation(),
            lanes: bothShort,
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: { kind: 'not-run', },
          },);
        },);

        expect(refusalOfBothLedgersAreShort,).toBeInstanceOf(ArtifactPreparationMismatchError,);
        expect((refusalOfBothLedgersAreShort as Error).message,).toContain('1 row for a preparation of 2 slices',);
      },
    },),
    it({
      name:
        'REFUSES a preparation the ledgers were not built over, which nothing else can catch: the '
          + 'comparison proves the two ledgers agree with EACH OTHER, and two ledgers from some other '
          + 'preparation agree with each other perfectly',
      fn: async () => {
        /**
         Preparation whose first slice renders a different original, which is
         a whole other pair of documents wearing the same slice count.
         */
        const foreign = {
          ...catPreparation(),
          slices: catSlices()
            .map(function retranslate(pair, position,) {
              return (position === 0)
                ? {
                  ...pair,
                  source: {
                    ...pair.source,
                    text: '猫猫在椅子上睡觉。',
                  },
                }
                : pair;
            },),
        } as unknown as PreparedDocumentPair;

        /**
         What ledgersDescribeAnotherPreparation raised, read for its class as well as its wording.
         */
        const refusalOfLedgersDescribeAnotherPreparation = caught(function ledgersDescribeAnotherPreparation() {
          buildSettledTwoLaneArtifact({
            pageAssembly: NO_PAGE_ASSEMBLY,
            consistency: NO_CONSISTENCY,
            entryId: 'CatEntry1',
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            corpusSha: 'b'.repeat(40,),
            callConfig: {},
            durationMs: 1,
            prepared: foreign,
            lanes: catLanes(),
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: { kind: 'not-run', },
          },);
        },);

        expect(refusalOfLedgersDescribeAnotherPreparation,).toBeInstanceOf(ArtifactPreparationMismatchError,);
        expect((refusalOfLedgersDescribeAnotherPreparation as Error).message,).toContain('was built over',);
      },
    },),
    it({
      name:
        'REFUSES a ledger whose rows contradict the preparation even when its NAME agrees, since an '
          + 'equal identity is a hash claim and the per-slice facts are what every row is filed under',
      fn: async () => {
        /**
         Renumbers a ledger's anchor row, so BOTH lanes name a slice the
         preparation does not have there and agree with each other about it.

         @param records - one lane's rows

         @returns Ledger wearing the right name over rows the preparation
         contradicts

         @example
         ```ts
         const misfiled = renumbered({ records: repairLedger(), },);
         ```
         */
        function renumbered(
          { records, }: { readonly records: readonly SliceDeliveryRecord[]; },
        ) {
          return {
            preparationIdentity: catIdentity(),
            records: records.map(function renumber(record, position,): SliceDeliveryRecord {
              return (position === 1)
                ? {
                  ...record,
                  sliceIndex: 7,
                }
                : record;
            },),
          };
        }

        /**
         What rowsContradictTheName raised, read for its class as well as its wording.
         */
        const refusalOfRowsContradictTheName = caught(function rowsContradictTheName() {
          buildSettledTwoLaneArtifact({
            pageAssembly: NO_PAGE_ASSEMBLY,
            consistency: NO_CONSISTENCY,
            entryId: 'CatEntry1',
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            corpusSha: 'b'.repeat(40,),
            callConfig: {},
            durationMs: 1,
            prepared: catPreparation(),
            lanes: {
              ...catLanes(),
              repairDelivery: renumbered({ records: repairLedger(), },),
              translateDelivery: renumbered({ records: translateLedger(), },),
            },
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: { kind: 'not-run', },
          },);
        },);

        expect(refusalOfRowsContradictTheName,).toBeInstanceOf(ArtifactPreparationMismatchError,);
        expect((refusalOfRowsContradictTheName as Error).message,).toContain('names slice 7 at position 1',);
      },
    },),
    it({
      name:
        'REFUSES a lane result counting slices the preparation does not have, which the ledger checks '
          + 'cannot see: a driver result carrying one lane`s rows beside another lane`s result is '
          + 'structurally valid and describes two different runs',
      fn: async () => {
        /**
         What resultCountsAnotherRun raised, read for its class as well as its wording.
         */
        const refusalOfResultCountsAnotherRun = caught(function resultCountsAnotherRun() {
          buildSettledTwoLaneArtifact({
            pageAssembly: NO_PAGE_ASSEMBLY,
            consistency: NO_CONSISTENCY,
            entryId: 'CatEntry1',
            tip: 'a'.repeat(40,),
            pipelineDigest: DIGEST,
            corpusSha: 'b'.repeat(40,),
            callConfig: {},
            durationMs: 1,
            prepared: catPreparation(),
            lanes: {
              ...catLanes(),
              translate: {
                ...catLanes().translate,
                sliceCount: 9,
              },
            } as unknown as DocumentLanesResult,
            laneSelection: { kind: 'pending-human-decision', },
            consolidation: { kind: 'not-run', },
          },);
        },);

        expect(refusalOfResultCountsAnotherRun,).toBeInstanceOf(ArtifactPreparationMismatchError,);
        expect((refusalOfResultCountsAnotherRun as Error).message,).toContain('counts 9 slices',);
      },
    },),
    it({
      name:
        'writes only the fields version 2 names, dropping anything a live record has grown since: '
          + 'assignment into the frozen types accepts extra properties, JSON.stringify then serializes '
          + 'them, and the version 2 parser rejects keys the schema does not describe',
      fn: async () => {
        /**
         Lanes whose rows, outcomes and deliveries each carry a field no
         version 2 reader has heard of, standing in for what a live record
         looks like one commit after it grows.
         */
        const overgrown = {
          ...catLanes(),
          repairDelivery: {
            preparationIdentity: catIdentity(),
            records: repairLedger()
              .map(function grow(record,): SliceDeliveryRecord {
                return {
                  ...record,
                  purrLoudness: 11,
                  outcome: {
                    ...record.outcome,
                    whiskerCount: 24,
                  },
                  delivery: {
                    ...record.delivery,
                    napQuality: 'excellent',
                  },
                } as unknown as SliceDeliveryRecord;
              },),
          },
        } as unknown as DocumentLanesResult;

        /**
         Exactly what would be written to disk.
         */
        const written = JSON.stringify(buildSettledTwoLaneArtifact({
          pageAssembly: NO_PAGE_ASSEMBLY,
          consistency: NO_CONSISTENCY,
          entryId: 'CatEntry1',
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          callConfig: {},
          durationMs: 1,
          prepared: catPreparation(),
          lanes: overgrown,
          laneSelection: { kind: 'pending-human-decision', },
          consolidation: { kind: 'not-run', },
        },),);

        for (const invented of [
          'purrLoudness',
          'whiskerCount',
          'napQuality',
        ]) {
          expect(written.includes(invented,),).toBe(false,);
        }

        // And the rows are still there, so the `invented` checks are not
        // passing by writing nothing at all.
        expect(written.includes('replacement-shipped',),).toBe(true,);
      },
    },),
    it({
      name:
        'OMITS the pairing where the preparation carries none, rather than writing an empty list: no '
          + 'roster asked and a roster that agreed nothing are different facts, and an empty list is the second',
      fn: async () => {
        expect(Object.hasOwn(
          catArtifact().preparation,
          'blockPairing',
        ),).toBe(false,);
      },
    },),
    it({
      name:
        'WRITES the pairing the preparation was built on, section indices and all, so the decision no '
          + 'later stage can repair survives the slice cache being discarded at settlement',
      fn: async () => {
        /**
         Preparation carrying a pairing, as the roster shell returns one.
         */
        const paired = {
          ...catPreparation(),
          blockPairing: [{
            sectionIndex: 1,
            pairs: [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 0,
              },
            ],
          },],
        } as unknown as PreparedDocumentPair;
        expect(buildSettledTwoLaneArtifact({
          pageAssembly: NO_PAGE_ASSEMBLY,
          consistency: NO_CONSISTENCY,
          entryId: 'CatEntry1',
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          callConfig: {},
          durationMs: 1,
          prepared: paired,
          lanes: catLanes(),
          laneSelection: { kind: 'pending-human-decision', },
          consolidation: { kind: 'not-run', },
        },).preparation
          .blockPairing,).toEqual([{
            sectionIndex: 1,
            pairs: [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 0,
              },
            ],
          },],);
      },
    },),
    it({
      name:
        'ALWAYS WRITES which decider chose the sections, as deterministic when the preparation consumed '
          + 'no section pairing: unlike the block pairing, the deterministic path is the ordinary case, so '
          + 'an omitted field could not tell "the aligner decided" from "written before the field"',
      fn: async () => {
        expect(catArtifact().preparation
          .sectionPairing,).toEqual({ kind: 'deterministic', },);
      },
    },),
    it({
      name:
        'WRITES the supplied section pairing the preparation was built on, pairs and all, since the block '
          + 'pairing is keyed by aligned section and those keys only mean something under this alignment',
      fn: async () => {
        /**
         Preparation carrying a section pairing, as the roster shell returns
         one when its section round agreed.
         */
        const sectioned = {
          ...catPreparation(),
          sectionPairing: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 2,
              target: 1,
            },
          ],
        };
        expect(buildSettledTwoLaneArtifact({
          pageAssembly: NO_PAGE_ASSEMBLY,
          consistency: NO_CONSISTENCY,
          entryId: 'CatEntry1',
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          callConfig: {},
          durationMs: 1,
          prepared: sectioned,
          lanes: catLanes(),
          laneSelection: { kind: 'pending-human-decision', },
          consolidation: { kind: 'not-run', },
        },).preparation
          .sectionPairing,).toEqual({
            kind: 'supplied',
            pairs: [
              {
                source: 0,
                target: 0,
              },
              {
                source: 2,
                target: 1,
              },
            ],
          },);
      },
    },),
  ],
},);
