import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { requireKeyOf, } from './artifact-exact-guard.ts';
import type { ClaimAdmissibility, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type {
  ProbeClaimAttribution,
  TelemetryRegionTally,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Artifact probe tally
// Region-level parsing for the shadow-mode probe reader.
//
// Split out of `artifact-probe-read.ts` when that file crossed its line cap.
// The seam is the unit each half is about: this half turns one region's JSON
// into one tally, and the other half decides which records have a region worth
// reading at all.
//
// The split also fixes a reading order the single file could not have: the
// tally parser called its claim parser and its admissibility table before
// either was declared, which is legal and unreadable top-down.

/**
 Every admissibility the screen can record.

 KEYED BY `ClaimAdmissibility` rather than listed, so the compiler refuses a
 value the screen gains that this lacks: every artifact carrying it would
 otherwise be refused whole. The order is the order a refusal names them in.
 */
const ADMISSIBILITY_VALUES: Readonly<Record<ClaimAdmissibility, true>> = {
  corroborated: true,
  'removal-corroborated': true,
  contradicted: true,
  unanchored: true,
  'pre-existing': true,
};

/**
 Reads a claim's admissibility, refusing a value the screen cannot have
 written.

 Narrowing rather than asserting, because the majority rule counts only the
 two upheld values. An unrecognized string would be silently non-upholding, so
 a writer emitting a new verdict name would quietly zero the corroboration
 every region reports rather than announce that the schemas diverged.

 @param value - candidate admissibility from artifact JSON

 @param path - dotted path for error messages

 @returns Admissibility as the screen recorded it

 @throws {@link ArtifactParseError} when the value is not one the screen emits

 @example
 ```ts
 const admissibility = requireAdmissibility({ value, path, },);
 ```
 */
function requireAdmissibility(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ClaimAdmissibility {
  return requireKeyOf({
    value,
    record: ADMISSIBILITY_VALUES,
    path,
  },);
}

/**
 Parses a region's claim list down to who said what.

 Attribution only, and the return type says so. `evidence`, `omittedText` and
 `reason` carry UNLICENSED corpus text, this reader feeds a summary meant to
 be pasteable, and nothing downstream reads them. Returning a full claim with
 those fields blanked would be indistinguishable from a prober that quoted
 nothing.

 @param value - candidate claim array from artifact JSON

 @param path - dotted path of the owning tally, for error messages

 @returns Claims reduced to prober and admissibility

 @throws {@link ArtifactParseError} when a claim is malformed

 @example
 ```ts
 const claims = parseClaimAttributions({ value: tally.claims, path, },);
 ```
 */
function parseClaimAttributions(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly ProbeClaimAttribution[] {
  return requireArray({
    value,
    path: `${path}.claims`,
  },)
    .map(function toClaim(
      entry,
      index,
    ): ProbeClaimAttribution {
      /**
       Claim as a record.
       */
      const claim = requireRecord({
        value: entry,
        path: `${path}.claims[${String(index,)}]`,
      },);
      return {
        modelId: requireString({
          value: claim.modelId,
          path: `${path}.claims[${String(index,)}].modelId`,
        },),
        admissibility: requireAdmissibility({
          value: claim.admissibility,
          path: `${path}.claims[${String(index,)}].admissibility`,
        },),
      };
    },);
}

/**
 Counts a region tally declares, each the tally of one admissibility.
 */
const COUNTED_FIELDS = [
  'corroborated',
  'removalCorroborated',
  'contradicted',
  'unanchored',
  'preExisting',
] as const;

/**
 Admissibility each declared count is the tally of, keyed by the count's own
 name, which the type holds: a lookup by any other text does not compile, and
 every name has its entry (ledger B77).
 */
const ADMISSIBILITY_FIELDS: Readonly<Record<(typeof COUNTED_FIELDS)[number], ClaimAdmissibility>> = {
  corroborated: 'corroborated',
  removalCorroborated: 'removal-corroborated',
  contradicted: 'contradicted',
  unanchored: 'unanchored',
  preExisting: 'pre-existing',
};

/**
 Parses one region tally.

 @param value - candidate tally from artifact JSON

 @param path - dotted path for error messages

 @returns Tally as the summary reads it

 @throws {@link ArtifactParseError} when any count or id is malformed, or when
 the declared counts disagree with the claim list they were derived from

 @example
 ```ts
 const tally = parseRegionTally({ value, path: 'Kitten.issues[0]...regions[0]', },);
 ```
 */
export function parseRegionTally(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): TelemetryRegionTally {
  /**
   Tally as a record.
   */
  const tally = requireRecord({
    value,
    path,
  },);

  /**
   Reads one named count off the tally.

   @param field - count to read

   @returns Count value

   @example
   ```ts
   countAt('corroborated',);
   ```
   */
  function countAt(field: string,): number {
    return requireCount({
      value: tally[field],
      path: `${path}.${field}`,
    },);
  }

  // The majority rule counts distinct probers, so the verdict cannot be
  // computed without modelId and admissibility. Reading the counts alone
  // silently judged every region as uncorroborated.
  /**
   Screened claims of this region, attribution only.
   */
  const claims = parseClaimAttributions({
    value: tally.claims,
    path,
  },);

  /**
   Counts as the artifact declares them.
   */
  const declared = {
    corroborated: countAt('corroborated',),
    removalCorroborated: countAt('removalCorroborated',),
    contradicted: countAt('contradicted',),
    unanchored: countAt('unanchored',),
    // Read tolerantly, unlike every other count. This outcome did not exist
    // when the settled artifacts were written, and `requireCount` refuses an
    // absent field, so demanding it would make every earlier artifact
    // unreadable by the same reader that has to keep scoring them. Zero is the
    // honest default rather than a convenience: those runs rendered the
    // accepted issues into the prompt, so no claim could be dismissed as
    // restating one.
    preExisting: tally.preExisting === undefined ? 0 : countAt('preExisting',),
  };

  // The screen DERIVES each count from the claim list, so the two are one fact
  // written twice and can only disagree in a malformed artifact. They are not
  // interchangeable downstream: the CLAIMS report sums the counts while the
  // majority rule reads the claims, so a disagreement makes a region report one
  // corroboration and flag nothing, or flag a majority while reporting none.
  // Both look like ordinary output.
  for (const field of COUNTED_FIELDS) {
    /**
     The count the artifact declares for this admissibility.
     */
    const count = declared[field];

    /**
     Claims actually carrying this admissibility.
     */
    const observed = claims
      .filter(function matches(claim,) {
        return claim.admissibility === ADMISSIBILITY_FIELDS[field];
      },)
      .length;
    if (observed !== count)
      throw new ArtifactParseError({
        path: `${path}.${field}`,
        reason: `${String(observed,)} to match its claim list, not ${
          String(count,)
        }`,
      },);
  }

  return {
    envelopeId: requireString({
      value: tally.envelopeId,
      path: `${path}.envelopeId`,
    },),
    issueIds: requireArray({
      value: tally.issueIds,
      path: `${path}.issueIds`,
    },)
      .map(function toId(
        entry,
        index,
      ) {
        return requireString({
          value: entry,
          path: `${path}.issueIds[${String(index,)}]`,
        },);
      },),
    ...declared,
    noneFound: countAt('noneFound',),
    uncertain: countAt('uncertain',),
    claims,
  };
}

//endregion Artifact probe tally
