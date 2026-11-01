import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import currentTimeContext, { CURRENT_TIME_CONTEXT_TYPE, } from './index.ts';
import {
  createBeforeAgentStartEvent,
  createExtensionContext,
  fakePiApi,
  getBeforeAgentStartHandler,
} from './pi-test-harness.ts';
import { isTimeContextContent, } from './time-context-shape.ts';

await describe({
  name: currentTimeContext.name,
  children: [
    it({
      name: 'registers a before_agent_start handler',
      fn: async function testRegistersHandler() {
        const { api, registrations, } = fakePiApi();
        currentTimeContext(api,);

        expect(registrations,).toContain('event:before_agent_start',);
      },
    },),
    it({
      name: 'returns one hidden current-time custom message',
      fn: async function testReturnsHiddenCustomMessage() {
        const { api, handlers, } = fakePiApi();
        currentTimeContext(api,);

        const handler = getBeforeAgentStartHandler(handlers,);
        const result = await handler(
          createBeforeAgentStartEvent(),
          createExtensionContext(),
        );
        if (result === undefined)
          throw new Error('before_agent_start handler returned no result',);

        expect(result.message?.customType,).toBe(CURRENT_TIME_CONTEXT_TYPE,);
        expect(typeof result.message?.content,).toBe('string',);
        if ((typeof result.message?.content) !== 'string')
          throw new Error('current-time context content was not a string',);
        expect(isTimeContextContent(result.message.content,),).toBe(true,);
        expect(result.message.display,).toBe(false,);
      },
    },),
  ],
},);
