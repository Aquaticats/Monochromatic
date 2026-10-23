/**
 Tests that the credential holds `configureProviders` builds reach the `METERS` line its budgets print.

 CuspariaKLSY22 held Bedrock out on the first call (HTTP 401) and went on printing `bedrock=wet` on every `METERS` line.
 The holds live beside the router, and the line is printed by the budgets, so the two have to be handed the same holds.

 Every case hands over its own environment and a transport that answers every meter with the same stub, so none reaches
 a provider or opens the real Bedrock ledger.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  BEDROCK_CREDIT_USD_VAR,
  BEDROCK_LEDGER_PATH_VAR,
} from '@monochromatic-dev/translation-repair-provider';
import { consoleLinesDuring, } from '@monochromatic-dev/translation-repair-provider/ts/neuralwatt-ledger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { configureProviders, } from '../../dist/final/node/index.mjs';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';

await describe({
  name: configureProviders.name,
  // One case at a time: each captures the console, which is one stub for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS A PROVIDER WHOSE CREDENTIAL WAS REFUSED AS DRY ON THE METERS LINE, through the holds it returns, '
        + 'where the same provider read wet before the hold',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-providers-held-', },);
        /**
         Providers with Bedrock on, its ledger and credit in the scratch directory.
         */
        const providers = configureProviders({
          maxHoldMs: TEST_MAX_HOLD_MS,
          env: {
            TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: 'calico-not-a-real-key',
            [BEDROCK_LEDGER_PATH_VAR]: join(scratch.path, 'bedrock-spend.jsonl',),
            [BEDROCK_CREDIT_USD_VAR]: '40',
          },
          transport: async function refuseEveryMeter() {
            return { status: 400, bodyText: '{}', };
          },
          off: [],
        },);
        const lines = await consoleLinesDuring({
          run: async function readAfterTheRefusal(): Promise<void> {
            providers.credentialHolds.hold({ provider: 'bedrock', },);
            await providers.budgets.readStates({ signal: new AbortController().signal, },);
          },
          sinon: ctx.sinon,
        },);
        expect(lines.some(function isMeters(line,): boolean {
          return line.includes(' METERS ',) && line.includes('bedrock=dry',);
        },),).toBe(true,);
      },
    },),
  ],
},);
