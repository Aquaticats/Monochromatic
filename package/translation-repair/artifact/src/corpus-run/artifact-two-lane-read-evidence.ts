import {
  requireArray,
  requireCount,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  requireKeyOf,
  requireOpenRecord,
} from '../artifact-exact-guard.ts';
import type {
  ArtifactEvidenceRow,
  ArtifactRepairEvidence,
  ArtifactTranslateEvidence,
} from './artifact-two-lane-read-contract.ts';
import { parseEvidenceRow, } from './artifact-two-lane-read-rows.ts';
import type { ArtifactKeyVocabulary, } from '../artifact-key-vocabulary.ts';

//region Artifact version 2 evidence parsing
// Taking what version 2 requires OUT of the two raw lane results, which it
// otherwise leaves alone.
//
// BOTH LANES IN ONE FILE, rather than one file each as the seam sketch had it:
// the two parsers differ by three fields, they share `requireIndexList`,
// and splitting them would put the shared part somewhere neither lane
// owns. Either grows a file of its own the day it needs more than this.
//
// TOLERANT, and deliberately so. These records are typed by the live pipeline,
// they have gained fields before and will again, and requiring today's shape
// would make every later addition a retroactive requirement on artifacts
// already written. What each parser reads is exactly what a relation later
// checks: the ledger against the evidence, the counts against their own lists,
// and the blocked status against what the deliveries say.

/**
 How a repair run may end, as version 2 froze it.

 KEYED BY THE FROZEN `ArtifactRepairEvidence` field rather than listed, so the
 compiler refuses a status the contract gains that this reader lacks.
 */
const REPAIR_STATUSES: Readonly<Record<ArtifactRepairEvidence['status'], true>> = {
  repaired: true,
  unchanged: true,
  'blocked-non-translation': true,
};

/**
 Whether a translate run produced a whole translation, as version 2 froze it,
 keyed by the frozen `ArtifactTranslateEvidence` field for the reason
 `REPAIR_STATUSES` gives.
 */
const TRANSLATE_STATUSES: Readonly<Record<ArtifactTranslateEvidence['status'], true>> = {
  complete: true,
  unfilled: true,
};

/**
 Reads a list of slice indices.

 @param value - list JSON

 @param path - dotted path for error message

 @returns Indices in the order the file lists them, which is the order the
 relations read them in

 @throws {@link ArtifactParseError} when the value is not an array, or any
 element is not an index any preparation could have produced

 @example
 ```ts
 const shipped = requireIndexList({ value: record.changedSliceIndices, path, },);
 ```
 */
function requireIndexList(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly number[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readIndex(
      held,
      position,
    ): number {
      return requireCount({
        value: held,
        path: `${path}[${String(position,)}]`,
      },);
    },);
}

/**
 Reads a lane's slice texts, one evidence row each. Both lanes carry them in
 one shape, so both parsers read them here (audit area six, ledger B18: each
 kept its own copy).

 @param value - list JSON

 @param path - dotted path for error message

 @param keys - spelling the artifact's own generation gave the renamed keys

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Rows in the order the file lists them

 @throws {@link ArtifactParseError} when the value is not an array, or a row
 is not the shape the version requires

 @example
 ```ts
 const rows = requireEvidenceRows({ value: record.sliceTexts, path: `${path}.sliceTexts`, keys, },);
 ```
 */
function requireEvidenceRows(
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
): readonly ArtifactEvidenceRow[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readRow(
      row,
      position,
    ): ArtifactEvidenceRow {
      return parseEvidenceRow({
        value: row,
        path: `${path}[${String(position,)}]`,
        keys,
        benchFloorRecorded,
      },);
    },);
}

/**
 Reads what this shape requires of the repair lane's raw result.

 @param value - raw result JSON

 @param path - dotted path for error message

 @param keys - spelling the artifact's own generation gave the renamed keys,
 so a version 2 file is read under `chunk` and a version 3 file under `slice`
 without either being tried against the other

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Evidence core, with the rest of the record left where it is

 @throws {@link ArtifactParseError} when a field version 2 checks is missing,
 the wrong shape, or names a status this version does not describe

 @example
 ```ts
 const evidence = parseRepairEvidence({ value: raw, path: 'lanes.repair.result', keys, },);
 ```
 */
export function parseRepairEvidence(
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
): ArtifactRepairEvidence {
  /**
   Raw result as a record, whose other fields stay unread.
   */
  const record = requireOpenRecord({
    value,
    path,
  },);
  return {
    status: requireKeyOf({
      value: record.status,
      record: REPAIR_STATUSES,
      path: `${path}.status`,
    },),
    sliceCount: requireCount({
      value: record.sliceCount,
      path: `${path}.sliceCount`,
    },),
    changedSliceIndices: requireIndexList({
      value: record[keys.changedSliceIndices],
      path: `${path}.${keys.changedSliceIndices}`,
    },),
    withdrawnSliceIndices: requireIndexList({
      value: record[keys.withdrawnSliceIndices],
      path: `${path}.${keys.withdrawnSliceIndices}`,
    },),
    sliceTexts: requireEvidenceRows({
      value: record.sliceTexts,
      path: `${path}.sliceTexts`,
      keys,
      benchFloorRecorded,
    },),
  };
}

/**
 Reads what this shape requires of the translate lane's raw result.

 @param value - raw result JSON

 @param path - dotted path for error message

 @param keys - spelling the artifact's own generation gave the renamed keys,
 so a version 2 file is read under `chunk` and a version 3 file under `slice`
 without either being tried against the other

 @param benchFloorRecorded - whether the generation records the mark of an unfilled outcome the select bench never
 bought a candidate for

 @returns Evidence core, with the rest of the record left where it is

 @throws {@link ArtifactParseError} when a field version 2 checks is missing,
 the wrong shape, or names a status this version does not describe

 @example
 ```ts
 const evidence = parseTranslateEvidence({ value: raw, path: 'lanes.translate.result', keys, },);
 ```
 */
export function parseTranslateEvidence(
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
): ArtifactTranslateEvidence {
  /**
   Raw result as a record, whose other fields stay unread.
   */
  const record = requireOpenRecord({
    value,
    path,
  },);
  return {
    status: requireKeyOf({
      value: record.status,
      record: TRANSLATE_STATUSES,
      path: `${path}.status`,
    },),
    sliceCount: requireCount({
      value: record.sliceCount,
      path: `${path}.sliceCount`,
    },),
    changedSliceCount: requireCount({
      value: record.changedSliceCount,
      path: `${path}.changedSliceCount`,
    },),
    withdrawnSliceCount: requireCount({
      value: record.withdrawnSliceCount,
      path: `${path}.withdrawnSliceCount`,
    },),
    changedSliceIndices: requireIndexList({
      value: record[keys.changedSliceIndices],
      path: `${path}.${keys.changedSliceIndices}`,
    },),
    withdrawnSliceIndices: requireIndexList({
      value: record[keys.withdrawnSliceIndices],
      path: `${path}.${keys.withdrawnSliceIndices}`,
    },),
    sliceTexts: requireEvidenceRows({
      value: record.sliceTexts,
      path: `${path}.sliceTexts`,
      keys,
      benchFloorRecorded,
    },),
  };
}

//endregion Artifact version 2 evidence parsing
