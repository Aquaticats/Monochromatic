import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import type {
  JudgeReference,
  RestorationJudgeWire,
} from './restoration-judge-wire.ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Derivability probe wire format
// Calibration question behind the stable-partial verdicts: when a deleted
// English sentence keeps coming back as within-sentence paraphrase, is the
// missing information even PRESENT in the Chinese source? A sentence the
// original translator embellished beyond the Chinese can never be fully
// restored by a faithful zh-anchored editor, so "partial" is its correct
// ceiling and the strict rate under-reports the pipeline. The probe reuses
// the numbered-judgment-sheet wire shape of the restoration judge; only the
// question, the verdict vocabulary, and the resolution differ.


/**
 Every verdict a derivability judge may cast, closed vocabulary.

 @example
 ```ts
 DERIVABILITY_VERDICTS.includes('derivable',);
 ```
 */
export const DERIVABILITY_VERDICTS = [
  'derivable',
  'partially-derivable',
  'not-derivable',
] as const;

/**
 One derivability verdict.

 @example
 ```ts
 const verdict: DerivabilityVerdict = 'derivable';
 ```
 */
export type DerivabilityVerdict = typeof DERIVABILITY_VERDICTS[number];

/**
 Guards untrusted verdict strings from model JSON.

 @param value - candidate from unvalidated model output

 @returns Whether value names one listed verdict

 @example
 ```ts
 isDerivabilityVerdict('derivable',);
 ```
 */
export function isDerivabilityVerdict(value: unknown,): value is DerivabilityVerdict {
  if ((typeof value) !== 'string')
    return false;

  return (DERIVABILITY_VERDICTS as readonly string[]).includes(value,);
}

/**
 System instructions shared by every derivability probe call.
 */
const PROBE_SYSTEM_PROMPT = `You are a bilingual Chinese-to-English translation auditor.
The ORIGINAL is a Chinese document. Each numbered CANDIDATE is one English sentence taken from a human translation of it.
For each CANDIDATE, judge whether a faithful translator could produce ALL of its information using ONLY the ORIGINAL Chinese:
- derivable: every piece of the candidate's information is stated in, or directly inferable from, the ORIGINAL
- partially-derivable: some of the candidate's information is grounded in the ORIGINAL, but some is not (translator embellishment)
- not-derivable: the candidate's information is mostly absent from the ORIGINAL

Judge INFORMATION, not wording. Style, word order, and phrasing never matter; only whether the facts and details are grounded.

Reply with ONLY a JSON object of shape {"judgments": [{"reference": 1, "verdict": "derivable"}]}. No prose, no code fences.
Every candidate number must appear exactly once under "reference".`;

/**
 Messages plus the seed order judgments resolve through:
 candidate number N on the wire means `seedIds[N - 1]`.

 @example
 ```ts
 const plan: DerivabilityPlan = buildDerivabilityMessages({
   sourceText,
   references,
 },);
 ```
 */
export type DerivabilityPlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];

  /**
   Seed ids in candidate numbering order.
   */
  readonly seedIds: readonly string[];
};

/**
 Builds the probe sheet for one entry:
 Chinese source plus every deleted sentence as a numbered candidate.
 No repaired text appears; the probe grades the SOURCE, not any repair.

 @param sourceText - original Chinese document

 @param references - deleted sentences with their seed ids

 @returns Messages plus seed numbering order

 @example
 ```ts
 const plan = buildDerivabilityMessages({ sourceText, references, },);
 ```
 */
export function buildDerivabilityMessages(
  {
    sourceText,
    references,
  }: {
    readonly sourceText: string;
    readonly references: readonly JudgeReference[];
  },
): DerivabilityPlan {
  /**
   Rendered candidate blocks in seed order.
   */
  const blocks = references.map(function toBlock(
    reference,
    index,
  ) {
    return `CANDIDATE ${index + 1}: ${reference.deletedText}`;
  },);

  /**
   Fence no enclosed text can reproduce.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      ...blocks,
    ],
  },);

  return {
    messages: [
      {
        role: 'system',
        content: PROBE_SYSTEM_PROMPT,
      },
      {
        role: 'user',
        content: `${fence} ORIGINAL ${fence}
${sourceText}
${fence} CANDIDATES ${fence}
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
 Structured-output constraint for probe calls; the payload shape matches
 the restoration judge sheet, so its wire guard revalidates client-side.
 */
export const DERIVABILITY_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'derivability_judgment',
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
              // THE CLOSED VOCABULARY `isDerivabilityVerdict` reads, as an enum (P1 of the second review of the Claude route).
              verdict: {
                type: 'string',
                enum: [...DERIVABILITY_VERDICTS,],
              },
            },
          },
        },
      },
    },
  },
};

/**
 Resolves one wire report into seed-keyed derivability verdicts through
 the plan. Fails closed per item: out-of-range or duplicate candidate
 numbers and unknown verdicts become findings, and candidates left
 unanswered are recorded.

 @param wire - report as the judge reported it

 @param seedIds - seed ids in candidate numbering order

 @returns Verdicts keyed by seed id plus findings as data

 @example
 ```ts
 const { verdicts, } = resolveDerivabilityJudgment({ wire, seedIds, },);
 ```
 */
export function resolveDerivabilityJudgment(
  {
    wire,
    seedIds,
  }: {
    readonly wire: RestorationJudgeWire;
    readonly seedIds: readonly string[];
  },
): {
  readonly verdicts: Readonly<Record<string, DerivabilityVerdict>>;
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
  const verdicts = new Map<string, DerivabilityVerdict>();
  for (const judgment of wire.judgments) {
    /**
     Seed id referenced by this judgment's one-based candidate number.
     */
    const seedId = seedIds[judgment.reference - 1];
    if ((judgment.reference < 1) || (seedId === undefined)) {
      findings.push(`derivability-reference-out-of-range (${judgment.reference})`,);
      continue;
    }
    if (verdicts.has(seedId,)) {
      findings.push(`duplicate-derivability-judgment (${judgment.reference})`,);
      continue;
    }
    if (!isDerivabilityVerdict(judgment.verdict,)) {
      findings.push(`unknown-derivability-verdict (${judgment.verdict})`,);
      continue;
    }
    verdicts.set(
      seedId,
      judgment.verdict,
    );
  }
  for (const [index, seedId,] of seedIds.entries()) {
    if (!verdicts.has(seedId,))
      findings.push(`missing-derivability-judgment (${index + 1})`,);
  }

  return {
    verdicts: Object.fromEntries(verdicts,),
    findings,
  };
}

//endregion Derivability probe wire format
