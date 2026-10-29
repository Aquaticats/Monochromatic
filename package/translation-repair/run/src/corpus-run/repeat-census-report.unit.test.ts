/**
 Tests for what the repeat-census command prints and when it declines: the
 whole summary is printed whether or not a positive control held, and a
 control that read false, or could not be read, ends the command with the stated
 refusal's exit code naming it. The pages are invented cat pages whose lines
 stand where the live case's lines stand, so nothing reads the corpus.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  censusEntryOf,
  reportingRefusals,
  reportRepeatCensus,
} from '../../dist/final/node/index.mjs';
import {
  divertingConsoleError,
  divertingConsoleLog,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { captureReport, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';

/**
 Exit code a CLI leaves behind when it declined in its own words.
 */
const REFUSED_AS_STATED = 6;

/**
 Arguments a bare run of the command carries.
 */
const BARE_ARGV: readonly string[] = [
  '/usr/bin/node',
  'dist/final/node/repeat-census.mjs',
];

/**
 Environment holding no key, so the task runner guard lets the command run.
 */
const NO_KEYS: Readonly<NodeJS.ProcessEnv> = {};

/**
 Entry id the census reads as the live case.
 */
const CONTROL_ID = 'TianqiChen666';

/**
 Four clauses of one refrain frame, each with a one-character slot.
 */
const REFRAIN_LINES: readonly string[] = [
  '所以这只猫是个胖猫咪吧。',
  '所以这只猫是个瘦猫咪吧。',
  '所以这只猫是个软猫咪吧。',
  '所以这只猫是个圆猫咪吧。',
];

/**
 Sentence the live case states twice, of twelve characters and a stop.
 */
const SENTENCE = '猫咪在窗台上晒太阳打盹儿。';

/**
 Sentence of another length that no line repeats.
 */
const STRAY_SENTENCE = '尾巴一卷一卷地扫来扫去啊。';

/**
 Keys of the summary a whole report prints, in the order it prints them.
 */
const SUMMARY_KEYS: readonly string[] = [
  'entries',
  'package',
  'maximalGroups',
  'refrainGroups',
  'pagesWithAnyGroup',
  'maximalSizeHistogram',
  'refrainSizeHistogram',
  'refrainCountsPerSetting',
  'settings',
  'bodyOnlyGroups',
  'bodyOnlyPages',
  'bodyOnlyEntryIds',
  'bodyOnlyDetail',
  'sameLineDetail',
  'sameLineGroups',
  'groupsWithListOccurrence',
  'control',
];

/**
 Message the refusal carries after the failed names.
 */
const REFUSAL_TAIL = 'The report was printed in full, and its counts are not to be trusted until every control reads true.';

/**
 What follows a control's name where its reading found no control entry.
 */
const ABSENT_NOTE = '(the TianqiChen666 entry is absent from the census)';

/**
 Builds a page with text on the given lines and nothing on the rest, so each
 paragraph stands alone on its line as the live case's do.

 @param placed - text of each line to fill, by its one-based line number
 @param lineCount - lines the page runs to

 @returns Page text

 @example
 ```ts
 const text = pageWith({ placed: new Map([[1, '猫咪在窗台上。',],]), lineCount: 3, },);
 ```
 */
function pageWith(
  {
    placed,
    lineCount,
  }: {
    readonly placed: ReadonlyMap<number, string>;
    readonly lineCount: number;
  },
): string {
  return `${Array.from({ length: lineCount, }, (_, index,) => placed.get(index + 1,) ?? '',)
    .join('\n',)}\n`;
}

/**
 Builds the live case's page: the refrain's four clauses on the lines the
 controls read, and the repeated sentence on its first line and, at the
 given text, on its second.

 @param refrainLines - which of the lines 17, 25, 33 and 37 state their clause
 @param lastSentence - text of line 55, the sentence's second place

 @returns Page text

 @example
 ```ts
 const text = livePage({ refrainLines: [17, 25, 33, 37,], lastSentence: SENTENCE, },);
 ```
 */
function livePage(
  {
    refrainLines,
    lastSentence,
  }: {
    readonly refrainLines: readonly number[];
    readonly lastSentence: string;
  },
): string {
  return pageWith({
    placed: new Map([
      ...[17, 25, 33, 37,]
        .flatMap((line, index,) => (refrainLines.includes(line,) ? [[line, REFRAIN_LINES[index] ?? '',] as const,] : []),),
      [35, SENTENCE,],
      [55, lastSentence,],
    ],),
    lineCount: 60,
  },);
}

/**
 The live case's page with nothing missing: every control reads true.

 @returns Page text

 @example
 ```ts
 const text = wholeLivePage();
 ```
 */
function wholeLivePage(): string {
  return livePage({
    refrainLines: [17, 25, 33, 37,],
    lastSentence: SENTENCE,
  },);
}

/**
 Builds the message the refusal carries for the given failed names.

 @param names - names of the failed controls, in the order they are printed

 @returns Whole message

 @example
 ```ts
 const says = refusalSays({ names: ['lines.refrainCoversLines17To37',], },);
 ```
 */
function refusalSays({ names, }: { readonly names: readonly string[]; },): string {
  return `the positive controls of the census did not hold: ${names.join('; ',)}. ${REFUSAL_TAIL}`;
}

/**
 Reads the summary the report printed.

 @param lines - lines the report printed with console.log

 @returns Summary, parsed, as an object

 @example
 ```ts
 const summary = printedSummary({ lines: capture.lines, },);
 ```
 */
function printedSummary({ lines, }: { readonly lines: readonly string[]; },): object {
  expect(lines.length,).toBe(1,);

  /**
   The printed text read back.
   */
  const parsed: unknown = JSON.parse(lines.join('\n',),);
  if (((typeof parsed) !== 'object') || (parsed === null))
    throw new Error('unreachable: the census printed a summary that is not an object',);
  return parsed;
}

/**
 Reads the entry count and the two control readings off the summary the report printed.

 @param lines - lines the report printed with console.log

 @returns Entry count, the line reading's control and the package reading's control, as printed

 @example
 ```ts
 const controls = printedControls({ lines: capture.lines, },);
 ```
 */
function printedControls(
  { lines, }: { readonly lines: readonly string[]; },
): { readonly entries: unknown; readonly lines: unknown; readonly package: unknown; } {
  /**
   The summary the report printed.
   */
  const summary = printedSummary({ lines, },);

  /**
   The package reading's part of the summary.
   */
  const packaged: unknown = Reflect.get(
    summary,
    'package',
  );
  if (((typeof packaged) !== 'object') || (packaged === null))
    throw new Error('unreachable: the census printed a package reading that is not an object',);
  return {
    entries: Reflect.get(
      summary,
      'entries',
    ),
    lines: Reflect.get(
      summary,
      'control',
    ),
    package: Reflect.get(
      packaged,
      'control',
    ),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: censusEntryOf.name,
      children: [
        it({
          name: 'Reads a page by both readings, the line census and the package groups, under the entry id',
          fn: async () => {
            /** Entry read from the whole live page. */
            const entry = censusEntryOf({ id: CONTROL_ID, text: wholeLivePage(), },);

            expect(entry.id,).toBe(CONTROL_ID,);
            expect(entry.census.refrain.map((group,) => group.occurrences.map((place,) => place.line,),),).toEqual([
              [17, 25, 33, 37,],
            ],);
            expect(entry.census.maximal.map((group,) => group.occurrences.map((place,) => place.line,),),).toEqual([
              [35, 55,],
            ],);
            expect(entry.groups.map((group,) => [group.kind, group.occurrences.map((place,) => place.line,),],),).toEqual([
              ['refrain', [17, 25, 33, 37,],],
              ['maximal', [35, 55,],],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: reportRepeatCensus.name,
      concurrency: 1,
      children: [
        it({
          name: 'Prints the whole summary and returns when every control of both readings held',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            reportRepeatCensus({
              entries: [
                censusEntryOf({ id: CONTROL_ID, text: wholeLivePage(), },),
                censusEntryOf({ id: 'Mochi', text: `${SENTENCE}\n`, },),
              ],
            },);

            expect(printedControls({ lines: printed.lines, },),).toEqual({
              entries: 2,
              lines: {
                refrainCoversLines17To37: true,
                maximalCoversLines35And55: true,
              },
              package: {
                refrainCoversLines17To37: true,
                maximalCoversLines35And55: true,
              },
            },);
          },
        },),

        it({
          name: 'Refuses naming the refrain control of both readings when the refrain misses a line, after printing the whole summary',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /** What the report throws for a refrain that stands on three of its four lines. */
            const refusal = caught(function reportShortRefrain(): unknown {
              return reportRepeatCensus({
                entries: [
                  censusEntryOf({
                    id: CONTROL_ID,
                    text: livePage({ refrainLines: [17, 25, 33,], lastSentence: SENTENCE, },),
                  },),
                ],
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(`StatedRefusalError: ${refusalSays({
              names: ['lines.refrainCoversLines17To37', 'package.refrainCoversLines17To37',],
            },)}`,);
            /** The summary the report printed. */
            const summary = printedSummary({ lines: printed.lines, },);

            expect(Object.keys(summary,),).toEqual(SUMMARY_KEYS,);
            expect(printedControls({ lines: printed.lines, },),).toEqual({
              entries: 1,
              lines: {
                refrainCoversLines17To37: false,
                maximalCoversLines35And55: true,
              },
              package: {
                refrainCoversLines17To37: false,
                maximalCoversLines35And55: true,
              },
            },);
          },
        },),

        it({
          name: 'Refuses naming the maximal control of both readings when the sentence stands on one of its lines only',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /** What the report throws for a sentence that line 55 does not state. */
            const refusal = caught(function reportLoneSentence(): unknown {
              return reportRepeatCensus({
                entries: [
                  censusEntryOf({
                    id: CONTROL_ID,
                    text: livePage({ refrainLines: [17, 25, 33, 37,], lastSentence: STRAY_SENTENCE, },),
                  },),
                ],
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(`StatedRefusalError: ${refusalSays({
              names: ['lines.maximalCoversLines35And55', 'package.maximalCoversLines35And55',],
            },)}`,);
            expect(printedControls({ lines: printed.lines, },),).toEqual({
              entries: 1,
              lines: {
                refrainCoversLines17To37: true,
                maximalCoversLines35And55: false,
              },
              package: {
                refrainCoversLines17To37: true,
                maximalCoversLines35And55: false,
              },
            },);
          },
        },),

        it({
          name: 'Refuses naming all four controls when the control entry is not among the pages, after printing the summary',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /** What the report throws for pages without the control entry. */
            const refusal = caught(function reportWithoutControlEntry(): unknown {
              return reportRepeatCensus({ entries: [censusEntryOf({ id: 'Mochi', text: wholeLivePage(), },),], },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(`StatedRefusalError: ${refusalSays({
              names: [
                `lines.refrainCoversLines17To37 ${ABSENT_NOTE}`,
                `lines.maximalCoversLines35And55 ${ABSENT_NOTE}`,
                `package.refrainCoversLines17To37 ${ABSENT_NOTE}`,
                `package.maximalCoversLines35And55 ${ABSENT_NOTE}`,
              ],
            },)}`,);
            expect(printedControls({ lines: printed.lines, },),).toEqual({
              entries: 1,
              lines: 'absent',
              package: 'absent',
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'repeat-census exit',
      concurrency: 1,
      children: [
        it({
          name: 'Leaves the exit code unset when every control held, so a failing run is not the harness',
          fn: async (ctx) => {
            /** What the command printed and the exit code it left. */
            const captured = await captureReport({
              sinon: ctx.sinon,
              run: async function runHoldingCensus(): Promise<void> {
                await reportingRefusals({
                  what: 'repeat-census',
                  argv: BARE_ARGV,
                  env: NO_KEYS,
                  run: async function reportHoldingPages(): Promise<void> {
                    reportRepeatCensus({ entries: [censusEntryOf({ id: CONTROL_ID, text: wholeLivePage(), },),], },);
                  },
                },);
              },
            },);

            expect(captured.exitCode,).toBe('unset',);
            expect(captured.lines.length,).toBe(1,);
          },
        },),

        it({
          name: 'Exits at the stated refusal code, printing the whole summary and naming the failed control on stderr',
          fn: async (ctx) => {
            using said = divertingConsoleError({ sinon: ctx.sinon, },);
            /** What the command printed and the exit code it left. */
            const captured = await captureReport({
              sinon: ctx.sinon,
              run: async function runFailingCensus(): Promise<void> {
                await reportingRefusals({
                  what: 'repeat-census',
                  argv: BARE_ARGV,
                  env: NO_KEYS,
                  run: async function reportFailingPages(): Promise<void> {
                    reportRepeatCensus({
                      entries: [
                        censusEntryOf({
                          id: CONTROL_ID,
                          text: livePage({ refrainLines: [17, 25, 33,], lastSentence: SENTENCE, },),
                        },),
                      ],
                    },);
                  },
                },);
              },
            },);

            expect(captured.exitCode,).toBe(REFUSED_AS_STATED,);
            expect(printedControls({ lines: captured.lines, },),).toEqual({
              entries: 1,
              lines: {
                refrainCoversLines17To37: false,
                maximalCoversLines35And55: true,
              },
              package: {
                refrainCoversLines17To37: false,
                maximalCoversLines35And55: true,
              },
            },);
            expect(said.lines,).toEqual([
              `repeat-census: ${refusalSays({
                names: ['lines.refrainCoversLines17To37', 'package.refrainCoversLines17To37',],
              },)}`,
            ],);
          },
        },),
      ],
    },),
  ],
},);
