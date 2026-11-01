import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  continuationDepth,
  CONTINUATION_MARKER,
  continuationDepthAt,
  DEFAULT_MAX_DEPTH,
  FEEDBACK_PREFIX,
  MAX_DEPTH_ENV,
  maxContinuationDepth,
  readTranscriptTail,
} from './continuation-depth.ts';

/**
 Transcript line standing for one forced-continuation feedback record.
 */
const BLOCK_LINE = `{"type":"user","message":{"content":"${FEEDBACK_PREFIX}:\\n${CONTINUATION_MARKER}."}}`;

/**
 Attachment record Claude Code writes alongside every block.
 
 Carries the reason but not the feedback prefix, so it must not be counted.
 */
const ATTACHMENT_LINE =
  `{"type":"attachment","attachment":{"type":"hook_blocking_error","blockingError":{"blockingError":"${CONTINUATION_MARKER}."}}}`;

/**
 Transcript line standing for a genuine human turn, which closes the counting window.
 */
const HUMAN_LINE = '{"type":"user","origin":{"kind":"human"},"message":{"content":"do the thing"}}';

/**
 Transcript line standing for an assistant turn, which neither counts nor closes.
 */
const ASSISTANT_LINE = '{"type":"assistant","message":{"content":[{"type":"text","text":"working"}]}}';

await describe({
  name: 'forced-continuation depth guard',
  children: [
    describe({
      name: maxContinuationDepth.name,
      children: [
        it({
          name: 'falls back to the default when unset',
          fn: async () => {
            expect(maxContinuationDepth('',),).toBe(DEFAULT_MAX_DEPTH,);
          },
        },),
        it({
          name: 'falls back when the value is not an integer',
          fn: async () => {
            for (const value of ['abc', '3.5', '', '  ',]) {
              expect(maxContinuationDepth(value,),).toBe(DEFAULT_MAX_DEPTH,);
            }
          },
        },),
        it({
          name: 'falls back on zero and negatives rather than disabling the guard',
          fn: async () => {
            expect(maxContinuationDepth('0',),).toBe(DEFAULT_MAX_DEPTH,);
            expect(maxContinuationDepth('-4',),).toBe(DEFAULT_MAX_DEPTH,);
          },
        },),
        it({
          name: 'accepts an explicit positive integer',
          fn: async () => {
            expect(maxContinuationDepth(' 40 ',),).toBe(40,);
          },
        },),
        it({
          name: 'names the override variable exactly as the README and troubleshooting doc do',
          fn: async () => {
            // Both documents spell this out for the user to type, so a rename that
            // missed them would leave a documented control that silently does nothing.
            expect(MAX_DEPTH_ENV,).toBe('MONOCHROMATIC_STOP_AUTO_CONTINUE_MAX',);
          },
        },),
        it({
          name: 'defaults high enough to clear the longest productive run observed',
          fn: async () => {
            // The busiest probe reached 31 dispatches only because its agent was told to
            // work on every continuation; real sessions here stayed well under 20.
            expect(DEFAULT_MAX_DEPTH,).toBeGreaterThan(20,);
          },
        },),
      ],
    },),

    describe({
      name: continuationDepth.name,
      children: [
        it({
          name: 'counts zero for an empty transcript',
          fn: async () => {
            expect(continuationDepth([],),).toBe(0,);
          },
        },),
        it({
          name: 'counts consecutive forced continuations',
          fn: async () => {
            expect(continuationDepth([HUMAN_LINE, BLOCK_LINE, ASSISTANT_LINE, BLOCK_LINE,],),).toBe(2,);
          },
        },),
        it({
          name: 'stops counting at the last human turn so earlier turns do not accumulate',
          fn: async () => {
            expect(
              continuationDepth([BLOCK_LINE, BLOCK_LINE, BLOCK_LINE, HUMAN_LINE, BLOCK_LINE,],),
            ).toBe(1,);
          },
        },),
        it({
          name: 'ignores blank lines from a trailing newline',
          fn: async () => {
            expect(continuationDepth([HUMAN_LINE, BLOCK_LINE, '',],),).toBe(1,);
          },
        },),
        it({
          name: 'counts nothing when the turn has only assistant activity',
          fn: async () => {
            expect(continuationDepth([HUMAN_LINE, ASSISTANT_LINE, ASSISTANT_LINE,],),).toBe(0,);
          },
        },),
        it({
          name: 'overcounts rather than undercounts when no human turn is in range',
          fn: async () => {
            // A truncated tail loses the human turn that would close the window, so every
            // block still in range counts. Reaching the limit sooner allows the stop sooner,
            // which is the safe direction for a mechanism that is otherwise unbounded.
            expect(continuationDepth([BLOCK_LINE, ASSISTANT_LINE, BLOCK_LINE, BLOCK_LINE,],),).toBe(3,);
          },
        },),
        it({
          name: 'does not end the scan on a tool result that merely prints the human-origin marker',
          fn: async () => {
            // Observed live: inspecting transcripts printed this exact string into tool
            // output, which ended a substring-based scan early and undercounted depth by
            // nearly half. Undercounting lets the chain run past its limit.
            const toolResultQuotingOrigin =
              String.raw`{"type":"user","toolUseResult":{"stdout":"origin.kind is \"kind\":\"human\" here"}}`;

            expect(
              continuationDepth([HUMAN_LINE, BLOCK_LINE, toolResultQuotingOrigin, BLOCK_LINE,],),
            ).toBe(2,);
          },
        },),
        it({
          name: 'does not count a record that merely quotes the block reason',
          fn: async () => {
            // Documentation and tool output both quote this reason verbatim, so a
            // substring test would count blocks that never happened.
            const quotingRecord =
              `{"type":"assistant","message":{"content":[{"type":"text","text":"${CONTINUATION_MARKER}"}]}}`;

            expect(continuationDepth([HUMAN_LINE, quotingRecord, quotingRecord,],),).toBe(0,);
          },
        },),
        it({
          name: 'does not treat a subagent human-origin record as closing the window',
          fn: async () => {
            const sidechainHuman =
              '{"type":"user","origin":{"kind":"human"},"isSidechain":true,"message":{"content":"sub"}}';

            expect(continuationDepth([HUMAN_LINE, BLOCK_LINE, sidechainHuman, BLOCK_LINE,],),).toBe(2,);
          },
        },),
        it({
          name: 'skips a truncated final line rather than throwing',
          fn: async () => {
            expect(continuationDepth([HUMAN_LINE, BLOCK_LINE, '{"type":"user","mess',],),).toBe(1,);
          },
        },),
        it({
          name: 'counts each block once despite its paired attachment carrying the same reason',
          fn: async () => {
            expect(
              continuationDepth([HUMAN_LINE, BLOCK_LINE, ATTACHMENT_LINE, BLOCK_LINE, ATTACHMENT_LINE,],),
            ).toBe(2,);
          },
        },),
      ],
    },),

    describe({
      name: continuationDepthAt.name,
      children: [
        it({
          name: 'reports zero rather than throwing when the transcript is missing',
          fn: async () => {
            expect(await continuationDepthAt('/nonexistent/transcript.jsonl',),).toBe(0,);
          },
        },),
        it({
          name: 'marks an unreadable transcript truncated so absence-based releases stay off',
          fn: async () => {
            expect((await readTranscriptTail('/nonexistent/transcript.jsonl',)).truncated,).toBe(true,);
          },
        },),
        it({
          name: 'marks a whole small transcript untruncated',
          fn: async () => {
            expect((await readTranscriptTail('/dev/null',)).truncated,).toBe(false,);
          },
        },),
        it({
          name: 'reports zero for an empty transcript',
          fn: async () => {
            expect(await continuationDepthAt('/dev/null',),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
