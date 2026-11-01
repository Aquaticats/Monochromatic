/**
 Tests for the live listing the roster card command fetches for one provider.

 WHAT THE FETCH OWES THE CARD: each provider is asked at its own URL with its
 own key variable, a missing key is refused by that variable's name before any
 request is made, and a failure says only the status. The transport is
 scripted, so nothing here reaches a provider, and the keys are invented
 stand-ins.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { CREDENTIAL_MARKER, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  fetchListing,
  LISTING_URL,
} from '../../dist/final/node/index.mjs';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

/**
 Invented stand-in key, never a real one.
 */
const KEY = 'whisker-key-7421';

/**
 Key variable each provider's listing is read with.
 */
const KEY_VARIABLE_OF = {
  synthetic: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
  openrouter: 'TRANSLATION_REPAIR_OPENROUTER_API_KEY',
  bedrock: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY',
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
  deepseek: 'DEEPSEEK_API_KEY',
  xiaomi: 'TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY',
} as const;

/**
 Every provider whose listing the command can fetch.
 */
const PROVIDERS = [
  'synthetic',
  'hyper',
  'openrouter',
  'bedrock',
  'neuralwatt',
  'deepseek',
  'xiaomi',
] as const;

await describe({
  name: fetchListing.name,
  children: [
    ...PROVIDERS
      .map(function providerCase(provider,) {
        /**
         Variable this provider's key is read from.
         */
        const variable = KEY_VARIABLE_OF[provider];
        return it({
          name: `ASKS the ${provider} listing URL with the key of ${variable} and returns the parsed body`,
          fn: async () => {
            const {
              transport,
              exchanges,
            } = recordedTransport({
              replies: [
                {
                  status: 200,
                  bodyText: '{"data":[{"id":"hf:cat/Mittens-1"}]}',
                },
              ],
            },);

            const body = await fetchListing({
              provider,
              env: { [variable]: KEY, },
              transport,
            },);

            expect(body,).toEqual({ data: [{ id: 'hf:cat/Mittens-1', },], },);
            expect(exchanges.map(function seen(exchange,): readonly string[] {
              return [
                exchange.url,
                exchange.method,
                exchange.headers.authorization ?? '',
              ];
            },),).toEqual([[
              LISTING_URL[provider],
              'GET',
              `Bearer ${KEY}`,
            ],],);
          },
        },);
      },),
    it({
      name: 'REFUSES as stated, naming the variable of the provider asked for, when its key is not set, before any request',
      fn: async () => {
        const {
          transport,
          exchanges,
        } = recordedTransport({
          replies: [
            {
              status: 200,
              bodyText: '{}',
            },
          ],
        },);

        const refusal = await rejectionOf(async function fetchWithoutKey(): Promise<void> {
          await fetchListing({
            provider: 'hyper',
            // Only the other providers' keys are set, so reading the wrong variable would not refuse.
            env: {
              [KEY_VARIABLE_OF.synthetic]: KEY,
              [KEY_VARIABLE_OF.openrouter]: KEY,
              [KEY_VARIABLE_OF.bedrock]: KEY,
              [KEY_VARIABLE_OF.neuralwatt]: KEY,
            },
            transport,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is not set; run under mise so sops injects it',
        );
        expect(exchanges,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES as stated, as for an absent key, when the variable is set to nothing',
      fn: async () => {
        const { transport, } = recordedTransport({
          replies: [
            {
              status: 200,
              bodyText: '{}',
            },
          ],
        },);

        const refusal = await rejectionOf(async function fetchWithEmptyKey(): Promise<void> {
          await fetchListing({
            provider: 'bedrock',
            env: { [KEY_VARIABLE_OF.bedrock]: '', },
            transport,
          },);
        },);

        expect(String(refusal,),).toBe(
          'StatedRefusalError: TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is not set; run under mise so sops injects it',
        );
      },
    },),
    it({
      name: 'REFUSES as stated with the URL and the status alone when the provider answers 503',
      fn: async () => {
        const { transport, } = recordedTransport({
          replies: [
            {
              status: 503,
              bodyText: 'the cat is asleep',
            },
          ],
        },);

        const refusal = await rejectionOf(async function fetchFailing(): Promise<void> {
          await fetchListing({
            provider: 'openrouter',
            env: { [KEY_VARIABLE_OF.openrouter]: KEY, },
            transport,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: https://openrouter.ai/api/v1/models answered 503; the reason phrase is the '
            + 'provider\'s wording and is dropped',
        );
      },
    },),
    it({
      name: 'MASKS a key the provider echoes in a row, so the returned body never holds it',
      fn: async () => {
        const { transport, } = recordedTransport({
          replies: [
            {
              status: 200,
              bodyText: `{"data":[{"id":"${KEY}"}]}`,
            },
          ],
        },);

        const body = await fetchListing({
          provider: 'synthetic',
          env: { [KEY_VARIABLE_OF.synthetic]: KEY, },
          transport,
        },);

        expect(body,).toEqual({ data: [{ id: CREDENTIAL_MARKER, },], },);
      },
    },),
  ],
},);
