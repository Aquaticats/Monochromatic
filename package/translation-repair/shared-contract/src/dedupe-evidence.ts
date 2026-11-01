import { isDeepStrictEqual, } from 'node:util';
import type {
  AdjudicatedIssue,
  VoteTally,
} from './adjudicate-model.ts';
import type { ClaimPanelReading, } from './panel-reading.ts';

//region Duplicate decision evidence
// Known evidence follows every retained claim. Missing legacy readings remain
// unknown; contradictory records for the same claim must not overwrite each other.

/**
 Duplicate claim identity carrying incompatible decision evidence.

 @example
 ```ts
 throw new IssueEvidenceConflictError({ claimId, kind: 'tally' });
 ```
 */
export class IssueEvidenceConflictError extends Error {
  /**
   Stable diagnostic identity for callers and lifecycle logs.
   */
  override readonly name = 'IssueEvidenceConflictError';

  /**
   Reports only the claim identifier and evidence kind, never document or ballot content.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the claim whose evidence cannot be merged without loss.

   @param claimId - shared identity with conflicting records

   @param kind - decision evidence that disagrees
   */
  constructor({
    claimId,
    kind,
  }: {
    readonly claimId: string;
    readonly kind: 'tally' | 'reading'
  },) {
    super(`cannot deduplicate claim ${claimId}: conflicting ${kind} evidence`,);
  }
}

/**
 Unions known evidence in retained claim order, without inventing missing data.

 @param claimIds - ordered retained identities

 @param first - survivor's known evidence

 @param second - duplicate's known evidence

 @param kind - diagnostic label for a conflicting record

 @returns Exact union of known retained evidence

 @throws {@link IssueEvidenceConflictError} when repeated known records disagree

 @example
 ```ts
 const tallies = mergeClaimEvidence({ claimIds, first, second, kind: 'tally' });
 ```
 */
function mergeClaimEvidence<const EvidenceT extends VoteTally | ClaimPanelReading,>(
  {
    claimIds,
    first,
    second,
    kind,
  }: {
    readonly claimIds: readonly string[];
    readonly first: Readonly<Record<string, EvidenceT>>;
    readonly second: Readonly<Record<string, EvidenceT>>;
    readonly kind: 'tally' | 'reading';
  },
): Readonly<Record<string, EvidenceT>> {
  return Object.fromEntries(claimIds.flatMap(function evidenceFor(claimId,): readonly (readonly [
    string,
    EvidenceT
  ])[] {
    /**
     Evidence already retained for this identity, if known.
     */
    const retained = first[claimId];
    /**
     Evidence arriving with the duplicate, if known.
     */
    const incoming = second[claimId];
    if ((retained !== undefined) && (incoming !== undefined)
      && (!isDeepStrictEqual(
        retained,
        incoming,
      )))
      throw new IssueEvidenceConflictError({
        claimId,
        kind,
      },);
    /**
     Missing records are not zero-vote records.
     */
    const known = retained ?? incoming;
    return known === undefined ? [] : [[
      claimId,
      known,
    ],];
  },),);
}

/**
 Preserves the existing emission representative while joining member evidence.
 Identity and severity retain the survivor policy; this operation does not
 re-adjudicate or count another panel's opinion twice.

 @param survivor - first accepted issue for the emission key

 @param incoming - accepted duplicate carrying additional members

 @param claims - deduplicated members in stable emission order

 @returns Survivor with every retained member's known evidence

 @throws {@link IssueEvidenceConflictError} when repeated known records disagree

 @example
 ```ts
 const merged = mergeDuplicateEvidence({ survivor, incoming, claims });
 ```
 */
export function mergeDuplicateEvidence(
  {
    survivor,
    incoming,
    claims,
  }: {
    readonly survivor: AdjudicatedIssue;
    readonly incoming: AdjudicatedIssue;
    readonly claims: AdjudicatedIssue['claims'];
  },
): AdjudicatedIssue {
  /**
   Evidence maps follow the same ordering as the final member list.
   */
  const claimIds = claims.map(function identity(member,): string {
    return member.claimId;
  },);
  return {
    ...survivor,
    claims,
    tallies: mergeClaimEvidence({
      claimIds,
      first: survivor.tallies,
      second: incoming.tallies,
      kind: 'tally',
    },),
    ...(((survivor.readings === undefined) && (incoming.readings === undefined)) ? {} : {
      readings: mergeClaimEvidence({
        claimIds,
        first: survivor.readings ?? {},
        second: incoming.readings ?? {},
        kind: 'reading',
      },),
    }),
  };
}

//endregion Duplicate decision evidence
