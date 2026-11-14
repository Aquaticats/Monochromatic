/**
 What a message the driver sent Pi came to, read from the state Pi was left in and the words Pi refused with.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageToPiCameTo, } from '../dist/final/node/index.mjs';

/**
 The words Pi 1.5.2 refuses a prompt with when it is streaming, quoted from `dist/core/agent-session.js`.
 */
const PI_WORDS_FOR_WORKING = 'Agent is already processing. Specify streamingBehavior (\'steer\' or \'followUp\') to queue the message.';

/**
 The words the package's own stand-in refuses a prompt with.
 */
const STAND_IN_WORDS_FOR_WORKING = 'agent is already running';

await describe({
  name: messageToPiCameTo.name,
  children: [
    it({
      name: 'reads a taken message from Pi answer, whatever state Pi was left in',
      fn: async () => {
        expect(messageToPiCameTo({
          answer: {
            ending: 'answered',
            record: { success: true, },
          },
          piWorks: true,
        },),).toBe('taken',);
      },
    },),
    it({
      name: 'reads a refusal that names a Pi at work as at-work, even when Pi rests by the time it is read',
      fn: async () => {
        expect(messageToPiCameTo({
          answer: {
            ending: 'answered',
            record: {
              success: false,
              error: PI_WORDS_FOR_WORKING,
            },
          },
          piWorks: false,
        },),).toBe('at-work',);
        expect(messageToPiCameTo({
          answer: {
            ending: 'answered',
            record: {
              success: false,
              error: STAND_IN_WORDS_FOR_WORKING,
            },
          },
          piWorks: false,
        },),).toBe('at-work',);
      },
    },),
    it({
      name: 'reads a refusal with nothing about work as Pi own refusal of the message',
      fn: async () => {
        expect(messageToPiCameTo({
          answer: {
            ending: 'answered',
            record: {
              success: false,
              error: 'no key for this provider',
            },
          },
          piWorks: false,
        },),).toBe('refused',);
        // A refusal with no words at all, while Pi is seen working, is still no answer about the message.
        expect(messageToPiCameTo({
          answer: {
            ending: 'answered',
            record: { success: false, },
          },
          piWorks: true,
        },),).toBe('at-work',);
      },
    },),
    it({
      name: 'keeps a wait that saw no answer as unanswered',
      fn: async () => {
        expect(messageToPiCameTo({
          answer: {
            ending: 'silence',
            problem: 'no answer in 30000 ms',
          },
          piWorks: false,
        },),).toBe('unanswered',);
      },
    },),
  ],
},);
