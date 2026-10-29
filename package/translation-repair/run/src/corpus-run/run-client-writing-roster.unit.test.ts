/**
 Tests that the run client hands the roster of writing providers to its router, where it decides which wet providers
 count against "every provider dry".

 Xiaomi's meter always reads wet and its one seat is a checker, so on a run with Xiaomi alone configured nothing can
 write: a call for a seat only other providers serve ends as the refusal that ends the run (`EveryProviderDryError`).
 Were the roster not handed on, Xiaomi would count as a budget to buy writing with, and the same call would end as one
 unreachable seat (`NoProviderForModelError`) while the run idled. The transport refuses every request, so no case
 waits and no provider is asked.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_NO_OPENROUTER, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { runClientFrom, } from '../../dist/final/node/index.mjs';

await describe({
  name: `${runClientFrom.name} writing roster`,
  concurrency: 1,
  children: [
    it({
      name: 'HANDS THE WRITING ROSTER TO THE ROUTER: with Xiaomi alone configured, a call for a seat only other '
        + 'providers serve ends the run, and is not one unreachable seat beside a wet checker',
      fn: async () => {
        const client = runClientFrom({
          maxHoldMs: TEST_MAX_HOLD_MS,
          env: { TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'tabby-not-a-real-key', },
          transport: async function refuseEveryRequest() {
            return {
              status: 400,
              bodyText: '{}',
            };
          },
          off: [],
        },);
        /**
         What the call ended with.
         */
        const refusal = await rejectionOf(async function ask(): Promise<void> {
          await client.chatText({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: new AbortController().signal,
          },);
        },);
        expect(refusal instanceof NoProviderForModelError,).toBe(false,);
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
      },
    },),
  ],
},);
