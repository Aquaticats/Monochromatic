import {
  type BallotVerdict,
  isIssueSeverity,
  isPanelVoteState,
  ISSUE_SEVERITIES,
  PANEL_VOTE_STATES,
  type PanelBallot,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Adjudication wire format
// What panelists actually emit: integer claim references, closed-vocabulary
// votes, optional severity re-grades, and same-defect group opinions.
// Resolution maps integers back to ids through the prompt plan and fails
// closed per item: a bad verdict becomes a finding and an abstention, never
// an exception, because ballot irregularities are scorecard data.

/**
 One verdict as a panelist reports it.

 @example
 ```ts
 const wire: PanelVerdictWire = { claim: 1, reason: 'The quote already carries it.', vote: 'unsupported', };
 ```
 */
export type PanelVerdictWire = {
  /**
   One-based claim number from the prompt sheet.
   */
  readonly claim: number;

  /**
   What decides the claim, written BEFORE the vote (owner, 2026-09-27,
   "Reason before vote"). Required by the schema; optional here because a
   provider that does not hold replies to it can drop the field, and a
   missing reason is resolved as a finding beside a vote that still counts.
   */
  readonly reason?: string;

  /**
   Vote string; validated against the closed vocabulary at resolution.
   */
  readonly vote: string;

  /**
   Optional severity re-grade; validated at resolution.
   */
  readonly severity?: string;
};

/**
 One group opinion as a panelist reports it.

 @example
 ```ts
 const wire: PanelGroupWire = { group: 1, sameDefect: true, };
 ```
 */
export type PanelGroupWire = {
  /**
   One-based group number from the prompt sheet.
   */
  readonly group: number;

  /**
   Whether the group's claims describe one single defect.
   */
  readonly sameDefect: boolean;
};

/**
 Whole ballot on the wire.

 @example
 ```ts
 const wire: PanelBallotWire = { verdicts: [], groups: [], };
 ```
 */
export type PanelBallotWire = {
  /**
   Every verdict cast.
   */
  readonly verdicts: readonly PanelVerdictWire[];

  /**
   Group opinions; optional because single-claim sheets have none.
   */
  readonly groups?: readonly PanelGroupWire[];
};

/**
 Guards one wire verdict.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required verdict fields

 @example
 ```ts
 isPanelVerdictWire({ claim: 1, vote: 'supported', },);
 ```
 */
function isPanelVerdictWire(value: unknown,): value is PanelVerdictWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Claim reference as reported; integerness checked on the primitive copy.
   */
  const { claim, } = value;
  if ((typeof claim) !== 'number')
    return false;
  if ((claim % 1) !== 0)
    return false;
  if ((typeof value.vote) !== 'string')
    return false;
  if ((value.reason !== undefined) && ((typeof value.reason) !== 'string'))
    return false;
  return (value.severity === undefined) || ((typeof value.severity) === 'string');
}

/**
 Guards one wire group opinion.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required group fields

 @example
 ```ts
 isPanelGroupWire({ group: 1, sameDefect: false, },);
 ```
 */
function isPanelGroupWire(value: unknown,): value is PanelGroupWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Group reference as reported; integerness checked on the primitive copy.
   */
  const { group, } = value;
  if ((typeof group) !== 'number')
    return false;
  if ((group % 1) !== 0)
    return false;
  return (typeof value.sameDefect) === 'boolean';
}

/**
 Guards a whole ballot.

 @param value - parsed model JSON

 @returns Whether value is a wire ballot

 @example
 ```ts
 const outcome = await client.chatJson({ ..., validate: isPanelBallotWire, },);
 ```
 */
export function isPanelBallotWire(value: unknown,): value is PanelBallotWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.verdicts,))
    return false;
  if (!value.verdicts
    .every(function eachVerdict(verdict,) {
      return isPanelVerdictWire(verdict,);
    },))
    return false;
  if (value.groups === undefined)
    return true;
  if (!isJsonArray(value.groups,))
    return false;
  return value.groups
    .every(function eachGroup(group,) {
      return isPanelGroupWire(group,);
    },);
}

/**
 Guard for a ballot the gather may count as a heard voice: a wire ballot
 carrying at least one verdict with a known vote on a claim the packet showed.

 LEDGER L8: `isPanelBallotWire` accepts `{"verdicts": []}`, a ballot voting
 only on claim numbers the packet never showed, and one whose only vote is
 no vote at all ("minor"), so each counted as heard and closed a round whose
 tally then abstained on every claim (TianqiChen66616 slice 3 landed a claim
 in needs-human on two votes). Such a ballot carries no voice, so the gather
 reads it as unreadable and the recovery round re-asks the seat. A ballot
 usable on some claims is heard, and abstains on the rest at tally time.

 @param claimCount - claims the packet showed, numbered from one

 @returns Guard over parsed model JSON

 @example
 ```ts
 const gather = await gatherStageVoices({ ..., validate: usablePanelBallotFor({ claimCount: 3, },), },);
 ```
 */
export function usablePanelBallotFor(
  { claimCount, }: { readonly claimCount: number; },
): (value: unknown,) => value is PanelBallotWire {
  return function isUsablePanelBallot(value: unknown,): value is PanelBallotWire {
    if (!isPanelBallotWire(value,))
      return false;
    return value
      .verdicts
      .some(function votesOnShownClaim(verdict,): boolean {
        return (verdict.claim >= 1)
          && (verdict.claim <= claimCount)
          && isPanelVoteState(verdict.vote,);
      },);
  };
}

/**
 Structured-output constraint for panel calls;
 client-side validation through {@link isPanelBallotWire} stays regardless,
 because per-model schema strictness is unverified.
 */
export const ADJUDICATION_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'panel_ballot',
    schema: {
      type: 'object',
      required: ['verdicts',],
      additionalProperties: false,
      // DECLARED IN THE ORDER THE KEYS SORT (ledger B762), because Charm Hyper
      // returns a tool call's keys alphabetically and the reason must still
      // come before the vote there.
      properties: {
        groups: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'group',
              'sameDefect',
            ],
            additionalProperties: false,
            properties: {
              group: { type: 'integer', },
              sameDefect: { type: 'boolean', },
            },
          },
        },
        verdicts: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'claim',
              'reason',
              'vote',
            ],
            additionalProperties: false,
            // REASON DECLARED BEFORE VOTE, as the sheet's reply shape shows it
            // (owner, 2026-09-27, "Reason before vote"); severity sorts
            // between them and is optional, so it costs the reason nothing.
            properties: {
              claim: { type: 'integer', },
              reason: { type: 'string', },
              // THE CLOSED VOCABULARIES THE RESOLVER READS, as an enum (P1 of the second review of the Claude route; the
              // lead's ruling of 2026-10-15 lists labels as an enum): `resolvePanelBallot` drops a vote outside
              // `PANEL_VOTE_STATES` and a re-grade outside `ISSUE_SEVERITIES`.
              severity: {
                type: 'string',
                enum: [...ISSUE_SEVERITIES,],
              },
              vote: {
                type: 'string',
                enum: [...PANEL_VOTE_STATES,],
              },
            },
          },
        },
      },
    },
  },
};

/**
 Resolves one wire ballot into id-keyed verdicts through the prompt plan.
 Fails closed per item: out-of-range or duplicate references and unknown
 votes become findings, an invalid severity drops only the re-grade, a
 missing or blank reason is recorded while its vote still counts, and claims
 left without a verdict are recorded and abstain at tally time.

 @param wire - ballot as the panelist reported it

 @param claimIds - claim ids in prompt numbering order

 @param clusterIds - cluster ids in prompt numbering order

 @returns Resolved ballot with findings as data

 @example
 ```ts
 const ballot = resolvePanelBallot({ wire, claimIds, clusterIds, },);
 ```
 */
export function resolvePanelBallot(
  {
    wire,
    claimIds,
    clusterIds,
  }: {
    readonly wire: PanelBallotWire;
    readonly claimIds: readonly string[];
    readonly clusterIds: readonly string[];
  },
): PanelBallot {
  /**
   Findings accumulated across every wire item.
   */
  const findings: string[] = [];

  /**
   Resolved verdicts keyed by claim id; first occurrence wins. A map until
   handed back, as every record filled by a key is (ledger B77).
   */
  const verdicts = new Map<string, BallotVerdict>();
  for (const verdict of wire.verdicts) {
    /**
     Claim id referenced by this verdict's one-based number.
     */
    const claimId = claimIds[verdict.claim - 1];
    if ((verdict.claim < 1) || (claimId === undefined)) {
      findings.push(`verdict-index-out-of-range (${verdict.claim})`,);
      continue;
    }
    if (verdicts.has(claimId,)) {
      findings.push(`duplicate-verdict (${verdict.claim})`,);
      continue;
    }
    if (!isPanelVoteState(verdict.vote,)) {
      findings.push(`unknown-vote (${verdict.vote})`,);
      continue;
    }
    /**
     This verdict's reason, trimmed, or empty where none was given.
     */
    const reason = (verdict.reason ?? '').trim();
    /**
     Whether the reason shows a reader nothing: none, whitespace, or invisible
     characters `trim()` keeps (ledger B40).
     */
    const reasonMissing = rendersAsNothing({ text: reason, },);
    // A MISSING REASON IS AN AUDIT GAP, NOT A LOST VOICE: the vote still
    // counts, and the gap is recorded beside it.
    if (reasonMissing)
      findings.push(`missing-reason (${verdict.claim})`,);
    /**
     The reason as a spreadable field, absent where none was given.
     */
    const given = reasonMissing ? {} : { reason, };
    if ((verdict.severity !== undefined) && (!isIssueSeverity(verdict.severity,))) {
      findings.push(`unknown-regrade-severity (${verdict.severity})`,);
      verdicts.set(
        claimId,
        {
          vote: verdict.vote,
          ...given,
        },
      );
      continue;
    }
    verdicts.set(
      claimId,
      {
        vote: verdict.vote,
        ...(verdict.severity === undefined ? {} : { severity: verdict.severity, }),
        ...given,
      },
    );
  }
  for (const [index, claimId,] of claimIds.entries()) {
    if (!verdicts.has(claimId,))
      findings.push(`missing-verdict (${index + 1})`,);
  }

  /**
   Resolved group opinions keyed by cluster id; first occurrence wins. A map
   until handed back, like the verdicts.
   */
  const mergeOpinions = new Map<string, boolean>();
  for (const group of wire.groups ?? []) {
    /**
     Cluster id referenced by this opinion's one-based number.
     */
    const clusterId = clusterIds[group.group - 1];
    if ((group.group < 1) || (clusterId === undefined)) {
      findings.push(`group-index-out-of-range (${group.group})`,);
      continue;
    }
    if (mergeOpinions.has(clusterId,)) {
      findings.push(`duplicate-group-opinion (${group.group})`,);
      continue;
    }
    mergeOpinions.set(
      clusterId,
      group.sameDefect,
    );
  }

  return {
    verdicts: Object.fromEntries(verdicts,),
    mergeOpinions: Object.fromEntries(mergeOpinions,),
    findings,
  };
}

//endregion Adjudication wire format
