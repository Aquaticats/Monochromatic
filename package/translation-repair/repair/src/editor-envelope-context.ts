import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

/**
 Characters of translation shown on each side of an envelope so judges can
 assess register and tense against real neighbouring prose.
 */
const ENVELOPE_CONTEXT_CHARS = 400;

/**
 Renders the translation around one envelope, with the region under
 replacement marked rather than removed.

 Judges are asked whether a replacement fits its surroundings in register and
 tense. Handed only the replacement text and the Chinese source, that
 criterion is unanswerable: the surroundings are exactly what is missing. The
 window is bounded because whole chunks run to thousands of characters and
 every judge pays for them on every envelope.

 @param targetText - translation chunk text

 @param envelope - region being replaced

 @returns Bounded window with the replaced region marked

 @example
 ```ts
 const context = envelopeContext({ targetText, envelope, },);
 ```
 */
export function envelopeContext(
  {
    targetText,
    envelope,
  }: {
    readonly targetText: string;
    readonly envelope: EditableEnvelope;
  },
): string {
  /**
   Translation before the envelope, bounded to the context window.
   */
  const before = targetText.slice(
    Math.max(
      0,
      envelope.startOffset - ENVELOPE_CONTEXT_CHARS,
    ),
    envelope.startOffset,
  );

  /**
   Translation after the envelope, bounded to the context window.
   */
  const after = targetText.slice(
    envelope.endOffset,
    envelope.endOffset + ENVELOPE_CONTEXT_CHARS,
  );
  return `${before}[[PASSAGE BEING REPLACED]]${after}`;
}
