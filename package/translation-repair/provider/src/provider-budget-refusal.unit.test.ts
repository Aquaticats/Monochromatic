/**
 Guards class sixty-two (XingZ612, 2026-09-19): an HTTP 429 that OpenRouter
 passes on from one model's upstream endpoint ("Provider returned error",
 `limit_source` the upstream provider's shared pool) is that model's seat
 failure for the round, not the OpenRouter account's budget. Reading it as
 a budget refusal held the whole provider out for 60 s on every such reply,
 the OpenRouter-only editors and refiners read unreachable, and the entry
 stopped three attempts running. A bare 429 and a 402 stay budget refusals.
 The bodies here are invented provider replies; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  isBudgetRefusal,
  isCredentialRefusal,
  isPaymentRefusal,
  isUpstreamModelRefusal,
  statedWaitMsOf,
} from '../dist/final/node/index.mjs';

/**
 Status a subscription reports exhaustion as.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Status a provider reports a rejected credential as.
 */
const HTTP_UNAUTHORIZED = 401;

/**
 Status a provider reports a request it will not serve as, for reasons that
 are not always the credential.
 */
const HTTP_FORBIDDEN = 403;

/**
 Status a malformed request is rejected with, which Anthropic once used for an empty balance.
 */
const HTTP_BAD_REQUEST = 400;

/**
 Status a credit balance reports no funds as.
 */
const HTTP_PAYMENT_REQUIRED = 402;

/**
 Body OpenRouter answers with when one model's upstream endpoint rate-limits
 its shared pool, as XingZ612 received it, with the model renamed.
 */
const UPSTREAM_POOL_BODY = '{"error":{"message":"Provider returned error","code":429,"metadata":{"raw":"cats/whisker-1 is temporarily'
  + ' rate-limited upstream. Please retry shortly, or add your own key to accumulate your rate limits: https://example.invalid'
  + '/settings/integrations","provider_name":"Catnip","is_byok":false,"provider_error_code":"rate_limit_exceeded",'
  + '"limit_source":"upstream_provider_shared_pool","remedy_hint":"Retry shortly, add your own provider key, or route to'
  + ' another provider with provider routing"}}}';

/**
 Body a provider answers with when our own account is rate-limited.
 */
const ACCOUNT_LIMIT_BODY = '{"error":{"message":"Rate limit exceeded: free-models-per-day","code":429}}';

/**
 Body a credit balance answers with when it cannot pay.
 */
const PAYMENT_BODY = '{"error":{"message":"This request requires more credits","code":402}}';

/**
 Body a provider answers with when its daily limit names the wait, here two
 hours, twenty-five minutes and eighteen seconds.
 */
const DAILY_LIMIT_BODY = '{"error":{"message":"You\'ve hit your daily rate limit. Please try again in 2h25m18s"}}';

await describe({
  name: '',
  children: [
    describe({
      name: 'a 429 passed on from one model\'s upstream endpoint (class sixty-two, XingZ612)',
      children: [
        it({
          name: 'READS the upstream pool limit as the model\'s refusal, not the provider\'s budget',
          fn: async () => {
            const error = new ProviderHttpError({
              provider: 'synthetic',
              status: HTTP_TOO_MANY_REQUESTS,
              bodyText: UPSTREAM_POOL_BODY,
            },);
            expect(isUpstreamModelRefusal({ error, },),).toBe(true,);
            expect(isBudgetRefusal({ error, },),).toBe(false,);
          },
        },),
        it({
          name: 'KEEPS an account rate limit a budget refusal',
          fn: async () => {
            const error = new ProviderHttpError({
              provider: 'synthetic',
              status: HTTP_TOO_MANY_REQUESTS,
              bodyText: ACCOUNT_LIMIT_BODY,
            },);
            expect(isUpstreamModelRefusal({ error, },),).toBe(false,);
            expect(isBudgetRefusal({ error, },),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS a payment refusal a budget refusal whatever its body says',
          fn: async () => {
            const error = new ProviderHttpError({
              provider: 'synthetic',
              status: HTTP_PAYMENT_REQUIRED,
              bodyText: PAYMENT_BODY,
            },);
            expect(isUpstreamModelRefusal({ error, },),).toBe(false,);
            expect(isBudgetRefusal({ error, },),).toBe(true,);
          },
        },),
        it({
          name: 'READS nothing into a failure that is not a provider reply',
          fn: async () => {
            expect(isUpstreamModelRefusal({ error: new Error('the cat unplugged the router',), },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isPaymentRefusal.name,
      children: [
        it({
          name: 'READS a payment-required reply as the balance refusing and a rate limit as not, so only an '
            + 'empty balance holds the provider out until its meter moves',
          fn: async () => {
            expect(isPaymentRefusal({
              error: new ProviderHttpError({
                provider: 'synthetic',
                status: HTTP_PAYMENT_REQUIRED,
                bodyText: PAYMENT_BODY,
              },),
            },),).toBe(true,);
            expect(isPaymentRefusal({
              error: new ProviderHttpError({
                provider: 'synthetic',
                status: HTTP_TOO_MANY_REQUESTS,
                bodyText: ACCOUNT_LIMIT_BODY,
              },),
            },),).toBe(false,);
          },
        },),
        it({
          name: 'READS ANTHROPIC\'S BILLING ERROR as the balance refusing, in both shapes it has used: status 402 '
            + 'with a billing_error body, and status 400 with a "credit balance is too low" message',
          fn: async () => {
            for (const error of [
              new ProviderHttpError({
                provider: 'anthropic',
                status: HTTP_PAYMENT_REQUIRED,
                bodyText: '{"type":"error","error":{"type":"billing_error","message":"billing problem"}}',
              },),
              new ProviderHttpError({
                provider: 'anthropic',
                status: HTTP_BAD_REQUEST,
                bodyText: '{"type":"error","error":{"type":"invalid_request_error","message":"Your credit balance is '
                  + 'too low to access the Anthropic API. Please go to Plans & Billing to upgrade or purchase '
                  + 'credits."}}',
              },),
            ]) {
              expect(isPaymentRefusal({ error, },),).toBe(true,);
              expect(isBudgetRefusal({ error, },),).toBe(true,);
            }
            expect(isPaymentRefusal({
              error: new ProviderHttpError({
                provider: 'anthropic',
                status: HTTP_BAD_REQUEST,
                bodyText: '{"type":"error","error":{"type":"invalid_request_error","message":"max_tokens is too large"}}',
              },),
            },),).toBe(false,);
          },
        },),
        it({
          name: 'READS THE LOW-CREDIT SENTENCE AS ANTHROPIC\'S ALONE: another provider\'s 400 that happens to hold '
            + 'the words is a bad request and not an empty balance, so it neither marks that provider dry nor counts '
            + 'as a budget refusal',
          fn: async () => {
            for (const provider of ['hyper', 'synthetic', 'openrouter', 'bedrock',] as const) {
              /**
               A 400 of another provider whose body quotes the sentence.
               */
              const error = new ProviderHttpError({
                provider,
                status: HTTP_BAD_REQUEST,
                bodyText: '{"error":{"message":"Your credit balance is too low to access the Anthropic API."}}',
              },);
              expect(isPaymentRefusal({ error, },),).toBe(false,);
              expect(isBudgetRefusal({ error, },),).toBe(false,);
            }
          },
        },),
        it({
          name: 'READS NO PAYMENT REFUSAL where the failure is not a provider reply, so a local fault marks '
            + 'nothing refused',
          fn: async () => {
            expect(isPaymentRefusal({ error: new Error('the cat unplugged the router',), },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: statedWaitMsOf.name,
      children: [
        it({
          name: 'READS the wait a refusal names as milliseconds, and none where the refusal names none',
          fn: async () => {
            expect(statedWaitMsOf({
              error: new ProviderHttpError({
                provider: 'synthetic',
                status: HTTP_TOO_MANY_REQUESTS,
                bodyText: DAILY_LIMIT_BODY,
              },),
            },),).toBe(8_718_000,);
            expect(statedWaitMsOf({
              error: new ProviderHttpError({
                provider: 'synthetic',
                status: HTTP_TOO_MANY_REQUESTS,
                bodyText: ACCOUNT_LIMIT_BODY,
              },),
            },),).toBe(0,);
          },
        },),
        it({
          name: 'READS NO STATED WAIT where the failure is not a provider reply, so a local fault holds '
            + 'nothing out',
          fn: async () => {
            expect(statedWaitMsOf({ error: new Error('the cat unplugged the router',), },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: isCredentialRefusal.name,
      children: [
        it({
          name: 'READS a provider answering HTTP 401 as a refused credential, and not as a budget refusal',
          fn: async () => {
            /** Refusal Bedrock gave the pass whose token had expired. */
            const refusal = new ProviderHttpError({
              provider: 'bedrock',
              status: HTTP_UNAUTHORIZED,
              bodyText: '{"error":{"code":"invalid_api_key","message":"The security token included in the request has expired."}}',
            },);
            expect(isCredentialRefusal({ error: refusal, },),).toBe(true,);
            expect(isBudgetRefusal({ error: refusal, },),).toBe(false,);
          },
        },),
        it({
          name: 'READS NO OTHER STATUS, 403 among them, nor a failure that is not a provider reply, as a refused credential',
          fn: async () => {
            expect(isCredentialRefusal({
              error: new ProviderHttpError({
                provider: 'bedrock',
                status: HTTP_FORBIDDEN,
                bodyText: ACCOUNT_LIMIT_BODY,
              },),
            },),).toBe(false,);
            expect(isCredentialRefusal({
              error: new ProviderHttpError({
                provider: 'bedrock',
                status: HTTP_TOO_MANY_REQUESTS,
                bodyText: ACCOUNT_LIMIT_BODY,
              },),
            },),).toBe(false,);
            expect(isCredentialRefusal({ error: new Error('the cat unplugged the router',), },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
