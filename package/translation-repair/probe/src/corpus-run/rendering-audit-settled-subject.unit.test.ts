/**
 Tests for which lane the settled audit reads its subjects from.

 WHY THIS FILE EXISTS. The audit measures what a READER of the published page
 would meet, and a settled artifact carries two lanes that both delivered
 text. On 2026-08-25, reading the repair lane's ledger instead of the
 translate lane's failed no test in this package, so nothing said which of two
 plausible ledgers the instrument is pointed at. An audit run against the
 wrong one would report a denominator over slices no page ever carried.

 THE TWO LANES DIFFER IN EVERY FIELD THE SUBJECT COPIES, deliberately: the
 count of decided rows, the accepted wording, and the delivery kind that
 decides `auditsArchiveText`. One assertion each way would have passed on
 either lane.

 Fixtures are cast, since this reads a handful of fields off a whole parsed
 artifact, following `would-ship-text.unit.test.ts` beside it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { RenderingAuditInvariantError, } from '@monochromatic-dev/translation-repair-rendering-audit';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { ParsedTwoLaneArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import {
  identityOf,
  type SettledIdentity,
  subjectsOf,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Original passage at the slice both lanes delivered.
 */
const SOURCE_NAP = '小猫在窗台上睡到中午。';

/**
 Original passage at the slice only the repair lane decided.
 */
const SOURCE_FEATHER = '她的哥哥给她带来一根羽毛。';

/**
 Archive's own English at the first slice.
 */
const ARCHIVE_NAP = 'The cat sleeps on the window ledge.';

/**
 Translate lane's wording for it.
 */
const TRANSLATE_NAP = 'Mittens naps on the windowsill.';

/**
 Repair lane's wording for the same slice.
 */
const REPAIR_NAP = 'Mittens is asleep on the sill.';

/**
 Whole original page both slices were cut from, which every subject carries
 unchanged so the audit can read what the page cites.
 */
const PAGE_NAP = `${SOURCE_NAP}\n\n${SOURCE_FEATHER}\n`;

/**
 Names this run licensed, which every subject carries unchanged.
 */
const IDENTITY: SettledIdentity = {
  kind: 'declared',
  context: '- name: Mittens',
};

/**
 Builds one comparison row, which is what the would-ship reader walks.

 @param sliceIndex - stamped index of this slice

 @param translateText - what the translate lane wrote here

 @param repairText - what the repair lane wrote here

 @returns Row as the parsed artifact carries it

 @example
 ```ts
 const row = comparisonRow({ sliceIndex: 0, translateText, repairText, },);
 ```
 */
function comparisonRow(
  {
    sliceIndex,
    translateText,
    repairText,
  }: {
    readonly sliceIndex: number;
    readonly translateText: string;
    readonly repairText: string;
  },
): Record<string, unknown> {
  return {
    sliceIndex,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    repairText,
    translateText,
    laneRelation: 'both-differ',
    repairOutcome: {
      kind: 'decided',
      acceptedText: repairText,
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
  };
}

/**
 Builds one delivery row of one lane's ledger.

 @param sliceIndex - stamped index of this slice

 @param sourceText - original passage at it

 @param outcome - what the lane decided, or a state that is not a decision

 @param delivery - what the lane's document ended up carrying

 @returns Row as the parsed artifact carries it

 @example
 ```ts
 const row = deliveryRow({ sliceIndex: 0, sourceText, outcome, delivery, },);
 ```
 */
function deliveryRow(
  {
    sliceIndex,
    sourceText,
    outcome,
    delivery,
  }: {
    readonly sliceIndex: number;
    readonly sourceText: string;
    readonly outcome: Record<string, unknown>;
    readonly delivery: Record<string, unknown>;
  },
): Record<string, unknown> {
  return {
    sliceIndex,
    sourceText,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    outcome,
    shippedText: ARCHIVE_NAP,
    delivery,
  };
}

/**
 Artifact whose two lanes disagree about count, wording and delivery kind.

 The translate lane decided ONE slice and left the second unreached; the
 repair lane decided BOTH, retaining the archive at the first. So a reader
 pointed at the wrong ledger differs in every field a subject copies.
 */
const ARTIFACT = {
  id: 'mittens-window',
  pipelineDigest: 'digest-aa11',
  corpusSha: 'sha-bb22',
  comparison: [
    comparisonRow({
      sliceIndex: 0,
      translateText: TRANSLATE_NAP,
      repairText: REPAIR_NAP,
    },),
    comparisonRow({
      sliceIndex: 1,
      translateText: TRANSLATE_NAP,
      repairText: REPAIR_NAP,
    },),
  ],
  consolidation: { kind: 'not-run', },
  laneSelection: { kind: 'pending-human-decision', },
  lanes: {
    translate: {
      delivery: [
        deliveryRow({
          sliceIndex: 0,
          sourceText: SOURCE_NAP,
          outcome: {
            kind: 'decided',
            acceptedText: TRANSLATE_NAP,
          },
          delivery: { kind: 'replacement-shipped', },
        },),
        deliveryRow({
          sliceIndex: 1,
          sourceText: SOURCE_FEATHER,
          outcome: { kind: 'not-evaluated', },
          delivery: { kind: 'gap-remains', },
        },),
      ],
    },
    repair: {
      delivery: [
        deliveryRow({
          sliceIndex: 0,
          sourceText: SOURCE_NAP,
          outcome: {
            kind: 'decided',
            acceptedText: REPAIR_NAP,
          },
          delivery: { kind: 'incumbent-retained', },
        },),
        deliveryRow({
          sliceIndex: 1,
          sourceText: SOURCE_FEATHER,
          outcome: {
            kind: 'decided',
            acceptedText: REPAIR_NAP,
          },
          delivery: { kind: 'replacement-shipped', },
        },),
      ],
    },
  },
} as unknown as ParsedTwoLaneArtifact;

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: subjectsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the translate lane ledger and no other, since a settled artifact carries two that '
            + 'both delivered text and an audit pointed at the wrong one reports a denominator over slices '
            + 'no reader ever met',
          fn: async () => {
            const subjects = subjectsOf({
              artifact: ARTIFACT,
              runSet: 'run-cc33',
              identity: IDENTITY,
              pageSourceText: PAGE_NAP,
            },);

            expect(subjects.length,).toBe(1,);

            /**
             Only slice the translate lane decided.
             */
            const subject = nonNullishOrThrow(subjects[0],);

            expect(subject.sliceIndex,).toBe(0,);
            expect(subject.candidateText,).toBe(TRANSLATE_NAP,);
            expect(subject.sourceText,).toBe(SOURCE_NAP,);
            expect(subject.deliveryKind,).toBe('replacement-shipped',);
            expect(subject.auditsArchiveText,).toBe(false,);
          },
        },),
        it({
          name: 'CARRIES the archive provenance every join needs, naming the run set, the entry, the digest '
            + 'that produced the decision and the commit it was read at, since two runs of one entry are '
            + 'separated by nothing else',
          fn: async () => {
            const subjects = subjectsOf({
              artifact: ARTIFACT,
              runSet: 'run-cc33',
              identity: IDENTITY,
              pageSourceText: PAGE_NAP,
            },);

            /**
             Only slice the translate lane decided.
             */
            const subject = nonNullishOrThrow(subjects[0],);

            expect(subject.runSet,).toBe('run-cc33',);
            expect(subject.entryId,).toBe('mittens-window',);
            expect(subject.artifactDigest,).toBe('digest-aa11',);
            expect(subject.corpusSha,).toBe('sha-bb22',);
            expect(subject.identity,).toEqual(IDENTITY,);
            expect(subject.pageSourceText,).toBe(PAGE_NAP,);
          },
        },),

        it({
          name: 'THROWS the invariant naming a delivered slice no comparison row describes, rather than '
            + 'quietly shrinking the audited population',
          fn: async () => {
            /**
             Artifact whose translate lane delivered a slice the comparison
             rows never named.
             */
            const inconsistent = {
              ...ARTIFACT,
              lanes: {
                ...ARTIFACT.lanes,
                translate: {
                  delivery: [
                    ...ARTIFACT.lanes.translate.delivery,
                    deliveryRow({
                      sliceIndex: 7,
                      sourceText: SOURCE_NAP,
                      outcome: {
                        kind: 'decided',
                        acceptedText: TRANSLATE_NAP,
                      },
                      delivery: { kind: 'replacement-shipped', },
                    },),
                  ],
                },
              },
            } as unknown as ParsedTwoLaneArtifact;

            /**
             What reading its subjects raised.
             */
            const refusal = caught(function readSubjects(): unknown {
              return subjectsOf({
                artifact: inconsistent,
                runSet: 'run/one',
                identity: IDENTITY,
                pageSourceText: PAGE_NAP,
              },);
            },);
            expect(refusal,).toBeInstanceOf(RenderingAuditInvariantError,);
            expect(String(refusal,),).toBe(
              'RenderingAuditInvariantError: rendering audit invariant broken: slice 7 was delivered by the '
                + 'translate lane and named by no comparison row',
            );
          },
        },),
      ],
    },),

    describe({
      name: identityOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS no identity where the preparation carries no identity block, and none where the block '
            + 'it carries is empty, and READS the block as declared where it holds a line',
          fn: async () => {
            /**
             A pair declaring nothing: no front matter, no heading, no link,
             no signature, no pronoun.
             */
            const pair = {
              sourceText: '猫睡了。\n',
              targetText: 'The cat sleeps.\n',
            };
            expect([
              prepareDocumentPair(pair,),
              // One empty line from a caller makes the block the empty
              // string, which the sheets render as no block at all.
              prepareDocumentPair({
                ...pair,
                contextLines: ['',],
              },),
              prepareDocumentPair({
                ...pair,
                contextLines: ['- nickname: Mittens',],
              },),
            ].map(function identityRead(prepared,): SettledIdentity {
              return identityOf({ prepared, },);
            },),).toEqual([
              { kind: 'none', },
              { kind: 'none', },
              {
                kind: 'declared',
                context: '- nickname: Mittens',
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
