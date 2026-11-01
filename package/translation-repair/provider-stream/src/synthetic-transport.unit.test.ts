/**
 Tests for the fetch-backed default transport:
 request assembly (fresh header copy, body only on POST, dependent
 signal), raw status passthrough, and abort propagation.
 The global fetch is stubbed per test; children run sequentially so
 stubs never overlap.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CREDENTIAL_MARKER,
  exchangeFailureText,
  fetchTransport,
  StreamCutShortError,
  StreamOverrunError,
} from '../dist/final/node/index.mjs';
import {
  anthropicBlockDelta,
  anthropicBlockStart,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';
import { pieceResponse, } from '@monochromatic-dev/translation-repair-test-fixture/ts/piece-response.test-fixture.ts';
import {
  quotingFailure,
  WHISKER_KEY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { frameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/sse-frame.test-fixture.ts';


/**
 Request init the stubbed fetch captured, probed field by field.
 */
type CapturedInit = {
  readonly method?: string;
  readonly headers?: Readonly<Record<string, string>>;
  readonly body?: string;
  readonly signal?: AbortSignal;
};

await describe({
  name: fetchTransport.name,
  concurrency: 1,
  children: [
    it({
      name: 'assembles a POST with copied headers and returns the raw reply',
      fn: async ctx => {
        /**
         Headers object whose identity must not reach the platform request.
         */
        const callerHeaders = { Authorization: 'Bearer cat-key', };
        const fetchStub = ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            '{"cat":"喵"}',
            { status: 200, },
          ),);

        const reply = await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'POST',
          headers: callerHeaders,
          bodyJson: '{"model":"cat"}',
          signal: new AbortController().signal,
        },);

        expect(reply,).toEqual({
          status: 200,
          bodyText: '{"cat":"喵"}',
        },);
        expect(fetchStub,).toHaveBeenCalledTimes(1,);
        const [calledUrl, init,] = fetchStub.firstCall.args;
        const captured = init as CapturedInit;
        expect(calledUrl,).toBe('https://example.org/cat-chat',);
        expect(captured.method,).toBe('POST',);
        expect(captured.body,).toBe('{"model":"cat"}',);
        expect(captured.headers,).toEqual(callerHeaders,);
        expect(captured.headers,).not.toBe(callerHeaders,);
      },
    },),
    it({
      name: 'omits the body key entirely on GET exchanges',
      fn: async ctx => {
        const fetchStub = ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            '{}',
            { status: 200, },
          ),);

        await fetchTransport({
          url: 'https://example.org/cat-quotas',
          label: 'hf:whiskers',
          method: 'GET',
          headers: {},
          signal: new AbortController().signal,
        },);

        const [, init,] = fetchStub.firstCall.args;
        expect(Object.hasOwn(
          init as object,
          'body',
        ),).toBe(false,);
      },
    },),
    it({
      name: 'passes non-success statuses through as data, never throwing',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            'upstream napping',
            { status: 503, },
          ),);

        const reply = await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'POST',
          headers: {},
          bodyJson: '{}',
          signal: new AbortController().signal,
        },);

        expect(reply,).toEqual({
          status: 503,
          bodyText: 'upstream napping',
        },);
      },
    },),
    it({
      name: 'derives a dependent signal that carries the caller abort',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .callsFake(async function abortAwareFetch(_url, init,) {
            /**
             Signal the platform request would honor.
             */
            const { signal, } = (init ?? {}) as CapturedInit;
            if (signal?.aborted === true)
              throw signal.reason;
            return new Response(
              '{}',
              { status: 200, },
            );
          },);

        const caller = new AbortController();
        caller.abort(new Error('user steered away',),);

        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: {},
            bodyJson: '{}',
            signal: caller.signal,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('user steered away',);
      },
    },),
    it({
      name: 'hands fetch a signal distinct from the caller handle',
      fn: async ctx => {
        const fetchStub = ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            '{}',
            { status: 200, },
          ),);

        const caller = new AbortController();
        await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'GET',
          headers: {},
          signal: caller.signal,
        },);

        const [, init,] = fetchStub.firstCall.args;
        expect((init as CapturedInit).signal,).not.toBe(caller.signal,);
      },
    },),
    it({
      name: 'MASKS A CREDENTIAL THE PROVIDER ECHOES in a failure reply, the whole header value and the bare token alike',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            `{"error":{"message":"rejected Bearer ${WHISKER_KEY}, token ${WHISKER_KEY}"}}`,
            { status: 401, },
          ),);

        const reply = await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'POST',
          headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
          bodyJson: '{}',
          signal: new AbortController().signal,
        },);

        expect(reply,).toEqual({
          status: 401,
          bodyText: `{"error":{"message":"rejected ${CREDENTIAL_MARKER}, token ${CREDENTIAL_MARKER}"}}`,
        },);
      },
    },),
    it({
      name: 'MASKS A CREDENTIAL THE PROVIDER ECHOES IN OTHER SPELLINGS in a failure reply: JSON unicode escapes, '
        + 'percent escapes and base64',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            `{"error":"rejected whisker\\u002dkey\\u002d7421, whisker%2Dkey%2D7421, ${
              Buffer.from(`kit:${WHISKER_KEY}`,)
                .toString('base64url',)
            }"}`,
            { status: 401, },
          ),);

        const reply = await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'POST',
          headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
          bodyJson: '{}',
          signal: new AbortController().signal,
        },);

        expect(reply,).toEqual({
          status: 401,
          // The base64 run's first characters hold bits of the four bytes before the key.
          bodyText: `{"error":"rejected ${CREDENTIAL_MARKER}, ${CREDENTIAL_MARKER}, a2l0O${CREDENTIAL_MARKER}"}`,
        },);
      },
    },),
    it({
      name: 'MASKS A CREDENTIAL IN A SUCCESSFUL STREAM, one an error event echoes and one split across two chunks',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .callsFake(async function splitBody() {
            return pieceResponse({
              // The key is cut in two between the pieces.
              pieces: [
                'data: {"error":{"message":"bad key whisker-ke',
                'y-7421"}}\n\ndata: [DONE]\n\n',
              ],
              ending: 'close',
              status: 200,
            },);
          },);

        const reply = await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'POST',
          headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
          bodyJson: '{}',
          signal: new AbortController().signal,
        },);

        expect(reply,).toEqual({
          status: 200,
          bodyText: `data: {"error":{"message":"bad key ${CREDENTIAL_MARKER}"}}\n\ndata: [DONE]\n\n`,
        },);
      },
    },),
    it({
      name: 'MASKS A CREDENTIAL IN THE TEXT A CUT STREAM CARRIES on its error',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .callsFake(async function cutBody() {
            return pieceResponse({
              pieces: [`data: {"error":"bad key ${WHISKER_KEY}"}\n\n`,],
              ending: new Error('connection reset',),
              status: 200,
            },);
          },);
        /** Value caught from the exchange whose stream was cut. */
        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
            bodyJson: '{}',
            signal: new AbortController().signal,
          },);
        }
        catch (error) {
          caught = error;
        }
        if (!(caught instanceof StreamCutShortError))
          throw new Error('a cut-short error by construction',);
        expect(caught.partialText,).toBe(`data: {"error":"bad key ${CREDENTIAL_MARKER}"}\n\n`,);
      },
    },),
    it({
      name: 'KEEPS THE CHARACTERS OF THE ANSWER AND OF THE REASONING A CUT STREAM COUNTED on the error it masks, so the abandonment '
        + 'line of a synthetic call reports them in place of the wire length',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .callsFake(async function cutBody() {
            return pieceResponse({
              pieces: [
                frameOf({ channel: 'reasoning', text: 'Pondering the cat.', },) + frameOf({ channel: 'content', text: 'It is a cat.', },),
              ],
              ending: new Error('connection reset',),
              status: 200,
            },);
          },);
        /** Value caught from the exchange whose stream was cut. */
        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
            bodyJson: '{}',
            signal: new AbortController().signal,
          },);
        }
        catch (error) {
          caught = error;
        }
        if (!(caught instanceof StreamCutShortError))
          throw new Error('a cut-short error by construction',);
        expect(caught.generatedChars,).toEqual({ content: 'It is a cat.'.length, reasoning: 'Pondering the cat.'.length, },);
      },
    },),
    it({
      name: 'NAMES A REQUEST THE RUNTIME REFUSED TO SEND by an authored account, the runtime\'s own words kept as the cause',
      fn: async ctx => {
        /**
         Rejection whose message quotes the header value.
         */
        const failure = quotingFailure();
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .rejects(failure,);
        /** Value caught from the exchange the runtime refused. */
        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
            bodyJson: '{}',
            signal: new AbortController().signal,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(exchangeFailureText({ error: caught, },),).toBe(
          'hf:whiskers: the request failed before any answer came back, either because the network could not be '
            + 'reached or because the transport refused to send it (a header value it cannot carry is one such '
            + 'refusal); check the connection and the key',
        );
        expect(Error.isError(caught,) ? caught.cause : undefined,).toBe(failure,);
      },
    },),
    it({
      name: 'PASSES THE CALLER\'S ABORT ON UNCHANGED when the runtime rejects the request with it',
      fn: async ctx => {
        /**
         Reason the caller aborted with.
         */
        const steering = new Error('user steered away',);
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .rejects(steering,);
        /**
         Caller whose abort is already standing.
         */
        const caller = new AbortController();
        caller.abort(steering,);
        /** Value caught from the exchange the caller abandoned. */
        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: {},
            bodyJson: '{}',
            signal: caller.signal,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(steering,);
      },
    },),
    it({
      name: 'raises the drain overrun on an answer past a caller maxAnswerChars, the bound riding '
        + 'through to the drain',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            frameOf({
              channel: 'content',
              text: 'Cat one sat on mat one. Cat two sat on mat two. ',
            },),
            { status: 200, },
          ),);
        /** Value caught from the exchange whose answer passed its bound. */
        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: {},
            bodyJson: '{"model":"cat"}',
            signal: new AbortController().signal,
            maxAnswerChars: 10,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught instanceof StreamOverrunError,).toBe(true,);
        expect(String(caught,),).toBe(
          'StreamOverrunError: hf:whiskers: ended a call that exceeded its content bound, 48 characters on the '
            + 'content channel against a bound of 10',
        );
      },
    },),
    it({
      name: 'reads an anthropic-shaped stream when the exchange names its wire format, where the '
        + 'default scanner reads no delta and the call simply completes',
      fn: async ctx => {
        /**
         Anthropic text frames whose answer passes the same small bound an
         OpenAI-shaped scanner never sees: an anthropic frame carries no
         `choices` key at all.
         */
        const raw = anthropicBlockStart({
          index: 0,
          type: 'text',
        },) + anthropicBlockDelta({
          index: 0,
          deltaType: 'text_delta',
          field: 'text',
          text: 'Cat one sat on mat one. ',
        },);
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .callsFake(async function freshBody() {
            return new Response(
              raw,
              { status: 200, },
            );
          },);
        /** Value caught from the exchange naming the anthropic wire. */
        let caught: unknown;
        try {
          await fetchTransport({
            url: 'https://example.org/cat-chat',
            label: 'hf:whiskers',
            method: 'POST',
            headers: {},
            bodyJson: '{"model":"cat"}',
            signal: new AbortController().signal,
            maxAnswerChars: 10,
            wireFormat: 'anthropic',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught instanceof StreamOverrunError,).toBe(true,);
        expect(String(caught,),).toBe(
          'StreamOverrunError: hf:whiskers: ended a call that exceeded its content bound, 24 characters on the '
            + 'content channel against a bound of 10',
        );
        /**
         The same body with no wire named: the default scanner reads no
         answer character of it, so the same bound never trips.
         */
        const reply = await fetchTransport({
          url: 'https://example.org/cat-chat',
          label: 'hf:whiskers',
          method: 'POST',
          headers: {},
          bodyJson: '{"model":"cat"}',
          signal: new AbortController().signal,
          maxAnswerChars: 10,
        },);
        expect(reply.bodyText,).toBe(raw,);
      },
    },),
  ],
},);
