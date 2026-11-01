import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Translate repair wire
// The follow-up turn a candidate's own author gets when structural validation
// found something.
//
// User decision, 2026-08-15: an invalid candidate is not dropped. "The pipeline
// should try fixing it by giving the findings to the original model in the same
// chat, and the original model can say it can fix it, it can't fix it, or for
// whatever reason the 'broken' candidate it produced is the best possible
// version for the information it has."
//
// The third answer is the one a filter could never have collected. A model that
// dropped a footnote marker because the definition it points at is not in this
// slice is reporting a SLICING defect, and a validator that silently dropped
// the candidate would have destroyed the only report of it.

/**
 Every answer a model may give when handed its own candidate's findings, for
 the schema and the guard to share one list.

 THE TYPE IS DERIVED FROM THIS LIST, so an answer cannot be added to the type
 without being listed here, and the schema and the guard cannot fall behind it.
 */
const REPAIR_RESOLUTIONS = [
  'revised',
  'unable',
  'as-intended',
] as const;

/**
 Answers a model may give when handed its own candidate's findings.

 @example
 ```ts
 const resolution: RepairResolution = 'as-intended';
 ```
 */
export type RepairResolution = typeof REPAIR_RESOLUTIONS[number];

/**
 One repair reply on the wire.

 Flat rather than a discriminated union of shapes, because every stage in this
 pipeline sends flat schemas and the providers handle them reliably. The union
 is enforced by {@link isTranslateRepairWire} instead: a `revised` reply
 carrying no translation is not a revision.

 @example
 ```ts
 const wire: TranslateRepairWire = {
   resolution: 'unable',
   translation: '',
   translatorNote: 'the footnote definition is not in this passage',
 };
 ```
 */
export type TranslateRepairWire = {
  /**
   What the model decided to do about the findings.
   */
  readonly resolution: RepairResolution;

  /**
   Replacement translation, empty unless the resolution is `revised`.
   */
  readonly translation: string;

  /**
   Why, in the model's own words; recorded whatever the resolution.

   NAMED `translatorNote` so that it sorts after `translation`, which the owner's ruling of 2026-10-21 asks of the
   explanation (it follows the text it explains; ledger B732, B762).
   */
  readonly translatorNote: string;
};

/**
 Guards a repair reply.

 @param value - parsed model JSON

 @returns Whether value is a well-formed repair reply

 @example
 ```ts
 const ok = isTranslateRepairWire(JSON.parse(text,),);
 ```
 */
export function isTranslateRepairWire(value: unknown,): value is TranslateRepairWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Fields as the model sent them.
   */
  const {
    resolution,
    translation,
    translatorNote,
  } = value;
  if ((typeof translation) !== 'string')
    return false;
  if ((typeof translatorNote) !== 'string')
    return false;
  if ((typeof resolution) !== 'string')
    return false;

  /**
   Whether the resolution names one of the three answers.
   */
  const named = REPAIR_RESOLUTIONS.some(function matches(allowed,): boolean {
    return allowed === resolution;
  },);
  if (!named)
    return false;

  // A revision with nothing in it is not a revision, and admitting one would
  // put an empty candidate on the ballot in the name of repairing it. Nothing
  // is what shows a reader nothing (ledger B40), which a zero-width space
  // passed when this asked `trim()`.
  if (resolution !== 'revised')
    return true;
  return !rendersAsNothing({ text: translation, },);
}

/**
 Instructions for the follow-up turn.

 Says the checks are MECHANICAL on purpose. A model told only that its work is
 wrong tends to rewrite whatever it can see; told that a specific structural
 comparison produced these lines, it can answer that the comparison is the
 thing at fault, which is an answer worth having.
 */
const REPAIR_RULES =
  `The translation you just produced went through a mechanical structural check, not a reader. The check compares Markdown block structure, footnote markers, link and image destinations, and inline code against the ORIGINAL and against the PAGE AS IT STANDS, and every finding names which of the two it came from. Where the two carry different destinations for one reference, the check accepts either and refuses both. It knows nothing about wording, and it can be wrong about what this passage needed.

Answer in one of exactly three ways:

- "revised": you can fix every finding without changing anything else about your translation. Put the whole corrected translation in "translation".
- "unable": you cannot fix them, and say why in "translatorNote". Your earlier translation still stands as a candidate.
- "as-intended": your translation is right as it is and the finding is a fact about the passage or the check rather than about your work. Say why in "translatorNote". A footnote marker whose definition is not in this passage is the usual case.

Do not rewrite for style, and do not translate anything you were not given.

Reply with ONLY a JSON object of shape {"resolution": "...", "translation": "...", "translatorNote": "..."}. Use an empty string for "translation" unless the resolution is "revised". No prose, no code fences, no commentary.`;

/**
 Builds the follow-up turn for one candidate's author.

 @param priorMessages - exact messages that produced the candidate

 @param priorTranslation - candidate this model returned

 @param findings - structural divergences, written for the model

 @returns Messages continuing that same exchange

 @example
 ```ts
 const messages = buildTranslateRepairMessages({ priorMessages, priorTranslation, findings, },);
 ```
 */
export function buildTranslateRepairMessages(
  {
    priorMessages,
    priorTranslation,
    findings,
  }: {
    readonly priorMessages: readonly ChatMessage[];
    readonly priorTranslation: string;
    readonly findings: readonly string[];
  },
): readonly ChatMessage[] {
  /**
   Findings as a list the model reads rather than a sentence it skims.
   */
  const listed = findings
    .map(function toLine(finding,): string {
      return `- ${finding}`;
    },)
    .join('\n',);

  return [
    ...priorMessages,
    // The model's own turn, reconstructed from the validated reply rather than
    // replayed byte for byte: `StageVoice` keeps the parsed value and not the
    // raw text. What differs is JSON whitespace, which is exactly what the
    // response format asked to be free of anyway.
    {
      role: 'assistant',
      content: JSON.stringify({ translation: priorTranslation, },),
    },
    {
      role: 'user',
      content: `${REPAIR_RULES}

FINDINGS
${listed}`,
    },
  ];
}

/**
 Structured-output constraint for repair replies.
 */
export const TRANSLATE_REPAIR_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'translation_repair_report',
    schema: {
      type: 'object',
      required: [
        'resolution',
        'translation',
        'translatorNote',
      ],
      additionalProperties: false,
      // DECLARED IN THE ORDER THE KEYS SORT (ledger B762): the note follows the translation it explains.
      properties: {
        resolution: {
          type: 'string',
          enum: [...REPAIR_RESOLUTIONS,],
        },
        translation: { type: 'string', },
        translatorNote: { type: 'string', },
      },
    },
  },
};

//endregion Translate repair wire
