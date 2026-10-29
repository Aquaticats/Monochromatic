/**
 Unit tests for the Advisor stream driver surface.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  AssistantMessage,
  AssistantMessageEvent,
} from '@earendil-works/pi-ai';

import { createDshAdvisorStreamDriver, } from '../dist/final/node/index.mjs';

/**
 Build one terminal assistant message fixture.

 @param text - assistant text recorded on the message

 @returns terminal assistant message

 @example
 ```ts
 const message = messageFixture('done');
 ```
 */
function messageFixture(text: string): AssistantMessage {
  return {
    role: 'assistant',
    content: [
      {
        type: 'text',
        text,
      },
    ],
    api: 'dsh',
    provider: 'p',
    model: 'm',
    usage: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
    stopReason: 'stop',
    timestamp: 1,
  } as unknown as AssistantMessage;
}

await describe({
  name: createDshAdvisorStreamDriver.name,
  children: [
    it({
      name: 'buffers early events and replays them in publish order',
      fn: async () => {
        const driver = createDshAdvisorStreamDriver();
        driver.publish({
          type: 'text-delta',
          text: 'a',
        } as unknown as AssistantMessageEvent,);
        driver.publish({
          type: 'text-delta',
          text: 'b',
        } as unknown as AssistantMessageEvent,);
        driver.conclude(messageFixture('done',),);
        const seen: string[] = [];
        for await (const event of driver.surface) {
          seen.push(event.type,);
        }
        expect(seen,).toEqual([
          'text-delta',
          'text-delta',
        ],);
        const result = await driver.surface.result();
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'suspends iteration until an event arrives',
      fn: async () => {
        const driver = createDshAdvisorStreamDriver();
        const iterator = driver.surface[Symbol.asyncIterator]();
        /**
         Pending pull started before any event is published.
         */
        const pending = iterator.next();
        driver.publish({
          type: 'text-delta',
          text: 'late',
        } as unknown as AssistantMessageEvent,);
        const first = await pending;
        expect(first.done,).toBe(false,);
        expect((first.value as { readonly type: string }).type,)
          .toBe('text-delta',);
        driver.conclude(messageFixture('done',),);
        const end = await iterator.next();
        expect(end.done,).toBe(true,);
      },
    },),
    it({
      name: 'throws when the stream concludes twice or publishes after conclusion',
      fn: async () => {
        const driver = createDshAdvisorStreamDriver();
        driver.conclude(messageFixture('done',),);
        let doubleConclude: unknown;
        try {
          driver.conclude(messageFixture('again',),);
        }
        catch (error) {
          doubleConclude = error;
        }
        expect(Error.isError(doubleConclude,),).toBe(true,);
        let latePublish: unknown;
        try {
          driver.publish({
            type: 'text-delta',
            text: 'late',
          } as unknown as AssistantMessageEvent,);
        }
        catch (error) {
          latePublish = error;
        }
        expect(Error.isError(latePublish,),).toBe(true,);
      },
    },),
  ],
},);
