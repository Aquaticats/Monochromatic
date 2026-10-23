import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { MEASUREMENT_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  isJsonArray,
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Restoration judge wire format
// Milestone-two grading anchored on the Chinese source (user directive):
// a bilingual judge reads the ORIGINAL Chinese, a numbered REFERENCE (the
// deleted English sentence, used only to pinpoint which content to check),
// and the REPAIRED translation, then rules whether the reference's meaning
// is back AND grounded in the Chinese. Wording may differ freely, so a
// terse but faithful re-translation counts as restored; a vocabulary-overlap
// grader under-credited exactly those, which is why this replaces it as the
// primary rate.


/**
 Every verdict a restoration judge may cast, closed vocabulary.

 @example
 ```ts
 RESTORATION_JUDGE_VERDICTS.includes('restored',);
 ```
 */
export const RESTORATION_JUDGE_VERDICTS = [
  'restored',
  'partial',
  'absent',
] as const;

/**
 One restoration verdict.

 @example
 ```ts
 const verdict: RestorationVerdict = 'restored';
 ```
 */
export type RestorationVerdict = typeof RESTORATION_JUDGE_VERDICTS[number];

/**
 Guards untrusted verdict strings from model JSON.

 @param value - candidate from unvalidated model output

 @returns Whether value names one listed verdict

 @example
 ```ts
 isRestorationVerdict('restored',);
 ```
 */
export function isRestorationVerdict(value: unknown,): value is RestorationVerdict {
  if ((typeof value) !== 'string')
    return false;

  return (RESTORATION_JUDGE_VERDICTS as readonly string[]).includes(value,);
}

/**
 System instructions shared by every restoration judge call.
 */
const JUDGE_SYSTEM_PROMPT = `You are a bilingual Chinese-to-English translation grader.
The ORIGINAL is a Chinese document. Its English translation had sentences removed, then a repair system tried to restore them.
For each numbered REFERENCE (an English sentence that was removed), judge the REPAIRED TRANSLATION:
- restored: the reference's information is fully present in the repaired translation AND supported by the ORIGINAL Chinese
- partial: some but not all of the reference's information is present and grounded
- absent: the information is missing from the repaired translation

Judge MEANING, not wording. Different phrasing, word order, or a terse-but-faithful rendering all count as restored. Do not require the exact words of the reference.
Information not supported by the ORIGINAL Chinese is never restored, even if it reads well.

${MEASUREMENT_POLICY_BLOCK}

A repaired sentence vaguer than the ORIGINAL Chinese because a house rule asks for it is restored rather than partial: the REFERENCE was cut from a translation written before those rules, so any detail it carries beyond them is what the rule keeps out, not information the repair lost. Where the repaired translation carries a detail a house rule keeps out, that is not a better restoration and no verdict is raised for it.

Reply with ONLY a JSON object of shape {"judgments": [{"reference": 1, "verdict": "restored"}]}. No prose, no code fences.
Every reference number must appear exactly once.`;

/**
 One reference to grade: its planted seed id and the deleted sentence.

 @example
 ```ts
 const reference: JudgeReference = { seedId: 'seed/omission-0', deletedText: '...', };
 ```
 */
export type JudgeReference = {
  /**
   Planted seed id carried onto the verdict.
   */
  readonly seedId: string;

  /**
   Deleted English sentence, the content pointer.
   */
  readonly deletedText: string;
};

/**
 Messages plus the seed order judgments resolve through:
 reference number N on the wire means `seedIds[N - 1]`.

 @example
 ```ts
 const plan: RestorationJudgePlan = buildRestorationJudgeMessages({
   sourceText,
   repairedText,
   references,
 },);
 ```
 */
export type RestorationJudgePlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];

  /**
   Seed ids in reference numbering order.
   */
  readonly seedIds: readonly string[];
};

/**
 Builds the judge sheet for one entry:
 Chinese source, repaired translation, and every deleted sentence as a
 numbered reference.

 @param sourceText - original Chinese document

 @param repairedText - repaired translation under grading

 @param references - deleted sentences with their seed ids

 @returns Messages plus seed numbering order

 @example
 ```ts
 const plan = buildRestorationJudgeMessages({ sourceText, repairedText, references, },);
 ```
 */
export function buildRestorationJudgeMessages(
  {
    sourceText,
    repairedText,
    references,
  }: {
    readonly sourceText: string;
    readonly repairedText: string;
    readonly references: readonly JudgeReference[];
  },
): RestorationJudgePlan {
  /**
   Rendered reference blocks in seed order.
   */
  const blocks = references.map(function toBlock(
    reference,
    index,
  ) {
    return `REFERENCE ${index + 1}: ${reference.deletedText}`;
  },);

  // THE JUDGE IS SHOWN THE REPAIRED TRANSLATION WITHOUT THE NOTES THE ARCHIVE'S
  // TRANSLATORS LEFT EACH OTHER (`withoutTranslatorsNotes`): no reader sees one,
  // so a note may not stand as restored wording. Judgments resolve through the
  // reference numbers, never through a quote, and the original's own comments
  // stay in view.
  /**
   The repaired translation as the judge reads it.
   */
  const repairedShown = withoutTranslatorsNotes({
    text: repairedText,
    original: sourceText,
  },);

  /**
   Fence no enclosed text can reproduce, the repaired translation as written
   and as shown.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      repairedText,
      repairedShown,
      ...blocks,
    ],
  },);

  return {
    messages: [
      {
        role: 'system',
        content: JUDGE_SYSTEM_PROMPT,
      },
      {
        role: 'user',
        content: `${fence} ORIGINAL ${fence}
${sourceText}
${fence} REPAIRED TRANSLATION ${fence}
${repairedShown}
${fence} REFERENCES ${fence}
${blocks.join('\n',)}
${fence} END ${fence}`,
      },
    ],
    seedIds: references.map(function toId(reference,) {
      return reference.seedId;
    },),
  };
}

/**
 One judgment as a judge reports it.

 @example
 ```ts
 const wire: RestorationJudgmentWire = { reference: 1, verdict: 'restored', };
 ```
 */
export type RestorationJudgmentWire = {
  /**
   One-based reference number from the judge sheet.
   */
  readonly reference: number;

  /**
   Verdict string; validated against the closed vocabulary at resolution.
   */
  readonly verdict: string;
};

/**
 Whole judge reply on the wire.

 @example
 ```ts
 const report: RestorationJudgeWire = { judgments: [], };
 ```
 */
export type RestorationJudgeWire = {
  /**
   Every judgment cast.
   */
  readonly judgments: readonly RestorationJudgmentWire[];
};

/**
 Guards one wire judgment.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required judgment fields

 @example
 ```ts
 isRestorationJudgmentWire({ reference: 1, verdict: 'restored', },);
 ```
 */
function isRestorationJudgmentWire(value: unknown,): value is RestorationJudgmentWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Reference number as reported; integerness checked on the primitive copy.
   */
  const { reference, } = value;
  if ((typeof reference) !== 'number')
    return false;
  if ((reference % 1) !== 0)
    return false;
  return (typeof value.verdict) === 'string';
}

/**
 Guards a whole judge reply.

 @param value - parsed model JSON

 @returns Whether value is a wire report

 @example
 ```ts
 const outcome = await client.chatJson({ ..., validate: isRestorationJudgeWire, },);
 ```
 */
export function isRestorationJudgeWire(value: unknown,): value is RestorationJudgeWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.judgments,))
    return false;
  return value.judgments
    .every(function eachJudgment(judgment,) {
      return isRestorationJudgmentWire(judgment,);
    },);
}

/**
 Structured-output constraint for judge calls;
 client-side validation through {@link isRestorationJudgeWire} stays
 regardless, because per-model schema strictness is unverified.
 */
export const RESTORATION_JUDGE_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'restoration_judgment',
    schema: {
      type: 'object',
      required: ['judgments',],
      additionalProperties: false,
      properties: {
        judgments: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'reference',
              'verdict',
            ],
            additionalProperties: false,
            properties: {
              reference: { type: 'integer', },
              // THE CLOSED VOCABULARY `isRestorationVerdict` reads, as an enum (P1 of the second review of the Claude route).
              verdict: {
                type: 'string',
                enum: [...RESTORATION_JUDGE_VERDICTS,],
              },
            },
          },
        },
      },
    },
  },
};

/**
 Resolves one wire report into seed-keyed verdicts through the plan.
 Fails closed per item: out-of-range or duplicate references and unknown
 verdicts become findings, and references left unanswered are recorded.

 @param wire - report as the judge reported it

 @param seedIds - seed ids in reference numbering order

 @returns Verdicts keyed by seed id plus findings as data

 @example
 ```ts
 const { verdicts, } = resolveRestorationJudgment({ wire, seedIds, },);
 ```
 */
export function resolveRestorationJudgment(
  {
    wire,
    seedIds,
  }: {
    readonly wire: RestorationJudgeWire;
    readonly seedIds: readonly string[];
  },
): {
  readonly verdicts: Readonly<Record<string, RestorationVerdict>>;
  readonly findings: readonly string[];
} {
  /**
   Findings accumulated across every wire item.
   */
  const findings: string[] = [];

  /**
   Resolved verdicts keyed by seed id; first occurrence wins. A map until
   handed back, as every record filled by a key is (ledger B77).
   */
  const verdicts = new Map<string, RestorationVerdict>();
  for (const judgment of wire.judgments) {
    /**
     Seed id referenced by this judgment's one-based number.
     */
    const seedId = seedIds[judgment.reference - 1];
    if ((judgment.reference < 1) || (seedId === undefined)) {
      findings.push(`judge-reference-out-of-range (${judgment.reference})`,);
      continue;
    }
    if (verdicts.has(seedId,)) {
      findings.push(`duplicate-judgment (${judgment.reference})`,);
      continue;
    }
    if (!isRestorationVerdict(judgment.verdict,)) {
      findings.push(`unknown-restoration-verdict (${judgment.verdict})`,);
      continue;
    }
    verdicts.set(
      seedId,
      judgment.verdict,
    );
  }
  for (const [index, seedId,] of seedIds.entries()) {
    if (!verdicts.has(seedId,))
      findings.push(`missing-judgment (${index + 1})`,);
  }

  return {
    verdicts: Object.fromEntries(verdicts,),
    findings,
  };
}

//endregion Restoration judge wire format
