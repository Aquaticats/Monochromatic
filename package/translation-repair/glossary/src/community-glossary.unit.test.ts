/**
 Tests for the community glossary: the terms an entry carries reach the
 identity context, and a candidate lacking every accepted rendering is named
 on the sheet as evidence, in any casing, with an empty candidate skipped
 (owner, 2026-09-09).

 Fixtures quote the seeded terms as single words. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  COMMUNITY_GLOSSARY,
  communityRenderingDepartures,
  communityRenderingsBlock,
  communityTermLines,
  communityTermsIn,
} from '../dist/final/node/index.mjs';

/**
 Original carrying one seeded term.
 */
const SOURCE = '那只猫讲起自切的经历时，朋友们都安静地听着。';

/**
 Rendering that carries the community's word.
 */
const KEPT = 'When the cat told of her self-surgery, her friends listened quietly.';

/**
 Rendering that lost it, as one archive page shipped.
 */
const LOST = 'When the cat told of her operation, her friends listened quietly.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: communityTermsIn.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEEDS THE TERMS the bench lost (two the archive had right, one the archive misread too, one the owner renders, one insult the bench ungendered, one the bench read as crossdressing, the fandom words the bench misread or the archive glossed away), each with the accepted rendering first',
          fn: async () => {
            // PROPERTIES AND NAMED TERMS, NOT THE WHOLE LIST (ledger H13): pinning
            // every term in order went red on each of three additions and said
            // nothing an addition could break.
            /**
             Seeded terms in order.
             */
            const terms = COMMUNITY_GLOSSARY.map(function termOf(entry,): string {
              return entry.term;
            },);
            expect(new Set(terms,).size,).toBe(terms.length,);
            for (const entry of COMMUNITY_GLOSSARY) {
              expect(entry.renderings.length,).toBeGreaterThan(0,);
              for (const rendering of entry.renderings)
                expect(rendering.trim(),).not.toBe('',);
            }
            /**
             First, accepted rendering of one seeded term.

             @param term - seeded term

             @returns Its first rendering

             @throws {@link Error} when the term is not seeded or has no rendering
             */
            function acceptedFor(term: string,): string {
              /**
               The term's first rendering, if the term is seeded.
               */
              const accepted = COMMUNITY_GLOSSARY.find(function isTerm(entry,): boolean {
                return entry.term === term;
              },)
                ?.renderings[0];
              if (accepted === undefined)
                throw new Error(`${term} is not seeded`,);
              return accepted;
            }
            expect(acceptedFor('自切',),).toBe('self-surgery',);
            expect(acceptedFor('超天酱',),).toBe('KAngel',);
            expect(acceptedFor('炸柜',),).toBe('outed',);
            expect(acceptedFor('逆子',),).toBe('unfilial son',);
            expect(acceptedFor('跨圈',),).toBe('trans community',);
            // The owner's renderings: 药娘 as "trans girl" or "trans woman", and 治愈 leading with "healed".
            expect(acceptedFor('药娘',),).toBe('trans girl',);
            expect(acceptedFor('治愈',),).toBe('healed',);
          },
        },),

        it({
          name: 'FINDS THE TERMS A TEXT CARRIES and nothing for a text carrying none',
          fn: async () => {
            expect(communityTermsIn({ text: SOURCE, },).map(function termOf(entry,): string {
              return entry.term;
            },),).toEqual(['自切',],);
            expect(communityTermsIn({ text: '猫在窗台上睡觉。', },),).toEqual([],);
          },
        },),

        it({
          name: 'RENDERS THE IDENTITY-CONTEXT LINES for the terms present, with the renderings and the why, '
            + 'and no heading for an entry carrying none',
          fn: async () => {
            /** Lines for the seeded source. */
            const lines = communityTermLines({ text: SOURCE, },);
            expect(lines[0],).toContain('COMMUNITY TERMS',);
            expect(lines[1],).toContain('- 自切: "self-surgery" (',);
            expect(lines,).toHaveLength(2,);
            expect(communityTermLines({ text: '猫在窗台上睡觉。', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: communityRenderingDepartures.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE CANDIDATE LACKING EVERY RENDERING where the source carries the term, in any '
            + 'casing, and leaves the one carrying it unnamed',
          fn: async () => {
            /** Departures over one kept and one lost rendering, the kept one in capitals. */
            const departures = communityRenderingDepartures({
              sourceText: SOURCE,
              candidates: [
                {
                  label: 'CANDIDATE 1',
                  text: KEPT.toUpperCase(),
                },
                {
                  label: 'CANDIDATE 2',
                  text: LOST,
                },
              ],
            },);
            expect(departures,).toEqual([
              'CANDIDATE 2 carries none of the community\'s renderings of 自切 ("self-surgery"); the community glossary renders it so',
            ],);
          },
        },),

        it({
          name: 'NAMES A RENDERING THAT UNGENDERS 逆子 (class one hundred fifty-one): a father calling his '
            + 'daughter a son keeps the son, and "child" loses it',
          fn: async () => {
            /** Departures over the kept and the ungendered insult. */
            const departures = communityRenderingDepartures({
              sourceText: '黑猫打翻了父亲的茶杯，被骂作「逆子」。',
              candidates: [
                {
                  label: 'CANDIDATE 1',
                  text: 'The black cat knocked over her father’s teacup and was called an “unfilial son”.',
                },
                {
                  label: 'CANDIDATE 2',
                  text: 'The black cat knocked over her father’s teacup and was called a “rebellious child”.',
                },
              ],
            },);
            expect(departures,).toHaveLength(1,);
            expect(departures[0],).toContain('CANDIDATE 2 carries none of the community\'s renderings of 逆子',);
          },
        },),

        it({
          name: 'NAMES NOTHING where the source carries no term, and skips an empty candidate, which is an '
            + 'absent archive rendering rather than a departure',
          fn: async () => {
            expect(communityRenderingDepartures({
              sourceText: '猫在窗台上睡觉。',
              candidates: [{
                label: 'CANDIDATE 1',
                text: LOST,
              },],
            },),).toEqual([],);
            expect(communityRenderingDepartures({
              sourceText: SOURCE,
              candidates: [{
                label: 'ARCHIVE RENDERING',
                text: '',
              },],
            },),).toEqual([],);
          },
        },),

        it({
          name: 'WRAPS THE DEPARTURES IN A SHEET BLOCK headed as evidence to weigh, with the inflection '
            + 'note, and yields nothing where no candidate departs',
          fn: async () => {
            /** Block over the lost rendering. */
            const block = communityRenderingsBlock({
              sourceText: SOURCE,
              candidates: [{
                label: 'CANDIDATE "translate"',
                text: LOST,
              },],
            },);
            expect(block[0],).toBe('COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',);
            expect(block[1],).toContain('- CANDIDATE "translate" carries none',);
            expect(block.at(-1,),).toBe('',);
            expect(communityRenderingsBlock({
              sourceText: SOURCE,
              candidates: [{
                label: 'CANDIDATE "translate"',
                text: KEPT,
              },],
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
