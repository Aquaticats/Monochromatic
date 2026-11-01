import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { alreadyRaised, } from './reminder-history.ts';
import { uncertaintyLead, } from './reminder-lead.ts';

/**
 Opening sentence of the reminder the tests look for.
 */
const LEAD = uncertaintyLead('probably',);

/**
 Transcript line for a message the human typed, which closes the window.
 */
const HUMAN_LINE = JSON.stringify({
  type: 'user',
  origin: { kind: 'human', },
  message: { content: 'next task', },
},);

/**
 Transcript line for the feedback record Claude Code writes when this hook blocks.

 @param lead - opening sentence of the reminder the record carries

 @returns one JSONL line

 @example
 ```ts
 feedbackLine(LEAD);
 ```
 */
function feedbackLine(lead: string,): string {
  return JSON.stringify({
    type: 'user',
    message: { content: `Stop hook feedback:\n${lead} Search for evidence.`, },
  },);
}

await describe({
  name: alreadyRaised.name,
  children: [
    it({
      name: 'is false with no transcript',
      fn: async () => {
        expect(alreadyRaised({ lead: LEAD, transcriptLines: [], },),).toBe(false,);
      },
    },),
    it({
      name: 'is true when the reminder was raised since the last human turn',
      fn: async () => {
        expect(alreadyRaised({
          lead: LEAD,
          transcriptLines: [HUMAN_LINE, feedbackLine(LEAD,), '{"type":"assistant"}',],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'is false when the reminder belongs to an earlier turn',
      fn: async () => {
        expect(alreadyRaised({
          lead: LEAD,
          transcriptLines: [feedbackLine(LEAD,), HUMAN_LINE, '{"type":"assistant"}',],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'is false when only a different trigger was raised',
      fn: async () => {
        expect(alreadyRaised({
          lead: LEAD,
          transcriptLines: [HUMAN_LINE, feedbackLine(uncertaintyLead('maybe',),),],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'is false for a record that only quotes the reminder',
      fn: async () => {
        /**
         Tool result that prints the feedback text, which is not a block the agent received.
         */
        const quoted = JSON.stringify({
          type: 'user',
          toolUseResult: { stdout: 'x', },
          message: { content: `Stop hook feedback:\n${LEAD}`, },
        },);
        /**
         Assistant message that repeats the sentence.
         */
        const assistantQuote = JSON.stringify({ type: 'assistant', message: { content: LEAD, }, },);
        /**
         User record whose content does not open with the feedback prefix.
         */
        const typed = JSON.stringify({ type: 'user', message: { content: `Why did it say: ${LEAD}`, }, },);

        expect(alreadyRaised({
          lead: LEAD,
          transcriptLines: [HUMAN_LINE, quoted, assistantQuote, typed,],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'skips a truncated line',
      fn: async () => {
        expect(alreadyRaised({
          lead: LEAD,
          transcriptLines: [HUMAN_LINE, feedbackLine(LEAD,), '{"type":"user","message":{"content":"Stop hook feedback',],
        },),).toBe(true,);
      },
    },),
  ],
},);
