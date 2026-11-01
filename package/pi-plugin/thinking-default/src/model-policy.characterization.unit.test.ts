/**
 Characterization tests for thinking-default policy before shared model-id extraction.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getThinkingDefaultForModel, } from './model-policy.ts';

//region Fixtures

/** Bare GPT-shaped model id fixture. */
const BARE_GPT_MODEL_ID = 'gpt-5.5';

/** Slash-prefixed GPT-shaped model id fixture. */
const SLASH_PREFIXED_GPT_MODEL_ID = 'openai/gpt-5.5';

//endregion Fixtures

await describe({
  name: getThinkingDefaultForModel.name,
  children: [
    it({
      name: 'keeps xhigh for bare GPT ids',
      fn: async function testBareGptDefault() {
        expect(
          getThinkingDefaultForModel({
            model: { id: BARE_GPT_MODEL_ID, },
          },),
        )
          .toBe('xhigh',);
      },
    },),
    it({
      name: 'keeps xhigh for slash-prefixed GPT ids',
      fn: async function testSlashPrefixedGptDefault() {
        expect(
          getThinkingDefaultForModel({
            model: { id: SLASH_PREFIXED_GPT_MODEL_ID, },
          },),
        )
          .toBe('xhigh',);
      },
    },),
  ],
},);
