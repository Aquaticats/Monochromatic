import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { ADDITION_IS_REMOVED_NOT_SOFTENED, } from './addition-repair-rule.ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { citedReferenceBlockText, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from './declared-identity-rule.ts';
import {
  CHECKER_IDENTITY_RULE,
  CHECKER_QUOTE_RULE,
  CHECKER_REFERENCE_RULE,
  MEASUREMENT_POLICY_BLOCK,
  resolutionIssueBlock,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  isJsonArray,
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Resolution check
// Region changed does not mean issue resolved (settled architecture): after
// patches apply, checker models look at each accepted issue against the
// revised translation and say whether the defect is actually gone. Checkers
// answer with issue numbers and a closed verdict vocabulary; a strict
// majority of `fixed` verdicts across checkers marks an issue resolved.


/**
 Every verdict a checker may cast on one issue, closed vocabulary.
 `worse` flags a repair that damaged the region beyond the original
 defect; the no-regression measurement counts it against the candidate.

 @example
 ```ts
 RESOLUTION_VERDICTS.includes('fixed',);
 ```
 */
export const RESOLUTION_VERDICTS = [
  'fixed',
  'not-fixed',
  'worse',
] as const;

/**
 One checker verdict on one issue.

 @example
 ```ts
 const verdict: ResolutionVerdict = 'fixed';
 ```
 */
export type ResolutionVerdict = typeof RESOLUTION_VERDICTS[number];

/**
 Guards untrusted verdict strings from model JSON.

 @param value - candidate from unvalidated model output

 @returns Whether value names one listed verdict

 @example
 ```ts
 isResolutionVerdict('fixed',);
 ```
 */
export function isResolutionVerdict(value: unknown,): value is ResolutionVerdict {
  if ((typeof value) !== 'string')
    return false;

  return (RESOLUTION_VERDICTS as readonly string[]).includes(value,);
}

/**
 How every checker judges, before any rule a sheet's evidence adds.
 */
const RESOLUTION_JUDGING_RULES = `You are a strict bilingual translation reviewer.
Editors revised the TRANSLATION of the ORIGINAL document to fix the numbered issues below.
For EVERY issue, judge the REVISED translation:
- fixed: the defect is gone and the fix reads correctly
- not-fixed: the defect is still present, in the same or another form
- worse: the revision introduced new damage around this issue

${CHECKER_QUOTE_RULE}

${ADDITION_IS_REMOVED_NOT_SOFTENED} For such an issue, such a restatement is not-fixed.

${MEASUREMENT_POLICY_BLOCK}

An issue asking for a detail reader protection keeps out in the account of how a death or an attempt happened is answered not-fixed, and there is no verdict here meaning the issue should never have been filed: the REVISED translation is right not to carry that detail, and saying fixed would agree that it should. Where the REVISED translation HAS restored such a detail, the verdict is worse.`;

/**
 Reply shape every checker is held to, stated last.
 */
const RESOLUTION_REPLY_RULE = `Reply with ONLY a JSON object of shape {"checks": [{"issue": 1, "verdict": "fixed"}]}. No prose, no code fences.
Every issue number must appear exactly once in checks.`;

/**
 System instructions for one checker sheet: the judging rules, the rules
 for the evidence this sheet shows, then the reply shape.

 THE EVIDENCE RULES RIDE WITH THEIR BLOCKS, as the introduced-defect probe's
 identity rules do (ledger H8): a sheet for a page declaring nothing and
 linking nowhere reads the same rules it always did.

 @param declaresNames - whether the sheet shows a DECLARED NAMES block

 @param citesReferences - whether the sheet shows the cited references

 @returns System prompt text

 @example
 ```ts
 const system = resolutionSystemPrompt({ declaresNames: false, citesReferences: false, },);
 ```
 */
function resolutionSystemPrompt(
  {
    declaresNames,
    citesReferences,
  }: {
    readonly declaresNames: boolean;
    readonly citesReferences: boolean;
  },
): string {
  return [
    RESOLUTION_JUDGING_RULES,
    ...(declaresNames ? [`${DECLARED_IDENTITY_RULES}\n${CHECKER_IDENTITY_RULE}`,] : []),
    ...(citesReferences ? [CHECKER_REFERENCE_RULE,] : []),
    RESOLUTION_REPLY_RULE,
  ].join('\n\n',);
}

/**
 Messages plus the issue order checks resolve through:
 issue number N on the wire means `issueIds[N - 1]`.

 @example
 ```ts
 const plan: ResolutionPromptPlan = buildResolutionMessages({
   sourceText,
   patchedText,
   issues,
 },);
 ```
 */
export type ResolutionPromptPlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];

  /**
   Issue ids in prompt numbering order.
   */
  readonly issueIds: readonly string[];
};

/**
 Builds the checker sheet: original, revised translation, and every
 accepted issue the editors were asked to fix with its claims' quotes, with
 the declared names before the documents and the cited references after
 them, as the panel that accepted the issues read them (ledger L14).

 @param sourceText - original chunk text

 @param patchedText - revised translation after patch application

 @param issues - accepted issues the editors addressed

 @param identityContext - declared names and handles, absent or empty on a
 page declaring none

 @param referenceContext - what the pages the original links say, absent or
 empty when it links nowhere

 @returns Messages plus issue numbering order

 @example
 ```ts
 const plan = buildResolutionMessages({ sourceText, patchedText, issues, },);
 ```
 */
export function buildResolutionMessages(
  {
    sourceText,
    patchedText,
    issues,
    identityContext,
    referenceContext,
  }: {
    readonly sourceText: string;
    readonly patchedText: string;
    readonly issues: readonly AdjudicatedIssue[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
  },
): ResolutionPromptPlan {
  /**
   Rendered issue blocks in issue order.
   */
  const blocks = issues.map(function toBlock(
    issue,
    index,
  ) {
    return resolutionIssueBlock({
      issue,
      index,
    },);
  },);

  // THE CHECKER IS SHOWN THE REVISED TRANSLATION WITHOUT THE NOTES THE
  // ARCHIVE'S TRANSLATORS LEFT EACH OTHER, as the critic and the panel that
  // filed the issues were (`critic-prompt.ts`): no reader sees one, so a note
  // may neither fix an issue nor leave it standing. Only what is SHOWN is cut;
  // the claims' quotes are those of the stored text, and the original's own
  // comments stay in view.
  /**
   The revised translation as the checker reads it.
   */
  const patchedShown = withoutTranslatorsNotes({
    text: patchedText,
    original: sourceText,
  },);

  /**
   Fence no enclosed text can reproduce, chosen against the declared names
   and the references too, since either is arbitrary page text, and against
   the revised translation as written and as shown.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      patchedText,
      patchedShown,
      identityContext ?? '',
      referenceContext ?? '',
      ...blocks,
    ],
  },);

  /**
   Declared names before the documents, or nothing on a page declaring none.
   */
  const identityBlock = declaredNamesBlock({
    fence,
    ...((identityContext === undefined) ? {} : { identityContext, }),
  },);

  /**
   Cited references after the revised translation, or nothing when the
   original links nowhere.
   */
  const referenceBlock = citedReferenceBlockText({
    fence,
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  },);

  return {
    messages: [
      {
        role: 'system',
        content: resolutionSystemPrompt({
          declaresNames: identityBlock !== '',
          citesReferences: referenceBlock !== '',
        },),
      },
      {
        role: 'user',
        content: `${identityBlock}${fence} ORIGINAL ${fence}
${sourceText}
${fence} REVISED TRANSLATION ${fence}
${patchedShown}
${referenceBlock}${fence} ISSUES ${fence}
${blocks.join('\n\n',)}
${fence} END ${fence}`,
      },
    ],
    issueIds: issues.map(function toId(issue,) {
      return issue.issueId;
    },),
  };
}

/**
 One check as a checker reports it.

 @example
 ```ts
 const wire: ResolutionCheckWire = { issue: 1, verdict: 'fixed', };
 ```
 */
export type ResolutionCheckWire = {
  /**
   One-based issue number from the prompt sheet.
   */
  readonly issue: number;

  /**
   Verdict string; validated against the closed vocabulary at resolution.
   */
  readonly verdict: string;
};

/**
 Whole checker reply on the wire.

 @example
 ```ts
 const report: ResolutionReportWire = { checks: [], };
 ```
 */
export type ResolutionReportWire = {
  /**
   Every check cast.
   */
  readonly checks: readonly ResolutionCheckWire[];
};

/**
 Guards one wire check.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required check fields

 @example
 ```ts
 isResolutionCheckWire({ issue: 1, verdict: 'fixed', },);
 ```
 */
function isResolutionCheckWire(value: unknown,): value is ResolutionCheckWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Issue reference as reported; integerness checked on the primitive copy.
   */
  const { issue, } = value;
  if ((typeof issue) !== 'number')
    return false;
  if ((issue % 1) !== 0)
    return false;
  return (typeof value.verdict) === 'string';
}

/**
 Guards a whole checker reply.

 @param value - parsed model JSON

 @returns Whether value is a wire report

 @example
 ```ts
 const outcome = await client.chatJson({ ..., validate: isResolutionReportWire, },);
 ```
 */
export function isResolutionReportWire(value: unknown,): value is ResolutionReportWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.checks,))
    return false;
  return value.checks
    .every(function eachCheck(check,) {
      return isResolutionCheckWire(check,);
    },);
}

/**
 Guard for a report the gather may count as a heard checker: a wire report
 carrying at least one known verdict on an issue the sheet showed.

 LEDGER L8: `isResolutionReportWire` accepts `{"checks": []}`, a report
 checking only issue numbers the sheet never showed, and one whose only
 verdict is no verdict at all, so each counted as heard and closed a round
 (XingZ6014 slice 87 resolved an issue on the one other ballot). Such a
 report carries no voice, so the gather reads it as unreadable and the
 recovery round re-asks the seat. A report usable on some issues is heard,
 and abstains on the rest at tally time.

 @param issueCount - issues the sheet showed, numbered from one

 @returns Guard over parsed model JSON

 @example
 ```ts
 const gather = await gatherStageVoices({ ..., validate: usableResolutionReportFor({ issueCount: 2, },), },);
 ```
 */
export function usableResolutionReportFor(
  { issueCount, }: { readonly issueCount: number; },
): (value: unknown,) => value is ResolutionReportWire {
  return function isUsableResolutionReport(value: unknown,): value is ResolutionReportWire {
    if (!isResolutionReportWire(value,))
      return false;
    return value
      .checks
      .some(function checksShownIssue(check,): boolean {
        return (check.issue >= 1)
          && (check.issue <= issueCount)
          && isResolutionVerdict(check.verdict,);
      },);
  };
}

/**
 Structured-output constraint for checker calls;
 client-side validation through {@link isResolutionReportWire} stays
 regardless, because per-model schema strictness is unverified.
 */
export const RESOLUTION_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'resolution_report',
    schema: {
      type: 'object',
      required: ['checks',],
      additionalProperties: false,
      properties: {
        checks: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'issue',
              'verdict',
            ],
            additionalProperties: false,
            properties: {
              issue: { type: 'integer', },
              // THE CLOSED VOCABULARY `isResolutionVerdict` reads, as an enum (P1 of the second review of the Claude route).
              verdict: {
                type: 'string',
                enum: [...RESOLUTION_VERDICTS,],
              },
            },
          },
        },
      },
    },
  },
};

//endregion Resolution check
