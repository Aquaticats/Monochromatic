/**
 Tests the run client with Bedrock left off.

 The owner ruled on 2026-10-20 to leave Bedrock off rather than issue a new key. Before this, every run still held the
 expired key, asked Bedrock once, was refused (HTTP 401, `SEAT CREDENTIAL REFUSED google.gemma-4-e2b on bedrock`,
 CuspariaKLSY22 log lines 214 and 215) and only then held it out.

 Every case hands over its own environment, with the Bedrock key set, and a transport that records what it was asked,
 so none reaches a provider or opens the real Bedrock ledger.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  BEDROCK_CREDIT_USD_VAR,
  BEDROCK_LEDGER_PATH_VAR,
} from '@monochromatic-dev/translation-repair-provider';
import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  judgeSeatsFor,
  runClientFrom,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_BEDROCK_ONLY_TEXT, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';

/**
 Name of the Bedrock key variable, named only.
 */
const BEDROCK_KEY_VAR = 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY';

/**
 Name of the Hyper key variable, named only.
 */
const HYPER_KEY_VAR = 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY';

await describe({
  name: 'run client with Bedrock left off',
  concurrency: 1,
  children: [
    it({
      name: 'ASKS BEDROCK NOTHING while it is off: its key is set, the call for the seat only it serves is refused as '
        + 'unreachable, and the transport sees no exchange at all',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-client-bedrock-off-', },);
        /**
         Every exchange the client made.
         */
        const exchanges: TransportExchange[] = [];
        const client = runClientFrom({
          maxHoldMs: TEST_MAX_HOLD_MS,
          env: {
            [BEDROCK_KEY_VAR]: 'calico-not-a-real-key',
            [BEDROCK_LEDGER_PATH_VAR]: join(scratch.path, 'bedrock-spend.jsonl',),
            [BEDROCK_CREDIT_USD_VAR]: '40',
            [HYPER_KEY_VAR]: 'mittens-not-a-real-key',
          },
          transport: async function record(exchange: TransportExchange,) {
            exchanges.push(exchange,);
            return { status: 401, bodyText: '{}', };
          },
        },);
        /**
         What the call for the Bedrock-only seat ended with.
         */
        const refusal = await rejectionOf(async function ask(): Promise<void> {
          await client.chatText({
            modelId: SEAT_BEDROCK_ONLY_TEXT,
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: new AbortController().signal,
          },);
        },);
        expect(refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(exchanges.filter(function isBedrock(exchange,): boolean {
          return exchange.url.includes('bedrock',);
        },).length,).toBe(0,);
      },
    },),

    it({
      name: 'READS BEDROCK DRY FROM THE FIRST READING, so the seat only it serves is named unserved before any call, '
        + 'where CuspariaKLSY22 read it wet once and unserved only from the second JUDGE SEATS line',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-client-bedrock-off-', },);
        const client = runClientFrom({
          maxHoldMs: TEST_MAX_HOLD_MS,
          env: {
            [BEDROCK_KEY_VAR]: 'calico-not-a-real-key',
            [BEDROCK_LEDGER_PATH_VAR]: join(scratch.path, 'bedrock-spend.jsonl',),
            [BEDROCK_CREDIT_USD_VAR]: '40',
            [HYPER_KEY_VAR]: 'mittens-not-a-real-key',
          },
          transport: async function refuseEveryMeter() {
            return { status: 400, bodyText: '{}', };
          },
        },);
        /**
         Dryness of the first reading of the run.
         */
        const dry = await client.providerDryness({ signal: new AbortController().signal, },);
        expect(dry.bedrock,).toBe(true,);
        expect(judgeSeatsFor({ dry, },).unserved.includes(SEAT_BEDROCK_ONLY_TEXT,),).toBe(true,);
      },
    },),
  ],
},);
