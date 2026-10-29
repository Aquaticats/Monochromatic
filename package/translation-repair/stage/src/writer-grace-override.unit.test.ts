/**
 Tests for the window the writer rounds run under.

 THE CASE THAT MATTERS IS PRECEDENCE. The writers have a built-in window of
 their own since 2026-09-06, the dial exists so a launch can move it, and the
 round window wins only when it is the longer one. A writer dial that lost to
 the round dial, a blank one that did not fall back to the built-in, or a
 built-in that dragged the calibration's longer window down, would run the
 writers under a window nobody chose while the launch note claimed otherwise.

 The refusal rule is shared with the round dial through `readWindowDial`, so
 this suite proves the writer dial reaches it and names ITS variable, and
 leaves the rule's own edges to `grace-override.unit.test.ts`.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CALIBRATION_STRAGGLER_GRACE_MS,
  MAX_TIMER_DELAY_MS,
  readWriterGrace,
  resolveWriterGraceMs,
  StatedRefusalError,
  STRAGGLER_GRACE_VAR,
  WRITER_GRACE_MS,
  WRITER_GRACE_VAR,
  WRITER_STAGE_LABELS,
  type WriterGrace,
  writerGraceOverrideNote,
} from '@monochromatic-dev/translation-repair-glossary';
import { STRAGGLER_GRACE_MS, } from '../dist/final/node/index.mjs';

/**
 Window the reader rounds are held to in the launch this dial was made for.
 */
const ROUND = 60_000;

/**
 Window a launch would give its writers instead.
 */
const WRITER = 240_000;

/**
 Round window longer than the built-in writer window, as the editor
 calibration sets it.
 */
const LONG_ROUND = 300_000;

/**
 Clears one dial's variable.

 ONE STATIC DELETE PER DIAL, because deleting a computed key is what
 `no-dynamic-delete` refuses and each dial has exactly one spelling. Written
 here rather than kept in a table of clearers, which nothing calls by name,
 so the scan for process-global writes can follow each delete to the cases
 that call this (ledger B79).

 @param variable - which dial

 @throws {@link Error} when no clearer is known for the variable, since a
 case that thinks it cleared a dial and did not would test the wrong window

 @example
 ```ts
 clearDial({ variable: WRITER_GRACE_VAR, },);
 ```
 */
function clearDial({ variable, }: { readonly variable: string; },): void {
  if (variable === STRAGGLER_GRACE_VAR) {
    delete process.env.TRANSLATION_REPAIR_STRAGGLER_GRACE_MS;
    return;
  }
  if (variable === WRITER_GRACE_VAR) {
    delete process.env.TRANSLATION_REPAIR_WRITER_GRACE_MS;
    return;
  }
  throw new Error(`no clearer for ${variable}`,);
}

/**
 Sets or clears one window variable for one case, restoring it after.

 @param variable - which dial

 @param says - value to set, or nothing to clear the variable

 @returns Disposable that puts the variable back

 @example
 ```ts
 using dial = dialSaying({ variable: WRITER_GRACE_VAR, },);
 ```
 */
function dialSaying(
  {
    variable,
    says,
  }: {
    readonly variable: string;
    readonly says?: string;
  },
): Disposable {
  /**
   Value before the case, restored on dispose.
   */
  const before = process.env[variable];

  if (says === undefined)
    clearDial({ variable, },);
  else
    process.env[variable] = says;

  return {
    [Symbol.dispose]: () => {
      if (before === undefined)
        clearDial({ variable, },);
      else
        process.env[variable] = before;
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveWriterGraceMs.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPELLS the variable the way the documentation does, since an operator who exports a '
            + 'name nothing reads gets the built-in window and no complaint',
          fn: async () => {
            expect(WRITER_GRACE_VAR,).toBe('TRANSLATION_REPAIR_WRITER_GRACE_MS',);
          },
        },),

        it({
          name: 'WAITS LONGER for a writer than for a reader by its built-in, since a writer cut loses a '
            + 'candidate where a reader cut loses a ballot, and a built-in at or under the round window '
            + 'would be the round window with a second name',
          fn: async () => {
            expect(WRITER_GRACE_MS,).toBeGreaterThan(STRAGGLER_GRACE_MS,);
          },
        },),

        it({
          name: 'NAMES exactly the gathers that pass the writer window, in the words their log '
            + 'lines use, written out here separately so the launch note cannot certify itself',
          fn: async () => {
            expect(WRITER_STAGE_LABELS,).toStrictEqual([
              'editor',
              'refiner',
              'translate',
              'produceConsolidations',
              'archive-block-review',
            ],);
          },
        },),

        it({
          name: 'FALLS BACK to the window it was handed when the text is unset or blank, which is every '
            + 'launch that did not ask for a writer window',
          fn: async () => {
            expect(resolveWriterGraceMs({
              fallback: ROUND,
              raw: '',
            },),).toBe(ROUND,);
            expect(resolveWriterGraceMs({
              fallback: ROUND,
              raw: '   ',
            },),).toBe(ROUND,);
          },
        },),

        it({
          name: 'HONORS a positive override, which is what lets one launch hold readers to one window '
            + 'and writers to another',
          fn: async () => {
            expect(resolveWriterGraceMs({
              fallback: ROUND,
              raw: String(WRITER,),
            },),).toBe(WRITER,);
          },
        },),

        it({
          name: 'REFUSES an unreadable value as a stated refusal naming ITS variable, not the round '
            + 'dial\'s, so the operator corrects the one they set',
          fn: async () => {
            /**
             What the reader threw on a value nothing could read.
             */
            const refusal = caught(function readProse(): number {
              return resolveWriterGraceMs({
                fallback: ROUND,
                raw: 'three minutes',
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect((refusal as Error).message,).toContain(WRITER_GRACE_VAR,);
            expect((refusal as Error).message,).not.toContain(STRAGGLER_GRACE_VAR,);
            expect((refusal as Error).message,).toContain('three minutes',);
          },
        },),
      ],
    },),

    describe({
      name: writerGraceOverrideNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS NOTHING when the writers follow the round window, so a run under the '
            + 'calibration\'s longer window carries no note claiming a writer window it did not have',
          fn: async () => {
            expect(writerGraceOverrideNote({
              grace: {
                writerMs: LONG_ROUND,
                roundMs: LONG_ROUND,
                source: 'round-window',
              },
            },),).toBe('',);
          },
        },),

        it({
          name: 'SAYS NOTHING on the source alone, even when the two numbers differ, because a note '
            + 'inferred from their difference could blame a dial nobody set',
          fn: async () => {
            expect(writerGraceOverrideNote({
              grace: {
                writerMs: WRITER,
                roundMs: ROUND,
                source: 'round-window',
              },
            },),).toBe('',);
          },
        },),

        it({
          name: 'NAMES BOTH WINDOWS, THE STAGES AND THE VARIABLE THAT MOVES IT when the built-in put '
            + 'the writers on their window, and says it was built in rather than overridden, so an '
            + 'ordinary run never hides which window its writers ran under (2026-09-06)',
          fn: async () => {
            /**
             Note for a launch that set nothing.
             */
            const note = writerGraceOverrideNote({
              grace: {
                writerMs: WRITER_GRACE_MS,
                roundMs: STRAGGLER_GRACE_MS,
                source: 'built-in',
              },
            },);

            expect(note,).toContain('built in',);
            expect(note,).not.toContain('OVERRIDDEN',);
            expect(note,).toContain(WRITER_GRACE_VAR,);
            expect(note,).toContain(`${String(WRITER_GRACE_MS,)}ms`,);
            expect(note,).toContain(`${String(STRAGGLER_GRACE_MS,)}ms`,);
            for (const label of WRITER_STAGE_LABELS)
              expect(note,).toContain(label,);
          },
        },),

        it({
          name: 'NAMES BOTH WINDOWS, THE VARIABLE AND THE STAGES when the writer dial set the window, '
            + 'so a reader of the log knows which rounds ran long without reading the shell that '
            + 'launched it',
          fn: async () => {
            /**
             Note for a launch that gave its writers the longer window.
             */
            const note = writerGraceOverrideNote({
              grace: {
                writerMs: WRITER,
                roundMs: ROUND,
                source: 'writer-dial',
              },
            },);

            expect(note,).toContain('OVERRIDDEN',);
            expect(note,).toContain(WRITER_GRACE_VAR,);
            expect(note,).toContain(`${String(WRITER,)}ms`,);
            expect(note,).toContain(`${String(ROUND,)}ms`,);
            for (const label of WRITER_STAGE_LABELS)
              expect(note,).toContain(label,);
          },
        },),
      ],
    },),

    describe({
      name: readWriterGrace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RUNS writers under their built-in window when neither dial is set, which is every '
            + 'ordinary run and the configuration every shipped page ran (2026-09-06)',
          fn: async () => {
            expect(readWriterGrace({
              env: {},
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: WRITER_GRACE_MS,
              roundMs: STRAGGLER_GRACE_MS,
              source: 'built-in',
            },);
          },
        },),

        it({
          name: 'HOLDS writers at their built-in when the round dial shortens every other round, which '
            + 'is the 2026-09-02 launch that cut the top editor mid-reply at 60 s',
          fn: async () => {
            expect(readWriterGrace({
              env: { [STRAGGLER_GRACE_VAR]: String(ROUND,), },
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: WRITER_GRACE_MS,
              roundMs: ROUND,
              source: 'built-in',
            },);
          },
        },),

        it({
          name: 'FOLLOWS the round window when it is the longer one, which is the editor calibration\'s '
            + '300000 ms of 2026-08-26, so the built-in never shortens a writer round',
          fn: async () => {
            expect(readWriterGrace({
              env: { [STRAGGLER_GRACE_VAR]: String(LONG_ROUND,), },
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: LONG_ROUND,
              roundMs: LONG_ROUND,
              source: 'round-window',
            },);
          },
        },),

        it({
          name: 'GIVES the calibration\'s writers its 300000 ms round window when nothing is set, by the round '
            + 'fallback it is handed, where the calibration once wrote the round variable for its rounds to read',
          fn: async () => {
            expect(readWriterGrace({
              env: {},
              roundFallback: CALIBRATION_STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: CALIBRATION_STRAGGLER_GRACE_MS,
              roundMs: CALIBRATION_STRAGGLER_GRACE_MS,
              source: 'round-window',
            },);
          },
        },),

        it({
          name: 'TREATS a blank writer dial as unset while the round dial is set, since an '
            + 'exported-but-empty variable is a shell accident rather than an intention',
          fn: async () => {
            expect(readWriterGrace({
              env: {
                [STRAGGLER_GRACE_VAR]: String(ROUND,),
                [WRITER_GRACE_VAR]: '  ',
              },
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: WRITER_GRACE_MS,
              roundMs: ROUND,
              source: 'built-in',
            },);
          },
        },),

        it({
          name: 'PREFERS the writer dial over the round dial when both are set, which is the launch '
            + 'this dial was made for, and says so in the source',
          fn: async () => {
            expect(readWriterGrace({
              env: {
                [STRAGGLER_GRACE_VAR]: String(ROUND,),
                [WRITER_GRACE_VAR]: String(WRITER,),
              },
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: WRITER,
              roundMs: ROUND,
              source: 'writer-dial',
            },);
          },
        },),

        it({
          name: 'LETS the writer dial shorten the writers below their built-in, since a launch that '
            + 'asks for a window gets that window and the note says who asked',
          fn: async () => {
            expect(readWriterGrace({
              env: { [WRITER_GRACE_VAR]: String(ROUND,), },
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: ROUND,
              roundMs: STRAGGLER_GRACE_MS,
              source: 'writer-dial',
            },);
          },
        },),

        it({
          name: 'REFUSES when the writer dial is unreadable, rather than running the writers under a window '
            + 'nobody chose',
          fn: async () => {
            /**
             What the read threw.
             */
            const refusal = caught(function readUnit(): WriterGrace {
              return readWriterGrace({
                env: { [WRITER_GRACE_VAR]: '180s', },
                roundFallback: STRAGGLER_GRACE_MS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${WRITER_GRACE_VAR} must be a whole number of milliseconds from 1 to `
                + `${String(MAX_TIMER_DELAY_MS,)}, and "180s" is not; leave it unset to give writer rounds the `
                + 'built-in writer window, or the round window when that is longer',
            );
          },
        },),

        it({
          name: 'NAMES the round dial first when both are unreadable, since the writer window is '
            + 'measured against the round window and the round dial is read first',
          fn: async () => {
            /**
             What the read threw with both dials wrong.
             */
            const refusal = caught(function readBoth(): WriterGrace {
              return readWriterGrace({
                env: {
                  [STRAGGLER_GRACE_VAR]: 'one minute',
                  [WRITER_GRACE_VAR]: 'three minutes',
                },
                roundFallback: STRAGGLER_GRACE_MS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${STRAGGLER_GRACE_VAR} must be a whole number of milliseconds from 1 to `
                + `${String(MAX_TIMER_DELAY_MS,)}, and "one minute" is not; leave it unset to run under the `
                + 'built-in window',
            );
          },
        },),
      ],
    },),

    describe({
      name: `${readWriterGrace.name} beside the process's own variables`,
      // ONE AT A TIME: every case points the process's own variables elsewhere.
      concurrency: 1,
      children: [
        it({
          name: 'READS BOTH DIALS FROM THE ENVIRONMENT IT IS HANDED, not from the process, whose unreadable '
            + 'values would refuse the read',
          fn: async () => {
            using round = dialSaying({
              variable: STRAGGLER_GRACE_VAR,
              says: 'one minute',
            },);
            using writer = dialSaying({
              variable: WRITER_GRACE_VAR,
              says: 'three minutes',
            },);

            expect(readWriterGrace({
              env: {
                [STRAGGLER_GRACE_VAR]: String(ROUND,),
                [WRITER_GRACE_VAR]: String(WRITER,),
              },
              roundFallback: STRAGGLER_GRACE_MS,
            },),).toStrictEqual({
              writerMs: WRITER,
              roundMs: ROUND,
              source: 'writer-dial',
            },);
          },
        },),
      ],
    },),
  ],
},);
