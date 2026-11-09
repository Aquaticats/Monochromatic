import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readFileSync,
  writeFileSync,
} from 'node:fs';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, resolve, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  buildTranscriptTail,
  continuationDepth,
  CONTINUATION_MARKER,
  continuationDepthAt,
  DEFAULT_MAX_DEPTH,
  FEEDBACK_PREFIX,
  MAX_DEPTH_ENV,
  maxContinuationDepth,
  readTranscriptTail,
  SCAN_FINISHED,
  scanTranscriptNewestFirst,
  TAIL_BYTES,
} from './continuation-depth.ts';

/**
 Repository root, resolved from this test file's directory so the document reads
 follow the checkout rather than the test runner's working directory.
 */
const REPO_ROOT = resolve(import.meta.dirname, '../../../../../..',);

/**
 Operator documentation that tells the user which environment variable to set.
 */
const README_PATH = resolve(
  REPO_ROOT,
  'package/claude-code-plugin/stop-reminder/README.md',
);

/**
 Troubleshooting document that repeats the same variable name as read-only evidence.
 */
const TROUBLESHOOTING_PATH = resolve(
  REPO_ROOT,
  'doc/troubleshooting/claude-code-opus-5-premature-turn-end.md',
);

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

/**
 Scratch directory holding this suite's transcript fixtures.
 */
const SCRATCH = await mkdtemp(join(scratchRoot(), 'continuation-depth-tail-',),);

/**
 One transcript record whose text can be found again after a tail read.
 */
const TAIL_RECORD = '{"type":"user","origin":{"kind":"human"}}';

/**
 Writes a transcript fixture under {@link SCRATCH}.
 
 @param name - fixture file name
 
 @param content - bytes to write
 
 @returns absolute path of the written fixture
 */
async function writeTranscriptFixture(
  {
    name,
    content,
  }: {
    readonly name: string;
    readonly content: string;
  },
): Promise<string> {
  /**
   Absolute path of this fixture inside the scratch directory.
   */
  const path = join(
    SCRATCH,
    name,
  );
  writeFileSync(
    path,
    content,
  );
  return path;
}

/**
 Whether any line carries a NUL character. A tail read that decodes unfilled
 buffer space produces these; transcript text never contains one.
 
 @param lines - lines the tail reader returned
 
 @returns whether a NUL byte reached the lines
 */
function anyLineCarriesNul(lines: readonly string[],): boolean {
  return lines.some(function hasNul(line,): boolean {
    return line.includes('\u0000',);
  },);
}

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
            // Both documents present the name as a backticked token for the user to type,
            // so requiring the whole token also fails when a document renames the variable
            // to a longer identifier that merely starts with the constant.
            expect(readFileSync(README_PATH, 'utf8',),).toContain(`\`${MAX_DEPTH_ENV}\``,);
            expect(readFileSync(TROUBLESHOOTING_PATH, 'utf8',),).toContain(`\`${MAX_DEPTH_ENV}\``,);
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


    describe({
      name: buildTranscriptTail.name,
      children: [
        it({
          name: 'splits whole-file bytes into lines and reports no truncation',
          fn: async () => {
            /**
             Bytes the stream would deliver for a whole small transcript.
             */
            const delivered = Buffer.from(`${TAIL_RECORD}\n`,);
            /**
             Tail built from those bytes with nothing skipped.
             */
            const tail = buildTranscriptTail({
              tailBytes: delivered,
              expectedBytes: delivered.length,
              skippedEarlierBytes: false,
            },);
            expect(tail.lines,)
              .toEqual([TAIL_RECORD, '',],);
            expect(tail.truncated,)
              .toBe(false,);
          },
        },),
        it({
          name: 'decodes only the bytes delivered when the file shrank after the size lookup',
          fn: async () => {
            /**
             Bytes the shrunk file still delivered, fewer than the size promised.
             */
            const delivered = Buffer.from('{"type":',);
            /**
             Tail built from a delivery short of the promised size.
             */
            const tail = buildTranscriptTail({
              tailBytes: delivered,
              expectedBytes: delivered.length + 1,
              skippedEarlierBytes: false,
            },);
            expect(tail.truncated,)
              .toBe(true,);
            expect(tail.lines,)
              .toEqual(['{"type":',],);
            expect(anyLineCarriesNul(tail.lines,),)
              .toBe(false,);
          },
        },),
        it({
          name: 'reports truncation when the read began past the start of the file',
          fn: async () => {
            /**
             Bytes the stream would deliver from a tail window.
             */
            const delivered = Buffer.from(`${TAIL_RECORD}\n`,);
            /**
             Tail built from a window that skipped earlier bytes.
             */
            const tail = buildTranscriptTail({
              tailBytes: delivered,
              expectedBytes: delivered.length,
              skippedEarlierBytes: true,
            },);
            expect(tail.truncated,)
              .toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'readTranscriptTail over a transcript fixture',
      children: [
        it({
          name: 'reads a small transcript whole without NUL padding',
          fn: async () => {
            /**
             Transcript content written for this case.
             */
            const content = `${TAIL_RECORD}\n${TAIL_RECORD}\n`;
            /**
             Fixture path holding that content.
             */
            const path = await writeTranscriptFixture({
              name: 'small.jsonl',
              content,
            },);
            /**
             Tail read back from the fixture.
             */
            const tail = await readTranscriptTail(path,);
            expect(tail.lines,)
              .toEqual(content.split('\n',),);
            expect(tail.truncated,)
              .toBe(false,);
            expect(anyLineCarriesNul(tail.lines,),)
              .toBe(false,);
          },
        },),
        it({
          name: 'keeps the final records of a transcript larger than the tail window',
          fn: async () => {
            /**
             Filler one byte past the tail window, so the window must skip at
             least one byte of it.
             */
            const filler = 'A'.repeat(
              TAIL_BYTES + 1,
            );
            /**
             Transcript whose tail window starts inside the filler run.
             */
            const content = `${filler}\n${TAIL_RECORD}\n`;
            /**
             Fixture path holding that content.
             */
            const path = await writeTranscriptFixture({
              name: 'oversized.jsonl',
              content,
            },);
            /**
             Tail read back from the fixture.
             */
            const tail = await readTranscriptTail(path,);
            expect(tail.truncated,)
              .toBe(true,);
            expect(tail.lines,)
              .toContain(TAIL_RECORD,);
            expect(anyLineCarriesNul(tail.lines,),)
              .toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: scanTranscriptNewestFirst.name,
      children: [
        it({
          name: 'returns the verdict of the newest record that produces one',
          fn: async () => {
            expect(
              scanTranscriptNewestFirst<string>({
                lines: [HUMAN_LINE, ASSISTANT_LINE,],
                classifyRecord: function classify(record,) {
                  return record.type === 'assistant'
                    ? 'newest assistant'
                    : SCAN_FINISHED;
                },
              },),
            ).toBe('newest assistant',);
          },
        },),
        it({
          name: 'keeps scanning past records that produce no verdict',
          fn: async () => {
            expect(
              scanTranscriptNewestFirst<string>({
                lines: [HUMAN_LINE, ASSISTANT_LINE,],
                classifyRecord: function classify(record,) {
                  return record.type === 'user' ? 'older user' : SCAN_FINISHED;
                },
              },),
            ).toBe('older user',);
          },
        },),
        it({
          name: 'skips blank lines without classifying them',
          fn: async () => {
            expect(
              scanTranscriptNewestFirst<string>({
                lines: [HUMAN_LINE, '',],
                classifyRecord: function classify(record,) {
                  return record.type === 'user' ? 'human turn' : SCAN_FINISHED;
                },
              },),
            ).toBe('human turn',);
          },
        },),
        it({
          name: 'skips an unparsable line instead of throwing',
          fn: async () => {
            expect(
              scanTranscriptNewestFirst<string>({
                lines: [HUMAN_LINE, '{"type":"user","mess',],
                classifyRecord: function classify(record,) {
                  return record.type === 'user' ? 'human turn' : SCAN_FINISHED;
                },
              },),
            ).toBe('human turn',);
          },
        },),
        it({
          name: 'never classifies a line the pre-filter rejects',
          fn: async () => {
            /**
             Record types the classifier saw, so a rejected line shows up as an absence.
             */
            const classified: string[] = [];
            /**
             Verdict of a scan whose pre-filter keeps every fixture line out.
             */
            const verdict = scanTranscriptNewestFirst<string>({
              lines: [HUMAN_LINE, ASSISTANT_LINE,],
              acceptLine: function mentionsToolUse(line,) {
                return line.includes('tool_use',);
              },
              classifyRecord: function classify(record,) {
                classified.push(String(record.type,),);
                return 'classified';
              },
            },);
            expect(verdict,)
              .toBe(SCAN_FINISHED,);
            expect(classified,)
              .toEqual([],);
          },
        },),
        it({
          name: 'returns SCAN_FINISHED when no record produces a verdict',
          fn: async () => {
            expect(
              scanTranscriptNewestFirst<string>({
                lines: [HUMAN_LINE, ASSISTANT_LINE,],
                classifyRecord: function classify() {
                  return SCAN_FINISHED;
                },
              },),
            ).toBe(SCAN_FINISHED,);
          },
        },),
        it({
          name: 'returns a falsy verdict rather than treating it as no verdict',
          fn: async () => {
            expect(
              scanTranscriptNewestFirst<false>({
                lines: [HUMAN_LINE,],
                classifyRecord: function classify() {
                  return false;
                },
              },),
            ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

await rm(
  SCRATCH,
  {
    recursive: true,
    force: true,
  },
);
