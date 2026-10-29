import type {
  UserPromptSubmitInput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  correctionReminderHandler,
  correctionReminderParser,
  correctionReminderWriter,
  detectCorrection,
} from './correction-reminder.ts';

function makeEvent(prompt: string,): UserPromptSubmitInput {
  return {
    hook_event_name: 'UserPromptSubmit',
    session_id: 'session-test',
    transcript_path: '/tmp/transcript.jsonl',
    cwd: '/tmp/cwd',
    permission_mode: 'default',
    prompt,
  };
}

await describe({
  name: 'correction-reminder handler',
  children: [
    describe({
      name: detectCorrection.name,
      children: [
        it({
          name: 'detects "demonstrably false"',
          fn: async () => {
            expect(detectCorrection("That's demonstrably false.",),).toBe(true,);
          },
        },),
        it({
          name: 'detects "you missed"',
          fn: async () => {
            expect(detectCorrection('You missed the JSX runtime.',),).toBe(true,);
          },
        },),
        it({
          name: 'detects "didn\'t you"',
          fn: async () => {
            expect(detectCorrection("Didn't you check the config first?",),).toBe(
              true,
            );
          },
        },),
        it({
          name: 'detects "you\'re wrong"',
          fn: async () => {
            expect(detectCorrection("You're wrong about that claim.",),).toBe(
              true,
            );
          },
        },),
        it({
          name: 'detects "shouldn\'t have"',
          fn: async () => {
            expect(
              detectCorrection("You shouldn't have included that.",),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'detects "why would you"',
          fn: async () => {
            expect(
              detectCorrection('Why would you skip the AGENTS.md check?',),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'detects "please be more careful"',
          fn: async () => {
            expect(detectCorrection('Please be more careful next time.',),).toBe(
              true,
            );
          },
        },),
        it({
          name: 'returns false for a neutral prompt',
          fn: async () => {
            expect(detectCorrection("What's the build status?",),).toBe(false,);
          },
        },),
        it({
          name: 'returns false for a clarifying question',
          fn: async () => {
            expect(
              detectCorrection('Can you tell me more about the next step?',),
            )
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: correctionReminderHandler.name,
      children: [
        it({
          name: 'injects reminder when a correction phrase fires',
          fn: async () => {
            const out = correctionReminderHandler(
              makeEvent("That's demonstrably false.",),
            );
            expect(out.hookSpecificOutput?.additionalContext,).toContain(
              'correction-detected',
            );
          },
        },),
        it({
          name: 'reminder rejects same-session self-review',
          fn: async () => {
            const out = correctionReminderHandler(
              makeEvent('You missed the JSX runtime.',),
            );
            expect(out.hookSpecificOutput?.additionalContext,).toContain(
              'self-review is not',
            );
          },
        },),
        it({
          name: 'reminder cites AGENTS.md rule CKB',
          fn: async () => {
            const out = correctionReminderHandler(
              makeEvent('Why would you include that rule?',),
            );
            expect(out.hookSpecificOutput?.additionalContext,).toContain(
              'rule CKB',
            );
          },
        },),
        it({
          name: 'emits empty additionalContext for non-correction prompts',
          fn: async () => {
            const out = correctionReminderHandler(
              makeEvent("What's the build status?",),
            );
            expect(out.hookSpecificOutput?.additionalContext,).toBe('',);
          },
        },),
        it({
          name: 'always tags hookEventName as UserPromptSubmit',
          fn: async () => {
            const out = correctionReminderHandler(makeEvent('anything',),);
            expect(out.hookSpecificOutput?.hookEventName,).toBe(
              'UserPromptSubmit',
            );
          },
        },),
      ],
    },),
    describe({
      name: correctionReminderParser.name,
      children: [
        it({
          name: 'round-trips a serialized UserPromptSubmit event',
          fn: async () => {
            const event = makeEvent('test',);
            const parsed = correctionReminderParser(JSON.stringify(event,),);
            expect(parsed.prompt,).toBe('test',);
            expect(parsed.hook_event_name,).toBe('UserPromptSubmit',);
          },
        },),
      ],
    },),
    describe({
      name: correctionReminderWriter.name,
      children: [
        it({
          name: 'serializes the output as JSON with no trailing newline',
          fn: async () => {
            const written = correctionReminderWriter({
              hookSpecificOutput: {
                hookEventName: 'UserPromptSubmit',
                additionalContext: '',
              },
            },);
            expect(written.endsWith('\n',),).toBe(false,);
            expect(written.startsWith('{',),).toBe(true,);
            expect(written.endsWith('}',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
