/**
 Tests the report `reader-measure` prints.

 WHAT THE REPORT OWES THE PERSON WHO RAN IT with a key: the model and route
 measured, how many pictures were matched and how many readings the candidate
 produced, the seated band it was held to, a verdict on each clause, and the
 one edit that follows from a pass (removing the hold from the model's card),
 so the next step is never a question. A candidate that produced too few
 readings is reported as not measured rather than as failing, and the line
 that says what to do next names the input that limited the count: the trusted
 records when every picture was tried, the sample when more pictures were left,
 the candidate's own answers when pictures matched and few readings came back.
 The candidate's own rates are printed beside the band they are held to, and
 the entries the matched pictures came from are counted, so a verdict is
 checked from the printout and a narrow sample is visible.

 Fixtures are cat-themed invention. No provider is reached.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MINIMUM_PRODUCED_READINGS,
  renderMeasureReport,
} from '../../dist/final/node/index.mjs';

/**
 Band every case reports against.
 */
const BAND = {
  floorCorroboratedByAll: 0.8,
  ceilingCorroboratedByNone: 0.1,
  readings: 20,
};

/**
 Rates of a candidate that met both clauses.
 */
const GOOD = {
  produced: MINIMUM_PRODUCED_READINGS,
  corroboratedByAll: 0.9,
  corroboratedByNone: 0,
  meanOverlap: 0.7,
};

/**
 Pictures of two entries, the kind of narrow sample a measurement can rest on.
 */
const TWO_ENTRIES = [
  ...Array.from({ length: 7, }, function mittens(_unused, index,): string {
    return `Mittens/photos/photo${String(index + 1,)}.webp`;
  },),
  'Whiskers/photos/photo1.webp',
  'Whiskers/photos/photo2.webp',
];

/**
 Inputs every case reports with unless it names its own.
 */
const BASE = {
  modelId: 'gemma-4-26b-a4b-it',
  matchedAssets: TWO_ENTRIES,
  sample: 9,
  trustedRecords: 192,
  pictures: 291,
  band: BAND,
  candidate: GOOD,
  verdict: { kind: 'meets-both-clauses', } as const,
};

await describe({
  name: renderMeasureReport.name,
  children: [
    it({
      name: 'SAYS WHAT WAS MEASURED, THE BAND, THE RATES AND THE EDIT A PASS CALLS FOR',
      fn: async () => {
        await Promise.resolve();
        const text = renderMeasureReport(BASE,).join('\n',);
        expect(text,).toContain('gemma-4-26b-a4b-it',);
        expect(text,).toContain('9 pictures matched',);
        expect(text,).toContain('seated floor 0.80',);
        expect(text,).toContain('seated ceiling 0.10',);
        expect(text,).toContain('both clauses met',);
        expect(text,).toContain('openrouter-pictures-unmeasured',);
      },
    },),

    it({
      name: 'NAMES EACH CLAUSE THAT FAILED AND CALLS FOR NO EDIT',
      fn: async () => {
        await Promise.resolve();
        const text = renderMeasureReport({
          ...BASE,
          verdict: { kind: 'fails-clauses', clauses: ['second',], },
        },).join('\n',);
        expect(text,).toContain('second clause failed',);
        expect(text,).not.toContain('first clause failed',);
        expect(text,).toContain('keep the hold',);
      },
    },),

    it({
      name: 'REPORTS TOO FEW PRODUCED READINGS AS NOT MEASURED, naming how many were produced and how many a verdict needs',
      fn: async () => {
        await Promise.resolve();
        const text = renderMeasureReport({
          ...BASE,
          candidate: {
            ...GOOD,
            produced: 2,
          },
          verdict: { kind: 'inconclusive', produced: 2, },
        },).join('\n',);
        expect(text,).toContain('not measured',);
        expect(text,).toContain(`2 of the ${String(MINIMUM_PRODUCED_READINGS,)}`,);
      },
    },),

    it({
      name: 'PRINTS THE CANDIDATE\'S OWN RATES BESIDE THE SEATED BAND, so a verdict is checked from the printout',
      fn: async () => {
        await Promise.resolve();
        const lines = renderMeasureReport(BASE,);
        expect(lines,).toContain(
          'CANDIDATE: 6 readings; corroborated by all 0.90 (floor 0.80), corroborated by none 0.00 (ceiling 0.10).',
        );
        expect(lines,).toContain(
          'OVERLAP: mean trigram overlap of the candidate\'s readings with the trusted readings 0.70.',
        );
      },
    },),

    it({
      name: 'SAYS ONE READING IN THE SINGULAR AND NO RATES FOR NO READINGS, since a share of nothing is not a rate',
      fn: async () => {
        await Promise.resolve();
        const single = renderMeasureReport({
          ...BASE,
          candidate: {
            ...GOOD,
            produced: 1,
          },
          verdict: { kind: 'inconclusive', produced: 1, },
        },);
        expect(single.join('\n',),).toContain('CANDIDATE: 1 reading; corroborated by all 0.90',);
        const none = renderMeasureReport({
          ...BASE,
          candidate: {
            produced: 0,
            corroboratedByAll: 0,
            corroboratedByNone: 0,
            meanOverlap: 0,
          },
          verdict: { kind: 'inconclusive', produced: 0, },
        },).join('\n',);
        expect(none,).toContain('CANDIDATE: no readings were produced, so there are no rates to hold to the band.',);
        expect(none,).not.toContain('corroborated by all 0.00 (floor',);
      },
    },),

    it({
      name: 'COUNTS THE ENTRIES THE MATCHED PICTURES CAME FROM, most pictures first, so a narrow sample is visible',
      fn: async () => {
        await Promise.resolve();
        expect(renderMeasureReport(BASE,),).toContain(
          'ENTRIES: 9 matched pictures came from 2 entries: Mittens 7, Whiskers 2.',
        );
        expect(renderMeasureReport({
          ...BASE,
          matchedAssets: ['Mittens/photos/photo1.webp',],
        },),).toContain('ENTRIES: 1 matched picture came from 1 entry: Mittens 1.',);
        expect(renderMeasureReport({
          ...BASE,
          matchedAssets: ['stray.webp', 'Mittens/a.webp',],
        },),).toContain('ENTRIES: 2 matched pictures came from 2 entries: (no entry directory) 1, Mittens 1.',);
        expect(renderMeasureReport({
          ...BASE,
          matchedAssets: [],
          verdict: { kind: 'inconclusive', produced: 0, },
        },),).toContain('ENTRIES: no picture matched a trusted record.',);
      },
    },),

    it({
      name: 'ASKS FOR MORE TRUSTED RECORDS, not a larger sample, WHEN EVERY PICTURE WAS TRIED AND THE RECORDS MATCHED TOO FEW '
        + '(the runs of 2026-10-21: 3 records matched 3 of 291 pictures at --sample 9)',
      fn: async () => {
        await Promise.resolve();
        const text = renderMeasureReport({
          ...BASE,
          matchedAssets: [
            'Mittens/photos/photo1.webp',
            'Mittens/photos/photo2.webp',
            'Mittens/photos/photo3.webp',
          ],
          trustedRecords: 3,
          candidate: {
            ...GOOD,
            produced: 3,
          },
          verdict: { kind: 'inconclusive', produced: 3, },
        },).join('\n',);
        expect(text,).toContain('RESULT: not measured: 3 of the 6 readings a verdict needs were produced.',);
        expect(text,).toContain('more trusted records are needed',);
        expect(text,).toContain('the 3 trusted records matched 3 of the 291 pictures',);
        expect(text,).toContain('--trusted',);
        expect(text,).toContain('slice-cache/<entry>/picture.<key>.json',);
        expect(text,).toContain('other entries',);
        expect(text,).toContain('--pictures',);
        expect(text,).toContain('keep the hold',);
        expect(text,).not.toContain('larger --sample',);
      },
    },),

    it({
      name: 'ASKS FOR A LARGER SAMPLE WHEN THE SAMPLE ENDED THE COUNT: pictures were left untried',
      fn: async () => {
        await Promise.resolve();
        const text = renderMeasureReport({
          ...BASE,
          matchedAssets: [
            'Mittens/photos/photo1.webp',
            'Mittens/photos/photo2.webp',
            'Mittens/photos/photo3.webp',
          ],
          sample: 3,
          candidate: {
            ...GOOD,
            produced: 3,
          },
          verdict: { kind: 'inconclusive', produced: 3, },
        },).join('\n',);
        expect(text,).toContain('larger --sample',);
        expect(text,).toContain('the sample of 3 ended the count at 3 matched pictures',);
        expect(text,).not.toContain('more trusted records are needed',);
      },
    },),

    it({
      name: 'POINTS AT THE PER-PICTURE REASONS WHEN PICTURES MATCHED AND THE CANDIDATE GAVE FEW READINGS, '
        + 'with a larger sample only where pictures were left',
      fn: async () => {
        await Promise.resolve();
        const answeredLittle = {
          ...BASE,
          candidate: {
            ...GOOD,
            produced: 2,
          },
          verdict: { kind: 'inconclusive', produced: 2, } as const,
        };
        const untried = renderMeasureReport(answeredLittle,).join('\n',);
        expect(untried,).toContain('per-picture reasons',);
        expect(untried,).toContain('larger --sample',);
        expect(untried,).not.toContain('more trusted records are needed',);
        const exhausted = renderMeasureReport({
          ...answeredLittle,
          sample: 12,
        },).join('\n',);
        expect(exhausted,).toContain('per-picture reasons',);
        expect(exhausted,).not.toContain('larger --sample',);
      },
    },),
  ],
},);
