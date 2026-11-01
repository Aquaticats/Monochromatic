import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Editor wire format
// What editors actually emit: integer region references and replacement
// text. Resolution binds numbers to envelopes from the prompt plan and
// fails closed per item; everything else (staleness, drift, unchanged
// regions) is the apply gate's job.

/**
 One edit as an editor reports it.

 @example
 ```ts
 const wire: EditorEditWire = { region: 1, newText: 'The cat naps at noon.', };
 ```
 */
export type EditorEditWire = {
  /**
   One-based region number from the prompt sheet.
   */
  readonly region: number;

  /**
   Full replacement for exactly that region.
   */
  readonly newText: string;
};

/**
 Whole editor reply on the wire.

 @example
 ```ts
 const report: EditorReportWire = { edits: [], };
 ```
 */
export type EditorReportWire = {
  /**
   Every edit proposed; empty means the editor fixed nothing.
   */
  readonly edits: readonly EditorEditWire[];
};

/**
 Guards one wire edit.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required edit fields

 @example
 ```ts
 isEditorEditWire({ region: 1, newText: 'text', },);
 ```
 */
function isEditorEditWire(value: unknown,): value is EditorEditWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Region reference as reported; integerness checked on the primitive copy.
   */
  const { region, } = value;
  if ((typeof region) !== 'number')
    return false;
  if ((region % 1) !== 0)
    return false;
  return (typeof value.newText) === 'string';
}

/**
 Guards a whole editor reply.

 @param value - parsed model JSON

 @returns Whether value is a wire report

 @example
 ```ts
 const outcome = await client.chatJson({ ..., validate: isEditorReportWire, },);
 ```
 */
export function isEditorReportWire(value: unknown,): value is EditorReportWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.edits,))
    return false;
  return value.edits
    .every(function eachEdit(edit,) {
      return isEditorEditWire(edit,);
    },);
}

/**
 Structured-output constraint for editor calls;
 client-side validation through {@link isEditorReportWire} stays
 regardless, because per-model schema strictness is unverified.
 */
export const EDITOR_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'editor_report',
    schema: {
      type: 'object',
      required: ['edits',],
      additionalProperties: false,
      properties: {
        edits: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'newText',
              'region',
            ],
            additionalProperties: false,
            // DECLARED IN THE ORDER THE KEYS SORT (ledger B762), since no ruling fixes the order of the two.
            properties: {
              newText: { type: 'string', },
              region: { type: 'integer', },
            },
          },
        },
      },
    },
  },
};

/**
 Edits resolved into patch operations plus wire irregularities.

 @example
 ```ts
 const { operations, findings, } = resolveEditorEdits({ wire, envelopes, },);
 ```
 */
export type EditorEditResolution = {
  /**
   Operations ready for the apply gate, in wire order.
   */
  readonly operations: readonly PatchOperation[];

  /**
   Wire irregularities in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Resolves wire edits into patch operations through the prompt plan.
 Fails closed per item: out-of-range and duplicate region references
 become findings and no operation. The envelope's own base hash rides
 onto the operation because the plan that numbered the regions is the
 plan that cut the envelopes; the apply gate still re-proves the region
 against the document.

 @param wire - reply as the editor reported it

 @param envelopes - envelopes in prompt numbering order

 @returns Operations plus findings as data

 @example
 ```ts
 const resolution = resolveEditorEdits({ wire, envelopes, },);
 ```
 */
export function resolveEditorEdits(
  {
    wire,
    envelopes,
  }: {
    readonly wire: EditorReportWire;
    readonly envelopes: readonly EditableEnvelope[];
  },
): EditorEditResolution {
  /**
   Findings accumulated across every wire item.
   */
  const findings: string[] = [];

  /**
   Region numbers already resolved; first occurrence wins.
   */
  const seen = new Set<number>();

  /**
   Operations in wire order.
   */
  const operations: PatchOperation[] = [];
  for (const edit of wire.edits) {
    /**
     Envelope referenced by this edit's one-based number.
     */
    const envelope = envelopes[edit.region - 1];
    if ((edit.region < 1) || (envelope === undefined)) {
      findings.push(`edit-region-out-of-range (${edit.region})`,);
      continue;
    }
    if (seen.has(edit.region,)) {
      findings.push(`duplicate-edit (${edit.region})`,);
      continue;
    }
    seen.add(edit.region,);

    /**
     Replacement text with invisible variants folded, and what was folded.
     */
    const folded = foldInvisibleVariants({ text: edit.newText, },);
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

//endregion Editor wire format
