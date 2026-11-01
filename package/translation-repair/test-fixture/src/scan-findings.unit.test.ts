/**
 Tests the helpers a source scan's package-wide case compares through, so a
 failing scan names what it found rather than the cut the assertion library
 prints (`expected [ …(31) ] to deeply equal []`): each case plants a
 difference of one shape and asserts the whole failure text.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  expectFindingsAsListed,
  expectNoFindings,
  expectRecordAsListed,
} from './scan-findings.test-fixture.ts';

await describe({
  name: 'scan findings',
  children: [
    describe({
      name: expectNoFindings.name,
      children: [
        it({
          name: 'FAILS with every finding whole on a line of its own, one longer than the assertion library shows',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectNoFindings({
                findings: [
                  'cat.ts:4',
                  'kitten.ts:12 a nap taken in the sun on the window sill, longer than forty characters',
                ],
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan found these, one per line:',
              'cat.ts:4',
              'kitten.ts:12 a nap taken in the sun on the window sill, longer than forty characters',
            ].join('\n',),);
          },
        },),
        it({
          name: 'PASSES a scan that found nothing',
          fn: async () => {
            expectNoFindings({ findings: [], },);
          },
        },),
      ],
    },),
    describe({
      name: expectFindingsAsListed.name,
      children: [
        it({
          name: 'FAILS naming each entry found and not listed and each listed and not found, an entry found more '
            + 'often than listed once per extra copy',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectFindingsAsListed({
                findings: [
                  'cat.ts#nap',
                  'cat.ts#purr',
                  'cat.ts#purr',
                ],
                listed: [
                  'cat.ts#purr',
                  'kitten.ts#knead',
                ],
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan\'s findings differ from the listed ones, one difference per line:',
              'found and not listed: cat.ts#nap',
              'found and not listed: cat.ts#purr',
              'listed and not found: kitten.ts#knead',
            ].join('\n',),);
          },
        },),
        it({
          name: 'FAILS with both lists whole in their orders where they hold the same entries in another order',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectFindingsAsListed({
                findings: [
                  'kitten.ts#knead',
                  'cat.ts#nap',
                ],
                listed: [
                  'cat.ts#nap',
                  'kitten.ts#knead',
                ],
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan\'s findings differ from the listed ones, one difference per line:',
              'found in this order: kitten.ts#knead',
              'found in this order: cat.ts#nap',
              'listed in this order: cat.ts#nap',
              'listed in this order: kitten.ts#knead',
            ].join('\n',),);
          },
        },),
        it({
          name: 'FAILS naming a number and a group of texts as their JSON',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectFindingsAsListed({
                findings: [
                  585,
                  612,
                  ['cat.ts#nap', 'kitten.ts#nap',],
                ],
                listed: [
                  585,
                  610,
                  ['cat.ts#nap',],
                ],
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan\'s findings differ from the listed ones, one difference per line:',
              'found and not listed: 612',
              'found and not listed: ["cat.ts#nap","kitten.ts#nap"]',
              'listed and not found: 610',
              'listed and not found: ["cat.ts#nap"]',
            ].join('\n',),);
          },
        },),
        it({
          name: 'FAILS THROUGH THE SCAN\'S OWN COMPARISON, with the assertion library\'s error, where the two lists '
            + 'differ in a way no line names: zero against negative zero, whose JSON is one text',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectFindingsAsListed({
                findings: [0,],
                listed: [-0,],
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(Error.isError(refusal,) ? refusal.name : 'not an error',).toBe('AssertionError',);
          },
        },),
        it({
          name: 'PASSES lists equal entry for entry, numbers and groups among them',
          fn: async () => {
            expectFindingsAsListed({
              findings: [
                'cat.ts#nap',
                610,
                ['cat.ts#nap', 'kitten.ts#nap',],
              ],
              listed: [
                'cat.ts#nap',
                610,
                ['cat.ts#nap', 'kitten.ts#nap',],
              ],
            },);
          },
        },),
      ],
    },),
    describe({
      name: expectRecordAsListed.name,
      children: [
        it({
          name: 'FAILS naming a key found and not listed and a key listed and not found with its value, and a count '
            + 'the two records hold apart with both counts',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectRecordAsListed({
                found: {
                  'cat.ts#purr: Number': 2,
                  'cat.ts#nap: parseInt': 1,
                },
                listed: {
                  'cat.ts#purr: Number': 1,
                  'kitten.ts#knead: Date.parse': 1,
                },
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan\'s record differs from the listed one, one difference per line:',
              '"cat.ts#purr: Number": found 2 and listed 1',
              '"cat.ts#nap: parseInt": found 1 and not listed',
              '"kitten.ts#knead: Date.parse": listed 1 and not found',
            ].join('\n',),);
          },
        },),
        it({
          name: 'FAILS naming under a key each entry its two lists differ by, a flag the records hold apart, and a '
            + 'list held against a count, with both values',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectRecordAsListed({
                found: {
                  unreached: ['cat.ts#hiss', 'cat.ts#nap',],
                  staleSeams: [],
                  readsBuilders: false,
                  naps: ['cat.ts#nap',],
                },
                listed: {
                  unreached: ['cat.ts#nap', 'cat.ts#tail',],
                  staleSeams: [],
                  readsBuilders: true,
                  naps: 1,
                },
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan\'s record differs from the listed one, one difference per line:',
              '"unreached": found and not listed: cat.ts#hiss',
              '"unreached": listed and not found: cat.ts#tail',
              '"readsBuilders": found false and listed true',
              '"naps": found ["cat.ts#nap"] and listed 1',
            ].join('\n',),);
          },
        },),
        it({
          name: 'FAILS with a key\'s two lists whole in their orders where they hold the same entries in another order',
          fn: async () => {
            /** What the helper threw. */
            const refusal = caught(function act(): void {
              expectRecordAsListed({
                found: { reads: ['purr-report.ts', 'nap-census.ts',], },
                listed: { reads: ['nap-census.ts', 'purr-report.ts',], },
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe([
              'Error: the scan\'s record differs from the listed one, one difference per line:',
              '"reads": found in this order: purr-report.ts',
              '"reads": found in this order: nap-census.ts',
              '"reads": listed in this order: nap-census.ts',
              '"reads": listed in this order: purr-report.ts',
            ].join('\n',),);
          },
        },),
        it({
          name: 'PASSES records equal key for key, counts, flags and lists among them',
          fn: async () => {
            expectRecordAsListed({
              found: {
                'cat.ts#purr: Number': 1,
                readsBuilders: true,
                unreached: ['cat.ts#nap',],
              },
              listed: {
                'cat.ts#purr: Number': 1,
                readsBuilders: true,
                unreached: ['cat.ts#nap',],
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
