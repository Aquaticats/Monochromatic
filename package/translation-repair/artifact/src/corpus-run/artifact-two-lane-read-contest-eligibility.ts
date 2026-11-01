import {
  requireExactKeys,
  requireOneOf,
} from '../artifact-exact-guard.ts';
import {
  type ArtifactComparisonRow,
  ArtifactParseError,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { splitFrontMatter, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  frontMatterContestEligibility,
  type LaneContestEligibility,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Contest eligibility reading
// Source-backed syntax admission is parsed separately from raw contest ballots
// so the main contest reader stays focused on ballot/verdict agreement.

/**
 Candidate statuses carried by source-backed eligibility.
 */
const ELIGIBILITY_CANDIDATES = [
  'archive',
  'repair',
  'translate',
] as const;

/**
 Reports whether comparison row carries explicit front matter slice.

 @param row - recomputed lane comparison row

 @returns Whether schema 7 must carry source-backed eligibility

 @example
 ```ts
 const required = contestEligibilityRequired({ row, });
 ```
 */
export function contestEligibilityRequired(
  { row, }: { readonly row: ArtifactComparisonRow; },
): boolean {
  /**
   Archive wording parsed under same front matter recognizer as preparation.
   */
  const parsed = splitFrontMatter({ text: row.incumbentText, });
  return parsed.frontMatter !== undefined;
}

/**
 Reads and re-derives source-backed candidate eligibility.

 @param value - recorded eligibility

 @param row - lane candidates eligibility governs

 @param path - dotted eligibility path

 @returns Proven deterministic eligibility

 @throws ArtifactParseError when shape or any stored status disagrees

 @example
 ```ts
 const eligibility = parseContestEligibility({ value, row, path, });
 ```
 */
export function parseContestEligibility(
  {
    value,
    row,
    path,
  }: {
    readonly value: unknown;
    readonly row: ArtifactComparisonRow;
    readonly path: string;
  },
): LaneContestEligibility {
  /**
   Recorded eligibility before fields are read.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'syntax',
      'sourceText',
      'archive',
      'repair',
      'translate',
    ],
    path,
  },);
  requireOneOf({
    value: record.syntax,
    allowed: ['front-matter',],
    path: `${path}.syntax`,
  },);
  /**
   Original metadata stored so deterministic statuses can be recomputed.
   */
  const sourceText = requireString({
    value: record.sourceText,
    path: `${path}.sourceText`,
  },);
  /**
   Eligibility recomputed from source, archive, and both lanes.
   */
  const derived = frontMatterContestEligibility({
    sourceText,
    incumbentText: row.incumbentText,
    repairText: row.repairText,
    translateText: row.translateText,
  },);
  for (const candidate of ELIGIBILITY_CANDIDATES) {
    /**
     Status artifact claims for this candidate.
     */
    const claimed = requireOneOf({
      value: record[candidate],
      allowed: [
        'eligible',
        'ineligible',
      ],
      path: `${path}.${candidate}`,
    },);
    if (claimed !== derived[candidate]) {
      throw new ArtifactParseError({
        path: `${path}.${candidate}`,
        reason: `${derived[candidate]}, which deterministic syntax guard derives, rather than ${claimed}`,
      },);
    }
  }
  return derived;
}

//endregion Contest eligibility reading
