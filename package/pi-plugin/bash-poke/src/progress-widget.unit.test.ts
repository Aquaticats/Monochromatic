/**
 Tests for the throttled progress display in the built bash-poke artifact.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createOutputRecorder,
  createProgressView,
  renderProgressLines,
  type ProgressSurface,
  type RunningJob,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Milliseconds a coalesced redraw is given to fire in real time.
 */
const REDRAW_WAIT_MS = 80;

/**
 Builds a stand-in job carrying the output lines a case needs.
 
 @param command - command line shown in the heading
 
 @param startedAt - timestamp elapsed time is measured from
 
 @param lines - output lines the recorder holds
 
 @returns job suitable for progress cases
 
 @example
 ```ts
 fakeJob({ command: 'make', startedAt: 0, lines: ['one'], },);
 ```
 */
function fakeJob(
  {
    command,
    startedAt,
    lines,
  }: {
    readonly command: string;
    readonly startedAt: number;
    readonly lines: readonly string[];
  },
): RunningJob {
  /**
   Recorder holding the case's output lines.
   */
  const recorder = createOutputRecorder({ headChars: 1_000, tailChars: 1_000, }, );
  for (const line of lines)
    recorder.append(`${line}\n`, );
  return {
    id: command,
    command,
    startedAt,
    recorder,
    cancel(): void {},
    finished: Promise.resolve({ cancelled: false, }, ),
  };
}

/**
 Creates a drawing surface that records every call.
 
 @returns surface plus its recorded draws and clears
 
 @example
 ```ts
 const fake = createRecordingSurface();
 ```
 */
function createRecordingSurface(): {
  readonly surface: ProgressSurface;
  readonly draws: (readonly string[])[];
  readonly clears: number[];
} {
  /**
   Line arrays passed to draw, in order.
   */
  const draws: (readonly string[])[] = [];

  /**
   Markers recording each clear call.
   */
  const clears: number[] = [];
  return {
    draws,
    clears,
    surface: {
      draw(lines: readonly string[], ): void {
        draws.push(lines, );
      },
      clear(): void {
        clears.push(1, );
      },
    },
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region renderProgressLines

    describe({
      name: renderProgressLines.name,
      children: [
        it({
          name: 'renders nothing when no job is running',
          fn: async () => {
            expect(renderProgressLines({ jobs: [], tailLines: 3, now: 0, }, ), ).toEqual([]);
          },
        }, ),
        it({
          name: 'heads each job with its command and elapsed time',
          fn: async () => {
            const lines = renderProgressLines({
              jobs: [fakeJob({ command: 'make', startedAt: 0, lines: [], }, ), ],
              tailLines: 3,
              now: 90_000,
            }, );
            expect(lines[0], ).toBe('! make (1m30s)');
          },
        }, ),
        it({
          name: 'indents the most recent output lines only',
          fn: async () => {
            const lines = renderProgressLines({
              jobs: [
                fakeJob({
                  command: 'build',
                  startedAt: 0,
                  lines: ['one', 'two', 'three', 'four', ],
                }, ),
              ],
              tailLines: 2,
              now: 1_000,
            }, );
            expect(lines, ).toEqual(['! build (1s)', '  three', '  four', ]);
          },
        }, ),
        it({
          name: 'renders one block per running job',
          fn: async () => {
            const lines = renderProgressLines({
              jobs: [
                fakeJob({ command: 'a', startedAt: 0, lines: [], }, ),
                fakeJob({ command: 'b', startedAt: 0, lines: [], }, ),
              ],
              tailLines: 1,
              now: 0,
            }, );
            expect(lines, ).toEqual(['! a (0s)', '! b (0s)', ]);
          },
        }, ),
      ],
    }, ),

    //endregion renderProgressLines

    //region createProgressView

    describe({
      name: createProgressView.name,
      concurrency: 1,
      children: [
        it({
          name: 'stays inert when progress display is disabled',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: false,
              tailLines: 3,
              refreshMs: 10,
              tickMs: 0,
              jobs: function noJobs(): readonly RunningJob[] {
                return [];
              },
            }, );
            view.refresh();
            view.clear();
            await wait(REDRAW_WAIT_MS, );
            expect(fake.draws, ).toHaveLength(0);
            expect(fake.clears, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'clears the surface when no job is running',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 3,
              refreshMs: 10,
              tickMs: 0,
              jobs: function noJobs(): readonly RunningJob[] {
                return [];
              },
            }, );
            view.refresh();
            expect(fake.clears, ).toHaveLength(1);
            expect(fake.draws, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'draws the running jobs on refresh',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 1,
              refreshMs: 10,
              tickMs: 0,
              jobs: function oneJob(): readonly RunningJob[] {
                return [fakeJob({ command: 'make', startedAt: 0, lines: ['out', ], }, ), ];
              },
              now: function fixedClock(): number {
                return 1_000;
              },
            }, );
            view.refresh();
            expect(fake.draws, ).toHaveLength(1);
            expect(fake.draws[0]?.[0], ).toBe('! make (1s)');
          },
        }, ),
        it({
          name: 'coalesces refreshes inside the refresh interval',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 1,
              refreshMs: 40,
              tickMs: 0,
              jobs: function oneJob(): readonly RunningJob[] {
                return [fakeJob({ command: 'make', startedAt: 0, lines: [], }, ), ];
              },
            }, );
            view.refresh();
            view.refresh();
            view.refresh();
            expect(fake.draws, ).toHaveLength(1);
            await wait(REDRAW_WAIT_MS, );
            expect(fake.draws, ).toHaveLength(2);
          },
        }, ),
        it({
          name: 'drops a pending redraw when cleared',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 1,
              refreshMs: 40,
              tickMs: 0,
              jobs: function oneJob(): readonly RunningJob[] {
                return [fakeJob({ command: 'make', startedAt: 0, lines: [], }, ), ];
              },
            }, );
            view.refresh();
            view.refresh();
            view.clear();
            await wait(REDRAW_WAIT_MS, );
            expect(fake.draws, ).toHaveLength(1);
            expect(fake.clears, ).toHaveLength(1);
          },
        }, ),
        it({
          name: 'redraws on a tick while a silent job runs',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 1,
              refreshMs: 10,
              tickMs: 25,
              jobs: function oneJob(): readonly RunningJob[] {
                return [fakeJob({ command: 'sleep', startedAt: 0, lines: [], }, ), ];
              },
            }, );
            view.refresh();
            expect(fake.draws, ).toHaveLength(1);
            await wait(REDRAW_WAIT_MS, );
            // A job that prints nothing still has to show its clock advancing,
            // which only a ticker can deliver.
            expect(fake.draws.length > 1, ).toBe(true);
            view.clear();
          },
        }, ),
        it({
          name: 'stops ticking once no job remains',
          fn: async () => {
            const fake = createRecordingSurface();

            /**
             Job list the view reads, emptied mid-case to model a finishing job.
             */
            const tracked: { jobs: RunningJob[]; } = {
              jobs: [fakeJob({ command: 'make', startedAt: 0, lines: [], }, ), ],
            };
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 1,
              refreshMs: 10,
              tickMs: 25,
              jobs: function currentJobs(): readonly RunningJob[] {
                return tracked.jobs;
              },
            }, );
            view.refresh();
            tracked.jobs = [];
            await wait(REDRAW_WAIT_MS, );
            expect(fake.clears.length > 0, ).toBe(true);

            /**
             Draw count after the ticker had its chance to disarm.
             */
            const settled = fake.draws.length;
            await wait(REDRAW_WAIT_MS, );
            expect(fake.draws, ).toHaveLength(settled);
            view.clear();
          },
        }, ),
        it({
          name: 'does not tick when the tick interval is zero',
          fn: async () => {
            const fake = createRecordingSurface();
            const view = createProgressView({
              surface: fake.surface,
              enabled: true,
              tailLines: 1,
              refreshMs: 10,
              tickMs: 0,
              jobs: function oneJob(): readonly RunningJob[] {
                return [fakeJob({ command: 'sleep', startedAt: 0, lines: [], }, ), ];
              },
            }, );
            view.refresh();
            expect(fake.draws, ).toHaveLength(1);
            await wait(REDRAW_WAIT_MS, );
            expect(fake.draws, ).toHaveLength(1);
            view.clear();
          },
        }, ),
      ],
    }, ),

    //endregion createProgressView
  ],
}, );
