/**
 Tests for rendering what a model exchange threw: the marked class keeps its
 sentence, a provider status failure keeps its status and drops its body, and
 anything else is named by class. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { PROVIDER_ORDER, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  exchangeFailureLogText,
  exchangeFailureText,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { SyntheticRequestTooLargeError, } from '../dist/final/node/index.mjs';
import {
  quotingFailure,
  WHISKER_KEY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';

await describe({
  name: exchangeFailureText.name,
  children: [
    it({
      name: 'REPEATS A MARKED CLASS\'S OWN SENTENCE, which is authored from names and counts',
      fn: async () => {
        expect(
          exchangeFailureText({
            error: new NoProviderForModelError({
              modelId: 'hf:zai-org/GLM-5.3-Flash',
              reason: 'every provider reads dry',
            },),
          },),
        ).toBe('no provider can take hf:zai-org/GLM-5.3-Flash: every provider reads dry',);
      },
    },),

    it({
      name: 'NAMES THE CLASS AND THE HTTP STATUS OF A PROVIDER FAILURE and drops the body excerpt its message carries',
      fn: async () => {
        /**
         Failure whose message carries the provider's body excerpt.
         */
        const failure = new ProviderHttpError({
          provider: 'synthetic',
          status: 429,
          bodyText: `the cat key ${WHISKER_KEY} is over its weekly credit`,
        },);

        expect(failure.message.includes(WHISKER_KEY,),).toBe(true,);
        expect(exchangeFailureText({ error: failure, },),).toBe(
          'refused by ProviderHttpError from synthetic with HTTP 429',
        );
      },
    },),

    it({
      name: 'NAMES THE PROVIDER THAT SERVED THE CALL beside the class and the status, for each provider, so a Bedrock '
        + 'refusal never reads as a Synthetic one',
      fn: async () => {
        /**
         What each provider's refusal says, in the order the owner spends on them.
         */
        const said = PROVIDER_ORDER.map(function sayRefusal(provider,): string {
          return exchangeFailureText({
            error: new ProviderHttpError({ provider, status: 401, bodyText: 'the cat token expired', },),
          },);
        },);

        expect(said,).toStrictEqual([
          'refused by ProviderHttpError from neuralwatt with HTTP 401',
          'refused by ProviderHttpError from synthetic with HTTP 401',
          'refused by ProviderHttpError from bedrock with HTTP 401',
          'refused by ProviderHttpError from deepseek with HTTP 401',
          'refused by ProviderHttpError from xiaomi with HTTP 401',
          'refused by ProviderHttpError from anthropic with HTTP 401',
          'refused by ProviderHttpError from hyper with HTTP 401',
          'refused by ProviderHttpError from openrouter with HTTP 401',
        ],);
      },
    },),

    it({
      name: 'LOG TEXT NAMES THE PROVIDER BEFORE ITS WORDS, so the line says whose words follow',
      fn: async () => {
        expect(exchangeFailureLogText({
          error: new ProviderHttpError({
            provider: 'bedrock',
            status: 401,
            bodyText: 'The security token included in the request has expired.',
          },),
        },),).toBe(
          'refused by ProviderHttpError from bedrock with HTTP 401 '
            + '(the provider said: "The security token included in the request has expired.")',
        );
      },
    },),

    it({
      name: 'NAMES THE CLASS ALONE OF AN UNMARKED RUNTIME FAILURE whose message quotes a header value',
      fn: async () => {
        expect(quotingFailure().message.includes(WHISKER_KEY,),).toBe(true,);
        expect(exchangeFailureText({ error: quotingFailure(), },),).toBe('refused by TypeError',);
      },
    },),

    it({
      name: 'LOG TEXT ADDS THE PROVIDER\'S WORDS labelled as its own, quoted so a line break or a quote cannot end '
        + 'the line or the quotation early',
      fn: async () => {
        /**
         The words as one quoted line, every control character and line separator written as an escape.
         */
        const quoted = String.raw`"the cat\nsaid \"no\"\u2028and\u2029left\u0085\u001b[0m"`;
        expect(exchangeFailureLogText({
          error: new ProviderHttpError({
            provider: 'synthetic',
            status: 400,
            bodyText: 'the cat\nsaid "no"\u2028and\u2029left\u0085\u001B[0m',
          },),
        },),).toBe(
          `refused by ProviderHttpError from synthetic with HTTP 400 (the provider said: ${quoted})`,
        );
      },
    },),

    it({
      name: 'LOG TEXT ADDS THE PROVIDER\'S WORDS TO A MARKED SUBCLASS\'S OWN SENTENCE, the words its message withholds',
      fn: async () => {
        /**
         Refusal whose message withholds the gateway's words and whose excerpt keeps them.
         */
        const failure = new SyntheticRequestTooLargeError({
          status: 400,
          bodyText: 'cat gateway: could not parse the body',
          bodyBytes: 5_000_000,
        },);
        expect(exchangeFailureLogText({ error: failure, },),).toBe(
          `${exchangeFailureText({ error: failure, },)} (the provider said: "cat gateway: could not parse the body")`,
        );
      },
    },),

    it({
      name: 'LOG TEXT IS THE PLAIN TEXT where the provider said nothing: a blank excerpt, another class, or a thrown value',
      fn: async () => {
        /**
         Failure whose body held only whitespace.
         */
        const blank = new ProviderHttpError({ provider: 'synthetic', status: 502, bodyText: ' \n ', },);
        /**
         Failure whose body held only a zero-width space, which shows a reader nothing.
         */
        const invisible = new ProviderHttpError({ provider: 'synthetic', status: 502, bodyText: '\u200B', },);
        expect(exchangeFailureLogText({ error: blank, },),).toBe(exchangeFailureText({ error: blank, },),);
        expect(exchangeFailureLogText({ error: invisible, },),).toBe(exchangeFailureText({ error: invisible, },),);
        expect(exchangeFailureLogText({ error: quotingFailure(), },),).toBe('refused by TypeError',);
        expect(exchangeFailureLogText({ error: 'a bare string', },),).toBe('refused by a thrown value that is not an Error',);
      },
    },),

    it({
      name: 'NAMES A THROWN VALUE THAT IS NOT AN ERROR by the stand-in phrase',
      fn: async () => {
        expect(exchangeFailureText({ error: `the cat key ${WHISKER_KEY}`, },),).toBe(
          'refused by a thrown value that is not an Error',
        );
      },
    },),
  ],
},);
