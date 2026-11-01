import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  compileRegex,
  FLAG_UNSET,
  parseKillSignal,
  parseTypeToken,
  resolveBoolPair,
} from './cli-helpers.ts';

await describe({
  name: 'cli-helpers',
  children: [
    describe({
      name: parseTypeToken.name,
      children: [
        it({
          name: 'returns "file" for "file"',
          fn: async function returnsFile() {
            expect(parseTypeToken('file',),).toBe('file',);
          },
        },),
        it({
          name: 'returns "dir" for "dir"',
          fn: async function returnsDir() {
            expect(parseTypeToken('dir',),).toBe('dir',);
          },
        },),
        it({
          name: 'throws on unknown token',
          fn: async function throwsUnknown() {
            expect(function bogus(): void {
              parseTypeToken('symlink',);
            },)
              .toThrow('Unknown --type token "symlink"',);
          },
        },),
      ],
    },),
    describe({
      name: parseKillSignal.name,
      children: [
        it({
          name: 'returns "SIGTERM" for "SIGTERM"',
          fn: async function returnsSigterm() {
            expect(parseKillSignal('SIGTERM',),).toBe('SIGTERM',);
          },
        },),
        it({
          name: 'returns "SIGHUP" for "SIGHUP"',
          fn: async function returnsSighup() {
            expect(parseKillSignal('SIGHUP',),).toBe('SIGHUP',);
          },
        },),
        it({
          name: 'throws on unknown signal name',
          fn: async function throwsUnknownSig() {
            expect(function bogus(): void {
              parseKillSignal('SIGNUKE',);
            },)
              .toThrow('Unknown --signal "SIGNUKE"',);
          },
        },),
        it({
          name: 'throws on signal we do not allow as a restart trigger',
          fn: async function throwsDangerous() {
            expect(function bogus(): void {
              parseKillSignal('SIGSEGV',);
            },)
              .toThrow('Unknown --signal',);
          },
        },),
      ],
    },),
    describe({
      name: compileRegex.name,
      children: [
        it({
          name: 'compiles a valid pattern',
          fn: async function valid() {
            const re = compileRegex(String.raw`\.ts$`,);
            expect(re.test('foo.ts',),).toBe(true,);
            expect(re.test('foo.tsx',),).toBe(false,);
          },
        },),
        it({
          name: 'throws SyntaxError on an invalid pattern',
          fn: async function invalid() {
            expect(function bogus(): void {
              compileRegex('(',);
            },)
              .toThrow();
          },
        },),
      ],
    },),
    describe({
      name: resolveBoolPair.name,
      children: [
        it({
          name: 'returns FLAG_UNSET when neither flag is passed',
          fn: async function neither() {
            expect(resolveBoolPair({
              positive: false,
              negative: false,
              flag: 'foo',
            },),)
              .toBe(FLAG_UNSET,);
          },
        },),
        it({
          name: 'returns true when only positive is passed',
          fn: async function positiveOnly() {
            expect(resolveBoolPair({
              positive: true,
              negative: false,
              flag: 'foo',
            },),)
              .toBe(true,);
          },
        },),
        it({
          name: 'returns false when only negative is passed',
          fn: async function negativeOnly() {
            expect(resolveBoolPair({
              positive: false,
              negative: true,
              flag: 'foo',
            },),)
              .toBe(false,);
          },
        },),
        it({
          name: 'throws when both forms are passed',
          fn: async function both() {
            expect(function bogus(): void {
              resolveBoolPair({
                positive: true,
                negative: true,
                flag: 'hidden',
              },);
            },)
              .toThrow('Cannot pass both --hidden and --no-hidden',);
          },
        },),
      ],
    },),
  ],
},);
