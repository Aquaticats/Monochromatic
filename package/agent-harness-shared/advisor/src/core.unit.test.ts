/**
 Unit tests for Advisor-specific core helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareAdvisorArguments, } from '../dist/final/node/index.mjs';

await describe({
  name: prepareAdvisorArguments.name,
  children: [
    it({
      name: 'normalizes raw string arguments',
      fn: async function testRawStringArguments() {
        expect(prepareAdvisorArguments('expensive/reviewer',),).toEqual({
          model: 'expensive/reviewer',
        },);
      },
    },),
    it({
      name: 'normalizes focused question arguments',
      fn: async function testFocusedQuestionArguments() {
        expect(prepareAdvisorArguments({
          question: '  Did I miss verification?  ',
        },),).toEqual({
          question: 'Did I miss verification?',
        },);
      },
    },),
    it({
      name: 'normalizes combined model and question arguments',
      fn: async function testCombinedModelAndQuestionArguments() {
        expect(prepareAdvisorArguments({
          model: 'expensive/reviewer',
          question: 'Which assumption is weakest?',
        },),).toEqual({
          model: 'expensive/reviewer',
          question: 'Which assumption is weakest?',
        },);
      },
    },),
    it({
      name: 'rejects unsupported argument fields',
      fn: async function testUnsupportedArgumentFields() {
        expect(function parseBadField() {
          prepareAdvisorArguments({ prompt: 'Use old name', },);
        },).toThrow('advisor: unsupported argument fields: prompt',);
      },
    },),
  ],
},);
