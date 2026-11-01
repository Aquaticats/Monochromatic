/**
 Unit tests for the system-prompt project-context renderer.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  AssembleContext,
} from '@deepseek-ai/dsh-system-prompt';

import { buildDshAdvisorProjectContext, } from '../dist/final/node/index.mjs';

await describe({
  name: buildDshAdvisorProjectContext.name,
  children: [
    it({
      name: 'assembles through the agent context and renders the sections',
      fn: async () => {
        const assemblyContext = {
          marker: 'context',
        } as unknown as AssembleContext;
        const controller = new AbortController();
        let capturedContext: unknown;
        let capturedSignal: unknown;
        const projectContext = await buildDshAdvisorProjectContext({
          systemPrompt: {
            assemble: (context: unknown,) => {
              capturedContext = context;
              return Promise.resolve({
                sections: [
                  {
                    name: 'unit',
                    order: 0,
                    text: 'advisor project context text',
                  },
                ],
                variables: {},
              },);
            },
          },
          agent: {
            assembleContext: (signal?: AbortSignal,) => {
              capturedSignal = signal;
              return assemblyContext;
            },
          },
          signal: controller.signal,
        } as unknown as Parameters<typeof buildDshAdvisorProjectContext>[0],);
        expect(capturedSignal,).toBe(controller.signal,);
        expect(capturedContext,).toBe(assemblyContext,);
        expect(projectContext
          .includes('advisor project context text',),).toBe(true,);
      },
    },),
  ],
},);
