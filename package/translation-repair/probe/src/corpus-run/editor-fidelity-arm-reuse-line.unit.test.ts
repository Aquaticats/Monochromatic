/**
 Tests for how the read step finds the line the run client writes when it answers a prompt from an earlier reply
 (`PROMPT-REUSE`): through any number of tags the logger puts before the words of a line, since a logger tagged at its
 function boundary writes `[translation-repair] [claimedReply] PROMPT-REUSE ...`, and in the shape the stage's own writer
 gives it today, taken from the writer and not copied by hand. A row that follows such a line made no call.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ChatTextRequest, } from '@monochromatic-dev/translation-repair-provider';
import { promptUniqueClient, } from '@monochromatic-dev/translation-repair-stage';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { infoLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';

import {
  type ArmLog,
  armFiguresOf,
} from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  type RowSpec,
  tallyFromRows,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import { HIGH_CARD, } from '../editor-fidelity.test-fixture.ts';

/**
 The stamp and level the log file writes before the logger's own tags.
 */
const FILE_PREFIX = '[info] [2026-10-23T12:00:00.000Z] ';

/**
 Two rows of one arm, the second following a reuse line that the case gives.

 @param reuseLine - line written in front of second row

 @returns Arm's log, fixture's hand-written reuse line replaced by the given one

 @example
 ```ts
 const log = armWithReuseLine({ reuseLine: '[info] [2026-10-23T12:00:00.000Z] [translation-repair] PROMPT-REUSE source=memory model=m digest=d', },);
 ```
 */
function armWithReuseLine({ reuseLine, }: { readonly reuseLine: string; },): ArmLog {
  /**
   The two rows, the second marked so the fixture writes a reuse line before it.
   */
  const rows: readonly RowSpec[] = [
    { trial: 'cat-0-s0/insertion', entry: 'cat-0', fixed: true, stops: 'ok', },
    { trial: 'cat-0-s0/alteration', entry: 'cat-0', fixed: true, stops: 'ok', reused: true, },
  ];
  /**
   The log the fixture writes.
   */
  const written = armLogOf({
    arm: 'high',
    tally: tallyFromRows({ cardId: HIGH_CARD, rows, },),
    stops: [],
    rows,
    capped: false,
    seat: `SEAT ${HIGH_CARD} asked=2 usable=2 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
    benchSpentUsd: 0,
    guardUsd: 1,
  },);
  /**
   The log's lines with the fixture's reuse line, which it writes behind its own prefix, swapped for the given one.
   */
  const lines = written.lines.map(function swapped(line,): string {
    return line.includes('PROMPT-REUSE ',) ? reuseLine : line;
  },);
  // The swap must have happened, or a case would read the fixture's own line and pass for the wrong reason.
  expect(lines.includes(reuseLine,),).toBe(true,);
  return { ...written, lines, };
}

await describe({
  name: 'the reuse line the read step finds',
  // SEQUENTIAL: the second case diverts console.info to take the line from the stage's writer, so cases must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'FINDS THE REUSE LINE THROUGH ANY NUMBER OF TAGS: none, one function tag and two, each in front of the words '
        + 'PROMPT-REUSE, and counts the row that follows it as a row that made no call',
      fn: async () => {
        for (const tags of [
          '',
          '[claimedReply] ',
          '[claimedReply] [storedPayload] ',
        ]) {
          const figures = armFiguresOf({
            log: armWithReuseLine({
              reuseLine: `${FILE_PREFIX}[translation-repair] ${tags}PROMPT-REUSE source=memory model=${HIGH_CARD} digest=d`,
            },),
          },);
          expect(figures.reusedRows,).toBe(1,);
          expect(figures.rows.length,).toBe(1,);
        }
      },
    },),

    it({
      name: 'FINDS NO REUSE LINE IN A BODY THAT OPENS WITH A BRACKET AND NEVER CLOSES IT, since a line whose words begin '
        + 'inside a tag that is not closed has no words to read, and counts the row after it as a row that made a call',
      fn: async () => {
        /**
         The log whose second row follows a line with an unclosed tag in front of the marker.
         */
        const log = armWithReuseLine({
          reuseLine: `${FILE_PREFIX}[translation-repair] [claimedReply PROMPT-REUSE source=memory model=${HIGH_CARD} digest=d`,
        },);
        const figures = armFiguresOf({ log, },);
        expect(figures.reusedRows,).toBe(0,);
        expect(figures.rows.length,).toBe(2,);
      },
    },),

    it({
      name: 'FINDS THE REUSE LINE THE STAGE\'S OWN WRITER GIVES, taken from a prompt the run client answered from its memory '
        + 'and not copied by hand, so a change of the writer\'s tags is seen here',
      fn: async () => {
        /**
         The request asked twice.
         */
        const request: ChatTextRequest = {
          modelId: HIGH_CARD,
          messages: [{ role: 'user', content: 'Judge this one cat sentence.', },],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
        };
        /**
         The reuse layer over a client that answers once.
         */
        const client = promptUniqueClient({
          inner: {
            chatText: async () => ({ text: 'payload', usage: { prompt_tokens: 10, completion_tokens: 20, }, }),
            chatJson: async () => {
              throw new Error('chatJson unused by the text fixture',);
            },
            quotas: async () => {
              throw new Error('quotas unused by the text fixture',);
            },
          },
        },);
        const { logged, } = await infoLinesDuring({
          run: async () => {
            await client.chatText(request,);
            await client.chatText(request,);
          },
        },);
        /**
         The lines the writer gave for the second ask.
         */
        const reuseLines = logged.filter(function isReuse(line,): boolean {
          return line.includes('PROMPT-REUSE ',);
        },);
        expect(reuseLines.length,).toBe(1,);
        const figures = armFiguresOf({
          log: armWithReuseLine({ reuseLine: `${FILE_PREFIX}${reuseLines[0] ?? ''}`, },),
        },);
        expect(figures.reusedRows,).toBe(1,);
        expect(figures.rows.length,).toBe(1,);
      },
    },),
  ],
},);
