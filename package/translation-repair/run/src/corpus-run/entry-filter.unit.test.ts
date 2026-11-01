/**
 Tests for the pass entry allowlist.

 The cases that matter are the ones where a misread flag runs the WHOLE
 corpus instead of one entry. That is expensive to discover afterwards and
 looks like an ordinary long pass while it happens, so every shape that could
 parse to nothing throws instead. Each case reads the pass's own command line
 as `reportingRefusals` reads it (ledger B75), so a shape the line reader
 refuses is refused here too.

 Entry ids are invented, since the flag reads any id the same way.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { readOnlyIds, } from '../../dist/final/node/index.mjs';
import { lineOf, } from './command-line.test-fixture.ts';

/**
 Entry ids the pass reads from what a person typed after it.

 @param typed - arguments after the script path

 @returns Ids to run, empty when unrestricted

 @throws StatedRefusalError when the line or the filter is refused

 @example
 ```ts
 const ids = idsOf({ typed: ['--only', 'Tabby_01',], },);
 ```
 */
function idsOf({ typed, }: { readonly typed: readonly string[]; },): ReadonlySet<string> {
  return readOnlyIds({
    line: lineOf({
      command: 'corpus-pass',
      typed,
    },),
  },);
}

await describe({
  name: readOnlyIds.name,
  children: [
    it({
      name: 'returns an EMPTY set when the flag is absent, which is what keeps '
        + 'the ordinary pass untouched: absence and no-restriction are the same '
        + 'value, so a caller cannot forget to handle one of them',
      fn: async () => {
        expect(idsOf({ typed: [], },).size,).toBe(0,);
        expect(idsOf({ typed: ['--plan',], },).size,).toBe(0,);
      },
    },),

    it({
      name: 'reads one id, which is the case this exists for: one entry sat at '
        + 'position 22 of 71 pending entries, about fourteen hours away, for a '
        + 'question one entry answers',
      fn: async () => {
        expect([...idsOf({ typed: ['--only', 'Tabby_01',], },),],).toEqual(['Tabby_01',],);
      },
    },),

    it({
      name: 'splits a comma-separated list and trims each id, so a value pasted '
        + 'with spaces after the commas still names the entries it looks like '
        + 'it names',
      fn: async () => {
        expect([...idsOf({ typed: ['--only', 'Tabby_01, Ginger42 ,Calico',], },),].toSorted(),).toEqual([
          'Calico',
          'Ginger42',
          'Tabby_01',
        ],);
      },
    },),

    it({
      name: 'reads the flag wherever it sits, since mise passes task arguments '
        + 'after its own and the position is not ours to fix',
      fn: async () => {
        expect([...idsOf({ typed: ['--plan', '--only', 'Tabby_01',], },),],).toEqual(['Tabby_01',],);
      },
    },),

    it({
      name: 'READS the equals form as the id it names, which the shell and every parseArgs-based command accept, '
        + 'rather than reading no flag at all and running every entry (ledger B75)',
      fn: async () => {
        expect([...idsOf({ typed: ['--only=Tabby_01',], },),],).toEqual(['Tabby_01',],);
      },
    },),

    it({
      name: 'THROWS on a mistyped flag, which once read as no flag and ran every entry (ledger B75)',
      fn: async () => {
        expect(function readMistypedFlag() {
          idsOf({ typed: ['--olny', 'Tabby_01',], },);
        },).toThrow(StatedRefusalError,);
      },
    },),

    it({
      name: 'THROWS on the flag written twice, which read the first list and dropped the second in silence '
        + '(ledger B75)',
      fn: async () => {
        expect(function readRepeatedFlag() {
          idsOf({ typed: ['--only', 'Tabby_01', '--only', 'Ginger42',], },);
        },).toThrow(StatedRefusalError,);
      },
    },),

    it({
      name: 'THROWS when the flag ends the arguments, rather than reading it as '
        + 'no restriction and running all 92 entries',
      fn: async () => {
        expect(function readTrailingFlag() {
          idsOf({ typed: ['--only',], },);
        },).toThrow(StatedRefusalError,);
      },
    },),

    it({
      name: 'THROWS when the next argument is another flag, which is what a '
        + 'forgotten value looks like: --only --plan would otherwise silently '
        + 'take "--plan" for an entry id and match nothing',
      fn: async () => {
        expect(function readMissingValue() {
          idsOf({ typed: ['--only', '--plan',], },);
        },).toThrow(StatedRefusalError,);
      },
    },),

    it({
      name: 'THROWS when the value holds only separators and whitespace, since '
        + 'an empty allowlist would run the whole corpus, the exact opposite of '
        + 'what was asked',
      fn: async () => {
        expect(function readEmptyList() {
          idsOf({ typed: ['--only', ' , , ',], },);
        },).toThrow(StatedRefusalError,);
      },
    },),
  ],
},);
