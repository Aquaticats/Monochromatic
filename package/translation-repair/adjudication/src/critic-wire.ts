import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type AnchorTarget,
  type DocumentSide,
  isIssueCategory,
  isIssueSeverity,
  ISSUE_CATEGORIES,
  ISSUE_SEVERITIES,
  type IssueClaim,
  remapCategoryLeaf,
  type SpanAnchor,
  validateIssueClaim,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { locateQuote, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Critic wire format
// What critics actually emit: category, severity, one-sentence summary, and exact
// quotes. Models cannot know node ids or hashes, so anchoring is deterministic
// resolution work done in locate-quote.ts: find each quote, reject absence and
// ambiguity, bind the region to blocks, and gate the result through span
// validation here. Resolution failures are data for the scorecard, never
// exceptions.

/**
 One issue as a critic reports it on the wire.

 @example
 ```ts
 const wire: CriticIssueWire = {
   category: 'accuracy/omission',
   severity: 'major',
   summary: 'Second clause is untranslated.',
   sourceQuote: '也喜欢追蝴蝶',
   targetQuote: 'The cat likes to nap in the sun.',
 };
 ```
 */
export type CriticIssueWire = {
  /**
   Category slug; validated against the closed taxonomy at resolution.
   */
  readonly category: string;

  /**
   Severity; validated at resolution.
   */
  readonly severity: string;

  /**
   One-sentence statement of the single defect claimed.
   */
  readonly summary: string;

  /**
   Exact substring of the original document, when the defect has a
   source-side anchor.
   */
  readonly sourceQuote?: string;

  /**
   Exact substring of the translation, when the defect has a
   target-side anchor.
   */
  readonly targetQuote?: string;
};

/**
 Whole critic reply on the wire.

 @example
 ```ts
 const report: CriticReportWire = { issues: [], };
 ```
 */
export type CriticReportWire = {
  /**
   Every issue found; empty means the critic found nothing.
   */
  readonly issues: readonly CriticIssueWire[];
};

/**
 Guards one wire issue.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required wire fields

 @example
 ```ts
 isCriticIssueWire({ category: 'x', severity: 'y', summary: 'z', },);
 ```
 */
function isCriticIssueWire(value: unknown,): value is CriticIssueWire {
  if (!isJsonRecord(value,))
    return false;
  if ((typeof value.category) !== 'string')
    return false;
  if ((typeof value.severity) !== 'string')
    return false;
  if ((typeof value.summary) !== 'string')
    return false;
  if ((value.sourceQuote !== undefined) && ((typeof value.sourceQuote) !== 'string'))
    return false;
  return (value.targetQuote === undefined) || ((typeof value.targetQuote) === 'string');
}

/**
 Guards a whole critic reply.

 @param value - parsed model JSON

 @returns Whether value is a wire report

 @example
 ```ts
 const outcome = await client.chatJson({ ..., validate: isCriticReportWire, },);
 ```
 */
export function isCriticReportWire(value: unknown,): value is CriticReportWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.issues,))
    return false;
  return value
    .issues
    .every(function eachIssue(issue,) {
      return isCriticIssueWire(issue,);
    },);
}

/**
 Structured-output constraint for critic calls;
 client-side validation through {@link isCriticReportWire} stays regardless,
 because per-model schema strictness is unverified.
 */
export const CRITIC_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'critic_report',
    schema: {
      type: 'object',
      required: ['issues',],
      additionalProperties: false,
      properties: {
        issues: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'category',
              'severity',
              'summary',
            ],
            additionalProperties: false,
            properties: {
              // THE CLOSED VOCABULARIES THE READER CHECKS, as an enum (P1 of the second review of the Claude route): the
              // reader also remaps a bare leaf name of a category, which the schema does not offer, so a reply that
              // follows the schema is always read as written.
              category: {
                type: 'string',
                enum: [...ISSUE_CATEGORIES,],
              },
              severity: {
                type: 'string',
                enum: [...ISSUE_SEVERITIES,],
              },
              sourceQuote: { type: 'string', },
              summary: { type: 'string', },
              targetQuote: { type: 'string', },
            },
          },
        },
      },
    },
  },
};

/**
 Outcome of resolving one wire issue into an anchored claim.

 @example
 ```ts
 const resolution: CriticIssueResolution = { resolved: false, reason: 'quote-not-found (target)', };
 ```
 */
export type CriticIssueResolution =
  | {
    /**
     Every quote anchored and validated.
     */
    readonly resolved: true;

    /**
     Anchored atomic claim ready for adjudication or grading.
     */
    readonly claim: IssueClaim;
  }
    | {
    /**
     Some step refused; the reason feeds the scorecard.
     */
      readonly resolved: false;

    /**
     Which step failed and why, in scorecard-stable wording.
     */
      readonly reason: string;
    };

/**
 Resolves one wire issue into an anchored, validated claim.
 Category and severity must belong to the closed vocabularies;
 each present quote must locate uniquely inside one block;
 the assembled claim must pass deterministic span validation.

 @param wire - issue as the critic reported it

 @param documents - current pair anchors resolve against

 @param original - the texts whose comments are the original's own on the sheet
 the critic read (`criticOriginalsOf`): the translation quote is located in the
 translation as that sheet showed it, without the notes between translators, and
 comes back as the stored stretches of words either side of each note it
 crosses; absent when the translation was not cut

 @returns Anchored claim, or the failure reason as data

 @example
 ```ts
 const resolution = resolveCriticIssue({ wire, documents, original, },);
 if (resolution.resolved) grade(resolution.claim,);
 ```
 */
export function resolveCriticIssue(
  {
    wire,
    documents,
    original,
  }: {
    readonly wire: CriticIssueWire;
    readonly documents: Readonly<Record<DocumentSide, AnchorTarget>>;
    readonly original?: string;
  },
): CriticIssueResolution {
  /**
   Category after tolerant family remap:
   models slip families on known leaves (fluency/awkward-phrasing for
   style/awkward-phrasing), and a leaf owned by exactly one family maps
   onto its owner; unknown or ambiguous leaves stay rejected.
   */
  const remap = isIssueCategory(wire.category,)
    ? {
      remapped: true as const,
      category: wire.category,
    }
    : remapCategoryLeaf({ category: wire.category, },);
  if (!remap.remapped) {
    return {
      resolved: false,
      reason: `unknown-category (${wire.category})`,
    };
  }
  if (!isIssueSeverity(wire.severity,)) {
    return {
      resolved: false,
      reason: `unknown-severity (${wire.severity})`,
    };
  }

  /**
   Quotes present on the wire, paired with their sides.
   */
  const quotes: readonly {
    readonly side: DocumentSide;
    readonly quote: string;
  }[] = [
    ...(wire.sourceQuote === undefined
      ? []
      : [{
        side: 'source' as const,
        quote: wire.sourceQuote,
      },]),
    ...(wire.targetQuote === undefined
      ? []
      : [{
        side: 'target' as const,
        quote: wire.targetQuote,
      },]),
  ];
  if (quotes.length === 0) {
    return {
      resolved: false,
      reason: 'no-quotes',
    };
  }

  /**
   Located anchors in side order; first failure wins.
   */
  const anchors: SpanAnchor[] = [];
  for (
    const {
      side,
      quote,
    } of quotes
  ) {
    /**
     Location attempt for this side's quote.
     */
    const located = locateQuote({
      document: documents[side],
      side,
      quote,
      // Only the translation is cut on the sheet; the original is shown as it stands.
      ...(((side === 'target') && (original !== undefined)) ? { original, } : {}),
    },);
    if (!located.located) {
      return {
        resolved: false,
        reason: located.reason,
      };
    }
    anchors.push(...located.anchors,);
  }

  /**
   Assembled claim awaiting the deterministic gate.
   */
  const claim: IssueClaim = {
    category: remap.category,
    severity: wire.severity,
    summary: wire.summary,
    spans: anchors,
  };

  /**
   Final deterministic gate; failures here mean resolution built a bad
   anchor, and the claim must not pass as if it were validated.
   */
  const rejections = validateIssueClaim({
    claim,
    documents,
  },);
  if (rejections.length > 0) {
    return {
      resolved: false,
      reason: `anchor-validation (${
        rejections.map(function toKind(rejection,) {
          return rejection.kind;
        },)
          .join(',',)
      })`,
    };
  }

  return {
    resolved: true,
    claim,
  };
}

//endregion Critic wire format
