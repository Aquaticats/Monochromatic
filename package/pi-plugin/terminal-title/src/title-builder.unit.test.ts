/**
 Tests for pi title builder.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  TITLE_PREFIX,
  titleForEvent,
  titleForTool,
} from './title-builder.ts';

await describe({
  name: titleForTool.name,
  children: [
    it({
      name: 'formats bash with meaningful lifecycle command title',
      fn: async () => {
        expect(
          titleForTool({
            toolName: 'bash',
            args: { command: 'env timeout 10 npm test', },
            tense: 'pre',
          },),
        ).toBe('Running npm test',);
      },
    },),
    it({
      name: 'formats read with smart path context',
      fn: async () => {
        expect(
          titleForTool({
            toolName: 'read',
            args: { path: '/home/user/project/src/index.ts', },
            tense: 'post',
          },),
        ).toBe('Read src/index.ts',);
      },
    },),
    it({
      name: 'formats grep with lifecycle text title',
      fn: async () => {
        expect(
          titleForTool({
            toolName: 'grep',
            args: { pattern: 'TODO', },
            tense: 'pre',
          },),
        ).toBe('Searching for TODO',);
      },
    },),
    it({
      name: 'uses unified generic unknown-tool fallback',
      fn: async () => {
        expect(
          titleForTool({
            toolName: 'mcp__weather',
            args: { city: 'Tokyo', },
            tense: 'post',
          },),
        ).toBe('Ran mcp__weather',);
      },
    },),
  ],
},);

await describe({
  name: titleForEvent.name,
  children: [
    it({
      name: 'prefixes tool execution title',
      fn: async () => {
        expect(
          titleForEvent({
            eventType: 'tool_execution_start',
            data: {
              toolName: 'bash',
              args: { command: 'npm test', },
            },
          },),
        ).toBe(`${TITLE_PREFIX} Running npm test`,);
      },
    },),
    it({
      name: 'renames session start title with lifecycle wording',
      fn: async () => {
        expect(
          titleForEvent({ eventType: 'session_start', data: { reason: 'startup', }, },),
        ).toBe(`${TITLE_PREFIX} Started session: startup`,);
      },
    },),
    it({
      name: 'renames prompt title with lifecycle wording',
      fn: async () => {
        expect(
          titleForEvent({
            eventType: 'before_agent_start',
            data: { prompt: 'Refactor auth', },
          },),
        ).toBe(`${TITLE_PREFIX} Received prompt: Refactor auth`,);
      },
    },),
    it({
      name: 'does not display-cap long prompts',
      fn: async () => {
        /**
         Prompt body longer than historical 60-character display cap.
         */
        const prompt = 'a'.repeat(200,);
        expect(
          titleForEvent({
            eventType: 'before_agent_start',
            data: { prompt, },
          },),
        ).toBe(`${TITLE_PREFIX} Received prompt: ${prompt}`,);
      },
    },),
  ],
},);
