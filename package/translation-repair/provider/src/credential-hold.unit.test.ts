/**
 Tests the credential holds a router keeps (zheermao19, 2026-10-20): a provider
 whose credential was refused is held out for every model for the rest of the
 run, the hold is said once, and the line names where the seat goes. The
 routed behaviour is in `provider-router-credential-hold.unit.test.ts`. Cat-
 themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  exchangeFailureLogText,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { SEAT_HYPER_TEXT_BEDROCK, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type BudgetView,
  createCredentialHolds,
  credentialRefusalLine,
  foldCredentialHolds,
  holdOnCredentialRefusal,
  reachPastCredentialHolds,
} from '../dist/final/node/index.mjs';

/**
 Status a provider answers with when it rejects the request's credential.
 */
const HTTP_UNAUTHORIZED = 401;

/**
 The refusal a provider raises when it rejects the credential.

 @param provider - provider that refused

 @returns The failure the router catches

 @example
 ```ts
 const error = refusalFrom({ provider: 'bedrock', },);
 ```
 */
function refusalFrom({ provider, }: { readonly provider: ProviderName; },): ProviderHttpError {
  return new ProviderHttpError({
    provider,
    status: HTTP_UNAUTHORIZED,
    bodyText: 'the security token has expired',
  },);
}

/**
 Reach of a model Bedrock, Hyper and OpenRouter serve.
 */
const BEDROCK_HYPER_OPENROUTER = {
  neuralwatt: false,
  synthetic: false,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: true,
};

await describe({
  name: 'the credential holds a router keeps (zheermao19)',
  children: [
    describe({
      name: createCredentialHolds.name,
      children: [
        it({
          name: 'HOLDS A PROVIDER once and says so only to the call that began the hold, for every model',
          fn: async () => {
            const holds = createCredentialHolds();

            expect(holds.isHeld({ provider: 'bedrock', },),).toBe(false,);
            expect(holds.hold({ provider: 'bedrock', },),).toBe(true,);
            expect(holds.hold({ provider: 'bedrock', },),).toBe(false,);
            expect(holds.isHeld({ provider: 'bedrock', },),).toBe(true,);
            expect(holds.isHeld({ provider: 'hyper', },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: reachPastCredentialHolds.name,
      children: [
        it({
          name: 'TAKES OUT A HELD PROVIDER THAT SERVES THE MODEL and lists it, and lists none that does not serve it',
          fn: async () => {
            const holds = createCredentialHolds();
            holds.hold({ provider: 'bedrock', },);
            holds.hold({ provider: 'synthetic', },);

            expect(reachPastCredentialHolds({ reach: BEDROCK_HYPER_OPENROUTER, holds, },),).toEqual({
              reach: {
                neuralwatt: false,
                synthetic: false,
                hyper: true,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: true,
              },
              refused: ['bedrock',],
            },);
          },
        },),
        it({
          name: 'LEAVES THE REACH AS IT STANDS where no serving provider is held',
          fn: async () => {
            expect(reachPastCredentialHolds({ reach: BEDROCK_HYPER_OPENROUTER, holds: createCredentialHolds(), },),).toEqual({
              reach: BEDROCK_HYPER_OPENROUTER,
              refused: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: credentialRefusalLine.name,
      children: [
        it({
          name: 'NAMES THE PROVIDERS THE SEAT MOVES TO in the order they are tried, saying they are the ones that can take '
            + 'a call now',
          fn: async () => {
            const error = refusalFrom({ provider: 'bedrock', },);
            const said = exchangeFailureLogText({ error, },);

            expect(credentialRefusalLine({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              provider: 'bedrock',
              error,
              outlook: {
                open: ['hyper', 'openrouter',],
                dry: [],
                heldForModel: [],
              },
            },),).toBe(
              `SEAT CREDENTIAL REFUSED ${SEAT_HYPER_TEXT_BEDROCK} on bedrock: ${said}. `
                + 'bedrock is held out for every model for the rest of the run, because one credential serves them all. '
                + `${SEAT_HYPER_TEXT_BEDROCK} moves to hyper, then openrouter `
                + '(every provider that serves it and can take a call now). '
                + 'Renew bedrock\'s credential before the next run, or leave it.',
            );
          },
        },),
        it({
          name: 'SAYS THE SEAT IS WITHHELD only where no provider serves it, and NOT WITHHELD where providers serve it '
            + 'but are dry or held for the model, naming each',
          fn: async () => {
            const error = refusalFrom({ provider: 'bedrock', },);

            expect(credentialRefusalLine({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              provider: 'bedrock',
              error,
              outlook: {
                open: [],
                dry: [],
                heldForModel: [],
              },
            },),).toContain(
              `No provider that serves ${SEAT_HYPER_TEXT_BEDROCK} is left, so the seat is withheld for the rest of the run.`,
            );
            expect(credentialRefusalLine({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              provider: 'bedrock',
              error,
              outlook: {
                open: [],
                dry: ['hyper',],
                heldForModel: ['openrouter',],
              },
            },),).toContain(
              `No provider that serves ${SEAT_HYPER_TEXT_BEDROCK} can take a call now `
                + '(hyper is dry, openrouter is held for this model), so the seat is not withheld on this refusal: '
                + 'the router goes on by its rules for a dry or held provider. ',
            );
          },
        },),
      ],
    },),

    describe({
      name: holdOnCredentialRefusal.name,
      children: [
        it({
          name: 'WARNS ONCE PER PROVIDER, leaving out of the seat\'s next providers the one refused, a dry one and one '
            + 'the model\'s stream bound holds out, and reads the budgets only for the call that began the hold',
          fn: async () => {
            const holds = createCredentialHolds();
            const lines: string[] = [];
            const rl = levelCapturingLogger({ lines, },);
            const error = refusalFrom({ provider: 'bedrock', },);
            /** Budget reads made. */
            let reads = 0;
            const budgets = {
              read: async function readOpenrouterDry(): Promise<BudgetView> {
                reads += 1;
                return {
                  neuralwatt: true,
                  synthetic: true,
                  hyper: false,
                  bedrock: false,
                  deepseek: false,
                  xiaomi: false,
                  anthropic: false,
                  openrouter: false,
                };
              },
            };
            const boundHolds = {
              hold: function holdNothing(): void {
                return undefined;
              },
              remainingMs: function hyperHeldForThisModel(
                { provider, modelId, }: { readonly provider: ProviderName; readonly modelId: string; },
              ): number {
                return ((provider === 'hyper') && (modelId === SEAT_HYPER_TEXT_BEDROCK)) ? 1_000 : 0;
              },
            };
            const call = {
              rl,
              provider: 'bedrock' as const,
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              error,
              reach: BEDROCK_HYPER_OPENROUTER,
              holds,
              boundHolds,
              budgets,
              signal: new AbortController().signal,
            };

            await holdOnCredentialRefusal(call,);
            await holdOnCredentialRefusal(call,);

            expect(lines.length,).toBe(1,);
            expect(reads,).toBe(1,);
            expect(lines[0],).toContain(`${SEAT_HYPER_TEXT_BEDROCK} moves to openrouter `,);
            expect(lines[0],).toContain('openrouter (every provider',);
            expect(lines[0],).not.toContain('hyper is',);
            expect(holds.isHeld({ provider: 'bedrock', },),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES A DRY PROVIDER AND A PROVIDER HELD FOR THE MODEL, each under its own word, when neither can take the call',
          fn: async () => {
            const lines: string[] = [];

            await holdOnCredentialRefusal({
              rl: levelCapturingLogger({ lines, },),
              provider: 'bedrock',
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              error: refusalFrom({ provider: 'bedrock', },),
              reach: BEDROCK_HYPER_OPENROUTER,
              holds: createCredentialHolds(),
              boundHolds: {
                hold: function holdNothing(): void {
                  return undefined;
                },
                remainingMs: function hyperHeld({ provider, }: { readonly provider: ProviderName; },): number {
                  return (provider === 'hyper') ? 1_000 : 0;
                },
              },
              budgets: {
                read: async function openrouterDry(): Promise<BudgetView> {
                  return {
                    neuralwatt: false,
                    synthetic: false,
                    hyper: false,
                    bedrock: false,
                    deepseek: false,
                    xiaomi: false,
                    anthropic: false,
                    openrouter: true,
                  };
                },
              },
              signal: new AbortController().signal,
            },);

            expect(lines[0],).toContain('can take a call now (hyper is held for this model, openrouter is dry), ',);
          },
        },),
      ],
    },),

    describe({
      name: foldCredentialHolds.name,
      children: [
        it({
          name: 'READS A PROVIDER WHOSE CREDENTIAL WAS REFUSED AS DRY and leaves every other provider as the meters read it',
          fn: async () => {
            const holds = createCredentialHolds();
            holds.hold({ provider: 'bedrock', },);
            const metered = {
              neuralwatt: false,
              synthetic: true,
              hyper: false,
              bedrock: false,
              deepseek: false,
              xiaomi: false,
              anthropic: false,
              openrouter: false,
            };

            expect(foldCredentialHolds({ dry: metered, holds, },),).toEqual({
              neuralwatt: false,
              synthetic: true,
              hyper: false,
              bedrock: true,
              deepseek: false,
              xiaomi: false,
              anthropic: false,
              openrouter: false,
            },);
            expect(foldCredentialHolds({ dry: metered, holds: createCredentialHolds(), },),).toEqual(metered,);
          },
        },),
      ],
    },),
  ],
},);
