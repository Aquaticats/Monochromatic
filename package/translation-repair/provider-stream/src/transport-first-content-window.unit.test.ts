/**
 Tests that an exchange can name its own first-content window and that the
 transport arms the stream guard with it.

 WHY: in `XingZ6017` twenty-six rounds waited 250 to 360 seconds on an
 `inception/mercury-2.5` stream (two rounds on two in a row) that sent `: OPENROUTER PROCESSING` comment
 lines and no text, until the gateway ended it with HTTP 502 at about 301 s
 (ledger B809). The guard of ledger B804 cuts such a stream at one window for
 every model, 180 s; Mercury's own streams that carried text all did so inside
 57 s, so its card names a shorter window, and the card's number reaches the
 guard only through the exchange.

 WHAT THESE PIN: a window named on the exchange is the window the guard cuts a
 comment-only stream at, and an exchange that names none keeps the shared one.

 Fixtures are cat-themed invention. The window is tiny so a case ends in
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
  fetchTransport,
  StreamCutShortError,
  StreamStalledError,
  type TransportExchange,
} from '../dist/final/node/index.mjs';

/**
 Window the exchange names, short enough to trip inside a case and long enough
 that a loaded machine does not trip it between two chunks that arrive every
 {@link GAP_MS}.
 */
const WINDOW_MS = 250;

/**
 Pause between two comment lines, a fraction of the window.
 */
const GAP_MS = 8;

/**
 Comment lines the stream sends, which at {@link GAP_MS} run well past the
 window.
 */
const KEEP_ALIVES = 80;

/**
 The comment line OpenRouter sends while the upstream provider has not
 answered, with the blank line that ends the event.
 */
const PROCESSING = ': OPENROUTER PROCESSING\n\n';

/**
 Request init the stubbed fetch is handed, which always carries the transport's dependent signal.
 */
type CapturedInit = { readonly signal: AbortSignal; };

/**
 Builds a response whose body sends {@link KEEP_ALIVES} comment lines one per
 {@link GAP_MS}, then a frame of text and the end marker, and errors with the
 abort reason when the request's signal aborts, as a torn-down request does.

 @param signal - signal of the request the body belongs to

 @returns Response carrying that body

 @example
 ```ts
 const response = keepAliveResponse({ signal, },);
 ```
 */
function keepAliveResponse({ signal, }: { readonly signal: AbortSignal; },): Response {
  /**
   Encoder, since a body carries bytes.
   */
  const encoder = new TextEncoder();
  return new Response(
    new ReadableStream<Uint8Array>({
      async start(controller,): Promise<void> {
        signal.addEventListener(
          'abort',
          function tearDown() {
            controller.error(signal.reason,);
          },
          { once: true, },
        );
        for (let sent = 0; sent < KEEP_ALIVES; sent += 1) {
          if (signal.aborted)
            return;
          controller.enqueue(encoder.encode(PROCESSING,),);
          // oxlint-disable-next-line no-await-in-loop -- the pause between two comment lines is the case
          await wait(GAP_MS,);
        }
        if (!signal.aborted) {
          controller.enqueue(encoder.encode(
            `data: ${JSON.stringify({ choices: [{ index: 0, delta: { content: 'meow', }, finish_reason: null, },], },)}\n\n`,
          ),);
          controller.close();
        }
      },
    },),
  );
}

/**
 Exchange of a chat call to the Mercury seat that names no first-content window.

 @returns Exchange for {@link fetchTransport}

 @example
 ```ts
 const exchange = { ...unwindowedExchange(), firstContentMs: 250, };
 ```
 */
function unwindowedExchange(): TransportExchange {
  return {
    url: 'https://example.org/cat-chat',
    label: 'inception/mercury-2.5',
    provider: 'openrouter',
    method: 'POST',
    headers: {},
    bodyJson: '{}',
    signal: new AbortController().signal,
  };
}

await describe({
  name: 'the first-content window an exchange names',
  concurrency: 1,
  children: [
    it({
      name: 'CUTS A COMMENT-ONLY STREAM AT THE WINDOW THE EXCHANGE NAMES, where the shared 180 s window would still be open, '
        + 'and KEEPS THE SHARED WINDOW FOR AN EXCHANGE THAT NAMES NONE, so the same stream is read whole',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .callsFake(async function streamingFetch(_url, init,) {
            await Promise.resolve();
            return keepAliveResponse({ signal: (init as CapturedInit).signal, },);
          },);

        /** Value caught from the exchange that names the window. */
        let caught: unknown;
        try {
          await fetchTransport({ ...unwindowedExchange(), firstContentMs: WINDOW_MS, },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught instanceof StreamCutShortError,).toBe(true,);
        expect((caught as StreamCutShortError).cause instanceof StreamStalledError,).toBe(true,);
        expect(((caught as StreamCutShortError).cause as StreamStalledError).message,).toContain(
          `for ${String(WINDOW_MS,)}ms (first-content)`,
        );

        // The same stream under an exchange that names no window is read whole.
        const reply = await fetchTransport(unwindowedExchange(),);
        expect(reply.status,).toBe(200,);
        expect(reply.bodyText,).toContain('meow',);
      },
    },),
  ],
},);
