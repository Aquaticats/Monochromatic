import type { AdjudicatedIssue, } from './adjudicate-model.ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Patch model
// Editable envelopes are the only places editors may touch: regions of the
// translation derived from accepted issues' target-side evidence. Editors
// return operations against envelope base hashes, never whole rewritten
// chunks (settled architecture), so a deterministic gate can prove every
// change stayed inside its envelope and applied to the text it was written
// against.

/**
 One region of the translation an editor may rewrite.
 Zero-width envelopes (`startOffset === endOffset`, empty base) are
 insertion points for omitted content.

 @example
 ```ts
 const envelope: EditableEnvelope = {
   envelopeId: 'envelope/abc',
   startOffset: 42,
   endOffset: 42,
   baseText: '',
   baseHash: hashContent({ content: '', },),
   issueIds: ['adjudicated/def',],
 };
 ```
 */
export type EditableEnvelope = {
  /**
   Deterministic `envelope/<hash>` identity over offsets and base text.
   */
  readonly envelopeId: string;

  /**
   Absolute start within the full translation text.
   */
  readonly startOffset: number;

  /**
   Absolute end (exclusive); equal to start for insertion envelopes.
   */
  readonly endOffset: number;

  /**
   Exact text currently occupying the envelope.
   */
  readonly baseText: string;

  /**
   Content hash of the base text; editors echo it so application can
   prove the edit was written against the text it replaces.
   */
  readonly baseHash: string;

  /**
   Accepted issues this envelope serves, in issue order.
   */
  readonly issueIds: readonly string[];
};

/**
 Envelopes plus the accepted issues no envelope could serve.

 @example
 ```ts
 const plan: EnvelopePlan = deriveEditableEnvelopes({ issues, targetText, },);
 ```
 */
export type EnvelopePlan = {
  /**
   Non-overlapping envelopes in document order.
   */
  readonly envelopes: readonly EditableEnvelope[];

  /**
   Accepted issues without any target-side anchor;
   they stay unresolved because no envelope can host their repair.
   */
  readonly unenveloped: readonly string[];
};

/**
 One target-side interval contributed by one accepted issue.
 */
type IssueInterval = {
  /**
   Absolute start of the contributed span.
   */
  readonly start: number;

  /**
   Absolute end (exclusive).
   */
  readonly end: number;

  /**
   Issue contributing the span.
   */
  readonly issueId: string;
};

/**
 Derives editable envelopes from accepted issues:
 every accepted issue's target-side spans become intervals, overlapping or
 touching intervals merge into one envelope carrying every contributing
 issue, and non-accepted issues contribute nothing.
 Merging keeps envelopes non-overlapping by construction, which the
 deterministic apply gate relies on.

 @param issues - adjudicated issues; only accepted ones contribute

 @param targetText - full translation the envelopes cut from

 @returns Envelopes in document order plus unenveloped accepted issues

 @example
 ```ts
 const { envelopes, unenveloped, } = deriveEditableEnvelopes({ issues, targetText, },);
 ```
 */
export function deriveEditableEnvelopes(
  {
    issues,
    targetText,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
    readonly targetText: string;
  },
): EnvelopePlan {
  /**
   Accepted issues in adjudication order.
   */
  const accepted = issues.filter(function isAccepted(issue,) {
    return issue.status === 'accepted';
  },);

  /**
   Target-side intervals contributed by every accepted issue.
   */
  const intervals: readonly IssueInterval[] = accepted
    .flatMap(function toIntervals(issue,): readonly IssueInterval[] {
      return issue.claims
        .flatMap(function claimIntervals(member,) {
          return member
            .claim
            .spans
            .flatMap(function spanInterval(span,): readonly IssueInterval[] {
              if (span.side !== 'target')
                return [];
              return [{
                start: span.startOffset,
                end: span.endOffset,
                issueId: issue.issueId,
              },];
            },);
        },);
    },)
    .toSorted(function byStart(
      left,
      right,
    ) {
      if (left.start !== right.start)
        return left.start - right.start;
      return left.end - right.end;
    },);

  /**
   Accepted issues that contributed no target-side interval.
   */
  const unenveloped = accepted
    .filter(function lacksInterval(issue,) {
      return !intervals.some(function contributes(interval,) {
        return interval.issueId === issue.issueId;
      },);
    },)
    .map(function toId(issue,) {
      return issue.issueId;
    },);

  /**
   One merged region under construction.
   */
  type IntervalGroup = {
    readonly start: number;
    readonly end: number;
    readonly issueIds: readonly string[];
  };

  /**
   Merged interval groups: overlapping or touching intervals coalesce.
   Single linear pass over the sorted intervals; the trailing group is
   replaced in place while it keeps absorbing neighbors.
   */
  const merged: IntervalGroup[] = [];
  for (const interval of intervals) {
    /**
     Group currently open for merging, when one exists.
     */
    const last = merged.at(-1,);
    if ((last !== undefined) && (interval.start <= last.end)) {
      merged[merged.length - 1] = {
        start: last.start,
        end: Math.max(
          last.end,
          interval.end,
        ),
        issueIds: last.issueIds
          .includes(interval.issueId,)
          ? last.issueIds
          : [
            ...last.issueIds,
            interval.issueId,
          ],
      };
      continue;
    }
    merged.push({
      start: interval.start,
      end: interval.end,
      issueIds: [interval.issueId,],
    },);
  }

  return {
    envelopes: merged.map(function toEnvelope(group,): EditableEnvelope {
      /**
       Exact text currently occupying the group's region.
       */
      const baseText = targetText.slice(
        group.start,
        group.end,
      );

      return {
        envelopeId: `envelope/${
          hashContent({
            content: JSON.stringify([
              group.start,
              group.end,
              baseText,
            ],),
          },)
        }`,
        startOffset: group.start,
        endOffset: group.end,
        baseText,
        baseHash: hashContent({ content: baseText, },),
        issueIds: group.issueIds,
      };
    },),
    unenveloped,
  };
}

//endregion Patch model
