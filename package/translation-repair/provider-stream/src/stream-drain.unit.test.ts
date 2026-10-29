/**
 Tests for the stream drain, at the boundary where a call is actually ended.

 The composition layer is tested in `stream-runaway-watch.unit.test.ts`. What
 is tested here is the thing that matters to a running pipeline: that a
 degenerating call STOPS, rather than that something correctly formed an
 opinion about it. A verdict nobody acts on ends nothing, and the provider
 ends nothing either.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { logger as frameworkLogger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography';
import {
  armIdleGuard,
  CREDENTIAL_MARKER,
  drainBody,
  StreamCutShortError,
  StreamDegenerateError,
  StreamOverrunError,
  StreamStalledError,
  ProviderHttpError,
} from '../dist/final/node/index.mjs';
import {
  anthropicBlockDelta,
  anthropicBlockStart,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  quotingFailure,
  WHISKER_KEY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import {
  frameOf,
  longVariedStream,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/sse-frame.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Characters in each piece `streamOf` hands over; every fixture here is ASCII,
 so a piece decodes to exactly this many characters.
 */
const PIECE_CHARS = 4_096;

/**
 Wraps text in a response whose body arrives in pieces, counting how many
 pieces were actually pulled.

 THE COUNT IS THE POINT. A drain that read the whole body and then complained
 would pass every assertion about the error while leaving the socket open for
 the entire runaway, which is the cost this guard exists to avoid.

 @param raw - whole body

 @returns Response, and a reader of how much of it was consumed

 @example
 ```ts
 const { response, pulled, } = streamOf({ raw, },);
 ```
 */
function streamOf({ raw, }: { readonly raw: string; },): {
  readonly response: Response;
  readonly pulled: () => number;
} {
  /**
   Piece width, near what a socket delivers.
   */
  const width = 4_096;

  /**
   Pieces the body is delivered in.
   */
  const pieces = Array.from(
    { length: Math.ceil(raw.length / width,), },
    function piece(
      _unused,
      at,
    ): string {
      return raw.slice(
        at * width,
        (at + 1) * width,
      );
    },
  );

  /**
   How many pieces the drain asked for.
   */
  const taken = { count: 0, };

  /**
   Encoder, since a body carries bytes rather than text.
   */
  const encoder = new TextEncoder();

  /**
   Body that hands over one piece per pull.
   */
  const body = new ReadableStream<Uint8Array>({
    pull(controller,): void {
      /**
       Next piece, absent once they run out.
       */
      const next = pieces[taken.count];
      if (next === undefined) {
        controller.close();
        return;
      }
      taken.count += 1;
      controller.enqueue(encoder.encode(next,),);
    },
  },);

  return {
    response: new Response(
      body,
      { headers: { 'content-type': 'text/event-stream', }, },
    ),
    pulled(): number {
      return taken.count;
    },
  };
}

/**
 Wraps a model thinking the same sentence forever in a response whose body's
 own cancellation always fails, the way a socket that will not tear down
 cleanly behaves, counting how often that cancellation ran.

 @returns Response, and a reader of how many times its body's cancel ran

 @example
 ```ts
 const { response, cancels, } = uncancellableRunaway();
 ```
 */
function uncancellableRunaway(): {
  readonly response: Response;
  readonly cancels: () => number;
} {
  /**
   A model thinking the same sentence forever, same shape as the other
   runaway cases, so the ending is reached the same way.
   */
  const raw = frameOf({
    channel: 'reasoning',
    text: 'I will output. ',
  },)
    .repeat(30_000,);

  /**
   Pieces the body hands over one at a time.
   */
  const pieces = Array.from(
    { length: Math.ceil(raw.length / PIECE_CHARS,), },
    function piece(
      _unused,
      at,
    ): string {
      return raw.slice(
        at * PIECE_CHARS,
        (at + 1) * PIECE_CHARS,
      );
    },
  );

  /**
   How many times the underlying source's own `cancel` ran.
   */
  const cancelled = { count: 0, };

  /**
   Encoder, since a body carries bytes.
   */
  const encoder = new TextEncoder();

  /**
   A body whose own cancellation always fails.
   */
  const body = new ReadableStream<Uint8Array>({
    pull(controller,): void {
      const next = pieces.shift();
      if (next === undefined) {
        controller.close();
        return;
      }
      controller.enqueue(encoder.encode(next,),);
    },
    cancel(): never {
      cancelled.count += 1;
      throw quotingFailure();
    },
  },);

  return {
    response: new Response(
      body,
      { headers: { 'content-type': 'text/event-stream', }, },
    ),
    cancels(): number {
      return cancelled.count;
    },
  };
}

/**
 What one drain did, as a value.

 @example
 ```ts
 const outcome: DrainOutcome = { kind: 'drained', };
 ```
 */
type DrainOutcome = {
  readonly kind: 'drained';

  /**
   Body it handed back.
   */
  readonly body: string;
} | {
  readonly kind: 'raised';

  /**
   What it threw.
   */
  readonly error: unknown;
};

/**
 Drains a response, reporting a throw as a value so the assertion reads as an
 expectation rather than as control flow.

 @param response - response to drain

 @param guard - silence guard to pass through

 @param wireFormat - event grammar to name to the drain, absent for the
 module's own default

 @param label - model name to drain under

 @param credentials - secrets the request carried, none where absent

 @mutates response - its body is drained and cannot be read again

 @mutates guard - the drain notifies it per chunk

 @returns What the drain did

 @example
 ```ts
 const outcome = await drainOutcome({ response, guard, },);
 ```
 */
async function drainOutcome(
  {
    response,
    guard,
    callerSignal = new AbortController().signal,
    wireFormat,
    label = 'hf:whiskers',
    credentials = [],
  }: {
    readonly response: Response;
    readonly guard: Parameters<typeof drainBody>[0]['guard'];
    readonly callerSignal?: AbortSignal;
    readonly wireFormat?: Parameters<typeof drainBody>[0]['wireFormat'];
    readonly label?: string;
    readonly credentials?: readonly string[];
  },
): Promise<DrainOutcome> {
  try {
    return {
      kind: 'drained',
      body: await drainBody({
        response,
        guard,
        callerSignal,
        label,
        // Conditional spread keeps the knob absent instead of undefined.
        ...(wireFormat === undefined ? {} : { wireFormat, }),
        credentials,
      },),
    };
  }
  catch (error) {
    return {
      kind: 'raised',
      error,
    };
  }
}

/**
 Label the credential cases drain under, so the progress line a case reads
 is told from the lines the other cases write.
 */
const CREDENTIAL_LABEL = 'hf:whiskers-key';

/**
 Names of the clock readings a progress line states, which differ on every
 run.
 */
const CLOCK_READINGS: readonly string[] = ['elapsed', 'firstByte', 'firstContent', 'maxGap',];

/**
 Frame carrying text on the answer channel, and the gateway's name for the
 upstream where one is given.

 @param text - text the frame carries

 @param provider - upstream the frame names, absent for none

 @returns Frame as the wire sends it

 @example
 ```ts
 const raw = namedFrame({ text: 'purr', provider: 'Parasail', },);
 ```
 */
function namedFrame(
  {
    text,
    provider,
  }: {
    readonly text: string;
    readonly provider?: string;
  },
): string {
  return `data: ${
    JSON.stringify({
      ...((provider === undefined) ? {} : { provider, }),
      choices: [{
        index: 0,
        delta: { content: text, },
        finish_reason: null,
      },],
    },)
  }\n\n`;
}

/**
 Response whose body hands over each piece in turn and is then torn down,
 as an abort tears a fetch down, or closes cleanly where the ending is
 clean.

 @param pieces - texts delivered one per pull

 @param ending - whether the body is torn down after the last piece or closes

 @returns Response over that body

 @example
 ```ts
 const response = piecesThen({ pieces: [frame,], ending: 'torn down', },);
 ```
 */
function piecesThen(
  {
    pieces,
    ending,
  }: {
    readonly pieces: readonly string[];
    readonly ending: 'torn down' | 'closed';
  },
): Response {
  /**
   Pieces handed over so far.
   */
  const sent = { count: 0, };
  /**
   Encoder, since a body carries bytes.
   */
  const encoder = new TextEncoder();
  return new Response(new ReadableStream<Uint8Array>({
    pull(controller,): void {
      /**
       Next piece, absent once they run out.
       */
      const next = pieces[sent.count];
      if (next !== undefined) {
        sent.count += 1;
        controller.enqueue(encoder.encode(next,),);
        return;
      }
      if (ending === 'closed') {
        controller.close();
        return;
      }
      controller.error(new Error('exchange torn down by abort',),);
    },
  },),);
}

/**
 Drains a response under the credential label, handing the drain the
 credentials a request carried, and returns the progress line the drain
 logged, with the clock readings taken out so the line is the same on every
 run.

 @param response - response to drain

 @param credentials - secrets the request carried, left out of the call
 where absent

 @mutates response - its body is drained and cannot be read again

 @returns Progress line's message, readings of elapsed time blanked

 @example
 ```ts
 const line = await progressLineOf({ response, credentials: [WHISKER_KEY,], },);
 ```
 */
async function progressLineOf(
  {
    response,
    credentials,
  }: {
    readonly response: Response;
    readonly credentials?: readonly string[];
  },
): Promise<string> {
  // Both loggers are drained first, so a line another case wrote is printed
  // rather than kept here as the drain's.
  await contextRoot({ tag: 'stream-drain-credential-cases', },)
    .flush();
  await frameworkLogger.flush();
  /**
   Lines `console.info` was handed while the drain ran.
   */
  const kept: string[] = [];
  /**
   `console.info` as it was, put back when the drain is over.
   */
  const original = console.info;
  console.info = (...parts: readonly unknown[]) => {
    kept.push(parts.join(' ',),);
  };
  await using restore = {
    [Symbol.asyncDispose]: async () => {
      console.info = original;
    },
  };
  using guard = armIdleGuard({
    label: CREDENTIAL_LABEL,
    firstContentMs: HANG_STOP_MS,
    idleMs: HANG_STOP_MS,
  },);
  /**
   What the drain did, which these cases do not read: the line it logged is
   the evidence.
   */
  const outcome = await drainOutcome({
    response,
    guard,
    label: CREDENTIAL_LABEL,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((credentials === undefined) ? {} : { credentials, }),
  },);
  expect(['drained', 'raised',].includes(outcome.kind,),).toBe(true,);
  await contextRoot({ tag: 'stream-drain-credential-cases', },)
    .flush();
  /**
   The one line this drain's report wrote.
   */
  const [line, ...others] = kept.filter(function ours(entry,): boolean {
    return entry.includes(`stream ${CREDENTIAL_LABEL}:`,);
  },);
  if ((line === undefined) || (others.length > 0))
    throw new Error(`expected one progress line for ${CREDENTIAL_LABEL}, got ${JSON.stringify(kept,)}`,);
  return line
    .slice(line.indexOf(`stream ${CREDENTIAL_LABEL}:`,),)
    .split(', ',)
    .map(function blanked(part,): string {
      return CLOCK_READINGS.some(function isReading(reading,): boolean {
        return part.startsWith(`${reading} `,);
      },)
        ? `${part.split(' ',)[0] ?? ''} Xms`
        : part;
    },)
    .join(', ',);
}

/**
 The progress line the drain writes for a cut stream that delivered one
 piece of answer text.

 @param rawChars - raw wire characters delivered

 @param contentChars - characters of generated answer text

 @param opening - excerpt of that text the line shows

 @param served - `, served by "..."` part, empty for none

 @returns Line's message

 @example
 ```ts
 const line = cutLine({ rawChars: 90, contentChars: 20, opening: 'purr', served: '', },);
 ```
 */
function cutLine(
  {
    rawChars,
    contentChars,
    opening,
    served,
  }: {
    readonly rawChars: number;
    readonly contentChars: number;
    readonly opening: string;
    readonly served: string;
  },
): string {
  return `stream ${CREDENTIAL_LABEL}: cut, elapsed Xms, firstByte Xms, firstContent Xms, maxGap Xms, ${String(rawChars,)} raw chars, `
    + `0 unreadable frames, ${String(contentChars,)} content chars, 0 reasoning chars${served}, `
    + `opening ${JSON.stringify(opening,)}`;
}

/**
 A response whose body delivers one piece and is then torn down, which is what an abort does to a fetch in production.

 @param delivered - one piece, as the wire carries it

 @param steering - caller's own steering, aborted when the body is torn down

 @returns Response

 @example
 ```ts
 const response = tornDownResponseOf({ delivered: 'purr', steering: new AbortController(), },);
 ```
 */
function tornDownResponseOf(
  { delivered, steering, }: { readonly delivered: string; readonly steering: AbortController; },
): Response {
  /**
   How many pieces have gone out.
   */
  const sent = { count: 0, };
  /**
   Encoder, since a body carries bytes.
   */
  const encoder = new TextEncoder();
  return new Response(new ReadableStream<Uint8Array>({
    pull(controller,): void {
      if (sent.count === 0) {
        sent.count += 1;
        controller.enqueue(encoder.encode(delivered,),);
        return;
      }
      steering.abort();
      controller.error(new Error('exchange torn down by abort',),);
    },
  },),);
}

await describe({
  name: drainBody.name,
  // ONE AT A TIME: a case diverts the process-wide `console.warn` across an await.
  concurrency: 1,
  children: [
    it({
      name: 'CARRIES THE RAW CHARACTERS DELIVERED on both errors it ends a call with (ledger P7): the '
        + 'abandoned-spend reckoning divides raw wire characters by a raw-characters-per-token ratio, and '
        + 'an overrun or a degenerate ending carried only one channel\'s decoded count, about a '
        + 'hundredth of the raw figure',
      fn: async () => {
        /**
         Varied answer text, so the overrun bound ends the call and repetition does not.
         */
        const answering = streamOf({
          raw: Array.from(
            { length: 30_000, },
            function answer(_unused, at,): string {
              return frameOf({
                channel: 'content',
                text: `Cat ${String(at,)} sat on mat ${String(at * 7,)}. `,
              },);
            },
          ).join('',),
        },);

        using overrunGuard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         Drain of the varied answer under a small content bound.
         */
        const overrun = await (async function drainUnderBound(): Promise<unknown> {
          try {
            return await drainBody({
              credentials: [],
              response: answering.response,
              guard: overrunGuard,
              callerSignal: new AbortController().signal,
              label: 'hf:whiskers',
              maxAnswerChars: 2_000,
            },);
          }
          catch (error) {
            return error;
          }
        })();
        if (!(overrun instanceof StreamOverrunError))
          throw new Error('an overrun by construction',);

        /**
         A model thinking the same sentence forever.
         */
        const repeating = streamOf({
          raw: frameOf({
            channel: 'reasoning',
            text: 'I will output. ',
          },)
            .repeat(30_000,),
        },);

        using degenerateGuard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         What the drain did with the repetition.
         */
        const outcome = await drainOutcome({
          response: repeating.response,
          guard: degenerateGuard,
        },);
        if ((outcome.kind !== 'raised') || (!(outcome.error instanceof StreamDegenerateError)))
          throw new Error('a degeneration error by construction',);

        expect({
          overrunRaw: overrun.rawChars,
          degenerateRaw: outcome.error.rawChars,
        },).toEqual({
          overrunRaw: answering.pulled() * PIECE_CHARS,
          degenerateRaw: repeating.pulled() * PIECE_CHARS,
        },);
      },
    },),

    it({
      name: 'ENDS A RUNAWAY CALL rather than draining it, which is the whole point: the provider '
        + 'does not end these and no token cap bounds them, so this is the only place the call can '
        + 'stop. Asserting only the error would pass for a drain that read every byte first',
      fn: async () => {
        /**
         A model thinking the same sentence forever.
         */
        const {
          response,
          pulled,
        } = streamOf({
          raw: frameOf({
            channel: 'reasoning',
            text: 'I will output. ',
          },)
            .repeat(30_000,),
        },);

        using guard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         What the drain did, as a value, since the throw is what is asserted.
         */
        const outcome = await drainOutcome({
          response,
          guard,
        },);

        expect(outcome.kind,).toBe('raised',);
        if (outcome.kind !== 'raised')
          throw new Error('raised by construction',);
        if (!(outcome.error instanceof StreamDegenerateError))
          throw new Error('a degeneration error by construction',);
        expect(outcome.error.channel,).toBe('reasoning',);

        // Attributed to the model rather than to the endpoint: every
        // chat-completions call shares one URL across the whole roster, and a
        // constructed Response's `url` is the empty string, so a per-model
        // latency figure would be unreadable if this fell back to it.
        expect(outcome.error.label,).toBe('hf:whiskers',);

        // Stopped early rather than after the fact. Reading every piece would
        // mean the socket stayed open for the whole runaway.
        /**
         Pieces the whole runaway would have taken, had it been drained.
         */
        const whole = Math.ceil((30_000 * 60) / 4_096,);
        expect(pulled(),).toBeLessThan(whole,);
      },
    },),

    it({
      name: 'SWALLOWS A FAILED CANCEL rather than reporting it in place of the runaway diagnosis: '
        + 'the socket is released before the error is thrown, and releasing it failing must not '
        + 'replace the reason the call was ended with a reason it could not be torn down',
      fn: async () => {
        /**
         A runaway whose body's cancellation always fails.
         */
        const {
          response,
          cancels,
        } = uncancellableRunaway();

        using guard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        const {
          result: outcome,
          warned,
        } = await warnLinesDuring({
          run: async () =>
            drainOutcome({
              response,
              guard,
            },),
        },);

        // THE FAILED CANCEL IS NAMED BY CLASS: the failure's message quotes a header value.
        expect(warned,).toEqual(['[translation-repair] [drainBody] could not cancel : refused by TypeError',],);
        expect(outcome.kind,).toBe('raised',);
        if (outcome.kind !== 'raised')
          throw new Error('raised by construction',);

        // The runaway diagnosis survives the cancel failure: had the catch at
        // `stopReading` not swallowed it, this would have surfaced as a
        // StreamCutShortError wrapping the cancel's own error instead.
        if (!(outcome.error instanceof StreamDegenerateError))
          throw new Error('a degeneration error by construction',);
        expect(outcome.error.channel,).toBe('reasoning',);
        expect(cancels(),).toBe(1,);
      },
    },),

    it({
      name: 'MASKS THE CREDENTIAL A REDIRECT LEFT IN THE ADDRESS the failed cancel names, since after a redirect '
        + 'the address is one the provider chose and its query can repeat the key the request sent',
      fn: async () => {
        /**
         A runaway whose body's cancellation always fails.
         */
        const { response, } = uncancellableRunaway();
        // The address node's `fetch` leaves on a response whose request was
        // redirected: the one the redirect named, its path and query as the
        // provider wrote them. A constructed response has none of its own.
        Object.defineProperty(
          response,
          'url',
          { value: `https://stream.cat.example/landing?key=${WHISKER_KEY}`, },
        );

        using guard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        const {
          result: outcome,
          warned,
        } = await warnLinesDuring({
          run: async () =>
            drainOutcome({
              response,
              guard,
              credentials: [WHISKER_KEY,],
            },),
        },);

        expect(warned,).toEqual([
          `[translation-repair] [drainBody] could not cancel https://stream.cat.example/landing?key=${CREDENTIAL_MARKER}: `
            + 'refused by TypeError',
        ],);
        if ((outcome.kind !== 'raised') || (!(outcome.error instanceof StreamDegenerateError)))
          throw new Error('a degeneration error by construction',);
      },
    },),

    it({
      name: 'KEEPS WHAT THE STREAM ALREADY DELIVERED when a call is cut, which used to be dropped '
        + 'on the floor. This is the whole point: a call that never got a first byte leaves an '
        + 'empty string and one cut off mid-reasoning leaves a truncated thinking block, and those '
        + 'want opposite remedies. Nothing on disk could tell them apart before',
      fn: async () => {
        /**
         What the model manages to say before the plug is pulled.
         */
        const said = 'It is a cat. It did a backflip. It cras';

        /**
         That much, framed as the wire carries it.
         */
        const delivered = frameOf({
          channel: 'reasoning',
          text: said,
        },);

        /**
         Caller's own steering.
         */
        const steering = new AbortController();

        using guard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         What the drain did.
         */
        const outcome = await drainOutcome({
          response: tornDownResponseOf({ delivered, steering, },),
          guard,
          callerSignal: steering.signal,
        },);

        expect(outcome.kind,).toBe('raised',);
        if (outcome.kind !== 'raised')
          throw new Error('raised by construction',);
        if (!(outcome.error instanceof StreamCutShortError))
          throw new Error('a cut by construction',);

        // The text survives the cut, which is the entire fix.
        expect(outcome.error.partialText,).toBe(delivered,);
        expect(outcome.error.partialText.includes(said,),).toBe(true,);

        // And it is attributed to the model rather than to the endpoint, so a
        // per-model latency figure is readable at all.
        expect(outcome.error.label,).toBe('hf:whiskers',);

        // The original failure is preserved rather than replaced.
        expect(outcome.error.cause,).toBeInstanceOf(Error,);
      },
    },),

    it({
      name: 'HANDS THE CHARACTERS OF THE ANSWER AND OF THE REASONING THE WATCH COUNTED to the cut it throws, so the abandonment line '
        + 'can report them in place of the wire length',
      fn: async () => {
        /**
         What the model reasons and what it answers before the plug is pulled.
         */
        const reasoned = 'Pondering the cat.';
        const answered = 'It is a cat.';

        /**
         That much, framed as the wire carries it.
         */
        const delivered = frameOf({ channel: 'reasoning', text: reasoned, },) + frameOf({ channel: 'content', text: answered, },);

        /**
         Caller's own steering.
         */
        const steering = new AbortController();

        using guard = armIdleGuard({
          label: 'hf:whiskers',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         What the drain did.
         */
        const outcome = await drainOutcome({
          response: tornDownResponseOf({ delivered, steering, },),
          guard,
          callerSignal: steering.signal,
        },);

        if ((outcome.kind !== 'raised') || (!(outcome.error instanceof StreamCutShortError)))
          throw new Error('a cut by construction',);
        expect(outcome.error.generatedChars,).toEqual({ content: answered.length, reasoning: reasoned.length, },);
      },
    },),

    it({
      name: 'STATES A CUT STREAM\'S CAUSE BY CLASS in its message, which the error declares safe to repeat, '
        + 'repeats the sentence of a cause that declares the same, and names the HTTP status of a cause that is '
        + 'a provider\'s status failure without its body',
      fn: async () => {
        /**
         What the stream did before the cut, plain numbers.
         */
        const progress = {
          firstByteMs: 40,
          firstContentMs: 40,
          maxGapMs: 4,
          chars: 512,
          elapsedMs: 830,
        };

        /**
         Cut whose cause is a runtime failure quoting a header value.
         */
        const quoting = new StreamCutShortError({
          label: 'hf:whiskers',
          partialText: 'It is a cat.',
          progress,
          cause: quotingFailure(),
        },);

        /**
         Cut whose cause is a stall, a class that writes its own sentence.
         */
        const stalled = new StreamCutShortError({
          label: 'hf:whiskers',
          partialText: 'It is a cat.',
          progress,
          cause: new StreamStalledError({
            label: 'hf:whiskers',
            idleMs: 60_000,
            phase: 'body',
          },),
        },);

        /**
         Cut whose cause is a provider's status failure, whose own message
         excerpts the body.
         */
        const refused = new StreamCutShortError({
          label: 'hf:whiskers',
          partialText: 'It is a cat.',
          progress,
          cause: new ProviderHttpError({
            provider: 'synthetic',
            status: 503,
            bodyText: 'the cat shelf is full of naps',
          },),
        },);

        expect(refusalText({ error: quoting, },),).toBe(
          'hf:whiskers: stream cut after 12 characters (refused by TypeError)',
        );
        expect(refusalText({ error: refused, },),).toBe(
          'hf:whiskers: stream cut after 12 characters (refused by ProviderHttpError from synthetic with HTTP 503)',
        );
        expect(refusalText({ error: stalled, },),).toBe(
          'hf:whiskers: stream cut after 12 characters (Stalled: hf:whiskers emitted no content or reasoning for 60000ms (body))',
        );
      },
    },),

    it({
      name: 'RETURNS A HEALTHY BODY WHOLE, so the guard costs nothing to an ordinary call and no '
        + 'parser above the transport seam sees anything different',
      fn: async () => {
        /**
         An ordinary reply.
         */
        const raw = `${
          frameOf({
            channel: 'content',
            text: 'A tabby naps in the window. ',
          },)
        }${
          frameOf({
            channel: 'content',
            text: 'It wakes at dusk. ',
          },)
        }data: [DONE]\n\n`;

        const { response, } = streamOf({ raw, },);

        using guard = armIdleGuard({
          label: 'hf:mittens',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         What the drain did.
         */
        const outcome = await drainOutcome({
          response,
          guard,
        },);
        if (outcome.kind !== 'drained')
          throw new Error('drained by construction',);
        expect(outcome.body,).toBe(raw,);
      },
    },),

    it({
      name: 'LETS A LONG BUT VARIED CALL FINISH, because some models write a great deal and being '
        + 'verbose is not being broken',
      fn: async () => {
        /**
         Long, varied thinking followed by a long, varied answer.
         */
        const raw = longVariedStream();

        const { response, } = streamOf({ raw, },);

        using guard = armIdleGuard({
          label: 'hf:sable',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        /**
         What the drain did.
         */
        const outcome = await drainOutcome({
          response,
          guard,
        },);
        if (outcome.kind !== 'drained')
          throw new Error('drained by construction',);
        expect(outcome.body,).toBe(raw,);
      },
    },),

    it({
      name: 'READS AN ANTHROPIC-SHAPED STREAM WHEN TOLD ITS WIRE FORMAT, ending a thinking-trace '
        + 'runaway the default OpenAI-shaped scanner could never read a single character of: an '
        + 'Anthropic frame carries no `choices` key at all, so left unset the same body yields no '
        + 'delta and the call simply completes',
      fn: async () => {
        /**
         A model thinking the same sentence forever, spelled the way the
         Anthropic Messages wire spells a thinking delta.
         */
        const raw = anthropicBlockStart({
          index: 0,
          type: 'thinking',
        },) + anthropicBlockDelta({
          index: 0,
          deltaType: 'thinking_delta',
          field: 'thinking',
          text: 'I will output. ',
        },)
          .repeat(30_000,);

        const { response, } = streamOf({ raw, },);

        using guard = armIdleGuard({
          label: 'hf:sable',
          firstContentMs: HANG_STOP_MS,
          idleMs: HANG_STOP_MS,
        },);

        const outcome = await drainOutcome({
          response,
          guard,
          wireFormat: 'anthropic',
        },);

        expect(outcome.kind,).toBe('raised',);
        if (outcome.kind !== 'raised')
          throw new Error('raised by construction',);
        if (!(outcome.error instanceof StreamDegenerateError))
          throw new Error('a degeneration error by construction',);
        expect(outcome.error.channel,).toBe('reasoning',);
      },
    },),

    it({
      name: 'LOGS A CUT STREAM\'S OPENING WITH A SENT CREDENTIAL MASKED when the first content echoes the key, '
        + 'where the line read the key out of the generated text before any mask ran',
      fn: async () => {
        /**
         Frame whose text echoes the key.
         */
        const frame = namedFrame({ text: `The cat says ${WHISKER_KEY} twice`, },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [frame,],
            ending: 'torn down',
          },),
          credentials: [WHISKER_KEY,],
        },),).toBe(cutLine({
          rawChars: frame.length,
          contentChars: `The cat says ${WHISKER_KEY} twice`.length,
          opening: `The cat says ${CREDENTIAL_MARKER} twice`,
          served: '',
        },),);
      },
    },),

    it({
      name: 'LOGS A CUT STREAM\'S OPENING WITH A BASE64 SPELLING OF THE KEY MASKED, every character its bits '
        + 'touch going with it and no other',
      fn: async () => {
        /**
         Text echoing the key in base64 after a prefix of unknown alignment.
         */
        const said = `cat:${WHISKER_KEY} purrs`;
        /**
         Frame whose text echoes the key as part of a base64 run.
         */
        const frame = namedFrame({
          text: `The cat says ${Buffer.from(said,)
            .toString('base64',)} twice`,
        },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [frame,],
            ending: 'torn down',
          },),
          credentials: [WHISKER_KEY,],
        },),).toBe(cutLine({
          rawChars: frame.length,
          contentChars: `The cat says ${Buffer.from(said,)
            .toString('base64',)} twice`.length,
          opening: `The cat says Y2F0O${CREDENTIAL_MARKER}gcHVycnM= twice`,
          served: '',
        },),);
      },
    },),

    it({
      name: 'LOGS THE OPENING MASKED WHEN THE KEY ARRIVES IN TWO CHUNKS cut inside the key, and when it arrives '
        + 'in two frames of generated text cut inside the key',
      fn: async () => {
        /**
         Text echoing the key.
         */
        const said = `The cat says ${WHISKER_KEY} twice`;
        /**
         Frame carrying the whole text.
         */
        const whole = namedFrame({ text: said, },);
        /**
         Where the first chunk ends: inside the key.
         */
        const cutAt = whole.indexOf(WHISKER_KEY,) + 7;
        /**
         What both deliveries log.
         */
        const expected = cutLine({
          rawChars: whole.length,
          contentChars: said.length,
          opening: `The cat says ${CREDENTIAL_MARKER} twice`,
          served: '',
        },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [whole.slice(0, cutAt,), whole.slice(cutAt,),],
            ending: 'torn down',
          },),
          credentials: [WHISKER_KEY,],
        },),).toBe(expected,);

        /**
         The same text as two frames, split inside the key.
         */
        const first = namedFrame({ text: `The cat says ${WHISKER_KEY.slice(0, 7,)}`, },);
        const second = namedFrame({ text: `${WHISKER_KEY.slice(7,)} twice`, },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [first, second,],
            ending: 'torn down',
          },),
          credentials: [WHISKER_KEY,],
        },),).toBe(cutLine({
          rawChars: first.length + second.length,
          contentChars: said.length,
          opening: `The cat says ${CREDENTIAL_MARKER} twice`,
          served: '',
        },),);
      },
    },),

    it({
      name: 'LOGS NO HEAD OF THE KEY when the stream is cut inside it, so the opening ends in the marker where '
        + 'cutting the excerpt before the mask would have shown the key\'s first characters',
      fn: async () => {
        /**
         Text that ends eight units into the key.
         */
        const said = `The cat says ${WHISKER_KEY.slice(0, 8,)}`;
        /**
         Frame carrying it.
         */
        const frame = namedFrame({ text: said, },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [frame,],
            ending: 'torn down',
          },),
          credentials: [WHISKER_KEY,],
        },),).toBe(cutLine({
          rawChars: frame.length,
          contentChars: said.length,
          opening: `The cat says ${CREDENTIAL_MARKER}`,
          served: '',
        },),);
      },
    },),

    it({
      name: 'LOGS THE UPSTREAM NAME MASKED when the gateway\'s own name field carries a sent credential, on a '
        + 'stream that finished',
      fn: async () => {
        /**
         Frame naming the key as its upstream.
         */
        const frame = namedFrame({
          text: 'purr',
          provider: WHISKER_KEY,
        },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [frame,],
            ending: 'closed',
          },),
          credentials: [WHISKER_KEY,],
        },),).toBe(
          'stream hf:whiskers-key: completed, elapsed Xms, firstByte Xms, firstContent Xms, maxGap Xms, '
            + `${String(frame.length,)} raw chars, 0 unreadable frames, 4 content chars, 0 reasoning chars, `
            + `served by ${JSON.stringify(CREDENTIAL_MARKER,)}`,
        );
      },
    },),

    it({
      name: 'LOGS A STREAM THAT NEVER ECHOES THE KEY exactly as it logged before the drain was given credentials, '
        + 'whether or not the call names any',
      fn: async () => {
        /**
         Text never mentioning the key, ending on letters that begin it.
         */
        const said = 'The cat says purr, then wh';
        /**
         Frame carrying it.
         */
        const frame = namedFrame({
          text: said,
          provider: 'Parasail',
        },);
        /**
         What the drain has always logged for this stream.
         */
        const expected = cutLine({
          rawChars: frame.length,
          contentChars: said.length,
          opening: said,
          served: ', served by "Parasail"',
        },);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [frame,],
            ending: 'torn down',
          },),
        },),).toBe(expected,);
        expect(await progressLineOf({
          response: piecesThen({
            pieces: [frame,],
            ending: 'torn down',
          },),
          credentials: [WHISKER_KEY, `Bearer ${WHISKER_KEY}`,],
        },),).toBe(expected,);
      },
    },),
  ],
},);
