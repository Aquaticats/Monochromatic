import {
  ArtifactParseError,
  requireArray,
  requireBoolean,
  requireRecord,
  requireString,
} from './artifact-guard.ts';
import { SHIPPED_DISPOSITION, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type {
  GradableRepair,
  GradableRepairRegion,
} from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Artifact repair reading
// Reads one issue record's repair provenance back out of a run artifact.
//
// The one tolerance here is deliberate and narrow: an artifact written before
// repair recording existed carries no `repairDisposition`, and that absence
// returns `undefined` rather than throwing. Round two's thirty-one artifacts
// are exactly that, and they are the calibration set, so refusing to read them
// would trade a measurement for a schema opinion.
//
// The tolerance stops at absence. A record that DOES claim a disposition is
// parsed strictly, because a half-written repair is a malformed measurement and
// reading it leniently is how a repair denominator quietly goes wrong. Absence
// and emptiness stay distinct all the way to the sheet: no disposition means
// repair quality is unknowable for this item, while a disposition of
// `no-region` means the run measured that no targeted repair exists.

/**
 Field naming what became of an issue's repair.
 */
const DISPOSITION_FIELD = 'repairDisposition';

/**
 Every field a recorded repair contributes to an issue record.

 A record is treated as predating repair recording only when it carries NONE
 of them. Keying that judgement on the disposition alone would read a record
 carrying regions but no disposition as a legacy record, which is the one way
 a genuinely malformed repair could slip past a parser whose whole doctrine is
 that malformed measurements are surfaced rather than skipped.
 */
const REPAIR_FIELDS: readonly string[] = [
  DISPOSITION_FIELD,
  'repairRegions',
  'refined',
  'finalSliceText',
];

/**
 Parses one replaced region.

 @param value - region JSON

 @param path - dotted path for error message

 @returns Region as a grading sheet reads it

 @throws {@link ArtifactParseError} when the region is malformed

 @example
 ```ts
 const region = parseRepairRegion({ value, path: 'Kitten issues[0].repairRegions[0]', },);
 ```
 */
function parseRepairRegion(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): GradableRepairRegion {
  /**
   Region as a record.
   */
  const region = requireRecord({
    value,
    path,
  },);

  return {
    issueIds: requireArray({
      value: region.issueIds,
      path: `${path}.issueIds`,
    },)
      .map(function parseIdAt(
        idValue,
        idIndex,
      ) {
        return requireString({
          value: idValue,
          path: `${path}.issueIds[${String(idIndex,)}]`,
        },);
      },),
    before: requireString({
      value: region.before,
      path: `${path}.before`,
    },),
    editorAfter: requireString({
      value: region.editorAfter,
      path: `${path}.editorAfter`,
    },),
  };
}

/**
 What reading an issue record's repair provenance found.

 A named absence rather than a missing value, because "this run never recorded
 repairs" and "this run recorded that no repair exists" have to stay apart all
 the way to the denominator, and an absent field is the one thing a caller can
 forget to check.

 @example
 ```ts
 const reading: RecordRepairReading = { kind: 'unrecorded', };
 ```
 */
export type RecordRepairReading =
  | {
    /**
     Artifact predates repair recording, so repair quality is unknowable.
     */
    readonly kind: 'unrecorded';
  }
    | {
    /**
     Artifact carries repair provenance for this issue.
     */
      readonly kind: 'recorded';

    /**
     Provenance as a grading sheet reads it.
     */
      readonly repair: GradableRepair;
    };

/**
 Parses one issue record's repair provenance.

 @param record - issue record wrapper holding an issue and its repair

 @param path - dotted path for error message

 @returns Provenance, or a named absence for a pre-recording artifact

 @throws {@link ArtifactParseError} when a recorded repair is malformed

 @example
 ```ts
 const reading = parseRecordRepair({ record, path: 'Kitten issues[0]', },);
 ```
 */
export function parseRecordRepair(
  {
    record,
    path,
  }: {
    readonly record: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): RecordRepairReading {
  if (!REPAIR_FIELDS.some(function present(field,) {
    return field in record;
  },))
    return { kind: 'unrecorded', };

  /**
   Whether the naturalness lane rewrote this issue's slice afterwards.
   */
  const refined = requireBoolean({
    value: record.refined,
    path: `${path}.refined`,
  },);

  /**
   What became of this issue's repair, read before the final wording because
   it decides whether that wording has to be there.
   */
  const disposition = requireString({
    value: record[DISPOSITION_FIELD],
    path: `${path}.${DISPOSITION_FIELD}`,
  },);

  /**
   Whether a final wording has to be stated: refinement made the recorded
   replacement stale in a repair the document CARRIES, and that is the record
   a sheet grades. It cannot be graded against wording nothing states.

   Not required under any other disposition, because a slice the run took
   back, whether by non-translation dominance or by the assembly guard keeping
   a footnote relation whole, comes back as the archive wrote it and has no
   rewritten wording to state.
   */
  const finalWordingRequired = refined
    && (disposition === SHIPPED_DISPOSITION);

  /**
   Final slice text, read wherever it was written rather than only where it
   was required: a rewrite reaches the reader on a slice whose targeted repair
   did not, and that returned wording is worth showing even where nothing is
   graded. A present value is parsed strictly either way, so a field written
   as null or as a number is a malformed measurement wherever it appears.
   */
  const finalSliceText = (finalWordingRequired || ('finalSliceText' in record))
    ? requireString({
      value: record.finalSliceText,
      path: `${path}.finalSliceText`,
    },)
    : undefined;

  return {
    kind: 'recorded',
    repair: {
      disposition,
      regions: requireArray({
        value: record.repairRegions,
        path: `${path}.repairRegions`,
      },)
        .map(function parseRegionAt(
          regionValue,
          regionIndex,
        ) {
          return parseRepairRegion({
            value: regionValue,
            path: `${path}.repairRegions[${String(regionIndex,)}]`,
          },);
        },),
      refined,
      ...(finalSliceText === undefined ? {} : { finalSliceText, }),
    },
  };
}

//endregion Artifact repair reading
