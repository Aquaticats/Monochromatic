import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { INTRODUCED_DEFECT_VERDICTS, } from './introduced-defect-wire.ts';

//region Introduced-defect probe format
// The structured-output constraint of the prober call, split from `introduced-defect-wire.ts` at its line budget.

/**
 Structured-output constraint for prober calls;
 client-side validation through `isIntroducedDefectReportWire` stays
 regardless, because per-model schema strictness is unverified.

 Every field is required rather than optional, and negative verdicts carry
 empty strings, because optional properties are where per-model structured
 output diverges most and a lost voice costs a whole region's telemetry.
 */
export const INTRODUCED_DEFECT_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'introduced_defect_report',
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
              'category',
              'evidence',
              'omittedText',
              'region',
              'severity',
              'verdict',
              'verdictReason',
            ],
            additionalProperties: false,
            // DECLARED IN THE ORDER THE KEYS SORT (ledger B762), with the reason last by the owner's ruling of
            // 2026-10-21 (ledger B732): `verdictReason` sorts after `verdict`.
            properties: {
              category: { type: 'string', },
              evidence: { type: 'string', },
              omittedText: { type: 'string', },
              region: { type: 'integer', },
              severity: { type: 'string', },
              // THE CLOSED VOCABULARY `isIntroducedDefectVerdict` reads, as an enum (P1 of the second review of the Claude
              // route). `category` and `severity` stay plain strings: a negative verdict sends both as empty strings
              // and nothing reads either as a vocabulary.
              verdict: {
                type: 'string',
                enum: [...INTRODUCED_DEFECT_VERDICTS,],
              },
              verdictReason: { type: 'string', },
            },
          },
        },
      },
    },
  },
};

//endregion Introduced-defect probe format
