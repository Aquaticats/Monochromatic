/**
 Tests for the PROBE lines of the sentinel probe: what an entry's line says
 for a repair that ran and for a probe that failed, built from values a case
 writes, with no corpus read and no model asked.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type ProbedResult,
  probeErrorLine,
  probeResultLine,
} from '../../dist/final/node/index.mjs';

/**
 One adjudicated issue as a line reads it.

 @param status - what the panel decided

 @param repairDisposition - what became of the repair

 @param refined - whether the naturalness lane refined it

 @returns Issue record's read parts

 @example
 ```ts
 const record = issueOf({ status: 'accepted', repairDisposition: 'shipped', refined: false, },);
 ```
 */
function issueOf(
  {
    status,
    repairDisposition,
    refined,
  }: {
    readonly status: string;
    readonly repairDisposition: string;
    readonly refined: boolean;
  },
): ProbedResult['issues'][number] {
  return {
    issue: { status, },
    repairDisposition,
    refined,
  };
}

await describe({
  name: 'sentinel-probe-line',
  children: [
    describe({
      name: probeResultLine.name,
      children: [
        it({
          name: 'PRINTS none for the repairs and zero for every count of a run that raised nothing',
          fn: async () => {
            expect(probeResultLine({
              id: 'Mittens1',
              result: {
                status: 'unchanged',
                issues: [],
                findings: [],
              },
              elapsedMs: 7,
            },),).toBe(
              'PROBE Mittens1 status=unchanged issues=0 accepted=0 repairs=none refinedIssues=0 findings=0 ms=7',
            );
          },
        },),
        it({
          name: 'COUNTS the accepted issues per disposition in code point order, one issue alone and several together',
          fn: async () => {
            expect(probeResultLine({
              id: 'Mittens1',
              result: {
                status: 'repaired',
                issues: [
                  issueOf({ status: 'accepted', repairDisposition: 'shipped', refined: false, },),
                  issueOf({ status: 'accepted', repairDisposition: 'no-region', refined: false, },),
                  issueOf({ status: 'accepted', repairDisposition: 'shipped', refined: false, },),
                ],
                findings: [],
              },
              elapsedMs: 120,
            },),).toBe(
              'PROBE Mittens1 status=repaired issues=3 accepted=3 repairs=no-region:1,shipped:2 refinedIssues=0 findings=0 ms=120',
            );
          },
        },),
        it({
          name: 'LEAVES an issue the panel rejected out of the accepted count and the repairs, and counts the refined ones and the findings',
          fn: async () => {
            expect(probeResultLine({
              id: 'Whiskers2',
              result: {
                status: 'repaired',
                issues: [
                  issueOf({ status: 'rejected', repairDisposition: 'withdrawn', refined: true, },),
                  issueOf({ status: 'accepted', repairDisposition: 'shipped', refined: true, },),
                ],
                findings: ['one finding', 'another finding',],
              },
              elapsedMs: 3,
            },),).toBe(
              'PROBE Whiskers2 status=repaired issues=2 accepted=1 repairs=shipped:1 refinedIssues=2 findings=2 ms=3',
            );
          },
        },),
      ],
    },),
    describe({
      name: probeErrorLine.name,
      children: [
        it({
          name: 'PRINTS a marked refusal in its own words',
          fn: async () => {
            expect(probeErrorLine({
              id: 'Mittens1',
              error: new StatedRefusalError({ says: 'the cat left the page', },),
              elapsedMs: 4,
            },),).toBe('PROBE Mittens1 status=ERROR ms=4 error=the cat left the page',);
          },
        },),
        it({
          name: 'PRINTS an unmarked error by its name alone, never its message',
          fn: async () => {
            expect(probeErrorLine({
              id: 'Mittens1',
              error: new RangeError('secret whisker text',),
              elapsedMs: 4,
            },),).toBe('PROBE Mittens1 status=ERROR ms=4 error=refused by RangeError',);
          },
        },),
        it({
          name: 'KEEPS A CORPUS READ\'S KIND AND REMEDY WHOLE after a long entry id, cutting only the path and '
            + 'revision before them at the cap and marking the cut with an ellipsis inside it',
          fn: async () => {
            /**
             Entry id long enough to push the kind past the cap.
             */
            const id = 'tabby'.repeat(40,);

            /**
             Commit the page was asked for at.
             */
            const pinned = 'c'.repeat(40,);
            expect(probeErrorLine({
              id,
              error: new CorpusReadError({
                detail: `${pinned}:people/${id}/page.en.md`,
                cause: { stderr: `fatal: path 'people/${id}/page.en.md' does not exist in '${pinned}'`, },
                commit: 'lacking',
              },),
              elapsedMs: 8,
            },),).toBe(
              `PROBE ${id} status=ERROR ms=8 error=corpus read failed for ${pinned}:people/${'tabby'.repeat(25,)}tab… `
                + '(missing-commit); the clone holds no commit by that revision: fetch it, or pin '
                + 'a commit the clone holds.',
            );
          },
        },),
        it({
          name: 'KEEPS a message of exactly the cap whole and cuts one unit longer to the cap',
          fn: async () => {
            expect(probeErrorLine({
              id: 'Mittens1',
              error: new StatedRefusalError({ says: 'a'.repeat(200,), },),
              elapsedMs: 1,
            },),).toBe(`PROBE Mittens1 status=ERROR ms=1 error=${'a'.repeat(200,)}`,);
            expect(probeErrorLine({
              id: 'Mittens1',
              error: new StatedRefusalError({ says: 'a'.repeat(201,), },),
              elapsedMs: 1,
            },),).toBe(`PROBE Mittens1 status=ERROR ms=1 error=${'a'.repeat(200,)}`,);
          },
        },),
        it({
          name: 'CUTS before a character whose two units would cross the cap, never in its middle',
          fn: async () => {
            expect(probeErrorLine({
              id: 'Mittens1',
              error: new StatedRefusalError({ says: `${'a'.repeat(199,)}\u{1F431}tail`, },),
              elapsedMs: 1,
            },),).toBe(`PROBE Mittens1 status=ERROR ms=1 error=${'a'.repeat(199,)}`,);
          },
        },),
      ],
    },),
  ],
},);
