/**
 Unit tests for the Advisor command dispatcher.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { SessionId, } from '@deepseek-ai/dsh-session';

import {
  createDshAdvisorSessionState,
  runDshAdvisorCommand,
  type RunDshAdvisorCommandOptions,
} from '../dist/final/node/index.mjs';

/**
 Build one minimal command invocation fixture.

 @param rawInput - text following the command name

 @returns command options with inert host surfaces and recorded tool masks

 @example
 ```ts
 const options = commandFixture('off');
 ```
 */
function commandFixture(
  rawInput: string,
): RunDshAdvisorCommandOptions & {
  readonly restrictions: unknown[];
} {
  /**
   Tool restriction filters recorded by the fixture.
   */
  const restrictions: unknown[] = [];
  return {
    getConfig: () => Promise.resolve({} as Awaited<
      ReturnType<RunDshAdvisorCommandOptions['getConfig']>
      >),
    state: createDshAdvisorSessionState({
      initialEnabled: true,
    },),
    llm: {
      listProviders: () => [],
    },
    systemPrompt: {
      assemble: () => ({
        sections: [],
      }),
    },
    workingDirectory: {
      get: () => '/work',
    },
    tools: {
      restrict: (filter: unknown) => {
        restrictions.push(filter,);
        return () => undefined;
      },
    },
    rawInput,
    signal: new AbortController().signal,
    agent: {
      sessionId: SessionId('session-command-fixture',),
      session: {
        deriveMessages: () => [],
      },
      cwd: '/work',
      options: {},
      whenIdle: () => Promise.resolve(),
      appendOperationRecord: () => undefined,
      assembleContext: () => ({}),
    },
    restrictions,
  } as unknown as RunDshAdvisorCommandOptions & {
    readonly restrictions: unknown[];
  };
}

await describe({
  name: runDshAdvisorCommand.name,
  children: [
    it({
      name: 'disables Advisor and masks the tool for the off form',
      fn: async () => {
        const options = commandFixture('off',);
        const result = await runDshAdvisorCommand(options,);
        expect(result,).toEqual({
          kind: 'success',
          text: 'Advisor disabled for this session.',
        },);
        expect(options.state.getEnabled(),).toBe(false,);
        expect(options.restrictions.length,).toBe(1,);
      },
    },),
    it({
      name: 're-enables Advisor and lifts the mask for the on form',
      fn: async () => {
        const options = commandFixture('on',);
        options.state.setEnabled(false,);
        const result = await runDshAdvisorCommand(options,);
        expect(result,).toEqual({
          kind: 'success',
          text: 'Advisor enabled for this session.',
        },);
        expect(options.state.getEnabled(),).toBe(true,);
      },
    },),
  ],
},);
