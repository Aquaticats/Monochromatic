/**
 Tests that only generated text, content or reasoning, counts as progress for
 the first-content and idle windows of a stream, and that a cut names the
 provider, the model, the raw characters and the generated characters.

 WHY: in `zheermao22` eleven `inception/mercury-2.5` streams sent bytes and
 no text. OpenRouter writes the comment line `: OPENROUTER PROCESSING` while it
 waits for the upstream provider (`openrouter-client.unit.test.ts`), the idle
 guard reset its window on any body bytes, and each stream was cut only by the
 120 s straggler grace after quorum. Every raw count was a multiple of 25, the
 length of that comment line and its two newlines (ledger B804).

 WHAT THESE PIN: comment lines, blank keep-alive lines and events with an empty
 delta do not reset either window; a content delta and a reasoning delta do;
 and the log line of a cut carries the provider, the model, the raw characters
 and the content and reasoning characters.

 Fixtures are cat-themed invention. Windows are tiny so a case ends in
 milliseconds.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  armIdleGuard,
  drainBody,
  StreamCutShortError,
  StreamStalledError,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { frameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/sse-frame.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Window short enough to trip inside a case, long enough that a loaded machine
 does not trip it between two chunks that arrive every {@link GAP_MS}.
 */
const WINDOW_MS = 250;

/**
 Pause between two chunks of a stream that keeps sending, a fraction of the
 window.
 */
const GAP_MS = 8;

/**
 Chunks a keep-alive stream sends, which at {@link GAP_MS} run well past the
 window.
 */
const KEEP_ALIVES = 80;

/**
 The comment line OpenRouter sends while the upstream provider has not answered,
 with the blank line that ends the event.
 */
const PROCESSING = ': OPENROUTER PROCESSING\n\n';

/**
 A frame whose delta carries no text, as a gateway sends before the first word.
 */
const EMPTY_DELTA = `data: ${
  JSON.stringify({ choices: [{ index: 0, delta: { content: '', }, finish_reason: null, },], },)
}\n\n`;

/**
 A frame that only names the role of the speaker.
 */
const ROLE_ONLY = `data: ${
  JSON.stringify({ choices: [{ index: 0, delta: { role: 'assistant', }, finish_reason: null, },], },)
}\n\n`;

/**
 Builds a response whose body sends the chunks one per {@link GAP_MS} and then
 ends, and errors with the guard's reason when the guard aborts, as a real
 request does when the guard tears it down.

 @param guard - guard whose abort ends the body

 @param chunks - chunks to send in order

 @returns Response carrying that body

 @example
 ```ts
 const response = timedResponse({ guard, chunks: [PROCESSING,], },);
 ```
 */
function timedResponse(
  {
    guard,
    chunks,
  }: {
    readonly guard: ReturnType<typeof armIdleGuard>;
    readonly chunks: readonly string[];
  },
): Response {
  /**
   Encoder, since a body carries bytes.
   */
  const encoder = new TextEncoder();
  return new Response(
    new ReadableStream<Uint8Array>({
      async start(controller,): Promise<void> {
        guard.signal
          .addEventListener(
          'abort',
          function tearDown() {
            controller.error(guard.signal.reason,);
          },
          { once: true, },
          );
        for (const chunk of chunks) {
          if (guard.signal.aborted)
            return;
          controller.enqueue(encoder.encode(chunk,),);
          // oxlint-disable-next-line no-await-in-loop -- the pause between two chunks is the case
          await wait(GAP_MS,);
        }
        if (!guard.signal.aborted)
          controller.close();
      },
    },),
  );
}

/**
 Windows and chunks of one timed body.
 */
type TimedBody = {
  /**
   First-content window of the guard.
   */
  readonly firstContentMs: number;

  /**
   Idle window of the guard.
   */
  readonly idleMs: number;

  /**
   Chunks the body sends.
   */
  readonly chunks: readonly string[];
};

/**
 Drains one timed body under a guard with the given windows.

 @param body - windows and chunks of the body

 @returns Drained body text

 @throws {@link StreamCutShortError} when the guard cut the stream

 @example
 ```ts
 const text = await drainTimed({ firstContentMs: 50, idleMs: 50, chunks, },);
 ```
 */
async function drainTimed(
  {
    firstContentMs,
    idleMs,
    chunks,
  }: TimedBody,
): Promise<string> {
  using guard = armIdleGuard({
    label: 'inception/mercury-2.5',
    provider: 'openrouter',
    firstContentMs,
    idleMs,
  },);
  return await drainBody({
    credentials: [],
    response: timedResponse({ guard, chunks, },),
    guard,
    label: 'inception/mercury-2.5',
    callerSignal: new AbortController().signal,
  },);
}

/**
 Drains one timed body that the guard is expected to cut and returns the cut.

 @param body - windows and chunks of the body

 @returns Cut the drain raised

 @throws {@link Error} when the body ended whole, which is the failure under test

 @example
 ```ts
 const cut = await cutOf({ firstContentMs: 50, idleMs: 50, chunks, },);
 ```
 */
async function cutOf(body: TimedBody,): Promise<StreamCutShortError> {
  try {
    await drainTimed(body,);
  }
  catch (error) {
    if (error instanceof StreamCutShortError)
      return error;
    throw error;
  }
  throw new Error('the body ended whole, so no cut was raised',);
}

/**
 Message of the stall a cut carries as its cause.

 @param cut - cut whose cause is read

 @returns Stall's message

 @throws {@link Error} when the cause is not a stall

 @example
 ```ts
 const message = stallMessageOf({ cut, },);
 ```
 */
function stallMessageOf({ cut, }: { readonly cut: StreamCutShortError; },): string {
  if (!(cut.cause instanceof StreamStalledError))
    throw new Error('the cut was not caused by a stall',);
  return cut.cause.message;
}

await describe({
  name: 'stream progress is generated text',
  concurrency: 1,
  children: [
    it({
      name: 'CUTS A STREAM THAT SENDS ONLY `: OPENROUTER PROCESSING` COMMENT LINES at the first-content window, '
        + 'where any body bytes used to reset it',
      fn: async () => {
        const cut = await cutOf({
          firstContentMs: WINDOW_MS,
          idleMs: HANG_STOP_MS,
          chunks: Array.from({ length: KEEP_ALIVES, }, () => PROCESSING),
        },);
        expect(stallMessageOf({ cut, },),).toContain('first-content',);
        // The wire did carry bytes, which stay counted as the wire's.
        expect(cut.progress.chars,).toBeGreaterThan(0,);
        expect(cut.progress.firstByteMs,).toBeGreaterThanOrEqual(0,);
        // No text ever came, which the progress line states beside the byte that did (ledger B804).
        expect(cut.progress.firstContentMs,).toBeLessThan(0,);
      },
    },),

    it({
      name: 'CUTS A STREAM THAT SENDS ONLY BLANK KEEP-ALIVE LINES at the first-content window',
      fn: async () => {
        const cut = await cutOf({
          firstContentMs: WINDOW_MS,
          idleMs: HANG_STOP_MS,
          chunks: Array.from({ length: KEEP_ALIVES, }, () => '\n'),
        },);
        expect(stallMessageOf({ cut, },),).toContain('first-content',);
      },
    },),

    it({
      name: 'CUTS A STREAM THAT SENDS ONLY EVENTS WITH AN EMPTY DELTA, a role-only frame and an empty content '
        + 'string, at the first-content window',
      fn: async () => {
        const cut = await cutOf({
          firstContentMs: WINDOW_MS,
          idleMs: HANG_STOP_MS,
          chunks: Array.from({ length: KEEP_ALIVES, }, (_unused, at) => (((at % 2) === 0) ? ROLE_ONLY : EMPTY_DELTA)),
        },);
        expect(stallMessageOf({ cut, },),).toContain('first-content',);
      },
    },),

    it({
      name: 'CUTS A STREAM THAT FALLS TO COMMENT LINES AFTER ITS FIRST WORDS at the idle window, where the comment '
        + 'lines used to reset it',
      fn: async () => {
        const cut = await cutOf({
          firstContentMs: HANG_STOP_MS,
          idleMs: WINDOW_MS,
          chunks: [
            frameOf({ channel: 'content', text: 'The cat ', },),
            ...Array.from({ length: KEEP_ALIVES, }, () => PROCESSING),
          ],
        },);
        expect(stallMessageOf({ cut, },),).toContain('body',);
        expect(cut.generatedChars?.content,).toBe('The cat '.length,);
        // The first words stamped the first text, no earlier than the first byte that carried them.
        expect(cut.progress.firstContentMs,).toBeGreaterThanOrEqual(cut.progress.firstByteMs,);
      },
    },),

    it({
      name: 'COUNTS A CONTENT DELTA AS PROGRESS: a stream that sends a word every few milliseconds ends whole under '
        + 'a window shorter than the stream',
      fn: async () => {
        const bodyText = await drainTimed({
          firstContentMs: WINDOW_MS,
          idleMs: WINDOW_MS,
          chunks: Array.from({ length: KEEP_ALIVES, }, (_unused, at) => frameOf({ channel: 'content', text: `w${String(at,)} `, },)),
        },);
        expect(bodyText.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'COUNTS A REASONING DELTA AS PROGRESS, since a thinking model that has not answered is working',
      fn: async () => {
        const bodyText = await drainTimed({
          firstContentMs: WINDOW_MS,
          idleMs: WINDOW_MS,
          chunks: Array.from({ length: KEEP_ALIVES, }, (_unused, at) => frameOf({ channel: 'reasoning', text: `t${String(at,)} `, },)),
        },);
        expect(bodyText.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'LOGS THE CUT WITH THE PROVIDER, THE MODEL, THE RAW CHARACTERS AND THE CONTENT AND REASONING CHARACTERS',
      fn: async () => {
        const { result: cut, warned, } = await warnLinesDuring({
          run: async () =>
            await cutOf({
              firstContentMs: HANG_STOP_MS,
              idleMs: WINDOW_MS,
              chunks: [
                frameOf({ channel: 'content', text: 'The cat ', },),
                ...Array.from({ length: KEEP_ALIVES, }, () => PROCESSING),
              ],
            },),
        },);
        /**
         The line the guard wrote when it tripped.
         */
        const line = warned.find(function isOurs(entry,): boolean {
          return entry.includes('inception/mercury-2.5',) && entry.includes('aborting for retry',);
        },);
        expect(line,).toBeDefined();
        expect(line,).toContain('openrouter inception/mercury-2.5',);
        expect(line,).toContain(`no content or reasoning for ${String(WINDOW_MS,)}ms during body`,);
        expect(line,).toContain(`${String(cut.progress.chars,)} raw chars`,);
        expect(line,).toContain('8 content chars',);
        expect(line,).toContain('0 reasoning chars',);
      },
    },),
  ],
},);
