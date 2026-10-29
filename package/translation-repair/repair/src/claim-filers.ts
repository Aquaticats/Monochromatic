import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  AdjudicatedIssue,
  IssueClaim,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { panelClause, } from './claim-panel-voters.ts';
import type { ClaimAttribution, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { flattenSpace, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Claim filers
// Who filed each claim, as the log and the artifact name it. The attribution
// record (`critic-attribution.ts`) keeps this OUTSIDE the claim so the panel
// stays provenance-blind; that is right for adjudication and wrong for the
// reader, who on 2026-09-24 had to join `sliceCritics[].claimAttributions` by
// claim id to learn which critic had inverted the original's cause and effect
// (owner: "we're not logging enough then. Refine how we log."). The functions
// here run AFTER the panel has spoken, so nothing the panel sees changes.

/**
 Filers per claim id, sorted by model id.

 @example
 ```ts
 const filers: ClaimFilers = { 'issue/abc': ['hf:zai-org/GLM-5.3-Flash',], };
 ```
 */
export type ClaimFilers = Readonly<Record<string, readonly RosterModelId[]>>;

/**
 Names nobody on a claim the record does not attribute: an issue rebuilt from
 an older artifact, or a claim the reference screen filed.
 */
const NOBODY_ON_RECORD = 'nobody on record';

/**
 Folds the attribution record into filers per claim id.

 @param attributions - critic phase's attribution, already sorted

 @returns Filers keyed by claim id, in the attribution's order

 @example
 ```ts
 const filers = claimFilersOf({ attributions: critic.claimAttributions, },);
 ```
 */
export function claimFilersOf(
  {
    attributions,
  }: {
    readonly attributions: readonly ClaimAttribution[];
  },
): ClaimFilers {
  return Object.fromEntries(attributions.map(function toEntry(attribution,): readonly [
    string,
    readonly RosterModelId[]
  ] {
    return [
      attribution.claimId,
      attribution.proposers
        .map(function toModelId(proposer,): RosterModelId {
          return proposer.modelId;
        },),
    ];
  },),);
}

/**
 Names the filers of one claim for a log line.

 @param claimId - claim identity

 @param filers - filers per claim id

 @returns Comma-joined model ids, or the nobody wording

 @example
 ```ts
 const who = filersClause({ claimId, filers, },);
 ```
 */
function filersClause(
  {
    claimId,
    filers,
  }: {
    readonly claimId: string;
    readonly filers: ClaimFilers;
  },
): string {
  /**
   Filers on record for this claim.
   */
  const named = filers[claimId];
  if ((named === undefined) || (named.length === 0))
    return NOBODY_ON_RECORD;
  return named.join(', ',);
}

/**
 One claim's body for a log line: category, severity, summary.

 @param claim - claim as validated

 @returns Body clause

 @example
 ```ts
 const body = claimBody({ claim, },);
 ```
 */
function claimBody(
  {
    claim,
  }: {
    readonly claim: IssueClaim;
  },
): string {
  return `${claim.category} ${claim.severity}: ${flattenSpace({ text: claim.summary, },)}`;
}

/**
 Log line for one resolved claim at the critic stage.

 @param sliceIndex - chunk position

 @param claimId - claim identity

 @param claim - claim as validated

 @param filers - filers per claim id

 @returns Line naming the filers before the claim's body

 @example
 ```ts
 l.info(describeClaimFiling({ sliceIndex, claimId, claim, filers, },),);
 ```
 */
export function describeClaimFiling(
  {
    sliceIndex,
    claimId,
    claim,
    filers,
  }: {
    readonly sliceIndex: number;
    readonly claimId: string;
    readonly claim: IssueClaim;
    readonly filers: ClaimFilers;
  },
): string {
  return `chunk ${String(sliceIndex,)}: claim ${claimId} filed by ${
    filersClause({
      claimId,
      filers,
    },)
  }: ${claimBody({ claim, },)}`;
}

/**
 Log line for one adjudicated issue: its fate, then every member claim with
 its filers and the panel that voted on it (hulicaijia26, 2026-09-26: the
 owner asked who accepted a translator note as an addition).

 @param sliceIndex - chunk position

 @param issue - issue after the panel spoke

 @param filers - filers per claim id

 @returns Line naming the issue's status and severity, then each claim

 @example
 ```ts
 l.info(describeIssueFiling({ sliceIndex, issue, filers, },),);
 ```
 */
export function describeIssueFiling(
  {
    sliceIndex,
    issue,
    filers,
  }: {
    readonly sliceIndex: number;
    readonly issue: AdjudicatedIssue;
    readonly filers: ClaimFilers;
  },
): string {
  /**
   Each member claim with its filers and its panel.
   */
  const members = issue.claims
    .map(function toClause(member,): string {
      return `${member.claimId} filed by ${
        filersClause({
          claimId: member.claimId,
          filers,
        },)
      }, panel ${
        panelClause({
          claimId: member.claimId,
          issue,
        },)
      }: ${claimBody({ claim: member.claim, },)}`;
    },);
  return `chunk ${String(sliceIndex,)}: issue ${issue.issueId} ${issue.status} ${issue.severity}: ${
    members.join('; ',)
  }`;
}

/**
 Attaches the filers to each issue beside its claims, for the artifact.

 Only claims the record names appear in `filedBy`; a claim with nobody on
 record is left out rather than written as an empty list, so a reader can
 tell an unattributed claim from one whose filers were an empty bench.

 @param issues - issues after the panel spoke

 @param filers - filers per claim id

 @returns Same issues with `filedBy` set where any member claim is on record

 @example
 ```ts
 const recorded = attachClaimFilers({ issues, filers, },);
 ```
 */
export function attachClaimFilers(
  {
    issues,
    filers,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
    readonly filers: ClaimFilers;
  },
): readonly AdjudicatedIssue[] {
  return issues.map(function attach(issue,): AdjudicatedIssue {
    /**
     Filers of the member claims the record names.
     */
    const filedBy = Object.fromEntries(issue.claims
      .filter(function onRecord(member,): boolean {
        return filers[member.claimId] !== undefined;
      },)
      .map(function toEntry(member,): readonly [
        string,
        readonly RosterModelId[]
      ] {
        return [
          member.claimId,
          // Filers of this claim, present since the filter kept the claim.
          nonNullishOrThrow(filers[member.claimId],),
        ];
      },),);
    /**
     How many member claims are on record.
     */
    const onRecordCount = Object.keys(filedBy,)
      .length;
    if (onRecordCount === 0)
      return issue;
    return {
      ...issue,
      filedBy,
    };
  },);
}

/**
 Records a chunk's issues with their filers attached and logs one line per
 issue: the fate the panel gave it and, for every member claim, who filed it.

 @param sliceIndex - chunk position

 @param issues - reference screen's rejections then the panel's issues

 @param attributions - critic phase's attribution record

 @param l - pipeline logger

 @returns Same issues with `filedBy` attached where any claim is on record

 @example
 ```ts
 const recordedIssues = recordIssuesWithFilers({ sliceIndex, issues, attributions, l, },);
 ```
 */
export function recordIssuesWithFilers(
  {
    sliceIndex,
    issues,
    attributions,
    l,
  }: {
    readonly sliceIndex: number;
    readonly issues: readonly AdjudicatedIssue[];
    readonly attributions: readonly ClaimAttribution[];
    readonly l: Logger;
  },
): readonly AdjudicatedIssue[] {
  /**
   Filers per claim id.
   */
  const filers = claimFilersOf({ attributions, },);

  /**
   Issues with the filers attached.
   */
  const recorded = attachClaimFilers({
    issues,
    filers,
  },);
  // ONE LINE PER ADJUDICATED ISSUE, WITH ITS FILERS (owner, 2026-09-24).
  for (const issue of recorded) {
    l.info(describeIssueFiling({
      sliceIndex,
      issue,
      filers,
    },),);
  }
  return recorded;
}

//endregion Claim filers
