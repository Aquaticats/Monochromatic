/**
 Tests for reading back what a slice cost.

 THE CASE THAT DECIDES THIS PAIR is the round trip. The writer and the reader
 are the only two things that know this line's shape, and they live in separate
 files, so a reformatted message would leave a pass logging costs nobody can
 read while every other test still passed.

 The refusals matter for a second reason: a log is written WHILE a pass runs,
 so its last line can be half-written. A reader that skipped malformed lines in
 silence would report a smaller corpus without ever saying it had.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  armSliceCost,
  readSliceCosts,
  SLICE_START_MARKER,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';

/**
 Run that has not been stopped, which is every ordinary slice's condition.
 */
const LIVE_RUN = new AbortController();

/**
 Log holding one cost line per lane, among lines about other things.
 */
const TWO_LANE_LOG = [
  '[info] [2026-08-17T01:17:12.580Z] [Mittens] [repairPreparedDocument] critic stage: 6/6 heard',
  '[info] [2026-08-17T01:17:13.000Z] [Mittens] [repairPreparedDocument] SLICE-COST lane=repair '
    + 'chunk=0 sourceChars=812 ms=45210 exit=computed',
  '[info] [2026-08-17T01:17:14.000Z] [Mittens] [translateDocument] SLICE-COST lane=translate '
    + 'chunk=1 sourceChars=1204 ms=88130 exit=resumed',
  '[info] [2026-08-17T01:17:15.000Z] [Mittens] [drainBody] stream finished, 671606 chars',
].join('\n',);

await describe({
  name: '',
  children: [
    describe({
      name: readSliceCosts.name,
      children: [
        it({
          name:
            'ACCEPTS repair and translate cost lines and ignores every line that carries no marker, so a pass '
              + 'log can be read without being filtered first',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({ log: TWO_LANE_LOG, },);
            expect(dropped,).toHaveLength(0,);
            expect(rows,).toHaveLength(2,);
            expect(rows[0],).toEqual({
              lane: 'repair',
              sliceIndex: 0,
              sourceChars: 812,
              elapsedMs: 45_210,
              exit: 'computed',
            },);
            expect(rows[1]
              ?.lane,).toBe('translate',);
            expect(rows[1]
              ?.elapsedMs,).toBe(88_130,);
            expect(rows[1]
              ?.exit,).toBe('resumed',);
          },
        },),
        it({
          name:
            'REFUSES a half-written line and names every field it lacks, because a log is written while a '
              + 'pass runs and its last line can be cut mid-word',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=repair chunk=4 sourceCh',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped,).toHaveLength(1,);
            expect(dropped[0],).toBe('sourceChars missing, ms missing, exit missing',);
          },
        },),
        it({
          name: 'REFUSES a lane it does not know, rather than counting it under one it does',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=refine chunk=4 sourceChars=10 ms=20 exit=computed',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped[0],).toBe('lane refine',);
          },
        },),
        it({
          name:
            'REFUSES an exit it does not know, so a path added to a lane without being added here cannot '
              + 'be silently counted as ordinary work',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=repair chunk=4 sourceChars=10 ms=20 exit=abandoned',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped[0],).toBe('exit abandoned',);
          },
        },),
        it({
          name:
            'REFUSES a line carrying no exit at all, which is the shape written before exits were '
              + 'recorded and prices a cache hit the same as a full roster',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=repair chunk=4 sourceChars=10 ms=20',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped[0],).toBe('exit missing',);
          },
        },),
        it({
          name:
            'REFUSES a measurement that is not a whole number, which is what a truncated or garbled '
              + 'duration looks like',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=repair chunk=4 sourceChars=10 ms=45e exit=computed',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped[0],).toBe('ms 45e',);
          },
        },),
        it({
          name:
            'REFUSES an empty, hexadecimal, exponent or signed measurement, and names each, since the writer '
              + 'writes plain decimal digits and reading any other spelling as a number invents one: an empty '
              + 'field read as 0 and `0x1F` as 31 (ledger B71)',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=repair chunk= sourceChars=0x1F ms=1e3 exit=computed\n'
                + 'SLICE-COST lane=translate chunk=+2 sourceChars=10 ms=-3 exit=computed',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped,).toEqual([
              'chunk empty, sourceChars 0x1F, ms 1e3',
              'chunk +2, ms -3',
            ],);
          },
        },),
        it({
          name:
            'REFUSES a digit run past the largest integer a double holds exactly, since `9007199254740993` '
              + 'reads back as 9007199254740992 and the row would carry a count nobody wrote',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({
              log: 'SLICE-COST lane=repair chunk=9007199254740993 sourceChars=10 ms=5 exit=computed',
            },);
            expect(rows,).toHaveLength(0,);
            expect(dropped,).toEqual(['chunk 9007199254740993',],);
          },
        },),
        it({
          name:
            'REFUSES a line that writes one field more than once and names each such field, since the writer '
              + 'writes each once and reading the later value would report a cost no slice had, as two lines run '
              + 'together would',
          fn: async () => {
            expect(readSliceCosts({
              log: 'SLICE-COST lane=repair chunk=4 sourceChars=10 ms=20 ms=45210 exit=computed\n'
                + 'SLICE-COST lane=repair chunk=4 sourceChars=10 ms=20 exit=computed SLICE-COST lane=translate '
                + 'chunk=5 sourceChars=30 ms=40 exit=resumed',
            },),).toEqual({
              rows: [],
              dropped: [
                'ms written more than once',
                'lane written more than once, chunk written more than once, sourceChars written more than once, '
                  + 'ms written more than once, exit written more than once',
              ],
            },);
          },
        },),
        it({
          name: 'reports nothing at all, not even a refusal, for a log that mentions no cost',
          fn: async () => {
            const { rows, dropped, } = readSliceCosts({ log: 'three cats, no costs\nnor here', },);
            expect(rows,).toHaveLength(0,);
            expect(dropped,).toHaveLength(0,);
          },
        },),
      ],
    },),

    describe({
      name: armSliceCost.name,
      children: [
        it({
          name:
            'REPORTS a slice that was left EARLY, which is the case a closing call would miss: slice stages '
              + 'leave their loop body by more than one path',
          fn: async () => {
            const said: string[] = [];

            // Loop that leaves its body early for every slice, as a lane does for a
            // slice it has nothing to do with.
            for (const sliceIndex of [
              0,
              1,
            ]) {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'repair',
                sliceIndex,
                sourceChars: 7,
                signal: LIVE_RUN.signal,
              },);

              continue;
            }

            expect(said,).toHaveLength(4,);
            expect(said.filter(function isStart(line,): boolean {
              return line.startsWith(SLICE_START_MARKER,);
            },),).toHaveLength(2,);
          },
        },),
        it({
          name:
            'writes a line THIS READER can read, which is the only thing keeping the two files agreed '
              + 'about a shape neither one owns',
          fn: async () => {
            const said: string[] = [];
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'translate',
                sliceIndex: 12,
                sourceChars: 843,
                signal: LIVE_RUN.signal,
              },);
            }

            const { rows, dropped, } = readSliceCosts({ log: said.join('\n',), },);
            expect(dropped,).toHaveLength(0,);
            expect(rows,).toHaveLength(1,);
            expect(rows[0]
              ?.lane,).toBe('translate',);
            expect(rows[0]
              ?.sliceIndex,).toBe(12,);
            expect(rows[0]
              ?.sourceChars,).toBe(843,);
            expect(said[0],).toBe('SLICE-START lane=translate chunk=12 sourceChars=843',);
          },
        },),
        it({
          name:
            'TIMES THE SLICE ON A CLOCK THE SYSTEM TIME CANNOT MOVE (ledger B78): with the system clock set back an '
              + 'hour mid-slice, the line still carries the slice\'s own milliseconds, where it carried a negative '
              + 'count this reader drops',
          fn: async ctx => {
            const said: string[] = [];
            {
              const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'translate',
                sliceIndex: 5,
                sourceChars: 321,
                signal: LIVE_RUN.signal,
              },);
              wall.step({ byMs: -HOUR_MS, },);
            }

            const { rows, dropped, } = readSliceCosts({ log: said.join('\n',), },);
            expect(dropped,).toHaveLength(0,);
            expect(rows,).toHaveLength(1,);
            expect(rows[0]
              ?.elapsedMs,).toBeLessThan(HOUR_MS,);
          },
        },),
        it({
          name:
            'REPORTS consolidation before it finishes and keeps an unsettled exit distinct from completed work',
          fn: async () => {
            const said: string[] = [];
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'consolidation',
                sliceIndex: 2,
                sourceChars: 73,
                signal: LIVE_RUN.signal,
              },);
              cost.left({ exit: 'unsettled', },);
            }

            const { rows, dropped, } = readSliceCosts({ log: said.join('\n',), },);
            expect(dropped,).toHaveLength(0,);
            expect(said[0],).toBe('SLICE-START lane=consolidation chunk=2 sourceChars=73',);
            expect(rows[0]
              ?.lane,).toBe('consolidation',);
            expect(rows[0]
              ?.exit,).toBe('unsettled',);
          },
        },),
        it({
          name:
            'reports the ordinary exit for a slice that names none, so the common path needs no call and '
              + 'cannot be forgotten',
          fn: async () => {
            const said: string[] = [];
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'repair',
                sliceIndex: 0,
                sourceChars: 11,
                signal: LIVE_RUN.signal,
              },);
            }

            const { rows, } = readSliceCosts({ log: said.join('\n',), },);
            expect(rows[0]
              ?.exit,).toBe('computed',);
          },
        },),
        it({
          name:
            'REPORTS the exit a lane named rather than the ordinary one, which is what separates a cache '
              + 'hit costing nothing from a slice that bought a full roster',
          fn: async () => {
            const said: string[] = [];
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'translate',
                sliceIndex: 4,
                sourceChars: 96,
                signal: LIVE_RUN.signal,
              },);
              cost.left({ exit: 'resumed', },);
            }

            const { rows, dropped, } = readSliceCosts({ log: said.join('\n',), },);
            expect(dropped,).toHaveLength(0,);
            expect(rows[0]
              ?.exit,).toBe('resumed',);
          },
        },),
        it({
          name:
            'REPORTS a slice cut mid-flight as aborted rather than as ordinary work, which is the case '
              + 'that would otherwise put one near-cap row per aborted entry inside the computed population',
          fn: async () => {
            const said: string[] = [];

            /**
             Run stopped while this slice was in flight, as an entry deadline does.
             */
            const stopped = new AbortController();

            /**
             Thrown out of the slice body, so the measurement leaves scope the way
             a real abort takes it: by exception, naming no exit.
             */
            const cut = new Error('entry deadline',);
            try {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'repair',
                sliceIndex: 9,
                sourceChars: 4_096,
                signal: stopped.signal,
              },);

              stopped.abort(cut,);
              throw cut;
            }
            catch (error) {
              // Expected: this test drives the throwing path deliberately.
              if (error !== cut)
                throw error;
            }

            const { rows, dropped, } = readSliceCosts({ log: said.join('\n',), },);
            expect(dropped,).toHaveLength(0,);
            expect(rows[0]
              ?.exit,).toBe('aborted',);
          },
        },),
        it({
          name: 'REPORTS non-abort consolidation failure distinctly from completed work',
          fn: async () => {
            const said: string[] = [];
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'consolidation',
                sliceIndex: 6,
                sourceChars: 89,
                signal: LIVE_RUN.signal,
              },);
              cost.left({ exit: 'failed', },);
            }

            const { rows, } = readSliceCosts({ log: said.join('\n',), },);
            expect(rows[0]
              ?.exit,).toBe('failed',);
          },
        },),
        it({
          name:
            'REPORTS provisional failure as aborted when caller signal stopped active slice',
          fn: async () => {
            const said: string[] = [];
            const stopped = new AbortController();
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'consolidation',
                sliceIndex: 8,
                sourceChars: 144,
                signal: stopped.signal,
              },);
              cost.left({ exit: 'failed', },);
              stopped.abort(new Error('caller stopped active consolidation',),);
            }

            const { rows, } = readSliceCosts({ log: said.join('\n',), },);
            expect(rows[0]
              ?.exit,).toBe('aborted',);
          },
        },),
        it({
          name:
            'keeps a NAMED exit even once the run is stopped, because a slice that bought nothing bought '
              + 'nothing whether the run was later torn down or not',
          fn: async () => {
            const said: string[] = [];

            /**
             Run stopped after this slice had already answered from cache.
             */
            const stopped = new AbortController();
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'translate',
                sliceIndex: 3,
                sourceChars: 51,
                signal: stopped.signal,
              },);

              cost.left({ exit: 'resumed', },);
              stopped.abort(new Error('entry deadline',),);
            }

            const { rows, } = readSliceCosts({ log: said.join('\n',), },);
            expect(rows[0]
              ?.exit,).toBe('resumed',);
          },
        },),
        it({
          name: 'keeps the LAST exit named, so a path that refines its own answer reports the refined one',
          fn: async () => {
            const said: string[] = [];
            {
              using cost = armSliceCost({
                l: capturingLogger({ messages: said, },),
                lane: 'translate',
                sliceIndex: 5,
                sourceChars: 96,
                signal: LIVE_RUN.signal,
              },);
              cost.left({ exit: 'resumed', },);
              cost.left({ exit: 'unfilled', },);
            }

            const { rows, } = readSliceCosts({ log: said.join('\n',), },);
            expect(rows[0]
              ?.exit,).toBe('unfilled',);
          },
        },),
      ],
    },),
  ],
},);
