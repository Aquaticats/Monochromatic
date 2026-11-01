import type { StopInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  AUTO_CONTINUE_ENV,
  autoContinueActive,
  autoContinueEnabled,
  autoContinueReason,
  ENABLING_VALUES,
  UNSET_SETTING,
} from './auto-continue.ts';
import {
  createStopRemindersHandler,
  stopRemindersDecision,
} from './index.ts';
import {
  dismissalLead,
  questionLead,
  uncertaintyLead,
} from './reminder-lead.ts';

/**
 Fields a test varies on the built stop event; anything omitted keeps its default.
 */
type StopEventOverrides = {
  readonly stop_hook_active?: boolean;
  readonly last_assistant_message?: string;
};

/**
 Stop event with every field the handler reads, so each test varies one thing.
 
 @param overrides - fields replacing defaults on built event
 
 @returns event shaped like Claude Code's `Stop` payload
 
 @example
 ```ts
 stopEvent({ stop_hook_active: true });
 ```
 */
function stopEvent(overrides: StopEventOverrides = {},): StopInput {
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
 Transcript line for a message the human typed, which closes the window the once-per-trigger scan reads.
 */
const HUMAN_LINE = JSON.stringify({ type: 'user', origin: { kind: 'human', }, message: { content: 'go', }, },);

/**
 Transcript line for the feedback record Claude Code writes when this hook blocks.

 @param lead - opening sentence of the reminder the record carries

 @returns one JSONL line

 @example
 ```ts
 feedbackLine(uncertaintyLead('probably'));
 ```
 */
function feedbackLine(lead: string,): string {
  return JSON.stringify({ type: 'user', message: { content: `Stop hook feedback:\n${lead} Continue.`, }, },);
}

await describe({
  name: 'stop-reminder forced continuation',
  children: [
    describe({
      name: autoContinueEnabled.name,
      children: [
        it({
          name: 'defaults to disabled when the opt-in switch is unset',
          fn: async () => {
            expect(autoContinueEnabled(UNSET_SETTING,),).toBe(false,);
          },
        },),
        it({
          name: 'stays disabled for an unrecognized value',
          fn: async () => {
            expect(autoContinueEnabled('off',),).toBe(false,);
          },
        },),
        it({
          name: 'enables on every documented enabling value',
          fn: async () => {
            for (const value of ENABLING_VALUES) {
              expect(autoContinueEnabled(value,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'normalizes surrounding space and letter case',
          fn: async () => {
            expect(autoContinueEnabled('  ON  ',),).toBe(true,);
          },
        },),
        it({
          name: 'names the opt-in switch with the repository environment prefix',
          fn: async () => {
            expect(AUTO_CONTINUE_ENV,).toBe('MONOCHROMATIC_STOP_AUTO_CONTINUE',);
          },
        },),
        it({
          name: 'reads the opt-in switch from this process without re-deciding it',
          fn: async () => {
            expect(autoContinueActive(),).toBe(
              autoContinueEnabled(process.env[AUTO_CONTINUE_ENV] ?? UNSET_SETTING,),
            );
          },
        },),
      ],
    },),

    describe({
      name: autoContinueReason.name,
      children: [
        it({
          name: 'instructs the agent to keep status prose rather than suppress it',
          fn: async () => {
            expect(autoContinueReason()
              .join(' ',),).toContain('do not delete, shorten, or rephrase it',);
          },
        },),
        it({
          name: 'offers a blocker path so a genuine stop can be explained',
          fn: async () => {
            expect(autoContinueReason()
              .join(' ',),).toContain('name the concrete blocker',);
          },
        },),
        it({
          name: 'routes a decision-blocked agent to AskUserQuestion rather than to a stop',
          fn: async () => {
            // That tool waits for the user, so it is the one exit that gets the agent
            // what stopping was reaching for, and the hook has nothing to refuse.
            expect(autoContinueReason()
              .join(' ',),).toContain('AskUserQuestion',);
          },
        },),
      ],
    },),

    describe({
      name: stopRemindersDecision.name,
      children: [
        it({
          name: "blocks a clean response that would otherwise have been allowed",
          fn: async () => {
            expect(stopRemindersDecision({ event: stopEvent(), forcedContinuationAllowed: true, },).decision,)
              .toBe("block",);
          },
        },),
        it({
          name: "reports a hedge inside a chain the first time and still appends the continuation",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: "This probably works.", },),
              forcedContinuationAllowed: true,
            },);

            expect(output.decision,).toBe("block",);
            expect(output.reason,).toContain("uncertain language",);
            expect(output.reason,).toContain("Resume the next item now",);
          },
        },),
        it({
          name: "leaves a hedge out of a chain once it was raised in this turn, keeping only the continuation",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: "This probably works.", },),
              forcedContinuationAllowed: true,
              transcriptLines: [HUMAN_LINE, feedbackLine(uncertaintyLead("probably",),),],
            },);

            expect(output.decision,).toBe("block",);
            expect(output.reason,).not.toContain("uncertain language",);
            expect(output.reason,).toContain("Resume the next item now",);
          },
        },),
        it({
          name: "reports a different hedge inside a chain even after another was raised",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: "This is probably fine, maybe.", },),
              forcedContinuationAllowed: false,
              transcriptLines: [HUMAN_LINE, feedbackLine(uncertaintyLead("maybe",),),],
            },);

            expect(output.reason,).toContain("uncertain language",);
            expect(output.reason,).toContain("probably",);
          },
        },),
        it({
          name: "reports hedging on the first stop of a chain",
          fn: async () => {
            expect(stopRemindersDecision({
              event: stopEvent({ last_assistant_message: "This probably works.", },),
              forcedContinuationAllowed: true,
            },).reason,).toContain("uncertain language",);
          },
        },),
        it({
          name: "allows a clean stop once forced continuation is disallowed",
          fn: async () => {
            expect(stopRemindersDecision({ event: stopEvent(), forcedContinuationAllowed: false, },),)
              .toEqual({},);
          },
        },),
        it({
          name: "still reports hedging while forced continuation is disallowed",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ last_assistant_message: "This probably works.", },),
              forcedContinuationAllowed: false,
            },);

            expect(output.decision,).toBe("block",);
            expect(output.reason,).toContain("uncertain language",);
          },
        },),
        it({
          name: "reports an uncited dismissal on the first stop of a chain",
          fn: async () => {
            expect(stopRemindersDecision({
              event: stopEvent({ last_assistant_message: "This project doesn't use that.", },),
              forcedContinuationAllowed: true,
            },).reason,).toContain("categorical dismissal",);
          },
        },),
        it({
          name: "reports an uncited dismissal inside a chain the first time",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: "This project doesn't use that.", },),
              forcedContinuationAllowed: true,
            },);

            expect(output.decision,).toBe("block",);
            expect(output.reason,).toContain("categorical dismissal",);
          },
        },),
        it({
          name: "leaves the dismissal out of a chain once it was raised in this turn, leaving only continuation",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: "This project doesn't use that.", },),
              forcedContinuationAllowed: true,
              transcriptLines: [HUMAN_LINE, feedbackLine(dismissalLead("project doesn't use",),),],
            },);

            expect(output.decision,).toBe("block",);
            expect(output.reason,).not.toContain("categorical dismissal",);
            expect(output.reason,).toContain("Resume the next item now",);
          },
        },),
        it({
          name: "asks through the question tool inside a chain the first time and not twice",
          fn: async () => {
            /**
             Question the final message ends with.
             */
            const sentence = "Which package should I migrate first?";
            const first = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: sentence, },),
              forcedContinuationAllowed: true,
            },);
            const repeated = stopRemindersDecision({
              event: stopEvent({ stop_hook_active: true, last_assistant_message: sentence, },),
              forcedContinuationAllowed: true,
              transcriptLines: [HUMAN_LINE, feedbackLine(questionLead(sentence,),),],
            },);

            expect(first.reason,).toContain("ends with a question to the user",);
            expect(first.reason,).not.toContain("Resume the next item now",);
            expect(repeated.reason,).not.toContain("ends with a question to the user",);
            expect(repeated.reason,).toContain("Resume the next item now",);
          },
        },),
        it({
          name: "yields precedence to a trailing question instead of contradicting it",
          fn: async () => {
            const output = stopRemindersDecision({
              event: stopEvent({ last_assistant_message: "Which package should I migrate first?", },),
              forcedContinuationAllowed: true,
            },);

            expect(output.reason,).toContain("AskUserQuestion",);
            expect(output.reason,).not.toContain("Resume the next item now",);
          },
        },),
        it({
          name: "wires the handler through to the opt-in switch, with the Luna check off",
          fn: async () => {
            // The environment is injected, so the test neither races the others by mutating the
            // process nor reaches the gateway; the built-bundle suite drives the real process.
            const off = createStopRemindersHandler({ env: { MONOCHROMATIC_STOP_LUNA: "off", }, },);
            const on = createStopRemindersHandler({
              env: { MONOCHROMATIC_STOP_AUTO_CONTINUE: "on", MONOCHROMATIC_STOP_LUNA: "off", },
            },);

            expect((await off(stopEvent(),)).decision,).toBeUndefined();
            expect((await on(stopEvent(),)).decision,).toBe("block",);
          },
        },),
      ],
    },),
  ],
},);
