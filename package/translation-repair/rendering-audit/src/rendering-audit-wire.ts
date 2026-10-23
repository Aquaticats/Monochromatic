import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Rendering audit wire
// Asks whether one candidate rendering says what its original says, with NO
// before-text to compare against.
//
// WHY A SECOND INSTRUMENT RATHER THAN A WIDER FIRST ONE. The introduced-defect
// probe asks a DIFFERENTIAL question: does this replacement carry a defect the
// baseline did not have. That question needs a baseline, and a slice the
// translate lane rendered from scratch has none: there is no earlier wording of
// it that anybody produced. Widening that probe would have meant passing it an
// empty baseline and reading every defect as introduced, which is exactly the
// reading the differential form exists to avoid.
//
// SO THE QUESTION HERE IS ABSOLUTE: against the original, and against the
// identity evidence this run licensed, does the candidate fail to render or
// misrepresent anything. Nothing about improvement, nothing about the archive.
//
// THE ARCHIVE IS EXCLUDED AT THE CALLER, which is a narrower claim than an
// earlier version of this comment made. What is true: no archive text is ever
// serialized into these messages, and archive wording cannot anchor a quote,
// since a quote naming the original is searched in the ORIGINAL and nowhere
// else. What is NOT true, and was written here as though it were: that this
// makes a voice reason independently of an archive it may have seen elsewhere.
// Nothing here can establish that. The `reason` field is not screened at all,
// and a candidate-only finding needs no original quote.
//
// EVERY CATEGORY NAMES WHICH SIDE IT MUST ANCHOR IN, which is an evidence
// obligation rather than a proof. Content the candidate never rendered has
// nothing in the candidate to quote, its absence being the whole claim, so
// demanding a candidate span for an omission would make it unfileable. An
// addition nothing supports is the mirror. Everything else changes something
// both sides state, and must anchor in both. An anchored source span proves
// that the original says that; whether the candidate fails to render it is what
// the auditor claims and what corroboration weighs.

/**
 Every verdict one auditor may cast on one candidate, closed vocabulary.

 NO `faithful`, for the same reason the introduced-defect vocabulary has no
 `clean`: a voice reports what it found, and finding nothing is not proof of
 fidelity. `no-defect-found` says exactly that much and no more.

 @example
 ```ts
 RENDERING_AUDIT_VERDICTS.includes('defects-found',);
 ```
 */
export const RENDERING_AUDIT_VERDICTS = [
  'defects-found',
  'no-defect-found',
  'uncertain',
] as const;

/**
 One verdict, narrowed to the vocabulary.

 @example
 ```ts
 const verdict: RenderingAuditVerdict = 'no-defect-found';
 ```
 */
export type RenderingAuditVerdict = typeof RENDERING_AUDIT_VERDICTS[number];

/**
 Categories provable from the ORIGINAL alone, because the candidate holds
 nothing to quote.

 @example
 ```ts
 SOURCE_ONLY_CATEGORIES.includes('omission',);
 ```
 */
export const SOURCE_ONLY_CATEGORIES = ['omission',] as const;

/**
 Categories provable from the CANDIDATE alone, because the original holds
 nothing to quote.

 @example
 ```ts
 CANDIDATE_ONLY_CATEGORIES.includes('unsupported-addition',);
 ```
 */
export const CANDIDATE_ONLY_CATEGORIES = ['unsupported-addition',] as const;

/**
 Categories that change something both sides state, and must quote both.

 @example
 ```ts
 PAIRED_CATEGORIES.includes('altered-polarity',);
 ```
 */
export const PAIRED_CATEGORIES = [
  'altered-actor',
  'altered-referent',
  'altered-polarity',
  'altered-modality',
  'altered-time',
  'altered-number',
  'altered-relation',
  'altered-identity',
  'broken-structure',
] as const;

/**
 Every category an auditor may name, closed vocabulary.

 @example
 ```ts
 RENDERING_AUDIT_CATEGORIES.includes('omission',);
 ```
 */
export const RENDERING_AUDIT_CATEGORIES: readonly (
  | typeof SOURCE_ONLY_CATEGORIES[number]
    | typeof CANDIDATE_ONLY_CATEGORIES[number]
    | typeof PAIRED_CATEGORIES[number]
)[] = [
  ...SOURCE_ONLY_CATEGORIES,
  ...CANDIDATE_ONLY_CATEGORIES,
  ...PAIRED_CATEGORIES,
];

/**
 One category, narrowed to the vocabulary.

 @example
 ```ts
 const category: RenderingAuditCategory = 'omission';
 ```
 */
export type RenderingAuditCategory = typeof RENDERING_AUDIT_CATEGORIES[number];

/**
 One claimed defect as an auditor sent it, before any screening.

 FOUR QUOTES, TWO PER SIDE, because one span cannot both identify a place and
 name a change. A LOCATOR says which occurrence is meant and must be unique in
 its text; a FOCUS says what changed and need only be unique inside its own
 locator. The first version asked for one span per side and got both jobs done
 badly: a span wide enough to be unique was too wide to say what changed, so
 two different defects in one sentence arrived as the same quote.

 EVERY FIELD ALWAYS PRESENT ON THE WIRE, and empty where the category does not
 use that side. A schema whose required fields varied by category would ask a
 model to satisfy a conditional shape, which is the kind of instruction a
 model follows unevenly; the screen enforces the obligation instead, where it
 is deterministic, in BOTH directions: a missing required side is dropped, and
 so is a quote on a side the category forbids.

 @example
 ```ts
 const finding: RenderingAuditFindingWire = {
   category: 'omission',
   sourceLocator: '她们不吃罐头，每天傍晚只喝一碗温牛奶',
   sourceFocus: '不吃罐头',
   candidateLocator: '',
   candidateFocus: '',
   reason: 'the original denies it and the candidate states no counterpart',
 };
 ```
 */
export type RenderingAuditFindingWire = {
  /**
   What kind of defect this is, from the closed vocabulary.
   */
  readonly category: string;

  /**
   Span of the ORIGINAL identifying which occurrence this claim is about.
   */
  readonly sourceLocator: string;

  /**
   Smallest span of the ORIGINAL carrying the claimed change.
   */
  readonly sourceFocus: string;

  /**
   Span of the CANDIDATE identifying which occurrence this claim is about.
   */
  readonly candidateLocator: string;

  /**
   Smallest span of the CANDIDATE carrying the claimed change.
   */
  readonly candidateFocus: string;

  /**
   What the original asserts, what the candidate asserts, and why the
   difference is semantic.
   */
  readonly reason: string;
};

/**
 One auditor's whole answer about one candidate.

 @example
 ```ts
 const report: RenderingAuditReportWire = { verdict: 'no-defect-found', findings: [], };
 ```
 */
export type RenderingAuditReportWire = {
  /**
   What this voice concluded overall.
   */
  readonly verdict: string;

  /**
   Every defect it claims, empty when it claims none.
   */
  readonly findings: readonly RenderingAuditFindingWire[];
};

/**
 Every field a finding carries, named once so the guard and the schema cannot
 drift apart.

 LISTED IN ALPHABETICAL ORDER (ledger B762), because the schema takes its
 property order from this list and `source-scan` cannot see a schema built
 from a list: `rendering-audit-wire.unit.test.ts` pins the order instead.

 @example
 ```ts
 FINDING_FIELDS.includes('sourceFocus',);
 ```
 */
export const FINDING_FIELDS = [
  'candidateFocus',
  'candidateLocator',
  'category',
  'reason',
  'sourceFocus',
  'sourceLocator',
] as const;

/**
 Reads one wire finding out of an untyped value.

 @param value - one element of a reply's finding list

 @returns Whether it carries every field this wire requires, as strings

 @example
 ```ts
 const usable = isRenderingAuditFindingWire(value,);
 ```
 */
function isRenderingAuditFindingWire(value: unknown,): value is RenderingAuditFindingWire {
  if (!isJsonRecord(value,))
    return false;

  return FINDING_FIELDS.every(function carriesText(field,): boolean {
    return (typeof value[field]) === 'string';
  },);
}

/**
 Reads one auditor's reply out of an untyped value.

 SHAPE ONLY. Whether the words are ones this version knows, and whether the
 quotes prove anything, belong to the screen: a reply that is well shaped and
 unfounded is a different failure from one that did not parse, and reporting
 them as the same thing loses the voice-loss rate.

 @param value - parsed reply

 @returns Whether it is a report this wire can hand to the screen

 @example
 ```ts
 if (isRenderingAuditReportWire(parsed,)) screen({ report: parsed, },);
 ```
 */
export function isRenderingAuditReportWire(value: unknown,): value is RenderingAuditReportWire {
  if (!isJsonRecord(value,))
    return false;

  if ((typeof value.verdict) !== 'string')
    return false;

  if (!isJsonArray(value.findings,))
    return false;

  return value.findings
    .every(isRenderingAuditFindingWire,);
}

/**
 What one audit call is asked about.

 @example
 ```ts
 const subject: RenderingAuditSubject = { sourceText, candidateText, };
 ```
 */
export type RenderingAuditSubject = {
  /**
   Original passage, the only standard this audit has.
   */
  readonly sourceText: string;

  /**
   Rendering under audit.
   */
  readonly candidateText: string;

  /**
   Names and terms this run licensed, shown as evidence rather than as a rule.
   */
  readonly identityContext?: string;

  /**
   What the pages the ORIGINAL links say, as the producing critics and panels
   read them (ledger B29); empty or absent where it links nowhere.
   */
  readonly referenceContext?: string;
};

/**
 Response format one audit call asks for.

 @example
 ```ts
 const format = RENDERING_AUDIT_RESPONSE_FORMAT;
 ```
 */
export const RENDERING_AUDIT_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'rendering_audit_report',
    schema: {
      type: 'object',
      required: [
        'findings',
        'verdict',
      ],
      additionalProperties: false,
      // DECLARED IN THE ORDER THE KEYS SORT (ledger B762): the findings, each of which carries its reason, come before
      // the verdict they support. The owner's ruling of 2026-10-21 leaves this audit outside "Reason first everywhere"
      // and fixes no order of the two.
      properties: {
        findings: {
          type: 'array',
          items: {
            type: 'object',
            required: [...FINDING_FIELDS,],
            additionalProperties: false,
            // THE CATEGORY IS THE CLOSED VOCABULARY THE SCREEN FINDS IT IN, as an enum (P1 of the second review of the
            // Claude route); every other field is the text the screen quotes.
            properties: Object.fromEntries(FINDING_FIELDS.map(function toStringField(field,): readonly [
              string,
              {
                readonly type: 'string';
                readonly enum?: readonly string[];
              },
            ] {
              return [
                field,
                (field === 'category')
                  ? {
                    type: 'string',
                    enum: [...RENDERING_AUDIT_CATEGORIES,],
                  }
                  : { type: 'string', },
              ];
            },),),
          },
        },
        // THE VERDICT IS THE CLOSED VOCABULARY THE SCREEN READS; a verdict outside it would read as `uncertain`.
        verdict: {
          type: 'string',
          enum: [...RENDERING_AUDIT_VERDICTS,],
        },
      },
    },
  },
};

//endregion Rendering audit wire
