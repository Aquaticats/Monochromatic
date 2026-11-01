import type {
  UserPromptSubmitInput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  formatTimeContext,
  promptTimeHandler,
  promptTimeParser,
  promptTimeWriter,
} from './prompt-time.ts';

/**
 Builds a minimal `UserPromptSubmitInput` so the handler tests do not have to
 carry the full Claude Code envelope when only the event shape matters.
 
 @param prompt - text to place in the event's `prompt` field
 
 @returns a populated `UserPromptSubmitInput`
 */
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
  name: 'prompt-time handler',
  children: [
    describe({
      name: formatTimeContext.name,
      children: [
        it({
          name: 'zero-pads single-digit hours and minutes',
          fn: async () => {
            expect(
              formatTimeContext(new Date('2026-05-01T07:05:00',),),
            )
              .toBe('<time>07:05</time>',);
          },
        },),
        it({
          name: 'renders midnight as 00:00',
          fn: async () => {
            expect(
              formatTimeContext(new Date('2026-05-01T00:00:00',),),
            )
              .toBe('<time>00:00</time>',);
          },
        },),
        it({
          name: 'renders the example case 20:48 verbatim',
          fn: async () => {
            expect(
              formatTimeContext(new Date('2026-05-01T20:48:00',),),
            )
              .toBe('<time>20:48</time>',);
          },
        },),
        it({
          name: 'renders the last minute of the day as 23:59',
          fn: async () => {
            expect(
              formatTimeContext(new Date('2026-05-01T23:59:00',),),
            )
              .toBe('<time>23:59</time>',);
          },
        },),
      ],
    },),
    describe({
      name: promptTimeHandler.name,
      children: [
        it({
          name: 'returns a UserPromptSubmit additionalContext payload',
          fn: async () => {
            const out = promptTimeHandler(makeEvent('anything',),);
            expect(out.hookSpecificOutput?.hookEventName,).toBe('UserPromptSubmit',);
            expect(typeof out.hookSpecificOutput?.additionalContext,).toBe('string',);
          },
        },),
        it({
          name: 'emits a tag matching the <time>HH:MM</time> shape',
          fn: async () => {
            const out = promptTimeHandler(makeEvent('hello',),);
            /** Rendered `additionalContext` whose shape we assert below; falls back to empty when absent so each check produces a clear failure. */
            const actual = out.hookSpecificOutput?.additionalContext ?? '';
            /** Reference template fixes the expected length without hard-coding a magic number. */
            const TEMPLATE = '<time>HH:MM</time>';
            expect(actual.length,).toBe(TEMPLATE.length,);
            expect(actual.startsWith('<time>',),).toBe(true,);
            expect(actual.endsWith('</time>',),).toBe(true,);
            /** Five-character middle slice that should match `HH:MM`. */
            const middle = actual.slice(
              '<time>'.length,
              actual.length - '</time>'.length,
            );
            expect(middle.charAt(2,),).toBe(':',);
            /**
             Predicate that the four numeric positions of the middle slice
             are ASCII digits.
             
             @param c - single character from the middle slice
             
             @returns whether `c` is `[0-9]`
             
             @example
             ```ts
             isAsciiDigit('4'); // true
             ```
             */
            function isAsciiDigit(c: string,): boolean {
              return (c >= '0') && (c <= '9');
            }
            expect(
              isAsciiDigit(middle.charAt(0,),),
            ).toBe(true,);
            expect(
              isAsciiDigit(middle.charAt(1,),),
            ).toBe(true,);
            expect(
              isAsciiDigit(middle.charAt(3,),),
            ).toBe(true,);
            expect(
              isAsciiDigit(middle.charAt(4,),),
            ).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: promptTimeParser.name,
      children: [
        it({
          name: 'round-trips a serialized UserPromptSubmit event',
          fn: async () => {
            const event = makeEvent('round-trip me',);
            const parsed = promptTimeParser(JSON.stringify(event,),);
            expect(parsed.prompt,).toBe('round-trip me',);
            expect(parsed.hook_event_name,).toBe('UserPromptSubmit',);
          },
        },),
      ],
    },),
    describe({
      name: promptTimeWriter.name,
      children: [
        it({
          name: 'serializes the output as JSON with no trailing newline',
          fn: async () => {
            const written = promptTimeWriter({
              hookSpecificOutput: {
                hookEventName: 'UserPromptSubmit',
                additionalContext: '<time>20:48</time>',
              },
            },);
            expect(written,).toBe(
              '{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":"<time>20:48</time>"}}',
            );
          },
        },),
      ],
    },),
  ],
},);
