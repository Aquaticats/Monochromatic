//region SSE frame
// ONE SERVER-SENT EVENT FRAME CARRYING TEXT ON ONE CHANNEL, and two scripted
// per-index builders producing varying cat-themed sentences, for streaming
// tests that need many distinguishable frames.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several stream tests kept their own copy
// of these builders; all now import them from here.

/**
 Builds one server-sent event frame carrying text on one channel.

 @param channel - which channel the text arrives on

 @param text - text the frame carries

 @returns Frame as the wire sends it

 @example
 ```ts
 const raw = frameOf({ channel: 'reasoning', text: 'I will output. ', },);
 ```
 */
export function frameOf(
  {
    channel,
    text,
  }: {
    readonly channel: 'content' | 'reasoning';
    readonly text: string;
  },
): string {
  /**
   Delta object, whose field name distinguishes the channels.
   */
  const delta = (channel === 'content') ? { content: text, } : { reasoning_content: text, };

  return `data: ${
    JSON.stringify({
      choices: [{
        index: 0,
        delta,
        finish_reason: null,
      },],
    },)
  }\n\n`;
}

/**
 Minutes a content frame's note wraps at, so neighbouring sentences differ in
 more than their leading index.
 */
const MINUTES_PER_HOUR = 60;

/**
 Shelves between consecutive reasoning frames' options.
 */
const SHELF_STRIDE = 3;

/**
 Hours a reasoning frame's clock wraps at.
 */
const HOURS_PER_DAY = 24;

/**
 Builds a scripted content frame naming the given index, about an entirely
 separate cat each time.

 @param at - frame index

 @returns Frame as the wire sends it

 @example
 ```ts
 const raw = answerFrame({ at: 2, },);
 ```
 */
function answerFrame({ at, }: { readonly at: number; },): string {
  return frameOf({
    channel: 'content',
    text: `Sentence ${String(at,)} concerning an entirely separate cat, noted at ${String(at % MINUTES_PER_HOUR,)}. `,
  },);
}

/**
 Builds a scripted reasoning frame naming the given index.

 @param at - frame index

 @returns Frame as the wire sends it

 @example
 ```ts
 const raw = thinkFrame({ at: 2, },);
 ```
 */
function thinkFrame({ at, }: { readonly at: number; },): string {
  return frameOf({
    channel: 'reasoning',
    text: `Weighing option ${String(at,)} for shelf ${String(at * SHELF_STRIDE,)} at hour ${
      String(at % HOURS_PER_DAY,)
    }. `,
  },);
}

/**
 Reasoning frames in the long varied stream.
 */
const LONG_THINKING_FRAMES = 6_000;

/**
 Content frames in the long varied stream.
 */
const LONG_ANSWER_FRAMES = 120;

/**
 A healthy long call: long, varied thinking, then a long, varied answer,
 with no sentence repeated, which no runaway or repetition guard may end.

 @returns The stream as the wire sends it

 @example
 ```ts
 const raw = longVariedStream();
 ```
 */
export function longVariedStream(): string {
  /**
   Every reasoning frame, in order.
   */
  const thinking = Array.from(
    { length: LONG_THINKING_FRAMES, },
    function thinkAt(
      _unused,
      at,
    ): string {
      return thinkFrame({ at, },);
    },
  );

  /**
   Every content frame, in order.
   */
  const answering = Array.from(
    { length: LONG_ANSWER_FRAMES, },
    function answerAt(
      _unused,
      at,
    ): string {
      return answerFrame({ at, },);
    },
  );
  return [
    ...thinking,
    ...answering,
  ].join('',);
}

//endregion SSE frame
