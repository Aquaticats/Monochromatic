/**
 Tests for the rendering audit's command line surface.

 EVERY CASE HERE IS ABOUT A FLAG THE OPERATOR GOT WRONG, because the cases
 where they got it right were never the risk. A reader of a command line has
 exactly one dangerous failure mode: reading a typo as a default and running
 anyway. This one had two of those.

 `--cap once` PARSED TO `NaN`. `capped` in `rendering-audit-settled.ts` returns
 every subject when the cap is negative and `slice(0, cap)` otherwise; `NaN
 < 0` is false and `slice(0, NaN)` is empty, so a mistyped cap audited zero
 subjects, printed the archive population it had read, and exited clean. The
 operator would have read that as an audit of the whole archive.

 `--cap` AT THE END OF THE LINE PARSED TO "buy everything", which is the
 opposite of a cap, because a flag written with nothing after it and a flag
 never written both came back as the empty string. `--only` at the end of the
 line had the same shape and the same opposite meaning: audit every entry.

 ALL THREE NOW REFUSE IN OUR OWN WORDS, at exit code 6 through
 `reportingRefusals`, and the message may repeat the operator's own argument
 because `StatedRefusalError` is exactly the marker for text they typed.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run';
import {
  readAuditArguments,
  readReportArguments,
} from '../../dist/final/node/index.mjs';
import { capNotDigits, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/cap-argument-refusal.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

//region Settled rendering audit argument tests

/**
 Archive a run reads when nobody names one.
 */
const DEFAULT_ARCHIVE = join(
  homedir(),
  'translation-repair-v2-archive',
);

/**
 Clone of the run's corpus pin, which the entry hands in and a run reads
 when nobody names one; invented, so a reader that took any other clone
 shows.
 */
const RUN_CLONE = '/cats/corpus/clone';

/**
 Cap meaning "buy every subject", which is what no `--cap` asks for.
 */
const EVERY_SUBJECT = -1;

/**
 Archive one operator named instead.
 */
const OTHER_ARCHIVE = '/tmp/tabby-archive';

/**
 Clone one operator named instead.
 */
const OTHER_CLONE = '/tmp/tabby-clone';

/**
 Entry one operator asked for.
 */
const ONE_CAT = 'saffron';

/**
 Second entry, so a comma has something on both sides of it.
 */
const ANOTHER_CAT = 'pepperbox';

/**
 Cap one operator asked for.
 */
const SMALL_BUY = 4;

/**
 Cap that reads the archive and asks nobody anything, which is meaningful.
 */
const READ_ONLY_BUY = 0;

/**
 The settled audit's command line, read as `reportingRefusals` reads it.

 @param typed - what the operator wrote after the script path

 @returns Line the audit's body receives

 @throws StatedRefusalError when the line itself is refused (ledger B75)

 @example
 ```ts
 const line = auditLine({ typed: ['--cap', '4',], },);
 ```
 */
function auditLine(
  { typed, }: { readonly typed: readonly string[]; },
): CommandLineOf<'rendering-audit-settled'> {
  return lineOf({
    command: 'rendering-audit-settled',
    typed,
  },);
}

/**
 What the audit's argument reader makes of a command line, with the run's
 clone handed in as the entry hands it.

 @param typed - what the operator wrote after the script path

 @returns Archive, clone, entry filter and cap

 @throws StatedRefusalError when the line or one of its flags is refused

 @example
 ```ts
 const asked = auditArgumentsOf({ typed: ['--cap', '4',], },);
 ```
 */
function auditArgumentsOf(
  { typed, }: { readonly typed: readonly string[]; },
): ReturnType<typeof readAuditArguments> {
  return readAuditArguments({
    line: auditLine({ typed, },),
    runCloneDir: RUN_CLONE,
  },);
}

/**
 The settled audit report's command line, read as `reportingRefusals` reads
 it.

 @param typed - what the operator wrote after the script path

 @returns Line the report's body receives

 @throws StatedRefusalError when the line itself is refused (ledger B75)

 @example
 ```ts
 const line = reportLine({ typed: ['--run', '/tmp/tabby.json',], },);
 ```
 */
function reportLine(
  { typed, }: { readonly typed: readonly string[]; },
): CommandLineOf<'rendering-audit-settled-report'> {
  return lineOf({
    command: 'rendering-audit-settled-report',
    typed,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readAuditArguments.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANSWERS with the defaults for a command line that named nothing, the clone the run\'s pin it is '
            + 'handed names among them',
          fn: async () => {
            expect(auditArgumentsOf({ typed: [], },),).toEqual({
              archiveDir: DEFAULT_ARCHIVE,
              cloneDir: RUN_CLONE,
              onlyIds: [],
              cap: EVERY_SUBJECT,
            },);
          },
        },),
        it({
          name: 'READS every flag the operator did name, in any order',
          fn: async () => {
            expect(auditArgumentsOf({
              typed: [
                '--cap',
                String(SMALL_BUY,),
                '--only',
                `${ONE_CAT},${ANOTHER_CAT}`,
                '--clone',
                OTHER_CLONE,
                '--archive',
                OTHER_ARCHIVE,
              ],
            },),).toEqual({
              archiveDir: OTHER_ARCHIVE,
              cloneDir: OTHER_CLONE,
              onlyIds: [
                ONE_CAT,
                ANOTHER_CAT,
              ],
              cap: SMALL_BUY,
            },);
          },
        },),
        it({
          name: 'KEEPS a cap of zero, which reads the whole archive and buys nothing',
          fn: async () => {
            // Zero is not "no cap": it is the wiring check the audit exists to make
            // cheap. A reader that treated it as absent would spend a roster.
            expect(auditArgumentsOf({
              typed: [
                '--cap',
                String(READ_ONLY_BUY,),
              ],
            },).cap,).toBe(READ_ONLY_BUY,);
          },
        },),
        it({
          name: 'KEEPS a cap of zero written in the equals form, which once read as no cap and bought every subject '
            + 'in the archive (ledger B75)',
          fn: async () => {
            expect(auditArgumentsOf({ typed: [`--cap=${String(READ_ONLY_BUY,)}`,], },).cap,).toBe(READ_ONLY_BUY,);
          },
        },),
        it({
          name: 'REFUSES a mistyped flag rather than auditing as if nothing were typed (ledger B75)',
          fn: async () => {
            expect(function readsMistyped(): void {
              auditArgumentsOf({ typed: ['--cpa', String(READ_ONLY_BUY,),], },);
            },).toThrow(StatedRefusalError,);
          },
        },),
        it({
          name: 'REFUSES a fractional cap, and one written with an exponent, a radix, a sign or past the largest '
            + 'whole number a double holds exactly, rather than auditing a number of subjects nobody typed '
            + '(ledger B73)',
          fn: async () => {
            /**
             Caps that are no whole number written in digits.
             */
            const caps = [
              '4.9',
              '1e1',
              '0x4',
              '+4',
              String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,),
            ];
            expect(caps.map(function refusalOf(cap,): string {
              /**
               What the reader threw.
               */
              const refusal = caught(function readsCap(): void {
                auditArgumentsOf({ typed: ['--cap', cap,], },);
              },);
              expect(refusal,).toBeInstanceOf(StatedRefusalError,);
              return (refusal as Error).message;
            },),).toEqual(caps.map(function expectedOf(cap,): string {
              return capNotDigits({ cap, },);
            },),);
          },
        },),
        it({
          name: 'REFUSES a cap that is not a number, instead of auditing nothing in silence',
          fn: async () => {
            expect(() => {
              auditArgumentsOf({
                typed: [
                  '--cap',
                  'once',
                ],
              },);
            },).toThrow(StatedRefusalError,);
          },
        },),
        it({
          name: 'REPEATS what the operator typed, since the refusal is in our own words',
          fn: async () => {
            /**
             What the reader threw.
             */
            const refusal = caught(function readsWord(): void {
              auditArgumentsOf({
                typed: [
                  '--cap',
                  'once',
                ],
              },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect((refusal as Error).message,).toBe(
              `--cap needs a whole number written in digits, at most ${String(Number.MAX_SAFE_INTEGER,)}, `
                + 'and "once" is not one',
            );
          },
        },),
        it({
          name: 'REFUSES a cap below zero rather than reading it as every subject, since a mistyped '
            + 'sign would audit the whole archive in silence',
          fn: async () => {
            expect(() => {
              auditArgumentsOf({
                typed: [
                  '--cap',
                  '-3',
                ],
              },);
            },).toThrow(StatedRefusalError,);
            expect(() => {
              auditArgumentsOf({
                typed: [
                  '--cap',
                  '-3',
                ],
              },);
            },).toThrow('--cap cannot be below zero, and -3 is; leave it off to audit every subject',);
          },
        },),
        it({
          name: 'REFUSES a flag written at the end of the line with no value after it',
          fn: async () => {
            expect(() => {
              auditArgumentsOf({ typed: ['--cap',], },);
            },).toThrow('--cap needs a value written after it',);
          },
        },),
        it({
          name: 'REFUSES a flag followed by the next flag rather than by its value',
          fn: async () => {
            // `--only` standing where the archive path should be is a typo, and
            // reading it as a path would send the run at a directory named `--only`.
            expect(() => {
              auditArgumentsOf({
                typed: [
                  '--archive',
                  '--only',
                  ONE_CAT,
                ],
              },);
            },).toThrow('--archive needs a value written after it',);
          },
        },),
        it({
          name: 'REFUSES an entry filter that names nobody, which would read as every entry',
          fn: async () => {
            expect(() => {
              auditArgumentsOf({
                typed: [
                  '--only',
                  ',',
                ],
              },);
            },).toThrow(StatedRefusalError,);
          },
        },),
        it({
          name: 'DROPS a stray separator inside a filter that still names someone',
          fn: async () => {
            expect(auditArgumentsOf({
              typed: [
                '--only',
                `${ONE_CAT},,${ANOTHER_CAT}`,
              ],
            },).onlyIds,).toEqual([
              ONE_CAT,
              ANOTHER_CAT,
            ],);
          },
        },),
      ],
    },),

    //endregion Settled rendering audit argument tests

    describe({
      name: readReportArguments.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANSWERS with two empty lists for a command line that named nothing, which is the newest '
            + 'kept run and no across-run band',
          fn: async () => {
            expect(readReportArguments({ line: reportLine({ typed: [], },), },),).toEqual({
              run: [],
              against: [],
            },);
          },
        },),
        it({
          name: 'READS both files the operator named, in any order',
          fn: async () => {
            expect(readReportArguments({
              line: reportLine({
                typed: [
                  '--against',
                  '/tmp/tabby-earlier.json',
                  '--run',
                  '/tmp/tabby-later.json',
                ],
              },),
            },),).toEqual({
              run: ['/tmp/tabby-later.json',],
              against: ['/tmp/tabby-earlier.json',],
            },);
          },
        },),
        it({
          name: 'REFUSES --run written at the end of the line with no value after it, which used to read as '
            + 'absent and silently report the newest kept run',
          fn: async () => {
            expect(() => {
              readReportArguments({ line: reportLine({ typed: ['--run',], },), },);
            },).toThrow('--run needs a value written after it',);
          },
        },),
        it({
          name: 'REFUSES --against followed by the next flag rather than by its value, which used to read as '
            + 'absent and silently print no across-run band',
          fn: async () => {
            /**
             What the reader raised.
             */
            let raised: unknown;
            try {
              readReportArguments({
                line: reportLine({
                  typed: [
                    '--against',
                    '--run',
                    '/tmp/tabby-later.json',
                  ],
                },),
              },);
            }
            catch (error) {
              raised = error;
            }
            expect(raised,).toBeInstanceOf(StatedRefusalError,);
            expect((raised as Error).message,).toContain('--against needs a value written after it',);
          },
        },),
      ],
    },),
  ],
},);
