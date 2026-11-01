import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Refinement wire format
// One rewriter call per slice returns zero or more paragraph rewrites, each
// naming a paragraph by its prompt number.
//
// Batched per slice rather than one call per paragraph, and that is a
// correctness choice as much as a wall-clock one: paragraphs rewritten in
// separate calls are chosen against each other by nobody, so the assembled
// slice reads as stitched fragments. One call sees the whole slice, and the
// judges then rank whole slices.
//
// Returning FEWER rewrites than there are paragraphs is the expected outcome,
// not a degraded one. Nothing in this lane claims the text is wrong, so a
// rewriter with nothing worth changing should say so by staying silent about
// that paragraph.

/**
 One proposed paragraph rewrite on the wire.

 @example
 ```ts
 const rewrite: RefineRewriteWire = { paragraph: 2, newText: 'She wrote it at seventeen.', };
 ```
 */
export type RefineRewriteWire = {
  /**
   One-based paragraph number from the prompt sheet.
   */
  readonly paragraph: number;

  /**
   Full replacement for exactly that paragraph.
   */
  readonly newText: string;
};

/**
 Whole rewriter reply on the wire.

 @example
 ```ts
 const report: RefineReportWire = { rewrites: [], };
 ```
 */
export type RefineReportWire = {
  /**
   Every rewrite proposed; empty means nothing was worth changing.
   */
  readonly rewrites: readonly RefineRewriteWire[];
};

/**
 Guards one wire rewrite.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required rewrite fields

 @example
 ```ts
 isRefineRewriteWire({ paragraph: 1, newText: 'text', },);
 ```
 */
function isRefineRewriteWire(value: unknown,): value is RefineRewriteWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Fields the guard checks.
   */
  const {
    paragraph,
    newText,
  } = value;
  return ((typeof paragraph) === 'number')
    && Number.isInteger(paragraph,)
    && (paragraph > 0)
    && ((typeof newText) === 'string');
}

/**
 Guards a whole rewriter reply.

 @param value - candidate from parsed model JSON

 @returns Whether value is a well-formed report

 @example
 ```ts
 isRefineReportWire({ rewrites: [], },);
 ```
 */
export function isRefineReportWire(value: unknown,): value is RefineReportWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Proposed rewrites as the model sent them.
   */
  const { rewrites, } = value;
  return Array.isArray(rewrites,) && rewrites.every(isRefineRewriteWire,);
}

/**
 Structured-output constraint for a rewriter reply.
 */
export const REFINE_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'refine_report',
    schema: {
      type: 'object',
      required: ['rewrites',],
      additionalProperties: false,
      properties: {
        rewrites: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'newText',
              'paragraph',
            ],
            additionalProperties: false,
            // DECLARED IN THE ORDER THE KEYS SORT (ledger B762), since no ruling fixes the order of the two.
            properties: {
              newText: { type: 'string', },
              paragraph: { type: 'integer', },
            },
          },
        },
      },
    },
  },
};

/**
 Operations bound to paragraphs, plus what the wire got wrong.

 @example
 ```ts
 const { operations, findings, } = resolveRefineRewrites({ wire, envelopes, },);
 ```
 */
export type RefineResolution = {
  /**
   Operations in wire order, each bound to a real paragraph.
   */
  readonly operations: readonly PatchOperation[];

  /**
   Wire irregularities in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Binds wire rewrites to the paragraphs they name.

 A rewrite naming a paragraph outside the sheet, or naming one already
 rewritten, is recorded and dropped rather than throwing: a rewriter
 miscounting its own list says nothing about the paragraphs it got right.

 @param wire - reply as the rewriter reported it

 @param envelopes - eligible paragraphs in prompt numbering order

 @returns Operations plus findings as data

 @example
 ```ts
 const resolution = resolveRefineRewrites({ wire, envelopes, },);
 ```
 */
export function resolveRefineRewrites(
  {
    wire,
    envelopes,
  }: {
    readonly wire: RefineReportWire;
    readonly envelopes: readonly EditableEnvelope[];
  },
): RefineResolution {
  /**
   Findings accumulated across every wire item.
   */
  const findings: string[] = [];

  /**
   Paragraph numbers already bound; first occurrence wins.
   */
  const seen = new Set<number>();

  /**
   Operations in wire order.
   */
  const operations: PatchOperation[] = [];
  for (const rewrite of wire.rewrites) {
    /**
     Paragraph this rewrite's one-based number names.
     */
    const envelope = envelopes[rewrite.paragraph - 1];
    if (envelope === undefined) {
      findings.push(`refine-unknown-paragraph (${String(rewrite.paragraph,)})`,);
      continue;
    }
    if (seen.has(rewrite.paragraph,)) {
      findings.push(`refine-duplicate-paragraph (${String(rewrite.paragraph,)})`,);
      continue;
    }
    seen.add(rewrite.paragraph,);

    /**
     Rewrite with invisible variants folded, and what was folded.
     */
    const folded = foldInvisibleVariants({ text: rewrite.newText, },);
    findings.push(...folded.findings,);
    operations.push({
      envelopeId: envelope.envelopeId,
      baseHash: envelope.baseHash,
      newText: folded.text,
    },);
  }
  return {
    operations,
    findings,
  };
}

//endregion Refinement wire format
