/**
 Unit tests for Advisor request message formatting.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { buildAdvisorUserMessageText, } from '../dist/final/node/index.mjs';

await describe({
  name: buildAdvisorUserMessageText.name,
  children: [
    it({
      name: 'builds context-only request text',
      fn: async function testContextOnlyRequestText() {
        expect(buildAdvisorUserMessageText({
          contextText: 'serialized evidence',
        },),).toBe('## Serialized conversation\n\nserialized evidence',);
      },
    },),
    it({
      name: 'places focused question before serialized context',
      fn: async function testFocusedQuestionRequestText() {
        expect(buildAdvisorUserMessageText({
          question: '  Which assumption is weakest?  ',
          contextText: 'serialized evidence',
        },),).toBe(
          '## Focus question\n\nWhich assumption is weakest?\n\n## Serialized conversation\n\nserialized evidence',
        );
      },
    },),
  ],
},);
