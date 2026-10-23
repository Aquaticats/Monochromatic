import {
  type ArtifactComparisonRow,
  type ArtifactDeliveryRow,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  requireExactKeys,
  requireKeyOf,
} from '../artifact-exact-guard.ts';
import type { ArtifactEvidenceRow, } from './artifact-two-lane-read-contract.ts';
import {
  INCUMBENT_KINDS,
  LANE_RELATIONS,
  parseDecisionComparison,
  parseSliceDelivery,
  parseSliceOutcome,
} from './artifact-two-lane-read-vocabulary.ts';
import type { ArtifactKeyVocabulary, } from '../artifact-key-vocabulary.ts';

//region Artifact version 2 row parsing
// Reading the three rows a version 2 artifact carries: a ledger row, a
// comparison row, and the evidence row a lane's raw result holds behind them.
//
// THE FIRST TWO ARE EXACT and the third is not, which is the schema-ownership
// rule this generation reads by: version 2 owns the ledger and the comparison,
// so a key it does not name there is a file it cannot read, while the raw
// result belongs to the live pipeline and a key version 2 does not name there
// is evidence a later lane added.

/**
 Reads one row of one lane's delivery ledger.

 @param value - row JSON

 @param path - dotted path for error message

 @param keys - field spellings this artifact's generation uses, so an older
 file is read by its own names rather than by today's

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Ledger row as version 2 describes it

 @throws {@link ArtifactParseError} when the row carries a key this version
 does not name, or any field is the wrong shape

 @example
 ```ts
 const row = parseDeliveryRow({ value, path: 'lanes.repair.delivery[0]', keys, },);
 ```
 */
export function parseDeliveryRow(
  {
    value,
    path,
    keys,
    benchFloorRecorded = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly benchFloorRecorded?: boolean;
  },
): ArtifactDeliveryRow {
  /**
   Row as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      keys.sliceIndex,
      'sourceText',
      'incumbentKind',
      'incumbentText',
      'outcome',
      'shippedText',
      'delivery',
    ],
    path,
  },);
  return {
    sliceIndex: requireCount({
      value: record[keys.sliceIndex],
      path: `${path}.${keys.sliceIndex}`,
    },),
    sourceText: requireString({
      value: record.sourceText,
      path: `${path}.sourceText`,
    },),
    incumbentKind: requireKeyOf({
      value: record.incumbentKind,
      record: INCUMBENT_KINDS,
      path: `${path}.incumbentKind`,
    },),
    incumbentText: requireString({
      value: record.incumbentText,
      path: `${path}.incumbentText`,
    },),
    outcome: parseSliceOutcome({
      value: record.outcome,
      unknownKeys: 'refuse',
      path: `${path}.outcome`,
      benchFloorRecorded,
    },),
    shippedText: requireString({
      value: record.shippedText,
      path: `${path}.shippedText`,
    },),
    delivery: parseSliceDelivery({
      value: record.delivery,
      path: `${path}.delivery`,
    },),
  };
}

/**
 Reads one row of the recorded comparison.

 @param value - row JSON

 @param path - dotted path for error message

 @param keys - field spellings this artifact's generation uses, so an older
 file is read by its own names rather than by today's

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Comparison row as version 2 describes it

 @throws {@link ArtifactParseError} when the row carries a key this version
 does not name, or any field is the wrong shape

 @example
 ```ts
 const row = parseComparisonRow({ value, path: 'comparison[0]', keys, },);
 ```
 */
export function parseComparisonRow(
  {
    value,
    path,
    keys,
    benchFloorRecorded = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly benchFloorRecorded?: boolean;
  },
): ArtifactComparisonRow {
  /**
   Row as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  /**
   Key this row spells its lane relation under.

   TWO SPELLINGS, AND EXACTLY ONE PER ROW. The field was `verdict` until
   2026-08-22, sharing a name with `laneSelection.slices[].verdict`, which
   answers a different question at a sibling path. Renaming it removes the
   collision for everything written afterwards; reading both keeps the
   artifacts settled under the old name readable, since artifacts outlive
   the pipelines that wrote them.

   A ROW CARRYING BOTH IS REFUSED rather than resolved, because
   `requireExactKeys` is handed only the spelling chosen here. Two spellings in one
   row means two pipelines wrote it, and picking one would hide that.
   */
  const relationKey = ('laneRelation' in record) ? 'laneRelation' : 'verdict';
  requireExactKeys({
    record,
    allowed: [
      keys.sliceIndex,
      'incumbentKind',
      'incumbentText',
      'repairText',
      'translateText',
      relationKey,
      'repairOutcome',
      'translateOutcome',
      'decisionComparison',
      'repairDelivery',
      'translateDelivery',
    ],
    path,
  },);
  return {
    sliceIndex: requireCount({
      value: record[keys.sliceIndex],
      path: `${path}.${keys.sliceIndex}`,
    },),
    incumbentKind: requireKeyOf({
      value: record.incumbentKind,
      record: INCUMBENT_KINDS,
      path: `${path}.incumbentKind`,
    },),
    incumbentText: requireString({
      value: record.incumbentText,
      path: `${path}.incumbentText`,
    },),
    repairText: requireString({
      value: record.repairText,
      path: `${path}.repairText`,
    },),
    translateText: requireString({
      value: record.translateText,
      path: `${path}.translateText`,
    },),
    laneRelation: requireKeyOf({
      value: record[relationKey],
      record: LANE_RELATIONS,
      path: `${path}.${relationKey}`,
    },),
    repairOutcome: parseSliceOutcome({
      value: record.repairOutcome,
      unknownKeys: 'refuse',
      path: `${path}.repairOutcome`,
      benchFloorRecorded,
    },),
    translateOutcome: parseSliceOutcome({
      value: record.translateOutcome,
      unknownKeys: 'refuse',
      path: `${path}.translateOutcome`,
      benchFloorRecorded,
    },),
    decisionComparison: parseDecisionComparison({
      value: record.decisionComparison,
      path: `${path}.decisionComparison`,
    },),
    repairDelivery: parseSliceDelivery({
      value: record.repairDelivery,
      path: `${path}.repairDelivery`,
    },),
    translateDelivery: parseSliceDelivery({
      value: record.translateDelivery,
      path: `${path}.translateDelivery`,
    },),
  };
}

/**
 Reads one slice out of a lane's RAW result, taking only what version 2 checks.

 TOLERANT BY DESIGN, and it is the only row parser here that is. A raw slice
 row is typed by the live pipeline: it has gained fields before and will
 again, and requiring today's shape of it would make every later addition a
 retroactive requirement on artifacts already written. What stays required is
 the part this reader compares against the ledger.

 @param value - raw slice row JSON

 @param path - dotted path for error message

 @param keys - field spellings this artifact's generation uses, so an older
 file is read by its own names rather than by today's

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Evidence row, with everything else in the raw row left unread

 @throws {@link ArtifactParseError} when a field version 2 checks is missing
 or the wrong shape, or the outcome names a member this version cannot read

 @example
 ```ts
 const row = parseEvidenceRow({ value, path: 'lanes.repair.result.sliceTexts[0]', keys, },);
 ```
 */
export function parseEvidenceRow(
  {
    value,
    path,
    keys,
    benchFloorRecorded = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly benchFloorRecorded?: boolean;
  },
): ArtifactEvidenceRow {
  /**
   Row as a record, whose other fields stay where they are.
   */
  const record = requireRecord({
    value,
    path,
  },);
  return {
    sliceIndex: requireCount({
      value: record[keys.sliceIndex],
      path: `${path}.${keys.sliceIndex}`,
    },),
    incumbentKind: requireKeyOf({
      value: record.incumbentKind,
      record: INCUMBENT_KINDS,
      path: `${path}.incumbentKind`,
    },),
    incumbentText: requireString({
      value: record.incumbentText,
      path: `${path}.incumbentText`,
    },),
    outcome: parseSliceOutcome({
      value: record.outcome,
      unknownKeys: 'tolerate',
      path: `${path}.outcome`,
      benchFloorRecorded,
    },),
  };
}

//endregion Artifact version 2 row parsing
