import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { AdjudicatedIssue, } from './adjudicate-model.ts';
import {
  type IssueEvidenceConflictError,
  mergeDuplicateEvidence,
} from './dedupe-evidence.ts';

//region Dedupe issues
// One defect reaching the editor as several accepted issues is waste, not
// evidence. Measured over pass13, exact-place duplicates are 13.4% of accepted
// issues, and the human grader independently marked 14% of a 50-item draw as
// duplicates while grading detection. Ratified in
// `doc/decision/translation-repair-duplicate-issue-emission.md`.
//
// MERGED RATHER THAN DROPPED, which is the part that matters. A dropped
// duplicate takes its claim ids with it, and those ids are how attribution
// answers the question that made duplicates interesting in the first place:
// whether a duplicate came from one critic repeating itself or from several
// critics agreeing. The survivor carries every member claim, so collapsing the
// ISSUES does not collapse the proposer record.
//
// The key is category AND span set, never span alone. A shared span is narrow
// but not proof of identity, and two genuinely different complaints about one
// sentence must both survive.

/**
 Builds the identity two accepted issues must share to be one defect.

 @param issue - adjudicated issue

 @returns Stable key over category and the exact spans claimed

 @example
 ```ts
 const key = duplicateKey({ issue, },);
 ```
 */
function duplicateKey(
  {
    issue,
  }: {
    readonly issue: AdjudicatedIssue;
  },
): string {
  /**
   Categories claimed, deduplicated so member order cannot change the key.
   */
  const categories = textsInCodePointOrder({ texts: [...new Set(issue.claims
    .map(function toCategory(member,): string {
      return member.claim
        .category;
    },),),], },);

  /**
   Every span claimed, rendered field by field rather than by JSON, since
   property order in a serialized object is not a guarantee worth keying on.
   */
  const spans = textsInCodePointOrder({ texts: [...new Set(issue.claims
    .flatMap(function toSpans(member,): readonly string[] {
      return member.claim
        .spans
        .map(function render(span,): string {
          return `${span.side}|${span.nodeId}|${String(span.startOffset,)}`;
        },);
    },),),], },);

  return `${categories.join(',',)}::${spans.join(',',)}`;
}

/**
 Everything deduplication changed.

 @example
 ```ts
 const { issues, findings, } = dedupeAcceptedIssues({ issues: panel.issues, },);
 ```
 */
export type DedupeOutcome = {
  /**
   Issues with same-place accepted duplicates merged, in input order.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   One finding per merge, so a run records what it collapsed rather than
   quietly reporting a smaller number than it accepted.
   */
  readonly findings: readonly string[];
};

/**
 Merges accepted issues that name the same defect in the same place.

 Leaves every non-accepted issue untouched and in place: a rejected duplicate
 costs no repair budget, and collapsing rejections would change what the
 precision denominator counts.

 @param issues - adjudicated issues for one chunk, in panel order

 @returns Merged issues and a finding per merge

 @throws {@link IssueEvidenceConflictError} when repeated claim evidence conflicts

 @example
 ```ts
 const { issues, findings, } = dedupeAcceptedIssues({ issues, },);
 ```
 */
export function dedupeAcceptedIssues(
  {
    issues,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
  },
): DedupeOutcome {
  /**
   Position in the output of the accepted issue holding each key.
   */
  const firstAt = new Map<string, number>();

  /**
   Issues being built, with survivors accumulating merged claims.
   */
  const kept: AdjudicatedIssue[] = [];

  /**
   One line per merge.
   */
  const findings: string[] = [];

  for (const issue of issues) {
    if (issue.status !== 'accepted') {
      kept.push(issue,);
      continue;
    }

    /**
     Identity this issue shares with any duplicate of it.
     */
    const key = duplicateKey({ issue, },);

    /**
     Where the first issue with this identity landed, if any.
     */
    const at = firstAt.get(key,);
    if (at === undefined) {
      firstAt.set(
        key,
        kept.length,
      );
      kept.push(issue,);
      continue;
    }

    /**
     Survivor this duplicate folds into.
     */
    const survivor = kept[at];
    if (survivor === undefined)
      throw new Error(`unreachable: duplicate key ${key} indexed a position holding no issue`,);

    /**
     Claim ids the survivor already holds.
     */
    const held = new Set(survivor.claims
      .map(function toId(member,): string {
        return member.claimId;
      },),);

    kept[at] = mergeDuplicateEvidence({
      survivor,
      incoming: issue,
      claims: [
        ...survivor.claims,
        ...issue.claims
          .filter(function isNew(member,): boolean {
            return !held.has(member.claimId,);
          },),
      ],
    },);
    findings.push(`duplicate-issue-merged (${issue.issueId} into ${survivor.issueId})`,);
  }

  return {
    issues: kept,
    findings,
  };
}

//endregion Dedupe issues
