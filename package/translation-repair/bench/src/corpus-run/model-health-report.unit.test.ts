/**
 Tests for the walk over a roster the model health command makes.

 THE ONE DISTINCTION THE REPORT DRAWS is a model that answered badly against a
 model that could not be asked: the first is evidence about the model and the
 second about the provider. So each case reads the level a line was logged at
 (a warning is what a log reader looks for) and the exit code, which is what a
 caller of a diagnostic reads.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { reportModelHealth, } from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  refusedClient,
  statusFailureLogText,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import {
  modelReplyScriptedClient,
  type ScriptedModelReply,
} from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/model-reply-scripted-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 First model of every roster.
 */
const TABBY = 'hf:openai/gpt-oss-120b' as const;

/**
 Second model of every roster.
 */
const MOUSER = 'minimax-m3' as const;

/**
 Reply in the shape asked for.
 */
const HEALTHY: ScriptedModelReply = {
  kind: 'ok',
  text: '{"count":2,"first":"Mittens"}',
};

/**
 Walks a roster of scripted models.

 @param replies - what each model does, by model id, in roster order

 @returns Lines logged with their levels, exit code and the models asked

 @example
 ```ts
 const { lines, exitCode, } = await walkWith({ replies: { [TABBY]: HEALTHY, }, },);
 ```
 */
async function walkWith(
  { replies, }: { readonly replies: Readonly<Record<string, ScriptedModelReply>>; },
): Promise<{
  readonly lines: readonly string[];
  readonly exitCode: number;
  readonly asked: readonly string[];
}> {
  const { client, requests, } = modelReplyScriptedClient({ replies: new Map(Object.entries(replies,),), },);
  const lines: string[] = [];
  const exitCode = await reportModelHealth({
    client,
    roster: [
      TABBY,
      MOUSER,
    ].filter(function isScripted(modelId,): boolean {
      return Object.hasOwn(
        replies,
        modelId,
      );
    },),
    timeoutMs: HANG_STOP_MS,
    l: levelCapturingLogger({ lines, },),
  },);
  return {
    lines,
    exitCode,
    asked: requests.map(function modelOf(request,): string {
      return request.modelId;
    },),
  };
}

await describe({
  name: reportModelHealth.name,
  concurrency: 1,
  children: [
    it({
      name: 'COUNTS every model answered and returns zero when each one could be asked, in the plural',
      fn: async () => {
        const { lines, exitCode, asked, } = await walkWith({
          replies: {
            [TABBY]: HEALTHY,
            [MOUSER]: {
              kind: 'schema-mismatch',
              rawText: 'meow',
              detail: 'reply is not JSON',
            },
          },
        },);

        expect(exitCode,).toBe(0,);
        expect(asked,).toEqual([
          'hf:openai/gpt-oss-120b',
          'minimax-m3',
        ],);
        expect(lines,).toEqual([
          'info [reportModelHealth] hf:openai/gpt-oss-120b: ok',
          String.raw`info [reportModelHealth] hf:openai/gpt-oss-120b: raw reply opening (at most 300 UTF-16 units): "{\"count\":2,\"first\":\"Mittens\"}"`,
          'info [reportModelHealth] minimax-m3: schema-mismatch -- reply is not JSON',
          'info [reportModelHealth] minimax-m3: raw reply opening (at most 300 UTF-16 units): "meow"',
          'info [reportModelHealth] ROSTER 2 of 2 models answered',
        ],);
      },
    },),
    it({
      name: 'COUNTS a roster of one in the singular',
      fn: async () => {
        const { lines, exitCode, } = await walkWith({ replies: { [TABBY]: HEALTHY, }, },);

        expect(exitCode,).toBe(0,);
        expect(lines.at(-1,),).toBe('info [reportModelHealth] ROSTER 1 of 1 model answered',);
      },
    },),
    it({
      name: 'REPORTS an empty roster as none of none answered and returns zero',
      fn: async () => {
        const { lines, exitCode, asked, } = await walkWith({ replies: {}, },);

        expect(exitCode,).toBe(0,);
        expect(asked,).toEqual([],);
        expect(lines,).toEqual(['info [reportModelHealth] ROSTER 0 of 0 models answered',],);
      },
    },),
    it({
      name: 'WARNS that a model could not be asked, goes on to the models after it and returns one',
      fn: async () => {
        const { lines, exitCode, asked, } = await walkWith({
          replies: {
            [TABBY]: {
              kind: 'throws',
              error: new TypeError('the cat flap is shut',),
            },
            [MOUSER]: HEALTHY,
          },
        },);

        expect(exitCode,).toBe(1,);
        expect(asked,).toEqual([
          'hf:openai/gpt-oss-120b',
          'minimax-m3',
        ],);
        expect(lines,).toEqual([
          'warn [reportModelHealth] hf:openai/gpt-oss-120b: UNREACHABLE: refused by TypeError',
          'info [reportModelHealth] minimax-m3: ok',
          String.raw`info [reportModelHealth] minimax-m3: raw reply opening (at most 300 UTF-16 units): "{\"count\":2,\"first\":\"Mittens\"}"`,
          'info [reportModelHealth] ROSTER 1 of 2 models answered; unreachable: hf:openai/gpt-oss-120b',
        ],);
      },
    },),
    it({
      name: 'NAMES every model that could not be asked, in roster order, on the closing line',
      fn: async () => {
        const { lines, exitCode, } = await walkWith({
          replies: {
            [TABBY]: {
              kind: 'throws',
              error: new TypeError('the cat flap is shut',),
            },
            [MOUSER]: {
              kind: 'throws',
              error: new RangeError('the cat flap is wide',),
            },
          },
        },);

        expect(exitCode,).toBe(1,);
        expect(lines.at(-1,),).toBe(
          'info [reportModelHealth] ROSTER 0 of 2 models answered; unreachable: hf:openai/gpt-oss-120b, minimax-m3',
        );
      },
    },),
    it({
      name: 'WARNS with the status and the provider\'s own words for a provider status failure',
      fn: async () => {
        const { lines, exitCode, } = await walkWith({
          replies: {
            [TABBY]: {
              kind: 'throws',
              error: new ProviderHttpError({
                provider: 'synthetic',
                status: 402,
                bodyText: 'no treats left',
              },),
            },
          },
        },);

        expect(exitCode,).toBe(1,);
        expect(lines,).toEqual([
          'warn [reportModelHealth] hf:openai/gpt-oss-120b: UNREACHABLE: refused by ProviderHttpError from synthetic with HTTP 402 '
            + '(the provider said: "no treats left")',
          'info [reportModelHealth] ROSTER 0 of 1 model answered; unreachable: hf:openai/gpt-oss-120b',
        ],);
      },
    },),
    it({
      name: 'WARNS WITH THE STATUS AND THE PROVIDER\'S WORDS, the key its refusal echoed masked, for a model '
        + 'asked over the real client and transport',
      fn: async ctx => {
        /**
         Lines logged with their levels while the roster was walked.
         */
        const lines: string[] = [];
        /**
         Exit code of the walk over one model whose provider refuses the key.
         */
        const exitCode = await reportModelHealth({
          client: refusedClient({
            sinon: ctx.sinon,
            status: 401,
          },),
          roster: [TABBY,],
          timeoutMs: HANG_STOP_MS,
          l: levelCapturingLogger({ lines, },),
        },);

        expect(exitCode,).toBe(1,);
        expect(lines,).toEqual([
          `warn [reportModelHealth] ${TABBY}: UNREACHABLE: ${statusFailureLogText({ provider: 'synthetic', status: 401, },)}`,
          `info [reportModelHealth] ROSTER 0 of 1 model answered; unreachable: ${TABBY}`,
        ],);
      },
    },),
  ],
},);
