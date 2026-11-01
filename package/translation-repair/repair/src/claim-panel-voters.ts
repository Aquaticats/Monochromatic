import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Claim panel voters
// Who voted each claim through or out, as the adjudication log line names it.
//
// SPLIT FROM `claim-filers.ts` on the line budget, along its own seam: filers
// are the critics that raised a claim, voters the panel that judged it. The
// ballots already live on the issue (`AdjudicatedIssue.readings`); what was
// missing was the line. On 2026-09-26 the owner asked who had accepted
// hulicaijia26's translator note as an addition the original never states,
// and the log could name only the critic that filed it.

/**
 Names no panel on a claim the issue carries no reading for: an issue rebuilt
 from an older artifact, or one no panel adjudicated.
 */
const PANEL_NOT_ON_RECORD = 'not on record';

/**
 Names the panel behind one claim for a log line.

 @param claimId - claim identity

 @param issue - issue after the panel spoke

 @returns Each panelist with its vote in ballot order, or the not-on-record wording

 @example
 ```ts
 const who = panelClause({ claimId, issue, },);
 ```
 */
export function panelClause(
  {
    claimId,
    issue,
  }: {
    readonly claimId: string;
    readonly issue: AdjudicatedIssue;
  },
): string {
  /**
   Panel reading for this claim, when the issue carries one.
   */
  const reading = issue.readings?.[claimId];
  if (reading === undefined)
    return PANEL_NOT_ON_RECORD;

  /**
   Ballots the panel cast on this claim.
   */
  const { ballots, } = reading;
  if (ballots.length === 0)
    return PANEL_NOT_ON_RECORD;
  return ballots
    .map(function toVoter(ballot,): string {
      return `${ballot.panelistId} ${ballot.vote}`;
    },)
    .join(', ',);
}

//endregion Claim panel voters
