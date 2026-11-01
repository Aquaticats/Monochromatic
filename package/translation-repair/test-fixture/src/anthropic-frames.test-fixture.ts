//region Anthropic frames
// BUILDS ANTHROPIC MESSAGES STREAM FRAMES AS THE WIRE SENDS THEM: one event
// line naming the frame's own `type`, one data line holding the JSON payload,
// and the blank line ending the frame.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The anthropic-delta-scan test kept its own
// copy of these builders, and the stream-drain test began a second; both now
// import them from here.

/**
 Builds one Anthropic event frame, newline-terminated as the wire sends it.

 @param body - frame payload, which carries its own `type`

 @returns Frame ready to feed a scanner

 @example
 ```ts
 const raw = anthropicFrameOf({ body: { type: 'ping', }, },);
 ```
 */
export function anthropicFrameOf(
  { body, }: { readonly body: Readonly<Record<string, unknown>>; },
): string {
  /**
   Event name, which every Anthropic frame repeats inside its own payload.
   */
  const { type, } = body;

  return `event: ${String(type,)}\ndata: ${JSON.stringify(body,)}\n\n`;
}

/**
 Frame opening a content block of one type at one index.

 @param index - position the block occupies

 @param type - block type the server declares

 @returns Frame ready to feed a scanner

 @example
 ```ts
 const raw = anthropicBlockStart({ index: 0, type: 'thinking', },);
 ```
 */
export function anthropicBlockStart(
  {
    index,
    type,
  }: {
    readonly index: number;
    readonly type: string;
  },
): string {
  return anthropicFrameOf({
    body: {
      type: 'content_block_start',
      index,
      content_block: { type, },
    },
  },);
}

/**
 Frame carrying one delta of one kind at one index.

 @param index - position the delta belongs to

 @param deltaType - kind of delta, which names its text field

 @param field - field the text rides in

 @param text - text the frame carries

 @returns Frame ready to feed a scanner

 @example
 ```ts
 const raw = anthropicBlockDelta({ index: 0, deltaType: 'text_delta', field: 'text', text: 'Biscuit', },);
 ```
 */
export function anthropicBlockDelta(
  {
    index,
    deltaType,
    field,
    text,
  }: {
    readonly index: number;
    readonly deltaType: string;
    readonly field: string;
    readonly text: string;
  },
): string {
  return anthropicFrameOf({
    body: {
      type: 'content_block_delta',
      index,
      delta: {
        type: deltaType,
        [field]: text,
      },
    },
  },);
}

//endregion Anthropic frames
