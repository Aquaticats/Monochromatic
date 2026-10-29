import type { StopInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { stopRemindersDecision, } from './index.ts';
import {
  findParkedDecision,
  PARKED_DECISION_REMINDER_PREFIX,
  parkedDecisionLead,
  parkedDecisionReminder,
} from './parked-decision.ts';
import { NO_MATCH, } from './uncertainty.ts';

/**
 Stop event with every field the decision reads, so each test varies one thing.

 @param overrides - fields replacing defaults on the built event

 @returns event shaped like Claude Code's `Stop` payload

 @example
 ```ts
 stopEvent({ stop_hook_active: true });
 ```
 */
function stopEvent(
  overrides: {
    readonly stop_hook_active?: boolean;
    readonly last_assistant_message?: string;
  } = {},
): StopInput {
  return {
    session_id: 'test-session',
    transcript_path: '/dev/null',
    cwd: '/tmp',
    permission_mode: 'default',
    hook_event_name: 'Stop',
    stop_hook_active: false,
    last_assistant_message: 'Done.',
    ...overrides,
  };
}

/**
 Transcript line for the feedback record Claude Code writes when this hook blocks on a phrase.

 @param phrase - phrase named in the reminder

 @returns one JSONL line

 @example
 ```ts
 reminderLine('say so if');
 ```
 */
function reminderLine(phrase: string,): string {
  return JSON.stringify({
    type: 'user',
    message: { content: `Stop hook feedback:\n${parkedDecisionReminder(phrase,).join(' ',)}`, },
  },);
}

/**
 Transcript line for a message the human typed.
 */
const HUMAN_LINE = JSON.stringify({
  type: 'user',
  origin: { kind: 'human', },
  message: { content: 'next task', },
},);

await describe({
  name: 'parked decision detection',
  children: [
    describe({
      name: findParkedDecision.name,
      children: [
        it({
          name: 'matches each phrase that leaves a choice with the user',
          fn: async () => {
            const offers = [
              ['I did not post it. Say so if you want it posted.', 'say so if',],
              ['I still have no sample log. Say if you want it built.', 'say if you',],
              ['The drive is mounted. Tell me if you want me to unmount it.', 'tell me if you',],
              ["Say so if you'd rather keep it.", 'say so if',],
              ["If you'd rather have it gone, I will delete it.", "if you'd rather",],
              ['If you would rather attach the sheet, it is ready.', 'if you would rather',],
            ] as const;

            for (const [text, phrase,] of offers) {
              expect(findParkedDecision(text,),).toEqual({ phrase, },);
            }
          },
        },),
        it({
          name: 'matches a phrase written with a curly apostrophe',
          fn: async () => {
            expect(findParkedDecision('If you’d rather stop here, I will.',),).toEqual({ phrase: "if you'd rather", },);
          },
        },),
        it({
          name: 'leaves prose that offers nothing alone',
          fn: async () => {
            expect(findParkedDecision('Posted. The monitor will tell me if CPU crosses the limit.',),).toBe(NO_MATCH,);
            expect(findParkedDecision('Whether to ship is your call, and nothing else is pending.',),).toBe(NO_MATCH,);
          },
        },),
      ],
    },),
    describe({
      name: parkedDecisionLead.name,
      children: [
        it({
          name: 'is the prefix, the phrase in parentheses and a full stop',
          fn: async () => {
            expect(parkedDecisionLead('say so if',),).toBe(`${PARKED_DECISION_REMINDER_PREFIX} ("say so if").`,);
          },
        },),
      ],
    },),
    describe({
      name: parkedDecisionReminder.name,
      children: [
        it({
          name: 'opens with the lead the transcript scan looks for and names the question tool',
          fn: async () => {
            const lines = parkedDecisionReminder('say so if',);
            expect(lines[0],).toBe(parkedDecisionLead('say so if',),);
            expect(lines.join(' ',),).toContain('AskUserQuestion',);
          },
        },),
      ],
    },),
    describe({
      name: `${stopRemindersDecision.name} with a parked decision`,
      children: [
        it({
          name: 'blocks the first stop and routes the agent to the question tool',
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ last_assistant_message: 'Done. Say so if you want it posted.', },),
              forcedContinuationAllowed: false,
            },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain(`${PARKED_DECISION_REMINDER_PREFIX} ("say so if")`,);
            expect(output.reason,).toContain('AskUserQuestion',);
          },
        },),
        it({
          name: 'blocks inside a chain when the phrase was not raised in this turn',
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: 'Done. Say so if you want it posted.', },),
              forcedContinuationAllowed: false,
              transcriptLines: [HUMAN_LINE, reminderLine('tell me if you',),],
            },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain(`${PARKED_DECISION_REMINDER_PREFIX} ("say so if")`,);
          },
        },),
        it({
          name: 'lets the stop through inside a chain once the phrase was raised in this turn',
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: 'Done. Say so if you want it posted.', },),
              forcedContinuationAllowed: false,
              transcriptLines: [HUMAN_LINE, reminderLine('say so if',),],
            },);

            expect(output,).toEqual({},);
          },
        },),
        it({
          name: 'leaves a trailing question as the only reminder',
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ last_assistant_message: 'Done. Say so if you want it posted. Want me to commit?', },),
              forcedContinuationAllowed: false,
            },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('ends with a question to the user',);
            expect(output.reason,).not.toContain(PARKED_DECISION_REMINDER_PREFIX,);
          },
        },),
        it({
          name: 'passes a stop that offers nothing',
          fn: async () => {
            expect(stopRemindersDecision({ event: stopEvent(), forcedContinuationAllowed: false, },),).toEqual({},);
          },
        },),
      ],
    },),
  ],
},);
