/**
 Reads the assistant prose of pi messages:
 thinking and visible text blocks alike,
 since the detector measures overall dismissals.

 @module
 */

/**
 One content block of a pi message.
 */
type MessageBlock = {
  /**
   Block kind: `thinking`, `text`, or `reasoning`.
   */
  readonly type?: string;

  /**
   Rendered text of a text or reasoning block.
   */
  readonly text?: string;

  /**
   Reasoning text of a thinking block.
   */
  readonly thinking?: string;
};

/**
 Reads the assistant prose of one pi message:
 thinking and visible text blocks alike,
 since the detector measures overall dismissals.

 @param message - Finalized message of a `message_end` event

 @returns Text units of the message

 @example
 ```ts
 unitsOfMessage({ content: [{ type: 'text', text: 'done' }] });
 ```
 */
function unitsOfMessage(message: unknown,): readonly string[] {
  /**
   Message narrowed to its content carrier.
   */
  /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing a pi message to its content carrier */
  const typed = message as {
    readonly content?: unknown;
  };
  /**
   Message content, an array of blocks for assistant turns.
   */
  const { content, } = typed;
  if (!Array.isArray(content)) return [];
  /**
   Text units collected in block order.
   */
  const units: string[] = [];
  for (const item of content as readonly MessageBlock[]) {
    /**
     Block kind and its text carriers, read once.
     */
    const {
      type: kind,
      text,
      thinking,
    } = item;
    /**
     True when the block carries reasoning text.
     */
    const hasThinking = kind === 'thinking';
    if (hasThinking) {
      /**
       Reasoning text of the block, empty when absent.
       */
      const reasoning = thinking ?? '';
      /**
       True when the reasoning text is non-empty.
       */
      const hasReasoning = reasoning !== '';
      if (hasReasoning) units.push(reasoning,);
    }
    /**
     True when the block carries prose text.
     */
    const hasProse = (kind === 'text') || (kind === 'reasoning');
    if (hasProse) {
      /**
       Visible text of the block, empty when absent.
       */
      const prose = text ?? '';
      /**
       True when the prose text is non-empty.
       */
      const hasProseText = prose !== '';
      if (hasProseText) units.push(prose,);
    }
  }
  return units;
}


export { unitsOfMessage, };
