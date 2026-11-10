/**
 Built-artifact tests for provider-specific forced tool selection.

 Every Pi AI API the package dispatches must map to a selector its own
 provider module forwards, so each branch gets a test named for it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { toolChoiceForApi, } from '../dist/final/node/index.mjs';

/**
 Fixture forced tool name.
 */
const TOOL_NAME = 'submit_fixture_review';

/**
 APIs whose provider modules accept a generic "some function" selector.
 */
const GENERIC_FORCED_APIS: readonly string[] = [
  'openai-completions',
  'openai-responses',
  'azure-openai-responses',
  'openai-codex-responses',
];

/**
 APIs without a dedicated forced-selector form, which take the `any` default.
 */
const DEFAULTED_APIS: readonly string[] = [
  'mistral-conversations',
  'google-generative-ai',
  'google-vertex',
  'bedrock-converse-stream',
  'pi-messages',
];

await describe({
  name: toolChoiceForApi.name,
  children: [
    it({
      name: 'forces the exact named tool for anthropic-messages',
      fn: async () => {
        expect(toolChoiceForApi({
          api: 'anthropic-messages',
          toolName: TOOL_NAME,
        },),).toEqual({
          type: 'tool',
          name: TOOL_NAME,
        },);
      },
    },),
    ...GENERIC_FORCED_APIS.map(function genericForcedCase(api,) {
      return it({
        name: `forces some tool for ${api}`,
        fn: async () => {
          expect(toolChoiceForApi({
            api,
            toolName: TOOL_NAME,
          },),).toBe('required',);
        },
      },);
    },),
    ...DEFAULTED_APIS.map(function defaultedCase(api,) {
      return it({
        name: `falls back to the any default for ${api}`,
        fn: async () => {
          expect(toolChoiceForApi({
            api,
            toolName: TOOL_NAME,
          },),).toBe('any',);
        },
      },);
    },),
  ],
},);
