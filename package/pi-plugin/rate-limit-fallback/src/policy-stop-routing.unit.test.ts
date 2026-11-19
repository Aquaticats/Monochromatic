/**
 A policy refusal stays terminal without losing the provider's remediation.
 @module
 */

import { isRetryableAssistantError, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import { assistant, collect, fixture, immediate, poolModel, rateLimitRefusal, TARGETS, TRANSCRIPT, } from './stand-in-pi-fixture.ts';

/** OpenAI policy refusal, verbatim as truncated in the Pi driver's summary. */
const OPENAI_POLICY = 'This content was flagged for possible cybersecurity risk. If this seems wrong, '
  + 'try rephrasing your request. If you’re doing authorized security work that requires more cyber permissive '
  + 'safeguards, apply for Daybreak access via https://platform.openai.com/settings/organization/status-and-access befor';

/** Wording that identifies a policy stop without HTTP status evidence. */
const POLICY_MESSAGES = {
  'legacy wording without a 429 status': 'Request flagged as a possible risk',
  'verbatim OpenAI wording with a curly apostrophe': OPENAI_POLICY,
  'OpenAI wording truncated after the first sentence': 'This content was flagged for possible cybersecurity risk.',
  'OpenAI wording with a straight apostrophe': OPENAI_POLICY.replace('’', '\'',),
  'content-filter label': 'Rejected by the content-filter. Contact support.',
  'safety_policy label': 'Blocked by safety_policy. Contact support.',
  'safety violation label': 'Blocked for a safety violation. Contact support.',
  'policy violation label': 'Blocked for a policy violation. Contact support.',
} as const;

/** Physical writer head and reviewer head, followed by the reviewer's backup. */
const [OPENAI, QWEN, SYNTHETIC,] = TARGETS;

/** Member-attributed preface whose retryability is checked with Pi's classifier. */
const STOPPED = `${OPENAI.provider}/${OPENAI.id} stopped the request under its content policy:`;

/** Independent classifier examples across the literal, separated, status, and fetch match shapes. */
const TRANSIENT_SPANS = [
  'Service is temporarily unavailable',
  'Provider/returned_an error',
  'networkerror',
  'HTTP2 stream failed',
  'HTTP/2 stream failed',
  'request to https://stand-in.example/path failed, reason:',
  'ResourceExhausted',
  'timed out',
  'server_busy',
  '503',
] as const;

/** Near matches and Unicode text must be preserved, not removed with the retry spans. */
const RETAINED_DETAIL = 'Reference İ😀abc429xyz, 1503, and rate\nlimit.';

await describe({
  name: 'terminal policy refusals',
  children: [
    ...Object.entries(POLICY_MESSAGES,).map(function policyMessage([label, errorMessage,],) {
      return it({
        name: `preserves a terminal policy refusal with ${label}`,
        fn: async function policyStop() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) { return immediate({ model, errorMessage, },); },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.errorMessage,).toBe(`${STOPPED} ${errorMessage}`,);
          expect(isRetryableAssistantError(result,),).toBe(false,);
          expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider,],);
        },
      },);
    },),
    it({
      name: 'OpenAI policy opening outranks an adapter 429 and only retry wording is removed',
      fn: async function sanitizeRefusal() {
        const refusal = rateLimitRefusal({
          model: OPENAI,
          message: `${OPENAI_POLICY}. Server_busy, RAtE-Limit, and please retry your request. Reference abc429xyz.`,
        },);
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: refusal, },); },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.errorMessage,).toBe(
          `${STOPPED} OpenAI API error ([omitted]): {"message":"${OPENAI_POLICY}. `
            + '[omitted], [omitted], and [omitted]. Reference abc429xyz."}',
        );
        expect(isRetryableAssistantError(result,),).toBe(false,);
        expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider,],);
      },
    },),
    ...TRANSIENT_SPANS.map(function retrySpan(span,) {
      return it({
        name: `masks Pi retry span ${span} without changing adjacent advisory text`,
        fn: async function maskOnlyTransientSpan() {
          const errorMessage = `${OPENAI_POLICY}. ${RETAINED_DETAIL} (${span})`;
          const { provider, options, } = await fixture({
            extension,
            play({ model, }) {
              expect(isRetryableAssistantError(assistant({ model, errorMessage, },),),).toBe(true,);
              return immediate({ model, errorMessage, },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.errorMessage,).toBe(`${STOPPED} ${OPENAI_POLICY}. ${RETAINED_DETAIL} ([omitted])`,);
          expect(isRetryableAssistantError(result,),).toBe(false,);
        },
      },);
    },),
    ...['possible risk', 'possible cybersecurity risk',].map(function genericRisk(phrase,) {
      return it({
        name: `hands off on a Completions 429 that merely mentions ${phrase}`,
        fn: async function riskMentionInRateLimit() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              return immediate({
                model,
                ...(model.provider === QWEN.provider
                  ? { errorMessage: rateLimitRefusal({ model, message: `Too many requests while we assess ${phrase}`, },), }
                  : {}),
              },);
            },
          },);
          const model = poolModel({ provider, id: 'reviewer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.stopReason,).toBe('stop',);
          expect(calls.map(call => call.model.provider,),).toEqual([QWEN.provider, SYNTHETIC.provider,],);
        },
      },);
    },),
  ],
},);
