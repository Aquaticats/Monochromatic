import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  addTimingRecordOf,
  lastAddTimingRecordOf,
} from './fence-size-read-beside-the-check-2026-11-21-add-timing-record.ts';

/**
 One timing line of an add that landed, built the way the wrapper builds it: the format string of
 `src/native/worktree/add_timing_record.rs:277`, the steps of `spent_json` at `:118`, and a note of `note_json` at `:137`.

 Stands for that line's shape only; no field here is a capture of a real add.

 @returns one line the wrapper could have written
 */
function landedLine(): string {
  return '{"schemaVersion":1,"time":"2026-11-21T00:00:00.000Z","startMillis":1763683200000,"pid":4242,'
    + '"release":"0.6.1-alpha.21","exitCode":0,"totalMs":412.5,'
    + '"steps":{"git ls-tree":{"ms":12.5,"calls":1},"checkSource":{"ms":0.25,"calls":1}},'
    + '"otherMs":399.75,"work":{"fenceSizesBesideTheCheck":{"ms":3.5,"calls":1}},'
    + '"notes":{"route":"clone"},"quiet":{}}';
}

/**
 A copy of the landed line with one field replaced, so one test can break exactly one thing.

 @param fields - the text that replaces the line's steps, its totalMs, or its notes

 @returns a line whose one field is what the test put there
 */
function lineWith({ steps, totalMs, notes, }: { readonly steps?: string; readonly totalMs?: string; readonly notes?: string; }): string {
  /**
   The landed line, as the tests below break it.
   */
  const line = landedLine();
  return line
    .replace('"totalMs":412.5', `"totalMs":${totalMs ?? '412.5'}`,)
    .replace(
    '"steps":{"git ls-tree":{"ms":12.5,"calls":1},"checkSource":{"ms":0.25,"calls":1}}',
    `"steps":${steps ?? '{"git ls-tree":{"ms":12.5,"calls":1},"checkSource":{"ms":0.25,"calls":1}}'}`,
    )
    .replace('"notes":{"route":"clone"}', `"notes":${notes ?? '{"route":"clone"}'}`,);
}

await describe({
  name: '',
  children: [
    describe({
      name: addTimingRecordOf.name,
      children: [
        it({
          name: 'read the total time, the status, the steps, the work and the notes of one add',
          fn: async () => {
            expect(addTimingRecordOf({ line: landedLine(), },),).toEqual({
              totalMs: 412.5,
              exitCode: 0,
              steps: {
                'git ls-tree': {
                  ms: 12.5,
                  calls: 1,
                },
                checkSource: {
                  ms: 0.25,
                  calls: 1,
                },
              },
              work: {
                fenceSizesBesideTheCheck: {
                  ms: 3.5,
                  calls: 1,
                },
              },
              notes: { route: 'clone', },
            },);
          },
        }),
        it({
          name: 'read no steps, work or notes when the line holds none of them',
          fn: async () => {
            expect(addTimingRecordOf({
              line: lineWith({
                steps: '{}',
                notes: '{}',
              },),
            },).steps,).toEqual({},);
            expect(addTimingRecordOf({
              line: lineWith({
                steps: '{}',
                notes: '{}',
              },),
            },).notes,).toEqual({},);
          },
        }),
        ...[
          {
            name: 'a line that is not an object',
            line: '[]',
            expected: 'is not an object',
          },
          {
            name: 'a line with no total time',
            line: lineWith({ totalMs: '"412.5"', },),
            expected: 'holds no number for totalMs',
          },
          {
            name: 'a line whose step holds no time',
            line: lineWith({ steps: '{"git ls-tree":{"ms":"12.5","calls":1}}', },),
            expected: 'holds no number for ms',
          },
          {
            name: 'a line whose note is no word',
            line: lineWith({ notes: '{"route":3}', },),
            expected: 'the add noted no word for route',
          },
        ].map(function refuses({ name, line, expected, },) {
          return it({
            name: `refuse ${name}`,
            fn: async () => {
              /**
               What reading this line threw.
               */
              let caught: unknown;
              try {
                addTimingRecordOf({ line, },);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(Error,);
              expect((caught as Error).message,).toContain(expected,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: lastAddTimingRecordOf.name,
      children: [
        it({
          name: 'read the last add of a file the wrapper appended to, which is the add that just ended',
          fn: async () => {
            expect(lastAddTimingRecordOf({
              text: `${lineWith({ totalMs: '1', },)}\n${landedLine()}\n`,
            },).totalMs,).toBe(412.5,);
          },
        }),
        it({
          name: 'read the last add when the file ends with a blank line',
          fn: async () => {
            expect(lastAddTimingRecordOf({ text: `${landedLine()}\n\n`, },).exitCode,).toBe(0,);
          },
        }),
        it({
          name: 'refuse a file that holds no line',
          fn: async () => {
            /**
             What reading this file threw.
             */
            let caught: unknown;
            try {
              lastAddTimingRecordOf({ text: '\n', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toBe('the timing file holds no line',);
          },
        }),
      ],
    },),
  ],
},);
