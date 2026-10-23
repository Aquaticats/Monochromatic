/**
 Tests for the one reader of every runner's whole command line (ledger B75):
 what each runner declares is read, and everything else on the line is
 refused rather than ignored. Each case reads a real runner's declaration.
 Entry ids are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  COMMAND_LINES,
  readCommandLine,
} from '../../dist/final/node/index.mjs';
import { lineOf, } from './command-line.test-fixture.ts';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

/**
 Usage line every corpus-pass refusal ends with, as its declaration builds it.
 */
const PASS_USAGE = 'Usage: corpus-pass [--only <entry ids>] '
  + '[--require-providers <providers of neuralwatt, synthetic, bedrock, hyper, openrouter>] [--plan]';

await describe({
  name: readCommandLine.name,
  children: [
    it({
      name: 'READS the equals form as the separate form, which once read as no flag and ran every entry',
      fn: async () => {
        expect(lineOf({
          command: 'corpus-pass',
          typed: ['--only=Tabby_01',],
        },).flag('only',),).toEqual({
          kind: 'written',
          flag: '--only',
          value: 'Tabby_01',
        },);
        expect(lineOf({
          command: 'corpus-pass',
          typed: ['--only=Tabby_01=Ginger',],
        },).flag('only',),).toEqual({
          kind: 'written',
          flag: '--only',
          value: 'Tabby_01=Ginger',
        },);
      },
    },),
    it({
      name: 'ANSWERS unwritten for a flag nobody wrote, false for a switch nobody wrote, and the script path',
      fn: async () => {
        /**
         A pass asked for nothing.
         */
        const line = lineOf({
          command: 'corpus-pass',
          typed: [],
        },);
        expect(line.flag('only',),).toEqual({
          kind: 'unwritten',
          flag: '--only',
        },);
        expect(line.switched('plan',),).toBe(false,);
        expect(line.positionals,).toEqual([],);
        expect(line.script,).toBe('dist/final/node/corpus-pass.mjs',);
      },
    },),
    it({
      name: 'REFUSES a mistyped flag and a mistyped switch, each of which once ran every pending corpus entry, '
        + 'naming every problem and ending with the usage line',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsMistypedFlag(): unknown {
            return lineOf({
              command: 'corpus-pass',
              typed: ['--olny', 'Tabby_01',],
            },);
          },
        },),).toBe(
          '--olny is not a flag this command reads; this command takes no argument but its flags, and was given '
            + `"Tabby_01". ${PASS_USAGE}`,
        );
        expect(statedRefusalMessage({
          read: function readsMistypedSwitch(): unknown {
            return lineOf({
              command: 'corpus-pass',
              typed: ['--paln',],
            },);
          },
        },),).toBe(`--paln is not a flag this command reads. ${PASS_USAGE}`,);
      },
    },),
    it({
      name: 'REFUSES a once-only flag or a switch written twice, which kept one of the two in silence',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsTwice(): unknown {
            return lineOf({
              command: 'corpus-pass',
              typed: ['--only', 'Tabby_01', '--plan', '--only', 'Ginger42', '--plan',],
            },);
          },
        },),).toBe(
          `--only is written 2 times, and is read once; --plan is written 2 times, and is read once. ${PASS_USAGE}`,
        );
      },
    },),
    it({
      name: 'REFUSES a valued flag written last, followed by the next flag, followed by an empty argument, or '
        + 'with an empty equals form, each of which once read as unwritten',
      fn: async () => {
        /**
         Command lines whose flag carries nothing usable.
         */
        const valueless: readonly (readonly string[])[] = [
          ['--only',],
          ['--only', '--plan',],
          ['--only', '',],
          ['--only=',],
          ['--only', '--',],
        ];
        expect(valueless.map(function readsValueless(typed,): string {
          return statedRefusalMessage({
            read: function read(): unknown {
              return lineOf({
                command: 'corpus-pass',
                typed,
              },);
            },
          },);
        },),).toEqual(valueless.map(function expectedOf(): string {
          return `--only needs a value written after it. ${PASS_USAGE}`;
        },),);
      },
    },),
    it({
      name: 'REFUSES a switch given a value',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsValuedSwitch(): unknown {
            return lineOf({
              command: 'corpus-pass',
              typed: ['--plan=yes',],
            },);
          },
        },),).toBe(`--plan takes no value, and --plan=yes gives it one. ${PASS_USAGE}`,);
      },
    },),
    it({
      name: 'REFUSES any argument to a runner that reads none, which once ran as if nothing had been typed',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsFlagToBareRunner(): unknown {
            return lineOf({
              command: 'verify-published',
              typed: ['--only', 'Tabby_01',],
            },);
          },
        },),).toBe(
          '--only is not a flag this command reads; this command takes no argument but its flags, and was given '
            + '"Tabby_01". Usage: verify-published',
        );
      },
    },),
    it({
      name: 'REFUSES names that sit on Object.prototype and grouped short flags, the group once and whole',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsPrototypeNames(): unknown {
            return lineOf({
              command: 'verify-published',
              typed: ['--constructor', '--__proto__', '--toString=1', '-abc',],
            },);
          },
        },),).toBe(
          '--constructor is not a flag this command reads; --__proto__ is not a flag this command reads; '
            + '--toString is not a flag this command reads; -abc is not a flag this command reads. '
            + 'Usage: verify-published',
        );
      },
    },),
    it({
      name: 'REFUSES an argument past the last position read, and a required position left out',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function readsTwoLogs(): unknown {
            return lineOf({
              command: 'slice-cost-report',
              typed: ['tabby.log', 'ginger.log',],
            },);
          },
        },),).toBe(
          'this command takes nothing after <log file>, and was given "ginger.log". '
            + 'Usage: slice-cost-report <log file>',
        );
        expect(statedRefusalMessage({
          read: function readsHalfAnAsk(): unknown {
            return lineOf({
              command: 'roster-card',
              typed: ['hyper',],
            },);
          },
        },),).toBe(
          'this command needs <served id>. Usage: roster-card '
            + '<synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi|anthropic> <served id>',
        );
      },
    },),
    it({
      name: 'ENDS A REFUSAL WITH THE OTHER SHAPES A POSITION TAKES: one or more, any number, and one that may be '
        + 'left off (ledger T8, eighteenth batch: no case printed these, though runners declare each)',
      fn: async () => {
        expect([
          statedRefusalMessage({
            read: function readsLogsMistyped(): unknown {
              return lineOf({
                command: 'meter-report',
                typed: ['tabby.log', '--paln',],
              },);
            },
          },),
          statedRefusalMessage({
            read: function readsCensusMistyped(): unknown {
              return lineOf({
                command: 'coverage-census',
                typed: ['--paln',],
              },);
            },
          },),
          statedRefusalMessage({
            read: function readsBenchMistyped(): unknown {
              return lineOf({
                command: 'roster-bench',
                typed: ['--paln',],
              },);
            },
          },),
        ],).toEqual([
          '--paln is not a flag this command reads. Usage: meter-report <log file> [<log file> ...]',
          '--paln is not a flag this command reads. Usage: coverage-census [--baseline <census.json>] '
            + '[--source <src/file.ts> ...] [<unit test file> ...]',
          '--paln is not a flag this command reads. Usage: roster-bench [<slices>]',
        ],);
      },
    },),
    it({
      name: 'READS positions between flags and after the terminator, and a repeatable flag in the order written',
      fn: async () => {
        /**
         A census asked for two test files, one of them spelled like a flag.
         */
        const line = lineOf({
          command: 'coverage-census',
          typed: [
            'src/nap.unit.test.ts',
            '--baseline',
            'census.json',
            '--source',
            'src/nap.ts',
            '--source=src/purr.ts',
            '--',
            '--odd.unit.test.ts',
          ],
        },);
        expect(line.positionals,).toEqual(['src/nap.unit.test.ts', '--odd.unit.test.ts',],);
        expect(line.list('source',),).toEqual(['src/nap.ts', 'src/purr.ts',],);
        expect(line.flag('baseline',),).toEqual({
          kind: 'written',
          flag: '--baseline',
          value: 'census.json',
        },);
      },
    },),
    it({
      name: 'READS a minus sign before digits as a number, after a flag or as a position, grouped or not, and a '
        + 'lone dash as a position',
      fn: async () => {
        expect(lineOf({
          command: 'coverage-probe',
          typed: ['--cap', '-3',],
        },).flag('cap',),).toEqual({
          kind: 'written',
          flag: '--cap',
          value: '-3',
        },);
        expect(lineOf({
          command: 'editor-width-probe',
          typed: ['-35', 'a',],
        },).positionals,).toEqual(['-35', 'a',],);
        expect(lineOf({
          command: 'sentinel-probe',
          typed: ['-',],
        },).positionals,).toEqual(['-',],);
      },
    },),
    it({
      name: 'THROWS on a process argument list without a script path, which no runner is started with',
      fn: async () => {
        expect(function readsWithoutScript(): void {
          readCommandLine({
            command: 'verify-published',
            spec: COMMAND_LINES['verify-published'],
            argv: ['/usr/bin/node',],
          },);
        },).toThrow();
      },
    },),
  ],
},);
